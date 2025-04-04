# Developing Sandboxed Applications with FoldClient

This document provides guidelines for developing applications that run in the FoldClient sandbox and communicate with the DataFold node.

## Overview

Sandboxed applications run in a restricted environment provided by the FoldClient. They communicate with the DataFold node through the FoldClient's IPC mechanism, which enforces permissions and resource limits.

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│                 │     │                 │     │                 │
│  Sandboxed App  │◄────┤   FoldClient    │◄────┤  DataFold Node  │
│                 │     │                 │     │                 │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

## Getting Started

### Prerequisites

To develop a sandboxed application, you need:

1. The FoldClient installed and running
2. An app ID and token from the FoldClient
3. The Rust programming language and Cargo

### Setting Up a New Project

Create a new Rust project:

```bash
cargo new my_sandboxed_app
cd my_sandboxed_app
```

Add the FoldClient IPC client to your dependencies:

```toml
# Cargo.toml
[dependencies]
fold_client = { path = "/path/to/fold_client" }
tokio = { version = "1.28", features = ["full"] }
serde_json = "1.0"
```

### Basic Application Structure

Here's a basic structure for a sandboxed application:

```rust
use fold_client::ipc::client::{IpcClient, Result};
use serde_json::json;
use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Get the app ID and token from environment variables
    let app_id = env::var("FOLD_CLIENT_APP_ID")
        .expect("FOLD_CLIENT_APP_ID environment variable not set");
    let token = env::var("FOLD_CLIENT_APP_TOKEN")
        .expect("FOLD_CLIENT_APP_TOKEN environment variable not set");

    // Get the socket directory from the environment or use the default
    let socket_dir = if let Ok(dir) = env::var("FOLD_CLIENT_SOCKET_DIR") {
        PathBuf::from(dir)
    } else {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        home_dir.join(".datafold").join("sockets")
    };

    // Connect to the FoldClient
    let mut client = IpcClient::connect(&socket_dir, &app_id, &token).await?;

    // Use the client to communicate with the DataFold node
    // ...

    Ok(())
}
```

## Communicating with the DataFold Node

The FoldClient IPC client provides methods for communicating with the DataFold node:

### Listing Schemas

```rust
// List available schemas
let schemas = client.list_schemas().await?;
println!("Available schemas: {:?}", schemas);
```

### Querying Data

```rust
// Query data from a schema
let users = client.query(
    "user",                                // Schema name
    &["id", "username", "full_name"],      // Fields to select
    None,                                  // Filter (optional)
).await?;
println!("Users: {:?}", users);

// Query with a filter
let filter = Some(json!({
    "field": "username",
    "operator": "eq",
    "value": "alice",
}));
let alice = client.query(
    "user",
    &["id", "username", "full_name"],
    filter,
).await?;
println!("Alice: {:?}", alice);
```

### Creating Data

```rust
// Create a new user
let user_id = uuid::Uuid::new_v4().to_string();
let user_data = json!({
    "id": user_id,
    "username": "bob",
    "full_name": "Bob Smith",
    "bio": "Hello, world!",
    "created_at": chrono::Utc::now().to_rfc3339(),
});
let result = client.create("user", user_data).await?;
println!("User created with ID: {}", result);
```

### Updating Data

```rust
// Update a user
let user_data = json!({
    "id": user_id,
    "bio": "Updated bio",
});
let success = client.update("user", user_data).await?;
println!("User updated: {}", success);
```

### Deleting Data

```rust
// Delete a user
let success = client.delete("user", &user_id).await?;
println!("User deleted: {}", success);
```

### Discovering Remote Nodes

```rust
// Discover remote nodes
let nodes = client.discover_nodes().await?;
println!("Discovered nodes: {:?}", nodes);
```

### Querying Remote Nodes

```rust
// Query a remote node
if !nodes.is_empty() {
    let node_id = nodes[0].get("id")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    
    let remote_users = client.query_remote(
        node_id,
        "user",
        &["id", "username", "full_name"],
        None,
    ).await?;
    println!("Remote users: {:?}", remote_users);
}
```

## Sandbox Constraints

Sandboxed applications run with the following constraints:

### Network Access

By default, sandboxed applications cannot access the network directly. All network communication must go through the FoldClient.

If your application needs to access the network, you need to:

1. Request network access when registering the app with the FoldClient
2. Use the FoldClient's API for network communication

### File System Access

By default, sandboxed applications can only access their working directory. All file operations outside this directory will fail.

If your application needs to access other directories, you need to:

1. Request file system access when registering the app with the FoldClient
2. Use the FoldClient's API for file system operations

### Resource Limits

Sandboxed applications have limits on their resource usage:

1. Memory: Limited to the amount specified when registering the app
2. CPU: Limited to the percentage specified when registering the app

If your application exceeds these limits, it may be terminated by the FoldClient.

## Best Practices

### Error Handling

