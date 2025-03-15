# Network Layer Design

## Overview
The network layer enables DataFold nodes to discover other nodes, establish connections, and query data while maintaining the system's security and trust model. Each node operates independently, keeping its schemas private unless explicitly queried. The implementation uses libp2p for peer-to-peer networking.

## Core Components

### LibP2pNetwork
Core implementation of the network layer using libp2p.

```rust
pub struct LibP2pNetwork {
    /// Local node ID
    local_node_id: NodeId,
    /// Network configuration
    config: NetworkConfig,
    /// Map of known nodes by ID
    known_nodes: Arc<Mutex<HashMap<NodeId, NodeInfo>>>,
    /// Map of connected nodes by ID
    connected_nodes: Arc<Mutex<HashSet<NodeId>>>,
    /// Callback for handling query requests
    query_callback: Arc<Mutex<Box<dyn Fn(Query) -> QueryResult + Send + Sync>>>,
    /// Callback for handling schema list requests
    schema_list_callback: Arc<Mutex<Box<dyn Fn() -> Vec<SchemaInfo> + Send + Sync>>>,
    /// Whether the network is running
    running: Arc<Mutex<bool>>,
}

impl LibP2pNetwork {
    pub fn new(config: NetworkConfig, local_node_id: Option<NodeId>, public_key: Option<String>) -> FoldDbResult<Self>;
    pub async fn start(&mut self) -> FoldDbResult<()>;
    pub async fn stop(&mut self) -> FoldDbResult<()>;
    pub fn set_query_callback<F>(&mut self, callback: F) where F: Fn(Query) -> QueryResult + Send + Sync + 'static;
    pub fn set_schema_list_callback<F>(&mut self, callback: F) where F: Fn() -> Vec<SchemaInfo> + Send + Sync + 'static;
    pub async fn discover_nodes(&mut self) -> FoldDbResult<Vec<NodeInfo>>;
    pub async fn connect_to_node(&mut self, node_id: &NodeId) -> FoldDbResult<()>;
    pub async fn query_node(&self, node_id: &NodeId, query: Query) -> FoldDbResult<QueryResult>;
    pub async fn list_available_schemas(&self, node_id: &NodeId) -> FoldDbResult<Vec<SchemaInfo>>;
    pub fn connected_nodes(&self) -> HashSet<NodeId>;
    pub fn known_nodes(&self) -> HashMap<NodeId, NodeInfo>;
    pub fn get_node_id(&self) -> &NodeId;
}
```

### LibP2pManager
Wrapper around LibP2pNetwork that provides compatibility with the existing NetworkManager API.

```rust
pub struct LibP2pManager {
    /// The underlying libp2p network
    network: Arc<Mutex<LibP2pNetwork>>,
    /// Runtime for executing async tasks
    runtime: tokio::runtime::Handle,
}

impl LibP2pManager {
    pub fn new(config: NetworkConfig, local_node_id: NodeId, public_key: Option<String>) -> FoldDbResult<Self>;
    pub fn set_schema_list_callback<F>(&self, callback: F) where F: Fn() -> Vec<SchemaInfo> + Send + Sync + 'static;
    pub fn set_query_callback<F>(&self, callback: F) where F: Fn(Query) -> QueryResult + Send + Sync + 'static;
    pub fn start(&mut self) -> FoldDbResult<()>;
    pub fn stop(&mut self) -> FoldDbResult<()>;
    pub fn discover_nodes(&mut self) -> FoldDbResult<Vec<NodeInfo>>;
    pub fn connect_to_node(&self, node_id: &NodeId) -> FoldDbResult<()>;
    pub fn query_node(&self, node_id: &NodeId, query: Query) -> FoldDbResult<QueryResult>;
    pub fn list_available_schemas(&self, node_id: &NodeId) -> FoldDbResult<Vec<SchemaInfo>>;
    pub fn connected_nodes(&self) -> HashSet<NodeId>;
    pub fn known_nodes(&self) -> HashMap<NodeId, NodeInfo>;
}
```

### Network Types
Common types used by the network layer.

```rust
/// Unique identifier for a node in the network
pub type NodeId = String;

/// Configuration for the network layer
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

/// Information about a node in the network
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
pub struct NodeCapabilities {
    /// Whether the node supports querying
    pub supports_query: bool,
    /// Whether the node supports schema listing
    pub supports_schema_listing: bool,
}

/// Result of a query operation - internal type
pub type QueryResult = Vec<Result<Value, SchemaError>>;

/// Information about a schema available on a node
pub struct SchemaInfo {
    /// Name of the schema
    pub name: String,
    /// Version of the schema
    pub version: String,
    /// Description of the schema
    pub description: Option<String>,
}
```

## Implementation Details

### Node Discovery Process
1. Node starts up and initializes LibP2pManager
2. LibP2pNetwork begins listening for connections
3. Uses libp2p's discovery mechanisms (mDNS, Kademlia DHT)
4. Maintains list of known nodes with their information
5. Updates node status based on periodic health checks

### Connection Establishment
1. Node initiates connection to peer using libp2p
2. Noise protocol for secure channel
3. Exchange public keys and validate
4. Verify trust distance requirements
5. Establish message protocol version
6. Begin message exchange

### Query Flow
1. Node A connects to Node B
2. Node A requests list of available schemas
3. Node B responds with enabled schema list
4. Node A validates schema availability
5. Node A sends query
6. Node B validates trust and permissions
7. Node B executes query
8. Node B sends response
9. Node A processes results

### Security Measures
- Noise protocol encryption for all connections
- Public key authentication
- Trust distance validation
- Permission enforcement per query
- Connection timeouts
- Node validation checks
- Error handling for security violations

### Error Handling
- Connection failures
- Invalid messages
- Trust violations
- Timeout handling
- Resource exhaustion
- Protocol violations
- Schema mismatches

## Technical Requirements

### Dependencies
- tokio for async runtime
- libp2p for peer-to-peer networking
- serde for serialization
- uuid for message tracking

### Performance Considerations
- Connection pooling
- Message batching
- Efficient serialization
- Resource limits
- Timeout management
- Health monitoring

### Testing Strategy
1. Unit tests for each component
2. Integration tests for full workflows
3. Network simulation tests
4. Security vulnerability tests
5. Performance benchmarks
6. Error handling tests
7. Edge case validation

## Future Enhancements
1. Implement full libp2p functionality with actual networking
2. Add comprehensive tests for the libp2p implementation
3. Add security features for libp2p communication
4. Implement NAT traversal for better connectivity
5. Add node reputation tracking
6. Optimize network operations
