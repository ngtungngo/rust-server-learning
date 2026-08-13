use rust_server_learning::server::{serve, serve_one};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
mod common; // bindet tests/common/mod.rs ein
use common::get_health;

#[tokio::test]
async fn serve_one_responds_with_http() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Client im std::thread (echter OS-Thread, außerhalb der Runtime) — muss parallel
    // laufen, sonst Deadlock: serve_one awaitet accept(), der Client connectet.
    let client = std::thread::spawn(move || {
        let mut stream = TcpStream::connect(addr).unwrap();
        stream
            .write_all(b"GET /api/health HTTP/1.1\r\n\r\n")
            .unwrap();
        stream.shutdown(Shutdown::Write).unwrap(); // EOF signalisieren
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    });

    // serve_one bedient GENAU eine Verbindung im Testkörper (awaitet, blockiert nicht).
    serve_one(&listener, Duration::from_millis(100))
        .await
        .unwrap();

    let response = client.join().unwrap();
    assert!(response.contains("200"));
}

#[tokio::test]
async fn serve_one_returns_400_on_garbage() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let client = std::thread::spawn(move || {
        let mut stream = TcpStream::connect(addr).unwrap();
        stream.write_all(b"not a valid request\r\n\r\n").unwrap(); // Müll
        stream.shutdown(Shutdown::Write).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    });

    serve_one(&listener, Duration::from_millis(100))
        .await
        .unwrap();

    let response = client.join().unwrap();
    assert!(response.contains("400"));
}

// serve wird gespawnt UND der Client blockiert im Testkörper → multi_thread nötig,
// sonst hält der blockierende Client den einzigen Runtime-Thread und der serve-Task
// wird nie gepollt (Deadlock).
#[tokio::test(flavor = "multi_thread")]
async fn serve_one_serves_only_the_first_connection() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // serve_one (NICHT serve): bedient eine Verbindung, dann endet der Task und der
    // listener wird gedroppt → Port ist zu.
    tokio::spawn(async move {
        let _ = serve_one(&listener, Duration::from_millis(100)).await;
    });

    // Request 1: wird bedient
    let first = get_health(addr).unwrap_or_else(|e| panic!("first request failed: {e}"));
    assert!(
        first.contains("200"),
        "first request should get 200, got: {first}"
    );

    // Request 2: kein 200 mehr — Err (ConnectionRefused) ODER leere Antwort
    let second = get_health(addr);
    let served = second.map(|r| r.contains("200")).unwrap_or(false);
    assert!(!served, "second request should NOT be served");
}

#[tokio::test(flavor = "multi_thread")]
async fn serve_handles_multiple_connections() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let registry: rust_server_learning::server::Registry = Arc::new(Mutex::new(HashMap::new()));
    // serve läuft endlos (pending() wird nie fertig → kein Shutdown).
    tokio::spawn(async move {
        let _ = serve(
            &listener,
            std::future::pending::<()>(),
            Duration::from_millis(100),
            registry,
        )
        .await;
    });

    for i in 0..3 {
        let response = get_health(addr).unwrap_or_else(|e| panic!("request {i} failed: {e}"));
        assert!(
            response.contains("200"),
            "request {i} should get 200, got: {response}"
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn slow_connection_does_not_block_others() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let registry: rust_server_learning::server::Registry = Arc::new(Mutex::new(HashMap::new()));

    tokio::spawn(async move {
        let _ = serve(
            &listener,
            std::future::pending::<()>(),
            Duration::from_millis(100),
            registry,
        )
        .await;
    });

    // Client A: verbindet, schickt NICHTS. In async blockiert das die anderen NICHT,
    // weil read().await den Thread freigibt (nicht wegen eigener Threads wie in L15).
    let _slow = TcpStream::connect(addr).unwrap();
    thread::sleep(Duration::from_millis(100)); // A ist akzeptiert & wartet im read

    // Client B: vollständige Anfrage, MUSS trotzdem prompt bedient werden
    let mut fast = TcpStream::connect(addr).unwrap();
    fast.write_all(b"GET /api/health HTTP/1.1\r\n\r\n").unwrap();
    fast.shutdown(Shutdown::Write).unwrap();
    fast.set_read_timeout(Some(Duration::from_secs(2))).unwrap();

    let mut buf = String::new();
    fast.read_to_string(&mut buf)
        .expect("B should be served despite A hanging");
    assert!(buf.contains("200"), "B should get 200, got: {buf}");
}
