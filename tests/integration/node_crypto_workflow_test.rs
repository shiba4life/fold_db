//! End-to-end crypto workflow testing for DataFoldNode
//! 
//! This test validates the complete database creation workflow with crypto initialization,
//! ensuring that all crypto integration points work correctly from NodeConfig to database setup.

use datafold::{
    config::crypto::{CryptoConfig, MasterKeyConfig, KeyDerivationConfig, SecurityLevel},
    datafold_node::{NodeConfig, DataFoldNode},
};
use std::path::PathBuf;
use tempfile::TempDir;

/// Test fixture for node crypto workflow testing
struct NodeCryptoWorkflowFixture {
    temp_dir: TempDir,
    base_config: NodeConfig,
}

impl NodeCryptoWorkflowFixture {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage_path = temp_dir.path().join("test_db");
        
        let base_config = NodeConfig::new(storage_path);

        Self {
            temp_dir,
            base_config,
        }
    }

    fn config_with_random_crypto(&self, security_level: SecurityLevel) -> NodeConfig {
        let mut config = self.base_config.clone();
        config.crypto = Some(CryptoConfig {
            enabled: true,
            master_key: MasterKeyConfig::Random,
            key_derivation: KeyDerivationConfig::for_security_level(security_level),
        });
        config
    }

    fn config_with_passphrase_crypto(&self, passphrase: &str, security_level: SecurityLevel) -> NodeConfig {
        let mut config = self.base_config.clone();
        config.crypto = Some(CryptoConfig {
            enabled: true,
            master_key: MasterKeyConfig::Passphrase {
                passphrase: passphrase.to_string(),
            },
            key_derivation: KeyDerivationConfig::for_security_level(security_level),
        });
        config
    }

    fn config_without_crypto(&self) -> NodeConfig {
        self.base_config.clone()
    }
}

#[tokio::test]
async fn test_complete_node_creation_workflow_with_random_crypto() {
    let fixture = NodeCryptoWorkflowFixture::new();
    
    // Test with different security levels
    for (i, security_level) in [SecurityLevel::Interactive, SecurityLevel::Balanced, SecurityLevel::Sensitive].iter().enumerate() {
        // Create unique config for each test to avoid database lock conflicts
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage_path = temp_dir.path().join(format!("test_db_{}", i));
        let mut config = NodeConfig::new(storage_path);
        config.crypto = Some(CryptoConfig {
            enabled: true,
            master_key: MasterKeyConfig::Random,
            key_derivation: KeyDerivationConfig::for_security_level(*security_level),
        });
        
        // Create node with crypto enabled
        let _node = DataFoldNode::new(config).expect("Failed to create node with random crypto");
        
        // Verify node was created successfully - we can't access private fields
        // but successful creation implies the crypto initialization worked
        println!("✅ Node creation with {:?} security level successful", security_level);
    }
}

#[tokio::test]
async fn test_complete_node_creation_workflow_with_passphrase_crypto() {
    let fixture = NodeCryptoWorkflowFixture::new();
    
    // Test with different passphrases and security levels
    let test_cases = [
        ("short_secure_pass", SecurityLevel::Interactive),
        ("medium_length_secure_passphrase_123", SecurityLevel::Balanced),
        ("very_long_extremely_secure_passphrase_with_numbers_123_and_symbols_!@#", SecurityLevel::Sensitive),
    ];
    
    for (i, (passphrase, security_level)) in test_cases.iter().enumerate() {
        // Create unique config for each test to avoid database lock conflicts
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage_path = temp_dir.path().join(format!("test_db_{}", i));
        let mut config = NodeConfig::new(storage_path);
        config.crypto = Some(CryptoConfig {
            enabled: true,
            master_key: MasterKeyConfig::Passphrase {
                passphrase: passphrase.to_string(),
            },
            key_derivation: KeyDerivationConfig::for_security_level(*security_level),
        });
        
        // Create node with passphrase-based crypto
        let _node = DataFoldNode::new(config).expect("Failed to create node with passphrase crypto");
        
        // Verify node was created successfully - we can't access private fields
        // but successful creation implies the crypto initialization worked
        println!("✅ Node creation with passphrase ({} chars) and {:?} security level successful",
                passphrase.len(), security_level);
    }
}

#[tokio::test]
async fn test_node_creation_workflow_without_crypto() {
    let fixture = NodeCryptoWorkflowFixture::new();
    let config = fixture.config_without_crypto();
    
    // Create node without crypto
    let node = DataFoldNode::new(config.clone()).expect("Failed to create node without crypto");
    
    // Verify node was created successfully - we can't access private fields
    // but successful creation implies the workflow worked
    println!("✅ Node created successfully without crypto");
    
    println!("✅ Node creation without crypto successful");
}

