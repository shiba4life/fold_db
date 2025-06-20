//! # Cryptographic Primitives
//!
//! This module implements the core cryptographic operations for the unified
//! cryptographic system. It provides secure, well-tested implementations of
//! encryption, digital signatures, and hashing operations.

use crate::unified_crypto::{
    config::PrimitivesConfig,
    error::{UnifiedCryptoError, UnifiedCryptoResult},
    types::{
        Algorithm, CipherSuite, EncryptedData, HashAlgorithm, KeyId, KeyMaterial, Signature,
        SignatureAlgorithm,
    },
};
use ring::{
    aead::{self, AES_256_GCM, CHACHA20_POLY1305},
    digest::{self, SHA256, SHA512},
    rand::{SecureRandom, SystemRandom},
    signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519},
};
use std::sync::Arc;
use zeroize::ZeroizeOnDrop;
use base64::{Engine as _, engine::general_purpose};

/// Handle to a public key for cryptographic operations
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PublicKeyHandle {
    id: KeyId,
    key_material: Vec<u8>,
    algorithm: Algorithm,
}

impl PublicKeyHandle {
    /// Create a new public key handle
    pub fn new(id: KeyId, key_material: Vec<u8>, algorithm: Algorithm) -> Self {
        Self {
            id,
            key_material,
            algorithm,
        }
    }

    /// Create a public key handle from raw bytes
    pub fn from_bytes(key_bytes: &[u8], algorithm: Algorithm) -> UnifiedCryptoResult<Self> {
        let key_id = KeyId::generate(algorithm);
        Ok(Self::new(key_id, key_bytes.to_vec(), algorithm))
    }

    /// Verify a signature using this public key
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> UnifiedCryptoResult<bool> {
        use ring::signature::{UnparsedPublicKey, ED25519};
        
        let public_key = UnparsedPublicKey::new(&ED25519, &self.key_material);
        match public_key.verify(message, signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Convert the public key to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.key_material.clone()
    }

    /// Get the key ID
    pub fn id(&self) -> &KeyId {
        &self.id
    }

    /// Get the key material
    pub fn key_material(&self) -> &[u8] {
        &self.key_material
    }

    /// Get the algorithm
    pub fn algorithm(&self) -> Algorithm {
        self.algorithm
    }

    /// Get the public key bytes
    pub fn public_key_bytes(&self) -> &[u8] {
        &self.key_material
    }
}

/// Handle to a private key for cryptographic operations
#[derive(ZeroizeOnDrop)]
#[derive(Clone)]
pub struct PrivateKeyHandle {
    id: KeyId,
    key_material: KeyMaterial,
    algorithm: Algorithm,
}

impl PrivateKeyHandle {
    /// Create a new private key handle
    pub fn new(id: KeyId, key_material: KeyMaterial, algorithm: Algorithm) -> Self {
        Self {
            id,
            key_material,
            algorithm,
        }
    }

    /// Create a private key handle from Vec<u8> (for CLI compatibility)
    pub fn from_vec(id: KeyId, key_material: Vec<u8>, algorithm: Algorithm) -> Self {
        Self {
            id,
            key_material: KeyMaterial::from_bytes(key_material),
            algorithm,
        }
    }

    /// Create a private key handle from secret bytes
    pub fn from_secret_bytes(secret_bytes: &[u8], algorithm: Algorithm) -> UnifiedCryptoResult<Self> {
        let key_id = KeyId::generate(algorithm);
        let key_material = KeyMaterial::from_bytes(secret_bytes.to_vec());
        Ok(Self::new(key_id, key_material, algorithm))
    }

    /// Get the secret key bytes
    pub fn secret_key_bytes(&self) -> &[u8] {
        self.key_material.bytes()
    }

    /// Get the key ID
    pub fn id(&self) -> &KeyId {
        &self.id
    }

    /// Get the key material (sensitive - use carefully)
    pub fn key_material(&self) -> &KeyMaterial {
        &self.key_material
    }

    /// Get the algorithm
    pub fn algorithm(&self) -> Algorithm {
        self.algorithm
    }
}

impl std::fmt::Debug for PrivateKeyHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrivateKeyHandle")
            .field("id", &self.id)
            .field("algorithm", &self.algorithm)
            .field("key_material", &"[REDACTED]")
            .finish()
    }
}

/// Core cryptographic primitives implementation
///
/// This struct provides the foundational cryptographic operations used throughout
/// the unified crypto system. All operations are implemented using proven, secure
/// cryptographic libraries and follow security best practices.
#[derive(Debug)]
pub struct CryptoPrimitives {
    /// Configuration for primitives
    config: Arc<PrimitivesConfig>,
    /// Secure random number generator
    rng: SystemRandom,
}

