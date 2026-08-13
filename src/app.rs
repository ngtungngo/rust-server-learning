use crate::models::{CreateItem, Item};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;

async fn health() -> &'static str {
    "ok"
}

async fn get_item(Path(id): Path<String>) -> impl IntoResponse {
    Json(Item { id })
}

async fn create_item(Json(input): Json<CreateItem>) -> impl IntoResponse {
    let item = Item { id: input.name }; // placeholder: real id assignment arrives with the store (L20)
    (StatusCode::CREATED, Json(item))
}

async fn slow() -> impl IntoResponse {
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    "slow ok"
}

pub fn router(timeout: Duration) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/item/{id}", get(get_item))
        .route("/api/item", post(create_item))
        .route("/api/slow", get(slow))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            timeout,
        ))
}
