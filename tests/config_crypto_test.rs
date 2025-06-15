//! Integration tests for crypto configuration with NodeConfig

use datafold::{
    ConfigError, CryptoConfig, KeyDerivationConfig, MasterKeyConfig, NodeConfig, SecurityLevel,
};
use tempfile::tempdir;

#[test]
fn test_node_config_without_crypto() {
    let config = NodeConfig::default();
    assert!(!config.is_crypto_enabled());
    assert!(config.crypto_config().is_none());
    assert!(config.validate().is_ok());
}

#[test]
fn test_node_config_with_crypto_disabled() {
    let temp_dir = tempdir().unwrap();
    let crypto_config = CryptoConfig::disabled();
    let config = NodeConfig::with_crypto(temp_dir.path().to_path_buf(), crypto_config);

    assert!(!config.is_crypto_enabled());
    assert!(config.crypto_config().is_some());
    assert!(config.validate().is_ok());
}

#[test]
fn test_node_config_with_passphrase_crypto() {
    let temp_dir = tempdir().unwrap();
    let crypto_config = CryptoConfig::with_passphrase("secure-test-passphrase".to_string());
    let config = NodeConfig::with_crypto(temp_dir.path().to_path_buf(), crypto_config);

    assert!(config.is_crypto_enabled());
    assert!(config.crypto_config().is_some());
    assert!(config.crypto_config().unwrap().requires_passphrase());
    assert!(config.validate().is_ok());
}

#[test]
fn test_node_config_with_random_key_crypto() {
    let temp_dir = tempdir().unwrap();
    let crypto_config = CryptoConfig::with_random_key();
    let config = NodeConfig::with_crypto(temp_dir.path().to_path_buf(), crypto_config);

    assert!(config.is_crypto_enabled());
    assert!(config.crypto_config().is_some());
    assert!(config.crypto_config().unwrap().uses_random_key());
    assert!(config.validate().is_ok());
}

#[test]
fn test_node_config_enable_crypto_fluent_api() {
    let temp_dir = tempdir().unwrap();
    let crypto_config = CryptoConfig::with_enhanced_security("strong-passphrase".to_string());
    let config = NodeConfig::new(temp_dir.path().to_path_buf()).enable_crypto(crypto_config);

    assert!(config.is_crypto_enabled());
    let crypto = config.crypto_config().unwrap();
    assert!(crypto.requires_passphrase());
    assert_eq!(crypto.key_derivation.preset, Some(SecurityLevel::High));
    assert!(config.validate().is_ok());
}

#[test]
fn test_node_config_validation_with_invalid_crypto() {
    let temp_dir = tempdir().unwrap();
    let mut crypto_config = CryptoConfig::with_passphrase("short".to_string()); // Too short
    crypto_config.enabled = true;

    let config = NodeConfig::with_crypto(temp_dir.path().to_path_buf(), crypto_config);

    assert!(config.validate().is_err());
    match config.validate() {
        Err(ConfigError::CryptoValidation(_)) => {} // Expected
        _ => panic!("Expected CryptoValidation error"),
    }
}

#[test]
fn test_node_config_serialization_without_crypto() {
    let config = NodeConfig::default();

    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: NodeConfig = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(config.storage_path, deserialized.storage_path);
    assert_eq!(
        config.network_listen_address,
        deserialized.network_listen_address
    );
    assert_eq!(config.crypto.is_some(), deserialized.crypto.is_some());
}

#[test]
fn test_node_config_serialization_with_crypto() {
    let temp_dir = tempdir().unwrap();
    let crypto_config = CryptoConfig::with_passphrase("test-passphrase".to_string());
    let config = NodeConfig::with_crypto(temp_dir.path().to_path_buf(), crypto_config);

    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: NodeConfig = serde_json::from_str(&json).expect("Should deserialize");

    assert!(deserialized.is_crypto_enabled());
    assert!(deserialized.crypto_config().unwrap().requires_passphrase());
    assert!(deserialized.validate().is_ok());
}

#[test]
fn test_node_config_json_example_passphrase() {
    let json_config = r#"{
        "storage_path": "/tmp/test",
        "default_trust_distance": 2,
        "network_listen_address": "/ip4/0.0.0.0/tcp/8080",
        "crypto": {
            "enabled": true,
            "master_key": {
                "type": "passphrase",
                "passphrase": "my-secure-passphrase"
            },
            "key_derivation": {
                "preset": "High"
            }
        }
    }"#;

    let config: NodeConfig = serde_json::from_str(json_config).expect("Should parse JSON config");

    assert!(config.is_crypto_enabled());
    assert!(config.crypto_config().unwrap().requires_passphrase());
    assert_eq!(
        config.crypto_config().unwrap().key_derivation.preset,
        Some(SecurityLevel::High)
    );
    assert!(config.validate().is_ok());
}

#[test]
fn test_node_config_json_example_random_key() {
    let json_config = r#"{
        "storage_path": "/tmp/test",
        "default_trust_distance": 1,
        "network_listen_address": "/ip4/127.0.0.1/tcp/0",
        "crypto": {
            "enabled": true,
            "master_key": {
                "type": "random"
            },
            "key_derivation": {
                "preset": "Low"
            }
        }
    }"#;

    let config: NodeConfig = serde_json::from_str(json_config).expect("Should parse JSON config");

    assert!(config.is_crypto_enabled());
    assert!(config.crypto_config().unwrap().uses_random_key());
    assert_eq!(
        config.crypto_config().unwrap().key_derivation.preset,
        Some(SecurityLevel::Low)
    );
    assert!(config.validate().is_ok());
}

