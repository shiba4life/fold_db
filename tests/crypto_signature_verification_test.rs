//! Integration tests for digital signature verification endpoint
//!
//! This module tests the POST /api/crypto/signatures/verify endpoint
//! which verifies Ed25519 digital signatures from registered clients.

use actix_web::{test, web, App};
use datafold::crypto::{generate_master_keypair, MasterKeyPair};
use datafold::datafold_node::{
    config::NodeConfig, crypto_routes, http_server::AppState, DataFoldNode,
};
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

/// Helper to register a public key and return the client_id and keypair
async fn register_test_key() -> (String, MasterKeyPair, Arc<tokio::sync::Mutex<DataFoldNode>>) {
    // Create test app
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config = NodeConfig::new(temp_dir.path().to_path_buf());
    let node = DataFoldNode::new(config).expect("Failed to create DataFold node");
    let node_arc = Arc::new(tokio::sync::Mutex::new(node));

    // Create default signature auth for testing
    let sig_config = datafold::datafold_node::signature_auth::SignatureAuthConfig::default();
    let signature_auth =
        datafold::datafold_node::signature_auth::SignatureVerificationState::new(sig_config)
            .expect("Failed to create signature verification state for test");

    let app_state = web::Data::new(AppState {
        signature_auth: Arc::new(signature_auth),
        node: node_arc.clone(),
    });

    let app = test::init_service(
        App::new().app_data(app_state).service(
            web::scope("/api/crypto")
                .route(
                    "/keys/register",
                    web::post().to(crypto_routes::register_public_key),
                )
                .route(
                    "/signatures/verify",
                    web::post().to(crypto_routes::verify_signature),
                ),
        ),
    )
    .await;

    // Generate a test keypair
    let keypair = generate_master_keypair().expect("Failed to generate test keypair");
    let public_key_hex = hex::encode(keypair.public_key_bytes());

    let client_id = "test_client_001".to_string();

    // Register the public key
    let registration_request = json!({
        "client_id": client_id,
        "public_key": public_key_hex,
        "key_name": "test_key"
    });

    let req = test::TestRequest::post()
        .uri("/api/crypto/keys/register")
        .set_json(&registration_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Failed to register public key");

    (client_id, keypair, node_arc)
}

#[tokio::test]
async fn test_signature_verification_success_utf8() {
    let (client_id, keypair, node_arc) = register_test_key().await;

    // Create default signature auth for testing
    let sig_config = datafold::datafold_node::signature_auth::SignatureAuthConfig::default();
    let signature_auth =
        datafold::datafold_node::signature_auth::SignatureVerificationState::new(sig_config)
            .expect("Failed to create signature verification state for test");

    let app_state = web::Data::new(AppState {
        signature_auth: Arc::new(signature_auth),
        node: node_arc,
    });
    let app = test::init_service(App::new().app_data(app_state).service(
        web::scope("/api/crypto").route(
            "/signatures/verify",
            web::post().to(crypto_routes::verify_signature),
        ),
    ))
    .await;

    // Sign a test message
    let message = "Hello, DataFold!";
    let signature = keypair
        .sign_data(message.as_bytes())
        .expect("Failed to sign message");
    let signature_hex = hex::encode(signature);

    // Verify the signature
    let verification_request = json!({
        "client_id": client_id,
        "message": message,
        "signature": signature_hex,
        "message_encoding": "utf8"
    });

    let req = test::TestRequest::post()
        .uri("/api/crypto/signatures/verify")
        .set_json(&verification_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["verified"], true);
    assert_eq!(body["data"]["client_id"], client_id);
}

#[tokio::test]
async fn test_signature_verification_success_hex() {
    let (client_id, keypair, node_arc) = register_test_key().await;

    // Create default signature auth for testing
    let sig_config = datafold::datafold_node::signature_auth::SignatureAuthConfig::default();
    let signature_auth =
        datafold::datafold_node::signature_auth::SignatureVerificationState::new(sig_config)
            .expect("Failed to create signature verification state for test");

    let app_state = web::Data::new(AppState {
        signature_auth: Arc::new(signature_auth),
        node: node_arc,
    });
    let app = test::init_service(App::new().app_data(app_state).service(
        web::scope("/api/crypto").route(
            "/signatures/verify",
            web::post().to(crypto_routes::verify_signature),
        ),
    ))
    .await;

    // Sign a hex-encoded message
    let message_bytes = b"DataFold hex test";
    let message_hex = hex::encode(message_bytes);
    let signature = keypair
        .sign_data(message_bytes)
        .expect("Failed to sign message");
    let signature_hex = hex::encode(signature);

    // Verify the signature
    let verification_request = json!({
        "client_id": client_id,
        "message": message_hex,
        "signature": signature_hex,
        "message_encoding": "hex"
    });

    let req = test::TestRequest::post()
        .uri("/api/crypto/signatures/verify")
        .set_json(&verification_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["verified"], true);
}

#[tokio::test]
async fn test_signature_verification_invalid_signature() {
    let (client_id, _keypair, node_arc) = register_test_key().await;

    // Create default signature auth for testing
    let sig_config = datafold::datafold_node::signature_auth::SignatureAuthConfig::default();
    let signature_auth =
        datafold::datafold_node::signature_auth::SignatureVerificationState::new(sig_config)
            .expect("Failed to create signature verification state for test");

    let app_state = web::Data::new(AppState {
        signature_auth: Arc::new(signature_auth),
        node: node_arc,
    });
    let app = test::init_service(App::new().app_data(app_state).service(
        web::scope("/api/crypto").route(
            "/signatures/verify",
            web::post().to(crypto_routes::verify_signature),
        ),
    ))
    .await;

    // Use an invalid signature
    let message = "Hello, DataFold!";
    let invalid_signature = "0".repeat(128); // 64 bytes of zeros

    let verification_request = json!({
        "client_id": client_id,
        "message": message,
        "signature": invalid_signature,
        "message_encoding": "utf8"
    });

    let req = test::TestRequest::post()
        .uri("/api/crypto/signatures/verify")
        .set_json(&verification_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401); // Unauthorized for invalid signature

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "SIGNATURE_VERIFICATION_FAILED");
}

