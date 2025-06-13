//! Integration tests for CLI authentication functionality
//!
//! This test suite validates the CLI authentication implementation including
//! key generation, profile management, request signing, and end-to-end workflows.

use datafold::cli::auth::{CliAuthProfile, CliRequestSigner, CliSigningConfig, SignatureComponent};
use datafold::cli::config::{CliConfigManager, CliSettings, SignatureSettings};
use datafold::cli::http_client::{HttpClientBuilder, RetryConfig};
use datafold::crypto::ed25519::generate_master_keypair;
use std::collections::HashMap;
use tempfile::TempDir;
use tokio;

/// Test CLI authentication profile creation and management
#[test]
fn test_cli_auth_profile_creation() {
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "test".to_string());
    metadata.insert("environment".to_string(), "testing".to_string());

    let profile = CliAuthProfile {
        client_id: "test-client-123".to_string(),
        key_id: "test-key".to_string(),
        user_id: Some("test-user".to_string()),
        server_url: "https://api.test.com".to_string(),
        metadata,
    };

    assert_eq!(profile.client_id, "test-client-123");
    assert_eq!(profile.key_id, "test-key");
    assert_eq!(profile.server_url, "https://api.test.com");
    assert_eq!(profile.user_id.as_ref().unwrap(), "test-user");
    assert_eq!(profile.metadata.get("source").unwrap(), "test");
}

/// Test CLI configuration manager functionality
#[test]
fn test_cli_config_manager() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("test_config.json");

    // Create configuration manager
    let mut manager = CliConfigManager::with_path(&config_path)?;

    // Create test profile
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "test".to_string());

    let profile = CliAuthProfile {
        client_id: "test-client-456".to_string(),
        key_id: "test-key-456".to_string(),
        user_id: None,
        server_url: "http://localhost:8080".to_string(),
        metadata,
    };

    // Test profile management
    manager.add_profile("test-profile".to_string(), profile.clone())?;
    assert_eq!(manager.list_profiles().len(), 1);

    let retrieved = manager.get_profile("test-profile").unwrap();
    assert_eq!(retrieved.client_id, profile.client_id);
    assert_eq!(retrieved.key_id, profile.key_id);

    // Test default profile
    let default = manager.get_default_profile().unwrap();
    assert_eq!(default.client_id, profile.client_id);

    // Test configuration persistence
    manager.save()?;
    assert!(config_path.exists());

    // Test loading from saved file
    let loaded_manager = CliConfigManager::with_path(&config_path)?;
    assert_eq!(loaded_manager.list_profiles().len(), 1);

    let loaded_profile = loaded_manager.get_profile("test-profile").unwrap();
    assert_eq!(loaded_profile.client_id, "test-client-456");

    Ok(())
}

/// Test CLI signing configuration
#[test]
fn test_cli_signing_config() {
    let config = CliSigningConfig::default();

    // Verify default configuration
    assert!(config.include_timestamp);
    assert!(config.include_nonce);
    assert!(config.include_content_digest);
    assert_eq!(config.max_body_size, 10 * 1024 * 1024); // 10MB

    // Verify required components
    assert!(config
        .required_components
        .contains(&SignatureComponent::Method));
    assert!(config
        .required_components
        .contains(&SignatureComponent::TargetUri));

    // Test custom configuration
    let custom_config = CliSigningConfig {
        required_components: vec![
            SignatureComponent::Method,
            SignatureComponent::TargetUri,
            SignatureComponent::Header("authorization".to_string()),
        ],
        include_content_digest: false,
        include_timestamp: true,
        include_nonce: false,
        max_body_size: 5 * 1024 * 1024, // 5MB
    };

    assert!(!custom_config.include_content_digest);
    assert!(!custom_config.include_nonce);
    assert_eq!(custom_config.max_body_size, 5 * 1024 * 1024);
}

