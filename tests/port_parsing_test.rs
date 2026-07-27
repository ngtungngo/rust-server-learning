use rust_server_learning::parse_port;

#[test]
fn accepts_a_valid_port() {
    assert_eq!(parse_port("8080"), Ok(8080));
}

#[test]
fn rejects_non_numeric_input() {
    assert_eq!(
        parse_port("abc"),
        Err(String::from("'abc' is not a valid port number"))
    );
}

#[test]
fn rejects_port_zero() {
    assert_eq!(parse_port("0"), Err(String::from("Port 0 is not allowed")));
}
