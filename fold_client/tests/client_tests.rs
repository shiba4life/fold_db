use std::path::PathBuf;
use tempfile::tempdir;
use std::os::unix::fs::PermissionsExt;

use fold_client::{FoldClient, FoldClientConfig, FoldClientError};

// Helper function to create a temporary directory for testing
fn create_temp_dirs() -> (PathBuf, PathBuf, tempfile::TempDir, tempfile::TempDir) {
    let app_socket_dir = tempdir().expect("Failed to create temp directory for app sockets");
    let app_data_dir = tempdir().expect("Failed to create temp directory for app data");
    
    (
        app_socket_dir.path().to_path_buf(),
        app_data_dir.path().to_path_buf(),
        app_socket_dir,
        app_data_dir,
    )
}

#[tokio::test]
async fn test_fold_client_creation() {
    let (app_socket_dir, app_data_dir, _socket_dir_guard, _data_dir_guard) = create_temp_dirs();
    
    // Create a configuration
    let config = FoldClientConfig {
        node_socket_path: None,
        node_tcp_address: Some(("127.0.0.1".to_string(), 9000)),
        app_socket_dir,
        app_data_dir,
        allow_network_access: false,
        allow_filesystem_access: false,
        max_memory_mb: Some(1024),
        max_cpu_percent: Some(50),
        private_key: None,
    };
    
    // Create a FoldClient
    let fold_client = FoldClient::with_config(config);
    assert!(fold_client.is_ok(), "Failed to create FoldClient");
}

#[tokio::test]
async fn test_fold_client_with_invalid_config() {
    let (app_socket_dir, app_data_dir, _socket_dir_guard, _data_dir_guard) = create_temp_dirs();
    
    // Create a configuration with no node connection
    let config = FoldClientConfig {
        node_socket_path: None,
        node_tcp_address: None,
        app_socket_dir,
        app_data_dir,
        allow_network_access: false,
        allow_filesystem_access: false,
        max_memory_mb: Some(1024),
        max_cpu_percent: Some(50),
        private_key: None,
    };
    
    // Create a FoldClient, which should fail
    let fold_client = FoldClient::with_config(config);
    assert!(fold_client.is_err(), "FoldClient creation should fail with invalid config");
    
    match fold_client {
        Err(FoldClientError::Config(msg)) => {
            assert!(msg.contains("No node connection specified"), "Error message should indicate no node connection");
        }
        _ => panic!("Expected Config error"),
    }
}

#[tokio::test]
async fn test_fold_client_start_stop() {
    let (app_socket_dir, app_data_dir, _socket_dir_guard, _data_dir_guard) = create_temp_dirs();
    
    // Create a configuration
    let config = FoldClientConfig {
        node_socket_path: None,
        node_tcp_address: Some(("127.0.0.1".to_string(), 9000)),
        app_socket_dir,
        app_data_dir,
        allow_network_access: false,
        allow_filesystem_access: false,
        max_memory_mb: Some(1024),
        max_cpu_percent: Some(50),
        private_key: None,
    };
    
    // Create a FoldClient
    let mut fold_client = FoldClient::with_config(config).expect("Failed to create FoldClient");
    
    // Start the FoldClient
    let start_result = fold_client.start().await;
    assert!(start_result.is_ok(), "Failed to start FoldClient");
    
    // Stop the FoldClient
    let stop_result = fold_client.stop().await;
    assert!(stop_result.is_ok(), "Failed to stop FoldClient");
}

#[tokio::test]
async fn test_fold_client_register_app() {
    let (app_socket_dir, app_data_dir, _socket_dir_guard, _data_dir_guard) = create_temp_dirs();
    
    // Create a configuration
    let config = FoldClientConfig {
        node_socket_path: None,
        node_tcp_address: Some(("127.0.0.1".to_string(), 9000)),
        app_socket_dir,
        app_data_dir,
        allow_network_access: false,
        allow_filesystem_access: false,
        max_memory_mb: Some(1024),
        max_cpu_percent: Some(50),
        private_key: None,
    };
    
    // Create a FoldClient
    let mut fold_client = FoldClient::with_config(config).expect("Failed to create FoldClient");
    
    // Start the FoldClient
    fold_client.start().await.expect("Failed to start FoldClient");
    
    // Register an app
    let app_name = "test_app";
    let permissions = &["query", "mutation"];
    let app = fold_client.register_app(app_name, permissions).await;
    assert!(app.is_ok(), "Failed to register app");
    
    let app = app.unwrap();
    assert_eq!(app.app_name, app_name, "App name does not match");
    assert_eq!(app.permissions.len(), permissions.len(), "Permissions count does not match");
    
    // Stop the FoldClient
    fold_client.stop().await.expect("Failed to stop FoldClient");
}

#[tokio::test]
async fn test_fold_client_default_config() {
    // Create a FoldClient with default configuration
    let fold_client = FoldClient::new();
    assert!(fold_client.is_ok(), "Failed to create FoldClient with default config");
    
    let _fold_client = fold_client.unwrap();
    
    // We can't easily test the default configuration values directly since they're private,
    // but we can verify that the FoldClient was created successfully
}

