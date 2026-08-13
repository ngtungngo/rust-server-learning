#[derive(Debug, PartialEq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Connect,
    Patch,
    Trace,
}

#[derive(Debug, PartialEq)]
pub enum StatusCode {
    OK,
    NotFound,
    MethodNotAllowed,
    BadRequest,
    InternalServerError,
    UnsupportedMediaType,
    NotImplemented,
}

impl StatusCode {
    pub fn code(&self) -> u16 {
        match self {
            StatusCode::OK => 200,
            StatusCode::NotFound => 404,
            StatusCode::MethodNotAllowed => 405,
            StatusCode::BadRequest => 400,
            StatusCode::InternalServerError => 500,
            StatusCode::UnsupportedMediaType => 415,
            StatusCode::NotImplemented => 501,
        }
    }
    pub fn reason(&self) -> &str {
        match self {
            StatusCode::OK => "OK",
            StatusCode::NotFound => "Not Found",
            StatusCode::MethodNotAllowed => "Method Not Allowed",
            StatusCode::BadRequest => "Bad Request",
            StatusCode::InternalServerError => "Internal Server Error",
            StatusCode::UnsupportedMediaType => "Unsupported Media Type",
            StatusCode::NotImplemented => "Not Implemented",
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ContentType {
    TextPlain,
    Html,
    ApplicationOctetStream,
    ApplicationJson,
}

#[derive(Debug, PartialEq)]
pub struct Request {
    pub method: Method,
    pub content_type: Option<ContentType>,
    pub path: String,
    pub version: String,
    pub body: Option<Vec<u8>>,
}

impl Request {
    pub fn new(method: Method, path: String) -> Self {
        Self {
            method,
            path,
            version: "HTTP/1.1".to_owned(),
            content_type: None,
            body: None,
        }
    }

    pub fn with_content_type(mut self, ct: ContentType) -> Self {
        self.content_type = Some(ct);
        self
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }
}

#[derive(Debug, PartialEq)]
pub struct Response {
    pub status_code: StatusCode,
    pub content_type: Option<ContentType>,
    pub body: String,
}

impl Response {
    pub fn text(status: StatusCode, body: &str) -> Self {
        Self {
            status_code: status,
            content_type: Some(ContentType::TextPlain),
            body: body.to_owned(),
        }
    }

    pub fn json(status: StatusCode, body: impl Into<String>) -> Self {
        Self {
            status_code: status,
            content_type: Some(ContentType::ApplicationJson),
            body: body.into(),
        }
    }
}

impl ContentType {
    pub fn as_str(&self) -> &str {
        match self {
            ContentType::TextPlain => "text/plain",
            ContentType::ApplicationJson => "application/json",
            ContentType::Html => "text/html",
            ContentType::ApplicationOctetStream => "application/octet-stream",
        }
    }
}

impl std::str::FromStr for Method {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            "PATCH" => Ok(Method::Patch),
            "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete),
            _ => Err(()),
        }
    }
}
