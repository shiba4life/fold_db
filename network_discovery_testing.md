# Testing Network Discovery in FoldDB

This document explains how to test the network discovery functionality in the FoldDB distributed database system.

## Overview

Network discovery is a critical component of FoldDB that allows nodes to:
1. Announce their presence on the network
2. Discover other nodes
3. Establish connections
4. Share schema information
5. Execute queries across the network

## Test Components

The network discovery tests are organized into several components:

1. **Basic Discovery Initialization**: Tests that the discovery service can be properly initialized and started.
2. **Discovery Enabled vs. Disabled**: Tests the behavior when discovery is enabled or disabled.
3. **Manual Node Connection**: Tests manually adding trusted nodes and connecting to them.
4. **Multi-Node Discovery**: Tests discovery with multiple nodes running simultaneously.

## Running the Tests

### Using the Test Script

The simplest way to run the network discovery tests is to use the provided script:

```bash
./test_network_discovery.sh
```

This script runs the network discovery tests with the `--nocapture` flag to show the test output, including node IDs and discovery information.

The script also supports several options:

```bash
# Run only basic tests that don't require network discovery
./test_network_discovery.sh --basic

# Run tests without showing output
./test_network_discovery.sh --no-output

# Show help information
./test_network_discovery.sh --help
```

The `--basic` option is particularly useful in environments where UDP broadcasts are not supported or are blocked by firewalls, as it runs only the tests that don't rely on actual network discovery.

### Manual Test Execution

You can also run the tests manually using Cargo:

```bash
# Run all network discovery tests
cargo test --test network_discovery_tests

# Run a specific test
cargo test --test network_discovery_tests test_node_discovery_enabled

# Run with output displayed
cargo test --test network_discovery_tests -- --nocapture
```

## Test Implementation Details

The network discovery tests are implemented in `tests/unit_tests/network_discovery_tests.rs` and include:

1. **test_discovery_initialization**: Tests basic initialization of the discovery service.
2. **test_node_discovery_enabled**: Tests node discovery with discovery enabled.
3. **test_node_discovery_disabled**: Tests that no nodes are discovered when discovery is disabled.
4. **test_manual_node_connection**: Tests manually adding and connecting to trusted nodes.
5. **test_discovery_with_multiple_nodes**: Tests discovery with multiple nodes running simultaneously.

## Network Discovery Architecture

The network discovery system consists of several components:

1. **NodeDiscovery**: Handles node discovery and presence management.
2. **ConnectionManager**: Manages connections to remote nodes.
3. **NetworkCore**: Coordinates all network operations.
4. **NetworkManager**: Provides a simplified API for network operations.

## Testing in Production Environments

When testing in production environments, consider the following:

1. **Network Configuration**: Ensure proper network configuration, including firewall rules to allow UDP broadcasts on the discovery port.
2. **Trust Configuration**: Configure appropriate trust distances for production use.
3. **Security**: In production, ensure proper TLS configuration and public key validation.
4. **Monitoring**: Monitor node discovery and connection events to ensure proper operation.

## Troubleshooting

If you encounter issues with network discovery:

1. **Check Logs**: Look for error messages related to socket binding, broadcasts, or connection attempts.
2. **Verify Network**: Ensure that UDP broadcasts are allowed on your network.
3. **Check Ports**: Verify that the discovery and connection ports are available and not blocked.
4. **Test Connectivity**: Use tools like `netcat` or `telnet` to test basic connectivity between nodes.
5. **Network Errors**: Common network errors include:
   - "No route to host" (os error 65): This often occurs when UDP broadcasts are blocked or not supported in your network environment.
   - "Permission denied" (os error 13): This may occur if you don't have sufficient permissions to bind to the specified ports.
   - "Address already in use" (os error 48): This occurs if the port is already being used by another process.

### Handling Network Errors in Tests

The test suite is designed to handle network errors gracefully:

1. Tests will log discovery errors but continue execution
2. Tests verify that the discovery process can be initiated, even if actual discovery fails
3. For environments where UDP broadcasts are not supported, focus on the manual node connection tests

If you need to run tests in an environment where network discovery is not possible:

```bash
# Run only the tests that don't require network discovery
cargo test --test network_discovery_tests test_discovery_initialization test_node_discovery_disabled test_manual_node_connection
```

## Extending the Tests

To extend the network discovery tests:

1. Add new test functions to `tests/unit_tests/network_discovery_tests.rs`.
2. Update the `mod.rs` file if you create new test modules.
3. Consider adding integration tests for end-to-end testing of network functionality.

## Related Documentation

For more information, see:
- [Network Layer Design](cline_docs/network_layer.md)
- [DataFold Node Documentation](src/datafold_node/node.md)
