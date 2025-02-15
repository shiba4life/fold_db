use crate::schema::SchemaError;
use std::fmt;

/// Errors that can occur during node operations.
#[derive(Debug)]
pub enum NodeError {
    /// Error occurred in the underlying database
    DatabaseError(String),
    /// Error related to schema operations
    SchemaError(SchemaError),
    /// Error related to insufficient permissions
    PermissionError(String),
    /// Error in node configuration
    ConfigError(String),
    /// Error in Docker operations
    DockerError(String),
}

impl fmt::Display for NodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            Self::SchemaError(err) => write!(f, "Schema error: {}", err),
            Self::PermissionError(msg) => write!(f, "Permission error: {}", msg),
            Self::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            Self::DockerError(msg) => write!(f, "Docker error: {}", msg),
        }
    }
}

impl std::error::Error for NodeError {}

impl From<SchemaError> for NodeError {
    fn from(error: SchemaError) -> Self {
        NodeError::SchemaError(error)
    }
}

impl From<sled::Error> for NodeError {
    fn from(error: sled::Error) -> Self {
        NodeError::DatabaseError(error.to_string())
    }
}

pub type NodeResult<T> = Result<T, NodeError>;
