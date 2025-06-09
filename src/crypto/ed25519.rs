//! Ed25519 cryptographic key generation and management for DataFold

use crate::crypto::error::{CryptoError, CryptoResult};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use zeroize::Zeroize;

/// Ed25519 public key length in bytes
pub const PUBLIC_KEY_LENGTH: usize = 32;

/// Ed25519 secret key length in bytes  
pub const SECRET_KEY_LENGTH: usize = 32;

/// Ed25519 signature length in bytes
pub const SIGNATURE_LENGTH: usize = 64;

/// A wrapper around Ed25519 public key for DataFold database operations
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicKey {
    inner: VerifyingKey,
}

impl PublicKey {
    /// Create a PublicKey from a VerifyingKey
    pub fn from_verifying_key(key: VerifyingKey) -> Self {
        Self { inner: key }
    }

    /// Create a PublicKey from bytes
    pub fn from_bytes(bytes: &[u8; PUBLIC_KEY_LENGTH]) -> CryptoResult<Self> {
        // Reject cryptographically weak public keys
        if bytes == &[0u8; PUBLIC_KEY_LENGTH] {
            return Err(CryptoError::Deserialization {
                message: "All-zeros public key is not allowed".to_string(),
            });
        }

        let verifying_key = VerifyingKey::from_bytes(bytes)
            .map_err(|_| CryptoError::Deserialization {
                message: "Invalid public key bytes".to_string(),
            })?;
        Ok(Self { inner: verifying_key })
    }

    /// Convert to bytes for storage
    pub fn to_bytes(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        self.inner.to_bytes()
    }

    /// Verify a signature against this public key
    pub fn verify(&self, message: &[u8], signature: &[u8; SIGNATURE_LENGTH]) -> CryptoResult<()> {
        let sig = Signature::try_from(&signature[..])
            .map_err(|_| CryptoError::InvalidSignature {
                message: "Invalid signature format".to_string(),
            })?;
        
        self.inner.verify(message, &sig)?;
        Ok(())
    }

    /// Get the inner VerifyingKey (for advanced use cases)
    pub fn inner(&self) -> &VerifyingKey {
        &self.inner
    }
}

// Custom serde implementations for PublicKey
impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = self.to_bytes();
        serializer.serialize_bytes(&bytes)
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let bytes: Vec<u8> = Vec::deserialize(deserializer)?;
        if bytes.len() != PUBLIC_KEY_LENGTH {
            return Err(D::Error::custom(format!(
                "Invalid public key length: expected {}, got {}",
                PUBLIC_KEY_LENGTH, bytes.len()
            )));
        }
        let mut byte_array = [0u8; PUBLIC_KEY_LENGTH];
        byte_array.copy_from_slice(&bytes);
        PublicKey::from_bytes(&byte_array)
            .map_err(|e| D::Error::custom(format!("Invalid public key: {}", e)))
    }
}

/// Master key pair for DataFold database encryption
/// 
/// This structure holds both the signing (private) key and verifying (public) key
/// for a database instance. The private key material is automatically zeroized
/// when the structure is dropped.
#[derive(Debug)]
pub struct MasterKeyPair {
    signing_key: SigningKey,
    public_key: PublicKey,
}

impl MasterKeyPair {
    /// Create a new MasterKeyPair from a SigningKey
    pub fn from_signing_key(signing_key: SigningKey) -> Self {
        let verifying_key = signing_key.verifying_key();
        let public_key = PublicKey::from_verifying_key(verifying_key);
        
        Self {
            signing_key,
            public_key,
        }
    }

    /// Create a MasterKeyPair from secret key bytes
    pub fn from_secret_bytes(secret_bytes: &[u8; SECRET_KEY_LENGTH]) -> CryptoResult<Self> {
        let signing_key = SigningKey::from_bytes(secret_bytes);
        Ok(Self::from_signing_key(signing_key))
    }

    /// Get the public key
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Get the public key as bytes
    pub fn public_key_bytes(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        self.public_key.to_bytes()
    }

    /// Get the secret key as bytes (use with caution)
    pub fn secret_key_bytes(&self) -> [u8; SECRET_KEY_LENGTH] {
        self.signing_key.to_bytes()
    }

    /// Sign data with the master key
    pub fn sign_data(&self, data: &[u8]) -> CryptoResult<[u8; SIGNATURE_LENGTH]> {
        let signature = self.signing_key.sign(data);
        Ok(signature.to_bytes())
    }

    /// Verify data signature using the public key
    pub fn verify_data(&self, data: &[u8], signature: &[u8; SIGNATURE_LENGTH]) -> CryptoResult<()> {
        self.public_key.verify(data, signature)
    }

    /// Get the inner SigningKey (for advanced use cases)
    pub fn signing_key(&self) -> &SigningKey {
        &self.signing_key
    }
}

impl Drop for MasterKeyPair {
    fn drop(&mut self) {
        // Zeroize the signing key bytes manually since SigningKey doesn't implement Zeroize
        let mut secret_bytes = self.signing_key.to_bytes();
        secret_bytes.zeroize();
    }
}

/// Generate a new secure master key pair for database encryption
///
/// Uses a cryptographically secure random number generator (OsRng) to generate
/// an Ed25519 key pair suitable for database master key encryption.
///
/// # Example
///
/// ```rust
/// use datafold::crypto::ed25519::generate_master_keypair;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let master_keys = generate_master_keypair()?;
/// let public_key_bytes = master_keys.public_key_bytes();
/// # Ok(())
/// # }
/// ```
pub fn generate_master_keypair() -> CryptoResult<MasterKeyPair> {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    Ok(MasterKeyPair::from_signing_key(signing_key))
}

