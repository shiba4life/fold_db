# FoldDB Technical Whitepaper

## Executive Summary

FoldDB is an innovative database system designed to address the growing need for fine-grained data access control, immutable versioning, and schema-based validation. Built with Rust for performance and safety, FoldDB introduces a unique architecture that combines atomic operations, field-level permissions, and trust-based access control to create a secure and flexible data management solution.

This whitepaper presents the technical architecture, core concepts, implementation details, and future directions of the FoldDB project. It is intended for developers, system architects, and technical decision-makers interested in understanding the capabilities and potential applications of this database system.

## 1. Introduction

### 1.1 Background

Modern applications face increasing challenges in data management, including:
- Ensuring data integrity across distributed systems
- Implementing fine-grained access control
- Maintaining audit trails and version history
- Supporting schema evolution and transformation
- Enabling secure third-party integrations

Traditional database systems often require complex application-level code to address these challenges, leading to increased development time, potential security vulnerabilities, and maintenance overhead.

### 1.2 FoldDB Solution

FoldDB addresses these challenges through a novel architecture that incorporates:
- Immutable data storage with atomic operations
- Schema-based validation and transformation
- Field-level permission control
- Trust-based access model
- Version history tracking
- Secure sandbox environment for third-party applications

By integrating these features at the database level, FoldDB significantly reduces application complexity while enhancing security, data integrity, and flexibility.

## 2. Core Architecture

### 2.1 System Overview

FoldDB follows a modular design with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────┐
│                         FoldDB Core                         │
├─────────────┬─────────────┬─────────────┬─────────────┬─────┘
│ Schema      │ Permission  │ Atom        │ Payment     │
│ Management  │ System      │ Storage     │ System      │
├─────────────┼─────────────┼─────────────┼─────────────┤
│ Schema      │ Permission  │ Atom        │ Lightning   │
│ Interpreter │ Manager     │ Manager     │ Integration │
├─────────────┼─────────────┼─────────────┼─────────────┤
│ Field       │ Trust       │ Version     │ Payment     │
│ Mapping     │ Calculator  │ Control     │ Calculator  │
└─────────────┴─────────────┴─────────────┴─────────────┘
        ▲             ▲             ▲             ▲
        │             │             │             │
┌───────┴─────────────┴─────────────┴─────────────┴───────┐
│                    DataFold Node                        │
├─────────────┬─────────────┬─────────────┬──────────────┤
│ Network     │ Web         │ Sandbox     │ App          │
│ Layer       │ Server      │ Environment │ System       │
└─────────────┴─────────────┴─────────────┴──────────────┘
```

### 2.2 Key Components

#### 2.2.1 FoldDB Core

The central coordinator for all database operations, managing interactions between:
- SchemaManager: Handles schema validation and management
- PermissionManager: Controls access and trust calculations
- AtomManager: Manages immutable data storage and versioning
- PaymentManager: Handles Lightning Network integration for micropayments

#### 2.2.2 DataFold Node

Provides network connectivity, API access, and application management:
- NetworkLayer: Handles node discovery and peer-to-peer communication
- WebServer: Exposes REST and WebSocket APIs
- SandboxEnvironment: Manages secure Docker containers for third-party applications
- AppSystem: Manages application lifecycle and resources

### 2.3 Data Model

#### 2.3.1 Atoms & AtomRefs

The fundamental data storage units in FoldDB:

- **Atoms**: Immutable data containers with:
  - Content: The actual data payload
  - Metadata: Information about the data
  - Schema reference: Link to the schema definition
  - Previous version: Link to the previous version of this data
  - Timestamp: Creation time for auditing
  - Signature: Cryptographic signature for verification

- **AtomRefs**: Mutable references that:
  - Always point to the latest version of an Atom
  - Enable atomic updates through reference switching
  - Provide indirection for version management
  - Track update timestamps

#### 2.3.2 Schema System

Defines the structure and validation rules for data:

- **Schema**: Contains collection of SchemaFields
- **SchemaField**: Defines individual field properties, including:
  - Data type and constraints
  - Permission policies
  - Payment configurations
  - Field mappings to other schemas

#### 2.3.3 Permissions Model

Controls access to data through:

- **Trust Distance**: Lower numbers indicate higher trust levels
- **Field-Level Control**: Permissions set for individual data fields
- **Access Policies**: Explicit read/write permissions using public key authentication
- **PermissionWrapper**: Wraps data with permission checks

## 3. Technical Implementation

### 3.1 Core Components Implementation

#### 3.1.1 SchemaCore

```rust
pub struct SchemaCore {
    schemas: HashMap<String, Schema>,
    field_mappings: HashMap<Uuid, FieldMapping>,
    schema_relationships: HashMap<Uuid, Vec<SchemaRelationship>>,
    persistence: SchemaPersistence,
}

