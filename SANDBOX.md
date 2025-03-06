# Datafold Sandbox Environment

This document explains how to use the secure Docker sandbox environment for running third-party containers with Datafold API access.

## Overview

The Datafold Sandbox Environment provides a secure, isolated environment where third-party Docker containers can run without external network access, yet can securely interact with the Datafold API. This is achieved through:

1. An internal Docker network that isolates containers from the external internet
2. Strict security measures for sandboxed containers
3. A dedicated Datafold API container that provides controlled access to Datafold functionality
4. Optional Unix socket communication for maximum isolation

## Security Measures

The sandbox implements several security measures:

- **Network Isolation**: Containers run on an internal Docker network with no external internet access
- **Capability Restrictions**: All Linux capabilities are dropped by default
- **Privilege Escalation Prevention**: The `no-new-privileges` flag prevents privilege escalation
- **Resource Limits**: CPU, memory, and process limits are enforced
- **Read-Only Filesystem**: Containers run with a read-only root filesystem
- **Seccomp Profiles**: Default seccomp profiles restrict system calls

## Setup

### Prerequisites

- Docker installed
- Docker Compose installed
- Rust toolchain (for building the Datafold API)

### Installation

1. Clone the Datafold repository
2. Run the setup script:

```bash
./setup_sandbox.sh
```

This script will:
- Create the internal Docker network
- Build the Datafold API container
- Start the Datafold API service

### Quick Demo

To quickly set up the environment and run the tests in a single command:

```bash
./run_sandbox_demo.sh
```

This script will:
1. Set up the sandbox environment
2. Run the example application in a sandboxed container
3. Test that the sandbox security measures are working correctly

## Running Sandboxed Containers

### Network-based Communication

To run a container with network-based access to the Datafold API:

```bash
docker run --rm \
  --network=datafold_internal_network \
  --cap-drop=ALL \
  --security-opt no-new-privileges \
  --env DATAFOLD_API_HOST=datafold-api \
  --env DATAFOLD_API_PORT=8080 \
  your-image-name
```

### Unix Socket Communication

For maximum isolation, you can use Unix socket communication:

```bash
docker run --rm \
  --network=none \
  --cap-drop=ALL \
  --security-opt no-new-privileges \
  -v /var/run/datafold.sock:/datafold.sock \
  --env DATAFOLD_API_SOCKET=/datafold.sock \
  your-image-name
```

## Interacting with the Datafold API

### Network-based API Access

Inside the container, you can access the Datafold API using:

```bash
curl http://datafold-api:8080/run_app/<app_id>
```

### Unix Socket-based API Access

For Unix socket communication:

```bash
curl --unix-socket /datafold.sock http://localhost/run_app/<app_id>
```

## API Endpoints

The following API endpoints are available:

- `/query` - Execute a query against the database
- `/schema` - Get schema information
- `/node` - Interact with other nodes in the network

## Programmatic Usage

The Datafold Node provides a `SandboxManager` class that can be used to programmatically manage sandboxed containers:

```rust
use crate::datafold_node::sandbox::{SandboxManager, SandboxConfig};

// Create a sandbox manager
let config = SandboxConfig::default();
let sandbox_manager = SandboxManager::new(config)?;

// Register a container
sandbox_manager.register_container("container-id", "container-name", "image-name", None)?;

// Start the container
sandbox_manager.start_container("container-id")?;

// Proxy a request to the Datafold API
let request = Request {
    container_id: "container-id".to_string(),
    path: "/query".to_string(),
    method: "GET".to_string(),
    headers: HashMap::new(),
    body: None,
};
let response = sandbox_manager.proxy_request(request)?;
```

## Troubleshooting

### Container Cannot Connect to Datafold API

If a container cannot connect to the Datafold API, check:

1. The container is on the correct network (`datafold_internal_network`)
2. The Datafold API container is running
3. The correct environment variables are set (`DATAFOLD_API_HOST` and `DATAFOLD_API_PORT`)

### Permission Denied for Unix Socket

If you get a "permission denied" error when using the Unix socket:

1. Check that the socket exists at `/var/run/datafold.sock`
2. Ensure the socket has the correct permissions
3. Verify the volume mount is correct in the Docker run command

## Stopping the Environment

To stop the Datafold API container and clean up:

```bash
docker-compose down
```

## Advanced Configuration

The sandbox environment can be configured by modifying the `SandboxConfig` struct:

```rust
let config = SandboxConfig {
    network_name: "custom_network".to_string(),
    default_resource_limits: ResourceLimits {
        cpu_limit: Some(1.0),
        memory_limit: Some(1024 * 1024 * 1024), // 1 GB
        pids_limit: Some(200),
        ..Default::default()
    },
    use_unix_socket: true,
    ..Default::default()
};
