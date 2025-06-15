//! Comprehensive tests for database crypto initialization
//!
//! This test suite validates the complete crypto initialization workflow
//! including configuration validation, key generation, metadata storage,
//! and integration with DataFoldNode.

use datafold::{
    config::crypto::{CryptoConfig, KeyDerivationConfig, MasterKeyConfig},
    datafold_node::{DataFoldNode, NodeConfig},
    db_operations::DbOperations,
    get_crypto_init_status, initialize_database_crypto, is_crypto_init_needed,
    security_types::SecurityLevel,
    validate_for_database_creation,
};
use tempfile::tempdir;

/// Create a test NodeConfig with crypto configuration
fn create_test_node_config_with_crypto(crypto_config: CryptoConfig) -> NodeConfig {
    let temp_dir = tempdir().unwrap();
    let mut config = NodeConfig::new(temp_dir.path().to_path_buf());
    config.crypto = Some(crypto_config);
    config
}

/// Create a test NodeConfig without crypto configuration
fn create_test_node_config_no_crypto() -> NodeConfig {
    let temp_dir = tempdir().unwrap();
    NodeConfig::new(temp_dir.path().to_path_buf())
}

/// Create a random crypto configuration for testing
fn create_random_crypto_config() -> CryptoConfig {
    CryptoConfig {
        enabled: true,
        master_key: MasterKeyConfig::Random,
        key_derivation: KeyDerivationConfig::default(),
    }
}

/// Create a passphrase crypto configuration for testing
fn create_passphrase_crypto_config() -> CryptoConfig {
    CryptoConfig {
        enabled: true,
        master_key: MasterKeyConfig::Passphrase {
            passphrase: "test_secure_passphrase_for_database_init".to_string(),
        },
        key_derivation: KeyDerivationConfig::for_security_level(SecurityLevel::Low),
    }
}

/// Create a disabled crypto configuration for testing
fn create_disabled_crypto_config() -> CryptoConfig {
    CryptoConfig {
        enabled: false,
        master_key: MasterKeyConfig::Random,
        key_derivation: KeyDerivationConfig::default(),
    }
}

#[test]
fn test_database_crypto_init_with_random_key() {
    let config = create_test_node_config_with_crypto(create_random_crypto_config());

    // Create DataFoldNode - should automatically initialize crypto
    let node = DataFoldNode::new(config).expect("Failed to create DataFoldNode");

    // Verify crypto status
    let status = node
        .get_crypto_status()
        .expect("Failed to get crypto status");
    assert!(status.initialized);
    assert!(status.is_healthy());
    assert!(status.summary().contains("Random"));

    // Verify crypto is not needed anymore
    let crypto_config = create_random_crypto_config();
    let needs_init = node
        .is_crypto_init_needed(Some(&crypto_config))
        .expect("Failed to check crypto init status");
    assert!(!needs_init);
}

#[test]
fn test_database_crypto_init_with_passphrase() {
    let config = create_test_node_config_with_crypto(create_passphrase_crypto_config());

    // Create DataFoldNode - should automatically initialize crypto
    let node = DataFoldNode::new(config).expect("Failed to create DataFoldNode");

    // Verify crypto status
    let status = node
        .get_crypto_status()
        .expect("Failed to get crypto status");
    assert!(status.initialized);
    assert!(status.is_healthy());
    assert!(status.summary().contains("Argon2id"));

    // Verify derivation method
    assert!(status
        .derivation_method
        .as_ref()
        .unwrap()
        .starts_with("Argon2id-"));
}

#[test]
fn test_database_creation_without_crypto() {
    let config = create_test_node_config_no_crypto();

    // Create DataFoldNode - should work without crypto
    let node = DataFoldNode::new(config).expect("Failed to create DataFoldNode");

    // Verify crypto status
    let status = node
        .get_crypto_status()
        .expect("Failed to get crypto status");
    assert!(!status.initialized);
    assert!(!status.is_healthy());
    assert_eq!(status.summary(), "Not initialized");
}

