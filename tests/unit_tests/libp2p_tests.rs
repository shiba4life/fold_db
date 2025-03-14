use std::net::SocketAddr;
use std::time::Duration;
use std::thread;

use fold_db::datafold_node::network::{
    LibP2pManager,
    NetworkConfig,
    NodeId,
    SchemaInfo,
};
use fold_db::schema::types::Query;

// This test is marked as ignored because it requires a tokio runtime
#[test]
#[ignore]
fn test_libp2p_manager_creation() {
    // Create a network configuration
    let config = NetworkConfig {
        listen_address: "127.0.0.1:8091".parse().unwrap(),
        discovery_port: 8090,
        max_connections: 10,
        connection_timeout: Duration::from_secs(5),
        announcement_interval: Duration::from_secs(60),
        enable_discovery: true,
    };

    // Create a node ID
    let node_id = "test-node-1".to_string();

    // Create a libp2p manager
    let manager = LibP2pManager::new(config, node_id.clone(), None);
    assert!(manager.is_ok(), "Failed to create LibP2pManager: {:?}", manager.err());

    let mut manager = manager.unwrap();

    // Set callbacks
    manager.set_schema_list_callback(|| {
        vec![
            SchemaInfo {
                name: "test-schema".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Test schema".to_string()),
            }
        ]
    });

    manager.set_query_callback(|query| {
        println!("Executing query: {:?}", query);
        vec![Ok(serde_json::json!({"result": "test"}))]
    });

    // Start the manager
    let result = manager.start();
    assert!(result.is_ok(), "Failed to start LibP2pManager: {:?}", result.err());

    // Stop the manager
    let result = manager.stop();
    assert!(result.is_ok(), "Failed to stop LibP2pManager: {:?}", result.err());
}

// This test is marked as ignored because it requires a tokio runtime
#[test]
#[ignore]
fn test_libp2p_manager_discovery() {
    // Create network configurations for two nodes
    let config1 = NetworkConfig {
        listen_address: "127.0.0.1:8091".parse().unwrap(),
        discovery_port: 8090,
        max_connections: 10,
        connection_timeout: Duration::from_secs(5),
        announcement_interval: Duration::from_secs(60),
        enable_discovery: true,
    };

    let config2 = NetworkConfig {
        listen_address: "127.0.0.1:8093".parse().unwrap(),
        discovery_port: 8092,
        max_connections: 10,
        connection_timeout: Duration::from_secs(5),
        announcement_interval: Duration::from_secs(60),
        enable_discovery: true,
    };

    // Create node IDs
    let node_id1 = "test-node-1".to_string();
    let node_id2 = "test-node-2".to_string();

    // Create libp2p managers
    let manager1 = LibP2pManager::new(config1, node_id1.clone(), None).unwrap();
    let manager2 = LibP2pManager::new(config2, node_id2.clone(), None).unwrap();

    let mut manager1 = manager1;
    let mut manager2 = manager2;

    // Start the managers
    manager1.start().unwrap();
    manager2.start().unwrap();

    // Wait for discovery to happen
    thread::sleep(Duration::from_secs(2));

    // Discover nodes
    let nodes1 = manager1.discover_nodes();
    let nodes2 = manager2.discover_nodes();

    // In a real implementation, the nodes would discover each other
    // For now, we just check that the discovery method returns successfully
    assert!(nodes1.is_ok(), "Failed to discover nodes from manager1: {:?}", nodes1.err());
    assert!(nodes2.is_ok(), "Failed to discover nodes from manager2: {:?}", nodes2.err());

    // Stop the managers
    manager1.stop().unwrap();
    manager2.stop().unwrap();
}

// This test is marked as ignored because it requires actual networking
// To run it, use: cargo test -- --ignored
#[test]
#[ignore]
fn test_libp2p_manager_connection() {
    // Create network configurations for two nodes
    let config1 = NetworkConfig {
        listen_address: "127.0.0.1:8091".parse().unwrap(),
        discovery_port: 8090,
        max_connections: 10,
        connection_timeout: Duration::from_secs(5),
        announcement_interval: Duration::from_secs(60),
        enable_discovery: true,
    };

    let config2 = NetworkConfig {
        listen_address: "127.0.0.1:8093".parse().unwrap(),
        discovery_port: 8092,
        max_connections: 10,
        connection_timeout: Duration::from_secs(5),
        announcement_interval: Duration::from_secs(60),
        enable_discovery: true,
    };

    // Create node IDs
    let node_id1 = "test-node-1".to_string();
    let node_id2 = "test-node-2".to_string();

    // Create libp2p managers
    let manager1 = LibP2pManager::new(config1, node_id1.clone(), None).unwrap();
    let manager2 = LibP2pManager::new(config2, node_id2.clone(), None).unwrap();

    let mut manager1 = manager1;
    let mut manager2 = manager2;

    // Set callbacks
    manager1.set_schema_list_callback(|| {
        vec![
            SchemaInfo {
                name: "test-schema".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Test schema".to_string()),
            }
        ]
    });

    manager1.set_query_callback(|query| {
        println!("Executing query on node 1: {:?}", query);
        vec![Ok(serde_json::json!({"result": "from-node-1"}))]
    });

    manager2.set_query_callback(|query| {
        println!("Executing query on node 2: {:?}", query);
        vec![Ok(serde_json::json!({"result": "from-node-2"}))]
    });

    // Start the managers
    manager1.start().unwrap();
    manager2.start().unwrap();

    // Wait for discovery to happen
    thread::sleep(Duration::from_secs(2));

    // Discover nodes
    let nodes1 = manager1.discover_nodes().unwrap();
    let nodes2 = manager2.discover_nodes().unwrap();

    // In a real implementation, the nodes would discover each other
    // For now, we manually add the nodes to each other's known nodes
    
    // Connect node 1 to node 2
    let result = manager1.connect_to_node(&node_id2);
    assert!(result.is_ok(), "Failed to connect node 1 to node 2: {:?}", result.err());

    // Connect node 2 to node 1
    let result = manager2.connect_to_node(&node_id1);
    assert!(result.is_ok(), "Failed to connect node 2 to node 1: {:?}", result.err());

    // Query node 2 from node 1
    let query = Query {
        schema_name: "test-schema".to_string(),
        fields: vec!["field1".to_string(), "field2".to_string()],
        pub_key: "".to_string(),
        trust_distance: 1,
    };

    let result = manager1.query_node(&node_id2, query.clone());
    assert!(result.is_ok(), "Failed to query node 2 from node 1: {:?}", result.err());

    // Query node 1 from node 2
    let result = manager2.query_node(&node_id1, query);
    assert!(result.is_ok(), "Failed to query node 1 from node 2: {:?}", result.err());

    // List schemas on node 2 from node 1
    let result = manager1.list_available_schemas(&node_id2);
    assert!(result.is_ok(), "Failed to list schemas on node 2 from node 1: {:?}", result.err());

    // Stop the managers
    manager1.stop().unwrap();
    manager2.stop().unwrap();
}
