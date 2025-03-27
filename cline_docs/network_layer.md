# Network Layer Design

## Overview
The network layer enables DataFold nodes to discover other nodes, establish connections, and query data while maintaining the system's security model. Each node operates independently, keeping its schemas private unless explicitly queried. The implementation uses libp2p for robust P2P networking capabilities, leveraging its built-in concurrency guarantees for simplified state management.

## Core Components

### NetworkCore
Central component responsible for coordinating all network operations using libp2p.

```rust
pub struct NetworkCore {
    // Local peer ID
    local_peer_id: PeerId,
    // Schema service for handling schema operations
    schema_service: SchemaService,
    // Known peers
    known_peers: HashSet<PeerId>,
}

impl NetworkCore {
    pub async fn new(config: NetworkConfig) -> NetworkResult<Self>;
    pub async fn run(&mut self, listen_address: &str) -> NetworkResult<()>;
    pub async fn check_schemas(&mut self, peer_id: PeerId, schema_names: Vec<String>) -> NetworkResult<Vec<String>>;
}
```

### SchemaService
Unified service for handling schema operations. Provides a callback mechanism for schema availability checks.

```rust
pub struct SchemaService {
    // Function pointer for schema availability check, set once at initialization
    schema_check_callback: Box<dyn Fn(&[String]) -> Vec<String> + Send + Sync>,
}

impl SchemaService {
    pub fn new() -> Self;
    pub fn set_schema_check_callback<F>(&mut self, callback: F) 
        where F: Fn(&[String]) -> Vec<String> + Send + Sync + 'static;
    // Returns subset of input schema names that are available on this node
    pub fn check_schemas(&self, schema_names: &[String]) -> Vec<String>;
}
```

### Message Protocol
Defines the request-response protocol between nodes.

```rust
#[derive(Serialize, Deserialize)]
enum SchemaRequest {
    // Request to check availability of specific schemas
    CheckSchemas(Vec<String>),
}

#[derive(Serialize, Deserialize)]
enum SchemaResponse {
    // Returns subset of requested schemas that are available
    AvailableSchemas(Vec<String>),
    Error(String),
}
```

### NetworkConfig
Configuration options for the network layer.

```rust
pub struct NetworkConfig {
    // Local listening address
    pub listen_address: String,
    // Request timeout in seconds
    pub request_timeout: u64,
    // Enable mDNS discovery
    pub enable_mdns: bool,
    // Maximum number of concurrent connections
    pub max_connections: usize,
    // Connection keep-alive interval in seconds
    pub keep_alive_interval: u64,
    // Maximum message size in bytes
    pub max_message_size: usize,
    // UDP port for discovery
    pub discovery_port: u16,
    // Connection timeout in seconds
    pub connection_timeout: std::time::Duration,
    // Announcement interval in milliseconds
    pub announcement_interval: std::time::Duration,
}
```

## DataFoldNode Integration

The DataFoldNode has been extended with network capabilities:

```rust
impl DataFoldNode {
    // Initialize the network layer
    pub async fn init_network(&mut self, network_config: NetworkConfig) -> FoldDbResult<()>;
    
    // Start the network service
    pub async fn start_network(&self) -> FoldDbResult<()>;
    
    // Start the network service with a specific listen address
    pub async fn start_network_with_address(&self, listen_address: &str) -> FoldDbResult<()>;
    
    // Discover nodes on the local network using mDNS
    pub async fn discover_nodes(&self) -> FoldDbResult<Vec<PeerId>>;
    
    // Get the list of known nodes
    pub async fn get_known_nodes(&self) -> FoldDbResult<HashMap<String, NodeInfo>>;
    
    // Check which schemas are available on a remote peer
    pub async fn check_remote_schemas(
        &self,
        peer_id_str: &str,
        schema_names: Vec<String>,
    ) -> FoldDbResult<Vec<String>>;
}
```

## Implementation Details

### Node Discovery Process
1. Node starts up and initializes NetworkCore
2. libp2p mDNS discovery automatically finds peers on local network
3. Periodic announcements are made at the configured interval
4. Nodes can actively scan for peers using the discover_nodes method
5. Discovered peers are added to the known_peers list
6. Maintains automatic connections through libp2p

#### mDNS Discovery Implementation
The mDNS discovery process works as follows:

1. When a node starts, it checks if mDNS discovery is enabled in the configuration
2. If enabled, it starts a background task that periodically announces the node's presence
3. Announcements are made at the configured interval (announcement_interval)
4. The announcements include the node's PeerId and listening address
5. Other nodes on the local network receive these announcements and add the node to their known_peers list
6. Nodes can also actively scan for peers using the discover_nodes method
7. The discover_nodes method sends out mDNS queries and waits for responses
8. Discovered peers are added to the known_peers list and returned to the caller

### Connection Establishment
1. libp2p Noise protocol handles secure connection setup
2. Automatic multiplexing through yamux
3. Begin request-response protocol

### Schema Availability Check Flow
1. Node A discovers Node B through mDNS
2. Node A sends CheckSchemas request with list of schema names
3. Node B checks which schemas from the list are available
4. Node B responds with subset of available schemas
5. Node A receives and processes the response

### Concurrency Model
The network layer takes advantage of libp2p's built-in concurrency guarantees:

1. Event Serialization
   - Swarm processes events sequentially
   - Request-response handled in order
   - No manual synchronization needed

2. State Management
   - Network events handled in single event loop
   - Schema requests processed sequentially
   - No shared state between requests

3. Thread Safety
   - Swarm handles connection multiplexing
   - Request-response protocol manages message handling
   - No need for manual lock management

### Security Measures
- Noise protocol encryption for all connections
- libp2p PeerId-based authentication
- Automatic connection management
- Node validation checks
- Error handling for security violations

### Error Handling
- Connection failures
- Invalid requests/responses
- Timeout handling
- Resource exhaustion
- Protocol violations
- Schema mismatches

## Technical Requirements

### Dependencies
- libp2p for P2P networking
  - noise for encryption
  - yamux for multiplexing
  - mdns for discovery
  - request-response protocol
- tokio for async runtime
- serde for serialization
- uuid for request tracking

### Performance Considerations
- Efficient request-response handling
- Automatic connection management
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
1. Kademlia DHT for wider peer discovery
2. WebRTC transport support
3. Custom protocol handlers for specialized operations
4. Node reputation tracking
5. Advanced request validation
6. Cross-network bridging capabilities

## Benefits of libp2p Implementation
1. Reduced code complexity
2. Built-in security features
3. Automatic peer discovery
4. Privacy-preserving schema sharing
5. Battle-tested networking stack
6. Future extensibility
7. Cross-platform compatibility
8. Simplified concurrency model
9. Reduced synchronization overhead