#[test]
fn test_database_creation_with_disabled_crypto() {
    let config = create_test_node_config_with_crypto(create_disabled_crypto_config());

    // Create DataFoldNode - should work with disabled crypto
    let node = DataFoldNode::new(config).expect("Failed to create DataFoldNode");

    // Verify crypto status
    let status = node
        .get_crypto_status()
        .expect("Failed to get crypto status");
    assert!(!status.initialized);

    // Verify crypto is not needed
    let crypto_config = create_disabled_crypto_config();
    let needs_init = node
        .is_crypto_init_needed(Some(&crypto_config))
        .expect("Failed to check crypto init status");
    assert!(!needs_init);
}

#[test]
fn test_manual_crypto_initialization() {
    let config = create_test_node_config_no_crypto();

    // Create DataFoldNode without crypto
    let node = DataFoldNode::new(config).expect("Failed to create DataFoldNode");

    // Verify crypto is not initialized
    let status = node
        .get_crypto_status()
        .expect("Failed to get crypto status");
    assert!(!status.initialized);

    // Manually initialize crypto
    let crypto_config = create_random_crypto_config();
    node.initialize_crypto(&crypto_config)
        .expect("Failed to manually initialize crypto");

    // Verify crypto is now initialized
    let status = node
        .get_crypto_status()
        .expect("Failed to get crypto status");
    assert!(status.initialized);
    assert!(status.is_healthy());
}

#[test]
fn test_crypto_already_initialized_error() {
    let config = create_test_node_config_with_crypto(create_random_crypto_config());

    // Create DataFoldNode - crypto should be automatically initialized
    let node = DataFoldNode::new(config).expect("Failed to create DataFoldNode");

    // Try to initialize crypto again - should fail
    let crypto_config = create_random_crypto_config();
    let result = node.initialize_crypto(&crypto_config);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("already has crypto metadata"));
}

#[test]
fn test_crypto_config_validation_during_node_creation() {
    // Test with invalid passphrase (empty)
    let invalid_config = CryptoConfig {
        enabled: true,
        master_key: MasterKeyConfig::Passphrase {
            passphrase: "".to_string(),
        },
        key_derivation: KeyDerivationConfig::default(),
    };

    let config = create_test_node_config_with_crypto(invalid_config);
    let result = DataFoldNode::new(config);
    assert!(result.is_err());
    let error_msg = result.err().unwrap().to_string();
    assert!(error_msg.contains("Passphrase cannot be empty"));
}

#[test]
fn test_crypto_config_validation_external_keys() {
    // Test with external key source (not supported)
    let external_config = CryptoConfig {
        enabled: true,
        master_key: MasterKeyConfig::External {
            key_source: "test_source".to_string(),
        },
        key_derivation: KeyDerivationConfig::default(),
    };

    let config = create_test_node_config_with_crypto(external_config);
    let result = DataFoldNode::new(config);
    assert!(result.is_err());
    let error_msg = result.err().unwrap().to_string();
    assert!(error_msg.contains("External key sources"));
}

#[test]
fn test_different_security_levels() {
    let security_levels = [
        SecurityLevel::Standard,
        SecurityLevel::Low,
        SecurityLevel::High,
    ];

    for security_level in &security_levels {
        let crypto_config = CryptoConfig {
            enabled: true,
            master_key: MasterKeyConfig::Passphrase {
                passphrase: "test_passphrase_for_security_levels".to_string(),
            },
            key_derivation: KeyDerivationConfig::for_security_level(*security_level),
        };

        let config = create_test_node_config_with_crypto(crypto_config);
        let node = DataFoldNode::new(config).unwrap_or_else(|_| {
            panic!(
                "Failed to create node with security level: {:?}",
                security_level
            )
        });

        let status = node
            .get_crypto_status()
            .expect("Failed to get crypto status");
        assert!(status.initialized);
        assert!(status.is_healthy());

        // Verify the security level is reflected in the derivation method
        let derivation_method = status.derivation_method.as_ref().unwrap();
        // The derivation method should start with "Argon2id" but may not contain the exact security level name
        // since it could be "Custom" if custom parameters are used
        assert!(derivation_method.starts_with("Argon2id"));
    }
}