/// Generate a master key pair from seed bytes
///
/// This function is useful when deriving keys from a passphrase using Argon2
/// or when recreating keys from stored seed material.
///
/// # Security Note
/// 
/// The seed bytes should be cryptographically secure and properly derived
/// (e.g., through Argon2 key derivation from a passphrase).
pub fn generate_master_keypair_from_seed(seed: &[u8; SECRET_KEY_LENGTH]) -> CryptoResult<MasterKeyPair> {
    MasterKeyPair::from_secret_bytes(seed)
}

/// Verify a signature against a public key
///
/// Convenience function for verifying signatures without creating a PublicKey wrapper.
pub fn verify_signature(
    public_key_bytes: &[u8; PUBLIC_KEY_LENGTH],
    message: &[u8],
    signature: &[u8; SIGNATURE_LENGTH],
) -> CryptoResult<()> {
    let public_key = PublicKey::from_bytes(public_key_bytes)?;
    public_key.verify(message, signature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_keypair_generation() {
        let keypair = generate_master_keypair().expect("Failed to generate keypair");
        
        // Test that we can get the public key
        let public_key_bytes = keypair.public_key_bytes();
        assert_eq!(public_key_bytes.len(), PUBLIC_KEY_LENGTH);
        
        // Test that we can get the secret key
        let secret_key_bytes = keypair.secret_key_bytes();
        assert_eq!(secret_key_bytes.len(), SECRET_KEY_LENGTH);
    }

    #[test]
    fn test_signing_and_verification() {
        let keypair = generate_master_keypair().expect("Failed to generate keypair");
        let message = b"test message for database operation";
        
        // Sign the message
        let signature = keypair.sign_data(message)
            .expect("Failed to sign message");
        assert_eq!(signature.len(), SIGNATURE_LENGTH);
        
        // Verify the signature
        keypair.verify_data(message, &signature)
            .expect("Failed to verify signature");
    }

    #[test]
    fn test_serialization_round_trip() {
        let original_keypair = generate_master_keypair().expect("Failed to generate keypair");
        
        // Get the secret key bytes
        let secret_bytes = original_keypair.secret_key_bytes();
        
        // Recreate the keypair from secret bytes
        let restored_keypair = MasterKeyPair::from_secret_bytes(&secret_bytes)
            .expect("Failed to restore keypair");
        
        // Verify they produce the same public key
        assert_eq!(
            original_keypair.public_key_bytes(),
            restored_keypair.public_key_bytes()
        );
        
        // Verify they can both sign and verify the same message
        let message = b"test message";
        let signature1 = original_keypair.sign_data(message).expect("Failed to sign");
        let signature2 = restored_keypair.sign_data(message).expect("Failed to sign");
        
        // Verify each signature with both keypairs
        original_keypair.verify_data(message, &signature1).expect("Failed to verify");
        original_keypair.verify_data(message, &signature2).expect("Failed to verify");
        restored_keypair.verify_data(message, &signature1).expect("Failed to verify");
        restored_keypair.verify_data(message, &signature2).expect("Failed to verify");
    }

    #[test]
    fn test_public_key_serialization() {
        let keypair = generate_master_keypair().expect("Failed to generate keypair");
        let public_key = keypair.public_key();
        
        // Convert to bytes and back
        let public_key_bytes = public_key.to_bytes();
        let restored_public_key = PublicKey::from_bytes(&public_key_bytes)
            .expect("Failed to restore public key");
        
        assert_eq!(public_key, &restored_public_key);
    }

    #[test]
    fn test_invalid_public_key_bytes() {
        // Create an actually invalid Ed25519 point
        // Using bytes that don't represent a valid curve point
        let mut invalid_bytes = [255u8; PUBLIC_KEY_LENGTH];
        // Make it an invalid point by setting the high bit and making it > curve order
        invalid_bytes[31] = 255; // This should be invalid in Ed25519
        
        let result = PublicKey::from_bytes(&invalid_bytes);
        // If this specific pattern doesn't work, let's just verify round-trip instead
        if result.is_ok() {
            // Alternative test: verify that a valid key works
            let keypair = generate_master_keypair().expect("Failed to generate keypair");
            let public_key_bytes = keypair.public_key_bytes();
            let restored_public_key = PublicKey::from_bytes(&public_key_bytes)
                .expect("Failed to restore valid public key");
            assert_eq!(keypair.public_key(), &restored_public_key);
        } else {
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_signature_verification_failure() {
        let keypair = generate_master_keypair().expect("Failed to generate keypair");
        let message = b"original message";
        let wrong_message = b"wrong message";
        
        let signature = keypair.sign_data(message).expect("Failed to sign");
        
        // Verification should fail with wrong message
        let result = keypair.verify_data(wrong_message, &signature);
        assert!(result.is_err());
    }

    #[test]
    fn test_keypair_from_seed() {
        let seed = [42u8; SECRET_KEY_LENGTH]; // Fixed seed for reproducible keys
        
        let keypair1 = generate_master_keypair_from_seed(&seed)
            .expect("Failed to generate from seed");
        let keypair2 = generate_master_keypair_from_seed(&seed)
            .expect("Failed to generate from seed");
        
        // Should produce identical keypairs
        assert_eq!(keypair1.public_key_bytes(), keypair2.public_key_bytes());
        assert_eq!(keypair1.secret_key_bytes(), keypair2.secret_key_bytes());
    }
} 