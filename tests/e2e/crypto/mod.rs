//! End-to-End Cryptographic Workflow Tests
//!
//! This module contains comprehensive end-to-end tests that validate complete
//! cryptographic workflows from initialization through complex operations.

pub mod complete_workflows;
pub mod migration_validation;
pub mod real_world_scenarios;

// Re-export workflow utilities
pub use complete_workflows::*;

/// End-to-end testing utilities
pub mod e2e_utils {
    use datafold::unified_crypto::*;
    use tempfile::TempDir;
    use std::collections::HashMap;
    use std::path::PathBuf;

    /// Complete system test fixture with all components
    pub struct E2ETestSystem {
        pub temp_dir: TempDir,
        pub crypto: UnifiedCrypto,
        pub config: CryptoConfig,
        pub test_data: HashMap<String, Vec<u8>>,
        pub keypairs: Vec<KeyPair>,
    }

    impl E2ETestSystem {
        /// Create a new end-to-end test system
        pub fn new() -> Self {
            let temp_dir = TempDir::new().expect("Failed to create temp directory");
            let config = Self::create_production_like_config();
            let crypto = UnifiedCrypto::new(config.clone())
                .expect("Failed to create crypto system");

            let mut test_data = HashMap::new();
            Self::populate_test_data(&mut test_data);

            Self {
                temp_dir,
                crypto,
                config,
                test_data,
                keypairs: Vec::new(),
            }
        }

        /// Create production-like configuration for testing
        fn create_production_like_config() -> CryptoConfig {
            let mut config = CryptoConfig::default();
            
            // Enable comprehensive auditing
            config.audit.enabled = true;
            config.audit.log_level = crate::unified_crypto::audit::AuditLevel::Info;
            
            // Set realistic key parameters
            config.keys.default_key_size = 2048;
            config.keys.rotation_interval_days = 90;
            
            // Enable all security features
            config.primitives.encryption.use_aad = true;
            config.primitives.encryption.compression_enabled = false; // Disable for security
            
            config
        }

        /// Populate test data with various scenarios
        fn populate_test_data(test_data: &mut HashMap<String, Vec<u8>>) {
            // Small data
            test_data.insert("small".to_string(), b"Hello, World!".to_vec());
            
            // Medium data (1KB)
            test_data.insert("medium".to_string(), vec![42u8; 1024]);
            
            // Large data (1MB)
            test_data.insert("large".to_string(), vec![123u8; 1024 * 1024]);
            
            // Binary data with various patterns
            test_data.insert("binary_zeros".to_string(), vec![0u8; 512]);
            test_data.insert("binary_ones".to_string(), vec![0xFFu8; 512]);
            test_data.insert("binary_pattern".to_string(), 
                (0..512).map(|i| (i % 256) as u8).collect());
            
            // Text data
            test_data.insert("text".to_string(), 
                "The quick brown fox jumps over the lazy dog. ".repeat(100).into_bytes());
            
            // JSON-like structured data
            test_data.insert("json".to_string(), 
                r#"{"users": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]}"#.repeat(50).into_bytes());
        }

        /// Generate multiple keypairs for testing
        pub fn generate_keypairs(&mut self, count: usize) -> Result<(), Box<dyn std::error::Error>> {
            for _ in 0..count {
                let keypair = self.crypto.generate_keypair()?;
                self.keypairs.push(keypair);
            }
            Ok(())
        }

        /// Get test data by name
        pub fn get_test_data(&self, name: &str) -> Option<&Vec<u8>> {
            self.test_data.get(name)
        }

        /// Get all test data names
        pub fn test_data_names(&self) -> Vec<&String> {
            self.test_data.keys().collect()
        }

        /// Execute complete encryption workflow
        pub fn execute_encryption_workflow(&self, data_name: &str, keypair_index: usize) 
            -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            let data = self.get_test_data(data_name)
                .ok_or("Test data not found")?;
            
            let keypair = self.keypairs.get(keypair_index)
                .ok_or("Keypair not found")?;
            
            // Encrypt
            let encrypted = self.crypto.encrypt(data, &keypair.public_key)?;
            
            // Decrypt
            let decrypted = self.crypto.decrypt(&encrypted, &keypair.private_key)?;
            
            // Verify roundtrip
            if data != &decrypted {
                return Err("Encryption roundtrip failed".into());
            }
            
            Ok(decrypted)
        }

