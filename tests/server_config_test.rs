use rust_server_learning::{ConfigError, ServerConfig, parse_port};
use std::error::Error;
use tracing_test::traced_test;

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

    assert!(matches!(result, Err(ConfigError::InvalidPort { value, .. }) if value == "abc"));
}

#[test]
fn config_error_is_a_standard_error() {
    let error: Box<dyn std::error::Error> = ServerConfig::new("   ", "8080").unwrap_err().into();
    assert_eq!(error.to_string(), "Host must not be empty");
}

#[test]
fn invalid_port_reports_its_source() {
    let error = ServerConfig::new("localhost", "abc").unwrap_err();
    assert!(error.source().is_some()); // Ursachenkette existiert
}

#[traced_test]
#[test]
fn logs_a_warning_on_empty_host() {
    let _ = ServerConfig::new("   ", "8080");
    assert!(logs_contain("empty host")); // Text den DU gleich in lib.rs loggst
}

#[traced_test]
#[test]
fn logs_a_warning_on_invalid_port() {
    let _ = parse_port("abc");
    assert!(logs_contain("invalid port"));
}

#[traced_test]
#[test]
fn logs_a_warning_on_zero_port() {
    let _ = parse_port("0");
    assert!(logs_contain("must not be zero"));
}
