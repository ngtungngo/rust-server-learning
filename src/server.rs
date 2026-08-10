use std::str::FromStr;
use crate::http::{handle, Request, Method, Response, StatusCode};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::task::JoinSet;

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

pub async fn serve_one(listener: &tokio::net::TcpListener) -> std::io::Result<()> {
    let (stream, addr) = listener.accept().await?;   // 1. blockiert bis Verbindung
    tracing::info!(%addr, "connection accepted");   // per-Verbindung-Logging (deine Liste)
    handle_connection(stream).await
}

pub async fn serve(listener: &tokio::net::TcpListener,
                   shutdown: impl std::future::Future<Output = ()>,) -> std::io::Result<()> {
    tokio::pin!(shutdown); // ab hier: shutdown ist an DIESE Stack-Stelle gepinnt, unbeweglich
    let mut tasks = JoinSet::new();
    loop {
        tokio::select! {
            result = listener.accept() => {
                let (stream, _addr) = result?;
                tasks.spawn(async move {
                    if let Err(e) = handle_connection(stream).await {
                        tracing::warn!(error = %e, "connection failed");
                    }
                });
            }
            _ = &mut shutdown => { // dank Pin über viele Runden immer wieder pollbar
                break;
            }
        }
    }
    tasks.join_all().await;
    Ok(())
}

async fn handle_connection(mut stream: tokio::net::TcpStream) -> std::io::Result<()> {
    let peer = stream.peer_addr()?;
    let span = tracing::info_span!("connection", %peer);
    let _guard = span.enter();     // ab hier bis Funktionsende: alle Logs tragen peer

    // 1. lesen
    let mut buffer = [0u8; 1024];
    let idx = stream.read(&mut buffer).await?;
    let raw = String::from_utf8_lossy(&buffer[..idx]);

    // 2. parsen → Request
    let request = parse_request(&raw);
    // 3. handle + zurückschreiben
    let response = match request {
        Some(req) => {
            tracing::info!(method = ?req.method, path = %req.path, "request");
            handle(&req)
        }
        None => Response::text(StatusCode::BadRequest, "bad request"),
    };
    let bytes = to_http(&response);
    stream.write_all(bytes.as_bytes()).await?;
    Ok(())
}