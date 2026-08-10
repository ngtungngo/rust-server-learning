use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::time::Duration;
use tokio::time::Instant;
use rust_server_learning::server::serve;

#[tokio::test]
async fn serve_returns_when_shutdown_future_completes() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();

    // Shutdown-Signal: ein Future, das nach 100ms fertig ist
    let shutdown = tokio::time::sleep(std::time::Duration::from_millis(100));

    // Wenn serve das Signal beachtet, kehrt es zurück → der Test terminiert.
    // Ein Timeout drumherum macht "hängt ewig" zu einem klaren Fehler.
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        serve(&listener, shutdown),
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
    serve(&listener, shutdown).await.unwrap();
    let elapsed = start.elapsed();

    assert!(
        elapsed >= Duration::from_millis(450),
        "serve returned after {elapsed:?} — it did NOT wait for the in-flight connection"
    );

    let response = client.join().unwrap();
    assert!(response.contains("200"), "slow client should still get 200, got: {response}");
}
