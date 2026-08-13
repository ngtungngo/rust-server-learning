use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;   // für .collect()
use tower::ServiceExt;         // für .oneshot()

// die Router-Fabrik, die wir gleich in src/ bauen
use rust_server_learning::app::router;

#[tokio::test]
async fn health_returns_200_ok() {
    let app = router();

    let response = app
        .oneshot(Request::get("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"ok");
}