#[tokio::test]
async fn test_fold_client_app_lifecycle() {
    let (app_socket_dir, app_data_dir, _socket_dir_guard, _data_dir_guard) = create_temp_dirs();
    
    // Create a configuration
    let config = FoldClientConfig {
        node_socket_path: None,
        node_tcp_address: Some(("127.0.0.1".to_string(), 9000)),
        app_socket_dir,
        app_data_dir: app_data_dir.clone(),
        allow_network_access: true,
        allow_filesystem_access: true,
        max_memory_mb: Some(1024),
        max_cpu_percent: Some(50),
        private_key: None,
    };
    
    // Create a FoldClient
    let mut fold_client = FoldClient::with_config(config).expect("Failed to create FoldClient");
    
    // Start the FoldClient
    fold_client.start().await.expect("Failed to start FoldClient");
    
    // Register an app
    let app_name = "test_app";
    let permissions = &["query", "mutation"];
    let app = fold_client.register_app(app_name, permissions).await.expect("Failed to register app");
    
    // Create a dummy program for testing
    let program_path = app_data_dir.join("dummy_program.sh");
    std::fs::write(&program_path, "#!/bin/sh\nsleep 1\n").expect("Failed to write dummy program");
    std::fs::set_permissions(&program_path, std::fs::Permissions::from_mode(0o755)).expect("Failed to set permissions");
    
    // Launch the app
    let launch_result = fold_client.launch_app(&app.app_id, program_path.to_str().unwrap(), &[]).await;
    assert!(launch_result.is_ok(), "Failed to launch app");
    
    // Check if the app is running
    let is_running = fold_client.is_app_running(&app.app_id).await.expect("Failed to check if app is running");
    assert!(is_running, "App should be running");
    
    // List running apps
    let running_apps = fold_client.list_running_apps().await.expect("Failed to list running apps");
    assert!(running_apps.contains(&app.app_id), "App should be in the list of running apps");
    
    // Terminate the app
    let terminate_result = fold_client.terminate_app(&app.app_id).await;
    assert!(terminate_result.is_ok(), "Failed to terminate app");
    
    // Check if the app is still running (it shouldn't be)
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; // Give it a moment to terminate
    let is_running = fold_client.is_app_running(&app.app_id).await.expect("Failed to check if app is running");
    assert!(!is_running, "App should not be running after termination");
    
    // Stop the FoldClient
    fold_client.stop().await.expect("Failed to stop FoldClient");
}

#[test]
fn test_fold_client_error_display() {
    // Create various error types and check their display implementation
    let auth_error = FoldClientError::Auth("Authentication failed".to_string());
    assert_eq!(
        format!("{}", auth_error),
        "Authentication error: Authentication failed",
        "Auth error display does not match"
    );
    
    let ipc_error = FoldClientError::Ipc("IPC connection failed".to_string());
    assert_eq!(
        format!("{}", ipc_error),
        "IPC error: IPC connection failed",
        "IPC error display does not match"
    );
    
    let node_error = FoldClientError::Node("Node connection failed".to_string());
    assert_eq!(
        format!("{}", node_error),
        "Node communication error: Node connection failed",
        "Node error display does not match"
    );
    
    let process_error = FoldClientError::Process("Process creation failed".to_string());
    assert_eq!(
        format!("{}", process_error),
        "Process management error: Process creation failed",
        "Process error display does not match"
    );
    
    let sandbox_error = FoldClientError::Sandbox("Sandbox creation failed".to_string());
    assert_eq!(
        format!("{}", sandbox_error),
        "Sandbox error: Sandbox creation failed",
        "Sandbox error display does not match"
    );
    
    let io_error = FoldClientError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"));
    assert!(
        format!("{}", io_error).contains("I/O error"),
        "IO error display should contain 'I/O error'"
    );
    
    let serialization_error = FoldClientError::Serialization("Failed to serialize".to_string());
    assert_eq!(
        format!("{}", serialization_error),
        "Serialization error: Failed to serialize",
        "Serialization error display does not match"
    );
    
    let config_error = FoldClientError::Config("Invalid configuration".to_string());
    assert_eq!(
        format!("{}", config_error),
        "Configuration error: Invalid configuration",
        "Config error display does not match"
    );
}

#[test]
fn test_fold_client_config_default() {
    let config = FoldClientConfig::default();
    
    // Check that the default values are set correctly
    assert_eq!(config.node_socket_path, None, "Default node_socket_path should be None");
    assert_eq!(
        config.node_tcp_address,
        Some(("127.0.0.1".to_string(), 9000)),
        "Default node_tcp_address should be 127.0.0.1:9000"
    );
    assert!(!config.allow_network_access, "Default allow_network_access should be false");
    assert!(!config.allow_filesystem_access, "Default allow_filesystem_access should be false");
    assert_eq!(config.max_memory_mb, Some(1024), "Default max_memory_mb should be 1024");
    assert_eq!(config.max_cpu_percent, Some(50), "Default max_cpu_percent should be 50");
    
    // Check that the paths are set correctly
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let datafold_dir = home_dir.join(".datafold");
    assert_eq!(
        config.app_socket_dir,
        datafold_dir.join("sockets"),
        "Default app_socket_dir should be ~/.datafold/sockets"
    );
    assert_eq!(
        config.app_data_dir,
        datafold_dir.join("app_data"),
        "Default app_data_dir should be ~/.datafold/app_data"
    );
}
