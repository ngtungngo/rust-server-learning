use rust_server_learning::ServerConfig;

fn main() {
    match ServerConfig::new("localhost", "8080") {
        Ok(config) => {
            println!("Server will listen on {}", config.bind_address());
        }
        Err(message) => {
            println!("Configuration error: {message}");
        }
    }
}
