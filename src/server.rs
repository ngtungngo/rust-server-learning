use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
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

pub fn serve(listener: &TcpListener, flag: Arc<AtomicBool>) -> std::io::Result<()> {
    listener.set_nonblocking(true)?;
    let mut handles = Vec::new();
    while !flag.load(Ordering::SeqCst) {          // Flag gesetzt? → Loop verlassen
        match listener.accept() {
            Ok((stream, _addr)) => {
                let handle = std::thread::spawn(move || {
                    if let Err(e) = handle_connection(stream) {
                        tracing::warn!(error = %e, "connection failed");
                    }
                });
            handles.push(handle);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(50));   // nichts da → kurz schlafen
            }
            Err(e) => return Err(e),                              // echter Fehler → raus
        }
    }
    for handle in handles {
        let _ = handle.join();                    // in-flight-Anfragen zu Ende bedienen
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let peer = stream.peer_addr()?;
    let span = tracing::info_span!("connection", %peer);
    let _guard = span.enter();     // ab hier bis Funktionsende: alle Logs tragen peer

    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    // 1. lesen
    let mut buffer = [0u8; 1024];
    let idx = stream.read(&mut buffer)?;
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
    stream.write_all(bytes.as_bytes())?;
    Ok(())
}