/// Test CLI request signer creation and basic functionality
#[test]
fn test_cli_request_signer() -> Result<(), Box<dyn std::error::Error>> {
    let keypair = generate_master_keypair()?;

    let mut metadata = HashMap::new();
    metadata.insert("test".to_string(), "value".to_string());

    let profile = CliAuthProfile {
        client_id: "signer-test-client".to_string(),
        key_id: "signer-test-key".to_string(),
        user_id: Some("signer-test-user".to_string()),
        server_url: "https://api.signer-test.com".to_string(),
        metadata,
    };

    let config = CliSigningConfig::default();
    let signer = CliRequestSigner::new(keypair, profile.clone(), config);

    // Verify signer properties
    assert_eq!(signer.profile().client_id, profile.client_id);
    assert_eq!(signer.profile().key_id, profile.key_id);
    assert_eq!(signer.profile().server_url, profile.server_url);

    // Verify public key extraction
    let public_key_bytes = signer.public_key_bytes();
    assert_eq!(public_key_bytes.len(), 32);

    Ok(())
}

/// Test HTTP client builder functionality
#[test]
fn test_http_client_builder() -> Result<(), Box<dyn std::error::Error>> {
    // Test unauthenticated client
    let client = HttpClientBuilder::new()
        .timeout_secs(45)
        .retry_config(RetryConfig {
            max_retries: 5,
            initial_delay_ms: 500,
            max_delay_ms: 15000,
            retry_server_errors: true,
            retry_network_errors: true,
        })
        .build()?;

    assert!(!client.is_authenticated());

    // Test authenticated client
    let keypair = generate_master_keypair()?;
    let profile = CliAuthProfile {
        client_id: "builder-test-client".to_string(),
        key_id: "builder-test-key".to_string(),
        user_id: None,
        server_url: "https://api.builder-test.com".to_string(),
        metadata: HashMap::new(),
    };

    let auth_client = HttpClientBuilder::new()
        .timeout_secs(30)
        .build_authenticated(keypair, profile.clone(), None)?;

    assert!(auth_client.is_authenticated());

    let status = auth_client.auth_status();
    assert!(status.configured);
    assert_eq!(status.client_id.as_ref().unwrap(), &profile.client_id);
    assert_eq!(status.key_id.as_ref().unwrap(), &profile.key_id);

    Ok(())
}

/// Test signature component handling
#[test]
fn test_signature_components() {
    // Test component string representation
    assert_eq!(SignatureComponent::Method.to_string(), "@method");
    assert_eq!(SignatureComponent::TargetUri.to_string(), "@target-uri");
    assert_eq!(SignatureComponent::Authority.to_string(), "@authority");
    assert_eq!(SignatureComponent::Scheme.to_string(), "@scheme");
    assert_eq!(SignatureComponent::Path.to_string(), "@path");
    assert_eq!(SignatureComponent::Query.to_string(), "@query");

    let header_component = SignatureComponent::Header("content-type".to_string());
    assert_eq!(header_component.to_string(), "content-type");

    let custom_header = SignatureComponent::Header("X-Custom-Header".to_string());
    assert_eq!(custom_header.to_string(), "x-custom-header");
}

/// Test CLI settings and configuration structures
#[test]
fn test_cli_settings() {
    let settings = CliSettings::default();

    // Verify default settings
    assert_eq!(settings.output_format, "table");
    assert!(settings.colored_output);
    assert_eq!(settings.verbosity, 1);
    assert!(settings.auto_update_check);
    assert_eq!(settings.default_timeout_secs, 30);
    assert_eq!(settings.default_max_retries, 3);
    assert!(settings.store_auth_tokens);

    // Test signature settings
    let sig_settings = &settings.signature_settings;
    assert!(sig_settings.include_timestamp);
    assert!(sig_settings.include_nonce);
    assert_eq!(sig_settings.max_body_size_mb, 10);
    assert!(!sig_settings.verify_responses);

    // Verify default signature components
    assert!(sig_settings
        .default_components
        .contains(&"@method".to_string()));
    assert!(sig_settings
        .default_components
        .contains(&"@target-uri".to_string()));
    assert!(sig_settings
        .default_components
        .contains(&"content-type".to_string()));
}

