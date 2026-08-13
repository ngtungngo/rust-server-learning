use crate::models::Item;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode, header};
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

async fn create_item(headers: HeaderMap) -> impl IntoResponse {
    let is_json = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("application/json"));

    if is_json {
        (StatusCode::NOT_IMPLEMENTED, "not implemented yet").into_response()
    } else {
        (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "expected application/json",
        )
            .into_response()
    }
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
