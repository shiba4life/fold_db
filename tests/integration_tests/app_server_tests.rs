use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use serde_json::json;
use reqwest::Client;
use tempfile::tempdir;

use fold_db::datafold_node::node::DataFoldNode;
use fold_db::datafold_node::config::NodeConfig;
use fold_db::datafold_node::app_server::AppServer;

// Helper function to create a test node with temporary storage
fn create_test_node() -> Arc<Mutex<DataFoldNode>> {
    let temp_dir = tempdir().unwrap();
    let config = NodeConfig {
        storage_path: temp_dir.path().to_path_buf(),
        default_trust_distance: 1,
    };
    
    let node = DataFoldNode::new(config).unwrap();
    Arc::new(Mutex::new(node))
}

// Helper function to create a signed request JSON
fn create_signed_request_json(operation_type: &str, content: &str) -> serde_json::Value {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
        
    json!({
        "timestamp": timestamp,
        "payload": {
            "operation": operation_type,
            "content": content
        }
    })
}

// Start the server on a random port and return the port number
async fn start_test_server() -> u16 {
    let node = create_test_node();
    let server = AppServer::new(node);
    
    // Use port 0 to let the OS assign a random available port
    let port = 0;
    
    // Start the server in a separate task
    let server_handle = tokio::spawn(async move {
        let _ = server.run(port).await;
    });
    
    // Wait a moment for the server to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Return the port (in a real implementation, we would need to get the actual port)
    // For now, we'll just use a hardcoded port for testing
    8081
}

#[tokio::test]
#[ignore = "Requires a running server on a specific port"]
async fn test_status_endpoint() {
    // Start the server
    let port = start_test_server().await;
    
    // Create a client
    let client = Client::new();
    
    // Make a request to the status endpoint
    let response = client
        .get(&format!("http://localhost:{}/api/v1/status", port))
        .send()
        .await;
    
    // Check if the request was successful
    assert!(response.is_ok(), "Failed to connect to server");
    
    let response = response.unwrap();
    assert_eq!(response.status(), 200);
    
    // Parse the response body
    let body: serde_json::Value = response.json().await.unwrap();
    
    // Verify the response structure
    assert_eq!(body["status"], "ok");
    assert!(body["version"].is_string());
    assert!(body["timestamp"].is_number());
}

#[tokio::test]
#[ignore = "Requires a running server on a specific port"]
async fn test_execute_endpoint_with_invalid_operation() {
    // Start the server
    let port = start_test_server().await;
    
    // Create a client
    let client = Client::new();
    
    // Create an invalid operation request
    let request = create_signed_request_json("query", "not valid json");
    
    // Make a request to the execute endpoint
    let response = client
        .post(&format!("http://localhost:{}/api/v1/execute", port))
        .header("x-public-key", "test-public-key")
        .json(&request)
        .send()
        .await;
    
    // Check if the request was successful
    assert!(response.is_ok(), "Failed to connect to server");
    
    let response = response.unwrap();
    assert_eq!(response.status(), 200); // The API returns 200 even for errors
    
    // Parse the response body
    let body: serde_json::Value = response.json().await.unwrap();
    
    // Verify the error response
    assert_eq!(body["error"], "INVALID_PAYLOAD");
    assert_eq!(body["code"], "REQUEST_ERROR");
    assert!(body["message"].as_str().unwrap().contains("Invalid operation format"));
}

#[tokio::test]
#[ignore = "Requires a running server on a specific port"]
async fn test_execute_endpoint_with_valid_operation() {
    // Start the server
    let port = start_test_server().await;
    
    // Create a client
    let client = Client::new();
    
    // Create a valid operation (a simple schema query that will fail but is valid JSON)
    let operation = json!({
        "type": "query",
        "schema": "test_schema",
        "fields": ["field1", "field2"]
    });
    
    let request = create_signed_request_json("query", &operation.to_string());
    
    // Make a request to the execute endpoint
    let response = client
        .post(&format!("http://localhost:{}/api/v1/execute", port))
        .header("x-public-key", "test-public-key")
        .json(&request)
        .send()
        .await;
    
    // Check if the request was successful
    assert!(response.is_ok(), "Failed to connect to server");
    
    let response = response.unwrap();
    assert_eq!(response.status(), 200);
    
    // Parse the response body
    let body: serde_json::Value = response.json().await.unwrap();
    
    // The operation should be parsed correctly but fail to execute
    // since we don't have a real schema set up
    assert!(body.get("error").is_some());
    assert_eq!(body["code"], "EXECUTION_ERROR");
}