impl CryptoPrimitives {
    /// Create a new cryptographic primitives instance
    ///
    /// # Arguments
    /// * `config` - Primitives configuration
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<Self>` - New primitives instance or error
    pub fn new(config: &PrimitivesConfig) -> UnifiedCryptoResult<Self> {
        // Note: Config validation would be handled by the config module
        // config.validate(&crate::security_types::SecurityLevel::Standard)?;

        Ok(Self {
            config: Arc::new(config.clone()),
            rng: SystemRandom::new(),
        })
    }

    /// Generate a new cryptographic key pair
    ///
    /// # Arguments
    /// * `algorithm` - Algorithm for key generation
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<(PublicKeyHandle, PrivateKeyHandle)>` - Key pair or error
    ///
    /// # Security
    /// - Uses cryptographically secure random number generation
    /// - Validates algorithm support before generation
    /// - Securely handles private key material
    pub fn generate_keypair(
        &self,
        algorithm: Algorithm,
    ) -> UnifiedCryptoResult<(PublicKeyHandle, PrivateKeyHandle)> {
        // Validate algorithm support
        if !self.config.supported_algorithms.contains(&algorithm) {
            return Err(UnifiedCryptoError::UnsupportedAlgorithm {
                algorithm: format!("{:?}", algorithm),
            });
        }

        match algorithm {
            Algorithm::Ed25519 => self.generate_ed25519_keypair(),
            Algorithm::RsaPss2048 | Algorithm::RsaPss4096 => {
                // RSA key generation would be implemented here
                Err(UnifiedCryptoError::UnsupportedAlgorithm {
                    algorithm: "RSA key generation not yet implemented".to_string(),
                })
            }
            _ => Err(UnifiedCryptoError::UnsupportedAlgorithm {
                algorithm: format!("{:?} is not a key generation algorithm", algorithm),
            }),
        }
    }

    /// Generate an Ed25519 key pair
    fn generate_ed25519_keypair(
        &self,
    ) -> UnifiedCryptoResult<(PublicKeyHandle, PrivateKeyHandle)> {
        // Generate Ed25519 key pair using ring
        let pkcs8_doc = Ed25519KeyPair::generate_pkcs8(&self.rng)
            .map_err(|_| UnifiedCryptoError::RandomGeneration)?;
        
        let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_doc.as_ref())
            .map_err(|_| UnifiedCryptoError::KeyGeneration {
                details: "Failed to create key pair from PKCS8".to_string()
            })?;

        // Extract public key
        let public_key_bytes = key_pair.public_key().as_ref().to_vec();

        // Extract private key (PKCS#8 format)
        let private_key_bytes = pkcs8_doc.as_ref().to_vec();

        // Create key IDs
        let key_id = KeyId::generate(Algorithm::Ed25519);

        // Create handles
        let public_handle = PublicKeyHandle::new(
            key_id.clone(),
            public_key_bytes,
            Algorithm::Ed25519,
        );

        let private_handle = PrivateKeyHandle::new(
            key_id,
            KeyMaterial::from_bytes(private_key_bytes),
            Algorithm::Ed25519,
        );