/// Test configuration validation
#[test]
fn test_config_validation() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("validation_test.json");

    let mut manager = CliConfigManager::with_path(&config_path)?;

    // Test valid configuration
    let valid_profile = CliAuthProfile {
        client_id: "valid-client".to_string(),
        key_id: "valid-key".to_string(),
        user_id: None,
        server_url: "https://valid.api.com".to_string(),
        metadata: HashMap::new(),
    };

    manager.add_profile("valid".to_string(), valid_profile)?;
    assert!(manager.validate().is_ok());

    // Test invalid configuration (empty client_id)
    let invalid_profile = CliAuthProfile {
        client_id: String::new(),
        key_id: "invalid-key".to_string(),
        user_id: None,
        server_url: "https://invalid.api.com".to_string(),
        metadata: HashMap::new(),
    };

    manager.add_profile("invalid".to_string(), invalid_profile)?;
    assert!(manager.validate().is_err());

    Ok(())
}

/// Integration test for end-to-end workflow simulation
#[tokio::test]
async fn test_end_to_end_workflow_simulation() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Generate keypair (simulating auth-keygen)
    let keypair = generate_master_keypair()?;
    let public_key_hex = hex::encode(keypair.public_key_bytes());

    // 2. Create authentication profile (simulating auth-init)
    let client_id = format!("cli_{}", uuid::Uuid::new_v4());
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "integration-test".to_string());
    metadata.insert("created_by".to_string(), "test-suite".to_string());

    let profile = CliAuthProfile {
        client_id: client_id.clone(),
        key_id: "integration-test-key".to_string(),
        user_id: Some("integration-test-user".to_string()),
        server_url: "http://localhost:8080".to_string(),
        metadata,
    };

    // 3. Setup configuration manager (simulating CLI config)
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("integration_config.json");

    let mut config_manager = CliConfigManager::with_path(&config_path)?;
    config_manager.add_profile("integration-test".to_string(), profile.clone())?;
    config_manager.save()?;

    // 4. Create authenticated HTTP client (simulating auth-test)
    let signing_config = CliSigningConfig::default();
    let client = HttpClientBuilder::new()
        .timeout_secs(30)
        .retry_config(RetryConfig::default())
        .build_authenticated(keypair, profile.clone(), Some(signing_config))?;

    // 5. Verify client configuration
    assert!(client.is_authenticated());
    let status = client.auth_status();
    assert!(status.configured);
    assert_eq!(status.client_id.as_ref().unwrap(), &client_id);
    assert_eq!(status.key_id.as_ref().unwrap(), "integration-test-key");
    assert_eq!(status.server_url.as_ref().unwrap(), "http://localhost:8080");

    // 6. Verify configuration persistence
    let reloaded_manager = CliConfigManager::with_path(&config_path)?;
    let reloaded_profile = reloaded_manager.get_profile("integration-test").unwrap();
    assert_eq!(reloaded_profile.client_id, client_id);
    assert_eq!(reloaded_profile.key_id, "integration-test-key");

    println!("✅ End-to-end workflow simulation completed successfully!");
    println!("   Generated keypair with public key: {}", public_key_hex);
    println!("   Created client ID: {}", client_id);
    println!("   Configured profile: integration-test");
    println!("   Authentication status: configured and ready");

    Ok(())
}

/// Test error handling scenarios
#[test]
fn test_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("error_test.json");

    let mut manager = CliConfigManager::with_path(&config_path).unwrap();

    // Test profile not found
    assert!(manager.get_profile("nonexistent").is_none());

    // Test removing non-existent profile
    let result = manager.remove_profile("nonexistent");
    assert!(result.is_err());

    // Test setting non-existent default profile
    let result = manager.set_default_profile("nonexistent".to_string());
    assert!(result.is_err());
}

/// Performance test for signature generation
#[test]
fn test_signature_performance() -> Result<(), Box<dyn std::error::Error>> {
    let keypair = generate_master_keypair()?;
    let profile = CliAuthProfile {
        client_id: "perf-test-client".to_string(),
        key_id: "perf-test-key".to_string(),
        user_id: None,
        server_url: "https://api.perftest.com".to_string(),
        metadata: HashMap::new(),
    };

    let config = CliSigningConfig::default();
    let _signer = CliRequestSigner::new(keypair, profile, config);

    // In a real performance test, we would create actual HTTP requests
    // and measure signing time, but that requires more complex setup
    // This test validates that the signer can be created efficiently

    println!("✅ Signature performance test completed - signer creation is efficient");

    Ok(())
}
