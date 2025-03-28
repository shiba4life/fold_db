# Sandboxed Social App Architecture for DataFold

This document outlines a proposed architecture for social applications to interact with DataFold nodes while maintaining security, privacy, and the distributed nature of the network. This architecture allows social apps to run queries and mutations on both the local node and remote nodes in the network, with strict network isolation to prevent direct internet access.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      Social App Container                   │
│                                                             │
│  ┌─────────────┐    ┌────────────────┐    ┌──────────────┐  │
│  │ Social App  │    │ Permission     │    │ Network      │  │
│  │ Frontend    │◄───┤ Proxy Layer    │◄───┤ Query Router │  │
│  └─────────────┘    └────────────────┘    └──────────────┘  │
│         ▲                    ▲                   ▲          │
└─────────┼────────────────────┼───────────────────┼──────────┘
          │                    │                   │
          │                    │                   │
┌─────────┼────────────────────┼───────────────────┼──────────┐
│         │                    │                   │          │
│  ┌─────────────┐    ┌────────────────┐    ┌──────────────┐  │
│  │ App API     │    │ Permission     │    │ Network      │  │
│  │ Gateway     │    │ Validator      │    │ Bridge       │  │
│  └─────────────┘    └────────────────┘    └──────────────┘  │
│                                                             │
│                       DataFold Node                         │
└─────────────────────────────────────────────────────────────┘
                             │
                             ▼
                   ┌───────────────────┐
                   │  DataFold Network │
                   │  (Remote Nodes)   │
                   └───────────────────┘
```

## Network Isolation

The most critical security aspect of this architecture is complete network isolation for the social app container:

### 1. Container-Level Network Isolation

```rust
pub struct SocialAppContainer {
    // Unique identifier for this app instance
    app_id: String,
    // Public key for authentication
    app_public_key: String,
    // Permissions granted to this app
    permissions: AppPermissions,
    // Connection to the DataFold node (ONLY communication channel)
    node_connection: NodeConnection,
    // Network namespace configuration
    network_isolation: NetworkIsolation,
}

pub struct NetworkIsolation {
    // Container uses a separate network namespace
    isolated_namespace: bool,
    // No direct network interfaces except loopback
    network_interfaces: Vec<String>, // Only "lo" allowed
    // Firewall rules block all outbound connections
    firewall_rules: Vec<FirewallRule>,
}
```

### 2. Implementation Approaches

#### MicroVM Solutions (Preferred)

```rust
pub struct MicroVMConfig {
    // VM type (Firecracker, Kata Containers, etc.)
    vm_type: MicroVMType,
    // VM resources
    vcpu_count: u32,
    memory_mb: u32,
    // Network configuration - only allow communication with host
    network_config: NetworkConfig,
    // Vsock for communication with host
    vsock_config: VsockConfig,
    // Kernel parameters to restrict networking
    kernel_params: Vec<String>,
}

