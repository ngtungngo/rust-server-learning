use rust_server_learning::ServerConfig;

#[test]
fn creates_a_server_config_from_valid_values() {
    let config = ServerConfig::new("localhost", "8080").unwrap();

    assert_eq!(config.host(), "localhost");
    assert_eq!(config.port(), 8080);
}

#[test]
fn formats_a_bind_address() {
    let config = ServerConfig::new("localhost", "8080").unwrap();

    assert_eq!(config.bind_address(), "localhost:8080");
}

#[test]
fn rejects_an_empty_host() {
    let result = ServerConfig::new("   ", "8080");

    assert!(matches!(result, Err(message) if message == "Host must not be empty"));
}

#[test]
fn rejects_an_invalid_port() {
    let result = ServerConfig::new("localhost", "abc");

    assert!(matches!(result, Err(message) if message == "'abc' is not a valid port number"));
}
