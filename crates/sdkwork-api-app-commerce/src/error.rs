pub type CommerceResult<T> = std::result::Result<T, CommerceError>;

#[derive(Debug)]
pub enum CommerceError {
    InvalidInput(String),
    NotFound(String),
    Conflict(String),
    Storage(anyhow::Error),
}

impl std::fmt::Display for CommerceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(f, "{message}"),
            Self::NotFound(message) => write!(f, "{message}"),
            Self::Conflict(message) => write!(f, "{message}"),
            Self::Storage(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for CommerceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

impl From<anyhow::Error> for CommerceError {
    fn from(value: anyhow::Error) -> Self {
        Self::Storage(value)
    }
}

pub fn commerce_atomic_coupon_error(error: anyhow::Error) -> CommerceError {
    let message = error.to_string();
    if message.contains("changed concurrently")
        || message.contains("already exists with different state")
        || message.contains(" is missing")
    {
        CommerceError::Conflict(message)
    } else {
        CommerceError::Storage(error)
    }
}