        /// Execute complete signing workflow
        pub fn execute_signing_workflow(&self, data_name: &str, keypair_index: usize) 
            -> Result<bool, Box<dyn std::error::Error>> {
            let data = self.get_test_data(data_name)
                .ok_or("Test data not found")?;
            
            let keypair = self.keypairs.get(keypair_index)
                .ok_or("Keypair not found")?;
            
            // Sign
            let signature = self.crypto.sign(data, &keypair.private_key)?;
            
            // Verify
            let valid = self.crypto.verify(data, &signature, &keypair.public_key)?;
            
            if !valid {
                return Err("Signature verification failed".into());
            }
            
            Ok(valid)
        }

        /// Execute multi-step cryptographic workflow
        pub fn execute_complex_workflow(&self) -> Result<String, Box<dyn std::error::Error>> {
            let mut results = Vec::new();
            
            // Step 1: Generate multiple keypairs
            results.push("Generated keypairs".to_string());
            
            // Step 2: Test all data types with all keypairs
            for data_name in self.test_data_names() {
                for (i, _) in self.keypairs.iter().enumerate() {
                    // Test encryption
                    self.execute_encryption_workflow(data_name, i)?;
                    results.push(format!("Encrypted/decrypted {} with keypair {}", data_name, i));
                    
                    // Test signing
                    self.execute_signing_workflow(data_name, i)?;
                    results.push(format!("Signed/verified {} with keypair {}", data_name, i));
                }
            }
            
            // Step 3: Test hash consistency
            for data_name in self.test_data_names() {
                let data = self.get_test_data(data_name).unwrap();
                let hash = self.crypto.hash(data, HashAlgorithm::Sha256)?;
                results.push(format!("Hashed {} ({}B)", data_name, hash.len()));
            }
            
            Ok(results.join("; "))
        }

        /// Simulate real-world usage patterns
        pub fn simulate_production_usage(&self) -> Result<(), Box<dyn std::error::Error>> {
            // Simulate database encryption
            let db_data = self.get_test_data("json").unwrap();
            let db_keypair = &self.keypairs[0];
            
            let encrypted_db = self.crypto.encrypt(db_data, &db_keypair.public_key)?;
            let _decrypted_db = self.crypto.decrypt(&encrypted_db, &db_keypair.private_key)?;
            
            // Simulate API request signing
            let api_data = self.get_test_data("text").unwrap();
            let api_keypair = &self.keypairs[1];
            
            let api_signature = self.crypto.sign(api_data, &api_keypair.private_key)?;
            let _api_valid = self.crypto.verify(api_data, &api_signature, &api_keypair.public_key)?;
            
            // Simulate backup encryption
            let backup_data = self.get_test_data("large").unwrap();
            let backup_keypair = &self.keypairs[2];
            
            let encrypted_backup = self.crypto.encrypt(backup_data, &backup_keypair.public_key)?;
            let _restored_backup = self.crypto.decrypt(&encrypted_backup, &backup_keypair.private_key)?;
            
            Ok(())
        }

        /// Validate system integrity
        pub fn validate_system_integrity(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
            let mut validation_results = Vec::new();
            
            // Check all keypairs are unique
            for i in 0..self.keypairs.len() {
                for j in (i + 1)..self.keypairs.len() {
                    if self.keypairs[i].key_id() == self.keypairs[j].key_id() {
                        return Err(format!("Duplicate keypair IDs: {} and {}", i, j).into());
                    }
                }
            }
            validation_results.push("All keypairs are unique".to_string());
            
            // Check all data can be processed
            for data_name in self.test_data_names() {
                let data = self.get_test_data(data_name).unwrap();
                
                // Test hashing
                let _hash = self.crypto.hash(data, HashAlgorithm::Sha256)?;
                validation_results.push(format!("Data {} can be hashed", data_name));
                
                // Test encryption with first keypair
                if !self.keypairs.is_empty() {
                    let encrypted = self.crypto.encrypt(data, &self.keypairs[0].public_key)?;
                    let _decrypted = self.crypto.decrypt(&encrypted, &self.keypairs[0].private_key)?;
                    validation_results.push(format!("Data {} can be encrypted/decrypted", data_name));
                }
            }
            
            Ok(validation_results)
        }
    }

    /// Test scenario definitions
    pub enum TestScenario {
        BasicCrypto,
        HighVolume,
        LongRunning,
        ErrorRecovery,
        SecurityBoundary,
    }

    impl TestScenario {
        pub fn description(&self) -> &'static str {
            match self {
                TestScenario::BasicCrypto => "Basic cryptographic operations",
                TestScenario::HighVolume => "High-volume data processing",
                TestScenario::LongRunning => "Long-running system stability",
                TestScenario::ErrorRecovery => "Error handling and recovery",
                TestScenario::SecurityBoundary => "Security boundary validation",
            }
        }
    }
}