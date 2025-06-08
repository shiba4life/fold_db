//! Argon2id passphrase-based key derivation for DataFold

use crate::crypto::error::{CryptoError, CryptoResult};
use crate::crypto::ed25519::{generate_master_keypair_from_seed, MasterKeyPair, SECRET_KEY_LENGTH};
use argon2::{Argon2, Algorithm, Version, Params};
use rand::rngs::OsRng;
use rand::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Length of salt for Argon2 key derivation (32 bytes)
pub const SALT_LENGTH: usize = 32;

/// Default memory cost for Argon2 (64 MB)
pub const DEFAULT_MEMORY_COST: u32 = 65536; // 64 MB in KB

/// Default time cost for Argon2 (3 iterations)
pub const DEFAULT_TIME_COST: u32 = 3;

/// Default parallelism for Argon2 (4 threads)
pub const DEFAULT_PARALLELISM: u32 = 4;

/// Output length for key derivation (32 bytes for Ed25519 seed)
pub const OUTPUT_LENGTH: usize = SECRET_KEY_LENGTH;

/// Argon2 salt for key derivation
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Salt {
    bytes: [u8; SALT_LENGTH],
}

impl Salt {
    /// Create a Salt from bytes
    pub fn from_bytes(bytes: [u8; SALT_LENGTH]) -> Self {
        Self { bytes }
    }

    /// Get the salt as bytes
    pub fn as_bytes(&self) -> &[u8; SALT_LENGTH] {
        &self.bytes
    }

    /// Convert to a slice
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }
}

/// Argon2 parameters for key derivation
#[derive(Clone, Debug)]
pub struct Argon2Params {
    /// Memory cost in KB
    pub memory_cost: u32,
    /// Time cost (iterations)
    pub time_cost: u32,
    /// Parallelism degree
    pub parallelism: u32,
}

impl Default for Argon2Params {
    fn default() -> Self {
        Self {
            memory_cost: DEFAULT_MEMORY_COST,
            time_cost: DEFAULT_TIME_COST,
            parallelism: DEFAULT_PARALLELISM,
        }
    }
}

impl Argon2Params {
    /// Create new Argon2 parameters
    pub fn new(memory_cost: u32, time_cost: u32, parallelism: u32) -> CryptoResult<Self> {
        // Validate parameters according to Argon2 specification
        if memory_cost < 8 {
            return Err(CryptoError::KeyDerivation {
                message: "Memory cost must be at least 8 KB".to_string(),
            });
        }
        
        if time_cost < 1 {
            return Err(CryptoError::KeyDerivation {
                message: "Time cost must be at least 1".to_string(),
            });
        }
        
        if !(1..=16777215).contains(&parallelism) {
            return Err(CryptoError::KeyDerivation {
                message: "Parallelism must be between 1 and 16777215".to_string(),
            });
        }

        Ok(Self {
            memory_cost,
            time_cost,
            parallelism,
        })
    }

    /// Create parameters optimized for interactive use (faster)
    pub fn interactive() -> Self {
        Self {
            memory_cost: 32768, // 32 MB
            time_cost: 2,
            parallelism: 2,
        }
    }

    /// Create parameters optimized for sensitive operations (slower, more secure)
    pub fn sensitive() -> Self {
        Self {
            memory_cost: 131072, // 128 MB
            time_cost: 4,
            parallelism: 8,
        }
    }

    /// Validate parameters and convert to Argon2 Params
    fn to_argon2_params(&self) -> CryptoResult<Params> {
        Params::new(
            self.memory_cost,
            self.time_cost,
            self.parallelism,
            Some(OUTPUT_LENGTH),
        )
        .map_err(|e| CryptoError::KeyDerivation {
            message: format!("Invalid Argon2 parameters: {}", e),
        })
    }
}

/// Container for derived key material that automatically zeroizes on drop
#[derive(ZeroizeOnDrop)]
pub struct DerivedKey {
    key_bytes: [u8; OUTPUT_LENGTH],
}

impl DerivedKey {
    /// Create a DerivedKey from bytes
    fn from_bytes(bytes: [u8; OUTPUT_LENGTH]) -> Self {
        Self { key_bytes: bytes }
    }

    /// Get the derived key bytes (use with caution)
    pub fn as_bytes(&self) -> &[u8; OUTPUT_LENGTH] {
        &self.key_bytes
    }

