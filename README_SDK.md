# DataFold Social App SDK

The DataFold Social App SDK provides a secure framework for developing containerized social applications that can interact with DataFold nodes while maintaining security, privacy, and the distributed nature of the network.

## Key Features

- **Complete Network Isolation**: Apps have no direct internet access and can only communicate through the DataFold node.
- **Strong Isolation Boundaries**: Multiple layers of isolation prevent apps from bypassing restrictions.
- **Cryptographic Authentication**: All requests are signed using the app's private key.
- **Fine-grained Permissions**: Permissions are enforced at schema, field, and operation levels.
- **Rate Limiting**: Prevents abuse by limiting the frequency of operations.
- **Resource Limits**: CPU, memory, and storage limits prevent resource exhaustion.
- **Audit Logging**: All operations are logged for security auditing.

## Isolation Options

The SDK supports multiple isolation mechanisms:

### 1. MicroVM with Firecracker/Kata Containers

Provides hardware-level isolation through KVM with minimal overhead:

```rust
// Create a MicroVM configuration with Firecracker
let vm_config = MicroVMConfig::new(
    MicroVMType::Firecracker, 
    "/var/lib/datafold/vm-images/minimal-rootfs.ext4"
)
    .with_vcpu_count(1)
    .with_memory_mb(128);

// Create a container configuration
let container_config = ContainerConfig::new_microvm(
    PathBuf::from("/path/to/app/binary"),
    vm_config
);
```

### 2. Linux Containers

Uses namespace isolation and capability restrictions:

```rust
// Create a Linux container configuration with maximum isolation
let container_config = ContainerConfig::new_linux_container(
    PathBuf::from("/path/to/app/binary"),
    LinuxContainerConfig::maximum()
);
```

### 3. WebAssembly Sandbox

For web-based or lightweight applications:

```rust
// Create a WebAssembly sandbox configuration with maximum isolation
let container_config = ContainerConfig::new_wasm_sandbox(
    PathBuf::from("/path/to/app/wasm"),
    WasmSandboxConfig::maximum()
);
```

## Client Usage

The SDK provides a client for interacting with the DataFold network:

```rust
// Create a client
let client = DataFoldClient::new(
    "my-social-app",
    "private-key",
    "public-key"
);

// Query user profiles
let users = client.query("user_profile")
    .select(&["username", "full_name", "bio"])
    .filter(QueryFilter::eq("location", serde_json::json!("San Francisco")))
    .execute()
    .await?;

// Create a new post
let result = client.mutate("post")
    .operation(MutationType::Create)
    .set("title", serde_json::json!("Hello DataFold Network"))
    .set("content", serde_json::json!("This is my first post on the decentralized social network!"))
    .set("author_id", serde_json::json!(current_user_id))
    .execute()
    .await?;
```

## Container Management

The SDK provides a container manager for running social apps:

```rust
// Create app permissions
let permissions = AppPermissions::new()
    .allow_schemas(&["user", "post", "comment"])
    .with_field_permissions(
        "user",
        FieldPermissions::new()
            .allow_reads(&["id", "username", "full_name", "bio"])
            .allow_writes(&["bio"])
    )
    .allow_remote_nodes(&["node1", "node2"])
    .with_max_trust_distance(2);

// Create a social app container
let mut container = SocialAppContainer::new(
    "my-social-app",
    "public-key",
    permissions,
    container_config
);

// Start the container
container.start().await?;

// Stop the container
container.stop().await?;
```

## Network Operations

The SDK provides network operations for discovering and interacting with remote nodes:

```rust
// Discover available nodes
let nodes = client.discover_nodes().await?;

// Check if a node is available
let available = client.is_node_available("node1").await?;

// Get information about a node
let node_info = client.get_node_info("node1").await?;

// Query on a remote node
let remote_results = client.query_on_node("user", "node1")
    .select(&["id", "username", "full_name"])
    .execute()
    .await?;
```

## Schema Discovery

The SDK provides schema discovery for exploring available data models:

```rust
// Discover local schemas
let schemas = client.discover_local_schemas().await?;

// Discover remote schemas
let remote_schemas = client.discover_remote_schemas("node1").await?;

// Get schema details
let schema_details = client.get_schema_details("user", None).await?;
```

## Example

See the [social_app_example.rs](examples/social_app_example.rs) file for a complete example of using the SDK.

## Security Considerations

1. **Network Isolation**: Apps have no direct internet access and can only communicate through the DataFold node.
2. **Cryptographic Authentication**: All requests are signed using the app's private key.
3. **Fine-grained Permissions**: Permissions are enforced at schema, field, and operation levels.
4. **Rate Limiting**: Prevents abuse by limiting the frequency of operations.
5. **Resource Limits**: CPU, memory, and storage limits prevent resource exhaustion.
6. **Audit Logging**: All operations are logged for security auditing.

## Implementation Details

The SDK is implemented in Rust and provides a safe, efficient, and flexible API for developing social applications on the DataFold network. It uses a combination of isolation mechanisms, cryptographic authentication, and permission enforcement to ensure security and privacy.

For more details on the implementation, see the [sandboxed_social_app.md](cline_docs/sandboxed_social_app.md) document.
