use std::time::Duration;
use rust_server_learning::ServerConfig;
use rust_server_learning::server::serve;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = ServerConfig::new("localhost", "8080")?;
    let listener = tokio::net::TcpListener::bind(config.bind_address()).await?; // Socket öffnen
    tracing::info!("Server will listen on {}", config.bind_address());
    let shutdown = async {
        let _ = tokio::signal::ctrl_c().await; // warte auf Strg-C; Fehler ignorieren
    };
    serve(&listener, shutdown, Duration::from_millis(100)).await?;
    Ok(())
}
