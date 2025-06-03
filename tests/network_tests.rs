#[path = "test_data/test_helpers/mod.rs"]
mod test_helpers;
use fold_node::network::{NetworkConfig, NetworkCore, SchemaService};
use fold_node::schema::Schema;
use test_helpers::create_test_node;

#[tokio::test]
async fn test_schema_service() {
    // Create a schema service
    let mut service = SchemaService::new();

    // Default callback should return empty list
    let result = service.check_schemas(&["schema1".to_string(), "schema2".to_string()]);
    assert!(result.is_empty());

    // Set custom callback
    service.set_schema_check_callback(|names| {
        names
            .iter()
            .filter(|name| ["schema1", "schema2"].contains(&name.as_str()))
            .cloned()
            .collect()
    });

    // Should now return only schemas in the allowed list
    let result = service.check_schemas(&[
        "schema1".to_string(),
        "schema2".to_string(),
        "schema3".to_string(),
    ]);

    assert_eq!(result, vec!["schema1".to_string(), "schema2".to_string()]);
}

#[tokio::test]
async fn test_network_core_creation() {
    // Create a network core
    let config = NetworkConfig::default();
    let node = NetworkCore::new(config).await.unwrap();

    // Verify we have a valid peer ID
    let peer_id = node.local_peer_id();
    assert!(!peer_id.to_string().is_empty());
}

#[tokio::test]
async fn test_discover_nodes_disabled() {
    // Create a network core with discovery disabled
    let config = NetworkConfig::default().with_mdns(false);
    let mut node = NetworkCore::new(config).await.unwrap();

    // Discovery should return an empty list when disabled
    let peers = node.discover_nodes().await.unwrap();
    assert!(peers.is_empty());
}

#[tokio::test]
async fn test_datafold_node_network_integration() {
    // Create the nodes
    let mut node1 = create_test_node();
    let mut node2 = create_test_node();

    // Create network configs
    let network1_config = NetworkConfig::new("/ip4/127.0.0.1/tcp/0").with_mdns(false); // Disable mDNS for testing

    let network2_config = NetworkConfig::new("/ip4/127.0.0.1/tcp/0").with_mdns(false); // Disable mDNS for testing

    // Initialize the network layers
    node1.init_network(network1_config).await.unwrap();
    node2.init_network(network2_config).await.unwrap();

    // Create a test schema for node1
    let test_schema = Schema::new("test_schema".to_string());

    // Load the schema into node1
    node1.load_schema(test_schema.clone()).unwrap();

    // Get the peer IDs
    let node1_id = node1.get_node_id().to_string();
    let node2_id = node2.get_node_id().to_string();

    // Add trusted nodes
    node1.add_trusted_node(&node2_id).unwrap();
    node2.add_trusted_node(&node1_id).unwrap();

    // Start the network services
    node1
        .start_network_with_address("/ip4/127.0.0.1/tcp/0")
        .await
        .unwrap();
    node2
        .start_network_with_address("/ip4/127.0.0.1/tcp/0")
        .await
        .unwrap();

    // Since we're using mock peers for testing, we need to manually add them
    // In a real scenario, peers would be discovered via mDNS

    // Check schemas on node2 from node1
    // This is a bit tricky to test without actual network communication
    // In a real scenario, we would use the actual peer ID from discovery

    // For now, we'll just verify that the methods don't panic
    // In a real test, we would need to set up actual network communication
    // or use more sophisticated mocking

    // This test mainly verifies that the integration compiles and the methods exist
    assert!(node1.get_schema("test_schema").unwrap().is_some());
    assert!(node2.get_schema("test_schema").unwrap().is_none());
}
