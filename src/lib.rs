pub fn parse_port(input: &str) -> Result<u16, ConfigError> {
    let port: u16 = input
        .parse()
        .map_err(|_| ConfigError::InvalidPort(input.to_owned()))?;

    if port == 0 {
        return Err(ConfigError::PortZero);
    }

    Ok(port)
}

#[derive(Debug, PartialEq)]
pub enum ConfigError {
    EmptyHost,
    InvalidPort(String),
    PortZero
}

#[derive(Debug, PartialEq)]
pub struct ServerConfig {
    host: String,
    port: u16,
}

impl ServerConfig {
    pub fn new(host: &str, port_input: &str) -> Result<Self, ConfigError> {
        let host = host.trim();

        if host.is_empty() {
            return Err(ConfigError::EmptyHost);
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

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::EmptyHost => write!(f, "Host must not be empty"),
            ConfigError::InvalidPort(value) => write!(f, "'{value}' is not a valid port number"),
            ConfigError::PortZero => write!(f, "Port 0 is not allowed"),
        }
    }
}