impl SchemaCore {
    pub fn new(persistence: SchemaPersistence) -> Self;
    pub fn load_schema(&mut self, name: &str, schema_json: &str) -> Result<Uuid, SchemaError>;
    pub fn get_schema(&self, name: &str) -> Option<&Schema>;
    pub fn validate_data(&self, schema_name: &str, data: &Value) -> Result<(), ValidationError>;
    pub fn transform_data(&self, source_schema: &str, target_schema: &str, data: &Value) -> Result<Value, TransformError>;
}
```

#### 3.1.2 NetworkCore

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
}
```

#### 3.1.3 PermissionManager

```rust
pub struct PermissionManager {
    policies: HashMap<Uuid, PermissionPolicy>,
    trust_calculator: TrustCalculator,
}

impl PermissionManager {
    pub fn new() -> Self;
    pub fn add_policy(&mut self, policy: PermissionPolicy) -> Uuid;
    pub fn check_permission(&self, policy_id: &Uuid, public_key: &str, operation: Operation) -> Result<bool, PermissionError>;
    pub fn calculate_trust_distance(&self, source_key: &str, target_key: &str) -> Result<u32, TrustError>;
    pub fn wrap_data<T>(&self, data: T, policy_id: &Uuid) -> PermissionWrapper<T>;
}
```

#### 3.1.4 PaymentManager

```rust
pub struct PaymentManager {
    lightning_client: Box<dyn LightningClient>,
    calculator: PaymentCalculator,
    config: PaymentConfig,
}

impl PaymentManager {
    pub fn new(lightning_client: Box<dyn LightningClient>, config: PaymentConfig) -> Self;
    pub fn calculate_payment(&self, schema_name: &str, fields: &[String], trust_distance: u32) -> Result<u64, PaymentError>;
    pub fn create_invoice(&self, amount: u64, description: &str) -> Result<Invoice, PaymentError>;
    pub fn verify_payment(&self, invoice_id: &str) -> Result<PaymentStatus, PaymentError>;
    pub fn create_hold_invoice(&self, amount: u64, description: &str) -> Result<HoldInvoice, PaymentError>;
    pub fn settle_hold_invoice(&self, invoice_id: &str) -> Result<(), PaymentError>;
    pub fn cancel_hold_invoice(&self, invoice_id: &str) -> Result<(), PaymentError>;
}
```

### 3.2 DataFold Node Implementation

#### 3.2.1 SandboxManager

```rust
pub struct SandboxManager {
    config: SandboxConfig,
    containers: HashMap<String, ContainerInfo>,
    network: DockerNetwork,
}

impl SandboxManager {
    pub fn new(config: SandboxConfig) -> Result<Self, SandboxError>;
    pub fn register_container(&mut self, id: &str, name: &str, image: &str, resource_limits: Option<ResourceLimits>) -> Result<(), SandboxError>;
    pub fn start_container(&mut self, id: &str) -> Result<(), SandboxError>;
    pub fn stop_container(&mut self, id: &str) -> Result<(), SandboxError>;
    pub fn proxy_request(&self, request: Request) -> Result<Response, SandboxError>;
}
```

#### 3.2.2 AppRegistry

