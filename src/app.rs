use axum::Router;
use axum::routing::get;

async fn health() -> &'static str {
    "ok"
}

pub fn router() -> Router {
    Router::new().route("/api/health", get(health))
}