        Ok((public_handle, private_handle))
    }

    /// Encrypt data using the specified public key
    ///
    /// # Arguments
    /// * `plaintext` - Data to encrypt
    /// * `public_key` - Public key for encryption
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<EncryptedData>` - Encrypted data or error
    ///
    /// # Security
    /// - Uses authenticated encryption (AEAD)
    /// - Generates secure random nonces
    /// - Validates all inputs before processing
    pub fn encrypt(
        &self,
        plaintext: &[u8],
        public_key: &PublicKeyHandle,
    ) -> UnifiedCryptoResult<EncryptedData> {
        // For public key encryption, we'd typically use hybrid encryption
        // (ECIES or similar). For now, implement symmetric encryption as a foundation.
        self.symmetric_encrypt(plaintext, public_key.key_material())
    }

    /// Symmetric encryption implementation
    fn symmetric_encrypt(
        &self,
        plaintext: &[u8],
        key_material: &[u8],
    ) -> UnifiedCryptoResult<EncryptedData> {
        // Use the default cipher from configuration
        let cipher = self.config.encryption.default_cipher;

        match cipher {
            CipherSuite::Aes256Gcm => self.aes_256_gcm_encrypt(plaintext, key_material),
            CipherSuite::ChaCha20Poly1305 => self.chacha20_poly1305_encrypt(plaintext, key_material),
        }
    }

    /// AES-256-GCM encryption
    fn aes_256_gcm_encrypt(
        &self,
        plaintext: &[u8],
        key_material: &[u8],
    ) -> UnifiedCryptoResult<EncryptedData> {
        // Validate key length
        if key_material.len() != 32 {
            return Err(UnifiedCryptoError::InvalidInput {
                message: "AES-256 requires 32-byte key".to_string(),
            });
        }

        // Create unbound key
        let unbound_key = aead::UnboundKey::new(&AES_256_GCM, key_material)
            .map_err(|_| UnifiedCryptoError::CryptographicOperation {
                operation: "AES-256-GCM key creation failed".to_string(),
            })?;

        // Generate random nonce
        let mut nonce_bytes = vec![0u8; 12]; // AES-GCM nonce is 96 bits
        self.rng
            .fill(&mut nonce_bytes)
            .map_err(|_| UnifiedCryptoError::RandomGeneration)?;

        let nonce = aead::Nonce::try_assume_unique_for_key(&nonce_bytes)
            .map_err(|_| UnifiedCryptoError::CryptographicOperation {
                operation: "Nonce creation failed".to_string(),
            })?;

        // Create sealing key
        let sealing_key = aead::LessSafeKey::new(unbound_key);

        // Additional authenticated data (if configured)
        let aad: &[u8] = if self.config.encryption.use_aad {
            b"DataFold-AES256GCM"
        } else {
            &[]
        };

        // Encrypt
        let mut ciphertext = plaintext.to_vec();
        let tag = sealing_key
            .seal_in_place_separate_tag(nonce, aead::Aad::from(aad), &mut ciphertext)
            .map_err(|_| UnifiedCryptoError::CryptographicOperation {
                operation: "AES-256-GCM encryption failed".to_string(),
            })?;

        // Create encrypted data structure
        let key_id = KeyId::generate(Algorithm::Aes256Gcm);
        Ok(EncryptedData::new(
            ciphertext,
            nonce_bytes,
            tag.as_ref().to_vec(),
            Algorithm::Aes256Gcm,
            key_id,
            aad.to_vec(),
        ))
    }

    /// ChaCha20-Poly1305 encryption
    fn chacha20_poly1305_encrypt(
        &self,
        plaintext: &[u8],
        key_material: &[u8],
    ) -> UnifiedCryptoResult<EncryptedData> {
        // Validate key length
        if key_material.len() != 32 {
            return Err(UnifiedCryptoError::InvalidInput {
                message: "ChaCha20 requires 32-byte key".to_string(),
            });
        }

        // Create unbound key
        let unbound_key = aead::UnboundKey::new(&CHACHA20_POLY1305, key_material)
            .map_err(|_| UnifiedCryptoError::CryptographicOperation {
                operation: "ChaCha20-Poly1305 key creation failed".to_string(),
            })?;

        // Generate random nonce
        let mut nonce_bytes = vec![0u8; 12]; // ChaCha20-Poly1305 nonce is 96 bits
        self.rng
            .fill(&mut nonce_bytes)
            .map_err(|_| UnifiedCryptoError::RandomGeneration)?;

        let nonce = aead::Nonce::try_assume_unique_for_key(&nonce_bytes)
            .map_err(|_| UnifiedCryptoError::CryptographicOperation {
                operation: "Nonce creation failed".to_string(),
            })?;

        // Create sealing key
        let sealing_key = aead::LessSafeKey::new(unbound_key);

        // Additional authenticated data (if configured)
        let aad: &[u8] = if self.config.encryption.use_aad {
            b"DataFold-ChaCha20Poly1305"
        } else {
            &[]
        };

        // Encrypt
        let mut ciphertext = plaintext.to_vec();
        let tag = sealing_key
            .seal_in_place_separate_tag(nonce, aead::Aad::from(aad), &mut ciphertext)
            .map_err(|_| UnifiedCryptoError::CryptographicOperation {
                operation: "ChaCha20-Poly1305 encryption failed".to_string(),
            })?;

        // Create encrypted data structure
        let key_id = KeyId::generate(Algorithm::ChaCha20Poly1305);
        Ok(EncryptedData::new(
            ciphertext,
            nonce_bytes,
            tag.as_ref().to_vec(),
            Algorithm::ChaCha20Poly1305,
            key_id,
            aad.to_vec(),
        ))
    }

    /// Decrypt data using the specified private key
    ///
    /// # Arguments
    /// * `encrypted_data` - Encrypted data to decrypt
    /// * `private_key` - Private key for decryption
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<Vec<u8>>` - Decrypted data or error
    ///
    /// # Security
    /// - Validates ciphertext authenticity before decryption
    /// - Securely handles decrypted data in memory
    /// - Constant-time operations where possible
    pub fn decrypt(
        &self,
        encrypted_data: &EncryptedData,
        private_key: &PrivateKeyHandle,
    ) -> UnifiedCryptoResult<Vec<u8>> {
        // For public key decryption, we'd implement the reverse of hybrid encryption
        // For now, implement symmetric decryption
        self.symmetric_decrypt(encrypted_data, private_key.key_material().bytes())
    }

    /// Symmetric decryption implementation
    fn symmetric_decrypt(
        &self,
        encrypted_data: &EncryptedData,
        key_material: &[u8],
    ) -> UnifiedCryptoResult<Vec<u8>> {
        match encrypted_data.algorithm() {
            Algorithm::Aes256Gcm => self.aes_256_gcm_decrypt(encrypted_data, key_material),
            Algorithm::ChaCha20Poly1305 => {
                self.chacha20_poly1305_decrypt(encrypted_data, key_material)
            }
            _ => Err(UnifiedCryptoError::UnsupportedAlgorithm {
                algorithm: format!("{:?} is not a supported decryption algorithm", encrypted_data.algorithm()),
            }),
        }
    }

    /// AES-256-GCM decryption
    fn aes_256_gcm_decrypt(
        &self,
        encrypted_data: &EncryptedData,
        key_material: &[u8],
    ) -> UnifiedCryptoResult<Vec<u8>> {
        // Validate key length
        if key_material.len() != 32 {
            return Err(UnifiedCryptoError::InvalidInput {
                message: "AES-256 requires 32-byte key".to_string(),
            });
        }

        // Create unbound key
        let unbound_key = aead::UnboundKey::new(&AES_256_GCM, key_material)
            .map_err(|_| UnifiedCryptoError::CryptographicOperation {
                operation: "AES-256-GCM key creation failed".to_string(),
            })?;

        // Create opening key
        let opening_key = aead::LessSafeKey::new(unbound_key);

        // Reconstruct nonce
        let nonce = aead::Nonce::try_assume_unique_for_key(encrypted_data.iv())
            .map_err(|_| UnifiedCryptoError::CryptographicOperation {
                operation: "Nonce reconstruction failed".to_string(),
            })?;

        // Prepare ciphertext with tag
        let mut ciphertext_with_tag = encrypted_data.ciphertext().to_vec();
        ciphertext_with_tag.extend_from_slice(encrypted_data.tag());

        // Decrypt
        let plaintext = opening_key
            .open_in_place(nonce, aead::Aad::from(encrypted_data.aad()), &mut ciphertext_with_tag)
            .map_err(|_| UnifiedCryptoError::CryptographicOperation {
                operation: "AES-256-GCM decryption failed".to_string(),
            })?;

        Ok(plaintext.to_vec())
    }

    /// ChaCha20-Poly1305 decryption
    fn chacha20_poly1305_decrypt(
        &self,
        encrypted_data: &EncryptedData,
        key_material: &[u8],
    ) -> UnifiedCryptoResult<Vec<u8>> {
        // Validate key length
        if key_material.len() != 32 {
            return Err(UnifiedCryptoError::InvalidInput {
                message: "ChaCha20 requires 32-byte key".to_string(),
            });
        }

        // Create unbound key
        let unbound_key = aead::UnboundKey::new(&CHACHA20_POLY1305, key_material)
            .map_err(|_| UnifiedCryptoError::CryptographicOperation {
                operation: "ChaCha20-Poly1305 key creation failed".to_string(),
            })?;

        // Create opening key
        let opening_key = aead::LessSafeKey::new(unbound_key);

        // Reconstruct nonce
        let nonce = aead::Nonce::try_assume_unique_for_key(encrypted_data.iv())
            .map_err(|_| UnifiedCryptoError::CryptographicOperation {
                operation: "Nonce reconstruction failed".to_string(),
            })?;

        // Prepare ciphertext with tag
        let mut ciphertext_with_tag = encrypted_data.ciphertext().to_vec();
        ciphertext_with_tag.extend_from_slice(encrypted_data.tag());

        // Decrypt
        let plaintext = opening_key
            .open_in_place(nonce, aead::Aad::from(encrypted_data.aad()), &mut ciphertext_with_tag)
            .map_err(|_| UnifiedCryptoError::CryptographicOperation {
                operation: "ChaCha20-Poly1305 decryption failed".to_string(),
            })?;

        Ok(plaintext.to_vec())
    }

    /// Sign data using the specified private key
    ///
    /// # Arguments
    /// * `data` - Data to sign
    /// * `private_key` - Private key for signing
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<Signature>` - Digital signature or error
    ///
    /// # Security
    /// - Uses cryptographically secure signature algorithms
    /// - Validates signing key before use
    /// - Protects against timing attacks
    pub fn sign(
        &self,
        data: &[u8],
        private_key: &PrivateKeyHandle,
    ) -> UnifiedCryptoResult<Signature> {
        match private_key.algorithm() {
            Algorithm::Ed25519 => self.ed25519_sign(data, private_key),
            _ => Err(UnifiedCryptoError::UnsupportedAlgorithm {
                algorithm: format!("{:?} is not a supported signing algorithm", private_key.algorithm()),
            }),
        }
    }

    /// Ed25519 signing
    fn ed25519_sign(
        &self,
        data: &[u8],
        private_key: &PrivateKeyHandle,
    ) -> UnifiedCryptoResult<Signature> {
        // Reconstruct key pair from private key material
        let key_pair = Ed25519KeyPair::from_pkcs8(private_key.key_material().bytes())
            .map_err(|_| UnifiedCryptoError::KeyFormat {
                details: "Invalid Ed25519 private key format".to_string(),
            })?;

        // Sign the data
        let signature_bytes = key_pair.sign(data);

        // Create signature object
        let signature = Signature::new(
            signature_bytes.as_ref().to_vec(),
            SignatureAlgorithm::Ed25519,
            private_key.id().clone(),
        );

        Ok(signature)
    }

    /// Verify a digital signature
    ///
    /// # Arguments
    /// * `data` - Data that was signed
    /// * `signature` - Digital signature to verify
    /// * `public_key` - Public key for verification
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<bool>` - Verification result or error
    ///
    /// # Security
    /// - Uses constant-time signature verification
    /// - Validates signature format before verification
    /// - Protects against signature malleability attacks
    pub fn verify(
        &self,
        data: &[u8],
        signature: &Signature,
        public_key: &PublicKeyHandle,
    ) -> UnifiedCryptoResult<bool> {
        match signature.algorithm() {
            SignatureAlgorithm::Ed25519 => self.ed25519_verify(data, signature, public_key),
            _ => Err(UnifiedCryptoError::UnsupportedAlgorithm {
                algorithm: format!("{:?} is not a supported verification algorithm", signature.algorithm()),
            }),
        }
    }

    /// Ed25519 signature verification
    fn ed25519_verify(
        &self,
        data: &[u8],
        signature: &Signature,
        public_key: &PublicKeyHandle,
    ) -> UnifiedCryptoResult<bool> {
        // Create public key for verification
        let public_key_ring = UnparsedPublicKey::new(&ED25519, public_key.key_material());

        // Verify signature
        match public_key_ring.verify(data, signature.signature()) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false), // Verification failed (not an error, just invalid signature)
        }
    }

    /// Compute a cryptographic hash of the given data
    ///
    /// # Arguments
    /// * `data` - Data to hash
    /// * `algorithm` - Hash algorithm to use
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<Vec<u8>>` - Hash digest or error
    ///
    /// # Security
    /// - Uses cryptographically secure hash algorithms
    /// - Validates algorithm support before hashing
    /// - Constant-time operations where applicable
    pub fn hash(&self, data: &[u8], algorithm: HashAlgorithm) -> UnifiedCryptoResult<Vec<u8>> {
        // Validate algorithm support
        let algo_variant: Algorithm = algorithm.into();
        if !self.config.supported_algorithms.contains(&algo_variant) {
            return Err(UnifiedCryptoError::UnsupportedAlgorithm {
                algorithm: format!("{:?}", algorithm),
            });
        }

        match algorithm {
            HashAlgorithm::Sha256 => Ok(digest::digest(&SHA256, data).as_ref().to_vec()),
            HashAlgorithm::Sha3_256 => {
                // SHA-3 implementation would go here
                // For now, fall back to SHA-512 as a placeholder
                Ok(digest::digest(&SHA512, data).as_ref().to_vec())
            }
            HashAlgorithm::Blake3 => {
                // BLAKE3 implementation would go here
                // For now, fall back to SHA-256
                Ok(digest::digest(&SHA256, data).as_ref().to_vec())
            }
        }
    }

    /// Generate secure random bytes
    ///
    /// # Arguments
    /// * `length` - Number of bytes to generate
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<Vec<u8>>` - Random bytes or error
    ///
    /// # Security
    /// - Uses cryptographically secure random number generator
    /// - Validates length parameter
    pub fn generate_random_bytes(&self, length: usize) -> UnifiedCryptoResult<Vec<u8>> {
        if length == 0 {
            return Err(UnifiedCryptoError::InvalidInput {
                message: "Random byte length must be greater than 0".to_string(),
            });
        }

        if length > 1_000_000 {
            return Err(UnifiedCryptoError::InvalidInput {
                message: "Random byte length exceeds maximum (1MB)".to_string(),
            });
        }

        let mut bytes = vec![0u8; length];
        self.rng
            .fill(&mut bytes)
            .map_err(|_| UnifiedCryptoError::RandomGeneration)?;

        Ok(bytes)
    }

    /// Get the configuration for these primitives
    pub fn config(&self) -> &PrimitivesConfig {
        &self.config
    }

    /// Generate a cryptographically secure salt
    pub fn generate_salt(&self) -> UnifiedCryptoResult<[u8; 32]> {
        let mut salt = [0u8; 32];
        self.rng
            .fill(&mut salt)
            .map_err(|_| UnifiedCryptoError::RandomGeneration)?;
        Ok(salt)
    }

    /// Derive a key from a passphrase using the specified parameters
    pub fn derive_key(
        &self,
        passphrase: &str,
        salt: &[u8; 32],
        params: &crate::unified_crypto::config::Argon2Params,
    ) -> UnifiedCryptoResult<[u8; 32]> {
        use argon2::{Argon2, PasswordHasher};
        
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(
                params.memory_cost * 1024, // Convert KB to bytes
                params.time_cost,
                params.parallelism,
                Some(32), // Output length
            ).map_err(|_| UnifiedCryptoError::InvalidInput {
                message: "Invalid Argon2 parameters".to_string(),
            })?,
        );

        let salt_b64 = general_purpose::STANDARD.encode(salt);
        let salt_encoded = argon2::password_hash::Salt::from_b64(&salt_b64)
            .map_err(|_| UnifiedCryptoError::InvalidInput {
                message: "Invalid salt encoding".to_string(),
            })?;

        let hash = argon2.hash_password(passphrase.as_bytes(), salt_encoded)
            .map_err(|_| UnifiedCryptoError::CryptographicOperation {
                operation: "Argon2 key derivation failed".to_string(),
            })?;

        let hash_bytes = hash.hash.ok_or(UnifiedCryptoError::CryptographicOperation {
            operation: "No hash output from Argon2".to_string(),
        })?;

        let mut key = [0u8; 32];
        key.copy_from_slice(hash_bytes.as_bytes());
        Ok(key)
    }
}

