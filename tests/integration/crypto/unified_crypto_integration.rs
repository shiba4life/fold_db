//! Unified Crypto Integration Tests
//!
//! Comprehensive integration tests for the unified cryptographic system.

use datafold::unified_crypto::{UnifiedCrypto, CryptoConfig};
use tempfile::TempDir;

/// Integration test fixture for unified crypto testing
pub struct UnifiedCryptoTestFixture {
    pub temp_dir: TempDir,
    pub crypto: UnifiedCrypto,
    pub config: CryptoConfig,
}

impl UnifiedCryptoTestFixture {
    /// Create a new test fixture
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config = CryptoConfig::default();
        let crypto = UnifiedCrypto::new(config.clone())
            .expect("Failed to create unified crypto instance");

        Self {
            temp_dir,
            crypto,
            config,
        }
    }

    /// Create fixture with custom config
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafold::unified_crypto::HashAlgorithm;

    #[test]
    fn test_unified_crypto_initialization() {
        let _fixture = UnifiedCryptoTestFixture::new();
        // If we get here, initialization succeeded
    }

    #[test]
    fn test_unified_crypto_with_custom_config() {
        let mut config = CryptoConfig::default();
        
        // Customize configuration for testing
        config.keys.default_key_size = 2048;
        config.audit.enabled = true;
        
        let _fixture = UnifiedCryptoTestFixture::with_config(config);
        // If we get here, custom config initialization succeeded
    }

    #[test]
    fn test_keypair_generation_integration() {
        let fixture = UnifiedCryptoTestFixture::new();
        
        // Test keypair generation
        let keypair1 = fixture.crypto.generate_keypair()
            .expect("First keypair generation should succeed");
        
        let keypair2 = fixture.crypto.generate_keypair()
            .expect("Second keypair generation should succeed");
        
        // Keys should be different
        assert_ne!(keypair1.key_id(), keypair2.key_id());
        assert_ne!(keypair1.public_key.id(), keypair2.public_key.id());
    }

    #[test]
    fn test_encryption_decryption_integration() {
        let fixture = UnifiedCryptoTestFixture::new();
        
        // Generate keypair for testing
        let keypair = fixture.crypto.generate_keypair()
            .expect("Keypair generation should succeed");
        
        let test_data = b"Integration test data for encryption";
        
        // Test encryption
        let encrypted = fixture.crypto.encrypt(test_data, &keypair.public_key)
            .expect("Encryption should succeed");
        
        // Verify encrypted data is different from plaintext
        assert_ne!(encrypted.ciphertext(), test_data);
        
        // Test decryption
        let decrypted = fixture.crypto.decrypt(&encrypted, &keypair.private_key)
            .expect("Decryption should succeed");
        
        // Verify roundtrip
        assert_eq!(test_data, &decrypted[..]);
    }

    #[test]
    fn test_signing_verification_integration() {
        let fixture = UnifiedCryptoTestFixture::new();
        
        // Generate keypair for testing
        let keypair = fixture.crypto.generate_keypair()
            .expect("Keypair generation should succeed");
        
        let test_data = b"Integration test data for signing";
        
        // Test signing
        let signature = fixture.crypto.sign(test_data, &keypair.private_key)
            .expect("Signing should succeed");
        
        // Test verification with correct data
        let valid = fixture.crypto.verify(test_data, &signature, &keypair.public_key)
            .expect("Verification should succeed");
        assert!(valid, "Signature should be valid");
        
        // Test verification with wrong data
        let wrong_data = b"Different data that wasn't signed";
        let invalid = fixture.crypto.verify(wrong_data, &signature, &keypair.public_key)
            .expect("Verification should succeed");
        assert!(!invalid, "Signature should be invalid for wrong data");
    }

    #[test]
    fn test_hashing_integration() {
        let fixture = UnifiedCryptoTestFixture::new();
        
        let test_data = b"Integration test data for hashing";
        
        // Test different hash algorithms
        let sha256_hash = fixture.crypto.hash(test_data, HashAlgorithm::Sha256)
            .expect("SHA-256 hashing should succeed");
        
        let sha3_hash = fixture.crypto.hash(test_data, HashAlgorithm::Sha3_256)
            .expect("SHA3-256 hashing should succeed");
        
        let blake3_hash = fixture.crypto.hash(test_data, HashAlgorithm::Blake3)
            .expect("BLAKE3 hashing should succeed");
        
        // Hashes should be different algorithms
        assert_ne!(sha256_hash, sha3_hash);
        assert_ne!(sha256_hash, blake3_hash);
        assert_ne!(sha3_hash, blake3_hash);
        
        // Hashes should be deterministic
        let sha256_hash2 = fixture.crypto.hash(test_data, HashAlgorithm::Sha256)
            .expect("Second SHA-256 hash should succeed");
        assert_eq!(sha256_hash, sha256_hash2);
    }

    #[test]
    fn test_multiple_operations_integration() {
        let fixture = UnifiedCryptoTestFixture::new();
        
        // Generate multiple keypairs
        let keypair1 = fixture.crypto.generate_keypair()
            .expect("First keypair generation should succeed");
        let keypair2 = fixture.crypto.generate_keypair()
            .expect("Second keypair generation should succeed");
        
        let test_data = b"Multi-operation test data";
        
        // Test encryption with first key
        let encrypted1 = fixture.crypto.encrypt(test_data, &keypair1.public_key)
            .expect("Encryption with first key should succeed");
        
        // Test encryption with second key
        let encrypted2 = fixture.crypto.encrypt(test_data, &keypair2.public_key)
            .expect("Encryption with second key should succeed");
        
        // Encrypted data should be different (different keys + random nonces)
        assert_ne!(encrypted1.ciphertext(), encrypted2.ciphertext());
        
        // Both should decrypt correctly
        let decrypted1 = fixture.crypto.decrypt(&encrypted1, &keypair1.private_key)
            .expect("Decryption with first key should succeed");
        let decrypted2 = fixture.crypto.decrypt(&encrypted2, &keypair2.private_key)
            .expect("Decryption with second key should succeed");
        
        assert_eq!(test_data, &decrypted1[..]);
        assert_eq!(test_data, &decrypted2[..]);
        
        // Test signing with both keys
        let sig1 = fixture.crypto.sign(test_data, &keypair1.private_key)
            .expect("Signing with first key should succeed");
        let sig2 = fixture.crypto.sign(test_data, &keypair2.private_key)
            .expect("Signing with second key should succeed");
        
        // Signatures should be different (different keys)
        assert_ne!(sig1.signature(), sig2.signature());
        
        // Both should verify correctly
        assert!(fixture.crypto.verify(test_data, &sig1, &keypair1.public_key).unwrap());
        assert!(fixture.crypto.verify(test_data, &sig2, &keypair2.public_key).unwrap());
        
        // Cross-verification should fail
        assert!(!fixture.crypto.verify(test_data, &sig1, &keypair2.public_key).unwrap());
        assert!(!fixture.crypto.verify(test_data, &sig2, &keypair1.public_key).unwrap());
    }

    #[test]
    fn test_large_data_integration() {
        let fixture = UnifiedCryptoTestFixture::new();
        
        // Generate keypair
        let keypair = fixture.crypto.generate_keypair()
            .expect("Keypair generation should succeed");
        
        // Test with 1MB of data
        let large_data = vec![42u8; 1024 * 1024];
        
        // Test encryption/decryption of large data
        let encrypted = fixture.crypto.encrypt(&large_data, &keypair.public_key)
            .expect("Large data encryption should succeed");
        
        let decrypted = fixture.crypto.decrypt(&encrypted, &keypair.private_key)
            .expect("Large data decryption should succeed");
        
        assert_eq!(large_data, decrypted);
        
        // Test hashing of large data
        let hash = fixture.crypto.hash(&large_data, HashAlgorithm::Sha256)
            .expect("Large data hashing should succeed");
        assert_eq!(hash.len(), 32); // SHA-256 produces 32-byte hash
        
        // Test signing of large data
        let signature = fixture.crypto.sign(&large_data, &keypair.private_key)
            .expect("Large data signing should succeed");
        
        let valid = fixture.crypto.verify(&large_data, &signature, &keypair.public_key)
            .expect("Large data verification should succeed");
        assert!(valid);
    }

    #[test]
    fn test_concurrent_operations_integration() {
        use std::sync::Arc;
        use std::thread;
        
        let fixture = Arc::new(UnifiedCryptoTestFixture::new());
        let mut handles = vec![];
        
        // Spawn multiple threads performing crypto operations
        for i in 0..10 {
            let fixture_clone = Arc::clone(&fixture);
            let handle = thread::spawn(move || {
                let thread_data = format!("Thread {} test data", i).into_bytes();
                
                // Generate keypair
                let keypair = fixture_clone.crypto.generate_keypair()
                    .expect("Thread keypair generation should succeed");
                
                // Test encryption/decryption
                let encrypted = fixture_clone.crypto.encrypt(&thread_data, &keypair.public_key)
                    .expect("Thread encryption should succeed");
                let decrypted = fixture_clone.crypto.decrypt(&encrypted, &keypair.private_key)
                    .expect("Thread decryption should succeed");
                assert_eq!(thread_data, decrypted);
                
                // Test signing/verification
                let signature = fixture_clone.crypto.sign(&thread_data, &keypair.private_key)
                    .expect("Thread signing should succeed");
                let valid = fixture_clone.crypto.verify(&thread_data, &signature, &keypair.public_key)
                    .expect("Thread verification should succeed");
                assert!(valid);
                
                // Test hashing
                let hash = fixture_clone.crypto.hash(&thread_data, HashAlgorithm::Sha256)
                    .expect("Thread hashing should succeed");
                assert_eq!(hash.len(), 32);
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("Thread should complete successfully");
        }
    }

    #[test]
    fn test_error_conditions_integration() {
        let fixture = UnifiedCryptoTestFixture::new();
        
        // Test encryption with empty data
        let keypair = fixture.crypto.generate_keypair()
            .expect("Keypair generation should succeed");
        
        let empty_data = b"";
        let result = fixture.crypto.encrypt(empty_data, &keypair.public_key);
        assert!(result.is_err(), "Encryption of empty data should fail");
        
        // Test verification with wrong data
        let test_data = b"Test data for error conditions";
        let signature = fixture.crypto.sign(test_data, &keypair.private_key)
            .expect("Signing should succeed");
        
        let wrong_data = b"Wrong data";
        let valid = fixture.crypto.verify(wrong_data, &signature, &keypair.public_key)
            .expect("Verification should succeed");
        assert!(!valid, "Verification with wrong data should return false");
    }

    #[test]
    fn test_key_manager_access_integration() {
        let fixture = UnifiedCryptoTestFixture::new();
        
        // Test access to key manager
        let _key_manager = fixture.crypto.key_manager();
        
        // Test access to audit logger
        let _audit_logger = fixture.crypto.audit_logger();
        
        // Test access to config
        let config = fixture.crypto.config();
        assert!(config.audit.enabled); // Should be enabled by default in tests
    }

    #[test]
    fn test_audit_logging_integration() {
        let fixture = UnifiedCryptoTestFixture::new();
        
        // Perform operations that should be audited
        let keypair = fixture.crypto.generate_keypair()
            .expect("Keypair generation should succeed");
        
        let test_data = b"Data for audit testing";
        
        // These operations should be logged
        let _encrypted = fixture.crypto.encrypt(test_data, &keypair.public_key)
            .expect("Encryption should succeed");
        
        let _signature = fixture.crypto.sign(test_data, &keypair.private_key)
            .expect("Signing should succeed");
        
        // Audit logger should have recorded these operations
        // Note: In a full implementation, we'd verify the audit log contents
        let audit_logger = fixture.crypto.audit_logger();
        // audit_logger functionality would be tested here
    }
}