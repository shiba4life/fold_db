use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::net::{TcpListener, UnixListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::json;

use fold_client::auth::AuthManager;
use fold_client::node::{NodeClient, NodeConnection};
use fold_client::FoldClientError;

// Helper function to create a temporary directory for testing
fn create_temp_auth_dir() -> (PathBuf, tempfile::TempDir) {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let auth_dir = temp_dir.path().join("auth");
    std::fs::create_dir_all(&auth_dir).expect("Failed to create auth directory");
    (auth_dir, temp_dir)
}

// Helper function to create a mock Unix socket server
async fn create_mock_unix_server(socket_path: PathBuf) -> tokio::task::JoinHandle<()> {
    // Remove the socket file if it exists
    let _ = std::fs::remove_file(&socket_path);
    
    // Create the listener
    let listener = UnixListener::bind(&socket_path).expect("Failed to bind Unix socket");
    
    // Start the server task
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            // Read the request length
            let request_len = stream.read_u32().await.expect("Failed to read request length") as usize;
            
            // Read the request
            let mut request_bytes = vec![0u8; request_len];
            stream.read_exact(&mut request_bytes).await.expect("Failed to read request");
            
            // Parse the request
            let request: serde_json::Value = serde_json::from_slice(&request_bytes).expect("Failed to parse request");
            
            // Create a mock response
            let response = json!({
                "success": true,
                "result": {
                    "message": "Mock response",
                    "request": request,
                }
            });
            
            // Serialize the response
            let response_bytes = serde_json::to_vec(&response).expect("Failed to serialize response");
            
            // Send the response length
            stream.write_u32(response_bytes.len() as u32).await.expect("Failed to send response length");
            
            // Send the response
            stream.write_all(&response_bytes).await.expect("Failed to send response");
        }
    })
}

// Helper function to create a mock TCP server
async fn create_mock_tcp_server(port: u16) -> tokio::task::JoinHandle<()> {
    // Create the listener
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind TCP socket");
    
    // Start the server task
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            // Read the request length
            let request_len = stream.read_u32().await.expect("Failed to read request length") as usize;
            
            // Read the request
            let mut request_bytes = vec![0u8; request_len];
            stream.read_exact(&mut request_bytes).await.expect("Failed to read request");
            
            // Parse the request
            let request: serde_json::Value = serde_json::from_slice(&request_bytes).expect("Failed to parse request");
            
            // Create a mock response
            let response = json!({
                "success": true,
                "result": {
                    "message": "Mock TCP response",
                    "request": request,
                }
            });
            
            // Serialize the response
            let response_bytes = serde_json::to_vec(&response).expect("Failed to serialize response");
            
            // Send the response length
            stream.write_u32(response_bytes.len() as u32).await.expect("Failed to send response length");
            
            // Send the response
            stream.write_all(&response_bytes).await.expect("Failed to send response");
        }
    })
}

#[tokio::test]
async fn test_node_client_unix_socket() {
    let (auth_dir, _temp_dir) = create_temp_auth_dir();
    
    // Create a temporary socket path
    let socket_dir = tempdir().expect("Failed to create temp directory for socket");
    let socket_path = socket_dir.path().join("test.sock");
    
    // Create a mock Unix socket server
    let server_handle = create_mock_unix_server(socket_path.clone()).await;
    
    // Create an AuthManager
    let auth_manager = Arc::new(AuthManager::new(auth_dir.clone()).expect("Failed to create AuthManager"));
    
    // Register a test app
    let app = auth_manager.register_app("test_app", &["query"]).expect("Failed to register app");
    
    // Create a NodeClient with Unix socket connection
    let node_client = NodeClient::new(
        NodeConnection::UnixSocket(socket_path.clone()),
        auth_manager.clone(),
    );
    
    // Send a request
    let response = node_client.send_request(
        &app.app_id,
        "test_operation",
        json!({"param1": "value1", "param2": 42}),
    ).await;
    
    // Abort the server task
    server_handle.abort();
    
    // Verify the response
    assert!(response.is_ok(), "Failed to send request");
    let response_value = response.unwrap();
    assert!(response_value.get("success").is_some(), "Response should have a success field");
    assert!(response_value.get("result").is_some(), "Response should have a result field");
    
    let result = response_value.get("result").unwrap();
    assert_eq!(result.get("message").unwrap().as_str().unwrap(), "Mock response", "Response message does not match");
    
    // Verify that the request was correctly sent
    let request = result.get("request").unwrap();
    assert_eq!(request.get("app_id").unwrap().as_str().unwrap(), app.app_id, "App ID in request does not match");
    assert_eq!(request.get("operation").unwrap().as_str().unwrap(), "test_operation", "Operation in request does not match");
    
    let params = request.get("params").unwrap();
    assert_eq!(params.get("param1").unwrap().as_str().unwrap(), "value1", "Param1 in request does not match");
    assert_eq!(params.get("param2").unwrap().as_i64().unwrap(), 42, "Param2 in request does not match");
}

