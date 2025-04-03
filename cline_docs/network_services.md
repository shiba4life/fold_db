# DataFold Network Services Documentation

## Overview

The DataFold system uses two distinct network services to enable distributed data storage and querying:

1. **Network Service**: Handles peer-to-peer (P2P) communication between nodes
2. **TCP Server**: Handles client-to-node communication

These services work together to create a distributed social network where data can be stored on different nodes but queried from any node in the network.

## Network Service

### Purpose

The Network Service is responsible for node-to-node communication in the DataFold distributed system. It enables:

- Node discovery using mDNS
- Schema sharing between nodes
- Cross-node querying
- Data synchronization

### Implementation

The Network Service is implemented in the `NetworkCore` class in `src/network/core.rs`. It uses libp2p for P2P communication and provides the following functionality:

- **Node Discovery**: Uses mDNS to discover other nodes on the local network
- **Schema Protocol**: Implements a custom protocol for sharing schema information between nodes
- **Peer Management**: Maintains a list of known peers and their trust distances
- **Remote Querying**: Allows querying data from remote nodes

### Configuration

The Network Service is configured using the `NetworkConfig` struct, which includes:

- **Listen Address**: The address and port to listen on (e.g., `/ip4/127.0.0.1/tcp/9001`)
- **mDNS Discovery**: Whether to enable mDNS discovery
- **Discovery Port**: The port to use for mDNS discovery
- **Announcement Interval**: How often to announce the node's presence

### Initialization and Starting

```rust
// Initialize the network layer
let network_config = NetworkConfig::new("/ip4/127.0.0.1/tcp/9001");
node.init_network(network_config).await?;

// Start the network service
node.start_network_with_address("/ip4/127.0.0.1/tcp/9001").await?;
```

## TCP Server

### Purpose

The TCP Server is responsible for client-to-node communication in the DataFold system. It enables:

- Client connections to nodes
- Schema creation and management
- Data querying and mutation
- Remote node discovery

### Implementation

The TCP Server is implemented in the `TcpServer` class in `src/datafold_node/tcp_server.rs`. It uses tokio for asynchronous TCP communication and provides the following functionality:

- **Connection Handling**: Accepts and manages client connections
- **Request Processing**: Processes client requests for operations like querying and mutation
- **Schema Management**: Allows clients to create, update, and delete schemas
- **Node Discovery**: Allows clients to discover remote nodes

### Configuration

The TCP Server is configured with:

- **Node Reference**: A reference to the DataFoldNode instance
- **Port**: The port to listen on (e.g., 8001)

### Initialization and Starting

```rust
// Create and start the TCP server
let tcp_server = TcpServer::new(node.clone(), 8001).await?;
let tcp_server_handle = tokio::spawn(async move {
    if let Err(e) = tcp_server.run().await {
        eprintln!("TCP server error: {}", e);
    }
});
```

## How They Work Together

1. **Node Setup**:
   - Each node initializes and starts its Network Service for P2P communication
   - Each node initializes and starts its TCP Server for client connections

2. **Client Connection**:
   - Clients connect to a node's TCP Server to interact with the DataFold system
   - The TCP Server processes client requests and forwards them to the node

3. **Cross-Node Querying**:
   - A client connects to Node A's TCP Server
   - The client requests data from Node B through Node A
   - Node A's Network Service communicates with Node B's Network Service to retrieve the data
   - Node A's TCP Server returns the data to the client

4. **Data Flow Example**:
   ```
   Client → TCP Server (Node A) → Network Service (Node A) → 
   Network Service (Node B) → Node B Data → Network Service (Node B) → 
   Network Service (Node A) → TCP Server (Node A) → Client
   ```

## Port Configuration

To avoid conflicts, the Network Service and TCP Server should use different ports:

- **Network Service**: Typically uses ports like 9001, 9002, etc.
- **TCP Server**: Typically uses ports like 8001, 8002, etc.

Example configuration for two nodes:

| Node | Network Service Port | TCP Server Port |
|------|---------------------|----------------|
| 1    | 9001                | 8001           |
| 2    | 9002                | 8002           |

## Best Practices

1. **Port Separation**: Always use different ports for the Network Service and TCP Server to avoid conflicts
2. **Error Handling**: Properly handle errors from both services
3. **Async Tasks**: Run the TCP Server in a separate async task to avoid blocking
4. **Shutdown**: Properly shut down both services when the node is shutting down
5. **Trust Management**: Configure appropriate trust distances between nodes

## Example Implementation

See the `social_app_two_nodes.rs` example for a complete implementation of a two-node system with both services running.
