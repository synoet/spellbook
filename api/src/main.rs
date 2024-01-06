use anyhow::Result;
use axum::http::StatusCode;
use axum::{extract::Query, routing::get, routing::post, Extension, Json, Router, routing::get_service};
use qdrant_client::prelude::QdrantClient;
use qdrant_client::qdrant::{
    points_selector::PointsSelectorOneOf, with_payload_selector::SelectorOptions, PointStruct,
    PointsIdsList, PointsSelector, SearchPoints, WithPayloadSelector,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use serde_json::{json, Value};
use shuttle_secrets;
use shuttle_secrets::SecretStore;
use std::sync::Arc;
mod command;
mod github;
mod open_ai;
mod utils;

use command::CompareSubCommands;
use open_ai::{embed_command, initialize_openai};

static COLLECTION_NAME: &str = "commands-v0";

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn validate(Json(payload): Json<Value>) -> (StatusCode, String) {
    match serde_json::from_value::<command::Command>(payload) {
        Ok(_) => (StatusCode::OK, "Valid".to_string()),
        Err(e) => return (StatusCode::BAD_REQUEST, e.to_string()),
    }
}

async fn make_client(url: &str, token: &str) -> Result<QdrantClient> {
    let client = QdrantClient::from_url(url).with_api_key(token).build()?;
    Ok(client)
}

async fn process_webhook(
    Extension(q_client): Extension<Arc<QdrantClient>>,
    Json(payload): Json<github::PushWebhookPayload>,
) -> StatusCode {
    let result = github::process_payload(payload).unwrap();

    let mut commands_to_remove: Vec<command::SubCommand> = result
        .removed
        .iter()
        .flat_map(|file| {
            let command = serde_json::from_str::<command::Command>(&file).unwrap();
            return command.commands;
        })
        .collect();

    let mut commands_to_add: Vec<command::SubCommand> = result
        .added
        .iter()
        .flat_map(|file| {
            let command = serde_json::from_str::<command::Command>(&file).unwrap();
            return command.commands;
        })
        .collect();

    for (curr_file, old_file) in result.modified.iter() {
        let curr_command = serde_json::from_str::<command::Command>(&curr_file).unwrap();
        let old_command = serde_json::from_str::<command::Command>(&old_file).unwrap();

        let (to_add, to_remove) = curr_command.commands.compare(&old_command.commands);

        commands_to_add.extend(to_add);
        commands_to_remove.extend(to_remove);
    }

    let add_count = commands_to_add.len();
    let remove_count = commands_to_remove.len();

    for command in commands_to_remove.into_iter() {
        let id = utils::uuid_hash(&command.command.clone());
        let point_selector_one_of = PointsSelectorOneOf::Points(PointsIdsList {
            ids: vec![id.try_into().unwrap()],
        });
        let points_selector = PointsSelector {
            points_selector_one_of: Some(point_selector_one_of),
        };
        q_client
            .delete_points(COLLECTION_NAME, None, &points_selector, None)
            .await
            .unwrap();
    }

    for command in commands_to_add.into_iter() {
        let id = utils::uuid_hash(&command.command.clone());
        let payload = serde_json::to_value(&command).unwrap().try_into().unwrap();
        if let Ok(embedding) = embed_command(command).await {
            let vec: Vec<f32> = embedding.vec.iter().map(|&x| x as f32).collect();
            let points = vec![PointStruct::new(id, vec, payload)];
            q_client
                .upsert_points(COLLECTION_NAME, None, points, None)
                .await
                .unwrap();
        }
    }

    println!("[INFO] Added {} commands", add_count);
    println!("[INFO] Removed {} commands", remove_count);
    StatusCode::OK
}

#[derive(serde::Deserialize)]
struct SearchQueryParams {
    query: String,
}

async fn search(
    Extension(q_client): Extension<Arc<QdrantClient>>,
    Query(query): Query<SearchQueryParams>,
) -> (StatusCode, Json<Vec<command::SubCommand>>) {
    let embedded_query = open_ai::embed_query(&query.query).await.unwrap();

    let vec: Vec<f32> = embedded_query.vec.iter().map(|&x| x as f32).collect();
    let payload_selector = WithPayloadSelector {
        selector_options: Some(SelectorOptions::Enable(true)),
    };

    let search_points = SearchPoints {
        collection_name: COLLECTION_NAME.to_string(),
        vector: vec,
        limit: 5,
        with_payload: Some(payload_selector),
        ..Default::default()
    };

    let search_result = q_client.search_points(&search_points).await.unwrap();

    let parsed_results = search_result.result.into_iter().map(|result| {
        let payload_str = json!(result.payload).to_string();
        dbg!(&payload_str);
        serde_json::from_str::<command::SubCommand>(&payload_str).unwrap()
    });

    (StatusCode::OK, Json(parsed_results.collect()))
}

#[shuttle_runtime::main]
async fn main(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> shuttle_axum::ShuttleAxum {
    let q_client = make_client(
        &secret_store.get("QDRANT_URL").unwrap(),
        &secret_store.get("QDRANT_TOKEN").unwrap(),
    )
    .await
    .unwrap();
    let q_client = Arc::new(q_client);
    initialize_openai(&secret_store).unwrap();
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any);


    let router = Router::new()
        .nest_service("/", get_service(ServeDir::new("dist")))
        .route("/health", get(health))
        .route("/validate", post(validate))
        .route("/search", get(search))
        .route("/webhook", post(process_webhook))
        .layer(Extension(q_client))
        .layer(cors);

    Ok(router.into())
}
