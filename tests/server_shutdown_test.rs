use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use rust_server_learning::server::serve;

#[test]
fn serve_returns_after_shutdown_signal() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let shutdown = Arc::new(AtomicBool::new(false));

    let flag = Arc::clone(&shutdown);          // zweiter Handle auf DASSELBE Flag
    let server = thread::spawn(move || {
        serve(&listener, flag)                  // Server-Thread liest das Flag
    });

    thread::sleep(Duration::from_millis(100));  // Server sicher im Loop
    shutdown.store(true, Ordering::SeqCst);     // Signal setzen (Test-Thread schreibt)

    // Beweis: der Server-Thread endet von selbst — join() blockiert nicht ewig.
    let result = server.join().expect("server thread panicked");
    assert!(result.is_ok(), "serve should return Ok after shutdown");
}

#[test]
fn serve_waits_for_inflight_connection() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let shutdown = Arc::new(AtomicBool::new(false));

    let flag = Arc::clone(&shutdown);
    let server = thread::spawn(move || serve(&listener, flag));

    // Client öffnet eine langsame Verbindung (Handler schläft 500ms)
    let client = thread::spawn(move || {
        let mut stream = TcpStream::connect(addr).unwrap();
        stream.write_all(b"GET /api/slow HTTP/1.1\r\n\r\n").unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
        let mut buf = String::new();
        stream.read_to_string(&mut buf).unwrap();
        buf
    });

    thread::sleep(Duration::from_millis(100));   // Handler ist jetzt mitten im 500ms-sleep
    let start = Instant::now();
    shutdown.store(true, Ordering::SeqCst);       // Signal MITTEN in der laufenden Anfrage

    let result = server.join().expect("server thread panicked");
    let elapsed = start.elapsed();

    assert!(result.is_ok(), "serve should return Ok");
    // graceful: serve wartet auf den ~400ms Rest-Schlaf. detached: serve kehrt sofort (~0ms) zurück.
    assert!(
        elapsed >= Duration::from_millis(300),
        "serve returned too early ({elapsed:?}) — it did not wait for the in-flight handler"
    );
    // der Client bekommt trotzdem seine vollständige Antwort
    assert!(client.join().unwrap().contains("200"));
}
