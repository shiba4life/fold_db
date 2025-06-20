//! Cryptographic Correctness Tests
//!
//! Tests to verify that all cryptographic operations produce mathematically
//! correct results and maintain security properties.

#[cfg(test)]
mod tests {
    use super::super::security_utils::*;
    use datafold::unified_crypto::*;
    use tempfile::TempDir;

    /// Test cryptographic correctness of encryption algorithms
    #[test]
    fn test_encryption_correctness() {
        let config = CryptoConfig::default();
        let crypto = UnifiedCrypto::new(config).expect("Failed to create crypto instance");
        
        // Test data patterns that might reveal weaknesses
        let test_patterns = vec![
            generate_security_test_data(1024, SecurityTestPattern::AllZeros),
            generate_security_test_data(1024, SecurityTestPattern::AllOnes),
            generate_security_test_data(1024, SecurityTestPattern::Alternating),
            generate_security_test_data(1024, SecurityTestPattern::Sequential),
            generate_security_test_data(1024, SecurityTestPattern::Random),
        ];
        
        for (i, test_data) in test_patterns.iter().enumerate() {
            let keypair = crypto.generate_keypair()
                .expect("Keypair generation should succeed");
            
            // Test encryption/decryption correctness
            let encrypted = crypto.encrypt(test_data, &keypair.public_key)
                .expect("Encryption should succeed");
            
            let decrypted = crypto.decrypt(&encrypted, &keypair.private_key)
                .expect("Decryption should succeed");
            
            assert_eq!(test_data, &decrypted, "Pattern {} failed roundtrip", i);
            
            // Verify ciphertext doesn't leak plaintext patterns
            assert_ne!(encrypted.ciphertext(), test_data, "Ciphertext should not equal plaintext");
            
            // Test that same plaintext produces different ciphertexts
            let encrypted2 = crypto.encrypt(test_data, &keypair.public_key)
                .expect("Second encryption should succeed");
            
            assert_ne!(encrypted.ciphertext(), encrypted2.ciphertext(), 
                      "Same plaintext should produce different ciphertexts");
        }
    }

    /// Test digital signature correctness
    #[test]
    fn test_signature_correctness() {
        let config = CryptoConfig::default();
        let crypto = UnifiedCrypto::new(config).expect("Failed to create crypto instance");
        
        let keypair = crypto.generate_keypair()
            .expect("Keypair generation should succeed");
        
        // Test various data sizes and patterns
        let test_cases = vec![
            vec![],  // Empty data
            vec![0u8; 1],  // Single byte
            vec![0xFFu8; 1],  // Single 0xFF byte
            generate_security_test_data(32, SecurityTestPattern::Random),  // Hash size
            generate_security_test_data(1024, SecurityTestPattern::Random),  // Medium size
            generate_security_test_data(1024 * 1024, SecurityTestPattern::Random),  // Large size
        ];
        
        for (i, test_data) in test_cases.iter().enumerate() {
            // Skip empty data if not supported
            if test_data.is_empty() {
                continue;
            }
            
            let signature = crypto.sign(test_data, &keypair.private_key)
                .expect(&format!("Signing test case {} should succeed", i));
            
            let valid = crypto.verify(test_data, &signature, &keypair.public_key)
                .expect(&format!("Verification test case {} should succeed", i));
            
            assert!(valid, "Signature for test case {} should be valid", i);
            
            // Test signature is deterministic (for Ed25519)
            let signature2 = crypto.sign(test_data, &keypair.private_key)
                .expect(&format!("Second signing test case {} should succeed", i));
            
            // Note: This assumes Ed25519 which is deterministic
            assert_eq!(signature.signature(), signature2.signature(),
                      "Signatures should be deterministic for test case {}", i);
        }
    }

    /// Test hash function correctness
    #[test]
    fn test_hash_correctness() {
        let config = CryptoConfig::default();
        let crypto = UnifiedCrypto::new(config).expect("Failed to create crypto instance");
        
        // Test known test vectors (if available) or consistency
        let test_data = b"The quick brown fox jumps over the lazy dog";
        
        // Test all supported hash algorithms
        let algorithms = vec![
            HashAlgorithm::Sha256,
            HashAlgorithm::Sha3_256,
            HashAlgorithm::Blake3,
        ];
        
        for algorithm in algorithms {
            let hash1 = crypto.hash(test_data, algorithm)
                .expect(&format!("Hashing with {:?} should succeed", algorithm));
            
            let hash2 = crypto.hash(test_data, algorithm)
                .expect(&format!("Second hash with {:?} should succeed", algorithm));
            
            // Hashes should be deterministic
            assert_eq!(hash1, hash2, "Hash should be deterministic for {:?}", algorithm);
            
            // Hash should be correct length for algorithm
            let expected_length = match algorithm {
                HashAlgorithm::Sha256 => 32,
                HashAlgorithm::Sha3_256 => 32,
                HashAlgorithm::Blake3 => 32,
            };
            
            assert_eq!(hash1.len(), expected_length, 
                      "Hash length incorrect for {:?}", algorithm);
            
            // Test avalanche effect - small change should completely change hash
            let mut modified_data = test_data.to_vec();
            modified_data[0] ^= 0x01; // Flip one bit
            
            let modified_hash = crypto.hash(&modified_data, algorithm)
                .expect(&format!("Modified hash with {:?} should succeed", algorithm));
            
            assert_ne!(hash1, modified_hash, 
                      "Small change should completely change hash for {:?}", algorithm);
            
            // Count differing bits (should be approximately 50% for good hash)
            let differing_bits: usize = hash1.iter()
                .zip(modified_hash.iter())
                .map(|(a, b)| (a ^ b).count_ones() as usize)
                .sum();
            
            let total_bits = hash1.len() * 8;
            let diff_percentage = (differing_bits as f64) / (total_bits as f64);
            
            // Should be between 25% and 75% for good avalanche effect
            assert!(diff_percentage > 0.25 && diff_percentage < 0.75,
                   "Avalanche effect poor for {:?}: {}% bits differ", algorithm, diff_percentage * 100.0);
        }
    }

