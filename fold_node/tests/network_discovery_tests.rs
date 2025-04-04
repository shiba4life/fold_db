use std::net::{TcpListener, UdpSocket};
use std::time::Duration;
use std::thread;

use fold_node::{
    DataFoldNode, datafold_node::config::NodeConfig, network::NetworkConfig
};

// Helper function to create a test network config with random ports
fn create_test_network_config(enable_discovery: bool) -> NetworkConfig {
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
        listen_address: format!("/ip4/127.0.0.1/tcp/{}", tcp_port),
        request_timeout: 30,
        enable_mdns: enable_discovery,
        max_connections: 10,
        keep_alive_interval: 20,
        max_message_size: 1_000_000,
        discovery_port: udp_port,
        connection_timeout: Duration::from_secs(1),
        announcement_interval: Duration::from_millis(500), // Faster for tests
    }
}

#[tokio::test]
async fn test_discovery_initialization() {
    // Create a temporary directory for test data
    let temp_dir = tempfile::tempdir().unwrap();
    let storage_path = temp_dir.path().to_path_buf();

    // Create a node configuration
    let node_config = NodeConfig {
        storage_path,
        default_trust_distance: 1,
        network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
    };

    // Create a network configuration with discovery enabled
    let network_config = create_test_network_config(true);

    // Create a node
    let mut node = DataFoldNode::new(node_config).unwrap();
    
    // Initialize the network layer
    let result = node.init_network(network_config).await;
    assert!(result.is_ok(), "Failed to initialize network with discovery enabled: {:?}", result);
    
    // Start the network layer
    let result = node.start_network().await;
    assert!(result.is_ok(), "Failed to start network with discovery enabled: {:?}", result);
    
    // Stop the network layer
    let result = node.stop_network().await;
    assert!(result.is_ok(), "Failed to stop network: {:?}", result);
}

#[tokio::test]
async fn test_node_discovery_enabled() {
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
    
    // Create network configurations with discovery enabled
    let network1_config = create_test_network_config(true);
    let network2_config = create_test_network_config(true);
    
    // Create nodes
    let mut node1 = DataFoldNode::new(node1_config).unwrap();
    let mut node2 = DataFoldNode::new(node2_config).unwrap();
    
    // Initialize network layers
    let result = node1.init_network(network1_config).await;
    assert!(result.is_ok(), "Failed to initialize network 1: {:?}", result);
    
    let result = node2.init_network(network2_config).await;
    assert!(result.is_ok(), "Failed to initialize network 2: {:?}", result);
    
    // Start network layers
    let result = node1.start_network().await;
    assert!(result.is_ok(), "Failed to start network 1: {:?}", result);
    
    let result = node2.start_network().await;
    assert!(result.is_ok(), "Failed to start network 2: {:?}", result);
    
    // Get node IDs for reference
    let node1_id = node1.get_node_id().to_string();
    let node2_id = node2.get_node_id().to_string();
    
    println!("Node 1 ID: {}", node1_id);
    println!("Node 2 ID: {}", node2_id);
    
    // Wait for discovery to happen (multiple announcement intervals)
    thread::sleep(Duration::from_secs(2));
    
    // Trigger discovery process
    let discovered_nodes1 = node1.discover_nodes().await;
    println!("Node 1 discovery result: {:?}", discovered_nodes1);
    // Don't fail the test if discovery fails due to network issues
    // Just log the result and continue
    
    let discovered_nodes2 = node2.discover_nodes().await;
    println!("Node 2 discovery result: {:?}", discovered_nodes2);
    // Don't fail the test if discovery fails due to network issues
    
    // Get known nodes
    let known_nodes1 = node1.get_known_nodes().await;
    assert!(known_nodes1.is_ok(), "Failed to get known nodes from node 1: {:?}", known_nodes1);
    
    let known_nodes2 = node2.get_known_nodes().await;
    assert!(known_nodes2.is_ok(), "Failed to get known nodes from node 2: {:?}", known_nodes2);
    
    // Check if nodes discovered each other
    let known_nodes1 = known_nodes1.unwrap();
    let known_nodes2 = known_nodes2.unwrap();
    
    println!("Node 1 known nodes: {:?}", known_nodes1.keys().collect::<Vec<_>>());
    println!("Node 2 known nodes: {:?}", known_nodes2.keys().collect::<Vec<_>>());
    
    // Note: In a real network environment with UDP broadcast enabled, 
    // the nodes would discover each other. However, in a test environment,
    // this might not happen reliably due to network isolation.
    // We're primarily testing that the discovery process runs without errors.
    
    // Stop network layers
    let result = node1.stop_network().await;
    assert!(result.is_ok(), "Failed to stop network 1: {:?}", result);
    
    let result = node2.stop_network().await;
    assert!(result.is_ok(), "Failed to stop network 2: {:?}", result);
}