#[tokio::test]
async fn test_signature_verification_unregistered_client() {
    // Create an app without registering any keys
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config = NodeConfig::new(temp_dir.path().to_path_buf());
    let node = DataFoldNode::new(config).expect("Failed to create DataFold node");
    let node_arc = Arc::new(tokio::sync::Mutex::new(node));

    // Create default signature auth for testing
    let sig_config = datafold::datafold_node::signature_auth::SignatureAuthConfig::default();
    let signature_auth =
        datafold::datafold_node::signature_auth::SignatureVerificationState::new(sig_config)
            .expect("Failed to create signature verification state for test");

    let app_state = web::Data::new(AppState {
        signature_auth: Arc::new(signature_auth),
        node: node_arc,
    });
    let app = test::init_service(App::new().app_data(app_state).service(
        web::scope("/api/crypto").route(
            "/signatures/verify",
            web::post().to(crypto_routes::verify_signature),
        ),
    ))
    .await;

    // Try to verify with unregistered client
    let unregistered_client = "unregistered_client".to_string();
    let message = "Hello, DataFold!";
    let fake_signature = "a".repeat(128); // Valid length but fake signature

    let verification_request = json!({
        "client_id": unregistered_client,
        "message": message,
        "signature": fake_signature,
        "message_encoding": "utf8"
    });

    let req = test::TestRequest::post()
        .uri("/api/crypto/signatures/verify")
        .set_json(&verification_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404); // Not Found for unregistered client

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "CLIENT_NOT_FOUND");
}

