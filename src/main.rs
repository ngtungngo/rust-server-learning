use rust_server_learning::ServerConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::new("localhost", "8080")?;
    println!("Server will listen on {}", config.bind_address());
    Ok(())
}