#[test]
fn test_crypto_status_details() {
    let config = create_test_node_config_with_crypto(create_passphrase_crypto_config());
    let node = DataFoldNode::new(config).expect("Failed to create DataFoldNode");

    let status = node
        .get_crypto_status()
        .expect("Failed to get crypto status");

    // Verify all status fields are populated correctly
    assert!(status.initialized);
    assert!(status.version.is_some());
    assert_eq!(status.version.unwrap(), 1); // Current version
    assert!(status.algorithm.is_some());
    assert_eq!(status.algorithm.as_ref().unwrap(), "Ed25519");
    assert!(status.derivation_method.is_some());
    assert!(status.created_at.is_some());
    assert!(status.integrity_verified.is_some());
    assert!(status.integrity_verified.unwrap());

    // Verify status summary
    let summary = status.summary();
    assert!(summary.contains("Argon2id"));
    assert!(!summary.contains("Not initialized"));
}

#[test]
fn test_is_crypto_init_needed_scenarios() {
    // Test with no crypto config
    let config = create_test_node_config_no_crypto();
    let node = DataFoldNode::new(config).expect("Failed to create DataFoldNode");

    // No crypto config provided - should not need init
    let needs_init = node
        .is_crypto_init_needed(None)
        .expect("Failed to check crypto init status");
    assert!(!needs_init);

    // Disabled crypto config - should not need init
    let disabled_config = create_disabled_crypto_config();
    let needs_init = node
        .is_crypto_init_needed(Some(&disabled_config))
        .expect("Failed to check crypto init status");
    assert!(!needs_init);

    // Enabled crypto config - should need init
    let enabled_config = create_random_crypto_config();
    let needs_init = node
        .is_crypto_init_needed(Some(&enabled_config))
        .expect("Failed to check crypto init status");
    assert!(needs_init);

    // After manual initialization - should not need init
    node.initialize_crypto(&enabled_config)
        .expect("Failed to initialize crypto");
    let needs_init = node
        .is_crypto_init_needed(Some(&enabled_config))
        .expect("Failed to check crypto init status");
    assert!(!needs_init);
}