```rust
pub struct AppRegistry {
    apps: HashMap<String, RegisteredApp>,
    resource_manager: Arc<Mutex<AppResourceManager>>,
    api_manager: Arc<Mutex<ApiManager>>,
}

impl AppRegistry {
    pub fn new(resource_manager: Arc<Mutex<AppResourceManager>>, api_manager: Arc<Mutex<ApiManager>>) -> Self;
    pub fn register_app(&mut self, manifest: AppManifest) -> Result<(), AppError>;
    pub fn unregister_app(&mut self, app_name: &str) -> Result<(), AppError>;
    pub fn start_app(&mut self, app_name: &str) -> Result<(), AppError>;
    pub fn stop_app(&mut self, app_name: &str) -> Result<(), AppError>;
    pub fn list_apps(&self) -> Vec<String>;
}
```

### 3.3 Schema System Implementation

#### 3.3.1 Schema Definition

```json
{
  "name": "user-profile",
  "fields": {
    "username": {
      "type": "string",
      "unique": true,
      "permissions": {
        "read": {
          "trust_distance": 2,
          "explicit_keys": ["key1", "key2"]
        },
        "write": {
          "trust_distance": 1,
          "explicit_keys": ["key1"]
        }
      },
      "payment": {
        "base_amount": 10,
        "trust_scaling_factor": 2.0
      }
    },
    "email": {
      "type": "string",
      "permissions": {
        "read": {
          "trust_distance": 1,
          "explicit_keys": []
        },
        "write": {
          "trust_distance": 0,
          "explicit_keys": []
        }
      },
      "payment": {
        "base_amount": 50,
        "trust_scaling_factor": 3.0
      }
    },
    "friends": {
      "type": "array",
      "items": {
        "type": "ref",
        "schema": "user-profile"
      },
      "permissions": {
        "read": {
          "trust_distance": 3,
          "explicit_keys": []
        },
        "write": {
          "trust_distance": 1,
          "explicit_keys": []
        }
      }
    }
  }
}
```

#### 3.3.2 Field Mapping

```json
{
  "mapping": {
    "source_schema": "user-profile-v1",
    "target_schema": "user-profile-v2",
    "fields": {
      "name": {
        "target_field": "username",
        "transformation": "direct"
      },
      "email": {
        "target_field": "email",
        "transformation": "direct"
      },
      "contact.phone": {
        "target_field": "phone_number",
        "transformation": "format",
        "format_string": "+{}"
      }
    }
  }
}
```

### 3.4 Network Layer Implementation

#### 3.4.1 Node Discovery

```rust
pub struct NodeDiscovery {
    discovery_method: DiscoveryMethod,
    socket: UdpSocket,
    known_nodes: HashSet<NodeId>,
}

impl NodeDiscovery {
    pub fn new(config: DiscoveryConfig) -> Result<Self, NetworkError>;
    pub fn find_nodes(&mut self) -> Result<Vec<NodeInfo>, NetworkError>;
    pub fn announce_presence(&self) -> Result<(), NetworkError>;
    pub fn handle_node_announcement(&mut self, announcement: NodeAnnouncement) -> Result<(), NetworkError>;
}
```

#### 3.4.2 Message Protocol

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
```

### 3.5 App System Implementation

#### 3.5.1 App Manifest

```json
{
  "name": "social-app",
  "version": "1.0.0",
  "description": "Social networking app for Datafold",
  "entry": "/apps/social-app/index.html",
  "schemas": [
    "user-profile",
    "post",
    "comment"
  ],
  "window": {
    "defaultSize": { "width": 800, "height": 600 },
    "minSize": { "width": 400, "height": 300 },
    "title": "Social App",
    "resizable": true
  },
  "permissions": {
    "required": [
      "read:profiles",
      "write:posts"
    ]
  },
  "apis": {
    "required": ["data", "users"],
    "optional": ["notifications"]
  }
}
```

#### 3.5.2 App Communication

```javascript
// App A: Sending a message
class AppA {
    sendMessage(targetApp, messageType, data) {
        window.parent.postMessage({
            type: 'app-message',
            target: targetApp,
            messageType: messageType,
            data: data
        }, '*');
    }
    
    broadcastMessage(messageType, data) {
        window.parent.postMessage({
            type: 'app-message',
            messageType: messageType,
            data: data
        }, '*');
    }
}

