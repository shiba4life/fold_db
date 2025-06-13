//! Integration tests for CLI crypto initialization functionality
//!
//! This module tests the command-line interface for database cryptographic
//! initialization, including random key generation, passphrase-based derivation,
//! and status checking.

use serde_json;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

use datafold::config::crypto::{CryptoConfig, KeyDerivationConfig, MasterKeyConfig, SecurityLevel};
use datafold::datafold_node::config::NodeConfig;
use datafold::datafold_node::crypto_init::{get_crypto_init_status, initialize_database_crypto};
use datafold::datafold_node::{load_node_config, DataFoldNode};

/// Helper function to create a temporary node configuration with crypto settings
fn create_test_node_config_with_crypto(
    crypto_config: CryptoConfig,
) -> (NodeConfig, tempfile::TempDir) {
    let temp_dir = tempdir().unwrap();
    let storage_path = temp_dir.path().join("test_db");

    let mut node_config = NodeConfig::new(storage_path);
    node_config.crypto = Some(crypto_config);

    (node_config, temp_dir)
}

/// Helper function to create a temporary node configuration file
fn create_test_config_file(crypto_config: Option<CryptoConfig>) -> (PathBuf, tempfile::TempDir) {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("test_config.json");
    let storage_path = temp_dir.path().join("test_db");

    let mut node_config = NodeConfig::new(storage_path);
    node_config.crypto = crypto_config;

    let config_json = serde_json::to_string_pretty(&node_config).unwrap();
    fs::write(&config_path, config_json).unwrap();

    (config_path, temp_dir)
}

#[tokio::test]
async fn test_crypto_init_with_random_key() {
    // Create test configuration
    let crypto_config = CryptoConfig::with_random_key();
    let (node_config, _temp_dir) = create_test_node_config_with_crypto(crypto_config);

    // Initialize node
    let mut node = DataFoldNode::load(node_config).await.unwrap();

    // Check crypto status - should be initialized
    let fold_db = node.get_fold_db().unwrap();
    let db_ops = fold_db.db_ops();
    let status = get_crypto_init_status(db_ops).unwrap();

    assert!(status.initialized);
    assert!(status.is_healthy());
    assert_eq!(status.derivation_method, Some("Random".to_string()));
}

#[tokio::test]
async fn test_crypto_init_with_passphrase() {
    // Create test configuration with passphrase
    let crypto_config = CryptoConfig::with_passphrase("test-passphrase-12345".to_string());
    let (node_config, _temp_dir) = create_test_node_config_with_crypto(crypto_config);

    // Initialize node
    let mut node = DataFoldNode::load(node_config).await.unwrap();

    // Check crypto status
    let fold_db = node.get_fold_db().unwrap();
    let db_ops = fold_db.db_ops();
    let status = get_crypto_init_status(db_ops).unwrap();

    assert!(status.initialized);
    assert!(status.is_healthy());
    assert!(status.derivation_method.unwrap().starts_with("Argon2id"));
}

#[tokio::test]
async fn test_crypto_init_with_different_security_levels() {
    for security_level in &[
        SecurityLevel::Interactive,
        SecurityLevel::Balanced,
        SecurityLevel::Sensitive,
    ] {
        // Create configuration with specific security level
        let crypto_config = CryptoConfig {
            enabled: true,
            master_key: MasterKeyConfig::Passphrase {
                passphrase: "secure-test-passphrase".to_string(),
            },
            key_derivation: KeyDerivationConfig::for_security_level(*security_level),
        };

        let (node_config, _temp_dir) = create_test_node_config_with_crypto(crypto_config);

        // Initialize node
        let mut node = DataFoldNode::load(node_config).await.unwrap();

        // Check crypto status
        let fold_db = node.get_fold_db().unwrap();
        let db_ops = fold_db.db_ops();
        let status = get_crypto_init_status(db_ops).unwrap();

        assert!(status.initialized);
        assert!(status.is_healthy());

        let method = status.derivation_method.unwrap();
        match security_level {
            SecurityLevel::Interactive => assert_eq!(method, "Argon2id-Interactive"),
            SecurityLevel::Balanced => assert_eq!(method, "Argon2id-Balanced"),
            SecurityLevel::Sensitive => assert_eq!(method, "Argon2id-Sensitive"),
        }
    }
}

#[tokio::test]
async fn test_crypto_status_on_uninitialized_database() {
    // Create node without crypto configuration
    let temp_dir = tempdir().unwrap();
    let storage_path = temp_dir.path().join("test_db");
    let node_config = NodeConfig::new(storage_path);

    // Initialize node
    let mut node = DataFoldNode::load(node_config).await.unwrap();

    // Check crypto status - should not be initialized
    let fold_db = node.get_fold_db().unwrap();
    let db_ops = fold_db.db_ops();
    let status = get_crypto_init_status(db_ops).unwrap();

    assert!(!status.initialized);
    assert!(!status.is_healthy());
    assert_eq!(status.summary(), "Not initialized");
}

#[test]
fn test_crypto_config_validation_valid_configurations() {
    // Test valid random key configuration
    let random_config = CryptoConfig::with_random_key();
    assert!(random_config.validate().is_ok());

    // Test valid passphrase configuration
    let passphrase_config = CryptoConfig::with_passphrase("valid-passphrase".to_string());
    assert!(passphrase_config.validate().is_ok());

    // Test enhanced security configuration
    let enhanced_config = CryptoConfig::with_enhanced_security("strong-passphrase".to_string());
    assert!(enhanced_config.validate().is_ok());
}

