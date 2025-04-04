use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use uuid::Uuid;

use fold_client::auth::AuthManager;
use fold_client::FoldClientError;
use ed25519_dalek::Signer;

// Helper function to create a temporary directory for testing
fn create_temp_auth_dir() -> (PathBuf, tempfile::TempDir) {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let auth_dir = temp_dir.path().join("auth");
    fs::create_dir_all(&auth_dir).expect("Failed to create auth directory");
    (auth_dir, temp_dir)
}

#[test]
fn test_auth_manager_creation() {
    let (auth_dir, _temp_dir) = create_temp_auth_dir();
    
    // Create a new AuthManager
    let auth_manager = AuthManager::new(auth_dir.clone());
    assert!(auth_manager.is_ok(), "Failed to create AuthManager");
    
    // Verify that the keypair file was created
    let keypair_path = auth_dir.join("fold_client.key");
    assert!(keypair_path.exists(), "Keypair file was not created");
}

#[test]
fn test_app_registration() {
    let (auth_dir, _temp_dir) = create_temp_auth_dir();
    
    // Create a new AuthManager
    let auth_manager = AuthManager::new(auth_dir.clone()).expect("Failed to create AuthManager");
    
    // Register a new app
    let app_name = "test_app";
    let permissions = &["query", "mutation"];
    let app = auth_manager.register_app(app_name, permissions);
    assert!(app.is_ok(), "Failed to register app");
    
    let app = app.unwrap();
    assert_eq!(app.app_name, app_name, "App name does not match");
    assert_eq!(app.permissions.len(), permissions.len(), "Permissions count does not match");
    
    // Verify that the app registration file was created
    let app_path = auth_dir.join(format!("{}.json", app.app_id));
    assert!(app_path.exists(), "App registration file was not created");
    
    // Verify that the app keypair file was created
    let keypair_path = auth_dir.join(format!("{}.key", app.app_id));
    assert!(keypair_path.exists(), "App keypair file was not created");
}

#[test]
fn test_get_app() {
    let (auth_dir, _temp_dir) = create_temp_auth_dir();
    
    // Create a new AuthManager
    let auth_manager = AuthManager::new(auth_dir.clone()).expect("Failed to create AuthManager");
    
    // Register a new app
    let app_name = "test_app";
    let permissions = &["query", "mutation"];
    let app = auth_manager.register_app(app_name, permissions).expect("Failed to register app");
    
    // Get the app by ID
    let retrieved_app = auth_manager.get_app(&app.app_id);
    assert!(retrieved_app.is_ok(), "Failed to get app");
    
    let retrieved_app = retrieved_app.unwrap();
    assert_eq!(retrieved_app.app_id, app.app_id, "App ID does not match");
    assert_eq!(retrieved_app.app_name, app.app_name, "App name does not match");
    assert_eq!(retrieved_app.token, app.token, "App token does not match");
    
    // Try to get a non-existent app
    let non_existent_app = auth_manager.get_app("non_existent_app");
    assert!(non_existent_app.is_err(), "Getting non-existent app should fail");
    
    match non_existent_app {
        Err(FoldClientError::Auth(msg)) => {
            assert!(msg.contains("App not found"), "Error message should indicate app not found");
        }
        _ => panic!("Expected Auth error"),
    }
}

#[test]
fn test_check_permission() {
    let (auth_dir, _temp_dir) = create_temp_auth_dir();
    
    // Create a new AuthManager
    let auth_manager = AuthManager::new(auth_dir.clone()).expect("Failed to create AuthManager");
    
    // Register a new app with specific permissions
    let app_name = "test_app";
    let permissions = &["query", "list_schemas"];
    let app = auth_manager.register_app(app_name, permissions).expect("Failed to register app");
    
    // Check permissions
    let has_query_permission = auth_manager.check_permission(&app.app_id, "query");
    assert!(has_query_permission.is_ok(), "Failed to check permission");
    assert!(has_query_permission.unwrap(), "App should have query permission");
    
    let has_list_schemas_permission = auth_manager.check_permission(&app.app_id, "list_schemas");
    assert!(has_list_schemas_permission.is_ok(), "Failed to check permission");
    assert!(has_list_schemas_permission.unwrap(), "App should have list_schemas permission");
    
    let has_mutation_permission = auth_manager.check_permission(&app.app_id, "mutation");
    assert!(has_mutation_permission.is_ok(), "Failed to check permission");
    assert!(!has_mutation_permission.unwrap(), "App should not have mutation permission");
}

