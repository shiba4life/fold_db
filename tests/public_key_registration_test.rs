//! Integration tests for public key registration API endpoints
//!
//! This module provides comprehensive tests for the public key registration
//! functionality including validation, storage, and retrieval.

use tempfile::TempDir;
use actix_web::{test, web, App};
use std::sync::Arc;
use tokio::sync::Mutex;

use datafold::datafold_node::{
    DataFoldNode,
    NodeConfig,
    http_server::AppState,
    crypto_routes::{
        register_public_key, get_public_key_status,
        PublicKeyRegistrationRequest, ApiResponse, PublicKeyRegistrationResponse,
        PublicKeyStatusResponse,
    },
};
use datafold::crypto::ed25519::generate_master_keypair;

/// Test fixture for public key registration tests
struct PublicKeyRegistrationTestFixture {
    _temp_dir: TempDir,
    node: DataFoldNode,
    app_state: web::Data<AppState>,
}

impl PublicKeyRegistrationTestFixture {
    /// Create a new test fixture with a fresh database
    async fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config = NodeConfig::new(temp_dir.path().to_path_buf());
        let node = DataFoldNode::new(config).expect("Failed to create test node");
        
        let app_state = web::Data::new(AppState {
            signature_auth: None,
            node: Arc::new(Mutex::new(node.clone())),
        });

        Self { 
            _temp_dir: temp_dir,
            node,
            app_state,
        }
    }

    /// Generate a valid Ed25519 public key for testing
    fn generate_test_public_key() -> String {
        let keypair = generate_master_keypair().expect("Failed to generate test keypair");
        hex::encode(keypair.public_key_bytes())
    }
}

#[tokio::test]
async fn test_register_public_key_success() {
    let fixture = PublicKeyRegistrationTestFixture::new().await;
    let public_key = PublicKeyRegistrationTestFixture::generate_test_public_key();

    let request = PublicKeyRegistrationRequest {
        client_id: Some("test-client-1".to_string()),
        user_id: Some("user-123".to_string()),
        public_key: public_key.clone(),
        key_name: Some("Test Key".to_string()),
        metadata: Some(std::collections::HashMap::from([
            ("environment".to_string(), "test".to_string()),
            ("purpose".to_string(), "integration-test".to_string()),
        ])),
    };

    let app = test::init_service(
        App::new()
            .app_data(fixture.app_state.clone())
            .route("/register", web::post().to(register_public_key))
    ).await;

    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201); // Created

    let body: ApiResponse<PublicKeyRegistrationResponse> = test::read_body_json(resp).await;
    assert!(body.success);
    
    let data = body.data.unwrap();
    assert_eq!(data.client_id, "test-client-1");
    assert_eq!(data.public_key, public_key);
    assert_eq!(data.key_name, Some("Test Key".to_string()));
    assert_eq!(data.status, "active");
    assert!(!data.registration_id.is_empty());
}

#[tokio::test]
async fn test_register_public_key_auto_generate_client_id() {
    let fixture = PublicKeyRegistrationTestFixture::new().await;
    let public_key = PublicKeyRegistrationTestFixture::generate_test_public_key();

    let request = PublicKeyRegistrationRequest {
        client_id: None, // Should auto-generate
        user_id: None,
        public_key: public_key.clone(),
        key_name: None,
        metadata: None,
    };

    let app = test::init_service(
        App::new()
            .app_data(fixture.app_state.clone())
            .route("/register", web::post().to(register_public_key))
    ).await;

    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let body: ApiResponse<PublicKeyRegistrationResponse> = test::read_body_json(resp).await;
    assert!(body.success);
    
    let data = body.data.unwrap();
    assert!(data.client_id.starts_with("client_"));
    assert_eq!(data.public_key, public_key);
    assert_eq!(data.status, "active");
}