#[tokio::test]
async fn test_node_client_tcp_socket() {
    let (auth_dir, _temp_dir) = create_temp_auth_dir();
    
    // Choose a port for the mock TCP server
    let port = 9876;
    
    // Create a mock TCP server
    let server_handle = create_mock_tcp_server(port).await;
    
    // Create an AuthManager
    let auth_manager = Arc::new(AuthManager::new(auth_dir.clone()).expect("Failed to create AuthManager"));
    
    // Register a test app
    let app = auth_manager.register_app("test_app", &["query"]).expect("Failed to register app");
    
    // Create a NodeClient with TCP socket connection
    let node_client = NodeClient::new(
        NodeConnection::TcpSocket("127.0.0.1".to_string(), port),
        auth_manager.clone(),
    );
    
    // Send a request
    let response = node_client.send_request(
        &app.app_id,
        "test_operation",
        json!({"param1": "value1", "param2": 42}),
    ).await;
    
    // Abort the server task
    server_handle.abort();
    
    // Verify the response
    assert!(response.is_ok(), "Failed to send request");
    let response_value = response.unwrap();
    assert!(response_value.get("success").is_some(), "Response should have a success field");
    assert!(response_value.get("result").is_some(), "Response should have a result field");
    
    let result = response_value.get("result").unwrap();
    assert_eq!(result.get("message").unwrap().as_str().unwrap(), "Mock TCP response", "Response message does not match");
    
    // Verify that the request was correctly sent
    let request = result.get("request").unwrap();
    assert_eq!(request.get("app_id").unwrap().as_str().unwrap(), app.app_id, "App ID in request does not match");
    assert_eq!(request.get("operation").unwrap().as_str().unwrap(), "test_operation", "Operation in request does not match");
    
    let params = request.get("params").unwrap();
    assert_eq!(params.get("param1").unwrap().as_str().unwrap(), "value1", "Param1 in request does not match");
    assert_eq!(params.get("param2").unwrap().as_i64().unwrap(), 42, "Param2 in request does not match");
}

#[tokio::test]
async fn test_node_client_connection_error() {
    let (auth_dir, _temp_dir) = create_temp_auth_dir();
    
    // Create an AuthManager
    let auth_manager = Arc::new(AuthManager::new(auth_dir.clone()).expect("Failed to create AuthManager"));
    
    // Register a test app
    let app = auth_manager.register_app("test_app", &["query"]).expect("Failed to register app");
    
    // Create a NodeClient with a non-existent Unix socket
    let non_existent_socket = PathBuf::from("/tmp/non_existent_socket.sock");
    let node_client = NodeClient::new(
        NodeConnection::UnixSocket(non_existent_socket),
        auth_manager.clone(),
    );
    
    // Send a request, which should fail
    let response = node_client.send_request(
        &app.app_id,
        "test_operation",
        json!({"param1": "value1"}),
    ).await;
    
    // Verify that the request failed with a Node error
    assert!(response.is_err(), "Request to non-existent socket should fail");
    match response {
        Err(FoldClientError::Node(msg)) => {
            assert!(msg.contains("Failed to connect"), "Error message should indicate connection failure");
        }
        _ => panic!("Expected Node error"),
    }
    
    // Create a NodeClient with a non-existent TCP server
    let node_client = NodeClient::new(
        NodeConnection::TcpSocket("127.0.0.1".to_string(), 12345), // Assuming this port is not in use
        auth_manager.clone(),
    );
    
    // Send a request, which should fail
    let response = node_client.send_request(
        &app.app_id,
        "test_operation",
        json!({"param1": "value1"}),
    ).await;
    
    // Verify that the request failed with a Node error
    assert!(response.is_err(), "Request to non-existent TCP server should fail");
    match response {
        Err(FoldClientError::Node(msg)) => {
            assert!(msg.contains("Failed to connect"), "Error message should indicate connection failure");
        }
        _ => panic!("Expected Node error"),
    }
}

#[test]
fn test_node_connection_creation() {
    // Test Unix socket connection creation
    let path = PathBuf::from("/tmp/test.sock");
    let unix_conn = NodeConnection::unix_socket(&path);
    match unix_conn {
        NodeConnection::UnixSocket(p) => {
            assert_eq!(p, path, "Unix socket path does not match");
        }
        _ => panic!("Expected UnixSocket connection"),
    }
    
    // Test TCP socket connection creation
    let host = "localhost";
    let port = 8080;
    let tcp_conn = NodeConnection::tcp_socket(host, port);
    match tcp_conn {
        NodeConnection::TcpSocket(h, p) => {
            assert_eq!(h, host, "TCP host does not match");
            assert_eq!(p, port, "TCP port does not match");
        }
        _ => panic!("Expected TcpSocket connection"),
    }
}
