use rust_server_learning::ServerConfig;
use rust_server_learning::app::router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = ServerConfig::new("localhost", "8080")?;
    let listener = tokio::net::TcpListener::bind(config.bind_address()).await?;
    tracing::info!("Server will listen on {}", config.bind_address());

    let shutdown = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    axum::serve(listener, router(std::time::Duration::from_secs(50)))
        .with_graceful_shutdown(shutdown)
        .await?;
    Ok(())
}
