use rust_server_learning::server::serve_one;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::thread;

#[test]
fn serve_one_responds_with_http() {
    // (1) Port :0 → das OS wählt einen freien Port. Kein Konflikt mit anderen Tests.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap(); // welchen Port haben wir bekommen?

    // (2) Client in eigenem Thread — MUSS parallel laufen, sonst Deadlock:
    //     serve_one blockiert bei accept(), der Client bei connect().
    let client = std::thread::spawn(move || {
        let mut stream = TcpStream::connect(addr).unwrap();
        stream
            .write_all(b"GET /api/health HTTP/1.1\r\n\r\n")
            .unwrap();
        // (3) Wir müssen die Schreibrichtung schließen, sonst wartet read_to_string ewig:
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    });

    // (4) Server bedient GENAU EINE Verbindung, dann kehrt er zurück → Test terminiert.
    serve_one(&listener).unwrap();

    let response = client.join().unwrap();
    assert!(response.contains("200")); // grob: Statuszeile enthält 200
}

#[test]
fn serve_one_returns_400_on_garbage() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let client = std::thread::spawn(move || {
        let mut stream = TcpStream::connect(addr).unwrap();
        stream.write_all(b"not a valid request\r\n\r\n").unwrap(); // Müll
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    });
    serve_one(&listener).unwrap();
    let response = client.join().unwrap();
    assert!(response.contains("400"));
}

#[test]
fn serve_one_serves_only_the_first_connection() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    // serve_one bedient GENAU eine Verbindung, dann kehrt die Closure zurück,
    // der Thread endet und der listener wird gedroppt → Port ist zu.
    thread::spawn(move || {
        let _ = serve_one(&listener);
    });

    // Request 1: wird bedient
    let first = get_health(addr).unwrap_or_else(|e| panic!("first request failed: {e}"));
    assert!(first.contains("200"), "first request should get 200, got: {first}");

    // Request 2: kein 200 mehr — Err (ConnectionRefused) ODER leere Antwort
    let second = get_health(addr);
    let served = second.map(|r| r.contains("200")).unwrap_or(false);
    assert!(!served, "second request should NOT be served");
}

fn get_health(addr: SocketAddr) -> std::io::Result<String> {
    let mut stream = TcpStream::connect(addr)?;
    stream.write_all(b"GET /api/health HTTP/1.1\r\n\r\n")?;
    stream.shutdown(Shutdown::Write)?; // EOF signalisieren
    let mut buf = String::new();
    stream.read_to_string(&mut buf)?; // liest bis der Server schließt
    Ok(buf)
}
