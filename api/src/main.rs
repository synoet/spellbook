use anyhow::Result;
use axum::http::StatusCode;
use axum::{
    extract::Query,
    response::{IntoResponse, Response},
    routing::get,
    routing::get_service,
    routing::post,
    Extension, Json, Router,
};
use dotenv::dotenv;
use serde_json::Value;
use std::env;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
mod command;
mod github;
mod open_ai;
mod utils;
mod vector_db;

use command::CompareSubCommands;
use open_ai::{embed_command, initialize_openai};
use vector_db::VectorClient;

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn validate(Json(payload): Json<Value>) -> (StatusCode, String) {
    match serde_json::from_value::<command::Command>(payload) {
        Ok(_) => (StatusCode::OK, "Valid".to_string()),
        Err(e) => return (StatusCode::BAD_REQUEST, e.to_string()),
    }
}

async fn process_webhook(
    Extension(vector_client): Extension<Arc<VectorClient>>,
    Json(payload): Json<github::PushWebhookPayload>,
) -> Result<StatusCode, AppError> {
    let result = github::process_payload(payload)?;

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

    for command in commands_to_remove.into_iter() {
        let id = utils::uuid_hash(&command.command.clone());
        vector_client.delete(&id).await?;
    }

    for command in commands_to_add.into_iter() {
        let id = utils::uuid_hash(&command.command.clone());
        let payload = serde_json::to_value(&command).unwrap().try_into().unwrap();
        if let Ok(embedding) = embed_command(command).await {
            vector_client.insert(&id, embedding, payload).await?;
        }
    }

    Ok(StatusCode::OK)
}

#[derive(serde::Deserialize)]
struct SearchQueryParams {
    query: String,
}

async fn search(
    Extension(vector_client): Extension<Arc<VectorClient>>,
    Query(query): Query<SearchQueryParams>,
) -> Result<(StatusCode, Json<Vec<command::SubCommand>>), AppError> {
    let embedded_query = open_ai::embed_query(&query.query).await.unwrap();

    let search_result = vector_client.search(embedded_query, 5).await?;

    let sub_commands: Vec<command::SubCommand> = search_result
        .result
        .into_iter()
        .filter_map(|result| match result.try_into() {
            Ok(sub_command) => Some(sub_command),
            Err(_) => None,
        })
        .collect();

    Ok((StatusCode::OK, Json(sub_commands)))
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let vector_client = VectorClient::new(
        &env::var("QDRANT_URL").unwrap(),
        &env::var("QDRANT_TOKEN").unwrap(),
    )
    .await
    .unwrap();
    let vector_client = Arc::new(vector_client);
    initialize_openai(env::var("OPENAI_TOKEN").unwrap()).unwrap();
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any);

    let router = Router::new()
        .nest_service("/", get_service(ServeDir::new("dist")))
        .route("/health", get(health))
        .route("/validate", post(validate))
        .route("/search", get(search))
        .route("/webhook", post(process_webhook))
        .layer(Extension(vector_client))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, router).await.unwrap();

    Ok(())
}