// App B: Receiving messages
class AppB {
    constructor() {
        window.addEventListener('message', this.handleMessage.bind(this));
    }
    
    handleMessage(event) {
        if (event.data.type === 'app-message') {
            const { messageType, data } = event.data;
            
            switch (messageType) {
                case 'content:shared':
                    this.handleSharedContent(data);
                    break;
                case 'user:action':
                    this.handleUserAction(data);
                    break;
            }
        }
    }
}
```

## 4. Security Model

### 4.1 Trust-Based Access Control

FoldDB implements a trust-based access control system where:

1. Each entity has a public key identity
2. Trust relationships are established between entities
3. Trust distance determines access level (lower = higher trust)
4. Field-level permissions specify required trust distance
5. Explicit permissions can override trust distance requirements

```
┌────────────┐     Trust Distance = 1     ┌────────────┐
│            │◄────────────────────────────│            │
│  Entity A  │                            │  Entity B  │
│            │                            │            │
└────────────┘                            └────────────┘
       ▲                                         ▲
       │                                         │
       │ Trust Distance = 2                      │ Trust Distance = 1
       │                                         │
       │                                         │
┌──────┴───────┐                          ┌──────┴───────┐
│              │                          │              │
│   Entity C   │                          │   Entity D   │
│              │                          │              │
└──────────────┘                          └──────────────┘
```

In this example:
- Entity B has trust distance 1 from Entity A
- Entity C has trust distance 2 from Entity A
- Entity D has trust distance 1 from Entity B and 2 from Entity A (transitive)

### 4.2 Sandbox Security

The sandbox environment implements multiple layers of security:

1. **Network Isolation**: Internal Docker network with no external access
2. **Capability Restrictions**: All Linux capabilities dropped
3. **Privilege Escalation Prevention**: No-new-privileges flag
4. **Resource Limits**: CPU, memory, and process limits
5. **Read-Only Filesystem**: Containers run with read-only root filesystem
6. **Unix Socket Communication**: Optional socket-based communication for maximum isolation

### 4.3 Payment Verification

The payment system ensures:

1. Payment requirements are calculated based on:
   - Field-specific base amounts
   - Trust distance scaling factors
   - Operation complexity

2. Payments are verified through:
   - Lightning Network integration
   - Hold invoices for complex operations
   - Payment verification before operation execution

## 5. Performance Considerations

### 5.1 Concurrency Model

FoldDB is designed for concurrent operation with:

- Thread-safe components using Arc and Mutex
- Atomic operations for data changes
- Lock-free read operations where possible
- Optimistic concurrency control for writes

### 5.2 Caching Strategy

Performance is optimized through:

- Schema caching for frequent operations
- Connection pooling in the network layer
- Query result caching with invalidation
- Atom reference caching for fast lookups

### 5.3 Resource Management

System resources are managed through:

- Configurable resource limits for applications
- Memory usage optimization for large datasets
- Connection timeouts and cleanup
- Background garbage collection for old versions

## 6. Future Enhancements

### 6.1 AI Integration Opportunities

FoldDB has identified several AI integration opportunities:

#### 6.1.1 Natural Language Query Interface

An AI layer for natural language interaction:
- Transform natural language into schema-compliant queries
- Provide intelligent query suggestions based on context
- Help users discover data relationships through conversation
- Create sophisticated queries from simple descriptions

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│                 │    │                 │    │                 │
│  User Request   │───►│ NLQueryProcessor│───►│SchemaInterpreter│
│                 │    │                 │    │                 │
└─────────────────┘    └────────┬────────┘    └────────┬────────┘
                               │                       │
                               ▼                       ▼
                       ┌─────────────────┐    ┌─────────────────┐
                       │                 │    │                 │
                       │  Query Engine   │◄───┤   FoldDB Core   │
                       │                 │    │                 │
                       └─────────────────┘    └─────────────────┘
```

#### 6.1.2 Schema Evolution Intelligence

AI-powered schema version management:
- Generate optimal schema transformations
- Predict potential issues in schema changes
- Generate migration scripts automatically
- Learn from historical schema changes