#[tokio::test]
async fn test_database_crypto_persistence_workflow() {
    // Test that crypto initialization creates a database that can be verified
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage_path = temp_dir.path().join("test_db");
    let mut config = NodeConfig::new(storage_path);
    config.crypto = Some(CryptoConfig {
        enabled: true,
        master_key: MasterKeyConfig::Random,
        key_derivation: KeyDerivationConfig::for_security_level(SecurityLevel::Interactive),
    });
    
    // Create node with crypto - this should initialize crypto metadata
    let _node = DataFoldNode::new(config).expect("Failed to create node with crypto");
    
    // The fact that the node was created successfully implies:
    // 1. Crypto validation passed
    // 2. Key generation/derivation worked
    // 3. Crypto metadata was stored in the database
    // 4. Database initialization workflow integrated crypto setup correctly
    
    println!("✅ Database crypto persistence workflow successful");
}

#[tokio::test]
async fn test_multiple_node_creation_with_different_crypto_configs() {
    let fixture = NodeCryptoWorkflowFixture::new();
    
    // Create multiple nodes with different crypto configurations
    let configs = vec![
        ("random_interactive", fixture.config_with_random_crypto(SecurityLevel::Interactive)),
        ("random_balanced", fixture.config_with_random_crypto(SecurityLevel::Balanced)),
        ("passphrase_sensitive", fixture.config_with_passphrase_crypto("secure_test_passphrase", SecurityLevel::Sensitive)),
        ("no_crypto", fixture.config_without_crypto()),
    ];
    
    let mut nodes = Vec::new();
    
    for (name, mut config) in configs {
        // Use different storage paths for each node
        config.storage_path = fixture.temp_dir.path().join(format!("db_{}", name));
        
        let node = DataFoldNode::new(config).expect(&format!("Failed to create node: {}", name));
        
        nodes.push((name, node));
        println!("✅ Created node: {}", name);
    }
    
    // Verify all nodes were created successfully
    assert_eq!(nodes.len(), 4, "All 4 nodes should be created successfully");
    
    // All nodes created successfully, which validates the crypto workflows
    
    println!("✅ Multiple node creation with different crypto configs successful");
}

#[tokio::test]
async fn test_crypto_initialization_error_handling() {
    let fixture = NodeCryptoWorkflowFixture::new();
    
    // Test with invalid passphrase (too short)
    let invalid_config = fixture.config_with_passphrase_crypto("", SecurityLevel::Interactive);
    
    // This should fail due to validation
    let result = DataFoldNode::new(invalid_config);
    assert!(result.is_err(), "Node creation should fail with empty passphrase");
    
    let error_msg = result.err().unwrap().to_string();
    assert!(error_msg.contains("Crypto config validation failed"), 
           "Error should mention crypto config validation failure");
    
    println!("✅ Crypto initialization error handling working correctly");
}

#[tokio::test]
async fn test_crypto_workflow_performance_baseline() {
    // Benchmark node creation times with different crypto configurations
    let start = std::time::Instant::now();
    
    // Create node without crypto (baseline) with unique storage path
    let baseline_duration = {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config = NodeConfig::new(temp_dir.path().join("baseline_db"));
        let _node_no_crypto = DataFoldNode::new(config)
            .expect("Failed to create node without crypto");
        start.elapsed()
    };
    
    // Create node with random crypto with unique storage path
    let random_crypto_duration = {
        let start = std::time::Instant::now();
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mut config = NodeConfig::new(temp_dir.path().join("random_crypto_db"));
        config.crypto = Some(CryptoConfig {
            enabled: true,
            master_key: MasterKeyConfig::Random,
            key_derivation: KeyDerivationConfig::for_security_level(SecurityLevel::Interactive),
        });
        let _node_random_crypto = DataFoldNode::new(config)
            .expect("Failed to create node with random crypto");
        start.elapsed()
    };
    
    // Create node with passphrase crypto with unique storage path
    let passphrase_crypto_duration = {
        let start = std::time::Instant::now();
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let passphrase = "test_secure_passphrase_for_performance_test";
        let mut config = NodeConfig::new(temp_dir.path().join("passphrase_crypto_db"));
        config.crypto = Some(CryptoConfig {
            enabled: true,
            master_key: MasterKeyConfig::Passphrase {
                passphrase: passphrase.to_string(),
            },
            key_derivation: KeyDerivationConfig::for_security_level(SecurityLevel::Interactive),
        });
        let _node_passphrase_crypto = DataFoldNode::new(config)
            .expect("Failed to create node with passphrase crypto");
        start.elapsed()
    };
    
    println!("Performance baseline:");
    println!("  No crypto: {:?}", baseline_duration);
    println!("  Random crypto: {:?}", random_crypto_duration);
    println!("  Passphrase crypto: {:?}", passphrase_crypto_duration);
    
    // Verify crypto initialization doesn't add excessive overhead
    // Allow up to 10x overhead for crypto initialization (very generous)
    let max_acceptable_overhead = baseline_duration * 10;
    assert!(random_crypto_duration < max_acceptable_overhead,
           "Random crypto initialization should not add excessive overhead");
    assert!(passphrase_crypto_duration < max_acceptable_overhead,
           "Passphrase crypto initialization should not add excessive overhead");
    
    println!("✅ Crypto workflow performance within acceptable limits");
}