use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use super::types::ResourceLimits;

/// Configuration for the sandbox environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Name of the internal Docker network
    pub network_name: String,
    /// Default resource limits for containers
    pub default_resource_limits: ResourceLimits,
    /// Path to the Docker socket
    pub docker_socket_path: PathBuf,
    /// Whether to use Unix socket communication instead of network
    pub use_unix_socket: bool,
    /// Path to the Unix socket for API communication
    pub api_socket_path: Option<PathBuf>,
    /// Port for the API server
    pub api_port: u16,
    /// Host for the API server
    pub api_host: String,
    /// Security options for containers
    pub security_options: SecurityOptions,
    /// Environment variables to pass to containers
    pub environment_variables: HashMap<String, String>,
    /// Volume mounts for containers
    pub volume_mounts: Vec<VolumeMount>,
    /// Allowed API operations
    pub allowed_operations: Vec<String>,
    /// Rate limits for API calls
    pub rate_limits: RateLimits,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            network_name: "datafold_internal_network".to_string(),
            default_resource_limits: ResourceLimits::default(),
            docker_socket_path: PathBuf::from("/var/run/docker.sock"),
            use_unix_socket: false,
            api_socket_path: Some(PathBuf::from("/var/run/datafold.sock")),
            api_port: 8080,
            api_host: "datafold-api".to_string(),
            security_options: SecurityOptions::default(),
            environment_variables: HashMap::new(),
            volume_mounts: Vec::new(),
            allowed_operations: vec![
                "query".to_string(),
                "schema".to_string(),
                "node".to_string(),
            ],
            rate_limits: RateLimits::default(),
        }
    }
}

/// Security options for containers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityOptions {
    /// Whether to drop all capabilities
    pub drop_all_capabilities: bool,
    /// Capabilities to add back
    pub add_capabilities: Vec<String>,
    /// Whether to disable privilege escalation
    pub no_new_privileges: bool,
    /// Whether to use a read-only root filesystem
    pub read_only_root_fs: bool,
    /// Seccomp profile to use
    pub seccomp_profile: Option<String>,
    /// AppArmor profile to use
    pub apparmor_profile: Option<String>,
}

impl Default for SecurityOptions {
    fn default() -> Self {
        Self {
            drop_all_capabilities: true,
            add_capabilities: Vec::new(),
            no_new_privileges: true,
            read_only_root_fs: true,
            seccomp_profile: Some("default".to_string()),
            apparmor_profile: None,
        }
    }
}

/// Volume mount for containers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    /// Source path on the host
    pub source: PathBuf,
    /// Target path in the container
    pub target: PathBuf,
    /// Whether the mount is read-only
    pub read_only: bool,
}

/// Rate limits for API calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    /// Maximum number of requests per minute
    pub requests_per_minute: u32,
    /// Maximum number of concurrent requests
    pub max_concurrent_requests: u32,
    /// Maximum request size in bytes
    pub max_request_size: u64,
}

impl Default for RateLimits {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            max_concurrent_requests: 10,
            max_request_size: 1024 * 1024, // 1 MB
        }
    }
}
