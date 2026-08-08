use super::types::{ContentType, Method, Request, Response, StatusCode};

fn handle_item(method: &Method, id: &str) -> Response {
    match method {
        Method::Get => Response::json(StatusCode::OK, format!("{{\"id\":\"{id}\"}}")),
        _             => Response::json(StatusCode::MethodNotAllowed, "{\"error\":\"method not allowed\"}".to_owned())
    }
}

fn handle_create_item(request: &Request) -> Response {
    match request.content_type {
        Some(ContentType::ApplicationJson) => Response::json(StatusCode::NotImplemented, "{\"message\":\"501 Not Implemented\"}".to_owned()),
        _ => Response::text(StatusCode::UnsupportedMediaType, "expected application/json"),
    }
}

pub fn handle(request: &Request) -> Response {
    if let Some(id) = request.path.strip_prefix("/api/item/") {
        return handle_item(&request.method, id);
    }
    match (&request.method, request.path.as_str()) {
        (Method::Post, "/api/item") => handle_create_item(request),
        (Method::Get, "/api/health") => Response::text(StatusCode::OK, "ok"),
        (_, "/api/health")           => Response::text(StatusCode::MethodNotAllowed, "method not allowed"),
        _                            => Response::text(StatusCode::NotFound, "path not found"),
    }
}
