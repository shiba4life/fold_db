use std::net::SocketAddr;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::schema::SchemaError;

/// Unique identifier for a node in the network
pub type NodeId = String;

/// Configuration for the network layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Address to listen for incoming connections
    pub listen_address: SocketAddr,
    /// Port for node discovery broadcasts
    pub discovery_port: u16,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Timeout for connection attempts
    pub connection_timeout: Duration,
    /// Whether to enable automatic node discovery
    pub enable_discovery: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_address: "127.0.0.1:9000".parse().unwrap(),
            discovery_port: 9001,
            max_connections: 50,
            connection_timeout: Duration::from_secs(10),
            enable_discovery: true,
        }
    }
}

/// Information about a node in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Unique identifier for the node
    pub node_id: NodeId,
    /// Network address of the node
    pub address: SocketAddr,
    /// Trust distance to this node
    pub trust_distance: u32,
    /// Public key for authentication
    pub public_key: Option<String>,
    /// Node capabilities
    pub capabilities: NodeCapabilities,
}

/// Capabilities of a node in the network
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeCapabilities {
    /// Whether the node supports querying
    pub supports_query: bool,
    /// Whether the node supports schema listing
    pub supports_schema_listing: bool,
}

/// Result of a query operation - internal type
pub type QueryResult = Vec<Result<Value, SchemaError>>;

/// Information about a schema available on a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    /// Name of the schema
    pub name: String,
    /// Version of the schema
    pub version: String,
    /// Description of the schema
    pub description: Option<String>,
}