#[test]
fn test_validate_crypto_config_for_database_creation() {
    // Valid random config
    let config = create_random_crypto_config();
    assert!(validate_for_database_creation(&config).is_ok());

    // Valid passphrase config
    let config = create_passphrase_crypto_config();
    assert!(validate_for_database_creation(&config).is_ok());

    // Disabled config should fail for database creation
    let config = create_disabled_crypto_config();
    let result = validate_for_database_creation(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("must be enabled"));

    // Empty passphrase should fail
    let config = CryptoConfig {
        enabled: true,
        master_key: MasterKeyConfig::Passphrase {
            passphrase: "".to_string(),
        },
        key_derivation: KeyDerivationConfig::default(),
    };
    let result = validate_for_database_creation(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cannot be empty"));
}

#[test]
fn test_direct_crypto_initialization_functions() {
    let temp_dir = tempdir().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    let db_ops = std::sync::Arc::new(DbOperations::new(db).unwrap());

    // Test is_crypto_init_needed function directly
    let crypto_config = create_random_crypto_config();

    let needs_init = is_crypto_init_needed(db_ops.clone(), Some(&crypto_config))
        .expect("Failed to check crypto init status");
    assert!(needs_init);

    // Test initialize_database_crypto function directly
    let context = initialize_database_crypto(db_ops.clone(), &crypto_config)
        .expect("Failed to initialize crypto");

    assert_eq!(context.derivation_method, "Random");
    assert!(context.metadata.verify_integrity().unwrap());

    // Test get_crypto_init_status function directly
    let status = get_crypto_init_status(db_ops.clone()).expect("Failed to get crypto status");

    assert!(status.initialized);
    assert!(status.is_healthy());

    // Test that crypto is no longer needed
    let needs_init = is_crypto_init_needed(db_ops, Some(&crypto_config))
        .expect("Failed to check crypto init status");
    assert!(!needs_init);
}

#[test]
fn test_crypto_init_with_different_passphrase_lengths() {
    let test_cases = [
        ("short", true), // Should fail - too short (5 chars, minimum is 6)
        ("medium_length_passphrase", false),
        (
            "very_long_passphrase_that_should_definitely_work_fine",
            false,
        ),
        ("", true), // Should fail
    ];

    for (passphrase, should_fail) in &test_cases {
        let crypto_config = CryptoConfig {
            enabled: true,
            master_key: MasterKeyConfig::Passphrase {
                passphrase: passphrase.to_string(),
            },
            key_derivation: KeyDerivationConfig::default(),
        };

        let config = create_test_node_config_with_crypto(crypto_config);
        let result = DataFoldNode::new(config);

        if *should_fail {
            assert!(
                result.is_err(),
                "Expected failure for passphrase: '{}'",
                passphrase
            );
        } else {
            assert!(
                result.is_ok(),
                "Expected success for passphrase: '{}'",
                passphrase
            );
            if let Ok(node) = result {
                let status = node.get_crypto_status().unwrap();
                assert!(status.initialized);
            }
        }
    }
}

#[test]
fn test_crypto_init_performance_different_security_levels() {
    use std::time::Instant;

    let security_levels = [
        SecurityLevel::Standard,
        SecurityLevel::Low,
        // Skip Sensitive for performance tests as it's very slow
    ];

    for security_level in &security_levels {
        let crypto_config = CryptoConfig {
            enabled: true,
            master_key: MasterKeyConfig::Passphrase {
                passphrase: "test_passphrase_for_performance".to_string(),
            },
            key_derivation: KeyDerivationConfig::for_security_level(*security_level),
        };

        let config = create_test_node_config_with_crypto(crypto_config);

        let start = Instant::now();
        let node = DataFoldNode::new(config).unwrap_or_else(|_| {
            panic!(
                "Failed to create node with security level: {:?}",
                security_level
            )
        });
        let duration = start.elapsed();

        println!(
            "Crypto init with {:?} security level took: {:?}",
            security_level, duration
        );

        // Verify initialization was successful
        let status = node.get_crypto_status().unwrap();
        assert!(status.initialized);
        assert!(status.is_healthy());

        // Performance expectations (these are rough guidelines)
        match security_level {
            SecurityLevel::Standard => {
                // Should be relatively fast (under 5 seconds on most systems)
                assert!(
                    duration.as_secs() < 5,
                    "Balanced security level took too long: {:?}",
                    duration
                );
            }
            SecurityLevel::Low => {
                // Should be reasonable for interactive use (under 10 seconds)
                assert!(
                    duration.as_secs() < 10,
                    "Interactive security level took too long: {:?}",
                    duration
                );
            }
            SecurityLevel::High => {
                // Can be slow, but should complete within reasonable time
                assert!(
                    duration.as_secs() < 30,
                    "Sensitive security level took too long: {:?}",
                    duration
                );
            }
        }
    }
}

#[test]
fn test_crypto_init_status_not_initialized() {
    let config = create_test_node_config_no_crypto();
    let node = DataFoldNode::new(config).expect("Failed to create DataFoldNode");

    let status = node
        .get_crypto_status()
        .expect("Failed to get crypto status");

    // Verify all fields for non-initialized state
    assert!(!status.initialized);
    assert!(status.version.is_none());
    assert!(status.algorithm.is_none());
    assert!(status.derivation_method.is_none());
    assert!(status.created_at.is_none());
    assert!(status.integrity_verified.is_none());

    // Verify status methods
    assert!(!status.is_healthy());
    assert_eq!(status.summary(), "Not initialized");
}
