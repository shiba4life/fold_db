# DataFold Node Application Layer Design

## Overview
This document outlines the design for an application layer that enables dockerized applications to interact with DataFold Node while maintaining strict network isolation. The design focuses on security, reliability, and ease of implementation.

## Architecture

### Components

1. **DataFold Node Container**
   - Runs the core DataFold Node service
   - Exposes API on internal network only
   - No external network access

2. **Application Container**
   - Contains the client application
   - No external network access
   - Communicates with DataFold Node via Unix Domain Sockets

3. **Shared Volume Layer**
   - Facilitates communication between containers
   - Houses Unix Domain Socket
   - Contains shared configuration

### Communication Flow

```
[Application Container] <-> [Unix Domain Socket] <-> [DataFold Node Container]
```

## Network Isolation

### Container Network Configuration

1. **Custom Bridge Network**
```yaml
networks:
  datafold_internal:
    driver: bridge
    internal: true  # Prevents all external network access
    enable_ipv6: false
```

2. **Container Network Policies**
   - No outbound internet access
   - No exposed ports to host network
   - Inter-container communication via Unix sockets only

### Unix Domain Socket Implementation

1. **Socket Configuration**
```rust
pub struct SocketConfig {
    socket_path: PathBuf,
    permissions: u32,
    buffer_size: usize,
}
```

2. **Socket Location**
```
/var/run/datafold/datafold.sock
```

## API Access Mechanism

### Socket-Based API Client

```rust
pub struct DataFoldClient {
    socket_path: PathBuf,
    timeout: Duration,
}

impl DataFoldClient {
    pub fn new(config: ClientConfig) -> Result<Self>;
    pub fn query(&self, query: Query) -> Result<QueryResult>;
    pub fn mutate(&self, mutation: Mutation) -> Result<MutationResult>;
    pub fn get_schema(&self, schema_id: &str) -> Result<Schema>;
}
```

### Request/Response Protocol

```rust
#[derive(Serialize, Deserialize)]
pub struct ApiRequest {
    request_id: String,
    operation_type: OperationType,
    payload: Value,
    auth: AuthContext,
}

#[derive(Serialize, Deserialize)]
pub struct ApiResponse {
    request_id: String,
    status: ResponseStatus,
    data: Option<Value>,
    error: Option<ErrorDetails>,
}
```

## Docker Implementation

### Docker Compose Configuration

```yaml
version: '3.8'
services:
  datafold_node:
    build: 
      context: ./datafold
      dockerfile: Dockerfile.node
    volumes:
      - socket-volume:/var/run/datafold
      - data-volume:/app/data
    networks:
      - datafold_internal
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    read_only: true

  application:
    build: 
      context: ./app
      dockerfile: Dockerfile.app
    volumes:
      - socket-volume:/var/run/datafold
    networks:
      - datafold_internal
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    depends_on:
      - datafold_node

volumes:
  socket-volume:
  data-volume:

networks:
  datafold_internal:
    driver: bridge
    internal: true
```

### Security Configuration

1. **Container Hardening**
   - Run as non-root user
   - Read-only filesystem
   - Dropped capabilities
   - No privilege escalation
   - Resource limits

2. **Socket Permissions**
   - Socket owned by datafold user
   - Group permissions for application access
   - Mode 660 (rw-rw----)

3. **Authentication**
   - Public key authentication required
   - Keys stored in secure volume
   - Trust distance enforced

## Implementation Guidelines

### 1. DataFold Node Setup

```rust
// Initialize socket listener
let socket_config = SocketConfig {
    socket_path: PathBuf::from("/var/run/datafold/datafold.sock"),
    permissions: 0o660,
    buffer_size: 8192,
};

let node = DataFoldNode::new(config)?;
let socket_server = SocketServer::new(socket_config, node)?;
socket_server.start()?;
```

### 2. Application Integration

```rust
// Client application code
let client = DataFoldClient::new(ClientConfig {
    socket_path: PathBuf::from("/var/run/datafold/datafold.sock"),
    timeout: Duration::from_secs(5),
})?;

// Execute operations
let query_result = client.query(Query::new("user_profile")
    .select(&["name", "email"]))?;
```

### 3. Error Handling

```rust
#[derive(Debug)]
pub enum ClientError {
    ConnectionFailed(std::io::Error),
    Timeout(Duration),
    AuthenticationFailed(String),
    OperationFailed(String),
    InvalidResponse(String),
}
```

## Testing Strategy

1. **Unit Tests**
   - Socket communication
   - Protocol serialization/deserialization
   - Error handling
   - Client operations

2. **Integration Tests**
   - End-to-end workflows
   - Network isolation verification
   - Authentication/authorization
   - Error scenarios

3. **Security Tests**
   - Network access attempts
   - Permission boundaries
   - Socket security
   - Authentication bypass attempts

## Monitoring and Observability

1. **Metrics**
   - Operation latency
   - Error rates
   - Connection counts
   - Resource usage

2. **Logging**
   - Operation logs
   - Error logs
   - Security events
   - Performance data

## Implementation Phases

1. **Phase 1: Core Implementation**
   - Unix Domain Socket setup
   - Basic protocol implementation
   - Container configuration
   - Security hardening

2. **Phase 2: Enhanced Features**
   - Advanced error handling
   - Retry mechanisms
   - Connection pooling
   - Performance optimizations

3. **Phase 3: Monitoring & Tooling**
   - Metrics collection
   - Logging infrastructure
   - Administrative tools
   - Documentation

## Conclusion

This design provides a secure and efficient way for dockerized applications to interact with DataFold Node while maintaining strict network isolation. The use of Unix Domain Sockets provides high-performance local communication while the container configuration ensures proper security boundaries.
