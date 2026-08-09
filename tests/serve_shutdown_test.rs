use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
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