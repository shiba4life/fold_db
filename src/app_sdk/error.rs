use std::fmt;
use std::io;
use crate::error::FoldDbError;

/// Error type for the App SDK
#[derive(Debug)]
pub enum AppSdkError {
    /// Errors related to client operations
    Client(String),
    
    /// Errors related to container operations
    Container(String),
    
    /// Errors related to network operations
    Network(String),
    
    /// Errors related to permission checks
    Permission(String),
    
    /// Errors related to schema operations
    Schema(String),
    
    /// Errors related to authentication
    Authentication(String),
    
    /// Errors related to serialization/deserialization
    Serialization(String),
    
    /// Errors related to IO operations
    Io(io::Error),
    
    /// Errors from the underlying FoldDB
    FoldDb(FoldDbError),
    
    /// Other errors that don't fit into the above categories
    Other(String),
}

impl fmt::Display for AppSdkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Client(msg) => write!(f, "Client error: {}", msg),
            Self::Container(msg) => write!(f, "Container error: {}", msg),
            Self::Network(msg) => write!(f, "Network error: {}", msg),
            Self::Permission(msg) => write!(f, "Permission error: {}", msg),
            Self::Schema(msg) => write!(f, "Schema error: {}", msg),
            Self::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            Self::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            Self::Io(err) => write!(f, "IO error: {}", err),
            Self::FoldDb(err) => write!(f, "FoldDB error: {}", err),
            Self::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for AppSdkError {}

/// Conversion from io::Error to AppSdkError
impl From<io::Error> for AppSdkError {
    fn from(error: io::Error) -> Self {
        AppSdkError::Io(error)
    }
}

/// Conversion from serde_json::Error to AppSdkError
impl From<serde_json::Error> for AppSdkError {
    fn from(error: serde_json::Error) -> Self {
        AppSdkError::Serialization(error.to_string())
    }
}

/// Conversion from FoldDbError to AppSdkError
impl From<FoldDbError> for AppSdkError {
    fn from(error: FoldDbError) -> Self {
        AppSdkError::FoldDb(error)
    }
}

/// Result type alias for operations that can result in an AppSdkError
pub type AppSdkResult<T> = Result<T, AppSdkError>;
