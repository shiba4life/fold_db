use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

use fold_db::{
    datafold_node::{DataFoldNode, TcpServer, config::NodeConfig},
    network::NetworkConfig,
};

use crate::Logger;

/// Configuration for a test node
pub struct TestNodeConfig {
    pub node_dir: PathBuf,
    pub network_port: u16,
    pub tcp_port: u16,
}

/// A test node with network and TCP server
pub struct TestNode {
    pub node: DataFoldNode,
    pub node_id: String,
    pub peer_id: String,
    _tcp_server_handle: tokio::task::JoinHandle<()>,
}

/// Set up two test nodes with network connectivity
pub async fn setup_two_nodes(logger: &mut Logger) -> Result<(TestNode, TestNode), Box<dyn std::error::Error>> {
    logger.log("\nSetting up two DataFold nodes...");
    
    // Create test node configurations
    let node1_config = TestNodeConfig {
        node_dir: PathBuf::from("test_data/sandboxed_two_node_example/node1/db"),
        network_port: 9001,
        tcp_port: 8001,
    };
    
    let node2_config = TestNodeConfig {
        node_dir: PathBuf::from("test_data/sandboxed_two_node_example/node2/db"),
        network_port: 9002,
        tcp_port: 8002,
    };
    
    // Create and start the nodes
    let mut node1 = create_test_node(node1_config, logger).await?;
    
    // Wait a moment before starting the second node
    sleep(Duration::from_secs(1)).await;
    
    let mut node2 = create_test_node(node2_config, logger).await?;
    
    // Wait a moment to ensure both nodes are fully started
    sleep(Duration::from_secs(2)).await;
    
    logger.log("Network services and TCP servers started");
    
    // Add trusted nodes
    node1.node.add_trusted_node(&node2.node_id)?;
    node2.node.add_trusted_node(&node1.node_id)?;
    
    // Manually add peers to simulate discovery
    {
        let mut network1 = node1.node.get_network_mut().await?;
        network1.add_known_peer(node2.peer_id.parse()?);
        // Register the node ID to peer ID mapping
        network1.register_node_id(&node2.node_id, node2.peer_id.parse()?);
    }
    
    {
        let mut network2 = node2.node.get_network_mut().await?;
        network2.add_known_peer(node1.peer_id.parse()?);
        // Register the node ID to peer ID mapping
        network2.register_node_id(&node1.node_id, node1.peer_id.parse()?);
    }
    
    // Wait for the nodes to discover each other
    sleep(Duration::from_secs(2)).await;
    
    logger.log("Peers manually added");
    
    Ok((node1, node2))
}

/// Create a single test node with network and TCP server
async fn create_test_node(config: TestNodeConfig, logger: &mut Logger) -> Result<TestNode, Box<dyn std::error::Error>> {
    // Create temporary directory for the node
    std::fs::create_dir_all(&config.node_dir)?;
    
    // Create node config
    let node_config = NodeConfig {
        storage_path: config.node_dir,
        default_trust_distance: 1,
        network_listen_address: format!("/ip4/127.0.0.1/tcp/{}", config.network_port),
    };
    
    // Create the node
    let mut node = DataFoldNode::new(node_config)?;
    
    logger.log("Node created successfully");
    
    // Create network config
    let network_config = NetworkConfig::new(&format!("/ip4/127.0.0.1/tcp/{}", config.network_port));
    
    // Initialize the network layer
    node.init_network(network_config).await?;
    
    // Start the network service
    node.start_network_with_address(&format!("/ip4/127.0.0.1/tcp/{}", config.network_port)).await?;
    logger.log(&format!("Node network service started on port {}", config.network_port));
    
    // Start the TCP server
    logger.log(&format!("Starting TCP server on port {}...", config.tcp_port));
    let tcp_server = TcpServer::new(node.clone(), config.tcp_port).await?;
    let tcp_server_handle = tokio::spawn(async move {
        if let Err(e) = tcp_server.run().await {
            eprintln!("TCP server error: {}", e);
        }
    });
    logger.log(&format!("TCP server started on port {}", config.tcp_port));
    
    // Get the node ID and peer ID
    let node_id = node.get_node_id().to_string();
    
    let peer_id;
    {
        let network = node.get_network_mut().await?;
        peer_id = network.local_peer_id().to_string();
    }
    
    logger.log(&format!("Node ID: {}", node_id));
    logger.log(&format!("Peer ID: {}", peer_id));
    
    Ok(TestNode {
        node,
        node_id,
        peer_id,
        _tcp_server_handle: tcp_server_handle,
    })
}