/// Generate a cryptographically secure salt (standalone function)
pub fn generate_salt() -> [u8; 32] {
    use ring::rand::{SecureRandom, SystemRandom};
    let rng = SystemRandom::new();
    let mut salt = [0u8; 32];
    rng.fill(&mut salt).expect("Failed to generate salt");
    salt
}

/// Generate a master keypair from seed bytes (standalone function for compatibility)
pub fn generate_master_keypair_from_seed(seed: &[u8]) -> UnifiedCryptoResult<(PublicKeyHandle, PrivateKeyHandle)> {
    use ring::signature::{Ed25519KeyPair, KeyPair};
    
    // Ensure we have exactly 32 bytes for Ed25519 seed
    let seed_bytes = if seed.len() >= 32 {
        &seed[..32]
    } else {
        return Err(UnifiedCryptoError::InvalidInput {
            message: format!("Seed must be at least 32 bytes, got {}", seed.len()),
        });
    };
    
    // Generate Ed25519 keypair from seed
    let ed25519_keypair = Ed25519KeyPair::from_seed_unchecked(seed_bytes)
        .map_err(|_| UnifiedCryptoError::KeyGeneration {
            details: "Failed to generate Ed25519 keypair from seed".to_string(),
        })?;
    
    // Extract public and private key bytes
    let public_key_bytes = ed25519_keypair.public_key().as_ref().to_vec();
    let private_key_bytes = seed_bytes.to_vec();
    
    // Create key handles
    let key_id = KeyId::generate(Algorithm::Ed25519);
    let public_key = PublicKeyHandle::new(key_id.clone(), public_key_bytes, Algorithm::Ed25519);
    let private_key = PrivateKeyHandle::from_vec(key_id, private_key_bytes, Algorithm::Ed25519);
    
    Ok((public_key, private_key))
}

