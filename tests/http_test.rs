use rust_server_learning::http::{ContentType, Method, Request, StatusCode, handle};
use std::time::{Duration, Instant};

#[test]
fn health_endpoint_returns_ok() {
    let request = Request::new(Method::Get, "/api/health".into());
    assert_eq!(handle(&request).status_code, StatusCode::OK);
}

#[test]
fn unknown_path_returns_not_found() {
    let request = Request::new(Method::Get, "/nope".into());
    assert_eq!(handle(&request).status_code, StatusCode::NotFound);
}

#[test]
fn wrong_method_on_known_path_returns_405() {
    let request = Request::new(Method::Post, "/api/health".into());
    assert_eq!(handle(&request).status_code, StatusCode::MethodNotAllowed);
}

#[test]
fn get_item_returns_json_with_id() {
    let request = Request::new(Method::Get, "/api/item/42".into());
    let response = handle(&request);
    assert_eq!(response.status_code, StatusCode::OK);
    assert_eq!(response.content_type, Some(ContentType::ApplicationJson));
    assert_eq!(response.body, "{\"id\":\"42\"}");
}

#[test]
fn post_item_with_wrong_content_type_returns_415() {
    let request =
        Request::new(Method::Post, "/api/item".into()).with_content_type(ContentType::TextPlain);
    assert_eq!(
        handle(&request).status_code,
        StatusCode::UnsupportedMediaType
    );
}

#[test]
fn post_item_with_content_type() {
    let request = Request::new(Method::Post, "/api/item".into())
        .with_content_type(ContentType::ApplicationJson);
    assert_eq!(handle(&request).status_code, StatusCode::NotImplemented);
}

#[test]
fn slow_endpoint_returns_ok_after_delay() {
    let request = Request::new(Method::Get, "/api/slow".to_owned());
    let start = Instant::now();
    let response = handle(&request);
    assert!(start.elapsed() >= Duration::from_millis(400));
    assert_eq!(response.status_code, StatusCode::OK);
}