#[tokio::test]
async fn test_register_public_key_invalid_hex() {
    let fixture = PublicKeyRegistrationTestFixture::new().await;

    let request = PublicKeyRegistrationRequest {
        client_id: Some("test-client-invalid".to_string()),
        user_id: None,
        public_key: "invalid-hex-string".to_string(),
        key_name: None,
        metadata: None,
    };

    let app = test::init_service(
        App::new()
            .app_data(fixture.app_state.clone())
            .route("/register", web::post().to(register_public_key))
    ).await;

    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400); // Bad Request

    let body: ApiResponse<()> = test::read_body_json(resp).await;
    assert!(!body.success);
    assert!(body.error.is_some());
    assert_eq!(body.error.unwrap().code, "INVALID_PUBLIC_KEY");
}

#[tokio::test]
async fn test_register_public_key_wrong_length() {
    let fixture = PublicKeyRegistrationTestFixture::new().await;

    let request = PublicKeyRegistrationRequest {
        client_id: Some("test-client-wrong-length".to_string()),
        user_id: None,
        public_key: "deadbeef".to_string(), // Only 4 bytes, should be 32
        key_name: None,
        metadata: None,
    };

    let app = test::init_service(
        App::new()
            .app_data(fixture.app_state.clone())
            .route("/register", web::post().to(register_public_key))
    ).await;

    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let body: ApiResponse<()> = test::read_body_json(resp).await;
    assert!(!body.success);
    assert_eq!(body.error.unwrap().code, "INVALID_PUBLIC_KEY");
}

#[tokio::test]
async fn test_register_public_key_empty() {
    let fixture = PublicKeyRegistrationTestFixture::new().await;

    let request = PublicKeyRegistrationRequest {
        client_id: Some("test-client-empty".to_string()),
        user_id: None,
        public_key: "".to_string(),
        key_name: None,
        metadata: None,
    };

    let app = test::init_service(
        App::new()
            .app_data(fixture.app_state.clone())
            .route("/register", web::post().to(register_public_key))
    ).await;

    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let body: ApiResponse<()> = test::read_body_json(resp).await;
    assert!(!body.success);
    assert_eq!(body.error.unwrap().code, "INVALID_PUBLIC_KEY");
}

#[tokio::test]
async fn test_register_duplicate_client_id() {
    let fixture = PublicKeyRegistrationTestFixture::new().await;
    let public_key1 = PublicKeyRegistrationTestFixture::generate_test_public_key();
    let public_key2 = PublicKeyRegistrationTestFixture::generate_test_public_key();

    let app = test::init_service(
        App::new()
            .app_data(fixture.app_state.clone())
            .route("/register", web::post().to(register_public_key))
    ).await;

    // First registration should succeed
    let request1 = PublicKeyRegistrationRequest {
        client_id: Some("duplicate-client".to_string()),
        user_id: None,
        public_key: public_key1,
        key_name: Some("First Key".to_string()),
        metadata: None,
    };

    let req1 = test::TestRequest::post()
        .uri("/register")
        .set_json(&request1)
        .to_request();

    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), 201);

    // Second registration with same client_id should fail
    let request2 = PublicKeyRegistrationRequest {
        client_id: Some("duplicate-client".to_string()),
        user_id: None,
        public_key: public_key2,
        key_name: Some("Second Key".to_string()),
        metadata: None,
    };

    let req2 = test::TestRequest::post()
        .uri("/register")
        .set_json(&request2)
        .to_request();

    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), 409); // Conflict

    let body: ApiResponse<()> = test::read_body_json(resp2).await;
    assert!(!body.success);
    assert_eq!(body.error.unwrap().code, "CLIENT_ALREADY_REGISTERED");
}

