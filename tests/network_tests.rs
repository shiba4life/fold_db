use fold_db::network::{NetworkCore, NetworkConfig, SchemaService};

#[tokio::test]
async fn test_schema_service() {
    // Create a schema service
    let mut service = SchemaService::new();
    
    // Default callback should return empty list
    let result = service.check_schemas(&["schema1".to_string(), "schema2".to_string()]);
    assert!(result.is_empty());
    
    // Set custom callback
    service.set_schema_check_callback(|names| {
        names.iter()
            .filter(|name| ["schema1", "schema2"].contains(&name.as_str()))
            .cloned()
            .collect()
    });
    
    // Should now return only schemas in the allowed list
    let result = service.check_schemas(&[
        "schema1".to_string(),
        "schema2".to_string(),
        "schema3".to_string()
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
