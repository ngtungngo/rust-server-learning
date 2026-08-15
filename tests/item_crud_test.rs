use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt; // für .collect()
use std::time::Duration;
use tower::ServiceExt; // für .oneshot()

use rust_server_learning::app::router;
use rust_server_learning::store::AppState;

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
async fn create_duplicate_name_returns_409() {
    let state = AppState::new();

    // erster POST mit "widget" → 201
    let first = router(Duration::from_secs(30), state.clone())
        .oneshot(
            Request::post("/api/item")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"widget"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::CREATED);

    // zweiter POST mit demselben Namen → 409 Conflict (Eindeutigkeitsregel)
    let second = router(Duration::from_secs(30), state.clone())
        .oneshot(
            Request::post("/api/item")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"widget"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::CONFLICT);

    // der Store hat NICHT doppelt angelegt
    assert_eq!(state.list().len(), 1);
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

#[tokio::test]
async fn list_returns_all_items() {
    let state = AppState::new();
    state.insert("a".to_string()).unwrap(); // direkt über den Store befüllen — kürzer als 2x POST
    state.insert("b".to_string()).unwrap();

    let response = router(Duration::from_secs(30), state)
        .oneshot(Request::get("/api/item").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let items: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(items.len(), 2);
    let names: Vec<&str> = items.iter().map(|i| i["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"a") && names.contains(&"b"));
}

#[tokio::test]
async fn delete_existing_item_returns_204() {
    let state = AppState::new();
    let item = state.insert("widget".to_string()).unwrap(); // direkt befüllen, ID merken

    let response = router(Duration::from_secs(30), state.clone())
        .oneshot(
            Request::delete(format!("/api/item/{}", item.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // wirklich weg? GET → 404 (beweist, dass delete den Store verändert hat)
    assert!(state.get(&item.id).is_none());
}

#[tokio::test]
async fn delete_unknown_item_returns_404() {
    let state = AppState::new();
    let response = router(Duration::from_secs(30), state)
        .oneshot(
            Request::delete("/api/item/nope")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_existing_item_returns_200() {
    let state = AppState::new();
    let item = state.insert("old".to_string()).unwrap();

    let response = router(Duration::from_secs(30), state.clone())
        .oneshot(
            Request::put(format!("/api/item/{}", item.id))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"new"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(state.get(&item.id).unwrap().name, "new"); // Store wirklich geändert?
}

#[tokio::test]
async fn update_unknown_item_returns_404() {
    let state = AppState::new();
    let response = router(Duration::from_secs(30), state)
        .oneshot(
            Request::put("/api/item/nope")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"x"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_empty_name_returns_400() {
    let state = AppState::new();
    let response = router(Duration::from_secs(30), state.clone())
        .oneshot(
            Request::post("/api/item")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name": ""}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(state.list().len(), 0);
}

#[tokio::test]
async fn update_empty_name_returns_400() {
    let state = AppState::new();
    let item = state.insert("old".to_string()).unwrap();

    let response = router(Duration::from_secs(30), state.clone())
        .oneshot(
            Request::put(format!("/api/item/{}", item.id))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":""}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(state.get(&item.id).unwrap().name, "old");
}
