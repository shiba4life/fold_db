use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::json;
use uuid::Uuid;

use fold_client::auth::AuthManager;
use fold_client::ipc::{AppRequest, AppResponse, get_app_socket_path};
use fold_client::ipc::server::IpcServer;
use fold_client::node::{NodeClient, NodeConnection};

// Helper function to create a temporary directory for testing
fn create_temp_auth_dir() -> (PathBuf, tempfile::TempDir) {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let auth_dir = temp_dir.path().join("auth");
    std::fs::create_dir_all(&auth_dir).expect("Failed to create auth directory");
    (auth_dir, temp_dir)
}

// Helper function to create a mock client that connects to an IPC server
#[allow(dead_code)]
async fn create_mock_client(socket_path: &PathBuf) -> UnixStream {
    // Connect to the socket
    UnixStream::connect(socket_path).await.expect("Failed to connect to socket")
}

// Helper function to send a request to an IPC server and receive a response
#[allow(dead_code)]
async fn send_request_and_receive_response(
    mut stream: UnixStream,
    app_id: &str,
    token: &str,
    operation: &str,
    params: serde_json::Value,
) -> AppResponse {
    // Create a request
    let request = AppRequest::new(app_id, token, operation, params);
    
    // Serialize the request
    let request_bytes = serde_json::to_vec(&request).expect("Failed to serialize request");
    
    // Send the request length
    stream.write_u32(request_bytes.len() as u32).await.expect("Failed to send request length");
    
    // Send the request
    stream.write_all(&request_bytes).await.expect("Failed to send request");
    
    // Read the response length
    let response_len = stream.read_u32().await.expect("Failed to read response length") as usize;
    
    // Read the response
    let mut response_bytes = vec![0u8; response_len];
    stream.read_exact(&mut response_bytes).await.expect("Failed to read response");
    
    // Deserialize the response
    serde_json::from_slice(&response_bytes).expect("Failed to deserialize response")
}

#[tokio::test]
async fn test_ipc_server_creation() {
    let (auth_dir, _temp_dir) = create_temp_auth_dir();
    
    // Create a temporary socket directory
    let socket_dir = tempdir().expect("Failed to create temp directory for socket");
    let socket_dir_path = socket_dir.path().to_path_buf();
    
    // Create an AuthManager
    let auth_manager = Arc::new(AuthManager::new(auth_dir.clone()).expect("Failed to create AuthManager"));
    
    // Create a NodeClient with a mock connection
    let node_client = Arc::new(NodeClient::new(
        NodeConnection::TcpSocket("127.0.0.1".to_string(), 9000),
        auth_manager.clone(),
    ));
    
    // Create an IpcServer
    let ipc_server = IpcServer::new(
        socket_dir_path.clone(),
        auth_manager.clone(),
        node_client.clone(),
    );
    
    assert!(ipc_server.is_ok(), "Failed to create IpcServer");
}

#[tokio::test]
async fn test_app_request_response() {
    // Create a request
    let app_id = Uuid::new_v4().to_string();
    let token = Uuid::new_v4().to_string();
    let operation = "test_operation";
    let params = json!({"param1": "value1", "param2": 42});
    
    let request = AppRequest::new(&app_id, &token, operation, params.clone());
    
    // Verify the request fields
    assert_eq!(request.app_id, app_id, "App ID does not match");
    assert_eq!(request.token, token, "Token does not match");
    assert_eq!(request.operation, operation, "Operation does not match");
    assert_eq!(request.params, params, "Params do not match");
    assert!(request.signature.is_none(), "Signature should be None initially");
    
    // Sign the request
    let signature = "test_signature".to_string();
    let mut signed_request = request.clone();
    signed_request.sign(signature.clone());
    
    // Verify the signature
    assert_eq!(signed_request.signature, Some(signature), "Signature does not match");
    
    // Create a success response
    let result = json!({"result_key": "result_value"});
    let success_response = AppResponse::success(&request.request_id, result.clone());
    
    // Verify the success response fields
    assert_eq!(success_response.request_id, request.request_id, "Request ID does not match");
    assert!(success_response.success, "Success flag should be true");
    assert_eq!(success_response.result, Some(result), "Result does not match");
    assert!(success_response.error.is_none(), "Error should be None for success response");
    
    // Create an error response
    let error_message = "Test error message";
    let error_response = AppResponse::error(&request.request_id, error_message);
    
    // Verify the error response fields
    assert_eq!(error_response.request_id, request.request_id, "Request ID does not match");
    assert!(!error_response.success, "Success flag should be false");
    assert!(error_response.result.is_none(), "Result should be None for error response");
    assert_eq!(error_response.error, Some(error_message.to_string()), "Error message does not match");
}

