use rust_server_learning::{ConfigError, ServerConfig};

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
    assert_eq!(result, Err(ConfigError::EmptyHost));
    //assert!(matches!(result, Err(message) if message == "Host must not be empty"));
}

#[test]
fn rejects_an_invalid_port() {
    let result = ServerConfig::new("localhost", "abc");

    assert_eq!(result, Err(ConfigError::InvalidPort("abc".to_owned())));
}

#[test]
fn config_error_is_a_standard_error() {
    let error: Box<dyn std::error::Error> = ServerConfig::new("   ", "8080").unwrap_err().into();
    assert_eq!(error.to_string(), "Host must not be empty");
}
