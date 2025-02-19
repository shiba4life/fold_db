use std::path::PathBuf;
use serde::{Serialize, Deserialize};

/// Configuration for a DataFoldNode instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Path where the node will store its data
    pub storage_path: PathBuf,
    /// Default trust distance for queries when not explicitly specified
    /// Must be greater than 0
    pub default_trust_distance: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub trust_distance: u32,
}
