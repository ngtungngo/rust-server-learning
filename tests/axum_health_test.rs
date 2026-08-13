use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt; // für .collect()
use tower::ServiceExt; // für .oneshot()

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

#[tokio::test]
async fn get_item_returns_id_as_json() {
    let app = router();
    let response = app
        .oneshot(Request::get("/api/item/42").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], br#"{"id":"42"}"#);
}

#[tokio::test]
async fn slow_returns_ok() {
    let app = router();
    let response = app
        .oneshot(Request::get("/api/slow").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"slow ok");
}

#[tokio::test]
async fn create_item_json_returns_501() {
    let app = router();
    let response = app
        .oneshot(
            Request::post("/api/item")
                .header("content-type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
}

#[tokio::test]
async fn create_item_without_json_returns_415() {
    let app = router();
    let response = app
        .oneshot(Request::post("/api/item").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}
