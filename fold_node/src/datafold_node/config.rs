use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for a DataFoldNode instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Path where the node will store its data
    pub storage_path: PathBuf,
    /// Default trust distance for queries when not explicitly specified
    /// Must be greater than 0
    pub default_trust_distance: u32,
    /// Network listening address
    #[serde(default = "default_network_listen_address")]
    pub network_listen_address: String,
}

fn default_network_listen_address() -> String {
    "/ip4/0.0.0.0/tcp/0".to_string()
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            storage_path: PathBuf::from("data"),
            default_trust_distance: 1,
            network_listen_address: default_network_listen_address(),
        }
    }
}

impl NodeConfig {
    /// Create a new node configuration with the specified storage path
    pub fn new(storage_path: PathBuf) -> Self {
        Self {
            storage_path,
            ..Default::default()
        }
    }

    /// Set the network listening address
    pub fn with_network_listen_address(mut self, address: &str) -> Self {
        self.network_listen_address = address.to_string();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub trust_distance: u32,
}