#[tokio::test]
async fn test_signature_verification_invalid_encoding() {
    let (client_id, keypair, node_arc) = register_test_key().await;

    // Create default signature auth for testing
    let sig_config = datafold::datafold_node::signature_auth::SignatureAuthConfig::default();
    let signature_auth =
        datafold::datafold_node::signature_auth::SignatureVerificationState::new(sig_config)
            .expect("Failed to create signature verification state for test");

    let app_state = web::Data::new(AppState {
        signature_auth: Arc::new(signature_auth),
        node: node_arc,
    });
    let app = test::init_service(App::new().app_data(app_state).service(
        web::scope("/api/crypto").route(
            "/signatures/verify",
            web::post().to(crypto_routes::verify_signature),
        ),
    ))
    .await;

    let message = "Hello, DataFold!";
    let signature = keypair
        .sign_data(message.as_bytes())
        .expect("Failed to sign message");
    let signature_hex = hex::encode(signature);

    // Use invalid encoding
    let verification_request = json!({
        "client_id": client_id,
        "message": message,
        "signature": signature_hex,
        "message_encoding": "invalid_encoding"
    });

    let req = test::TestRequest::post()
        .uri("/api/crypto/signatures/verify")
        .set_json(&verification_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400); // Bad Request for invalid encoding

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "INVALID_ENCODING");
}

#[tokio::test]
async fn test_signature_verification_empty_fields() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config = NodeConfig::new(temp_dir.path().to_path_buf());
    let node = DataFoldNode::new(config).expect("Failed to create DataFold node");
    let node_arc = Arc::new(tokio::sync::Mutex::new(node));

    // Create default signature auth for testing
    let sig_config = datafold::datafold_node::signature_auth::SignatureAuthConfig::default();
    let signature_auth =
        datafold::datafold_node::signature_auth::SignatureVerificationState::new(sig_config)
            .expect("Failed to create signature verification state for test");

    let app_state = web::Data::new(AppState {
        signature_auth: Arc::new(signature_auth),
        node: node_arc,
    });
    let app = test::init_service(App::new().app_data(app_state).service(
        web::scope("/api/crypto").route(
            "/signatures/verify",
            web::post().to(crypto_routes::verify_signature),
        ),
    ))
    .await;

    // Test empty client_id
    let verification_request = json!({
        "client_id": "",
        "message": "test",
        "signature": "a".repeat(128),
        "message_encoding": "utf8"
    });

    let req = test::TestRequest::post()
        .uri("/api/crypto/signatures/verify")
        .set_json(&verification_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"]["code"], "INVALID_CLIENT_ID");
}

#[tokio::test]
async fn test_signature_verification_response_format() {
    let (client_id, keypair, node_arc) = register_test_key().await;

    // Create default signature auth for testing
    let sig_config = datafold::datafold_node::signature_auth::SignatureAuthConfig::default();
    let signature_auth =
        datafold::datafold_node::signature_auth::SignatureVerificationState::new(sig_config)
            .expect("Failed to create signature verification state for test");

    let app_state = web::Data::new(AppState {
        signature_auth: Arc::new(signature_auth),
        node: node_arc,
    });
    let app = test::init_service(App::new().app_data(app_state).service(
        web::scope("/api/crypto").route(
            "/signatures/verify",
            web::post().to(crypto_routes::verify_signature),
        ),
    ))
    .await;

    let message = "Response format test";
    let signature = keypair
        .sign_data(message.as_bytes())
        .expect("Failed to sign message");
    let signature_hex = hex::encode(signature);

    let verification_request = json!({
        "client_id": client_id,
        "message": message,
        "signature": signature_hex,
        "message_encoding": "utf8"
    });

    let req = test::TestRequest::post()
        .uri("/api/crypto/signatures/verify")
        .set_json(&verification_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;

    // Verify response structure
    assert_eq!(body["success"], true);
    assert!(body["data"]["verified"].as_bool().unwrap());
    assert_eq!(body["data"]["client_id"], client_id);
    assert!(!body["data"]["public_key"].as_str().unwrap().is_empty());
    assert!(body["data"]["verified_at"].as_str().is_some());
    assert!(body["data"]["message_hash"].as_str().unwrap().len() == 64); // SHA256 hex
}
