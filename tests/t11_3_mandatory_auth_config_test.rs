//! T11.3: Mandatory Signature Auth Configuration Tests
//!
//! This test module verifies that signature authentication is now a mandatory
//! component of node configuration and cannot be omitted.

use datafold::datafold_node::config::NodeConfig;
use datafold::datafold_node::signature_auth::{SecurityProfile, SignatureAuthConfig};
use datafold::datafold_node::DataFoldNode;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_node_config_default_includes_signature_auth() {
    // Test that default NodeConfig always includes signature auth
    let config = NodeConfig::default();

    // Signature auth should always be enabled (never None)
    assert!(config.is_signature_auth_enabled());

    // Should have a valid signature auth configuration
    let sig_config = config.signature_auth_config();
    assert_eq!(sig_config.security_profile, SecurityProfile::Standard);
    assert!(sig_config.allowed_time_window_secs > 0);
}

#[test]
fn test_node_config_new_includes_signature_auth() {
    // Test that NodeConfig::new() always includes signature auth
    let temp_dir = tempdir().unwrap();
    let config = NodeConfig::new(temp_dir.path().to_path_buf());

    // Signature auth should always be enabled
    assert!(config.is_signature_auth_enabled());

    // Should have default signature auth configuration
    let sig_config = config.signature_auth_config();
    assert_eq!(sig_config.security_profile, SecurityProfile::Standard);
}

#[test]
fn test_node_config_development_profile() {
    // Test development configuration includes lenient signature auth
    let temp_dir = tempdir().unwrap();
    let config = NodeConfig::development(temp_dir.path().to_path_buf());

    // Should always be enabled
    assert!(config.is_signature_auth_enabled());

    // Should have lenient security profile for development
    let sig_config = config.signature_auth_config();
    assert_eq!(sig_config.security_profile, SecurityProfile::Lenient);
    assert!(sig_config.allowed_time_window_secs >= 600); // More lenient time window
}

#[test]
fn test_node_config_production_profile() {
    // Test production configuration includes strict signature auth
    let temp_dir = tempdir().unwrap();
    let config = NodeConfig::production(temp_dir.path().to_path_buf());

    // Should always be enabled
    assert!(config.is_signature_auth_enabled());

    // Should have strict security profile for production
    let sig_config = config.signature_auth_config();
    assert_eq!(sig_config.security_profile, SecurityProfile::Strict);
    assert!(sig_config.allowed_time_window_secs <= 60); // Stricter time window
}

#[test]
fn test_node_config_validation_includes_signature_auth() {
    // Test that configuration validation includes signature auth validation
    let temp_dir = tempdir().unwrap();
    let mut config = NodeConfig::new(temp_dir.path().to_path_buf());

    // Valid configuration should pass validation
    assert!(config.validate().is_ok());

    // Create invalid signature auth config
    config.signature_auth.allowed_time_window_secs = 0; // Invalid: zero time window

    // Should fail validation due to invalid signature auth
    let validation_result = config.validate();
    assert!(validation_result.is_err());

    let error_msg = format!("{}", validation_result.unwrap_err());
    assert!(error_msg.contains("Signature auth validation failed"));
    assert!(error_msg.contains("Time window must be greater than 0"));
}

#[test]
fn test_node_config_with_custom_signature_auth() {
    // Test updating signature auth configuration
    let temp_dir = tempdir().unwrap();
    let custom_sig_config = SignatureAuthConfig {
        security_profile: SecurityProfile::Strict,
        allowed_time_window_secs: 120,
        ..SignatureAuthConfig::default()
    };

    let config = NodeConfig::new(temp_dir.path().to_path_buf())
        .with_signature_auth(custom_sig_config.clone());

    // Should have the custom configuration
    let sig_config = config.signature_auth_config();
    assert_eq!(sig_config.security_profile, SecurityProfile::Strict);
    assert_eq!(sig_config.allowed_time_window_secs, 120);
}

