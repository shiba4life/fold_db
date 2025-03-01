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
    /// Interval for sending node announcements
    pub announcement_interval: Duration,
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
            announcement_interval: Duration::from_secs(60),
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

/// Serializable version of a query result for network communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableQueryResult {
    /// Successful results
    pub successes: Vec<Value>,
    /// Error messages for failed results
    pub errors: Vec<String>,
}

impl From<QueryResult> for SerializableQueryResult {
    fn from(result: QueryResult) -> Self {
        let mut successes = Vec::new();
        let mut errors = Vec::new();
        
        for item in result {
            match item {
                Ok(value) => successes.push(value),
                Err(err) => errors.push(err.to_string()),
            }
        }
        
        Self { successes, errors }
    }
}

impl From<SerializableQueryResult> for QueryResult {
    fn from(result: SerializableQueryResult) -> Self {
        let mut query_result = Vec::new();
        
        // Add successful results
        for value in result.successes {
            query_result.push(Ok(value));
        }
        
        // Add error results
        for error_msg in result.errors {
            query_result.push(Err(SchemaError::InvalidData(error_msg)));
        }
        
        query_result
    }
}

/// State of a connection to a node
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Connection is being established
    Connecting,
    /// Connection is established and ready for use
    Connected,
    /// Connection is being authenticated
    Authenticating,
    /// Connection is authenticated and ready for use
    Ready,
    /// Connection is closing
    Closing,
    /// Connection is closed
    Closed,
    /// Connection has failed
    Failed,
}

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
