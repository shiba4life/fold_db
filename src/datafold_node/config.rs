use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

// Docker container configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    /// Memory limit in bytes
    pub memory_limit: u64,
    /// CPU limit (number of cores)
    pub cpu_limit: f64,
    /// Container environment variables
    pub environment: HashMap<String, String>,
    /// Network configuration
    pub network_config: DockerNetworkConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerNetworkConfig {
    /// Enable network isolation
    pub network_isolated: bool,
    /// Exposed ports configuration
    pub exposed_ports: HashMap<u16, u16>, // container_port -> host_port
}

impl Default for DockerConfig {
    fn default() -> Self {
        Self {
            memory_limit: 512 * 1024 * 1024, // 512MB
            cpu_limit: 1.0,
            environment: HashMap::new(),
            network_config: DockerNetworkConfig {
                network_isolated: true,
                exposed_ports: HashMap::new(),
            },
        }
    }
}

/// Configuration for a DataFoldNode instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Path where the node will store its data
    pub storage_path: PathBuf,
    /// Default trust distance for queries when not explicitly specified
    /// Must be greater than 0
    pub default_trust_distance: u32,
    /// Docker configuration for containerized applications
    #[serde(default)]
    pub docker: DockerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub trust_distance: u32,
}
