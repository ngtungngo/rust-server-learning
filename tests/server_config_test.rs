use rust_server_learning::ServerConfig;

#[test]
fn creates_a_server_config_from_valid_values() {
    let config = ServerConfig::new("localhost", "8080").unwrap();

    assert_eq!(config.host(), "localhost");
    assert_eq!(config.port(), 8080);
}

#[test]
fn rejects_an_empty_host() {
    assert_eq!(
        ServerConfig::new("   ", "8080"),
        Err(String::from("Host must not be empty"))
    );
}

#[test]
fn rejects_an_invalid_port() {
    assert_eq!(
        ServerConfig::new("localhost", "abc"),
        Err(String::from("'abc' is not a valid port number"))
    );
}
