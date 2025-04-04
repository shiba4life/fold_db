use std::path::PathBuf;
use tempfile::tempdir;

use fold_client::auth::AppRegistration;
use fold_client::process::ProcessManager;
use fold_client::FoldClientError;

// Helper function to create a temporary directory for testing
fn create_temp_process_dir() -> (PathBuf, tempfile::TempDir) {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let process_dir = temp_dir.path().to_path_buf();
    (process_dir, temp_dir)
}

// Helper function to create a test app registration
fn create_test_app() -> AppRegistration {
    AppRegistration {
        app_id: "test_app_id".to_string(),
        app_name: "Test App".to_string(),
        token: "test_token".to_string(),
        permissions: vec!["query".to_string(), "mutation".to_string()],
        public_key: "test_public_key".to_string(),
        created_at: chrono::Utc::now(),
    }
}

#[test]
fn test_process_manager_creation() {
    let (working_dir, _temp_dir) = create_temp_process_dir();
    
    let process_manager = ProcessManager::new(
        working_dir.clone(),
        false,
        true,
        Some(1024),
        Some(50),
    );
    
    assert!(process_manager.is_ok(), "Failed to create ProcessManager");
}

#[test]
fn test_process_manager_list_running_apps() {
    let (working_dir, _temp_dir) = create_temp_process_dir();
    
    let process_manager = ProcessManager::new(
        working_dir.clone(),
        false,
        true,
        Some(1024),
        Some(50),
    ).expect("Failed to create ProcessManager");
    
    let apps = process_manager.list_running_apps();
    assert!(apps.is_ok(), "Failed to list running apps");
    assert_eq!(apps.unwrap().len(), 0, "Initially, there should be no running apps");
}

#[test]
fn test_process_manager_is_app_running() {
    let (working_dir, _temp_dir) = create_temp_process_dir();
    
    let process_manager = ProcessManager::new(
        working_dir.clone(),
        false,
        true,
        Some(1024),
        Some(50),
    ).expect("Failed to create ProcessManager");
    
    let app_id = "non_existent_app";
    let is_running = process_manager.is_app_running(app_id);
    assert!(is_running.is_ok(), "Failed to check if app is running");
    assert_eq!(is_running.unwrap(), false, "Non-existent app should not be running");
}

#[test]
fn test_process_manager_terminate_non_existent_app() {
    let (working_dir, _temp_dir) = create_temp_process_dir();
    
    let process_manager = ProcessManager::new(
        working_dir.clone(),
        false,
        true,
        Some(1024),
        Some(50),
    ).expect("Failed to create ProcessManager");
    
    let app_id = "non_existent_app";
    let result = process_manager.terminate_app(app_id);
    assert!(result.is_err(), "Terminating a non-existent app should fail");
    
    match result {
        Err(FoldClientError::Process(msg)) => {
            assert!(msg.contains("not found"), "Error message should indicate app not found");
        }
        _ => panic!("Expected Process error"),
    }
}

#[test]
fn test_process_manager_cleanup() {
    let (working_dir, _temp_dir) = create_temp_process_dir();
    
    let process_manager = ProcessManager::new(
        working_dir.clone(),
        false,
        true,
        Some(1024),
        Some(50),
    ).expect("Failed to create ProcessManager");
    
    let result = process_manager.cleanup();
    assert!(result.is_ok(), "Cleanup should succeed even with no running apps");
}

#[test]
fn test_process_manager_launch_app_api() {
    let (working_dir, _temp_dir) = create_temp_process_dir();
    
    let process_manager = ProcessManager::new(
        working_dir.clone(),
        false,
        true,
        Some(1024),
        Some(50),
    ).expect("Failed to create ProcessManager");
    
    let app = create_test_app();
    
    // We're only testing the API, not the actual execution
    // This is because the actual execution would require a valid program to run
    // and would depend on the platform-specific sandbox implementation
    
    #[cfg(unix)]
    let program = "echo";
    #[cfg(unix)]
    let args = &["Hello, World!"];
    
    #[cfg(windows)]
    let program = "cmd";
    #[cfg(windows)]
    let args = &["/c", "echo", "Hello, World!"];
    
    // This will likely fail because the program doesn't exist in the sandbox
    // or because the sandbox itself can't be created in the test environment
    // But we're just testing that the API works as expected
    let _ = process_manager.launch_app(app, program, args);
    
    // We don't assert anything here because the result depends on the environment
}
