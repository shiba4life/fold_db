# Docker Sandboxing in FoldClient

This document provides a detailed explanation of how the Docker sandboxing implementation works in FoldClient.

## Overview

FoldClient uses Docker to provide a secure and isolated environment for applications to run in. This approach offers several advantages over the previous platform-specific sandbox implementations:

1. **Cross-Platform Compatibility**: Docker runs on Linux, macOS, and Windows, providing a consistent sandboxing experience across platforms.
2. **Strong Isolation**: Docker containers provide stronger isolation than the previous sandbox implementations, especially on macOS and Windows.
3. **Resource Management**: Docker provides built-in resource management capabilities, allowing for fine-grained control over CPU, memory, and storage usage.
4. **Network Isolation**: Docker's networking capabilities allow for precise control over network access.
5. **Simplified Implementation**: Using Docker simplifies the sandbox implementation, as we can leverage Docker's existing security features rather than implementing platform-specific sandboxing mechanisms.

## Docker Sandboxing Features

### Process Isolation

Docker containers run in isolated namespaces, providing process isolation from the host system and other containers. This prevents applications from accessing or interfering with processes outside their container.

### Network Isolation

FoldClient can configure Docker containers to run with or without network access:

- **No Network Access**: Containers can be configured to run with the `none` network driver, which completely disables network access.
- **Limited Network Access**: Containers can be configured to run with a specific Docker network, allowing for controlled network access.

### File System Isolation

Docker containers have their own isolated file system. FoldClient mounts the application's working directory into the container, allowing the application to access only its own files.

### Resource Limits

FoldClient can configure Docker containers with resource limits:

- **CPU Limits**: Limits the amount of CPU time the container can use.
- **Memory Limits**: Limits the amount of memory the container can use.
- **Storage Limits**: Limits the amount of disk space the container can use.

### IPC Communication

Applications in Docker containers communicate with the FoldClient using a Unix domain socket that is mounted into the container. This provides a secure and efficient communication channel between the application and the FoldClient.

## Implementation Details

### Docker API

FoldClient uses the [bollard](https://docs.rs/bollard) crate to interact with the Docker API. This allows FoldClient to create, start, stop, and manage Docker containers programmatically.

### Container Creation

When an application is launched, FoldClient creates a Docker container with the following configuration:

1. **Base Image**: The container uses a configurable base image (default: `alpine:latest`).
2. **Command**: The container runs the specified command with the provided arguments.
3. **Working Directory**: The container's working directory is set to `/app`, which is mounted from the application's working directory on the host.
4. **Environment Variables**: The container is provided with environment variables for the application ID, token, and socket directory.
5. **Resource Limits**: The container is configured with the specified CPU, memory, and storage limits.
6. **Network Mode**: The container is configured with the specified network mode (default: `none` for no network access).
7. **Mounts**: The container has the application's working directory mounted at `/app` and the socket directory mounted at `/tmp/fold_client_sockets`.

### Container Lifecycle

FoldClient manages the lifecycle of Docker containers:

1. **Creation**: When an application is launched, FoldClient creates a Docker container.
2. **Starting**: After creation, FoldClient starts the container.
3. **Monitoring**: FoldClient monitors the container's status to detect when it exits.
4. **Stopping**: When an application is terminated, FoldClient stops the container.
5. **Removal**: After stopping, FoldClient removes the container to clean up resources.

### IPC Communication

Applications in Docker containers communicate with the FoldClient using a Unix domain socket:

1. **Socket Creation**: FoldClient creates a Unix domain socket for each application.
2. **Socket Mounting**: The socket is mounted into the container at `/tmp/fold_client_sockets/{app_id}.sock`.
3. **Communication**: The application uses the FoldClient IPC client to communicate with the FoldClient over the socket.

## Security Considerations

While Docker provides strong isolation, it's important to be aware of potential security issues:

### Container Escape

Docker containers are not completely secure against determined attackers with sufficient privileges. Potential container escape vectors include:

1. **Kernel Vulnerabilities**: Vulnerabilities in the Linux kernel could potentially be exploited to escape the container.
2. **Docker Daemon Vulnerabilities**: Vulnerabilities in the Docker daemon could potentially be exploited to escape the container.
3. **Privileged Containers**: Containers running with elevated privileges (e.g., `--privileged`) have more potential escape vectors.

FoldClient mitigates these risks by:

1. **Unprivileged Containers**: Containers are run without elevated privileges.
2. **Resource Limits**: Containers are run with resource limits to prevent resource exhaustion attacks.
3. **Network Isolation**: Containers can be run without network access to prevent network-based attacks.

### Resource Exhaustion

While resource limits can be configured, a malicious application could still attempt to exhaust resources within its limits. FoldClient mitigates this risk by:

1. **Default Limits**: FoldClient sets default resource limits for containers.
2. **Configurable Limits**: Resource limits can be configured on a per-application basis.
3. **Auto-Removal**: Containers can be configured to be automatically removed when they exit, cleaning up resources.

### IPC Security

The IPC mechanism uses Unix domain sockets, which are generally secure but could potentially be accessed by other processes with sufficient privileges. FoldClient mitigates this risk by:

1. **Socket Permissions**: Socket files are created with appropriate permissions to prevent unauthorized access.
2. **Token Authentication**: Applications must provide a valid token to authenticate with the FoldClient.
3. **Signature Verification**: Applications can optionally sign their requests, which FoldClient verifies.

## Best Practices

To maximize the security of the Docker sandboxing implementation:

1. **Principle of Least Privilege**: Only grant applications the permissions they need.
2. **Regular Updates**: Keep Docker and the host system up to date to mitigate known vulnerabilities.
3. **Resource Limits**: Set appropriate resource limits for applications to prevent resource exhaustion.
4. **Network Isolation**: Disable network access for applications that don't need it.
5. **Monitoring**: Monitor container resource usage and behavior for signs of compromise.

## Comparison with Previous Sandbox Implementation

The Docker-based sandbox implementation offers several advantages over the previous platform-specific implementations:

### Linux

The previous Linux sandbox implementation used namespaces, cgroups, and seccomp to provide isolation. The Docker-based implementation leverages these same mechanisms but with a more mature and well-tested implementation.

### macOS

The previous macOS sandbox implementation used the `sandbox-exec` command, which provides weaker isolation than Docker. Docker for Mac uses a lightweight Linux VM to run containers, providing stronger isolation.

### Windows

The previous Windows sandbox implementation used job objects and integrity levels, which provide weaker isolation than Docker. Docker for Windows uses either Hyper-V or WSL2 to run containers, providing stronger isolation.

## Conclusion

The Docker-based sandbox implementation in FoldClient provides a secure, cross-platform, and feature-rich sandboxing solution for applications. By leveraging Docker's existing security features and resource management capabilities, FoldClient can provide a consistent and robust sandboxing experience across platforms.
