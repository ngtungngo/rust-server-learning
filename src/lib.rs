use thiserror::Error;

pub fn parse_port(input: &str) -> Result<u16, ConfigError> {
    let port: u16 = input.parse().map_err(|e| ConfigError::InvalidPort {
        value: input.to_owned(),
        source: e,
    })?;

    if port == 0 {
        return Err(ConfigError::PortZero);
    }

    Ok(port)
}

#[derive(Debug, PartialEq, Error)]
pub enum ConfigError {
    #[error("Host must not be empty")]
    EmptyHost,
    #[error("'{value}' is not a valid port number")]
    InvalidPort {
        value: String,
        source: std::num::ParseIntError,
    },
    #[error("Port 0 is not allowed")]
    PortZero,
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
