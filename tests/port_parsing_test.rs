use rust_server_learning::{parse_port, ConfigError};

#[test]
fn accepts_a_valid_port() {
    assert_eq!(parse_port("8080"), Ok(8080));
}

#[test]
fn rejects_non_numeric_input() {
    let source = "abc".parse::<u16>().unwrap_err();
    assert_eq!(
        parse_port("abc"),
        Err(ConfigError::InvalidPort{value: "abc".to_owned(), source})
    );
}

#[test]
fn rejects_port_zero() {
    assert_eq!(parse_port("0"), Err(ConfigError::PortZero));
}
