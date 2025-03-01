# Network Layer Design

## Overview
The network layer enables DataFold nodes to discover other nodes, establish connections, and query data while maintaining the system's security and trust model. Each node operates independently, keeping its schemas private unless explicitly queried.

## Core Components

### NetworkCore
Central component responsible for coordinating all network operations.

```rust
pub struct NetworkCore {
    connection_manager: ConnectionManager,
    message_router: Arc<MessageRouter>,
    query_service: Arc<QueryService>,
    schema_service: Arc<SchemaService>,
    discovery: Arc<Mutex<NodeDiscovery>>,
    config: NetworkConfig,
    local_node_id: NodeId,
}

impl NetworkCore {
    pub fn new(config: NetworkConfig, local_node_id: NodeId, public_key: Option<String>) -> NetworkResult<Self>;
    pub fn start(&mut self) -> NetworkResult<()>;
    pub fn stop(&mut self) -> NetworkResult<()>;
    pub fn set_query_callback<F>(&self, callback: F) where F: Fn(Query) -> QueryResult + Send + Sync + 'static;
    pub fn set_schema_list_callback<F>(&self, callback: F) where F: Fn() -> Vec<SchemaInfo> + Send + Sync + 'static;
    pub fn discover_nodes(&mut self) -> NetworkResult<Vec<NodeInfo>>;
    pub fn connect_to_node(&self, node_id: &NodeId) -> NetworkResult<()>;
    pub fn query_node(&self, node_id: &NodeId, query: Query) -> NetworkResult<QueryResult>;
    pub fn list_available_schemas(&self, node_id: &NodeId) -> NetworkResult<Vec<SchemaInfo>>;
    pub fn connected_nodes(&self) -> HashSet<NodeId>;
    pub fn known_nodes(&self) -> HashMap<NodeId, NodeInfo>;
}
```

### NetworkManager
Simplified wrapper around NetworkCore that provides backward compatibility with the old NetworkManager API.

```rust
pub struct NetworkManager {
    core: NetworkCore,
}

impl NetworkManager {
    pub fn new(config: NetworkConfig, local_node_id: NodeId, public_key: Option<String>) -> NetworkResult<Self>;
    pub fn set_schema_list_callback<F>(&self, callback: F) where F: Fn() -> Vec<SchemaInfo> + Send + Sync + 'static;
    pub fn set_query_callback<F>(&self, callback: F) where F: Fn(Query) -> QueryResult + Send + Sync + 'static;
    pub fn start(&mut self) -> NetworkResult<()>;
    pub fn stop(&mut self) -> NetworkResult<()>;
    pub fn discover_nodes(&mut self) -> NetworkResult<Vec<NodeInfo>>;
    pub fn connect_to_node(&self, node_id: &NodeId) -> NetworkResult<()>;
    pub fn query_node(&self, node_id: &NodeId, query: Query) -> NetworkResult<QueryResult>;
    pub fn list_available_schemas(&self, node_id: &NodeId) -> NetworkResult<Vec<SchemaInfo>>;
    pub fn connected_nodes(&self) -> HashSet<NodeId>;
    pub fn known_nodes(&self) -> HashMap<NodeId, NodeInfo>;
}
```

### NodeDiscovery
Handles node discovery and presence management on the network.

```rust
pub struct NodeDiscovery {
    discovery_method: DiscoveryMethod,
    socket: UdpSocket,
    known_nodes: HashSet<NodeId>,
}

pub struct NodeInfo {
    node_id: NodeId,
    public_key: PublicKey,
    address: SocketAddr,
    trust_distance: u32,
    capabilities: NodeCapabilities,
}

impl NodeDiscovery {
    pub fn new(config: DiscoveryConfig) -> Result<Self>;
    pub fn find_nodes(&mut self) -> Result<Vec<NodeInfo>>;
    pub fn announce_presence(&self) -> Result<()>;
    pub fn handle_node_announcement(&mut self, announcement: NodeAnnouncement) -> Result<()>;
}
```

### Connection
Manages individual peer connections and message handling.

```rust
pub struct Connection {
    node_id: NodeId,
    stream: TcpStream,
    trust_distance: u32,
    last_seen: Instant,
    state: ConnectionState,
}

impl Connection {
    pub fn new(stream: TcpStream, node_id: NodeId) -> Result<Self>;
    pub fn send_message(&mut self, message: Message) -> Result<()>;
    pub fn receive_message(&mut self) -> Result<Message>;
    pub fn validate_node(&self) -> Result<()>;
    pub fn is_healthy(&self) -> bool;
}
```