#[test]
fn test_node_creation_with_mandatory_signature_auth() {
    // Test that DataFoldNode can be created with mandatory signature auth
    let temp_dir = tempdir().unwrap();
    let config = NodeConfig::new(temp_dir.path().to_path_buf());

    // Node creation should succeed with valid signature auth config
    let node_result = DataFoldNode::new(config);
    assert!(node_result.is_ok());

    let _node = node_result.unwrap();

    // Node creation succeeded, which means signature auth is properly configured
    // (since the HTTP server initialization would fail if it wasn't)
}

#[test]
fn test_signature_auth_config_validation() {
    // Test individual signature auth configuration validation
    let mut sig_config = SignatureAuthConfig::default();

    // Valid config should pass
    assert!(sig_config.validate().is_ok());

    // Test invalid time window
    sig_config.allowed_time_window_secs = 0;
    assert!(sig_config.validate().is_err());

    // Reset and test invalid nonce TTL
    sig_config = SignatureAuthConfig::default();
    sig_config.nonce_ttl_secs = 0;
    assert!(sig_config.validate().is_err());

    // Reset and test invalid nonce store size
    sig_config = SignatureAuthConfig::default();
    sig_config.max_nonce_store_size = 0;
    assert!(sig_config.validate().is_err());

    // Reset and test invalid clock skew vs time window
    sig_config = SignatureAuthConfig::default();
    sig_config.clock_skew_tolerance_secs = sig_config.allowed_time_window_secs + 1;
    assert!(sig_config.validate().is_err());
}

#[test]
fn test_configuration_backward_compatibility() {
    // Test that old configuration loading patterns still work
    // but now always include mandatory signature auth

    let temp_dir = tempdir().unwrap();

    // Test with_crypto method includes signature auth
    let crypto_config = datafold::config::crypto::CryptoConfig::default();
    let config = NodeConfig::with_crypto(temp_dir.path().to_path_buf(), crypto_config);

    // Should include both crypto and signature auth
    assert!(config.crypto.is_some());
    assert!(config.is_signature_auth_enabled());
}

#[test]
fn test_security_profiles_have_different_settings() {
    // Verify that different security profiles have appropriate settings

    let strict_config = SignatureAuthConfig::strict();
    let standard_config = SignatureAuthConfig::default();
    let lenient_config = SignatureAuthConfig::lenient();

    // Verify security profiles are different
    assert_eq!(strict_config.security_profile, SecurityProfile::Strict);
    assert_eq!(standard_config.security_profile, SecurityProfile::Standard);
    assert_eq!(lenient_config.security_profile, SecurityProfile::Lenient);

    // Verify strict is more restrictive than standard
    assert!(strict_config.allowed_time_window_secs <= standard_config.allowed_time_window_secs);
    assert!(strict_config.clock_skew_tolerance_secs <= standard_config.clock_skew_tolerance_secs);

    // Verify lenient is more permissive than standard
    assert!(lenient_config.allowed_time_window_secs >= standard_config.allowed_time_window_secs);
    assert!(lenient_config.clock_skew_tolerance_secs >= standard_config.clock_skew_tolerance_secs);

    // Verify rate limiting and attack detection settings vary appropriately
    assert!(strict_config.rate_limiting.enabled);
    assert!(strict_config.attack_detection.enabled);
    assert!(!lenient_config.rate_limiting.enabled); // Disabled in lenient mode
    assert!(!lenient_config.attack_detection.enabled); // Disabled in lenient mode
}

#[test]
fn test_node_config_serialization_includes_signature_auth() {
    // Test that serialized configuration includes mandatory signature auth
    let temp_dir = tempdir().unwrap();
    let config = NodeConfig::new(temp_dir.path().to_path_buf());

    // Serialize the configuration
    let serialized = serde_json::to_string(&config).expect("Failed to serialize config");

    // Should include signature_auth field
    assert!(serialized.contains("signature_auth"));
    assert!(serialized.contains("security_profile"));

    // Deserialize should work and maintain signature auth
    let deserialized: NodeConfig =
        serde_json::from_str(&serialized).expect("Failed to deserialize config");

    assert!(deserialized.is_signature_auth_enabled());
    assert_eq!(
        deserialized.signature_auth_config().security_profile,
        config.signature_auth_config().security_profile
    );
}
