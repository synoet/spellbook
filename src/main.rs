use anyhow::Result;
use axum::http::StatusCode;
use axum::{extract::Query, routing::get, routing::post, Extension, Json, Router};
use qdrant_client::prelude::QdrantClient;
use qdrant_client::qdrant::{
    with_payload_selector::SelectorOptions, PointStruct, SearchPoints, WithPayloadSelector,
};
use serde_json::{json, Value};
use shuttle_secrets;
use shuttle_secrets::SecretStore;
use std::sync::Arc;
mod command;
mod open_ai;
mod utils;

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
    dbg!(url);
    dbg!(token);
    let client = QdrantClient::from_url(url).with_api_key(token).build()?;
    Ok(client)
}

async fn process(
    Extension(q_client): Extension<Arc<QdrantClient>>,
    Json(payload): Json<command::Command>,
) -> (StatusCode, String) {
    for command in payload.commands.into_iter() {
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

    (StatusCode::OK, "Processed".to_string())
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

    let router = Router::new()
        .route("/health", get(health))
        .route("/validate", post(validate))
        .route("/process", post(process))
        .route("/search", get(search))
        .layer(Extension(q_client));

    Ok(router.into())
}
