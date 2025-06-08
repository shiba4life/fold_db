//! Tests for crypto initialization functionality
//!
//! This module provides comprehensive tests for the crypto initialization functions
//! that are used by the HTTP API endpoints.

use tempfile::TempDir;

use datafold::datafold_node::{
    DataFoldNode,
    config::NodeConfig,
    crypto_init::{
        initialize_database_crypto, get_crypto_init_status, validate_crypto_config_for_init,
        is_crypto_init_needed, CryptoInitError
    },
};
use datafold::config::crypto::{CryptoConfig, MasterKeyConfig, KeyDerivationConfig, SecurityLevel};

/// Test fixture for crypto initialization tests
struct CryptoInitTestFixture {
    _temp_dir: TempDir,
    node: DataFoldNode,
}

impl CryptoInitTestFixture {
    /// Create a new test fixture with a fresh database
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config = NodeConfig::new(temp_dir.path().to_path_buf());
        let node = DataFoldNode::new(config).expect("Failed to create test node");
        
        Self { 
            _temp_dir: temp_dir,
            node,
        }
    }

    /// Get database operations from the node
    fn db_ops(&self) -> std::sync::Arc<datafold::db_operations::core::DbOperations> {
        let fold_db = self.node.get_fold_db().expect("FoldDB should be available");
        fold_db.db_ops()
    }
}

#[tokio::test]
async fn test_crypto_status_not_initialized() {
    let fixture = CryptoInitTestFixture::new();
    let db_ops = fixture.db_ops();

    let status = get_crypto_init_status(db_ops)
        .expect("Failed to get crypto status");

    assert!(!status.initialized);
    assert!(status.version.is_none());
    assert!(status.algorithm.is_none());
    assert!(status.derivation_method.is_none());
    assert!(status.created_at.is_none());
    assert!(status.integrity_verified.is_none());
}

#[tokio::test]
async fn test_random_key_initialization() {
    let fixture = CryptoInitTestFixture::new();
    let db_ops = fixture.db_ops();

    let crypto_config = CryptoConfig::with_random_key();

    // Verify initialization is needed
    let needs_init = is_crypto_init_needed(db_ops.clone(), Some(&crypto_config))
        .expect("Failed to check if crypto init is needed");
    assert!(needs_init);

    // Initialize crypto
    let context = initialize_database_crypto(db_ops.clone(), &crypto_config)
        .expect("Failed to initialize crypto");

    assert_eq!(context.derivation_method, "Random");
    assert!(!hex::encode(context.public_key().to_bytes()).is_empty());

    // Verify status after initialization
    let status = get_crypto_init_status(db_ops)
        .expect("Failed to get crypto status");
    
    assert!(status.initialized);
    assert_eq!(status.version, Some(1));
    assert_eq!(status.algorithm, Some("Ed25519".to_string()));
    assert_eq!(status.derivation_method, Some("Random".to_string()));
    assert!(status.created_at.is_some());
    assert_eq!(status.integrity_verified, Some(true));
}

#[tokio::test]
async fn test_passphrase_initialization() {
    let fixture = CryptoInitTestFixture::new();
    let db_ops = fixture.db_ops();

    let crypto_config = CryptoConfig {
        enabled: true,
        master_key: MasterKeyConfig::Passphrase {
            passphrase: "test-passphrase-for-initialization".to_string(),
        },
        key_derivation: KeyDerivationConfig::for_security_level(SecurityLevel::Interactive),
    };

    // Validate configuration
    validate_crypto_config_for_init(&crypto_config)
        .expect("Config validation should pass");

    // Initialize crypto
    let context = initialize_database_crypto(db_ops.clone(), &crypto_config)
        .expect("Failed to initialize crypto");

    assert!(context.derivation_method.contains("Argon2id"));
    assert!(!hex::encode(context.public_key().to_bytes()).is_empty());

    // Verify status after initialization
    let status = get_crypto_init_status(db_ops)
        .expect("Failed to get crypto status");
    
    assert!(status.initialized);
    assert!(status.derivation_method.is_some());
    assert!(status.derivation_method.unwrap().contains("Argon2id"));
}

#[tokio::test]
async fn test_passphrase_initialization_with_custom_params() {
    let fixture = CryptoInitTestFixture::new();
    let db_ops = fixture.db_ops();

    let mut key_derivation = KeyDerivationConfig::for_security_level(SecurityLevel::Balanced);
    key_derivation.memory_cost = 32768;
    key_derivation.time_cost = 3;
    key_derivation.parallelism = 2;

    let crypto_config = CryptoConfig {
        enabled: true,
        master_key: MasterKeyConfig::Passphrase {
            passphrase: "custom-passphrase-test".to_string(),
        },
        key_derivation,
    };

    // Initialize crypto
    let context = initialize_database_crypto(db_ops, &crypto_config)
        .expect("Failed to initialize crypto");

    assert!(context.derivation_method.contains("Argon2id"));
    assert!(context.derivation_method.contains("Custom"));
}

