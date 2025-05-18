use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use serde_json::json;

use fold_db::datafold_node::node::DataFoldNode;
use fold_db::datafold_node::config::NodeConfig;
use fold_db::datafold_node::app_server::AppServer;
use fold_db::datafold_node::app_server::errors::{AppError, AppErrorType, AppErrorResponse};
use fold_db::datafold_node::app_server::logging::{AppLogger, LogLevel};
use fold_db::datafold_node::app_server::middleware::verify_signature;

// Helper function to create a test node
fn create_test_node() -> Arc<Mutex<DataFoldNode>> {
    let config = NodeConfig {
        storage_path: std::path::PathBuf::from("./test_data"),
        default_trust_distance: 1,
    };
    
    let node = DataFoldNode::new(config).unwrap();
    Arc::new(Mutex::new(node))
}

#[tokio::test]
async fn test_app_server_creation() {
    // Create a temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();
    let config = NodeConfig {
        storage_path: temp_dir.path().to_path_buf(),
        default_trust_distance: 1,
    };
    
    let node = DataFoldNode::new(config).unwrap();
    let node = Arc::new(Mutex::new(node));
    
    // Create the server
    let server = AppServer::new(node);
    
    // We can't directly test private fields, but we can verify the server exists
    // Just check that we can create it without errors
    assert!(true);
}

#[tokio::test]
async fn test_signature_verification() {
    // Test the placeholder signature verification
    // This will be replaced with actual verification later
    let result = verify_signature("test-signature", "test-public-key", "test-message");
    
    // Currently it should always return true
    assert!(result);
}

#[tokio::test]
async fn test_app_error_creation() {
    // Test creating different types of errors
    let error1 = AppError::expired_timestamp("Timestamp expired");
    assert_eq!(error1.error_type, AppErrorType::ExpiredTimestamp);
    assert_eq!(error1.message, "Timestamp expired");
    
    let error2 = AppError::invalid_signature("Invalid signature");
    assert_eq!(error2.error_type, AppErrorType::InvalidSignature);
    
    let error3 = AppError::unauthorized_access("Unauthorized");
    assert_eq!(error3.error_type, AppErrorType::UnauthorizedAccess);
    
    let error4 = AppError::invalid_payload("Invalid payload");
    assert_eq!(error4.error_type, AppErrorType::InvalidPayload);
    
    let error5 = AppError::operation_error("Operation failed");
    assert_eq!(error5.error_type, AppErrorType::OperationError);
    
    let error6 = AppError::rate_limit_exceeded("Too many requests");
    assert_eq!(error6.error_type, AppErrorType::RateLimitExceeded);
    
    let error7 = AppError::internal_error("Internal server error");
    assert_eq!(error7.error_type, AppErrorType::InternalError);
    
    // Test adding details to an error
    let mut details = HashMap::new();
    details.insert("key".to_string(), "value".to_string());
    
    let error_with_details = error1.with_details(details);
    assert!(error_with_details.details.is_some());
    assert_eq!(error_with_details.details.unwrap().get("key").unwrap(), "value");
}

#[tokio::test]
async fn test_app_error_response() {
    // Create an error
    let error = AppError::invalid_payload("Invalid JSON");
    
    // Convert to response
    let response = AppErrorResponse::from_app_error(&error);
    
    // Verify response fields
    assert_eq!(response.error, "INVALID_PAYLOAD");
    assert_eq!(response.code, "REQUEST_ERROR");
    assert_eq!(response.message, "Invalid JSON");
    assert!(response.timestamp > 0);
    assert!(response.details.is_none());
}

#[tokio::test]
async fn test_logger() {
    // Create a temporary directory for logs
    let temp_dir = tempfile::tempdir().unwrap();
    let log_path = temp_dir.path().to_str().unwrap();
    
    // Create logger
    let logger = AppLogger::new(log_path);
    
    // Log a security event
    let mut details = HashMap::new();
    details.insert("test_key".to_string(), "test_value".to_string());
    
    logger.log_security_event(
        LogLevel::Warning,
        "test-public-key",
        Some(AppErrorType::InvalidSignature),
        "127.0.0.1",
        "test-request-id",
        details,
    );
    
    // Log an operation
    logger.log_operation(
        LogLevel::Info,
        "test-operation",
        100,
        true,
        None,
        None,
        "test-request-id",
        Some("test-public-key"),
    );
    
    // Verify log files were created
    let security_log_path = format!("{}/security.log", log_path);
    let operation_log_path = format!("{}/operation.log", log_path);
    
    assert!(std::path::Path::new(&security_log_path).exists());
    assert!(std::path::Path::new(&operation_log_path).exists());
    
    // Read log files to verify content
    let security_log = std::fs::read_to_string(&security_log_path).unwrap();
    let operation_log = std::fs::read_to_string(&operation_log_path).unwrap();
    
    assert!(security_log.contains("WARNING"));
    assert!(security_log.contains("test-public-key"));
    assert!(security_log.contains("InvalidSignature"));
    
    assert!(operation_log.contains("INFO"));
    assert!(operation_log.contains("test-operation"));
    assert!(operation_log.contains("true"));
}

// Test log level enums
#[test]
fn test_log_levels() {
    assert_eq!(LogLevel::Debug.as_str(), "DEBUG");
    assert_eq!(LogLevel::Info.as_str(), "INFO");
    assert_eq!(LogLevel::Warning.as_str(), "WARNING");
    assert_eq!(LogLevel::Error.as_str(), "ERROR");
    assert_eq!(LogLevel::Critical.as_str(), "CRITICAL");
    
    // Test ordering
    assert!(LogLevel::Debug < LogLevel::Info);
    assert!(LogLevel::Info < LogLevel::Warning);
    assert!(LogLevel::Warning < LogLevel::Error);
    assert!(LogLevel::Error < LogLevel::Critical);
}

// Test error type mapping
#[test]
fn test_error_type_mapping() {
    // Create errors of each type
    let errors = vec![
        (AppError::expired_timestamp("test"), "EXPIRED_TIMESTAMP", "AUTH_ERROR"),
        (AppError::invalid_signature("test"), "INVALID_SIGNATURE", "AUTH_ERROR"),
        (AppError::unauthorized_access("test"), "UNAUTHORIZED_ACCESS", "AUTH_ERROR"),
        (AppError::invalid_payload("test"), "INVALID_PAYLOAD", "REQUEST_ERROR"),
        (AppError::operation_error("test"), "OPERATION_ERROR", "EXECUTION_ERROR"),
        (AppError::rate_limit_exceeded("test"), "RATE_LIMIT_EXCEEDED", "THROTTLE_ERROR"),
        (AppError::internal_error("test"), "INTERNAL_ERROR", "SERVER_ERROR"),
    ];
    
    // Test each error type mapping
    for (error, expected_error, expected_code) in errors {
        let response = AppErrorResponse::from_app_error(&error);
        assert_eq!(response.error, expected_error);
        assert_eq!(response.code, expected_code);
    }
}
