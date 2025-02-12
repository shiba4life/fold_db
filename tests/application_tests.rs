use fold_db::{ClientConfig, DataFoldClient, DataFoldNode, NodeConfig, SocketConfig, SocketServer};
use fold_db::schema::{Schema, SchemaField};
use fold_db::permissions::types::policy::PermissionsPolicy;
use fold_db::fees::types::config::FieldPaymentConfig;
use serde_json::json;
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn test_socket_communication() -> std::io::Result<()> {
    // Create temporary directory for socket and database
    let temp_dir = tempdir()?;
    let socket_path = temp_dir.path().join("test.sock");
    let db_path = temp_dir.path().join("db");

    // Initialize node
    let node_config = NodeConfig {
        storage_path: db_path,
        default_trust_distance: 1,
    };
    let mut node = DataFoldNode::new(node_config).expect("Failed to create node");

    // Create and load test schema
    let mut test_schema = Schema::new("test_schema".to_string());
    
    // Add fields with default policies and payment configs
    test_schema.add_field(
        "name".to_string(),
        SchemaField::new(
            PermissionsPolicy::default(),
            "name_ref".to_string(),
            FieldPaymentConfig::default(),
        )
    );
    test_schema.add_field(
        "email".to_string(),
        SchemaField::new(
            PermissionsPolicy::default(),
            "email_ref".to_string(),
            FieldPaymentConfig::default(),
        )
    );

    node.load_schema(test_schema).expect("Failed to load schema");

    // Initialize server
    let socket_config = SocketConfig {
        socket_path: socket_path.clone(),
        permissions: 0o660,
        buffer_size: 8192,
    };
    let server = SocketServer::new(socket_config, node).expect("Failed to create server");
    let server_handle = server.start()?;
    
    // Give the server a moment to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Initialize client
    let client_config = ClientConfig {
        socket_path: socket_path.clone(),
        timeout: Duration::from_secs(1),
    };
    let client = DataFoldClient::new(client_config);

    // Test mutation operation
    let mutation_result: serde_json::Value = client
        .mutate(json!({
            "schema": "test_schema",
            "data": {
                "name": "Test User",
                "email": "test@example.com"
            },
            "pub_key": "test_key"
        }))
        .expect("Mutation failed");
    assert!(mutation_result["success"].as_bool().unwrap_or(false));

    // Test query operation to verify mutation
    let query_result: serde_json::Value = client
        .query(json!({
            "schema": "test_schema",
            "fields": ["name", "email"],
            "pub_key": "test_key"
        }))
        .expect("Query failed");
    
    let results = query_result["results"].as_array().expect("Expected results array");
    assert!(!results.is_empty());
    let first_result = &results[0];
    assert_eq!(first_result["name"], "Test User");
    assert_eq!(first_result["email"], "test@example.com");

    // Cleanup server
    server.shutdown()?;
    server_handle.join().unwrap();

    // Test schema retrieval
    let schema_result: serde_json::Value = client
        .get_schema("test_schema")
        .expect("Schema retrieval failed");
    assert!(schema_result.get("schema").is_some());

    Ok(())
}

#[tokio::test]
async fn test_socket_error_handling() -> std::io::Result<()> {
    // Create temporary directory for socket
    let temp_dir = tempdir()?;
    let socket_path = temp_dir.path().join("test.sock");

    // Try to connect to non-existent server
    let client_config = ClientConfig {
        socket_path,
        timeout: Duration::from_secs(1),
    };
    let client = DataFoldClient::new(client_config);

    // Attempt query should fail
    let query_result: Result<serde_json::Value, _> = client.query(json!({
        "schema": "test_schema",
        "fields": ["name"],
        "pub_key": "test_key"
    }));
    assert!(query_result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_socket_permissions() -> std::io::Result<()> {
    // Create temporary directory for socket and database
    let temp_dir = tempdir()?;
    let socket_path = temp_dir.path().join("test.sock");
    let db_path = temp_dir.path().join("db");

    // Initialize node
    let node_config = NodeConfig {
        storage_path: db_path,
        default_trust_distance: 1,
    };
    let node = DataFoldNode::new(node_config).expect("Failed to create node");

    // Initialize server with restrictive permissions
    let socket_config = SocketConfig {
        socket_path: socket_path.clone(),
        permissions: 0o600, // Only owner can read/write
        buffer_size: 8192,
    };
    let server = SocketServer::new(socket_config, node).expect("Failed to create server");
    let server_handle = server.start()?;

    // Give the server a moment to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify socket permissions
    use std::os::unix::fs::PermissionsExt;
    let metadata = std::fs::metadata(&socket_path)?;
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);

    // Cleanup server
    server.shutdown()?;
    server_handle.join().unwrap();

    Ok(())
}
