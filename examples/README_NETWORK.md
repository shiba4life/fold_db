# Network Layer Examples

This directory contains examples demonstrating how to use the network layer in FoldDB.

## Running the DataFold Node with Network Support

The `datafold_node` binary has been updated to support network functionality. You can run it with the following command:

```bash
cargo run --bin datafold_node -- --port 9000
```

This will start a DataFold node with the network service listening on port 9000. The node will automatically discover other nodes on the local network using mDNS.

## Running Multiple Nodes

The `run_network_nodes.sh` script demonstrates how to run multiple DataFold nodes with network support. Each node will run in its own process and use a different port.

```bash
# Make the script executable
chmod +x examples/run_network_nodes.sh

# Run the script
./examples/run_network_nodes.sh
```

This will start three nodes on ports 9001, 9002, and 9003. Each node will automatically discover the others using mDNS.

## Checking Remote Schemas

The `check_remote_schemas.rs` example demonstrates how to check which schemas are available on remote nodes. It creates two nodes, loads some schemas into one of them, and then checks which schemas are available on each node.

```bash
cargo run --example check_remote_schemas
```

This example shows:
1. How to create and initialize nodes with network support
2. How to load schemas into a node
3. How to check which schemas are available on remote nodes

## Network Configuration Options

The network layer can be configured with various options:

```rust
let network_config = NetworkConfig::new("/ip4/0.0.0.0/tcp/9000")
    .with_mdns(true)                  // Enable mDNS discovery
    .with_request_timeout(30)         // Set request timeout to 30 seconds
    .with_max_connections(50)         // Allow up to 50 concurrent connections
    .with_keep_alive_interval(20)     // Send keep-alive every 20 seconds
    .with_max_message_size(1_000_000); // Set maximum message size to 1MB
```

## Network Layer Integration with DataFoldNode

The DataFoldNode has been extended with network capabilities:

```rust
// Initialize the network layer
node.init_network(network_config).await?;

// Start the network service
node.start_network("/ip4/0.0.0.0/tcp/9000").await?;

// Check which schemas are available on a remote peer
let available_schemas = node.check_remote_schemas(peer_id, schema_names).await?;
```

## Security Considerations

The network layer uses libp2p's built-in security features:
- Noise protocol encryption for all connections
- PeerId-based authentication
- Automatic connection management
- Node validation checks
- Error handling for security violations

## Future Enhancements

Future enhancements to the network layer may include:
- Schema synchronization capabilities
- Kademlia DHT for wider peer discovery
- Node reputation tracking
- Custom protocol handlers for specialized operations
