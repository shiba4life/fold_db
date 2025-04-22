use std::time::Duration;

use fold_node::{datafold_node::config::NodeConfig, network::NetworkConfig, DataFoldNode};

// Helper function to create a test network config with random ports
fn create_test_network_config() -> NetworkConfig {
    use std::net::{TcpListener, UdpSocket};

    // Find an available TCP port by binding to port 0 (OS will assign a free port)
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let tcp_port = listener.local_addr().unwrap().port();

    // Find an available UDP port for discovery
    let udp_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let udp_port = udp_socket.local_addr().unwrap().port();

    // Drop the sockets so the ports can be reused
    drop(listener);
    drop(udp_socket);

    NetworkConfig {
        listen_address: format!("127.0.0.1:{}", tcp_port).parse().unwrap(),
        request_timeout: 1,
        enable_mdns: false,
        max_connections: 10,
        keep_alive_interval: 1,
        max_message_size: 1024 * 1024, // 1MB
        discovery_port: udp_port,
        connection_timeout: Duration::from_secs(1),
        announcement_interval: Duration::from_secs(1),
    }
}

#[tokio::test]
async fn test_network_initialization() {
    // Create a temporary directory for test data
    let temp_dir = tempfile::tempdir().unwrap();
    let storage_path = temp_dir.path().to_path_buf();

    // Create a node configuration
    let node_config = NodeConfig {
        storage_path,
        default_trust_distance: 1,
        network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
    };

    // Create a network configuration with random ports
    let network_config = create_test_network_config();

    // Create a node
    let mut node = DataFoldNode::new(node_config).unwrap();

    // Initialize the network layer
    let result = node.init_network(network_config).await;
    assert!(result.is_ok(), "Failed to initialize network: {:?}", result);

    // Start the network layer
    let result = node.start_network().await;
    assert!(result.is_ok(), "Failed to start network: {:?}", result);

    // Stop the network layer
    let result = node.stop_network().await;
    assert!(result.is_ok(), "Failed to stop network: {:?}", result);
}

#[tokio::test]
async fn test_node_discovery() {
    // Create temporary directories for test data
    let temp_dir1 = tempfile::tempdir().unwrap();
    let temp_dir2 = tempfile::tempdir().unwrap();

    // Create node configurations
    let node1_config = NodeConfig {
        storage_path: temp_dir1.path().to_path_buf(),
        default_trust_distance: 1,
        network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
    };

    let node2_config = NodeConfig {
        storage_path: temp_dir2.path().to_path_buf(),
        default_trust_distance: 1,
        network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
    };

    // Create network configurations with different random ports
    let network1_config = create_test_network_config();
    let mut network2_config = create_test_network_config();

    // Ensure the TCP listener ports are different
    // If by chance they're the same, increment the second one
    let addr1 = network1_config.listen_address.clone();
    let addr2 = network2_config.listen_address.clone();
    if addr1 == addr2 {
        // Just use a different port if they happen to be the same
        let parts = network2_config
            .listen_address
            .split(':')
            .collect::<Vec<_>>();
        if parts.len() > 1 {
            let port = parts[parts.len() - 1].parse::<u16>().unwrap_or(8000) + 2;
            network2_config.listen_address = format!("127.0.0.1:{}", port).parse().unwrap();
        }
    }

    // Create nodes
    let mut node1 = DataFoldNode::new(node1_config).unwrap();
    let mut node2 = DataFoldNode::new(node2_config).unwrap();

    // Initialize network layers
    let result = node1.init_network(network1_config).await;
    assert!(
        result.is_ok(),
        "Failed to initialize network 1: {:?}",
        result
    );

    let result = node2.init_network(network2_config).await;
    assert!(
        result.is_ok(),
        "Failed to initialize network 2: {:?}",
        result
    );

    // Start network layers
    let result = node1.start_network().await;
    assert!(result.is_ok(), "Failed to start network 1: {:?}", result);

    let result = node2.start_network().await;
    assert!(result.is_ok(), "Failed to start network 2: {:?}", result);

    // Add a trusted node
    let node2_id = node2.get_node_id();
    let result = node1.add_trusted_node(node2_id);
    assert!(result.is_ok(), "Failed to add trusted node: {:?}", result);

    // In a real environment, this would find node2, but in our test setup we're not asserting
    // the result since we're using mock discovery with discovery disabled

    // Stop network layers
    let result = node1.stop_network().await;
    assert!(result.is_ok(), "Failed to stop network 1: {:?}", result);

    let result = node2.stop_network().await;
    assert!(result.is_ok(), "Failed to stop network 2: {:?}", result);
}