Always handle errors from the FoldClient IPC client. If the connection to the FoldClient is lost, your application should gracefully exit or attempt to reconnect.

```rust
match client.list_schemas().await {
    Ok(schemas) => {
        println!("Available schemas: {:?}", schemas);
    }
    Err(e) => {
        eprintln!("Error listing schemas: {}", e);
        // Handle the error
    }
}
```

### Resource Management

Be mindful of your application's resource usage. The FoldClient enforces resource limits, so your application should use resources efficiently.

### Permission Requests

Only request the permissions your application needs. The FoldClient enforces permissions, so your application should only attempt operations it has permission for.

### Graceful Shutdown

Handle signals to gracefully shut down your application:

```rust
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the client
    // ...

    // Wait for a signal to shut down
    tokio::select! {
        _ = signal::ctrl_c() => {
            println!("Received Ctrl+C, shutting down");
        }
        _ = signal::unix::signal(signal::unix::SignalKind::terminate()) => {
            println!("Received SIGTERM, shutting down");
        }
    }

    // Clean up resources
    // ...

    Ok(())
}
```

## Debugging

### Logging

Use the `log` crate for logging:

```rust
use log::{info, warn, error};

// Initialize the logger
env_logger::init();

// Log messages
info!("Application started");
warn!("Something unexpected happened");
error!("An error occurred: {}", e);
```

### Environment Variables

The FoldClient sets the following environment variables for sandboxed applications:

- `FOLD_CLIENT_APP_ID`: The ID of the application
- `FOLD_CLIENT_APP_TOKEN`: The token for the application
- `FOLD_CLIENT_SOCKET_DIR`: The directory where the FoldClient's socket is located

You can use these environment variables for debugging:

```rust
println!("App ID: {}", env::var("FOLD_CLIENT_APP_ID").unwrap_or_default());
println!("Socket Dir: {}", env::var("FOLD_CLIENT_SOCKET_DIR").unwrap_or_default());
```

### Running Outside the Sandbox

During development, you may want to run your application outside the sandbox. You can do this by setting the environment variables manually:

```bash
FOLD_CLIENT_APP_ID=your-app-id \
FOLD_CLIENT_APP_TOKEN=your-app-token \
FOLD_CLIENT_SOCKET_DIR=/path/to/socket/dir \
cargo run
```

## Example Applications

### Simple Query Application

```rust
use fold_client::ipc::client::{IpcClient, Result};
use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Get the app ID and token from environment variables
    let app_id = env::var("FOLD_CLIENT_APP_ID")
        .expect("FOLD_CLIENT_APP_ID environment variable not set");
    let token = env::var("FOLD_CLIENT_APP_TOKEN")
        .expect("FOLD_CLIENT_APP_TOKEN environment variable not set");

    // Get the socket directory
    let socket_dir = if let Ok(dir) = env::var("FOLD_CLIENT_SOCKET_DIR") {
        PathBuf::from(dir)
    } else {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        home_dir.join(".datafold").join("sockets")
    };

    // Connect to the FoldClient
    let mut client = IpcClient::connect(&socket_dir, &app_id, &token).await?;

    // List available schemas
    let schemas = client.list_schemas().await?;
    println!("Available schemas: {:?}", schemas);

    // Query users
    if schemas.contains(&"user".to_string()) {
        let users = client.query("user", &["id", "username", "full_name"], None).await?;
        println!("Users: {:?}", users);
    }

    Ok(())
}
```

### Data Creation Application

```rust
use fold_client::ipc::client::{IpcClient, Result};
use serde_json::json;
use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Get the app ID and token from environment variables
    let app_id = env::var("FOLD_CLIENT_APP_ID")
        .expect("FOLD_CLIENT_APP_ID environment variable not set");
    let token = env::var("FOLD_CLIENT_APP_TOKEN")
        .expect("FOLD_CLIENT_APP_TOKEN environment variable not set");

    // Get the socket directory
    let socket_dir = if let Ok(dir) = env::var("FOLD_CLIENT_SOCKET_DIR") {
        PathBuf::from(dir)
    } else {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        home_dir.join(".datafold").join("sockets")
    };

    // Connect to the FoldClient
    let mut client = IpcClient::connect(&socket_dir, &app_id, &token).await?;

    // Create a new user
    let user_id = uuid::Uuid::new_v4().to_string();
    let user_data = json!({
        "id": user_id,
        "username": "new_user",
        "full_name": "New User",
        "bio": "Created by a sandboxed app",
        "created_at": chrono::Utc::now().to_rfc3339(),
    });
    let result = client.create("user", user_data).await?;
    println!("User created with ID: {}", result);

    Ok(())
}
```

## Conclusion

Developing sandboxed applications with FoldClient allows you to create secure applications that can access the DataFold node API without compromising security. By following the guidelines in this document, you can create applications that work well within the sandbox constraints and provide a good user experience.
