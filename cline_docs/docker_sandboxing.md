# Docker Sandboxing in DataFold

This document explains how Docker is used for sandboxing applications in the DataFold ecosystem.

## Overview

DataFold uses Docker containers to provide a secure, isolated execution environment for applications. This approach offers several benefits:

- **Process Isolation**: Applications run in isolated namespaces
- **Resource Limits**: CPU, memory, and storage usage can be controlled
- **Network Isolation**: Applications can be restricted from accessing the network
- **File System Isolation**: Applications only have access to specific directories

## How Docker is Used in DataFold

It's important to understand that DataFold uses Docker in a specific way:

1. **No Custom Docker Images**: DataFold does not build custom Docker images for applications. Instead, it uses existing base images (typically Alpine Linux) and runs pre-compiled binaries inside containers created from these images.

2. **Container Creation, Not Image Building**: When you run an application like the social app, DataFold creates a Docker container from an existing image (alpine:latest). It does not create a new Docker image.

3. **Ephemeral Containers**: The containers are typically configured with the `--rm` flag, which means they are automatically removed when they exit. This is why you won't see any containers in Docker Desktop after running the social app.

4. **Direct Execution in Latest Implementation**: In the most recent implementation of the social app, we've modified the `run_social_app.sh` script to run the application directly without Docker containerization. This was done to simplify the development process, but the Docker-based sandboxing capability is still available in the system.

## Why You Don't See New Docker Images

When you run the social app, you won't see any new Docker images in Docker Desktop because:

1. **Using Existing Images**: The system uses the existing `alpine:latest` image, which you already have on your system.

2. **No Image Building**: The system does not build new Docker images; it only creates containers from existing images.

3. **Ephemeral Containers**: The containers are removed after they exit, so you won't see them in Docker Desktop either.

4. **Direct Execution**: In the current implementation, the social app runs directly on the host, not in a Docker container.

## Docker Container Lifecycle

When Docker-based sandboxing is used, the container lifecycle is as follows:

1. **Creation**: A container is created from the `alpine:latest` image with specific configuration:
   - Mounts for the application directory and socket directory
   - Environment variables for authentication
   - Resource limits for CPU, memory, and storage
   - Network isolation

2. **Execution**: The pre-compiled application binary runs inside the container.

3. **Termination**: When the application exits, the container stops.

4. **Removal**: If the container was created with the `--rm` flag, it is automatically removed.

## Checking Docker Usage

You can verify Docker usage with the following commands:

```bash
# List Docker images (you should see alpine:latest, but no custom images)
docker images

# List running containers (you might see a container while the app is running)
docker ps

# List all containers, including stopped ones (you won't see any if they were removed)
docker ps -a
```

## Current Implementation

In the current implementation of the social app, we've modified the `run_social_app.sh` script to run the application directly without Docker containerization. This was done to simplify the development process and avoid issues with Docker configuration.

The script now sets the necessary environment variables and runs the social app binary directly:

```bash
# Set environment variables
export FOLD_CLIENT_APP_ID="$APP_ID"
export FOLD_CLIENT_APP_TOKEN="$APP_TOKEN"
export FOLD_CLIENT_SOCKET_DIR="$HOME/.datafold/sockets"
export RUST_LOG="info"

# Run the social app
echo "Running social_app..."
../target/debug/social_app
```

This approach still maintains the same functionality, but without the Docker containerization layer. The application connects directly to the DataFold node and performs the same operations.

## Docker-based Sandboxing Capability

While the current implementation doesn't use Docker containers, the system still has the capability to run applications in Docker containers for sandboxing. This capability is implemented in the `fold_client/src/docker/mod.rs` file, which provides functions for creating, starting, stopping, and removing Docker containers.

If you want to use Docker-based sandboxing, you can modify the `run_social_app.sh` script to use the Docker-based approach instead of direct execution. However, be aware that this might require additional configuration and troubleshooting, especially if you're running on a system with specific Docker settings or limitations.

## Conclusion

Docker is used in DataFold for sandboxing applications, but in a specific way that doesn't involve building custom Docker images. Instead, it creates containers from existing images and runs pre-compiled binaries inside them. In the current implementation of the social app, we're running the application directly without Docker containerization to simplify the development process.
