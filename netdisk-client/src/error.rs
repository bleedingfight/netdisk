use std::fmt;

/// Error type for netdisk client operations
#[derive(Debug)]
pub enum NetdiskError {
    /// HTTP request error
    RequestError(reqwest::Error),
    /// API returned an error response
    ApiError { code: i32, message: String },
    /// Authentication error
    AuthError(String),
    /// Invalid configuration
    ConfigError(String),
    /// Serialization/deserialization error
    SerializationError(String),
    /// Other errors
    Other(String),
}

impl fmt::Display for NetdiskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetdiskError::RequestError(e) => write!(f, "Request error: {}", e),
            NetdiskError::ApiError { code, message } => write!(f, "API error ({}): {}", code, message),
            NetdiskError::AuthError(msg) => write!(f, "Authentication error: {}", msg),
            NetdiskError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            NetdiskError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            NetdiskError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for NetdiskError {}

impl From<reqwest::Error> for NetdiskError {
    fn from(err: reqwest::Error) -> Self {
        NetdiskError::RequestError(err)
    }
}

impl From<serde_json::Error> for NetdiskError {
    fn from(err: serde_json::Error) -> Self {
        NetdiskError::SerializationError(err.to_string())
    }
}

/// Result type for netdisk client operations
pub type Result<T> = std::result::Result<T, NetdiskError>;