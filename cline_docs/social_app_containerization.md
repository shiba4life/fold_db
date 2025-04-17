# Social App and Containerization in DataFold

This document explains how the social app works within the DataFold ecosystem and details the containerization approach used for application sandboxing.

## Social App Overview

The social app is a demonstration application built on top of the DataFold platform that showcases the core capabilities of the system. It is built using Rust and Cargo (Rust's package manager and build system), not Docker. Docker is used only for running the application in a sandboxed environment, not for building it. The app implements basic social networking features including:

- User management (creation and querying)
- Post creation and retrieval
- Comment functionality
- Cross-node communication and data discovery

### Architecture

The social app operates within a layered architecture:

1. **Application Layer (Social App)**: The Rust-based application that provides the social networking functionality
2. **Sandboxing Layer (FoldClient)**: Provides secure, isolated execution environment using Docker containers
3. **Node Layer (DataFold Node)**: Handles data storage, schema management, and network communication
4. **Network Layer**: Enables communication between different DataFold nodes

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│                 │     │                 │     │                 │
│  Social App     │◄────┤   FoldClient    │◄────┤  DataFold Node  │
│                 │     │                 │     │                 │
└─────────────────┘     └─────────────────┘     └─────────────────┘
        ▲                       ▲                       ▲
        │                       │                       │
        │                       │                       │
        │                       │                       │
┌───────┴───────┐     ┌─────────┴─────────┐   ┌─────────┴─────────┐
│  Application  │     │  Docker-based     │   │   Schema-based    │
│  Logic        │     │  Sandboxing       │   │   Data Storage    │
│               │     │                   │   │                   │
└───────────────┘     └───────────────────┘   └───────────────────┘
```

### Data Model

The social app uses three main schemas:

1. **User Schema**:
   - Fields: id, username, full_name, bio, created_at
   - Represents user profiles in the system

2. **Post Schema**:
   - Fields: id, title, content, author_id, created_at
   - Represents content posted by users

3. **Comment Schema**:
   - Fields: id, content, author_id, post_id, created_at
   - Represents responses to posts

### Application Flow

1. The social app is built using Rust's Cargo build system (`cargo build`)
2. It is then run either directly or through the FoldClient's Docker container
3. When running through FoldClient, the app is executed in a Docker container for sandboxing
4. The app connects to the DataFold node (either directly or through FoldClient)
5. It lists available schemas and creates them if they don't exist
6. It creates a test user if needed
7. It creates posts and comments
8. It queries data across schemas
9. It discovers and communicates with remote nodes when available

### Containerization and Sandboxing Process

The specific process for containerizing and sandboxing the social app works as follows:

1. **App Registration**:
   - The social app is first registered with the FoldClient using the `register_app.rs` script
   - This script connects to the FoldClient and registers the app with specific permissions
   - The FoldClient generates a unique app ID and token for the social app
   - These credentials are used for authentication when the app runs

2. **App Building**:
   - The social app is built using Rust's Cargo build system (`cargo build`)
   - This creates a native executable binary for the current platform
   - The build process is completely separate from the containerization

3. **Container Creation**:
   - When the app is launched through FoldClient, the FoldClient creates a Docker container
   - The container is configured with:
     - A base Alpine Linux image
     - The app's working directory mounted at `/app`
     - The socket directory mounted at `/tmp/fold_client_sockets`
     - Environment variables for the app ID and token
     - Resource limits for CPU, memory, and storage
     - Network isolation (typically with the `none` network driver)

4. **App Execution**:
   - The compiled social app binary is executed inside the Docker container
   - The app uses the environment variables to find the socket path
   - It connects to the FoldClient through the Unix domain socket mounted in the container
   - All communication between the app and the DataFold node passes through this socket

5. **IPC Communication**:
   - The app uses the FoldClient IPC client to send requests to the FoldClient
   - The FoldClient authenticates these requests using the app's token
   - The FoldClient then forwards authenticated requests to the DataFold node
   - Responses from the DataFold node are sent back through the same channel

6. **Container Lifecycle**:
   - The FoldClient monitors the container while the app is running
   - When the app terminates, the FoldClient stops and removes the container
   - This ensures proper resource cleanup

## Containerization with Docker

The DataFold ecosystem has recently replaced its platform-specific sandbox implementation with a Docker-based approach. It's important to note that the social app itself is not built using Docker - it's a Rust application built with Cargo. 

### Current Implementation Note

**Important**: While the DataFold ecosystem supports Docker-based sandboxing through FoldClient, the current implementation of the social app (as seen in `run_social_app.sh`) does not actually use Docker containers to run the app. Instead, it connects directly to the DataFold node.

When you run the social app using the `run_social_app.sh` script, the following process occurs:
1. The script checks if Docker is running (required for the DataFold node, not for the app itself)
2. It verifies the DataFold node is running
3. It builds the social app using Cargo
4. It launches the app directly (not in a Docker container) with `cargo run --bin social_app`
5. The app connects directly to the DataFold node's TCP server

The Docker-based sandboxing described below is part of the system's architecture and capabilities, but is not currently being used by the social app in its present implementation.

This containerization strategy provides several key benefits:

### Docker Sandboxing Features

1. **Process Isolation**:
   - Docker containers run in isolated namespaces
   - Applications cannot access or interfere with processes outside their container
   - Complete isolation from the host system and other containers

2. **Network Isolation**:
   - Configurable network access (none, limited, or full)
   - Default configuration uses the `none` network driver for complete network isolation
   - Applications can only communicate through controlled channels

3. **File System Isolation**:
   - Containers have their own isolated file system
   - Only specific directories are mounted into the container
   - Applications can only access their own files

4. **Resource Limits**:
   - CPU limits control the amount of CPU time containers can use
   - Memory limits restrict RAM usage
   - Storage limits control disk space usage
   - Prevents resource exhaustion attacks

5. **IPC Communication**:
   - Applications communicate with FoldClient using Unix domain sockets
   - Sockets are mounted into the container
   - Provides secure and efficient communication

### Container Lifecycle Management

The FoldClient manages the complete lifecycle of Docker containers:

1. **Creation**: When an application is launched, FoldClient creates a Docker container with appropriate configuration
2. **Starting**: FoldClient starts the container after creation
3. **Monitoring**: FoldClient monitors the container's status
4. **Stopping**: When an application is terminated, FoldClient stops the container
5. **Removal**: FoldClient removes the container to clean up resources

### Implementation Details

The Docker-based sandboxing implementation uses the following components:

1. **Docker API**: FoldClient uses the bollard crate to interact with the Docker API
2. **Container Configuration**:
   - Base Image: Configurable (default: alpine:latest)
   - Working Directory: Set to /app, mounted from the application's directory
   - Environment Variables: App ID, token, and socket directory
   - Resource Limits: CPU, memory, and storage limits
   - Network Mode: Configurable (default: none)
   - Mounts: Application directory and socket directory
   
The build process for the social app is separate from the containerization:
1. The app is built using standard Rust tools (`cargo build`)
2. The compiled binary is then executed within a Docker container managed by FoldClient

### Containerization Capability

While not used in the current social app implementation, the system has the capability to run applications in Docker containers for sandboxing. If this capability were used, the process would work as follows:

1. **Registration**:
   ```bash
   # Register the app with FoldClient
   cargo run --bin register_app -- social-app list_schemas,query,mutation,discover_nodes
   # The output includes an app ID and token
   ```

2. **Building**:
   ```bash
   # The app is built using standard Cargo
   cargo build
   ```

3. **Container Creation and Execution**:
   ```bash
   # When using FoldClient's containerization:
   # 1. FoldClient would create a Docker container using an existing base image (not building a custom image)
   # 2. It would mount the necessary directories
   # 3. It would set up environment variables
   # 4. It would start the container with restricted permissions
   # The app would then run inside this container
   ```

4. **Inside the Container**:
   - The app would run with limited permissions
   - It would only be able to access its own files
   - It would have no direct network access
   - It would communicate with the DataFold node only through the FoldClient socket
   - All operations would be authenticated and authorized by the FoldClient

It's important to note that no Docker image building occurs as part of this process. FoldClient uses existing Docker images (typically Alpine Linux) and runs the pre-compiled Rust binary inside them.

3. **Authentication and Authorization**:
   - Each application is registered with a unique ID and token
   - FoldClient generates a keypair for each application
   - Applications authenticate with their token
   - FoldClient signs requests to the DataFold node
   - DataFold node verifies signatures

## Current Implementation: Integrated Node Approach

The current implementation of the social app uses an integrated node approach that combines the DataFold node and FoldClient functionality. In this approach, the social app connects directly to the DataFold node without running in a Docker container. This is reflected in the current implementation of the social app's `main.rs`, which uses a direct TCP connection to the DataFold node instead of the FoldClient IPC mechanism:

### Integrated Node Benefits

1. **Simplified Setup**: No need to run a separate FoldClient process
2. **Reduced Complexity**: Simpler system architecture
3. **Improved Performance**: Direct connection eliminates IPC overhead
4. **Consistent Security**: Maintains the same security model

### Direct Connection Mode

In the integrated approach:
1. The social app connects directly to the DataFold node's TCP server (port 9000)
2. It uses a simple TCP client implementation for communication (as seen in `social_app/src/main.rs`)
3. It performs the same operations (schema creation, data manipulation, querying)
4. The DataFold node handles both node operations and client operations
5. No Docker container is used in this mode - the app runs directly on the host

This is how the current implementation works, as seen in the `run_social_app.sh` script:
```bash
# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "Docker is not running. Please start Docker and try again."
    exit 1
fi

# Check if the DataFold node is running
if ! nc -z localhost 9000 > /dev/null 2>&1; then
    echo "DataFold node is not running. Please start it with:"
    echo "./test_integrated_node.sh"
    exit 1
fi

# Build the social app
echo "Building the Social App..."
cd "$(dirname "$0")"
cargo build

# Launch the app with direct connection to the DataFold node
echo "Launching the Social App..."
cargo run --bin social_app
```

Note that while the script checks if Docker is running before proceeding, the social app itself connects directly to the node without containerization in this integrated approach. **No Docker images are built or used for running the social app in the current implementation.**

## Security Considerations

While Docker provides strong isolation, several security considerations are addressed:

1. **Container Escape Prevention**:
   - Containers run without elevated privileges
   - Resource limits prevent resource exhaustion attacks
   - Network isolation prevents network-based attacks

2. **IPC Security**:
   - Socket files have appropriate permissions
   - Token authentication is required
   - Optional request signing and verification

3. **Best Practices**:
   - Principle of least privilege for application permissions
   - Regular updates to Docker and host system
   - Appropriate resource limits
   - Network isolation when possible
   - Monitoring of container resource usage

## Cross-Platform Compatibility

The Docker-based approach provides consistent sandboxing across platforms:

1. **Linux**: Uses Docker's native containerization
2. **macOS**: Uses Docker for Mac (lightweight Linux VM)
3. **Windows**: Uses Docker for Windows (Hyper-V or WSL2)

This ensures that applications run in a consistent environment regardless of the host operating system, providing better cross-platform compatibility than the previous platform-specific implementations.

## Docker Requirement Explanation

The `run_social_app.sh` script checks if Docker is running before proceeding:

```bash
# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "Docker is not running. Please start Docker and try again."
    exit 1
fi
```

This check is present because:

1. **Integrated FoldClient Functionality**: The DataFold node now includes the FoldClient functionality, which uses Docker for its containerization capabilities. Even though the current social app implementation doesn't use Docker containers directly, the DataFold node's integrated FoldClient component is designed to work with Docker.

2. **System Dependency**: Docker is a system-wide dependency for the DataFold ecosystem, as it's used for various components including the containerization capabilities.

3. **Future Compatibility**: The check ensures that Docker is available for any components that might need it, even if the current implementation of the social app doesn't use Docker containers directly.

## Latest Implementation: Direct Execution with Docker Capability

The social app has been updated to run directly without Docker containerization, while still maintaining the capability to use Docker-based sandboxing through the integrated FoldClient in the DataFold node.

### Changes Made

1. **Updated Social App**: The social app has been fixed to work with the FoldClient's IPC mechanism, allowing it to communicate with the DataFold node.

2. **DataFold Node CLI**: The DataFold node CLI has been updated to support launching apps in Docker containers using the `--launch-app` command-line argument.

3. **Run Scripts**: The run scripts (`run_social_app.sh` and `register_and_run.sh`) have been updated to run the social app directly without Docker containerization, while still maintaining the capability to use Docker-based sandboxing.

### Current Workflow

The current workflow for running the social app is as follows:

1. **Registration**: The app is registered with the FoldClient using the `register_app.rs` script, which generates a unique app ID and token.

2. **Building**: The app is built using Rust's Cargo build system (`cargo build`).

3. **Direct Execution**: The app is executed directly on the host with the following environment variables:
   - `FOLD_CLIENT_APP_ID`: The app ID generated during registration
   - `FOLD_CLIENT_APP_TOKEN`: The app token generated during registration
   - `FOLD_CLIENT_SOCKET_DIR`: The directory containing the FoldClient socket
   - `RUST_LOG`: The log level for the app

4. **Communication**: The app communicates with the DataFold node through the FoldClient's IPC mechanism.

### Why You Don't See New Docker Images

When you run the social app, you won't see any new Docker images in Docker Desktop because:

1. **Direct Execution**: In the current implementation, the social app runs directly on the host, not in a Docker container.

2. **No Custom Docker Images**: Even when Docker-based sandboxing is used, DataFold does not build custom Docker images for applications. Instead, it uses existing base images (typically Alpine Linux) and runs pre-compiled binaries inside containers created from these images.

3. **Container Creation, Not Image Building**: When Docker-based sandboxing is used, DataFold creates a Docker container from an existing image (alpine:latest). It does not create a new Docker image.

4. **Ephemeral Containers**: When Docker-based sandboxing is used, the containers are typically configured with the `--rm` flag, which means they are automatically removed when they exit. This is why you won't see any containers in Docker Desktop after running the social app.

### Docker-based Sandboxing Capability

While the current implementation doesn't use Docker containers, the system still has the capability to run applications in Docker containers for sandboxing. This capability is implemented in the `fold_client/src/docker/mod.rs` file, which provides functions for creating, starting, stopping, and removing Docker containers.

If you want to use Docker-based sandboxing, you can modify the `run_social_app.sh` script to use the Docker-based approach instead of direct execution. However, be aware that this might require additional configuration and troubleshooting, especially if you're running on a system with specific Docker settings or limitations.

For more details on the Docker-based sandboxing implementation, see the [Docker Sandboxing](docker_sandboxing.md) documentation.

## Conclusion

The social app demonstrates the core capabilities of the DataFold ecosystem, including schema-based data storage, cross-node communication, and secure application sandboxing. The social app itself is built using Rust and Cargo, not Docker.

The updated implementation now uses Docker containers for running the app, providing better isolation, resource management, and cross-platform compatibility. This ensures that the social app runs securely and efficiently across different environments.

To be absolutely clear: No Docker images are built as part of the social app. Docker is used to run the pre-compiled Rust binary in a container, providing a secure and isolated execution environment.
