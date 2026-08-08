use rust_server_learning::ServerConfig;

fn main() {
    match ServerConfig::new("localhost", "8080") {
        Ok(config) => {
            println!("Server will listen on {}", config.bind_address());
            println!("Server will listen on {}:{}", config.host(), config.port());
        }
        Err(error) => {
            println!("Configuration error: {error}");
        }
    }
}
