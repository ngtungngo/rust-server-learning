use crate::models::CreateItem;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;

use crate::store::AppState;

async fn health() -> &'static str {
    "ok"
}

async fn get_item(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.get(&id) {
        Some(item) => (StatusCode::OK, Json(item)).into_response(),
        None => (StatusCode::NOT_FOUND, "item not found").into_response(),
    }
}

async fn create_item(
    State(state): State<AppState>,
    Json(input): Json<CreateItem>,
) -> impl IntoResponse {
    let item = state.insert(input.name);
    (StatusCode::CREATED, Json(item))
}

async fn slow() -> impl IntoResponse {
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    "slow ok"
}

pub fn router(timeout: Duration, state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/item/{id}", get(get_item))
        .route("/api/item", post(create_item))
        .route("/api/slow", get(slow))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            timeout,
        ))
        .with_state(state)
}