#[test]
fn test_verify_app_token() {
    let (auth_dir, _temp_dir) = create_temp_auth_dir();
    
    // Create a new AuthManager
    let auth_manager = AuthManager::new(auth_dir.clone()).expect("Failed to create AuthManager");
    
    // Register a new app
    let app_name = "test_app";
    let permissions = &["query"];
    let app = auth_manager.register_app(app_name, permissions).expect("Failed to register app");
    
    // Verify the correct token
    let valid_token_result = auth_manager.verify_app_token(&app.app_id, &app.token);
    assert!(valid_token_result.is_ok(), "Failed to verify token");
    assert!(valid_token_result.unwrap(), "Valid token should be verified");
    
    // Verify an incorrect token
    let invalid_token = Uuid::new_v4().to_string();
    let invalid_token_result = auth_manager.verify_app_token(&app.app_id, &invalid_token);
    assert!(invalid_token_result.is_ok(), "Failed to verify token");
    assert!(!invalid_token_result.unwrap(), "Invalid token should not be verified");
}

#[test]
fn test_sign_and_verify_message() {
    let (auth_dir, _temp_dir) = create_temp_auth_dir();
    
    // Create a new AuthManager
    let auth_manager = AuthManager::new(auth_dir.clone()).expect("Failed to create AuthManager");
    
    // Register a new app
    let app_name = "test_app";
    let permissions = &["query"];
    let app = auth_manager.register_app(app_name, permissions).expect("Failed to register app");
    
    // Get the app keypair file path
    let keypair_path = auth_dir.join(format!("{}.key", app.app_id));
    
    // Read the keypair
    let keypair_bytes = fs::read(keypair_path).expect("Failed to read keypair file");
    let keypair = ed25519_dalek::Keypair::from_bytes(&keypair_bytes).expect("Failed to parse keypair");
    
    // Sign a message with the app's keypair
    let message = b"Test message";
    let signature = keypair.sign(message);
    
    // Verify the signature
    let verify_result = auth_manager.verify_signature(&app.app_id, message, &signature.to_bytes());
    assert!(verify_result.is_ok(), "Failed to verify signature");
    assert!(verify_result.unwrap(), "Valid signature should be verified");
    
    // Verify with a different message
    let different_message = b"Different message";
    let verify_different_result = auth_manager.verify_signature(&app.app_id, different_message, &signature.to_bytes());
    assert!(verify_different_result.is_ok(), "Failed to verify signature");
    assert!(!verify_different_result.unwrap(), "Signature for different message should not be verified");
}

#[test]
fn test_list_apps() {
    let (auth_dir, _temp_dir) = create_temp_auth_dir();
    
    // Create a new AuthManager
    let auth_manager = AuthManager::new(auth_dir.clone()).expect("Failed to create AuthManager");
    
    // Initially, there should be no apps
    let apps = auth_manager.list_apps().expect("Failed to list apps");
    assert_eq!(apps.len(), 0, "Initially, there should be no apps");
    
    // Register a few apps
    let app1 = auth_manager.register_app("app1", &["query"]).expect("Failed to register app1");
    let app2 = auth_manager.register_app("app2", &["mutation"]).expect("Failed to register app2");
    let app3 = auth_manager.register_app("app3", &["list_schemas"]).expect("Failed to register app3");
    
    // List apps again
    let apps = auth_manager.list_apps().expect("Failed to list apps");
    assert_eq!(apps.len(), 3, "There should be 3 apps");
    
    // Verify that all registered apps are in the list
    let app_ids: Vec<String> = apps.iter().map(|app| app.app_id.clone()).collect();
    assert!(app_ids.contains(&app1.app_id), "App1 should be in the list");
    assert!(app_ids.contains(&app2.app_id), "App2 should be in the list");
    assert!(app_ids.contains(&app3.app_id), "App3 should be in the list");
}