    /// Generate an Ed25519 master key pair from this derived key
    pub fn to_master_keypair(&self) -> CryptoResult<MasterKeyPair> {
        generate_master_keypair_from_seed(&self.key_bytes)
    }
}

impl Zeroize for DerivedKey {
    fn zeroize(&mut self) {
        self.key_bytes.zeroize();
    }
}

/// Generate a cryptographically secure salt for key derivation
pub fn generate_salt() -> Salt {
    let mut salt_bytes = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt_bytes);
    Salt::from_bytes(salt_bytes)
}

/// Derive a key from a passphrase using Argon2id
pub fn derive_key(
    passphrase: &str,
    salt: &Salt,
    params: &Argon2Params,
) -> CryptoResult<DerivedKey> {
    let argon2_params = params.to_argon2_params()?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon2_params);
    
    let mut output = [0u8; OUTPUT_LENGTH];
    
    argon2
        .hash_password_into(
            passphrase.as_bytes(),
            salt.as_slice(),
            &mut output,
        )
        .map_err(|e| CryptoError::KeyDerivation {
            message: format!("Argon2 derivation failed: {}", e),
        })?;

    Ok(DerivedKey::from_bytes(output))
}

/// Derive a master key pair directly from a passphrase
pub fn derive_master_keypair(
    passphrase: &str,
    salt: &Salt,
    params: &Argon2Params,
) -> CryptoResult<MasterKeyPair> {
    let derived_key = derive_key(passphrase, salt, params)?;
    derived_key.to_master_keypair()
}

/// Convenience function to derive a master key pair with default parameters
pub fn derive_master_keypair_default(
    passphrase: &str,
    salt: &Salt,
) -> CryptoResult<MasterKeyPair> {
    derive_master_keypair(passphrase, salt, &Argon2Params::default())
}

