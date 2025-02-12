use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Socket configuration for the server
#[derive(Debug, Clone)]
pub struct SocketConfig {
    /// Path to the Unix Domain Socket
    pub socket_path: PathBuf,
    /// File permissions for the socket
    pub permissions: u32,
    /// Buffer size for socket operations
    pub buffer_size: usize,
}

impl Default for SocketConfig {
    fn default() -> Self {
        Self {
            socket_path: PathBuf::from(super::DEFAULT_SOCKET_PATH),
            permissions: super::DEFAULT_SOCKET_PERMISSIONS,
            buffer_size: super::DEFAULT_BUFFER_SIZE,
        }
    }
}

/// Types of operations that can be performed
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationType {
    Query,
    Mutation,
    GetSchema,
}

/// Status of the API response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Success,
    Error,
}

/// Authentication context for requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    /// Public key for authentication
    pub public_key: String,
}

/// API request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequest {
    /// Unique request identifier
    pub request_id: String,
    /// Type of operation to perform
    pub operation_type: OperationType,
    /// Request payload as JSON value
    pub payload: serde_json::Value,
    /// Authentication context
    pub auth: AuthContext,
}

/// Error details for failed operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
}

/// API response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    /// Request identifier (matches request)
    pub request_id: String,
    /// Response status
    pub status: ResponseStatus,
    /// Response data (if successful)
    pub data: Option<serde_json::Value>,
    /// Error details (if failed)
    pub error: Option<ErrorDetails>,
}

/// Client-side errors
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(#[from] std::io::Error),
    
    #[error("Operation timed out after {0:?}")]
    Timeout(Duration),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Operation failed: {0}")]
    OperationFailed(String),
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
