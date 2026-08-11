use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::time::Duration;
use tokio::time::Instant;
use rust_server_learning::server::{serve, serve_one};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
mod common;                 // bindet tests/common/mod.rs ein
use common::get_health;

#[tokio::test]
async fn serve_returns_when_shutdown_future_completes() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();

    // Shutdown-Signal: ein Future, das nach 100ms fertig ist
    let shutdown = tokio::time::sleep(std::time::Duration::from_millis(100));

    // Wenn serve das Signal beachtet, kehrt es zurück → der Test terminiert.
    // Ein Timeout drumherum macht "hängt ewig" zu einem klaren Fehler.
    let registry: rust_server_learning::server::Registry =
        Arc::new(Mutex::new(HashMap::new()));

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        serve(&listener, shutdown, Duration::from_millis(100), registry),
    ).await;

    assert!(result.is_ok(), "serve did not return within 2s — it ignored the shutdown signal");
    assert!(result.unwrap().is_ok(), "serve should return Ok");
}

#[tokio::test(flavor = "multi_thread")]
async fn serve_waits_for_inflight_connection() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // NUR Client-Code im sync-Closure — kein .await hier drin.
    let client = std::thread::spawn(move || {
        let mut stream = TcpStream::connect(addr).unwrap();
        stream.write_all(b"GET /api/slow HTTP/1.1\r\n\r\n").unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    });

    // Ab hier wieder im async fn — .await ist erlaubt.
    let shutdown = tokio::time::sleep(Duration::from_millis(100));

    let start = Instant::now();
    let registry: rust_server_learning::server::Registry =
        Arc::new(Mutex::new(HashMap::new()));
    serve(&listener, shutdown, Duration::from_millis(100), registry).await.unwrap();
    let elapsed = start.elapsed();

    assert!(
        elapsed >= Duration::from_millis(450),
        "serve returned after {elapsed:?} — it did NOT wait for the in-flight connection"
    );

    let response = client.join().unwrap();
    assert!(response.contains("200"), "slow client should still get 200, got: {response}");
}

#[tokio::test(flavor = "multi_thread")]
async fn serve_one_drops_a_silent_connection_after_timeout() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Slow-Loris: verbindet (TCP-Handshake macht das OS), sendet NIE etwas,
    // hält die Verbindung offen (nicht droppen → _silent bindet sie).
    let _silent = std::net::TcpStream::connect(addr).unwrap();

    // Ohne Read-Timeout hinge serve_one ewig im read().await → äußerer timeout(2s) feuert → rot.
    let start = std::time::Instant::now();
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        serve_one(&listener, std::time::Duration::from_millis(100)),
    ).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok(), "serve_one hung — the connection timeout did not fire");
    assert!(elapsed < std::time::Duration::from_secs(1),
            "serve_one took {elapsed:?} — timeout fired too late");
}


#[tokio::test(flavor = "multi_thread")]
async fn can_abort_a_specific_connection() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let registry: rust_server_learning::server::Registry =
        Arc::new(Mutex::new(HashMap::new()));
    let reg = Arc::clone(&registry);

    tokio::spawn(async move {
        // langer Timeout, damit NICHT 4c die Verbindung beendet, sondern unser abort
        let _ = serve(&listener, std::future::pending::<()>(),
                      Duration::from_secs(30), reg).await;
    });

    // Slow-Loris A: verbindet, sendet nie → hängt in read().await → wird registriert
    let _a = std::net::TcpStream::connect(addr).unwrap();

    // warten, bis A registriert ist (max ~1s)
    let handle = loop {
        if let Some(h) = registry.lock().unwrap().values().next().cloned() {
            break h;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    };

    // Server lebt + andere Verbindungen gehen: B kriegt trotzdem 200
    let b = get_health(addr).unwrap();
    assert!(b.contains("200"), "B should be served while A hangs, got: {b}");

    // GEZIELT A killen
    assert!(!handle.is_finished(), "A should still be running before abort");
    handle.abort();

    // abort greift am nächsten Poll (A hängt in read().await = Yield-Punkt)
    for _ in 0..50 {
        if handle.is_finished() { break; }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(handle.is_finished(), "A's task was not aborted");
}