/// Generate salt and derive master key pair in one operation
pub fn generate_salt_and_derive_keypair(
    passphrase: &str,
    params: &Argon2Params,
) -> CryptoResult<(Salt, MasterKeyPair)> {
    let salt = generate_salt();
    let keypair = derive_master_keypair(passphrase, &salt, params)?;
    Ok((salt, keypair))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_salt_generation() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        
        assert_eq!(salt1.as_bytes().len(), SALT_LENGTH);
        assert_eq!(salt2.as_bytes().len(), SALT_LENGTH);
        
        // Different salts should be different
        assert_ne!(salt1.as_bytes(), salt2.as_bytes());
    }

    #[test]
    fn test_salt_round_trip() {
        let original_bytes = [42u8; SALT_LENGTH];
        let salt = Salt::from_bytes(original_bytes);
        
        assert_eq!(salt.as_bytes(), &original_bytes);
        assert_eq!(salt.as_slice(), &original_bytes[..]);
    }

    #[test]
    fn test_argon2_params_validation() {
        // Valid parameters
        let params = Argon2Params::new(1024, 2, 2).expect("Valid parameters");
        assert_eq!(params.memory_cost, 1024);
        assert_eq!(params.time_cost, 2);
        assert_eq!(params.parallelism, 2);
        
        // Invalid memory cost
        assert!(Argon2Params::new(7, 2, 2).is_err());
        
        // Invalid time cost
        assert!(Argon2Params::new(1024, 0, 2).is_err());
        
        // Invalid parallelism
        assert!(Argon2Params::new(1024, 2, 0).is_err());
        assert!(Argon2Params::new(1024, 2, 16777216).is_err());
    }

    #[test]
    fn test_preset_parameters() {
        let interactive = Argon2Params::interactive();
        assert!(interactive.memory_cost < DEFAULT_MEMORY_COST);
        assert!(interactive.time_cost <= DEFAULT_TIME_COST);
        
        let sensitive = Argon2Params::sensitive();
        assert!(sensitive.memory_cost > DEFAULT_MEMORY_COST);
        assert!(sensitive.time_cost >= DEFAULT_TIME_COST);
    }

    #[test]
    fn test_key_derivation_consistency() {
        let passphrase = "test-passphrase-123";
        let salt = Salt::from_bytes([1u8; SALT_LENGTH]);
        let params = Argon2Params::default();
        
        let key1 = derive_key(passphrase, &salt, &params).expect("Derivation 1");
        let key2 = derive_key(passphrase, &salt, &params).expect("Derivation 2");
        
        assert_eq!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_different_salts_produce_different_keys() {
        let passphrase = "test-passphrase-123";
        let salt1 = Salt::from_bytes([1u8; SALT_LENGTH]);
        let salt2 = Salt::from_bytes([2u8; SALT_LENGTH]);
        let params = Argon2Params::default();
        
        let key1 = derive_key(passphrase, &salt1, &params).expect("Derivation 1");
        let key2 = derive_key(passphrase, &salt2, &params).expect("Derivation 2");
        
        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_different_passphrases_produce_different_keys() {
        let salt = Salt::from_bytes([1u8; SALT_LENGTH]);
        let params = Argon2Params::default();
        
        let key1 = derive_key("passphrase1", &salt, &params).expect("Derivation 1");
        let key2 = derive_key("passphrase2", &salt, &params).expect("Derivation 2");
        
        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_master_keypair_derivation() {
        let passphrase = "test-passphrase-for-keypair";
        let salt = Salt::from_bytes([3u8; SALT_LENGTH]);
        let params = Argon2Params::default();
        
        let keypair = derive_master_keypair(passphrase, &salt, &params)
            .expect("Failed to derive master keypair");
        
        // Test that the keypair can sign and verify
        let message = b"test message";
        let signature = keypair.sign_data(message).expect("Failed to sign");
        keypair.verify_data(message, &signature).expect("Failed to verify");
    }

    #[test]
    fn test_derived_key_to_keypair() {
        let passphrase = "test-passphrase-key-conversion";
        let salt = Salt::from_bytes([4u8; SALT_LENGTH]);
        let params = Argon2Params::default();
        
        let derived_key = derive_key(passphrase, &salt, &params)
            .expect("Failed to derive key");
        
        let keypair = derived_key.to_master_keypair()
            .expect("Failed to convert to keypair");
        
        // Test that the keypair works
        let message = b"test message for keypair conversion";
        let signature = keypair.sign_data(message).expect("Failed to sign");
        keypair.verify_data(message, &signature).expect("Failed to verify");
    }

    #[test]
    fn test_default_derivation() {
        let passphrase = "test-default-derivation";
        let salt = Salt::from_bytes([5u8; SALT_LENGTH]);
        
        let keypair = derive_master_keypair_default(passphrase, &salt)
            .expect("Failed to derive with defaults");
        
        // Test that it works
        let message = b"test default derivation";
        let signature = keypair.sign_data(message).expect("Failed to sign");
        keypair.verify_data(message, &signature).expect("Failed to verify");
    }

    #[test]
    fn test_generate_salt_and_derive() {
        let passphrase = "test-generate-and-derive";
        let params = Argon2Params::interactive();
        
        let (salt, keypair) = generate_salt_and_derive_keypair(passphrase, &params)
            .expect("Failed to generate and derive");
        
        assert_eq!(salt.as_bytes().len(), SALT_LENGTH);
        
        // Test that the keypair works
        let message = b"test generate and derive";
        let signature = keypair.sign_data(message).expect("Failed to sign");
        keypair.verify_data(message, &signature).expect("Failed to verify");
    }

    #[test]
    fn test_derived_key_consistency_through_keypair() {
        let passphrase = "consistency-test-passphrase";
        let salt = Salt::from_bytes([6u8; SALT_LENGTH]);
        let params = Argon2Params::default();
        
        // Derive keypair directly
        let keypair1 = derive_master_keypair(passphrase, &salt, &params)
            .expect("Failed direct derivation");
        
        // Derive key then convert to keypair
        let derived_key = derive_key(passphrase, &salt, &params)
            .expect("Failed key derivation");
        let keypair2 = derived_key.to_master_keypair()
            .expect("Failed key to keypair conversion");
        
        // Both should produce the same public key
        assert_eq!(keypair1.public_key_bytes(), keypair2.public_key_bytes());
        assert_eq!(keypair1.secret_key_bytes(), keypair2.secret_key_bytes());
    }

    #[test]
    fn test_performance_reasonable() {
        let passphrase = "performance-test-passphrase";
        let salt = generate_salt();
        let params = Argon2Params::interactive(); // Faster parameters
        
        let start = std::time::Instant::now();
        let _keypair = derive_master_keypair(passphrase, &salt, &params)
            .expect("Failed performance test");
        let duration = start.elapsed();
        
        // Should complete in reasonable time (less than 5 seconds for interactive params)
        assert!(duration.as_secs() < 5, "Key derivation took too long: {:?}", duration);
    }
} 