#[test]
fn test_crypto_config_validation_invalid_configurations() {
    // Test empty passphrase
    let empty_passphrase_config = CryptoConfig {
        enabled: true,
        master_key: MasterKeyConfig::Passphrase {
            passphrase: "".to_string(),
        },
        key_derivation: KeyDerivationConfig::default(),
    };
    assert!(empty_passphrase_config.validate().is_err());

    // Test very short passphrase
    let short_passphrase_config = CryptoConfig {
        enabled: true,
        master_key: MasterKeyConfig::Passphrase {
            passphrase: "12345".to_string(),
        },
        key_derivation: KeyDerivationConfig::default(),
    };
    assert!(short_passphrase_config.validate().is_err());
}

#[tokio::test]
async fn test_manual_crypto_initialization() {
    // Create node without crypto in initial config
    let temp_dir = tempdir().unwrap();
    let storage_path = temp_dir.path().join("test_db");
    let node_config = NodeConfig::new(storage_path);

    // Initialize node
    let mut node = DataFoldNode::load(node_config).await.unwrap();

    // Manually initialize crypto
    let crypto_config = CryptoConfig::with_passphrase("manual-init-passphrase".to_string());
    let fold_db = node.get_fold_db().unwrap();
    let db_ops = fold_db.db_ops();

    let context = initialize_database_crypto(db_ops.clone(), &crypto_config).unwrap();

    // Verify initialization
    assert!(!context.derivation_method.is_empty());
    assert_eq!(context.derivation_method, "Argon2id-Balanced");

    // Check status
    let status = get_crypto_init_status(db_ops).unwrap();
    assert!(status.initialized);
    assert!(status.is_healthy());
}

#[test]
fn test_load_config_with_crypto() {
    // Test loading configuration file with crypto settings
    let crypto_config = CryptoConfig::with_passphrase("config-file-passphrase".to_string());
    let (config_path, _temp_dir) = create_test_config_file(Some(crypto_config));

    let loaded_config = load_node_config(Some(config_path.to_str().unwrap()), None).unwrap();

    assert!(loaded_config.crypto.is_some());
    let crypto = loaded_config.crypto.unwrap();
    assert!(crypto.enabled);
    assert!(crypto.requires_passphrase());
}

#[test]
fn test_load_config_without_crypto() {
    // Test loading configuration file without crypto settings
    let (config_path, _temp_dir) = create_test_config_file(None);

    let loaded_config = load_node_config(Some(config_path.to_str().unwrap()), None).unwrap();

    // Crypto should be None or disabled
    if let Some(crypto) = loaded_config.crypto {
        assert!(!crypto.enabled);
    }
}

#[tokio::test]
async fn test_crypto_double_initialization_prevention() {
    // Create and initialize node with crypto
    let crypto_config = CryptoConfig::with_random_key();
    let (node_config, _temp_dir) = create_test_node_config_with_crypto(crypto_config.clone());

    let mut node = DataFoldNode::load(node_config).await.unwrap();

    // Try to initialize crypto again - should fail
    let fold_db = node.get_fold_db().unwrap();
    let db_ops = fold_db.db_ops();

    let result = initialize_database_crypto(db_ops, &crypto_config);
    assert!(result.is_err());

    // Error should indicate already initialized
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("already has crypto metadata"));
}

#[test]
fn test_security_level_string_representations() {
    assert_eq!(SecurityLevel::Interactive.as_str(), "Interactive");
    assert_eq!(SecurityLevel::Balanced.as_str(), "Balanced");
    assert_eq!(SecurityLevel::Sensitive.as_str(), "Sensitive");
}

#[test]
fn test_key_derivation_config_presets() {
    let interactive_config = KeyDerivationConfig::interactive();
    assert_eq!(interactive_config.preset, Some(SecurityLevel::Interactive));
    assert_eq!(interactive_config.memory_cost, 32768); // 32 MB
    assert_eq!(interactive_config.time_cost, 2);
    assert_eq!(interactive_config.parallelism, 2);

    let sensitive_config = KeyDerivationConfig::sensitive();
    assert_eq!(sensitive_config.preset, Some(SecurityLevel::Sensitive));
    assert_eq!(sensitive_config.memory_cost, 131072); // 128 MB
    assert_eq!(sensitive_config.time_cost, 4);
    assert_eq!(sensitive_config.parallelism, 8);

    let balanced_config = KeyDerivationConfig::default();
    assert_eq!(balanced_config.memory_cost, 65536); // 64 MB
    assert_eq!(balanced_config.time_cost, 3);
    assert_eq!(balanced_config.parallelism, 4);
}

#[tokio::test]
async fn test_crypto_metadata_integrity() {
    // Initialize crypto and verify metadata integrity
    let crypto_config = CryptoConfig::with_passphrase("integrity-test-passphrase".to_string());
    let (node_config, _temp_dir) = create_test_node_config_with_crypto(crypto_config);

    let mut node = DataFoldNode::load(node_config).await.unwrap();

    let fold_db = node.get_fold_db().unwrap();
    let db_ops = fold_db.db_ops();

    // Check that metadata exists and is valid
    assert!(db_ops.has_crypto_metadata().unwrap());

    let metadata = db_ops.get_crypto_metadata().unwrap().unwrap();
    assert!(metadata.verify_integrity().unwrap());

    // Check status reflects healthy state
    let status = get_crypto_init_status(db_ops).unwrap();
    assert!(status.is_healthy());
    assert_eq!(status.integrity_verified, Some(true));
}