#[tokio::test]
async fn test_double_initialization_error() {
    let fixture = CryptoInitTestFixture::new();
    let db_ops = fixture.db_ops();

    let crypto_config = CryptoConfig::with_random_key();

    // First initialization
    let _context = initialize_database_crypto(db_ops.clone(), &crypto_config)
        .expect("First initialization should succeed");

    // Second initialization should fail
    let result = initialize_database_crypto(db_ops, &crypto_config);
    
    assert!(result.is_err());
    match result.unwrap_err() {
        CryptoInitError::AlreadyInitialized => {
            // Expected error
        }
        other => panic!("Expected AlreadyInitialized error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_validate_random_config() {
    let crypto_config = CryptoConfig::with_random_key();
    
    let result = validate_crypto_config_for_init(&crypto_config);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_passphrase_config() {
    let crypto_config = CryptoConfig {
        enabled: true,
        master_key: MasterKeyConfig::Passphrase {
            passphrase: "good-strong-passphrase-12345".to_string(),
        },
        key_derivation: KeyDerivationConfig::for_security_level(SecurityLevel::Balanced),
    };

    let result = validate_crypto_config_for_init(&crypto_config);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_empty_passphrase_error() {
    let crypto_config = CryptoConfig {
        enabled: true,
        master_key: MasterKeyConfig::Passphrase {
            passphrase: "".to_string(),
        },
        key_derivation: KeyDerivationConfig::for_security_level(SecurityLevel::Interactive),
    };

    let result = validate_crypto_config_for_init(&crypto_config);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_sensitive_security_level() {
    let fixture = CryptoInitTestFixture::new();
    let db_ops = fixture.db_ops();

    let crypto_config = CryptoConfig {
        enabled: true,
        master_key: MasterKeyConfig::Passphrase {
            passphrase: "sensitive-security-test-passphrase".to_string(),
        },
        key_derivation: KeyDerivationConfig::for_security_level(SecurityLevel::Sensitive),
    };

    let context = initialize_database_crypto(db_ops, &crypto_config)
        .expect("Failed to initialize crypto");

    assert!(context.derivation_method.contains("Sensitive"));
}

#[tokio::test]
async fn test_interactive_security_level() {
    let fixture = CryptoInitTestFixture::new();
    let db_ops = fixture.db_ops();

    let crypto_config = CryptoConfig {
        enabled: true,
        master_key: MasterKeyConfig::Passphrase {
            passphrase: "interactive-security-test-passphrase".to_string(),
        },
        key_derivation: KeyDerivationConfig::for_security_level(SecurityLevel::Interactive),
    };

    let context = initialize_database_crypto(db_ops, &crypto_config)
        .expect("Failed to initialize crypto");

    assert!(context.derivation_method.contains("Interactive"));
}

#[tokio::test]
async fn test_complete_crypto_initialization_workflow() {
    let fixture = CryptoInitTestFixture::new();
    let db_ops = fixture.db_ops();

    // Step 1: Check initial status (should be not initialized)
    let initial_status = get_crypto_init_status(db_ops.clone())
        .expect("Failed to get initial status");
    assert!(!initial_status.initialized);

    // Step 2: Validate configuration
    let crypto_config = CryptoConfig {
        enabled: true,
        master_key: MasterKeyConfig::Passphrase {
            passphrase: "complete-workflow-test-passphrase".to_string(),
        },
        key_derivation: KeyDerivationConfig::for_security_level(SecurityLevel::Balanced),
    };

    let validation_result = validate_crypto_config_for_init(&crypto_config);
    assert!(validation_result.is_ok());

    // Step 3: Check if initialization is needed
    let needs_init = is_crypto_init_needed(db_ops.clone(), Some(&crypto_config))
        .expect("Failed to check if init needed");
    assert!(needs_init);

    // Step 4: Initialize crypto
    let context = initialize_database_crypto(db_ops.clone(), &crypto_config)
        .expect("Failed to initialize crypto");
    assert!(context.derivation_method.contains("Argon2id"));

    // Step 5: Verify final status
    let final_status = get_crypto_init_status(db_ops.clone())
        .expect("Failed to get final status");
    
    assert!(final_status.initialized);
    assert!(final_status.derivation_method.is_some());
    assert!(final_status.created_at.is_some());
    assert_eq!(final_status.integrity_verified, Some(true));

    // Step 6: Verify initialization is no longer needed
    let needs_init_after = is_crypto_init_needed(db_ops.clone(), Some(&crypto_config))
        .expect("Failed to check if init needed after initialization");
    assert!(!needs_init_after);
}

#[tokio::test]
async fn test_crypto_config_validation_scenarios() {
    // Test disabled crypto config - it should be valid for validation but not used for init
    let disabled_config = CryptoConfig::disabled();
    let result = validate_crypto_config_for_init(&disabled_config);
    assert!(result.is_ok()); // Disabled config passes validation

    // Test passphrase with empty string (should fail)
    let empty_passphrase_config = CryptoConfig {
        enabled: true,
        master_key: MasterKeyConfig::Passphrase {
            passphrase: "".to_string(),
        },
        key_derivation: KeyDerivationConfig::for_security_level(SecurityLevel::Interactive),
    };
    let result = validate_crypto_config_for_init(&empty_passphrase_config);
    assert!(result.is_err()); // Should fail because empty passphrase
}

#[tokio::test]
async fn test_is_crypto_init_needed_scenarios() {
    let fixture = CryptoInitTestFixture::new();
    let db_ops = fixture.db_ops();

    // Test without any config
    let needs_init_no_config = is_crypto_init_needed(db_ops.clone(), None)
        .expect("Failed to check init needed without config");
    assert!(!needs_init_no_config); // No crypto config = no crypto needed

    // Test with disabled config
    let disabled_config = CryptoConfig::disabled();
    let needs_init_disabled = is_crypto_init_needed(db_ops.clone(), Some(&disabled_config))
        .expect("Failed to check init needed with disabled config");
    assert!(!needs_init_disabled); // Disabled crypto = no crypto needed

    // Test with enabled config
    let enabled_config = CryptoConfig::with_random_key();
    let needs_init_enabled = is_crypto_init_needed(db_ops, Some(&enabled_config))
        .expect("Failed to check init needed with enabled config");
    assert!(needs_init_enabled); // Enabled crypto on fresh DB = crypto needed
}