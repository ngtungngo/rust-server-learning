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