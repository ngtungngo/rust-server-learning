pub fn parse_port(input: &str) -> Result<u16, String> {
    let port: u16 = input
        .parse()
        .map_err(|_| format!("'{input}' is not a valid port number"))?;

    if port == 0 {
        return Err(String::from("Port 0 is not allowed"));
    }

    Ok(port)
}