pub enum MicroVMType {
    Firecracker,
    KataContainers,
    Cloud9VM,
    Weave,
}
```

Firecracker and Kata Containers offer several advantages for this use case:

1. **Strong Isolation**: MicroVMs provide hardware-level isolation through KVM, offering stronger security boundaries than containers.

2. **Minimal Attack Surface**: Firecracker has a minimal footprint (~5MB memory overhead per VM) and a reduced attack surface.

3. **Fast Startup**: Firecracker VMs can boot in as little as 125ms, making them suitable for on-demand app launching.

4. **Resource Efficiency**: Despite being VMs, they have minimal overhead compared to traditional virtualization.

5. **Network Isolation**: Complete network isolation with precise control over virtual interfaces.

Implementation example with Firecracker:

```rust
pub fn create_firecracker_vm(app_id: &str, app_binary: &str) -> Result<VMID, VMError> {
    // Create VM configuration
    let vm_config = FirecrackerConfig {
        vcpu_count: 1,
        mem_size_mib: 128,
        // Only create a vsock device for communication with host
        // No network devices at all
        vsock_device: Some(VsockDeviceConfig {
            vsock_id: format!("vsock-{}", app_id),
            guest_cid: 3,
            uds_path: format!("/tmp/firecracker-{}.sock", app_id),
        }),
        network_interfaces: vec![], // No network interfaces
        kernel_args: "console=ttyS0 reboot=k panic=1 pci=off nomodules ip=none".to_string(),
    };
    
    // Launch VM
    let vm_id = launch_firecracker_vm(vm_config, app_binary)?;
    
    Ok(vm_id)
}
```

#### Linux Containers (Alternative)

```rust
pub struct LinuxContainerConfig {
    // Use network namespace isolation
    network_namespace: bool,
    // Drop all network capabilities except local communication
    dropped_capabilities: Vec<String>, // NET_ADMIN, NET_RAW, NET_BROADCAST, etc.
    // Only allow communication with the DataFold node socket
    allowed_sockets: Vec<SocketPath>,
    // Seccomp filters to block network-related syscalls
    seccomp_filters: Vec<SeccompRule>,
}
```

#### WebAssembly Sandbox

```rust
pub struct WasmSandboxConfig {
    // No network access in WASM imports
    network_imports: Vec<String>, // Empty list
    // Only allow communication through provided DataFold API
    allowed_imports: Vec<String>, // Only DataFold API functions
    // Memory isolation
    memory_isolation: bool,
}
```

## Key Components

### 1. Social App Container

A strictly isolated environment where the social application runs with no direct internet access:

```rust
pub struct SocialAppContainer {
    // Unique identifier for this app instance
    app_id: String,
    // Public key for authentication
    app_public_key: String,
    // Permissions granted to this app
    permissions: AppPermissions,
    // Connection to the DataFold node (ONLY communication channel)
    node_connection: NodeConnection,
    // Network isolation configuration
    network_isolation: NetworkIsolation,
    // Resource limits
    resource_limits: ResourceLimits,
}

// Resource limits to prevent abuse
pub struct ResourceLimits {
    // Maximum CPU usage percentage
    max_cpu_percent: f32,
    // Maximum memory usage in MB
    max_memory_mb: u32,
    // Maximum storage in MB
    max_storage_mb: u32,
    // Maximum number of concurrent operations
    max_concurrent_ops: u32,
}
```

### 2. App API Gateway

Serves as the only entry/exit point for app communication:

```rust
pub struct AppApiGateway {
    // Reference to the DataFold node
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
    // Map of registered apps and their permissions
    registered_apps: HashMap<String, AppPermissions>,
    // Rate limiting configuration
    rate_limits: RateLimitConfig,
    // Communication channel with app containers
    app_channels: HashMap<String, AppChannel>,
}

impl AppApiGateway {
    // Register a new app with specific permissions
    pub fn register_app(&mut self, app_id: &str, permissions: AppPermissions) -> Result<(), AppError>;
    
    // Process an operation request from an app
    pub async fn process_app_request(&self, request: AppRequest) -> Result<Value, AppError>;
    
    // Launch an app in an isolated container
    pub async fn launch_app_container(&mut self, app_id: &str, 
                                     app_binary_path: &str) -> Result<(), AppError>;
                                     
    // Terminate an app container
    pub async fn terminate_app_container(&mut self, app_id: &str) -> Result<(), AppError>;
}
```

### 3. Permission Validator

Enforces fine-grained permissions for app operations:

```rust
pub struct PermissionValidator {
    // Default trust distance for apps
    default_trust_distance: u32,
    // Schema-specific permission rules
    schema_permissions: HashMap<String, SchemaPermissions>,
}

impl PermissionValidator {
    // Validate if an app has permission to perform an operation
    pub fn validate_operation(&self, app_id: &str, operation: &Operation, 
                             app_permissions: &AppPermissions) -> Result<(), PermissionError>;
    
    // Validate if an app has permission to access a remote node
    pub fn validate_remote_access(&self, app_id: &str, remote_node_id: &str, 
                                 app_permissions: &AppPermissions) -> Result<(), PermissionError>;
}
```

### 4. Network Bridge

The only component that can communicate with remote nodes:

```rust
pub struct NetworkBridge {
    // Reference to the DataFold node's network core
    network: Arc<tokio::sync::Mutex<NetworkCore>>,
    // Map of app permissions for remote access
    app_remote_permissions: HashMap<String, HashMap<String, RemotePermission>>,
    // Audit logging for all network operations
    network_audit_log: AuditLogger,
}

