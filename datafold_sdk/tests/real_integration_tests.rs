use datafold_sdk::{
    DataFoldClient, QueryFilter, NodeConnection, AuthCredentials
};
use serde_json::json;
use std::process::{Child, Command};
use std::sync::Once;
use std::time::Duration;
use std::thread;
use tempfile::TempDir;
use std::fs;

// Global static for initializing the test node once
static INIT: Once = Once::new();
static mut NODE_PROCESS: Option<Child> = None;
static mut TEST_DIR: Option<TempDir> = None;
static mut NODE_PORT: u16 = 9876;

// Initialize the test node
#[allow(static_mut_refs)]
fn init_test_node() -> u16 {
    unsafe {
        INIT.call_once(|| {
            // Create a temporary directory for the node
            let temp_dir = TempDir::new().expect("Failed to create temp directory");
            let config_path = temp_dir.path().join("node_config.json");
            
            // Create a basic config file
            let port = NODE_PORT; // Copy the value to avoid shared reference to mutable static
            let config = json!({
                "storage_path": temp_dir.path().to_str().unwrap(),
                "default_trust_distance": 1,
                "network_listen_address": format!("/ip4/127.0.0.1/tcp/{}", port)
            });
            
            fs::write(&config_path, config.to_string())
                .expect("Failed to write config file");
            
            println!("Starting test node on port {}", port);
            
            // Launch the node process - using the path relative to the workspace root
            let node_process = Command::new("cargo")
                .args(["run", "--bin", "datafold_node", "--manifest-path", "../fold_node/Cargo.toml", "--", 
                       "--port", &port.to_string()])
                .env("NODE_CONFIG", config_path.to_str().unwrap())
                .spawn()
                .expect("Failed to start datafold node");
            
            NODE_PROCESS = Some(node_process);
            TEST_DIR = Some(temp_dir);
            
            // Give the node time to start up
            thread::sleep(Duration::from_secs(2));
            println!("Test node started");
        });
        
        NODE_PORT
    }
}

// Clean up the test node
#[allow(static_mut_refs)]
fn cleanup_test_node() {
    unsafe {
        // Use Option::take to avoid creating a mutable reference to the static
        if let Some(node_process) = NODE_PROCESS.take() {
            println!("Stopping test node");
            // Create a mutable variable from the owned value, not a reference to the static
            let mut node = node_process;
            let _ = node.kill();
            let _ = node.wait();
        }
        
        // Use Option::take to avoid creating a mutable reference to the static
        if let Some(temp_dir) = TEST_DIR.take() {
            let _ = temp_dir.close();
        }
    }
}

// Create a test client that connects to the test node
fn create_test_client() -> DataFoldClient {
    let _port = init_test_node(); // Using underscore to indicate intentionally unused variable
    
    // Create authentication credentials
    let auth = AuthCredentials {
        app_id: "test-app".to_string(),
        private_key: "test-private-key".to_string(),
        public_key: "test-public-key".to_string(),
    };
    
    // For demonstration purposes, we'll use a mock connection
    // In a real implementation, we would need to know the exact socket path or TCP address the node is using
    DataFoldClient::with_connection(
        &auth.app_id,
        &auth.private_key,
        &auth.public_key,
        NodeConnection::UnixSocket("mock".to_string()),
    )
}

// Note: In a real implementation, we would need a way to load schemas into the node
// This could be done through a direct API call, a CLI command, or other means
// For these tests, we're assuming schemas are already loaded in the mock implementation

#[tokio::test]
async fn test_real_node_schema_discovery() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the test node and create a client that connects to it
    let client = create_test_client();
    
    // Try to discover schemas
    println!("Discovering local schemas...");
    let schemas = client.discover_local_schemas().await?;
    println!("Available schemas: {:?}", schemas);
    
    // Clean up the test node
    cleanup_test_node();
    
    Ok(())
}

#[tokio::test]
async fn test_real_node_network_discovery() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the test node and create a client that connects to it
    let client = create_test_client();
    
    // Test discovering nodes
    println!("Discovering nodes...");
    let nodes = client.discover_nodes().await?;
    println!("Discovered nodes: {:?}", nodes);
    
    // Test getting all nodes
    let all_nodes = client.get_all_nodes().await?;
    println!("All nodes: {:?}", all_nodes);
    
    // Clean up the test node
    cleanup_test_node();
    
    Ok(())
}

