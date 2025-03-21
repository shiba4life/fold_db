# Network Layer Design

## Overview
The network layer enables DataFold nodes to discover other nodes and list available schemas. The implementation uses libp2p for peer-to-peer networking, starting with a minimal viable product (MVP) focused on schema discovery.

## Current Implementation Status

### Completed Components
1. Basic Structure
   - LibP2pNetwork class for network operations
   - LibP2pManager wrapper for compatibility
   - SchemaListProtocol for schema listing
   - FoldDbBehaviour for network behavior

2. Dependencies
   - Added core libp2p crate with necessary features
   - Included support for various protocols (noise, yamux, gossipsub, mdns, kad, etc.)

3. Basic Network Operations
   - Implemented start/stop functionality
   - Added support for node discovery
   - Added support for connecting to nodes
   - Implemented remote querying and schema listing

### In Progress Components
1. Schema Protocol
   - Created SchemaListProtocol for schema listing
   - Implemented SchemaCodec for serializing/deserializing schema messages
   - Added SchemaMessage enum for request/response messages
   - Working on fixing syntax issues and ensuring proper parameter passing

2. Network Behavior
   - Created FoldDbBehaviour for handling network events
   - Added support for mDNS discovery
   - Working on integrating with the LibP2pNetwork implementation

### Encountered Issues
1. Schema Protocol Implementation
   - Syntax errors in the SchemaCodec implementation
   - Issues with parameter passing in function signatures
   - Missing commas in generic type parameters
   - Challenges with the libp2p request-response protocol

2. Network Behavior Integration
   - Challenges with integrating mDNS discovery
   - Issues with the libp2p NetworkBehaviour trait
   - Difficulties with event handling

## Core Components

### SchemaListProtocol
Core implementation of the schema listing protocol.

```rust
/// Protocol name and version
#[derive(Debug, Clone)]
pub struct SchemaListProtocol;

impl ProtocolName for SchemaListProtocol {
    const PROTOCOL_NAME: &'static [u8] = b"/datafold/schema-list/1.0.0";
}

/// Message types for the protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaMessage {
    // Request doesn't need parameters for simple listing
    ListRequest,
    // Response includes vector of available schemas
    ListResponse(Vec<SchemaInfo>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}
```

### SchemaCodec
Handles serialization/deserialization of schema messages.

```rust
#[derive(Debug, Clone)]
pub struct SchemaCodec;

#[async_trait]
impl RequestResponseCodec for SchemaCodec {
    type Protocol = SchemaListProtocol;
    type Request = SchemaMessage;
    type Response = SchemaMessage;

    async fn read_request<T>(&mut self, _: &SchemaListProtocol, io: &mut T) -> io::Result<Self::Request>;
    async fn read_response<T>(&mut self, _: &SchemaListProtocol, io: &mut T) -> io::Result<Self::Response>;
    async fn write_request<T>(&mut self, _: &SchemaListProtocol, io: &mut T, _: SchemaMessage) -> io::Result<()>;
    async fn write_response<T>(&mut self, _: &SchemaListProtocol, io: &mut T, response: SchemaMessage) -> io::Result<()>;
}
```

### SchemaListHandler
Manages schema listing requests and responses.

```rust
pub struct SchemaListHandler {
    request_response: RequestResponse<SchemaCodec>,
    schema_manager: Arc<SchemaManager>,
}

impl SchemaListHandler {
    pub fn new(schema_manager: Arc<SchemaManager>) -> Self;
    async fn handle_request(&self, request: SchemaMessage) -> SchemaMessage;
    pub async fn request_schemas(&mut self, peer_id: PeerId) -> Result<Vec<SchemaInfo>>;
}
```

## Implementation Details

### Protocol Flow
1. Node Discovery
   - Uses mDNS for local network discovery
   - Simple peer discovery without complex routing

2. Schema Listing
   - Node A discovers Node B via mDNS
   - Node A sends ListRequest to Node B
   - Node B responds with available schemas
   - No payment or trust validation in MVP

### Message Format
```rust
// Request
SchemaMessage::ListRequest

// Response
SchemaMessage::ListResponse(vec![
    SchemaInfo {
        name: "user_profile".to_string(),
        version: "1.0.0".to_string(),
        description: Some("User profile schema".to_string()),
    },
    // ... more schemas
])
```

### Error Handling
- IO errors during message reading/writing
- Serialization/deserialization errors
- Unexpected message types
- Network timeouts

## Technical Requirements

### Dependencies
- libp2p for peer-to-peer networking
- tokio for async runtime
- serde for serialization
- mDNS for local discovery

### Performance Considerations
- Simple request/response flow
- No complex routing or caching
- Basic error handling
- Local network focus

### Testing Strategy
1. Unit tests for protocol implementation
2. Integration tests for node discovery
3. Error handling tests
4. Local network tests

## Future Enhancements
1. Add trust-based access control
2. Implement payment requirements
3. Add wide-area discovery (Kademlia DHT)
4. Support schema querying
5. Add connection pooling
6. Implement NAT traversal

## Next Steps
1. Fix schema protocol implementation issues:
   - Fix syntax errors in SchemaCodec implementation
   - Ensure proper parameter passing in function signatures
   - Add proper comma separation in generic type parameters

2. Complete network behavior implementation:
   - Implement NetworkBehaviour trait for FoldDbBehaviour
   - Add event handling for network events
   - Integrate with the LibP2pNetwork implementation

3. Implement transport layer setup:
   - Add support for TCP transport
   - Add support for WebSocket transport
   - Add support for QUIC transport (optional)
   - Integrate with the LibP2pNetwork implementation

4. Add peer management:
   - Add support for connection limits
   - Add support for peer prioritization
   - Add support for trust distance checking
   - Integrate with the LibP2pNetwork implementation

5. Add security features:
   - Add support for Noise protocol encryption
   - Add support for public key authentication
   - Add support for trust distance checking
   - Integrate with the LibP2pNetwork implementation

6. Add payment features:
   - Add support for Lightning Network payments
   - Add support for payment verification
   - Integrate with the LibP2pNetwork implementation

7. Add comprehensive tests:
   - Unit tests for protocol implementation
   - Integration tests for node discovery
   - Error handling tests
   - Local network tests