impl NetworkBridge {
    // Forward a request to a remote node
    pub async fn forward_request(&self, app_id: &str, remote_node_id: &str, 
                                operation: Operation) -> Result<Value, NetworkError>;
    
    // Discover available nodes for an app
    pub async fn discover_available_nodes(&self, app_id: &str) -> Result<Vec<NodeInfo>, NetworkError>;
}
```

### 5. Network Query Router (App-side)

Routes queries to the appropriate node through the DataFold node only:

```rust
pub struct NetworkQueryRouter {
    // Connection to the local node (ONLY communication channel)
    local_connection: NodeConnection,
    // App's public key for authentication
    app_public_key: String,
}

impl NetworkQueryRouter {
    // Route an operation to the appropriate node through the DataFold node
    pub async fn route_operation(&self, operation: Operation, 
                               target_node_id: Option<String>) -> Result<Value, RoutingError>;
    
    // Discover available nodes and their schemas through the DataFold node
    pub async fn discover_nodes(&self) -> Result<Vec<RemoteNodeInfo>, RoutingError>;
}
```

## Implementation Strategy

### 1. Isolation Mechanisms

#### MicroVM with Firecracker

1. Create a minimal VM with Firecracker:
   - No network interfaces
   - Vsock for communication with host
   - Minimal kernel with networking disabled
   - Minimal root filesystem with only the app and dependencies

```rust
pub fn create_firecracker_vm(app_id: &str, app_binary: &str) -> Result<VMID, VMError> {
    // Create VM configuration
    let vm_config = FirecrackerConfig {
        vcpu_count: 1,
        mem_size_mib: 128,
        // Only create a vsock device for communication with host
        vsock_device: Some(VsockDeviceConfig {
            vsock_id: format!("vsock-{}", app_id),
            guest_cid: 3,
            uds_path: format!("/tmp/firecracker-{}.sock", app_id),
        }),
        network_interfaces: vec![], // No network interfaces
        kernel_args: "console=ttyS0 reboot=k panic=1 pci=off nomodules ip=none".to_string(),
        root_drive: RootDrive {
            path_on_host: format!("/var/lib/datafold/vm-images/minimal-rootfs.ext4"),
            is_read_only: true,
        },
        // Additional drive for app binary and data
        drives: vec![
            Drive {
                path_on_host: format!("/var/lib/datafold/apps/{}/drive.ext4", app_id),
                is_read_only: false,
                is_root_device: false,
            }
        ],
    };
    
    // Launch VM
    let vm_id = launch_firecracker_vm(vm_config, app_binary)?;
    
    Ok(vm_id)
}
```

#### MicroVM with Kata Containers

For integration with container ecosystems:

```rust
pub fn create_kata_container(app_id: &str, app_binary: &str) -> Result<ContainerId, ContainerError> {
    // Create container with Kata Containers runtime
    let container_config = ContainerConfig {
        image: "datafold-app-base:latest",
        runtime: "kata-runtime", // Use Kata Containers runtime
        network_mode: "none", // No network access
        annotations: {
            "io.kata-containers.config.hypervisor.disable_nesting_checks": "true",
            "io.kata-containers.config.hypervisor.kernel_params": "ip=none nomodules",
            "io.kata-containers.config.agent.enable_vsock": "true",
        },
        mounts: [
            // Mount app binary
            Mount { source: app_binary, target: "/app/binary", read_only: true },
            // Mount communication socket via vsock
            Mount { source: format!("/var/run/datafold/app_{}.sock", app_id), 
                   target: "/var/run/datafold/node.sock" },
        ],
    };
    
    // Launch container with Kata runtime
    let container_id = launch_container(container_config)?;
    
    Ok(container_id)
}
```

#### Linux Containers (Alternative)

1. Create a custom container profile with:
   - Network namespace isolation
   - No network interfaces except loopback
   - Dropped network capabilities
   - Seccomp filters to block network syscalls
   - Restricted filesystem access

```rust
pub fn create_isolated_container(app_id: &str, app_binary: &str) -> Result<ContainerId, ContainerError> {
    // Create container with network isolation
    let container_config = ContainerConfig {
        image: "datafold-app-base:latest",
        network_mode: "none", // No network access
        capabilities: {
            drop: ["NET_ADMIN", "NET_RAW", "NET_BROADCAST"],
            add: [],
        },
        mounts: [
            // Mount app binary
            Mount { source: app_binary, target: "/app/binary", read_only: true },
            // Mount communication socket
            Mount { source: format!("/var/run/datafold/app_{}.sock", app_id), 
                   target: "/var/run/datafold/node.sock" },
        ],
        seccomp_profile: "app_seccomp_profile.json", // Blocks network syscalls
    };
    
    // Launch container
    let container_id = launch_container(container_config)?;
    
    Ok(container_id)
}
```

#### WebAssembly Sandbox

For web-based or lightweight applications:

```rust
pub fn create_wasm_sandbox(app_id: &str, wasm_module: &[u8]) -> Result<WasmInstance, WasmError> {
    // Create WASM instance with no network imports
    let allowed_imports = WasmImports {
        // Only DataFold API functions
        datafold: vec![
            "query_local", "query_remote", "mutate_local", "mutate_remote",
            "discover_nodes", "discover_schemas"
        ],
        // No network functions
        network: vec![],
        // No filesystem access except for allowed paths
        filesystem: vec!["read_app_data", "write_app_data"],
    };
    
    // Create sandbox
    let instance = WasmSandbox::new(wasm_module, allowed_imports)?;
    
    Ok(instance)
}
```

### 2. App Registration and Permissions

Apps register with the DataFold node, receiving a unique app ID and public/private key pair:

```rust
// App permissions definition
pub struct AppPermissions {
    // Schemas the app can access
    allowed_schemas: HashSet<String>,
    // Fields the app can read/write per schema
    field_permissions: HashMap<String, FieldPermissions>,
    // Remote nodes the app can access
    allowed_remote_nodes: HashSet<String>,
    // Maximum trust distance for queries
    max_trust_distance: u32,
    // Rate limits for operations
    rate_limits: OperationRateLimits,
}

