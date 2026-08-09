use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;
use crate::http::{handle, Request, Method, Response, StatusCode};


fn to_http(response: &Response) -> String {
    let content_type = response
        .content_type
        .as_ref()                        // Option<ContentType> → Option<&ContentType> (nicht bewegen)
        .map(|ct| ct.as_str())           // Option<&ContentType> → Option<&str>  (jedes Some umwandeln)
        .unwrap_or("application/octet-stream");  // Option<&str> → &str  (Default bei None)

    format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        response.status_code.code(),
        response.status_code.reason(),
        content_type,
        response.body.len(),
        response.body,
    )
}


fn parse_request(raw: &str) -> Option<Request> {
    let first_line = raw.lines().next()?;        // None → früh raus
    let mut parts = first_line.split(' ');
    let method = Method::from_str(parts.next()?).ok()?; // zwei ? : Teil da? + Methode gültig?
    let path = parts.next()?;                      // None → früh raus
    Some(Request::new(method, path.to_owned()))
}

pub fn serve_one(listener: &TcpListener) -> std::io::Result<()> {
    let (stream, addr) = listener.accept()?;   // 1. blockiert bis Verbindung
    tracing::info!(%addr, "connection accepted");   // per-Verbindung-Logging (deine Liste)
    handle_connection(stream)
}

pub fn serve(listener: &TcpListener) -> std::io::Result<()> {
    for stream in listener.incoming() {
        let stream = stream?;              // Result<TcpStream> → TcpStream (oder früh raus)
        std::thread::spawn(move || {
            if let Err(e) = handle_connection(stream) {
                tracing::warn!(error = %e, "connection failed");
            }
        });
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    tracing::info!("start handle connection");   // per-Verbindung-Logging (deine Liste)
    // 1. lesen
    let mut buffer = [0u8; 1024];
    let idx = stream.read(&mut buffer)?;
    let raw = String::from_utf8_lossy(&buffer[..idx]);

    // 2. parsen → Request
    let request = parse_request(&raw);
    tracing::info!("start handle request: {:?}", request);
    // 3. handle + zurückschreiben
    let response = match request {
        Some(req) => handle(&req),
        None => Response::text(StatusCode::BadRequest, "bad request"),
    };
    let bytes = to_http(&response);
    stream.write_all(bytes.as_bytes())?;
    tracing::info!("end handle request");
    Ok(())
}