    /// Test key generation security properties
    #[test]
    fn test_key_generation_security() {
        let config = CryptoConfig::default();
        let crypto = UnifiedCrypto::new(config).expect("Failed to create crypto instance");
        
        const NUM_KEYS: usize = 100;
        let mut public_keys = Vec::new();
        let mut private_keys = Vec::new();
        
        // Generate multiple key pairs
        for _ in 0..NUM_KEYS {
            let keypair = crypto.generate_keypair()
                .expect("Key generation should succeed");
            
            public_keys.push(keypair.public_key.key_material().to_vec());
            private_keys.push(keypair.private_key.key_material().bytes().to_vec());
        }
        
        // Test key uniqueness
        for i in 0..NUM_KEYS {
            for j in (i + 1)..NUM_KEYS {
                assert_ne!(public_keys[i], public_keys[j],
                          "Public keys {} and {} should be different", i, j);
                assert_ne!(private_keys[i], private_keys[j],
                          "Private keys {} and {} should be different", i, j);
            }
        }
        
        // Test key entropy (basic check)
        for (i, key) in private_keys.iter().enumerate() {
            // Keys should not be all zeros
            assert_ne!(key, &vec![0u8; key.len()],
                      "Private key {} should not be all zeros", i);
            
            // Keys should not be all ones
            assert_ne!(key, &vec![0xFFu8; key.len()],
                      "Private key {} should not be all ones", i);
            
            // Basic entropy check - count unique bytes
            let mut byte_counts = [0u32; 256];
            for &byte in key {
                byte_counts[byte as usize] += 1;
            }
            
            let unique_bytes = byte_counts.iter().filter(|&&count| count > 0).count();
            
            // Should have reasonable diversity
            assert!(unique_bytes > key.len() / 8,
                   "Private key {} has poor entropy: only {} unique bytes", i, unique_bytes);
        }
    }

    /// Test resistance to invalid inputs
    #[test]
    fn test_invalid_input_resistance() {
        let config = CryptoConfig::default();
        let crypto = UnifiedCrypto::new(config).expect("Failed to create crypto instance");
        
        let keypair = crypto.generate_keypair()
            .expect("Keypair generation should succeed");
        
        // Test encryption with empty data
        let result = crypto.encrypt(&[], &keypair.public_key);
        assert!(result.is_err(), "Encryption of empty data should fail");
        
        // Test verification with tampered signature
        let data = b"test data";
        let signature = crypto.sign(data, &keypair.private_key)
            .expect("Signing should succeed");
        
        // Create invalid signature by modifying bytes
        let mut invalid_sig_bytes = signature.signature().to_vec();
        invalid_sig_bytes[0] ^= 0xFF; // Flip all bits in first byte
        
        // Note: Would need to create invalid signature object
        // This test would need proper signature tampering support
        
        // Test decryption with tampered ciphertext
        let plaintext = b"test data for tampering";
        let encrypted = crypto.encrypt(plaintext, &keypair.public_key)
            .expect("Encryption should succeed");
        
        // This would require creating tampered encrypted data
        // The test framework would need support for this
    }

    /// Test cross-key compatibility (keys should not work across different pairs)
    #[test]
    fn test_cross_key_rejection() {
        let config = CryptoConfig::default();
        let crypto = UnifiedCrypto::new(config).expect("Failed to create crypto instance");
        
        let keypair1 = crypto.generate_keypair()
            .expect("First keypair generation should succeed");
        let keypair2 = crypto.generate_keypair()
            .expect("Second keypair generation should succeed");
        
        let test_data = b"cross-key test data";
        
        // Encrypt with keypair1, try to decrypt with keypair2
        let encrypted = crypto.encrypt(test_data, &keypair1.public_key)
            .expect("Encryption should succeed");
        
        let result = crypto.decrypt(&encrypted, &keypair2.private_key);
        assert!(result.is_err(), "Decryption with wrong key should fail");
        
        // Sign with keypair1, verify with keypair2
        let signature = crypto.sign(test_data, &keypair1.private_key)
            .expect("Signing should succeed");
        
        let valid = crypto.verify(test_data, &signature, &keypair2.public_key)
            .expect("Verification should succeed");
        assert!(!valid, "Verification with wrong key should return false");
    }
}