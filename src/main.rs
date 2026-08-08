use std::net::TcpListener;
use rust_server_learning::server::serve_one;
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
    serve_one(&listener)?;
    Ok(())
}
