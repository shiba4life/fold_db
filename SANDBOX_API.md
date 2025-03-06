# Datafold Sandbox API Access

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
./setup_sandbox_local.sh
```

This script will:
- Create the internal Docker network
- Build the Datafold API container
- Start the Datafold API service

### Quick Demo

To quickly set up the environment and run the tests in a single command:

```bash
./run_sandbox_api_demo.sh
```

This script will:
1. Set up the sandbox environment
2. Run the example application in a sandboxed container
3. Test that the sandbox security measures are working correctly
4. Test the API access from the sandboxed container

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

For maximum isolation, you can use Unix socket communication. This provides an even higher level of security by completely removing network access from the container:

```bash
docker run --rm \
  --network=none \
  --cap-drop=ALL \
  --security-opt no-new-privileges \
  -v ./socket/datafold.sock:/datafold.sock \
  --env DATAFOLD_API_SOCKET=/datafold.sock \
  your-image-name
```

The Unix socket implementation provides several security benefits:
- Complete network isolation (--network=none)
- No need for an internal Docker network
- Direct file-based communication without network stack
- Reduced attack surface

## Interacting with the Datafold API

### Network-based API Access

Inside the container, you can access the Datafold API using:

```bash
curl http://datafold-api:8080/run_app/<app_id>
```

### Unix Socket-based API Access

For Unix socket communication, applications can use the socket path to communicate with the Datafold API:

```bash
curl --unix-socket /datafold.sock http://localhost/run_app/<app_id>
```

In Node.js applications, you can use axios with socketPath:

```javascript
const axios = require('axios');
const client = axios.create({
  socketPath: '/datafold.sock',
  baseURL: 'http://localhost',
  timeout: 5000
});

// Then use the client normally
client.get('/query').then(response => {
  console.log(response.data);
});
```

In Python applications, you can use requests with unix_socket:

```python
import requests
import urllib.parse

response = requests.get(
    'http://localhost/query',
    unix_socket='/datafold.sock'
)
print(response.json())
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

### Configuring the Datafold API for Unix Socket Communication

The Datafold API can be configured to use Unix socket communication by setting the following environment variables:

```bash
USE_UNIX_SOCKET=true
UNIX_SOCKET_PATH=/var/run/datafold.sock
```

These can be set in the docker-compose-local.yml file or when running the container directly.

### Troubleshooting Unix Socket Communication

If you get a "permission denied" error when using the Unix socket:

1. Check that the socket exists at `./socket/datafold.sock`
2. Ensure the socket has the correct permissions (should be 777 for testing)
3. Verify the volume mount is correct in the Docker run command
4. Check that the Datafold API is configured to use Unix socket communication

You can test the Unix socket directly with:

```bash
curl --unix-socket ./socket/datafold.sock http://localhost/
```

## Stopping the Environment

To stop the Datafold API container and clean up:

```bash
docker-compose -f docker-compose-local.yml down
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
```