// Field-level permissions
pub struct FieldPermissions {
    readable_fields: HashSet<String>,
    writable_fields: HashSet<String>,
}
```

### 3. Secure Communication Protocol

All communication between the app container and DataFold node uses signed requests through a local socket only:

```rust
// Enhanced signed request with target node information
pub struct AppRequest {
    // App identifier
    app_id: String,
    // Unix timestamp
    timestamp: u64,
    // Target node (None for local node)
    target_node_id: Option<String>,
    // Operation to perform
    operation: Operation,
    // Signature using app's private key
    signature: String,
}

// Communication channel between app and node
pub enum AppChannel {
    // Unix socket (preferred for security)
    UnixSocket(String),
    // Shared memory region
    SharedMemory(SharedMemoryRegion),
    // Named pipe (for Windows)
    NamedPipe(String),
}
```

### 4. Permission Enforcement

The Permission Validator enforces rules at multiple levels:

1. Schema-level permissions (can the app access this schema?)
2. Field-level permissions (can the app read/write these fields?)
3. Operation-level permissions (can the app perform this mutation?)
4. Remote access permissions (can the app access this remote node?)

### 5. Network Query Routing

The Network Bridge handles routing queries to remote nodes:

1. App sends operation with optional target_node_id through the local socket only
2. If target_node_id is None, operation is performed on local node
3. If target_node_id is specified, Permission Validator checks remote access permissions
4. If permitted, Network Bridge forwards the request to the remote node
5. Remote node applies its own permission checks before processing

## Security Considerations

1. **Complete Network Isolation**: Apps have no direct internet access and can only communicate through the DataFold node.

2. **Strong Isolation Boundaries**: Multiple layers of isolation prevent apps from bypassing restrictions:
   - Hardware-level isolation with MicroVMs (Firecracker/Kata Containers)
   - Network namespace isolation
   - Dropped network capabilities
   - Seccomp filters to block network syscalls
   - Restricted filesystem access

3. **Cryptographic Authentication**: All requests are signed using the app's private key.

4. **Fine-grained Permissions**: Permissions are enforced at schema, field, and operation levels.

5. **Rate Limiting**: Prevents abuse by limiting the frequency of operations.

6. **Resource Limits**: CPU, memory, and storage limits prevent resource exhaustion.

7. **Audit Logging**: All operations are logged for security auditing.

## Benefits

1. **Decentralized Social Applications**: Apps can interact with multiple nodes in the network, enabling truly decentralized social experiences.

2. **Privacy Control**: Users maintain control over their data through fine-grained permissions.

3. **Network Security**: Apps cannot make unauthorized network requests or access the internet directly.

4. **Developer Flexibility**: Developers can build rich social experiences while respecting user privacy and security constraints.

5. **Network Scalability**: The architecture supports distributed data access across the network.

## Developer SDK

To simplify the development of social applications for the DataFold network, a comprehensive SDK would be essential. This SDK would abstract away the complexities of the sandboxed environment and provide developers with intuitive APIs for interacting with the DataFold network.

### SDK Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Social App SDK                          │
│                                                             │
│  ┌─────────────┐    ┌────────────────┐    ┌──────────────┐  │
│  │ Query       │    │ Authentication │    │ Schema       │  │
│  │ Builder     │◄───┤ Manager        │◄───┤ Discovery    │  │
│  └─────────────┘    └────────────────┘    └──────────────┘  │
│         ▲                    ▲                   ▲          │
│         │                    │                   │          │
│  ┌─────────────┐    ┌────────────────┐    ┌──────────────┐  │
│  │ Mutation    │    │ Network        │    │ Error        │  │
│  │ Builder     │    │ Manager        │    │ Handler      │  │
│  └─────────────┘    └────────────────┘    └──────────────┘  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Key SDK Components

#### 1. DataFold Client

The main entry point for applications:

```rust
pub struct DataFoldClient {
    // Connection to the local node
    connection: NodeConnection,
    // Authentication credentials
    auth: AuthCredentials,
    // Schema cache
    schema_cache: SchemaCache,
    // Network manager for remote operations
    network_manager: NetworkManager,
}

