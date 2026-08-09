use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use rust_server_learning::server::serve;
use rust_server_learning::ServerConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = ServerConfig::new("localhost", "8080")?;
    let listener = TcpListener::bind(config.bind_address())?;   // Socket öffnen
    tracing::info!("Server will listen on {}", config.bind_address());
    let shutdown = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&shutdown);          // zweit
    serve(&listener, flag)?;
    Ok(())
}