#[test]
fn test_node_config_json_example_custom_argon2() {
    let json_config = r#"{
        "storage_path": "/tmp/test",
        "default_trust_distance": 1,
        "network_listen_address": "/ip4/127.0.0.1/tcp/0",
        "crypto": {
            "enabled": true,
            "master_key": {
                "type": "passphrase",
                "passphrase": "custom-config-passphrase"
            },
            "key_derivation": {
                "memory_cost": 32768,
                "time_cost": 2,
                "parallelism": 2
            }
        }
    }"#;

    let config: NodeConfig = serde_json::from_str(json_config).expect("Should parse JSON config");

    assert!(config.is_crypto_enabled());
    let crypto = config.crypto_config().unwrap();
    assert_eq!(crypto.key_derivation.memory_cost, 32768);
    assert_eq!(crypto.key_derivation.time_cost, 2);
    assert_eq!(crypto.key_derivation.parallelism, 2);
    assert!(config.validate().is_ok());
}

#[test]
fn test_node_config_json_disabled_crypto() {
    let json_config = r#"{
        "storage_path": "/tmp/test",
        "default_trust_distance": 1,
        "network_listen_address": "/ip4/127.0.0.1/tcp/0",
        "crypto": {
            "enabled": false
        }
    }"#;

    let config: NodeConfig = serde_json::from_str(json_config).expect("Should parse JSON config");

    assert!(!config.is_crypto_enabled());
    assert!(config.crypto_config().is_some());
    assert!(config.validate().is_ok());
}

#[test]
fn test_node_config_backward_compatibility() {
    // Old config format without crypto field should still work
    let json_config = r#"{
        "storage_path": "/tmp/test",
        "default_trust_distance": 1,
        "network_listen_address": "/ip4/127.0.0.1/tcp/0"
    }"#;

    let config: NodeConfig =
        serde_json::from_str(json_config).expect("Should parse old JSON config");

    assert!(!config.is_crypto_enabled());
    assert!(config.crypto_config().is_none());
    assert!(config.validate().is_ok());
}

#[test]
fn test_crypto_config_integration_with_argon2() {
    let temp_dir = tempdir().unwrap();
    let crypto_config = CryptoConfig::with_passphrase("integration-test-passphrase".to_string());
    let config = NodeConfig::with_crypto(temp_dir.path().to_path_buf(), crypto_config);

    // Test that we can convert config to Argon2 params
    let crypto = config.crypto_config().unwrap();
    let argon2_params = crypto
        .key_derivation
        .to_argon2_params()
        .expect("Should convert to Argon2Params");

    assert_eq!(argon2_params.memory_cost, 65536); // Default
    assert_eq!(argon2_params.time_cost, 3);
    assert_eq!(argon2_params.parallelism, 4);
}

#[test]
fn test_crypto_config_security_levels() {
    let temp_dir = tempdir().unwrap();

    // Test interactive level
    let mut crypto_config = CryptoConfig::with_passphrase("test-passphrase".to_string());
    crypto_config.key_derivation = KeyDerivationConfig::interactive();
    let config = NodeConfig::with_crypto(temp_dir.path().to_path_buf(), crypto_config);

    let argon2_params = config
        .crypto_config()
        .unwrap()
        .key_derivation
        .to_argon2_params()
        .unwrap();
    assert_eq!(argon2_params.memory_cost, 32768); // Interactive

    // Test sensitive level
    let mut crypto_config = CryptoConfig::with_passphrase("test-passphrase".to_string());
    crypto_config.key_derivation = KeyDerivationConfig::sensitive();
    let config = NodeConfig::with_crypto(temp_dir.path().to_path_buf(), crypto_config);

    let argon2_params = config
        .crypto_config()
        .unwrap()
        .key_derivation
        .to_argon2_params()
        .unwrap();
    assert_eq!(argon2_params.memory_cost, 131072); // Sensitive
}

#[test]
fn test_external_key_source_config() {
    let json_config = r#"{
        "storage_path": "/tmp/test",
        "default_trust_distance": 1,
        "network_listen_address": "/ip4/127.0.0.1/tcp/0",
        "crypto": {
            "enabled": true,
            "master_key": {
                "type": "external",
                "key_source": "/path/to/keyfile"
            }
        }
    }"#;

    let config: NodeConfig =
        serde_json::from_str(json_config).expect("Should parse external key config");

    assert!(config.is_crypto_enabled());
    let crypto = config.crypto_config().unwrap();
    assert!(matches!(
        crypto.master_key,
        MasterKeyConfig::External { .. }
    ));
    assert!(config.validate().is_ok());
}

#[test]
fn test_invalid_json_configs() {
    // Empty passphrase should fail validation
    let json_config = r#"{
        "storage_path": "/tmp/test",
        "default_trust_distance": 1,
        "network_listen_address": "/ip4/127.0.0.1/tcp/0",
        "crypto": {
            "enabled": true,
            "master_key": {
                "type": "passphrase",
                "passphrase": ""
            }
        }
    }"#;

    let config: NodeConfig =
        serde_json::from_str(json_config).expect("Should parse JSON but fail validation");

    assert!(config.validate().is_err());

    // Invalid Argon2 parameters should fail validation
    let json_config = r#"{
        "storage_path": "/tmp/test",
        "default_trust_distance": 1,
        "network_listen_address": "/ip4/127.0.0.1/tcp/0",
        "crypto": {
            "enabled": true,
            "master_key": {
                "type": "random"
            },
            "key_derivation": {
                "memory_cost": 7,
                "time_cost": 1,
                "parallelism": 1
            }
        }
    }"#;

    let config: NodeConfig =
        serde_json::from_str(json_config).expect("Should parse JSON but fail validation");

    assert!(config.validate().is_err());
}
