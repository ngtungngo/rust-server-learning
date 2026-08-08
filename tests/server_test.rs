use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use rust_server_learning::server::serve_one;

#[test]
fn serve_one_responds_with_http() {
    // (1) Port :0 → das OS wählt einen freien Port. Kein Konflikt mit anderen Tests.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();  // welchen Port haben wir bekommen?

    // (2) Client in eigenem Thread — MUSS parallel laufen, sonst Deadlock:
    //     serve_one blockiert bei accept(), der Client bei connect().
    let client = std::thread::spawn(move || {
        let mut stream = TcpStream::connect(addr).unwrap();
        stream.write_all(b"GET /api/health HTTP/1.1\r\n\r\n").unwrap();
        // (3) Wir müssen die Schreibrichtung schließen, sonst wartet read_to_string ewig:
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    });

    // (4) Server bedient GENAU EINE Verbindung, dann kehrt er zurück → Test terminiert.
    serve_one(&listener).unwrap();

    let response = client.join().unwrap();
    assert!(response.contains("200"));  // grob: Statuszeile enthält 200
}

#[test]
fn serve_one_returns_400_on_garbage() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let client = std::thread::spawn(move || {
        let mut stream = TcpStream::connect(addr).unwrap();
        stream.write_all(b"not a valid request\r\n\r\n").unwrap();   // Müll
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    });
    serve_one(&listener).unwrap();
    let response = client.join().unwrap();
    assert!(response.contains("400"));
}