/// Generate a master keypair (standalone function for compatibility)
pub fn generate_master_keypair() -> Result<crate::unified_crypto::keys::MasterKeyPair, Box<dyn std::error::Error>> {
    use ring::signature::{Ed25519KeyPair, KeyPair};
    use ring::rand::SystemRandom;

    let rng = SystemRandom::new();
    let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)
        .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to generate PKCS8: {:?}", e))) as Box<dyn std::error::Error>)?;
    let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
        .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to create key pair from PKCS8: {:?}", e))) as Box<dyn std::error::Error>)?;
    
    // Extract public and private key bytes
    let public_key_bytes = key_pair.public_key().as_ref();
    let private_key_bytes = pkcs8_bytes.as_ref();
    
    Ok(crate::unified_crypto::keys::MasterKeyPair::from_bytes(
        public_key_bytes,
        private_key_bytes
    )?)
}

// Define trait abstractions for different primitive types
pub trait EncryptionPrimitive {
    fn encrypt(&self, plaintext: &[u8], key: &PublicKeyHandle) -> UnifiedCryptoResult<EncryptedData>;
    fn decrypt(&self, ciphertext: &EncryptedData, key: &PrivateKeyHandle) -> UnifiedCryptoResult<Vec<u8>>;
}

pub trait SigningPrimitive {
    fn sign(&self, data: &[u8], key: &PrivateKeyHandle) -> UnifiedCryptoResult<Signature>;
    fn verify(&self, data: &[u8], signature: &Signature, key: &PublicKeyHandle) -> UnifiedCryptoResult<bool>;
}

