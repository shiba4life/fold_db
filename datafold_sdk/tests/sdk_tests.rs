use datafold_sdk::{
    DataFoldClient, QueryFilter, AppPermissions, FieldPermissions,
    ContainerConfig, SocialAppContainer, MicroVMConfig, MicroVMType
};
use serde_json::json;
use std::path::PathBuf;

#[tokio::test]
async fn test_client_creation() {
    // Test creating a client with default connection
    let client = DataFoldClient::new("test-app", "test-private-key", "test-public-key");
    assert_eq!(client.get_app_id(), "test-app");
    assert_eq!(client.get_public_key(), "test-public-key");
}

#[tokio::test]
async fn test_schema_discovery() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let client = DataFoldClient::new("test-app", "test-private-key", "test-public-key");
    
    // Test discovering local schemas
    let schemas = client.discover_local_schemas().await?;
    assert!(!schemas.is_empty(), "Local schemas should not be empty");
    assert!(schemas.contains(&"user".to_string()), "User schema should be available");
    
    // Test getting schema details
    let schema_details = client.get_schema_details("user", None).await?;
    assert!(schema_details.is_object(), "Schema details should be an object");
    assert_eq!(schema_details["name"], json!("user"), "Schema name should be 'user'");
    
    // Test clearing cache
    client.clear_cache().await?;
    
    Ok(())
}

#[tokio::test]
async fn test_query_operations() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let client = DataFoldClient::new("test-app", "test-private-key", "test-public-key");
    
    // Test basic query
    let query_result = client.query("user")
        .select(&["id", "name", "email"])
        .execute()
        .await?;
    
    assert!(!query_result.results.is_empty(), "Query results should not be empty");
    assert!(query_result.errors.is_empty(), "Query should not have errors");
    
    // Test query with filter
    let query_result = client.query("user")
        .select(&["id", "name", "email"])
        .filter(QueryFilter::eq("name", json!("Test User")))
        .execute()
        .await?;
    
    assert!(!query_result.results.is_empty(), "Filtered query results should not be empty");
    
    // Test query with multiple filters
    let query_result = client.query("user")
        .select(&["id", "name", "email"])
        .filter(QueryFilter::eq("name", json!("Test User")))
        .filter(QueryFilter::contains("email", json!("example.com")))
        .execute()
        .await?;
    
    assert!(!query_result.results.is_empty(), "Multi-filtered query results should not be empty");
    
    Ok(())
}

#[tokio::test]
async fn test_mutation_operations() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let client = DataFoldClient::new("test-app", "test-private-key", "test-public-key");
    
    // Test create mutation
    let mutation_result = client.mutate("user")
        .set("name", json!("New User"))
        .set("email", json!("new@example.com"))
        .execute()
        .await?;
    
    assert!(mutation_result.success, "Create mutation should succeed");
    assert!(mutation_result.id.is_some(), "Create mutation should return an ID");
    
    // Test update mutation
    let mutation_result = client.mutate("user")
        .operation(datafold_sdk::mutation_builder::MutationType::Update)
        .set("id", json!("123"))
        .set("name", json!("Updated User"))
        .execute()
        .await?;
    
    assert!(mutation_result.success, "Update mutation should succeed");
    
    // Test delete mutation
    let mutation_result = client.mutate("user")
        .operation(datafold_sdk::mutation_builder::MutationType::Delete)
        .set("id", json!("123"))
        .execute()
        .await?;
    
    assert!(mutation_result.success, "Delete mutation should succeed");
    
    Ok(())
}

#[tokio::test]
async fn test_network_operations() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let client = DataFoldClient::new("test-app", "test-private-key", "test-public-key");
    
    // Test discovering nodes
    let nodes = client.discover_nodes().await?;
    assert!(!nodes.is_empty(), "Should discover at least one node");
    
    if !nodes.is_empty() {
        let node_id = &nodes[0].id;
        
        // Test checking node availability
        let available = client.is_node_available(node_id).await?;
        assert!(available, "Node should be available");
        
        // Test getting node info
        let node_info = client.get_node_info(node_id).await?;
        assert_eq!(node_info.id, *node_id, "Node ID should match");
        
        // Test discovering remote schemas
        let remote_schemas = client.discover_remote_schemas(node_id).await?;
        assert!(!remote_schemas.is_empty(), "Remote node should have schemas");
        
        if !remote_schemas.is_empty() {
            let schema_name = &remote_schemas[0];
            
            // Test querying remote data
            let remote_query_result = client.query_on_node(schema_name, node_id)
                .select(&["id", "name"])
                .execute()
                .await?;
            
            assert!(!remote_query_result.results.is_empty(), "Remote query should return results");
        }
    }
    
    // Test getting all nodes
    let all_nodes = client.get_all_nodes().await?;
    assert!(!all_nodes.is_empty(), "Should get at least one node");
    
    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let client = DataFoldClient::new("test-app", "test-private-key", "test-public-key");
    
    // Test query with no fields (should error)
    let query_result = client.query("user")
        .execute()
        .await;
    
    assert!(query_result.is_err(), "Query with no fields should fail");
    
    // Test mutation with no data (should error)
    let mutation_result = client.mutate("user")
        .execute()
        .await;
    
    assert!(mutation_result.is_err(), "Mutation with no data should fail");
    
    // Test getting schema details for non-existent schema
    let schema_result = client.get_schema_details("non_existent_schema", None).await;
    assert!(schema_result.is_err(), "Getting non-existent schema should fail");
    
    Ok(())
}

#[tokio::test]
async fn test_container_management() -> Result<(), Box<dyn std::error::Error>> {
    // Create permissions
    let permissions = AppPermissions::new()
        .allow_schemas(&["user", "post"])
        .with_field_permissions("user", FieldPermissions::new()
            .allow_reads(&["id", "name", "email"])
            .allow_writes(&["name", "email"]))
        .with_max_trust_distance(2);
    
    // Create VM configuration
    let vm_config = MicroVMConfig::new(
        MicroVMType::Firecracker,
        "/var/lib/datafold/vm-images/minimal-rootfs.ext4"
    )
    .with_vcpu_count(1)
    .with_memory_mb(128);
    
    // Create container configuration
    let config = ContainerConfig::new_microvm(
        PathBuf::from("/path/to/app"),
        vm_config
    );
    
    // Create container
    let container = SocialAppContainer::new(
        "test-app",
        "test-public-key",
        permissions,
        config
    );
    
    // Test container status before starting
    assert!(!container.is_running(), "Container should not be running initially");
    
    // Note: We're not actually starting the container in tests
    // as it would require the actual VM infrastructure
    
    Ok(())
}

#[tokio::test]
async fn test_raw_request() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let client = DataFoldClient::new("test-app", "test-private-key", "test-public-key");
    
    // Create a raw request
    let request = datafold_sdk::types::AppRequest::new(
        "test-app",
        None,
        "custom_operation",
        json!({
            "param1": "value1",
            "param2": "value2"
        }),
        "test-private-key"
    );
    
    // Send the raw request
    let response = client.send_request(request).await?;
    
    assert!(response.is_object(), "Response should be an object");
    assert_eq!(response["success"], json!(true), "Response should indicate success");
    
    Ok(())
}
