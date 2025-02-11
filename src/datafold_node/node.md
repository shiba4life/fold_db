# DataFold Node

## Overview
DataFold Node is a module that provides a complete interface to FoldDB's functionality, allowing users to:
- Load and manage FoldDB instances
- Access schema management capabilities
- Execute mutations and queries
- Handle permissions and trust distance
- Manage atomic operations

## Core Features

### Database Management
- Initialize and load FoldDB instances
- Configure storage locations
- Handle database connections
- Manage database lifecycle

### Schema Operations
- Load schema definitions
- Validate schema structures
- Access schema information
- Update schemas when needed

### Data Operations
- Execute mutations with atomic guarantees
- Perform queries with field-level permissions
- Handle versioning and history tracking
- Manage AtomRefs and version chains

### Permission Management
- Configure trust distances
- Set explicit permissions
- Validate access rights
- Handle public key authentication

## API Structure

### Initialization
```rust
pub struct DataFoldNode {
    db: FoldDB,
    config: NodeConfig,
}

pub struct NodeConfig {
    storage_path: PathBuf,
    default_trust_distance: u32,
}
```

### Core Methods
```rust
impl DataFoldNode {
    // Initialize a new node
    pub fn new(config: NodeConfig) -> Result<Self>;
    
    // Load an existing database
    pub fn load(config: NodeConfig) -> Result<Self>;
    
    // Schema operations
    pub fn load_schema(&self, schema: Schema) -> Result<()>;
    pub fn get_schema(&self, schema_id: &str) -> Result<Schema>;
    
    // Data operations
    pub fn query(&self, query: Query) -> Result<QueryResult>;
    pub fn mutate(&self, mutation: Mutation) -> Result<MutationResult>;
    
    // Permission operations
    pub fn set_trust_distance(&self, distance: u32) -> Result<()>;
    pub fn set_permissions(&self, permissions: Permissions) -> Result<()>;
}
```

## Usage Examples

### Initialize Node
```rust
let config = NodeConfig {
    storage_path: PathBuf::from("./data"),
    default_trust_distance: 1,
};
let node = DataFoldNode::new(config)?;
```

### Load Schema
```rust
let schema = Schema::from_json(schema_json)?;
node.load_schema(schema)?;
```

### Execute Query
```rust
let query = Query::new("user_profile")
    .select(&["name", "email"])
    .filter("id", "=", "123");
let result = node.query(query)?;
```

### Execute Mutation
```rust
let mutation = Mutation::new("user_profile")
    .set("name", "John Doe")
    .set("email", "john@example.com");
let result = node.mutate(mutation)?;
```

## Technical Requirements

### Dependencies
- Core FoldDB library
- Serde for serialization
- Standard Rust async runtime

### Constraints
- All operations must maintain atomic guarantees
- Schema must be loaded before any data operations
- Permissions must be validated for all operations
- Trust distance rules must be enforced
- Version history must be maintained

## Error Handling
- Clear error types for different failure scenarios
- Proper propagation of underlying FoldDB errors
- Validation errors for schema and data operations
- Permission-related error cases
- Configuration and initialization errors

## Testing Strategy
- Unit tests for all public methods
- Integration tests for full workflows
- Permission validation tests
- Error handling tests
- Schema validation tests
- Performance benchmarks

## Docker Deployment

### Overview
The DataFold Node is designed to run within a Docker container with restricted network access. This containerization provides:
- Isolation of the node's runtime environment
- Network access control for security
- Consistent deployment across different environments
- API exposure through controlled ports

### Docker Configuration

#### Base Image
```dockerfile
FROM rust:1.70-slim
WORKDIR /app
COPY . .
RUN cargo build --release
```

#### Network Restrictions
- Only expose the DataFold Node API port (default: 8080)
- Block all outbound connections by default
- Allow specific outbound connections through explicit configuration

#### Security Considerations
- Run container as non-root user
- Read-only filesystem where possible
- Drop unnecessary capabilities
- Resource limits enforcement

### Example Docker Configuration
```dockerfile
# Build stage
FROM rust:1.70-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim
WORKDIR /app
COPY --from=builder /app/target/release/datafold_node .

# Create non-root user
RUN useradd -r -u 1001 -g root datafold
USER datafold

# Configure volumes
VOLUME ["/app/data"]

# Expose API port only
EXPOSE 8080

# Set resource limits
ENV MEMORY_LIMIT=1g
ENV CPU_LIMIT=1.0

ENTRYPOINT ["./datafold_node"]
```

### Docker Compose Example
```yaml
version: '3.8'
services:
  datafold_node:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - ./data:/app/data
    networks:
      - datafold_net
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    cap_add:
      - NET_BIND_SERVICE
    read_only: true
    tmpfs:
      - /tmp
    environment:
      - NODE_CONFIG=/app/config/node_config.json
    deploy:
      resources:
        limits:
          cpus: '1.0'
          memory: 1G

networks:
  datafold_net:
    driver: bridge
    internal: true  # Prevents outbound internet access
```

### Network Security Rules
1. Inbound Traffic:
   - Allow port 8080 for API access
   - Block all other inbound ports

2. Outbound Traffic:
   - Block all outbound internet access by default
   - Allow specific outbound connections through explicit configuration
   - Use internal Docker network for inter-container communication

### API Access
- The DataFold Node API is exposed on port 8080
- API endpoints follow RESTful conventions
- Authentication required for all API access
- HTTPS recommended for production deployments

### Deployment Recommendations
1. Use multi-stage builds to minimize image size
2. Implement health checks for container orchestration
3. Configure logging to external collectors
4. Use secrets management for sensitive configuration
5. Implement rate limiting for API endpoints
6. Regular security updates for base images

### Monitoring
- Container health metrics
- API endpoint metrics
- Resource usage monitoring
- Network traffic monitoring
- Error rate tracking
