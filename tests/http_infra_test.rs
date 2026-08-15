use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt; // für .collect()
use std::time::Duration;
use tower::ServiceExt; // für .oneshot()

use rust_server_learning::app::router;
use rust_server_learning::store::AppState;

#[tokio::test]
async fn health_returns_200_ok() {
    let app = router(Duration::from_secs(30), AppState::new());

    let response = app
        .oneshot(Request::get("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"ok");
}

#[tokio::test]
async fn slow_returns_ok() {
    let app = router(Duration::from_secs(30), AppState::new());
    let response = app
        .oneshot(Request::get("/api/slow").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"slow ok");
}

#[tokio::test]
async fn create_item_without_json_returns_415() {
    let app = router(Duration::from_secs(30), AppState::new());
    let response = app
        .oneshot(Request::post("/api/item").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn slow_route_times_out_with_408() {
    let app = router(Duration::from_millis(50), AppState::new());

    let response = app
        .oneshot(Request::get("/api/slow").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT); // 408
}