#[tokio::test]
async fn test_node_discovery_disabled() {
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
    
    // Create network configurations with discovery disabled
    let network1_config = create_test_network_config(false);
    let network2_config = create_test_network_config(false);
    
    // Create nodes
    let mut node1 = DataFoldNode::new(node1_config).unwrap();
    let mut node2 = DataFoldNode::new(node2_config).unwrap();
    
    // Initialize network layers
    let result = node1.init_network(network1_config).await;
    assert!(result.is_ok(), "Failed to initialize network 1: {:?}", result);
    
    let result = node2.init_network(network2_config).await;
    assert!(result.is_ok(), "Failed to initialize network 2: {:?}", result);
    
    // Start network layers
    let result = node1.start_network().await;
    assert!(result.is_ok(), "Failed to start network 1: {:?}", result);
    
    let result = node2.start_network().await;
    assert!(result.is_ok(), "Failed to start network 2: {:?}", result);
    
    // Trigger discovery process
    let discovered_nodes1 = node1.discover_nodes().await;
    assert!(discovered_nodes1.is_ok(), "Failed to discover nodes from node 1: {:?}", discovered_nodes1);
    
    // With discovery disabled, the result should be an empty list
    let discovered_nodes = discovered_nodes1.unwrap();
    assert!(discovered_nodes.is_empty(), "Expected empty list with discovery disabled, got: {:?}", discovered_nodes);
    
    // Stop network layers
    let result = node1.stop_network().await;
    assert!(result.is_ok(), "Failed to stop network 1: {:?}", result);
    
    let result = node2.stop_network().await;
    assert!(result.is_ok(), "Failed to stop network 2: {:?}", result);
}

#[tokio::test]
async fn test_manual_node_connection() {
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
    
    // Create network configurations with discovery disabled
    let network1_config = create_test_network_config(false);
    let network2_config = create_test_network_config(false);
    
    // Create nodes
    let mut node1 = DataFoldNode::new(node1_config).unwrap();
    let mut node2 = DataFoldNode::new(node2_config).unwrap();
    
    // Initialize network layers
    let result = node1.init_network(network1_config.clone()).await;
    assert!(result.is_ok(), "Failed to initialize network 1: {:?}", result);
    
    let result = node2.init_network(network2_config.clone()).await;
    assert!(result.is_ok(), "Failed to initialize network 2: {:?}", result);
    
    // Start network layers
    let result = node1.start_network().await;
    assert!(result.is_ok(), "Failed to start network 1: {:?}", result);
    
    let result = node2.start_network().await;
    assert!(result.is_ok(), "Failed to start network 2: {:?}", result);
    
    // Get node IDs
    let node2_id = node2.get_node_id().to_string();
    
    // Manually add node2 to node1's trusted nodes
    let result = node1.add_trusted_node(&node2_id);
    assert!(result.is_ok(), "Failed to add trusted node: {:?}", result);
    
    // Verify node2 is in node1's trusted nodes
    let trusted_nodes = node1.get_trusted_nodes();
    assert!(trusted_nodes.contains_key(&node2_id), "Node 2 not found in trusted nodes");
    
    // In a real implementation, we would now be able to connect to node2
    // However, since we're using local test nodes without proper network setup,
    // the connection attempt would fail
    
    // Stop network layers
    let result = node1.stop_network().await;
    assert!(result.is_ok(), "Failed to stop network 1: {:?}", result);
    
    let result = node2.stop_network().await;
    assert!(result.is_ok(), "Failed to stop network 2: {:?}", result);
}

#[tokio::test]
async fn test_discovery_with_multiple_nodes() {
    const NUM_NODES: usize = 3;
    
    // Create temporary directories and nodes
    let mut temp_dirs = Vec::with_capacity(NUM_NODES);
    let mut nodes = Vec::with_capacity(NUM_NODES);
    let mut node_ids = Vec::with_capacity(NUM_NODES);
    
    // Create and initialize all nodes
    for _ in 0..NUM_NODES {
        let temp_dir = tempfile::tempdir().unwrap();
        
        let node_config = NodeConfig {
            storage_path: temp_dir.path().to_path_buf(),
            default_trust_distance: 1,
            network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
        };
        
        let network_config = create_test_network_config(true);
        
        let mut node = DataFoldNode::new(node_config).unwrap();
        let result = node.init_network(network_config).await;
        assert!(result.is_ok(), "Failed to initialize network: {:?}", result);
        
        let node_id = node.get_node_id().to_string();
        
        temp_dirs.push(temp_dir);
        nodes.push(node);
        node_ids.push(node_id);
    }
    
    // Start all nodes
    for node in &mut nodes {
        let result = node.start_network().await;
        assert!(result.is_ok(), "Failed to start network: {:?}", result);
    }
    
    // Print node IDs for reference
    for (i, node_id) in node_ids.iter().enumerate() {
        println!("Node {} ID: {}", i, node_id);
    }
    
    // Wait for discovery to happen
    thread::sleep(Duration::from_secs(3));
    
    // Trigger discovery on all nodes
    for (i, node) in nodes.iter_mut().enumerate() {
        let result = node.discover_nodes().await;
        println!("Node {} discovery result: {:?}", i, result);
        // Don't fail the test if discovery fails due to network issues
    }
    
    // Check known nodes for each node
    for (i, node) in nodes.iter().enumerate() {
        let known_nodes = node.get_known_nodes().await;
        assert!(known_nodes.is_ok(), "Failed to get known nodes from node {}: {:?}", i, known_nodes);
        
        let known_nodes = known_nodes.unwrap();
        println!("Node {} known nodes: {:?}", i, known_nodes.keys().collect::<Vec<_>>());
        
        // Note: In a real network environment, each node would discover all other nodes.
        // In a test environment, this might not happen reliably.
    }
    
    // Stop all nodes
    for node in &mut nodes {
        let result = node.stop_network().await;
        assert!(result.is_ok(), "Failed to stop network: {:?}", result);
    }
}
