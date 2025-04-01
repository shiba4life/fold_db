use std::fmt;
use std::io;
use thiserror::Error;

/// Error type for the App SDK
#[derive(Debug, Error)]
pub enum AppSdkError {
    /// Errors related to client operations
    #[error("Client error: {0}")]
    Client(String),
    
    /// Errors related to container operations
    #[error("Container error: {0}")]
    Container(String),
    
    /// Errors related to network operations
    #[error("Network error: {0}")]
    Network(String),
    
    /// Errors related to permission checks
    #[error("Permission error: {0}")]
    Permission(String),
    
    /// Errors related to schema operations
    #[error("Schema error: {0}")]
    Schema(String),
    
    /// Errors related to authentication
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    /// Errors related to serialization/deserialization
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Errors related to IO operations
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    /// Errors from the underlying database
    #[error("Database error: {0}")]
    Database(String),
    
    /// Other errors that don't fit into the above categories
    #[error("Error: {0}")]
    Other(String),
}

/// Conversion from serde_json::Error to AppSdkError
impl From<serde_json::Error> for AppSdkError {
    fn from(error: serde_json::Error) -> Self {
        AppSdkError::Serialization(error.to_string())
    }
}

/// Result type alias for operations that can result in an AppSdkError
pub type AppSdkResult<T> = Result<T, AppSdkError>;