#### 6.1.3 Smart Payment Optimization

Machine learning for dynamic pricing:
- Analyze usage patterns to optimize pricing
- Identify potential payment defaults
- Adjust trust distance calculations based on behavior
- Dynamically adjust hold invoice durations

### 6.2 Planned Technical Improvements

#### 6.2.1 Enhanced Sandbox Environment

- Fine-grained permission system for API access
- Audit logging for all API requests
- Resource usage monitoring and reporting
- Automatic container cleanup

#### 6.2.2 API Layer Enhancements

- GraphQL support
- WebSocket support for real-time updates
- Batch operations
- Query caching

#### 6.2.3 Network Layer Improvements

- DHT-based discovery
- NAT traversal
- Connection multiplexing
- Protocol versioning
- Node reputation tracking

## 7. Conclusion

FoldDB represents a significant advancement in database technology, addressing critical needs for data integrity, access control, and version management. By integrating these features at the database level, FoldDB reduces application complexity while enhancing security and flexibility.

The modular architecture, immutable data model, and innovative trust-based access control system provide a solid foundation for building secure, scalable applications. The sandbox environment and app system extend these capabilities to support third-party integrations while maintaining security boundaries.

Future enhancements, particularly in AI integration, promise to further improve usability, performance, and capabilities, making FoldDB a powerful platform for modern application development.

## Appendix A: Technical Specifications

### A.1 System Requirements

- **Operating System**: Linux, macOS, Windows
- **Memory**: 4GB minimum, 8GB recommended
- **Storage**: SSD recommended for optimal performance
- **Processor**: Multi-core CPU recommended for concurrent operations
- **Dependencies**: Rust toolchain, Docker (for sandbox environment)

### A.2 Performance Metrics

- **Query Performance**: 1000+ simple queries per second on standard hardware
- **Write Performance**: 500+ writes per second
- **Concurrency**: Supports 100+ concurrent connections
- **Latency**: Sub-millisecond for cached operations, 1-10ms for typical operations

### A.3 Security Compliance

- **Authentication**: Public key based authentication
- **Authorization**: Field-level permission control
- **Audit**: Complete version history for all changes
- **Isolation**: Secure sandbox environment for third-party code

## Appendix B: API Reference

### B.1 Core API

```rust
// Initialize database
let db = FoldDB::new("path/to/db")?;

// Load schema
let schema = Schema::from_json(schema_json)?;
db.load_schema("user_profile", schema)?;

// Write data
let data = json!({
    "name": "Alice",
    "email": "alice@example.com"
});
db.write("user_profile", data, public_key)?;

// Read data
let user = db.read("user_profile", "user123")?;

// Query data
let query = Query::new("user_profile")
    .filter("name", "Alice")
    .select(&["name", "email"]);
let results = db.query(query, public_key)?;

// Get version history
let history = db.get_history("user_profile", "user123")?;
```

### B.2 Network API

```rust
// Initialize network
let network = NetworkManager::new(config, node_id, public_key)?;

// Discover nodes
let nodes = network.discover_nodes()?;

// Connect to node
network.connect_to_node(&node_id)?;

// Query remote node
let query = Query::new("user_profile")
    .filter("name", "Alice")
    .select(&["name", "email"]);
let results = network.query_node(&node_id, query)?;

// List available schemas
let schemas = network.list_available_schemas(&node_id)?;
```

### B.3 Sandbox API

```rust
// Create sandbox manager
let sandbox = SandboxManager::new(config)?;

// Register container
sandbox.register_container("app1", "My App", "app-image:latest", None)?;

// Start container
sandbox.start_container("app1")?;

// Proxy request
let request = Request {
    container_id: "app1".to_string(),
    path: "/query".to_string(),
    method: "GET".to_string(),
    headers: HashMap::new(),
    body: None,
};
let response = sandbox.proxy_request(request)?;
```

## Appendix C: License

FoldDB is released under the MIT License, one of the most permissive open-source licenses available.

```
MIT License

Copyright (c) 2025 FoldDB Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