#[tokio::test]
async fn test_register_duplicate_public_key() {
    let fixture = PublicKeyRegistrationTestFixture::new().await;
    let public_key = PublicKeyRegistrationTestFixture::generate_test_public_key();

    let app = test::init_service(
        App::new()
            .app_data(fixture.app_state.clone())
            .route("/register", web::post().to(register_public_key))
    ).await;

    // First registration should succeed
    let request1 = PublicKeyRegistrationRequest {
        client_id: Some("client-1".to_string()),
        user_id: None,
        public_key: public_key.clone(),
        key_name: Some("First Key".to_string()),
        metadata: None,
    };

    let req1 = test::TestRequest::post()
        .uri("/register")
        .set_json(&request1)
        .to_request();

    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), 201);

    // Second registration with same public key should fail
    let request2 = PublicKeyRegistrationRequest {
        client_id: Some("client-2".to_string()),
        user_id: None,
        public_key,
        key_name: Some("Duplicate Key".to_string()),
        metadata: None,
    };

    let req2 = test::TestRequest::post()
        .uri("/register")
        .set_json(&request2)
        .to_request();

    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), 409); // Conflict

    let body: ApiResponse<()> = test::read_body_json(resp2).await;
    assert!(!body.success);
    assert_eq!(body.error.unwrap().code, "DUPLICATE_PUBLIC_KEY");
}

#[tokio::test]
async fn test_get_public_key_status_success() {
    let fixture = PublicKeyRegistrationTestFixture::new().await;
    let public_key = PublicKeyRegistrationTestFixture::generate_test_public_key();
    let client_id = "status-test-client";

    let app = test::init_service(
        App::new()
            .app_data(fixture.app_state.clone())
            .route("/register", web::post().to(register_public_key))
            .route("/status/{client_id}", web::get().to(get_public_key_status))
    ).await;

    // First register a key
    let register_request = PublicKeyRegistrationRequest {
        client_id: Some(client_id.to_string()),
        user_id: Some("test-user".to_string()),
        public_key: public_key.clone(),
        key_name: Some("Status Test Key".to_string()),
        metadata: Some(std::collections::HashMap::from([
            ("test".to_string(), "true".to_string()),
        ])),
    };

    let register_req = test::TestRequest::post()
        .uri("/register")
        .set_json(&register_request)
        .to_request();

    let register_resp = test::call_service(&app, register_req).await;
    assert_eq!(register_resp.status(), 201);

    // Then get the status
    let status_req = test::TestRequest::get()
        .uri(&format!("/status/{}", client_id))
        .to_request();

    let status_resp = test::call_service(&app, status_req).await;
    assert_eq!(status_resp.status(), 200);

    let body: ApiResponse<PublicKeyStatusResponse> = test::read_body_json(status_resp).await;
    assert!(body.success);
    
    let data = body.data.unwrap();
    assert_eq!(data.client_id, client_id);
    assert_eq!(data.public_key, public_key);
    assert_eq!(data.key_name, Some("Status Test Key".to_string()));
    assert_eq!(data.status, "active");
    assert!(data.last_used.is_none());
    assert!(!data.registration_id.is_empty());
}

#[tokio::test]
async fn test_get_public_key_status_not_found() {
    let fixture = PublicKeyRegistrationTestFixture::new().await;

    let app = test::init_service(
        App::new()
            .app_data(fixture.app_state.clone())
            .route("/status/{client_id}", web::get().to(get_public_key_status))
    ).await;

    let req = test::TestRequest::get()
        .uri("/status/nonexistent-client")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404); // Not Found

    let body: ApiResponse<()> = test::read_body_json(resp).await;
    assert!(!body.success);
    assert_eq!(body.error.unwrap().code, "CLIENT_NOT_FOUND");
}

#[tokio::test]
async fn test_invalid_ed25519_public_key() {
    let fixture = PublicKeyRegistrationTestFixture::new().await;

    // Generate 32 bytes of invalid Ed25519 public key data
    // (all zeros is not a valid Ed25519 public key)
    let invalid_key = "0000000000000000000000000000000000000000000000000000000000000000";

    let request = PublicKeyRegistrationRequest {
        client_id: Some("invalid-key-client".to_string()),
        user_id: None,
        public_key: invalid_key.to_string(),
        key_name: None,
        metadata: None,
    };

    let app = test::init_service(
        App::new()
            .app_data(fixture.app_state.clone())
            .route("/register", web::post().to(register_public_key))
    ).await;

    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let body: ApiResponse<()> = test::read_body_json(resp).await;
    assert!(!body.success);
    assert_eq!(body.error.unwrap().code, "INVALID_PUBLIC_KEY");
}