#[test]
fn test_get_app_socket_path() {
    let app_socket_dir = PathBuf::from("/tmp/app_sockets");
    let app_id = "test_app_id";
    
    let socket_path = get_app_socket_path(&app_socket_dir, app_id);
    
    assert_eq!(
        socket_path,
        app_socket_dir.join(format!("{}.sock", app_id)),
        "Socket path does not match expected value"
    );
}

#[tokio::test]
async fn test_ipc_server_start_stop() {
    let (auth_dir, _temp_dir) = create_temp_auth_dir();
    
    // Create a temporary socket directory
    let socket_dir = tempdir().expect("Failed to create temp directory for socket");
    let socket_dir_path = socket_dir.path().to_path_buf();
    
    // Create an AuthManager
    let auth_manager = Arc::new(AuthManager::new(auth_dir.clone()).expect("Failed to create AuthManager"));
    
    // Register a test app
    let app = auth_manager.register_app("test_app", &["query"]).expect("Failed to register app");
    
    // Create a NodeClient with a mock connection
    let node_client = Arc::new(NodeClient::new(
        NodeConnection::TcpSocket("127.0.0.1".to_string(), 9000),
        auth_manager.clone(),
    ));
    
    // Create an IpcServer
    let mut ipc_server = IpcServer::new(
        socket_dir_path.clone(),
        auth_manager.clone(),
        node_client.clone(),
    ).expect("Failed to create IpcServer");
    
    // Start the IPC server
    let start_result = ipc_server.start().await;
    assert!(start_result.is_ok(), "Failed to start IpcServer");
    
    // Verify that the socket file was created
    let socket_path = get_app_socket_path(&socket_dir_path, &app.app_id);
    assert!(socket_path.exists(), "Socket file was not created");
    
    // Stop the IPC server
    let stop_result = ipc_server.stop().await;
    assert!(stop_result.is_ok(), "Failed to stop IpcServer");
    
    // Verify that the socket file was removed
    assert!(!socket_path.exists(), "Socket file was not removed");
}

#[tokio::test]
async fn test_app_request_serialization() {
    // Create a request
    let app_id = Uuid::new_v4().to_string();
    let token = Uuid::new_v4().to_string();
    let operation = "test_operation";
    let params = json!({"param1": "value1", "param2": 42});
    
    let request = AppRequest::new(&app_id, &token, operation, params.clone());
    
    // Serialize the request
    let serialized = serde_json::to_string(&request).expect("Failed to serialize request");
    
    // Deserialize the request
    let deserialized: AppRequest = serde_json::from_str(&serialized).expect("Failed to deserialize request");
    
    // Verify that the deserialized request matches the original
    assert_eq!(deserialized.app_id, request.app_id, "App ID does not match after serialization");
    assert_eq!(deserialized.token, request.token, "Token does not match after serialization");
    assert_eq!(deserialized.operation, request.operation, "Operation does not match after serialization");
    assert_eq!(deserialized.params, request.params, "Params do not match after serialization");
    assert_eq!(deserialized.signature, request.signature, "Signature does not match after serialization");
}

#[tokio::test]
async fn test_app_response_serialization() {
    // Create a request ID
    let request_id = Uuid::new_v4().to_string();
    
    // Create a success response
    let result = json!({"result_key": "result_value"});
    let success_response = AppResponse::success(&request_id, result.clone());
    
    // Serialize the response
    let serialized = serde_json::to_string(&success_response).expect("Failed to serialize response");
    
    // Deserialize the response
    let deserialized: AppResponse = serde_json::from_str(&serialized).expect("Failed to deserialize response");
    
    // Verify that the deserialized response matches the original
    assert_eq!(deserialized.request_id, success_response.request_id, "Request ID does not match after serialization");
    assert_eq!(deserialized.success, success_response.success, "Success flag does not match after serialization");
    assert_eq!(deserialized.result, success_response.result, "Result does not match after serialization");
    assert_eq!(deserialized.error, success_response.error, "Error does not match after serialization");
    
    // Create an error response
    let error_message = "Test error message";
    let error_response = AppResponse::error(&request_id, error_message);
    
    // Serialize the response
    let serialized = serde_json::to_string(&error_response).expect("Failed to serialize response");
    
    // Deserialize the response
    let deserialized: AppResponse = serde_json::from_str(&serialized).expect("Failed to deserialize response");
    
    // Verify that the deserialized response matches the original
    assert_eq!(deserialized.request_id, error_response.request_id, "Request ID does not match after serialization");
    assert_eq!(deserialized.success, error_response.success, "Success flag does not match after serialization");
    assert_eq!(deserialized.result, error_response.result, "Result does not match after serialization");
    assert_eq!(deserialized.error, error_response.error, "Error does not match after serialization");
}
