pub fn parse_port(input: &str) -> Result<u16, String> {
    let port: u16 = input
        .parse()
        .map_err(|_| format!("'{input}' is not a valid port number"))?;

    if port == 0 {
        return Err(String::from("Port 0 is not allowed"));
    }

    Ok(port)
}

pub struct ServerConfig {
    host: String,
    port: u16,
}

impl ServerConfig {
    pub fn new(host: &str, port_input: &str) -> Result<Self, String> {
        let host = host.trim();

        if host.is_empty() {
            return Err(String::from("Host must not be empty"));
        }

        Ok(Self {
            host: host.to_owned(),
            port: parse_port(port_input)?,
        })
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}