impl DataFoldClient {
    // Create a new client
    pub fn new(app_id: &str, private_key: &str) -> Result<Self, ClientError>;
    
    // Get a query builder for a specific schema
    pub fn query(&self, schema_name: &str) -> QueryBuilder;
    
    // Get a mutation builder for a specific schema
    pub fn mutate(&self, schema_name: &str) -> MutationBuilder;
    
    // Discover available schemas on the local node
    pub async fn discover_local_schemas(&self) -> Result<Vec<SchemaInfo>, ClientError>;
    
    // Discover available nodes in the network
    pub async fn discover_nodes(&self) -> Result<Vec<NodeInfo>, ClientError>;
    
    // Discover available schemas on a remote node
    pub async fn discover_remote_schemas(&self, node_id: &str) -> Result<Vec<SchemaInfo>, ClientError>;
}
```

#### 2. Query Builder

Fluent API for building queries:

```rust
pub struct QueryBuilder {
    schema_name: String,
    fields: Vec<String>,
    filter: Option<QueryFilter>,
    target_node: Option<String>,
    client: Arc<DataFoldClient>,
}

impl QueryBuilder {
    // Select fields to retrieve
    pub fn select(mut self, fields: &[&str]) -> Self;
    
    // Add a filter condition
    pub fn filter(mut self, filter: QueryFilter) -> Self;
    
    // Target a specific remote node
    pub fn on_node(mut self, node_id: &str) -> Self;
    
    // Execute the query
    pub async fn execute(&self) -> Result<QueryResult, ClientError>;
}
```

#### 3. Mutation Builder

Fluent API for building mutations:

```rust
pub struct MutationBuilder {
    schema_name: String,
    mutation_type: MutationType,
    data: HashMap<String, Value>,
    target_node: Option<String>,
    client: Arc<DataFoldClient>,
}

impl MutationBuilder {
    // Set the mutation type
    pub fn operation(mut self, operation: MutationType) -> Self;
    
    // Set a field value
    pub fn set(mut self, field: &str, value: Value) -> Self;
    
    // Set multiple field values
    pub fn set_many(mut self, values: HashMap<String, Value>) -> Self;
    
    // Target a specific remote node
    pub fn on_node(mut self, node_id: &str) -> Self;
    
    // Execute the mutation
    pub async fn execute(&self) -> Result<MutationResult, ClientError>;
}
```

#### 4. Network Manager

Handles communication with remote nodes:

```rust
pub struct NetworkManager {
    client: Arc<DataFoldClient>,
    known_nodes: HashMap<String, NodeInfo>,
}