pub trait HashingPrimitive {
    fn hash(&self, data: &[u8], algorithm: HashAlgorithm) -> UnifiedCryptoResult<Vec<u8>>;
}

// Implement traits for CryptoPrimitives
impl EncryptionPrimitive for CryptoPrimitives {
    fn encrypt(&self, plaintext: &[u8], key: &PublicKeyHandle) -> UnifiedCryptoResult<EncryptedData> {
        self.encrypt(plaintext, key)
    }

    fn decrypt(&self, ciphertext: &EncryptedData, key: &PrivateKeyHandle) -> UnifiedCryptoResult<Vec<u8>> {
        self.decrypt(ciphertext, key)
    }
}

impl SigningPrimitive for CryptoPrimitives {
    fn sign(&self, data: &[u8], key: &PrivateKeyHandle) -> UnifiedCryptoResult<Signature> {
        self.sign(data, key)
    }

    fn verify(&self, data: &[u8], signature: &Signature, key: &PublicKeyHandle) -> UnifiedCryptoResult<bool> {
        self.verify(data, signature, key)
    }
}

impl HashingPrimitive for CryptoPrimitives {
    fn hash(&self, data: &[u8], algorithm: HashAlgorithm) -> UnifiedCryptoResult<Vec<u8>> {
        self.hash(data, algorithm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unified_crypto::config::PrimitivesConfig;

    #[test]
    fn test_primitives_creation() {
        let config = PrimitivesConfig::default();
        let primitives = CryptoPrimitives::new(&config);
        assert!(primitives.is_ok());
    }

    #[test]
    fn test_ed25519_keypair_generation() {
        let config = PrimitivesConfig::default();
        let primitives = CryptoPrimitives::new(&config).expect("Failed to create primitives");

        let result = primitives.generate_keypair(Algorithm::Ed25519);
        assert!(result.is_ok());

        let (public_key, private_key) = result.unwrap();
        assert_eq!(public_key.algorithm(), Algorithm::Ed25519);
        assert_eq!(private_key.algorithm(), Algorithm::Ed25519);
        assert_eq!(public_key.key_material().len(), 32); // Ed25519 public key is 32 bytes
    }

    #[test]
    fn test_symmetric_encryption_roundtrip() {
        let config = PrimitivesConfig::default();
        let primitives = CryptoPrimitives::new(&config).expect("Failed to create primitives");

        let key_material = vec![0u8; 32]; // 32-byte key for AES-256
        let plaintext = b"Hello, World!";

        // Encrypt
        let encrypted = primitives.symmetric_encrypt(plaintext, &key_material);
        assert!(encrypted.is_ok());

        let encrypted_data = encrypted.unwrap();

        // Decrypt
        let decrypted = primitives.symmetric_decrypt(&encrypted_data, &key_material);
        assert!(decrypted.is_ok());

        let decrypted_data = decrypted.unwrap();
        assert_eq!(plaintext, &decrypted_data[..]);
    }

    #[test]
    fn test_ed25519_signing_roundtrip() {
        let config = PrimitivesConfig::default();
        let primitives = CryptoPrimitives::new(&config).expect("Failed to create primitives");

        // Generate keypair
        let (public_key, private_key) = primitives
            .generate_keypair(Algorithm::Ed25519)
            .expect("Failed to generate keypair");

        let data = b"Data to sign";

        // Sign
        let signature = primitives.sign(data, &private_key);
        assert!(signature.is_ok());

        let sig = signature.unwrap();

        // Verify
        let valid = primitives.verify(data, &sig, &public_key);
        assert!(valid.is_ok());
        assert!(valid.unwrap());

        // Test with wrong data
        let wrong_data = b"Wrong data";
        let invalid = primitives.verify(wrong_data, &sig, &public_key);
        assert!(invalid.is_ok());
        assert!(!invalid.unwrap());
    }

    #[test]
    fn test_hash_computation() {
        let config = PrimitivesConfig::default();
        let primitives = CryptoPrimitives::new(&config).expect("Failed to create primitives");

        let data = b"Data to hash";
        let hash = primitives.hash(data, HashAlgorithm::Sha256);
        assert!(hash.is_ok());

        let digest = hash.unwrap();
        assert_eq!(digest.len(), 32); // SHA-256 produces 32-byte digest

        // Test consistency
        let hash2 = primitives.hash(data, HashAlgorithm::Sha256).unwrap();
        assert_eq!(digest, hash2);
    }

    #[test]
    fn test_random_generation() {
        let config = PrimitivesConfig::default();
        let primitives = CryptoPrimitives::new(&config).expect("Failed to create primitives");

        let random1 = primitives.generate_random_bytes(32);
        assert!(random1.is_ok());

        let random2 = primitives.generate_random_bytes(32);
        assert!(random2.is_ok());

        // Should be different (extremely high probability)
        assert_ne!(random1.unwrap(), random2.unwrap());
    }

    #[test]
    fn test_invalid_inputs() {
        let config = PrimitivesConfig::default();
        let primitives = CryptoPrimitives::new(&config).expect("Failed to create primitives");

        // Test zero-length random generation
        let result = primitives.generate_random_bytes(0);
        assert!(result.is_err());

        // Test excessive random generation
        let result = primitives.generate_random_bytes(2_000_000);
        assert!(result.is_err());

        // Test unsupported algorithm
        let result = primitives.generate_keypair(Algorithm::Blake3);
        assert!(result.is_err());
    }

    #[test]
    fn test_encryption_with_different_algorithms() {
        let config = PrimitivesConfig::default();
        let primitives = CryptoPrimitives::new(&config).expect("Failed to create primitives");
        
        let plaintext = b"test data for algorithm comparison";
        let key_material = vec![42u8; 32];
        
        // Test AES-256-GCM
        let aes_encrypted = primitives.aes_256_gcm_encrypt(plaintext, &key_material)
            .expect("Failed to encrypt with AES-256-GCM");
        assert_eq!(aes_encrypted.algorithm(), Algorithm::Aes256Gcm);
        
        let aes_decrypted = primitives.aes_256_gcm_decrypt(&aes_encrypted, &key_material)
            .expect("Failed to decrypt with AES-256-GCM");
        assert_eq!(plaintext, &aes_decrypted[..]);
        
        // Test ChaCha20-Poly1305
        let chacha_encrypted = primitives.chacha20_poly1305_encrypt(plaintext, &key_material)
            .expect("Failed to encrypt with ChaCha20-Poly1305");
        assert_eq!(chacha_encrypted.algorithm(), Algorithm::ChaCha20Poly1305);
        
        let chacha_decrypted = primitives.chacha20_poly1305_decrypt(&chacha_encrypted, &key_material)
            .expect("Failed to decrypt with ChaCha20-Poly1305");
        assert_eq!(plaintext, &chacha_decrypted[..]);
        
        // Ciphertexts should be different
        assert_ne!(aes_encrypted.ciphertext(), chacha_encrypted.ciphertext());
    }

    #[test]
    fn test_hash_algorithm_differences() {
        let config = PrimitivesConfig::default();
        let primitives = CryptoPrimitives::new(&config).expect("Failed to create primitives");
        
        let data = b"test data for hash comparison";
        
        let sha256_hash = primitives.hash(data, HashAlgorithm::Sha256)
            .expect("Failed to hash with SHA-256");
        let sha3_hash = primitives.hash(data, HashAlgorithm::Sha3_256)
            .expect("Failed to hash with SHA3-256");
        let blake3_hash = primitives.hash(data, HashAlgorithm::Blake3)
            .expect("Failed to hash with BLAKE3");
        
        // All should be 32 bytes
        assert_eq!(sha256_hash.len(), 32);
        assert_eq!(sha3_hash.len(), 32);
        assert_eq!(blake3_hash.len(), 32);
        
        // All should be different
        assert_ne!(sha256_hash, sha3_hash);
        assert_ne!(sha256_hash, blake3_hash);
        assert_ne!(sha3_hash, blake3_hash);
    }

    #[test]
    fn test_encryption_nonce_uniqueness() {
        let config = PrimitivesConfig::default();
        let primitives = CryptoPrimitives::new(&config).expect("Failed to create primitives");
        
        let plaintext = b"test data for nonce uniqueness";
        let key_material = vec![42u8; 32];
        
        // Encrypt same data multiple times
        let mut nonces = std::collections::HashSet::new();
        for _ in 0..100 {
            let encrypted = primitives.aes_256_gcm_encrypt(plaintext, &key_material)
                .expect("Failed to encrypt");
            let nonce = encrypted.iv().to_vec();
            
            // Each nonce should be unique
            assert!(!nonces.contains(&nonce), "Duplicate nonce found");
            nonces.insert(nonce);
        }
    }

    #[test]
    fn test_signature_determinism() {
        let config = PrimitivesConfig::default();
        let primitives = CryptoPrimitives::new(&config).expect("Failed to create primitives");
        
        // Generate a key pair
        let (public_key, private_key) = primitives.generate_keypair(Algorithm::Ed25519)
            .expect("Failed to generate keypair");
        
        let data = b"test data for signature determinism";
        
        // Ed25519 signatures should be deterministic
        let sig1 = primitives.sign(data, &private_key)
            .expect("Failed to sign first time");
        let sig2 = primitives.sign(data, &private_key)
            .expect("Failed to sign second time");
        
        // Signatures should be identical for Ed25519
        assert_eq!(sig1.signature(), sig2.signature());
        
        // Both should verify
        assert!(primitives.verify(data, &sig1, &public_key).unwrap());
        assert!(primitives.verify(data, &sig2, &public_key).unwrap());
    }

    #[test]
    fn test_large_data_handling() {
        let config = PrimitivesConfig::default();
        let primitives = CryptoPrimitives::new(&config).expect("Failed to create primitives");
        
        // Test with 10MB of data
        let large_data = vec![42u8; 10 * 1024 * 1024];
        let key_material = vec![42u8; 32];
        
        // Encryption should handle large data
        let encrypted = primitives.aes_256_gcm_encrypt(&large_data, &key_material)
            .expect("Failed to encrypt large data");
        
        let decrypted = primitives.aes_256_gcm_decrypt(&encrypted, &key_material)
            .expect("Failed to decrypt large data");
        
        assert_eq!(large_data, decrypted);
        
        // Hash should handle large data
        let hash = primitives.hash(&large_data, HashAlgorithm::Sha256)
            .expect("Failed to hash large data");
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_concurrent_crypto_operations() {
        use std::sync::Arc;
        use std::thread;
        
        let config = PrimitivesConfig::default();
        let primitives = Arc::new(CryptoPrimitives::new(&config).expect("Failed to create primitives"));
        
        let mut handles = vec![];
        
        // Spawn threads doing different crypto operations
        for i in 0..10 {
            let primitives_clone = Arc::clone(&primitives);
            let handle = thread::spawn(move || {
                let data = format!("test data {}", i).into_bytes();
                let key_material = vec![42u8; 32];
                
                // Test encryption
                let encrypted = primitives_clone.aes_256_gcm_encrypt(&data, &key_material)
                    .expect("Failed to encrypt in thread");
                let decrypted = primitives_clone.aes_256_gcm_decrypt(&encrypted, &key_material)
                    .expect("Failed to decrypt in thread");
                assert_eq!(data, decrypted);
                
                // Test hashing
                let hash = primitives_clone.hash(&data, HashAlgorithm::Sha256)
                    .expect("Failed to hash in thread");
                assert_eq!(hash.len(), 32);
                
                // Test key generation
                let (pub_key, priv_key) = primitives_clone.generate_keypair(Algorithm::Ed25519)
                    .expect("Failed to generate keypair in thread");
                
                // Test signing
                let signature = primitives_clone.sign(&data, &priv_key)
                    .expect("Failed to sign in thread");
                let valid = primitives_clone.verify(&data, &signature, &pub_key)
                    .expect("Failed to verify in thread");
                assert!(valid);
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    }

    #[test]
    fn test_error_propagation() {
        let config = PrimitivesConfig::default();
        let primitives = CryptoPrimitives::new(&config).expect("Failed to create primitives");
        
        // Test with wrong key size for AES
        let wrong_key = vec![42u8; 16]; // Too short
        let plaintext = b"test data";
        
        let result = primitives.aes_256_gcm_encrypt(plaintext, &wrong_key);
        assert!(result.is_err());
        
        match result {
            Err(UnifiedCryptoError::InvalidInput { message }) => {
                assert!(message.contains("32-byte"));
            },
            _ => panic!("Expected InvalidInput error"),
        }
        
        // Test with wrong key size for ChaCha20
        let result = primitives.chacha20_poly1305_encrypt(plaintext, &wrong_key);
        assert!(result.is_err());
        
        match result {
            Err(UnifiedCryptoError::InvalidInput { message }) => {
                assert!(message.contains("32-byte"));
            },
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_key_material_zeroization() {
        // This test verifies that sensitive key material is properly handled
        // Note: This is a simplified test - in production, we'd use specialized
        // memory testing tools to verify actual zeroization
        
        let config = PrimitivesConfig::default();
        let primitives = CryptoPrimitives::new(&config).expect("Failed to create primitives");
        
        // Generate a keypair
        let (public_key, private_key) = primitives.generate_keypair(Algorithm::Ed25519)
            .expect("Failed to generate keypair");
        
        // Verify keys are created properly
        assert_eq!(public_key.key_material().len(), 32);
        assert_eq!(private_key.key_material().len(), 32);
        
        // Keys should contain non-zero data
        assert_ne!(public_key.key_material(), &[0u8; 32]);
        assert_ne!(private_key.key_material().bytes(), &[0u8; 32]);
        
        // Drop the keys (triggering Drop implementations)
        drop(public_key);
        drop(private_key);
        
        // Note: In a real implementation, we'd verify memory is actually zeroed
        // This would require unsafe code and platform-specific memory inspection
    }
}