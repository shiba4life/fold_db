# Application Layer Technical Specification

## Overview
The application layer provides a secure runtime environment for third-party applications while maintaining strict control over data access and network communications through the DataFold Node API.

## Container Architecture

### Container Runtime
- Docker-based sandboxed environments
- Stateless container design
- Controlled network access
- Resource limits enforcement
- Ephemeral storage only

### Security Model
1. Isolation
   - Network isolation through Docker networks
   - File system isolation
   - Process isolation
   - Resource quotas

2. Access Control
   - All external communication blocked
   - Node API as sole communication channel
   - Public key based authentication
   - Permission validation per request

## Node API Interface

### API Gateway
1. Request Flow
   ```
   Application -> Docker Network -> Node API Gateway -> Permission Check -> Schema Validation -> Data Operation
   ```

2. Authentication
   - Public key required for all operations
   - JWT tokens for session management
   - Request signing validation

### Available APIs

1. Schema Operations
   ```rust
   // Get available schemas
   fn get_schemas() -> Result<Vec<Schema>>
   
   // Get specific schema
   fn get_schema(name: String) -> Result<Schema>
   ```

2. Data Operations
   ```rust
   // Read operations
   fn read(schema: String, query: Query) -> Result<Data>
   
   // Write operations
   fn write(schema: String, data: Data) -> Result<AtomRef>
   
   // Update operations
   fn update(schema: String, atom_ref: AtomRef, data: Data) -> Result<AtomRef>
   ```

   Query Target Resolution:
   - Queries include a `target` field that determines the data source:
     ```rust
     pub enum QueryTarget {
         ActiveUser,    // Query authenticated user's data
         Node(NodeId),  // Query specific node's data
     }
     ```
   - ActiveUser queries:
     - Use authenticated user's public key for context
     - Apply user-specific permissions
     - Limited to user's trust distance
   
   - Node queries:
     - Require explicit node identification
     - Apply node-specific trust distance checks
     - Validate cross-node permissions

3. Permission Operations
   ```rust
   // Check permissions
   fn check_permission(schema: String, field: String) -> Result<Permission>
   
   // Get trust distance
   fn get_trust_distance(public_key: PublicKey) -> Result<u32>
   ```

4. Payment Operations
   ```rust
   // Get payment requirements
   fn get_payment_requirements(operation: Operation) -> Result<PaymentRequirements>
   
   // Submit payment proof
   fn submit_payment(proof: PaymentProof) -> Result<PaymentVerification>
   ```

5. Network Operations
   ```rust
   // Get network information
   fn get_network_info() -> Result<NetworkInfo>
   
   // Get node information
   fn get_node_info() -> Result<NodeInfo>

   // Get trusted nodes
   fn get_trusted_nodes() -> Result<Vec<NodeId>>

   // Get node status
   fn get_node_status() -> Result<NodeStatus>
   ```

## Application Container Lifecycle

1. Initialization
   ```
   1. Node receives application deployment request
   2. Validates application package
   3. Creates isolated Docker network
   4. Launches container with resource limits
   5. Injects Node API credentials
   6. Establishes API connection
   ```

2. Runtime
   ```
   1. Application runs in isolated environment
   2. All external requests routed through Node API
   3. Permissions checked per request
   4. Payments validated when required
   5. Data operations atomic and versioned
   ```

3. Termination
   ```
   1. Graceful shutdown signal sent
   2. Pending operations completed
   3. API connection closed
   4. Container stopped
   5. Network removed
   6. Resources cleaned up
   ```

## Implementation Guidelines

### Container Configuration
```yaml
version: '3'
services:
  app:
    network_mode: none
    security_opt:
      - no-new-privileges
    read_only: true
    tmpfs:
      - /tmp
    environment:
      - NODE_API_KEY=${API_KEY}
      - NODE_API_ENDPOINT=${API_ENDPOINT}
    resources:
      limits:
        memory: 512M
        cpu_count: 2
```

### API Client Implementation
```rust
pub struct NodeApiClient {
    endpoint: String,
    public_key: PublicKey,
    private_key: PrivateKey,
}

impl NodeApiClient {
    // Initialize client with credentials
    pub fn new(endpoint: String, public_key: PublicKey, private_key: PrivateKey) -> Self;
    
    // Sign and send request
    async fn send_request(&self, request: Request) -> Result<Response>;
    
    // Verify response signature
    fn verify_response(&self, response: Response) -> Result<bool>;
}
```

### Error Handling
```rust
pub enum ApplicationError {
    // Authentication errors
    InvalidCredentials,
    ExpiredToken,
    
    // Permission errors
    InsufficientPermissions,
    TrustDistanceExceeded,
    
    // Payment errors
    PaymentRequired,
    PaymentVerificationFailed,
    
    // API errors
    NetworkError,
    InvalidRequest,
    ServerError,
}
```

## Security Considerations

1. Container Security
   - No privileged access
   - Read-only file system
   - Memory/CPU limits
   - No host network access
   - No volume mounts

2. API Security
   - All requests signed
   - All responses verified
   - Rate limiting
   - Request validation
   - Token expiration

3. Data Security
   - Field-level permissions
   - Trust distance validation
   - Atomic operations
   - Version tracking
   - Payment verification

## Best Practices

1. Application Development
   - Use provided Node API client
   - Implement proper error handling
   - Handle payment requirements
   - Cache schema information
   - Batch operations when possible

2. Resource Management
   - Clean up resources
   - Handle container lifecycle
   - Implement graceful shutdown
   - Monitor resource usage
   - Cache appropriately

3. Security
   - Secure credential storage
   - Regular key rotation
   - Request signing
   - Response verification
   - Error handling

4. Performance
   - Connection pooling
   - Request batching
   - Local caching
   - Resource monitoring
   - Error recovery
