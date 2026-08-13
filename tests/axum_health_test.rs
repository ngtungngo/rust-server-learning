use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt; // für .collect()
use std::time::Duration;
use tower::ServiceExt; // für .oneshot()

// die Router-Fabrik, die wir gleich in src/ bauen
use rust_server_learning::app::router;
use rust_server_learning::store::AppState;

#[tokio::test]
async fn health_returns_200_ok() {
    let app = router(std::time::Duration::from_secs(30), AppState::new());

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
    let app = router(std::time::Duration::from_secs(30), AppState::new());
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
    let app = router(std::time::Duration::from_secs(30), AppState::new());
    let response = app
        .oneshot(Request::post("/api/item").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn slow_route_times_out_with_408() {
    let app = router(std::time::Duration::from_millis(50), AppState::new());

    let response = app
        .oneshot(Request::get("/api/slow").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT); // 408
}

#[tokio::test]
async fn create_then_get_roundtrip() {
    let state = AppState::new();

    // POST → 201, ID aus der Antwort holen
    let created = router(Duration::from_secs(30), state.clone())
        .oneshot(
            Request::post("/api/item")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"widget"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(created.status(), StatusCode::CREATED);
    let body = created.into_body().collect().await.unwrap().to_bytes();
    let item: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let id = item["id"].as_str().unwrap();
    assert_eq!(item["name"], "widget");

    // GET mit dieser ID → 200, gleiches Item (SELBER state!)
    let got = router(Duration::from_secs(30), state.clone())
        .oneshot(
            Request::get(format!("/api/item/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(got.status(), StatusCode::OK);
}

#[tokio::test]
async fn get_unknown_item_returns_404() {
    let state = AppState::new();
    let response = router(Duration::from_secs(30), state)
        .oneshot(
            Request::get("/api/item/does-not-exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
