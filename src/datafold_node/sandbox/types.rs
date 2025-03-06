use std::collections::HashMap;
use std::fmt;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

/// Result type for sandbox operations
pub type SandboxResult<T> = Result<T, SandboxError>;

/// Error type for sandbox operations
#[derive(Debug)]
pub enum SandboxError {
    /// Docker API error
    Docker(String),
    /// Network error
    Network(String),
    /// Configuration error
    Config(String),
    /// Security error
    Security(String),
    /// Resource limit error
    ResourceLimit(String),
    /// Container not found
    ContainerNotFound(String),
    /// API error
    Api(String),
    /// Internal error
    Internal(String),
}

impl fmt::Display for SandboxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SandboxError::Docker(msg) => write!(f, "Docker error: {}", msg),
            SandboxError::Network(msg) => write!(f, "Network error: {}", msg),
            SandboxError::Config(msg) => write!(f, "Configuration error: {}", msg),
            SandboxError::Security(msg) => write!(f, "Security error: {}", msg),
            SandboxError::ResourceLimit(msg) => write!(f, "Resource limit error: {}", msg),
            SandboxError::ContainerNotFound(msg) => write!(f, "Container not found: {}", msg),
            SandboxError::Api(msg) => write!(f, "API error: {}", msg),
            SandboxError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for SandboxError {}

/// Information about a container
#[derive(Debug, Clone)]
pub struct ContainerInfo {
    /// Container ID
    pub id: String,
    /// Container name
    pub name: String,
    /// Container image
    pub image: String,
    /// Container status
    pub status: ContainerStatus,
    /// Creation time
    pub created_at: SystemTime,
    /// Container labels
    pub labels: HashMap<String, String>,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Network ID
    pub network_id: Option<String>,
}

/// Container status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerStatus {
    /// Container is running
    Running,
    /// Container is paused
    Paused,
    /// Container is stopped
    Stopped,
    /// Container is exited
    Exited(i32),
    /// Container status is unknown
    Unknown,
}

/// Resource limits for a container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// CPU limit (in CPU shares)
    pub cpu_limit: Option<f64>,
    /// Memory limit (in bytes)
    pub memory_limit: Option<u64>,
    /// Disk limit (in bytes)
    pub disk_limit: Option<u64>,
    /// Network bandwidth limit (in bytes/s)
    pub network_limit: Option<u64>,
    /// Maximum number of processes
    pub pids_limit: Option<i64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu_limit: Some(0.5),  // 0.5 CPU cores
            memory_limit: Some(512 * 1024 * 1024),  // 512 MB
            disk_limit: Some(1024 * 1024 * 1024),  // 1 GB
            network_limit: None,  // No network limit
            pids_limit: Some(100),  // 100 processes
        }
    }
}

/// Request type for API calls
#[derive(Debug, Clone)]
pub struct Request {
    /// Container ID
    pub container_id: String,
    /// Request path
    pub path: String,
    /// Request method
    pub method: String,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Request body
    pub body: Option<Vec<u8>>,
}

/// Response type for API calls
#[derive(Debug, Clone)]
pub struct Response {
    /// Response status code
    pub status: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: Option<Vec<u8>>,
}

/// Operation type for API calls
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OperationType {
    /// Query operation
    Query,
    /// Mutation operation
    Mutation,
    /// Schema operation
    Schema,
    /// Node operation
    Node,
    /// App operation
    App,
    /// System operation
    System,
}
