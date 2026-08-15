use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct ItemName(String);

impl ItemName {
    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn parse(input: String) -> Result<ItemName, ValidationError> {
        let name = input.trim();
        if name.is_empty() {
            tracing::warn!("rejected empty name");
            return Err(ValidationError::Empty);
        }

        if name.len() > 100 {
            tracing::warn!("rejected to long name");
            return Err(ValidationError::TooLong { actual: name.len() });
        }

        Ok(ItemName(name.to_string()))
    }
}

#[derive(Debug, PartialEq, Error)]
pub enum ValidationError {
    #[error("name must not be empty")]
    Empty,
    #[error("name is too long: {actual} bytes (max 100)")]
    TooLong { actual: usize },
}
