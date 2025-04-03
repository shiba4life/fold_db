# DataFold SDK

The DataFold SDK is a client library for interacting with DataFold nodes. It provides a high-level API for querying and mutating data, discovering schemas and nodes, and managing containerized applications.

## Features

- **Client API**: High-level API for interacting with DataFold nodes
- **Query and Mutation Builders**: Fluent API for building and executing queries and mutations
- **Schema Management**: Create, update, and delete schemas with a fluent builder API
- **Schema Discovery**: Discover available schemas on local and remote nodes
- **Network Management**: Discover and interact with remote nodes
- **Container Management**: Manage containerized applications with various isolation mechanisms
- **Permission Management**: Fine-grained permission control for applications

## Usage

Add the SDK to your Cargo.toml:

```toml
[dependencies]
datafold_sdk = { path = "path/to/datafold_sdk" }
```

### Basic Example

```rust
use datafold_sdk::{DataFoldClient, QueryFilter};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let client = DataFoldClient::new("my-app", "private-key", "public-key");
    
    // Query data
    let result = client.query("user")
        .select(&["id", "name", "email"])
        .filter(QueryFilter::eq("name", json!("John Doe")))
        .execute()
        .await?;
    
    println!("Query results: {:?}", result);
    
    // Mutate data
    let result = client.mutate("user")
        .set("name", json!("Jane Doe"))
        .set("email", json!("jane@example.com"))
        .execute()
        .await?;
    
    println!("Mutation result: {:?}", result);
    
    Ok(())
}
```

### Schema Management

```rust
use datafold_sdk::{DataFoldClient, SchemaBuilder, FieldType, TrustDistance};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let client = DataFoldClient::new("my-app", "private-key", "public-key");
    
    // Create a schema using the builder
    let schema = client.schema_builder("user_profile")
        .add_field("username", |field| {
            field.field_type(FieldType::Single)
                .required(true)
                .description("User's unique username")
                .permissions(
                    TrustDistance::Distance(0), // Public read
                    TrustDistance::Distance(1)  // Owner-only write
                )
        })
        .add_field("email", |field| {
            field.field_type(FieldType::Single)
                .required(true)
                .description("User's email address")
                .validation(json!({
                    "format": "email"
                }))
        })
        .add_field("posts", |field| {
            field.field_type(FieldType::Collection)
                .required(false)
                .description("User's posts")
        })
        .build()?;
    
    // Create the schema on the node
    client.create_schema(schema).await?;
    
    // Get schema details
    let details = client.get_schema_details("user_profile", None).await?;
    println!("Schema details: {}", details);
    
    // Update the schema (add a new field)
    let updated_schema = client.schema_builder("user_profile")
        // Include existing fields...
        .add_field("bio", |field| {
            field.field_type(FieldType::Single)
                .required(false)
                .description("User's biography")
        })
        .build()?;
    
    client.update_schema(updated_schema).await?;
    
    // Delete a schema
    client.delete_schema("unused_schema").await?;
    
    Ok(())
}
```

### Discovering Schemas and Nodes

```rust
use datafold_sdk::DataFoldClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let client = DataFoldClient::new("my-app", "private-key", "public-key");
    
    // Discover local schemas
    let schemas = client.discover_local_schemas().await?;
    println!("Local schemas: {:?}", schemas);
    
    // Discover remote nodes
    let nodes = client.discover_nodes().await?;
    println!("Remote nodes: {:?}", nodes);
    
    // Discover schemas on a remote node
    let remote_schemas = client.discover_remote_schemas("node1").await?;
    println!("Remote schemas on node1: {:?}", remote_schemas);
    
    Ok(())
}
```

### Container Management

```rust
use datafold_sdk::{
    SocialAppContainer, ContainerConfig, AppPermissions, FieldPermissions,
    MicroVMConfig, MicroVMType
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create permissions
    let permissions = AppPermissions::new()
        .allow_schemas(&["user", "post"])
        .with_field_permissions("user", FieldPermissions::new()
            .allow_reads(&["id", "name", "email"])
            .allow_writes(&["name", "email"]))
        .with_max_trust_distance(2);
    
    // Create VM configuration
    let vm_config = MicroVMConfig::new(
        MicroVMType::Firecracker,
        "/var/lib/datafold/vm-images/minimal-rootfs.ext4"
    )
    .with_vcpu_count(1)
    .with_memory_mb(128);
    
    // Create container configuration
    let config = ContainerConfig::new_microvm(
        PathBuf::from("/path/to/app"),
        vm_config
    );
    
    // Create container
    let mut container = SocialAppContainer::new(
        "my-app",
        "public-key",
        permissions,
        config
    );
    
    // Start container
    container.start().await?;
    
    // Do something with the container
    
    // Stop container
    container.stop().await?;
    
    Ok(())
}
```

## License

This project is licensed under the MIT License.
