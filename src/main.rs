use rust_server_learning::parse_port;

fn main() {
    match parse_port("8080") {
        Ok(port) => println!("Server will use port {port}"),
        Err(message) => println!("Configuration error: {message}"),
    }
}
