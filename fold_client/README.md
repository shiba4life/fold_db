# FoldClient

FoldClient is a client for DataFold that provides sandboxed access to the node API. It acts as a mediator between applications and the DataFold node, providing sandboxed access to the node API. It handles authentication, permission enforcement, and sandboxing of applications.

## Overview

FoldClient solves the problem of providing a sandbox guarantee for applications that need to access the DataFold node API. It ensures that only processes initiated by the FoldClient can access the node API, and it enforces permissions and resource limits on those processes.

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│                 │     │                 │     │                 │
│  Sandboxed App  │◄────┤   FoldClient    │◄────┤  DataFold Node  │
│                 │     │                 │     │                 │
└─────────────────┘     └─────────────────┘     └─────────────────┘
        ▲                       ▲                       ▲
        │                       │                       │
        │                       │                       │
        │                       │                       │
┌───────┴───────┐     ┌─────────┴─────────┐   ┌─────────┴─────────┐
│  Restricted   │     │  Authentication   │   │   Node API with   │
│  Environment  │     │  & Permissions    │   │   Cryptographic   │
│               │     │                   │   │   Verification    │
└───────────────┘     └───────────────────┘   └───────────────────┘
```

## Features

- **Sandboxed Environment**: Applications run in a sandboxed environment with restricted access to system resources.
- **Network Isolation**: Applications can be prevented from accessing the network directly.
- **File System Isolation**: Applications can be restricted to a specific directory.
- **Resource Limits**: Applications can have memory and CPU usage limits.
- **Permission Enforcement**: Applications can only perform operations they have been granted permission for.
- **Cryptographic Authentication**: All communication is authenticated using cryptographic signatures.
- **Cross-Platform**: Works on Linux, macOS, and Windows.

## How It Works

FoldClient provides a sandbox guarantee through several mechanisms:

1. **Parent-Child Process Model**: Applications are launched as child processes of the FoldClient, which gives the FoldClient control over their lifecycle.

2. **Platform-Specific Sandboxing**:
   - On Linux: Uses namespaces, cgroups, and seccomp to isolate applications.
   - On macOS: Uses the sandbox-exec command to create a sandboxed environment.
   - On Windows: Uses job objects and integrity levels to restrict applications.

3. **IPC Mechanism**: Applications communicate with the FoldClient using a secure IPC mechanism (Unix domain sockets on Linux/macOS, named pipes on Windows).

4. **Request Signing**: All requests from the FoldClient to the DataFold node are cryptographically signed, ensuring that only the FoldClient can access the node API.

5. **Permission Enforcement**: The FoldClient checks if an application has permission to perform an operation before forwarding the request to the node.

## Usage

### Starting the FoldClient

```rust
use fold_client::{FoldClient, FoldClientConfig, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Create a FoldClient with a custom configuration
    let mut config = FoldClientConfig::default();
    config.allow_network_access = false;
    config.allow_filesystem_access = true;
    config.max_memory_mb = Some(512);
    config.max_cpu_percent = Some(25);

    // Create the FoldClient
    let mut client = FoldClient::with_config(config)?;

    // Start the FoldClient
    client.start().await?;

    // ...

    // Stop the FoldClient
    client.stop().await?;

    Ok(())
}
```

### Registering an App

```rust
// Register a new app
let app = client.register_app("My App", &["list_schemas", "query", "mutation"]).await?;
println!("App registered with ID: {}", app.app_id);
```

### Launching an App

```rust
// Launch the app
client.launch_app(&app.app_id, "path/to/app", &["arg1", "arg2"]).await?;
```

### Writing a Sandboxed App

Sandboxed applications use the FoldClient's IPC mechanism to communicate with the DataFold node:

```rust
use fold_client::ipc::client::{IpcClient, Result};
use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Get the app ID and token from environment variables
    let app_id = env::var("FOLD_CLIENT_APP_ID").expect("FOLD_CLIENT_APP_ID not set");
    let token = env::var("FOLD_CLIENT_APP_TOKEN").expect("FOLD_CLIENT_APP_TOKEN not set");

    // Get the socket directory
    let socket_dir = PathBuf::from("/path/to/socket/dir");

    // Connect to the FoldClient
    let mut client = IpcClient::connect(&socket_dir, &app_id, &token).await?;

    // List available schemas
    let schemas = client.list_schemas().await?;
    println!("Available schemas: {:?}", schemas);

    // ...

    Ok(())
}
```

## Security Considerations

- **Process Isolation**: Each application runs in its own process, providing isolation from other applications.
- **Network Isolation**: Applications can be prevented from accessing the network directly, ensuring that all communication goes through the FoldClient.
- **File System Isolation**: Applications can be restricted to a specific directory, preventing access to sensitive files.
- **Resource Limits**: Applications can have memory and CPU usage limits, preventing resource exhaustion attacks.
- **Permission Enforcement**: Applications can only perform operations they have been granted permission for, preventing unauthorized access.
- **Cryptographic Authentication**: All communication is authenticated using cryptographic signatures, preventing impersonation attacks.

## Building and Testing

To build the FoldClient:

```bash
cargo build
```

To run the tests:

```bash
cargo test
```

To run the examples:

```bash
# Start the FoldClient and register an app
cargo run --example simple_app

# Run a sandboxed app
cargo run --example sandboxed_app
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
