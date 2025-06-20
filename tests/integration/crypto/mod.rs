//! Integration tests for the unified cryptographic system
//!
//! This module contains comprehensive integration tests that validate the unified
//! cryptographic system across all modules and operational layers.

pub mod unified_crypto_integration;
pub mod key_management_integration;
pub mod database_crypto_integration;
pub mod auth_crypto_integration;
pub mod network_crypto_integration;
pub mod backup_crypto_integration;
pub mod cli_crypto_integration;
pub mod end_to_end_workflows;

// Re-export common test utilities
pub use unified_crypto_integration::*;

/// Common test utilities for crypto integration tests
pub mod test_utils {
    use datafold::unified_crypto::*;
    use tempfile::TempDir;
    use std::path::PathBuf;

    /// Test fixture for integration tests
    pub struct CryptoIntegrationFixture {
        pub temp_dir: TempDir,
        pub crypto: UnifiedCrypto,
        pub config: CryptoConfig,
    }

    impl CryptoIntegrationFixture {
        /// Create a new test fixture with default configuration
        pub fn new() -> Self {
            let temp_dir = TempDir::new().expect("Failed to create temp directory");
            let mut config = CryptoConfig::default();
            
            // Configure for testing environment
            config.audit.enabled = true;
            config.audit.log_level = crate::unified_crypto::audit::AuditLevel::Debug;
            
            let crypto = UnifiedCrypto::new(config.clone())
                .expect("Failed to create unified crypto instance");

            Self {
                temp_dir,
                crypto,
                config,
            }
        }

        /// Create a test fixture with custom configuration
        pub fn with_config(config: CryptoConfig) -> Self {
            let temp_dir = TempDir::new().expect("Failed to create temp directory");
            let crypto = UnifiedCrypto::new(config.clone())
                .expect("Failed to create unified crypto instance");

            Self {
                temp_dir,
                crypto,
                config,
            }
        }

        /// Get the temporary directory path
        pub fn temp_path(&self) -> PathBuf {
            self.temp_dir.path().to_path_buf()
        }

        /// Generate test data of specified size
        pub fn generate_test_data(size: usize) -> Vec<u8> {
            (0..size).map(|i| (i % 256) as u8).collect()
        }

        /// Create multiple test key pairs
        pub fn create_test_keypairs(&self, count: usize) -> Vec<KeyPair> {
            (0..count)
                .map(|_| self.crypto.generate_keypair().expect("Failed to generate keypair"))
                .collect()
        }
    }

    /// Assertion helpers for crypto testing
    pub mod assertions {
        use datafold::unified_crypto::*;

        /// Assert that encryption/decryption roundtrip works correctly
        pub fn assert_encryption_roundtrip(
            crypto: &UnifiedCrypto,
            plaintext: &[u8],
            public_key: &PublicKeyHandle,
            private_key: &PrivateKeyHandle,
        ) {
            let encrypted = crypto.encrypt(plaintext, public_key)
                .expect("Encryption should succeed");
            
            let decrypted = crypto.decrypt(&encrypted, private_key)
                .expect("Decryption should succeed");
            
            assert_eq!(plaintext, &decrypted[..], "Roundtrip should preserve data");
        }

        /// Assert that signing/verification roundtrip works correctly
        pub fn assert_signing_roundtrip(
            crypto: &UnifiedCrypto,
            data: &[u8],
            public_key: &PublicKeyHandle,
            private_key: &PrivateKeyHandle,
        ) {
            let signature = crypto.sign(data, private_key)
                .expect("Signing should succeed");
            
            let valid = crypto.verify(data, &signature, public_key)
                .expect("Verification should succeed");
            
            assert!(valid, "Signature should be valid");
        }

        /// Assert that hashing is consistent
        pub fn assert_hash_consistency(
            crypto: &UnifiedCrypto,
            data: &[u8],
            algorithm: HashAlgorithm,
        ) {
            let hash1 = crypto.hash(data, algorithm)
                .expect("First hash should succeed");
            
            let hash2 = crypto.hash(data, algorithm)
                .expect("Second hash should succeed");
            
            assert_eq!(hash1, hash2, "Hash should be deterministic");
        }
    }
}