### QueryService
Unified service for handling query operations (both client and server functionality).

```rust
pub struct QueryService {
    query_callback: Arc<Mutex<Box<dyn Fn(Query) -> QueryResult + Send + Sync>>>,
    pending_queries: Arc<Mutex<HashMap<Uuid, oneshot::Sender<QueryResult>>>>,
}

impl QueryService {
    pub fn new() -> Self;
    pub fn set_query_callback<F>(&mut self, callback: F) where F: Fn(Query) -> QueryResult + Send + Sync + 'static;
    pub fn execute_query(&self, query: Query) -> QueryResult;
    pub fn query_node(&self, connection: Arc<Mutex<Connection>>, query: Query, trust_proof: TrustProof) -> NetworkResult<QueryResult>;
}

impl MessageHandler for QueryService {
    fn handle(&self, message: &Message, node_id: &NodeId) -> NetworkResult<Option<Message>>;
    fn message_types(&self) -> Vec<MessageType>;
}
```

### SchemaService
Unified service for handling schema operations (both client and server functionality).

```rust
pub struct SchemaService {
    schema_list_callback: Arc<Mutex<Box<dyn Fn() -> Vec<SchemaInfo> + Send + Sync>>>,
    pending_requests: Arc<Mutex<HashMap<Uuid, oneshot::Sender<Vec<SchemaInfo>>>>>,
}

impl SchemaService {
    pub fn new() -> Self;
    pub fn set_schema_list_callback<F>(&mut self, callback: F) where F: Fn() -> Vec<SchemaInfo> + Send + Sync + 'static;
    pub fn list_schemas(&self) -> Vec<SchemaInfo>;
    pub fn list_remote_schemas(&self, connection: Arc<Mutex<Connection>>) -> NetworkResult<Vec<SchemaInfo>>;
}

impl MessageHandler for SchemaService {
    fn handle(&self, message: &Message, node_id: &NodeId) -> NetworkResult<Option<Message>>;
    fn message_types(&self) -> Vec<MessageType>;
}
```

### Message Protocol
Defines the communication protocol between nodes.

```rust
pub enum Message {
    Query(QueryMessage),
    QueryResponse(QueryResponseMessage),
    ListSchemasRequest(ListSchemasRequestMessage),
    SchemaListResponse(SchemaListResponseMessage),
    NodeAnnouncement(NodeAnnouncement),
    Error(ErrorMessage),
    Ping(PingMessage),
    Pong(PongMessage),
}

pub struct QueryMessage {
    query_id: Uuid,
    query: Query,
    trust_proof: TrustProof,
}

pub struct QueryResponseMessage {
    query_id: Uuid,
    result: SerializableQueryResult,
}

pub struct ListSchemasRequestMessage {
    request_id: Uuid,
}

pub struct SchemaListResponseMessage {
    request_id: Uuid,
    schemas: Vec<SchemaInfo>,
}

pub struct ErrorMessage {
    code: ErrorCode,
    message: String,
    details: Option<String>,
    related_message_id: Option<Uuid>,
}
```

## Implementation Details

### Node Discovery Process
1. Node starts up and initializes NetworkManager
2. NodeDiscovery begins listening on UDP port for announcements
3. Periodically broadcasts presence on local network
4. Maintains list of known nodes with their information
5. Updates node status based on periodic health checks

### Connection Establishment
1. Node initiates connection to peer
2. TLS handshake for secure channel
3. Exchange public keys and validate
4. Verify trust distance requirements
5. Establish message protocol version
6. Begin message exchange

### Query Flow
1. Node A connects to Node B
2. Node A requests list of available schemas
3. Node B responds with enabled schema list
4. Node A validates schema availability
5. Node A sends query with trust proof
6. Node B validates trust and permissions
7. Node B executes query
8. Node B sends response
9. Node A processes results

### Security Measures
- TLS encryption for all connections
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
- rustls for TLS
- serde for serialization
- uuid for message tracking
- trust-dns for discovery (optional)

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
1. DHT-based discovery
2. NAT traversal
3. Connection multiplexing
4. Protocol versioning
5. Node reputation tracking
