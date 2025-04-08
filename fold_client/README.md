# FoldClient

FoldClient is a client for DataFold that provides Docker-sandboxed access to the node API. It acts as a mediator between applications and the DataFold node, providing secure and isolated access to the node's functionality.

## Features

- **Docker-based Sandboxing**: Applications run in isolated Docker containers with configurable resource limits and network access.
- **Authentication and Authorization**: Applications must authenticate with the FoldClient and are only granted access to operations they have permission for.
- **IPC Communication**: Applications communicate with the FoldClient using a secure IPC mechanism.
- **Node Communication**: The FoldClient communicates with the DataFold node on behalf of applications.
- **Resource Management**: The FoldClient manages Docker containers and resources for applications.

## Requirements

- Rust and Cargo
- Docker
- DataFold node

## Installation

### From Source

1. Clone the repository:
   ```bash
   git clone https://github.com/your-org/fold_client.git
   cd fold_client
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Install the binary:
   ```bash
   cargo install --path .
   ```

## Configuration

FoldClient can be configured using environment variables or a `.env` file. The following environment variables are available:

- `FOLD_CLIENT_NODE_SOCKET_PATH`: Path to the DataFold node socket
- `FOLD_CLIENT_NODE_TCP_HOST`: Host for the DataFold node TCP connection
- `FOLD_CLIENT_NODE_TCP_PORT`: Port for the DataFold node TCP connection
- `FOLD_CLIENT_APP_SOCKET_DIR`: Path to the directory where app sockets will be created
- `FOLD_CLIENT_APP_DATA_DIR`: Path to the directory where app data will be stored
- `FOLD_CLIENT_DOCKER_HOST`: Docker API URL (e.g., `unix:///var/run/docker.sock`)
- `FOLD_CLIENT_DOCKER_NETWORK`: Docker network to use for containers
- `FOLD_CLIENT_DOCKER_CPU_LIMIT`: Default CPU limit for containers (in CPU shares)
- `FOLD_CLIENT_DOCKER_MEM_LIMIT`: Default memory limit for containers (in MB)
- `FOLD_CLIENT_DOCKER_STORAGE_LIMIT`: Default storage limit for containers (in MB)
- `FOLD_CLIENT_DOCKER_ALLOW_NETWORK`: Whether to enable network access for containers by default
- `FOLD_CLIENT_DOCKER_BASE_IMAGE`: Base image to use for containers
- `FOLD_CLIENT_DOCKER_AUTO_REMOVE`: Whether to auto-remove containers when they exit

## Usage

### Starting the FoldClient

```bash
fold_client
```

### Registering an App

Apps must be registered with the FoldClient before they can be used. This can be done programmatically using the FoldClient API:

```rust
use fold_client::FoldClient;

#[tokio::main]
async fn main() {
    // Create a FoldClient
    let mut client = FoldClient::new().unwrap();

    // Start the FoldClient
    client.start().await.unwrap();

    // Register an app
    let app = client.register_app("my-app", &["list_schemas", "query", "mutation"]).await.unwrap();

    println!("App registered with ID: {}", app.app_id);
    println!("App token: {}", app.token);

    // Stop the FoldClient
    client.stop().await.unwrap();
}
```

### Launching an App

Once an app is registered, it can be launched using the FoldClient API:

```rust
use fold_client::FoldClient;

#[tokio::main]
async fn main() {
    // Create a FoldClient
    let mut client = FoldClient::new().unwrap();

    // Start the FoldClient
    client.start().await.unwrap();

    // Launch an app
    client.launch_app("app-id", "program", &["arg1", "arg2"]).await.unwrap();

    // Stop the FoldClient
    client.stop().await.unwrap();
}
```

### Developing Apps for FoldClient

Apps that run in the FoldClient sandbox can use the FoldClient IPC client to communicate with the DataFold node:

```rust
use fold_client::ipc::client::IpcClient;
use serde_json::Value;

#[tokio::main]
async fn main() {
    // Get the app ID and token from environment variables
    let app_id = std::env::var("FOLD_CLIENT_APP_ID").unwrap();
    let token = std::env::var("FOLD_CLIENT_APP_TOKEN").unwrap();
    let socket_dir = std::env::var("FOLD_CLIENT_SOCKET_DIR").unwrap();

    // Connect to the FoldClient
    let mut client = IpcClient::connect(
        &std::path::PathBuf::from(socket_dir),
        &app_id,
        &token,
    ).await.unwrap();

    // List available schemas
    let schemas = client.list_schemas().await.unwrap();
    println!("Available schemas: {:?}", schemas);

    // Query data from a schema
    let results = client.query("user", &["id", "name"], None).await.unwrap();
    println!("Query results: {:?}", results);

    // Create data in a schema
    let data = serde_json::json!({
        "name": "John Doe",
        "email": "john@example.com",
    });
    let id = client.create("user", data).await.unwrap();
    println!("Created user with ID: {}", id);
}
```

## Docker Sandboxing

FoldClient uses Docker to sandbox applications. Each application runs in its own Docker container with the following features:

- **Resource Limits**: CPU, memory, and storage limits can be configured.
- **Network Isolation**: Network access can be disabled or restricted.
- **File System Isolation**: Applications can only access their own working directory.
- **IPC Communication**: Applications communicate with the FoldClient using a Unix domain socket.

## Security Considerations

- **Container Escape**: Docker containers provide a level of isolation, but they are not completely secure. A determined attacker with sufficient privileges could potentially escape the container.
- **Resource Exhaustion**: While resource limits can be configured, a malicious application could still attempt to exhaust resources within its limits.
- **IPC Security**: The IPC mechanism uses Unix domain sockets, which are generally secure but could potentially be accessed by other processes with sufficient privileges.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