impl NetworkManager {
    // Discover available nodes
    pub async fn discover_nodes(&mut self) -> Result<Vec<NodeInfo>, NetworkError>;
    
    // Check if a node is available
    pub async fn is_node_available(&self, node_id: &str) -> bool;
    
    // Get information about a node
    pub async fn get_node_info(&self, node_id: &str) -> Result<NodeInfo, NetworkError>;
}
```

#### 5. Schema Discovery

Helps developers understand available data models:

```rust
pub struct SchemaDiscovery {
    client: Arc<DataFoldClient>,
    schema_cache: SchemaCache,
}

impl SchemaDiscovery {
    // Get all available schemas on the local node
    pub async fn get_local_schemas(&mut self) -> Result<Vec<SchemaInfo>, ClientError>;
    
    // Get schemas available on a remote node
    pub async fn get_remote_schemas(&mut self, node_id: &str) -> Result<Vec<SchemaInfo>, ClientError>;
    
    // Get detailed information about a schema
    pub async fn get_schema_details(&self, schema_name: &str, 
                                  node_id: Option<&str>) -> Result<SchemaDetails, ClientError>;
}
```

### Language-Specific SDKs

The SDK would be implemented in multiple languages to support various development environments:

#### Rust SDK (Native)

```rust
// Example usage
let client = DataFoldClient::new("my_social_app", &private_key)?;

// Query user profiles
let users = client.query("user_profile")
    .select(&["username", "full_name", "bio"])
    .filter(QueryFilter::eq("location", "San Francisco"))
    .execute()
    .await?;

// Create a new post
let result = client.mutate("post")
    .operation(MutationType::Create)
    .set("title", "Hello DataFold Network")
    .set("content", "This is my first post on the decentralized social network!")
    .set("author_id", current_user_id)
    .execute()
    .await?;
```

#### TypeScript/JavaScript SDK

```typescript
// Example usage
const client = new DataFoldClient("my_social_app", privateKey);

// Query user profiles
const users = await client.query("user_profile")
    .select(["username", "full_name", "bio"])
    .filter({ location: "San Francisco" })
    .execute();

// Create a new post
const result = await client.mutate("post")
    .operation("create")
    .set({
        title: "Hello DataFold Network",
        content: "This is my first post on the decentralized social network!",
        author_id: currentUserId
    })
    .execute();
```

#### Python SDK

```python
# Example usage
client = DataFoldClient("my_social_app", private_key)

# Query user profiles
users = (client.query("user_profile")
    .select(["username", "full_name", "bio"])
    .filter(location="San Francisco")
    .execute())

# Create a new post
result = (client.mutate("post")
    .operation("create")
    .set("title", "Hello DataFold Network")
    .set("content", "This is my first post on the decentralized social network!")
    .set("author_id", current_user_id)
    .execute())
```

### SDK Development Tools

To further simplify development, the SDK would include:

1. **Schema Code Generator**: Automatically generates type-safe code from schemas
2. **Mock Server**: For local development without a full DataFold node
3. **Permission Simulator**: To test apps against different permission scenarios
4. **Network Simulator**: To test multi-node scenarios locally
5. **Debugging Tools**: For tracing requests and understanding errors

### SDK Security Features

1. **Automatic Signing**: Handles cryptographic signing of requests
2. **Permission Checking**: Client-side validation before sending requests
3. **Error Handling**: Detailed error information with recovery suggestions
4. **Rate Limiting**: Client-side throttling to prevent quota exhaustion
5. **Secure Storage**: Secure handling of private keys and credentials

## Implementation Path

1. Implement isolation mechanisms (MicroVMs with Firecracker/Kata Containers, Linux containers, or WASM sandbox)
2. Extend the DataFoldNode with app sandbox management capabilities
3. Implement the App API Gateway for handling app requests
4. Develop the Permission Validator for enforcing app permissions
5. Create the Network Bridge for secure remote node access
6. Build the app-side Network Query Router for operation routing
7. Develop the SDK for multiple programming languages
8. Create SDK development tools and documentation
9. Implement the app registration and key management system
10. Add audit logging for security monitoring
11. Create developer tools for app creation and testing

This architecture provides a secure, flexible foundation for building social applications on the DataFold network while maintaining the system's core principles of privacy, security, and distributed data access, with strict network isolation to prevent any direct internet access from the sandboxed applications.