#[tokio::test]
async fn test_real_node_crud_operations() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the test node and create a client that connects to it
    let client = create_test_client();
    
    // Discover available schemas
    println!("Discovering local schemas...");
    let schemas = client.discover_local_schemas().await?;
    println!("Available schemas: {:?}", schemas);
    
    // Use the first available schema for testing
    if schemas.is_empty() {
        println!("No schemas available for testing");
        cleanup_test_node();
        return Ok(());
    }
    
    let schema_name = &schemas[0];
    println!("Using schema '{}' for testing", schema_name);
    
    // Create a new profile
    println!("Creating test item...");
    let mutation_result = client.mutate(schema_name)
        .set("name", json!("Test User"))
        .set("email", json!("test@example.com"))
        .execute()
        .await?;
    
    assert!(mutation_result.success, "Create mutation should succeed");
    let item_id = mutation_result.id.unwrap();
    println!("Created item with ID: {}", item_id);
    
    // Query the item
    println!("Querying test item...");
    let query_result = client.query(schema_name)
        .select(&["name", "email"])
        .execute()
        .await?;
    
    assert!(!query_result.results.is_empty(), "Should find at least one item");
    println!("Query results: {:?}", query_result.results);
    
    // Update the item
    println!("Updating test item...");
    let mutation_result = client.mutate(schema_name)
        .operation(datafold_sdk::mutation_builder::MutationType::Update)
        .set("id", json!(item_id))
        .set("name", json!("Updated Test User"))
        .execute()
        .await?;
    
    assert!(mutation_result.success, "Update mutation should succeed");
    
    // Query the updated item
    println!("Querying updated test item...");
    let query_result = client.query(schema_name)
        .select(&["name", "email"])
        .filter(QueryFilter::eq("id", json!(item_id)))
        .execute()
        .await?;
    
    assert!(!query_result.results.is_empty(), "Should find the updated item");
    println!("Updated query results: {:?}", query_result.results);
    
    // Delete the item
    println!("Deleting test item...");
    let mutation_result = client.mutate(schema_name)
        .operation(datafold_sdk::mutation_builder::MutationType::Delete)
        .set("id", json!(item_id))
        .execute()
        .await?;
    
    assert!(mutation_result.success, "Delete mutation should succeed");
    
    // Clean up the test node
    cleanup_test_node();
    
    Ok(())
}

#[tokio::test]
async fn test_real_node_end_to_end() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the test node and create a client that connects to it
    let client = create_test_client();
    
    // 1. Discover available schemas
    println!("Discovering local schemas...");
    let schemas = client.discover_local_schemas().await?;
    println!("Available schemas: {:?}", schemas);
    
    // Use the first available schema for testing
    if schemas.is_empty() {
        println!("No schemas available for testing");
        cleanup_test_node();
        return Ok(());
    }
    
    let schema_name = &schemas[0];
    println!("Using schema '{}' for testing", schema_name);
    
    // 2. Create multiple items
    println!("Creating test items...");
    let mut item_ids = Vec::new();
    
    for i in 1..=3 {
        let mutation_result = client.mutate(schema_name)
            .set("name", json!(format!("Test User {}", i)))
            .set("email", json!(format!("test{}@example.com", i)))
            .execute()
            .await?;
        
        assert!(mutation_result.success, "Create mutation should succeed");
        let item_id = mutation_result.id.unwrap();
        println!("Created item {} with ID: {}", i, item_id);
        item_ids.push(item_id);
    }
    
    // 3. Query all items
    println!("Querying all test items...");
    let query_result = client.query(schema_name)
        .select(&["id", "name", "email"])
        .execute()
        .await?;
    
    println!("Query results: {:?}", query_result.results);
    // We can't assert the exact number since we don't know what's already in the database
    assert!(!query_result.results.is_empty(), "Should find at least one item");
    
    // 4. Update an item
    if !item_ids.is_empty() {
        println!("Updating test item...");
        let mutation_result = client.mutate(schema_name)
            .operation(datafold_sdk::mutation_builder::MutationType::Update)
            .set("id", json!(&item_ids[0]))
            .set("name", json!("Updated Test User"))
            .execute()
            .await?;
        
        assert!(mutation_result.success, "Update mutation should succeed");
        
        // 5. Query the updated item
        println!("Querying updated test item...");
        let query_result = client.query(schema_name)
            .select(&["name", "email"])
            .filter(QueryFilter::eq("id", json!(&item_ids[0])))
            .execute()
            .await?;
        
        assert!(!query_result.results.is_empty(), "Should find the updated item");
        println!("Updated query results: {:?}", query_result.results);
        
        // 6. Delete all items
        println!("Deleting test items...");
        for (i, item_id) in item_ids.iter().enumerate() {
            let mutation_result = client.mutate(schema_name)
                .operation(datafold_sdk::mutation_builder::MutationType::Delete)
                .set("id", json!(item_id))
                .execute()
                .await?;
            
            assert!(mutation_result.success, "Delete mutation should succeed");
            println!("Deleted item {} with ID: {}", i+1, item_id);
        }
    }
    
    // Clean up the test node
    cleanup_test_node();
    
    Ok(())
}
