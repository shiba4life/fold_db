//! DataFold Unified Backup Format Implementation for Rust
//!
//! This module implements the standardized encrypted backup format for cross-platform
//! compatibility following the specification from docs/delivery/10/backup/encrypted_backup_format.md

use crate::crypto::{MasterKeyPair, derive_key, Argon2Params};
use rand::{RngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::Utc;
use base64::{Engine as _, engine::general_purpose};

/// Unified backup format structure as defined in the specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedBackupFormat {
    pub version: u32,
    pub kdf: String,
    pub kdf_params: KdfParams,
    pub encryption: String,
    pub nonce: String,
    pub ciphertext: String,
    pub created: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BackupMetadata>,
}

/// KDF parameters for the unified format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdfParams {
    pub salt: String,
    pub iterations: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<u32>,      // Required for argon2id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallelism: Option<u32>, // Required for argon2id
}

/// Backup metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub key_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Migration result for converting legacy backups
#[derive(Debug, Clone)]
pub struct MigrationResult {
    pub success: bool,
    pub original_format: String,
    pub new_format: UnifiedBackupFormat,
    pub warnings: Vec<String>,
}

/// Test vector for cross-platform validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestVector {
    pub passphrase: String,
    pub salt: String,
    pub nonce: String,
    pub kdf: String,
    pub kdf_params: HashMap<String, serde_json::Value>,
    pub encryption: String,
    pub plaintext_key: String,
    pub ciphertext: String,
    pub created: String,
}

/// Constants for the unified format
pub const UNIFIED_BACKUP_VERSION: u32 = 1;
pub const MIN_SALT_LENGTH: usize = 16;
pub const PREFERRED_SALT_LENGTH: usize = 32;
pub const XCHACHA20_NONCE_LENGTH: usize = 24;
pub const AES_GCM_NONCE_LENGTH: usize = 12;

// Argon2id parameters (preferred)
pub const ARGON2_MIN_MEMORY: u32 = 65536; // 64 MiB
pub const ARGON2_MIN_ITERATIONS: u32 = 3;
pub const ARGON2_MIN_PARALLELISM: u32 = 2;

// PBKDF2 parameters (legacy compatibility)
pub const PBKDF2_MIN_ITERATIONS: u32 = 100000;

/// Unified Backup Manager for cross-platform compatibility
#[derive(Debug)]
pub struct UnifiedBackupManager {
    preferred_kdf: String,
    preferred_encryption: String,
}

impl Default for UnifiedBackupManager {
    fn default() -> Self {
        Self {
            preferred_kdf: "argon2id".to_string(),
            preferred_encryption: "xchacha20-poly1305".to_string(),
        }
    }
}

impl UnifiedBackupManager {
    /// Create a new unified backup manager with default preferences
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new unified backup manager with custom preferences
    pub fn with_preferences(kdf: String, encryption: String) -> Result<Self, BackupError> {
        let manager = Self {
            preferred_kdf: kdf,
            preferred_encryption: encryption,
        };
        manager.validate_algorithm_support(&manager.preferred_kdf, &manager.preferred_encryption)?;
        Ok(manager)
    }

    /// Export key using unified backup format
    pub fn export_key(
        &self,
        key_pair: &MasterKeyPair,
        passphrase: &str,
        options: ExportOptions,
    ) -> Result<String, BackupError> {
        self.validate_passphrase(passphrase)?;
        self.validate_key_pair(key_pair)?;

        let kdf = options.kdf.unwrap_or_else(|| self.preferred_kdf.clone());
        let encryption = options.encryption.unwrap_or_else(|| self.preferred_encryption.clone());
        
        self.validate_algorithm_support(&kdf, &encryption)?;

        // Generate salt and nonce
        let salt = self.generate_salt();
        let nonce_length = if encryption == "xchacha20-poly1305" { 
            XCHACHA20_NONCE_LENGTH 
        } else { 
            AES_GCM_NONCE_LENGTH 
        };
        let nonce = self.generate_nonce(nonce_length);

        // Prepare KDF parameters
        let kdf_params = self.prepare_kdf_params(&kdf, options.kdf_params)?;

        // Derive encryption key
        let encryption_key = self.derive_key(passphrase, &salt, &kdf, &kdf_params)?;

        // Prepare plaintext (Ed25519 keys concatenated)
        let plaintext = self.prepare_key_plaintext(key_pair);

        // Encrypt the key data
        let ciphertext = self.encrypt_data(&plaintext, &encryption_key, &nonce, &encryption)?;

        // Create unified backup format
        let backup = UnifiedBackupFormat {
            version: UNIFIED_BACKUP_VERSION,
            kdf: kdf.clone(),
            kdf_params: KdfParams {
                salt: general_purpose::STANDARD.encode(&salt),
                iterations: kdf_params.iterations.unwrap_or(100000),
                memory: if kdf == "argon2id" { kdf_params.memory } else { None },
                parallelism: if kdf == "argon2id" { kdf_params.parallelism } else { None },
            },
            encryption,
            nonce: general_purpose::STANDARD.encode(&nonce),
            ciphertext: general_purpose::STANDARD.encode(&ciphertext),
            created: Utc::now().to_rfc3339(),
            metadata: if options.label.is_some() {
                Some(BackupMetadata {
                    key_type: "ed25519".to_string(),
                    label: options.label,
                })
            } else {
                None
            },
        };

        serde_json::to_string_pretty(&backup).map_err(|e| BackupError::SerializationError(e.to_string()))
    }

    /// Import key from unified backup format
    pub fn import_key(
        &self,
        backup_data: &str,
        passphrase: &str,
    ) -> Result<(MasterKeyPair, Option<BackupMetadata>), BackupError> {
        self.validate_passphrase(passphrase)?;

        // Parse backup data
        let backup: UnifiedBackupFormat = serde_json::from_str(backup_data)
            .map_err(|e| BackupError::InvalidFormat(format!("Invalid JSON backup data: {}", e)))?;

        // Validate backup format
        self.validate_backup_format(&backup)?;

        // Extract parameters
        let salt = general_purpose::STANDARD.decode(&backup.kdf_params.salt)
            .map_err(|e| BackupError::InvalidFormat(format!("Invalid salt encoding: {}", e)))?;
        let nonce = general_purpose::STANDARD.decode(&backup.nonce)
            .map_err(|e| BackupError::InvalidFormat(format!("Invalid nonce encoding: {}", e)))?;
        let ciphertext = general_purpose::STANDARD.decode(&backup.ciphertext)
            .map_err(|e| BackupError::InvalidFormat(format!("Invalid ciphertext encoding: {}", e)))?;

        // Prepare KDF parameters
        let kdf_params = InternalKdfParams {
            iterations: Some(backup.kdf_params.iterations),
            memory: backup.kdf_params.memory,
            parallelism: backup.kdf_params.parallelism,
        };

        // Derive decryption key
        let decryption_key = self.derive_key(passphrase, &salt, &backup.kdf, &kdf_params)?;

        // Decrypt the key data
        let plaintext = self.decrypt_data(&ciphertext, &decryption_key, &nonce, &backup.encryption)?;

        // Extract key pair from plaintext
        let key_pair = self.extract_key_pair(&plaintext)?;

        Ok((key_pair, backup.metadata))
    }

    /// Generate test vector for cross-platform validation
    pub fn generate_test_vector(&self) -> Result<TestVector, BackupError> {
        // Use fixed test data for reproducible test vectors
        let passphrase = "correct horse battery staple";
        let salt = general_purpose::STANDARD.decode("w7Z3pQ2v5Q8v1Q2v5Q8v1Q==")
            .map_err(|e| BackupError::InvalidFormat(format!("Invalid test salt: {}", e)))?;
        let nonce = general_purpose::STANDARD.decode("AAAAAAAAAAAAAAAAAAAAAAAAAAA=")
            .map_err(|e| BackupError::InvalidFormat(format!("Invalid test nonce: {}", e)))?;

        // Create test key pair (deterministic)
        let test_private_key = [0x42u8; 32];
        let test_key_pair = MasterKeyPair::from_secret_bytes(&test_private_key)
            .map_err(|e| BackupError::InvalidFormat(format!("Failed to create test key pair: {}", e)))?;

        let kdf = "argon2id";
        let kdf_params = InternalKdfParams {
            iterations: Some(ARGON2_MIN_ITERATIONS),
            memory: Some(ARGON2_MIN_MEMORY),
            parallelism: Some(ARGON2_MIN_PARALLELISM),
        };
        let encryption = "xchacha20-poly1305";

        // Derive key and encrypt
        let derived_key = self.derive_key(passphrase, &salt, kdf, &kdf_params)?;
        let plaintext = self.prepare_key_plaintext(&test_key_pair);
        let ciphertext = self.encrypt_data(&plaintext, &derived_key, &nonce, encryption)?;

        let mut kdf_params_map = HashMap::new();
        kdf_params_map.insert("iterations".to_string(), serde_json::Value::Number(serde_json::Number::from(ARGON2_MIN_ITERATIONS)));
        kdf_params_map.insert("memory".to_string(), serde_json::Value::Number(serde_json::Number::from(ARGON2_MIN_MEMORY)));
        kdf_params_map.insert("parallelism".to_string(), serde_json::Value::Number(serde_json::Number::from(ARGON2_MIN_PARALLELISM)));

        Ok(TestVector {
            passphrase: passphrase.to_string(),
            salt: general_purpose::STANDARD.encode(&salt),
            nonce: general_purpose::STANDARD.encode(&nonce),
            kdf: kdf.to_string(),
            kdf_params: kdf_params_map,
            encryption: encryption.to_string(),
            plaintext_key: general_purpose::STANDARD.encode(&plaintext),
            ciphertext: general_purpose::STANDARD.encode(&ciphertext),
            created: "2025-06-08T17:00:00Z".to_string(),
        })
    }

    /// Validate cross-platform compatibility with test vector
    pub fn validate_test_vector(&self, test_vector: &TestVector) -> Result<bool, BackupError> {
        let salt = general_purpose::STANDARD.decode(&test_vector.salt)
            .map_err(|e| BackupError::InvalidFormat(format!("Invalid test vector salt: {}", e)))?;
        let nonce = general_purpose::STANDARD.decode(&test_vector.nonce)
            .map_err(|e| BackupError::InvalidFormat(format!("Invalid test vector nonce: {}", e)))?;
        let expected_ciphertext = general_purpose::STANDARD.decode(&test_vector.ciphertext)
            .map_err(|e| BackupError::InvalidFormat(format!("Invalid test vector ciphertext: {}", e)))?;
        let expected_plaintext = general_purpose::STANDARD.decode(&test_vector.plaintext_key)
            .map_err(|e| BackupError::InvalidFormat(format!("Invalid test vector plaintext: {}", e)))?;

        // Extract KDF parameters
        let iterations = test_vector.kdf_params.get("iterations")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| BackupError::InvalidFormat("Missing iterations in test vector".to_string()))? as u32;
        let memory = test_vector.kdf_params.get("memory")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);
        let parallelism = test_vector.kdf_params.get("parallelism")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        let kdf_params = InternalKdfParams {
            iterations: Some(iterations),
            memory,
            parallelism,
        };

        // Derive key using test vector parameters
        let derived_key = self.derive_key(&test_vector.passphrase, &salt, &test_vector.kdf, &kdf_params)?;

        // Decrypt test vector ciphertext
        let decrypted_plaintext = self.decrypt_data(&expected_ciphertext, &derived_key, &nonce, &test_vector.encryption)?;

        // Compare with expected plaintext
        Ok(decrypted_plaintext == expected_plaintext)
    }

    // Private helper methods

    fn validate_passphrase(&self, passphrase: &str) -> Result<(), BackupError> {
        if passphrase.is_empty() {
            return Err(BackupError::WeakPassphrase("Passphrase cannot be empty".to_string()));
        }
        if passphrase.len() < 8 {
            return Err(BackupError::WeakPassphrase("Passphrase must be at least 8 characters long".to_string()));
        }
        Ok(())
    }

    fn validate_key_pair(&self, key_pair: &MasterKeyPair) -> Result<(), BackupError> {
        let private_bytes = key_pair.secret_key_bytes();
        let public_bytes = key_pair.public_key_bytes();
        if private_bytes.len() != 32 || public_bytes.len() != 32 {
            return Err(BackupError::InvalidKeyPair("Invalid key pair: incorrect key lengths".to_string()));
        }
        Ok(())
    }

    pub fn validate_algorithm_support(&self, kdf: &str, encryption: &str) -> Result<(), BackupError> {
        if !["argon2id", "pbkdf2"].contains(&kdf) {
            return Err(BackupError::UnsupportedAlgorithm(format!("Unsupported KDF: {}", kdf)));
        }
        if !["xchacha20-poly1305", "aes-gcm"].contains(&encryption) {
            return Err(BackupError::UnsupportedAlgorithm(format!("Unsupported encryption: {}", encryption)));
        }
        Ok(())
    }

    fn generate_salt(&self) -> Vec<u8> {
        let mut salt = vec![0u8; PREFERRED_SALT_LENGTH];
        OsRng.fill_bytes(&mut salt);
        salt
    }

    fn generate_nonce(&self, length: usize) -> Vec<u8> {
        let mut nonce = vec![0u8; length];
        OsRng.fill_bytes(&mut nonce);
        nonce
    }

    fn prepare_kdf_params(&self, kdf: &str, custom_params: Option<CustomKdfParams>) -> Result<InternalKdfParams, BackupError> {
        if kdf == "argon2id" {
            Ok(InternalKdfParams {
                iterations: Some(custom_params.as_ref().and_then(|p| p.iterations).unwrap_or(ARGON2_MIN_ITERATIONS)),
                memory: Some(custom_params.as_ref().and_then(|p| p.memory).unwrap_or(ARGON2_MIN_MEMORY)),
                parallelism: Some(custom_params.as_ref().and_then(|p| p.parallelism).unwrap_or(ARGON2_MIN_PARALLELISM)),
            })
        } else {
            Ok(InternalKdfParams {
                iterations: Some(custom_params.as_ref().and_then(|p| p.iterations).unwrap_or(PBKDF2_MIN_ITERATIONS)),
                memory: None,
                parallelism: None,
            })
        }
    }

    fn derive_key(&self, passphrase: &str, salt: &[u8], kdf: &str, params: &InternalKdfParams) -> Result<Vec<u8>, BackupError> {
        match kdf {
            "argon2id" => {
                let argon2_params = Argon2Params {
                    memory_cost: params.memory.unwrap_or(ARGON2_MIN_MEMORY),
                    time_cost: params.iterations.unwrap_or(ARGON2_MIN_ITERATIONS),
                    parallelism: params.parallelism.unwrap_or(ARGON2_MIN_PARALLELISM),
                };
                
                let salt_obj = crate::crypto::argon2::Salt::from_bytes({
                    let mut salt_array = [0u8; 32];
                    let copy_len = 32.min(salt.len());
                    salt_array[..copy_len].copy_from_slice(&salt[..copy_len]);
                    salt_array
                });
                
                // Use the existing derive_key function with Argon2 parameters
                let derived = derive_key(passphrase, &salt_obj, &argon2_params)
                    .map_err(|e| BackupError::KeyDerivationFailed(format!("Argon2id derivation failed: {}", e)))?;
                
                Ok(derived.as_bytes().to_vec())
            }
            "pbkdf2" => {
                // For PBKDF2, we'll use a simplified implementation
                // In a full implementation, you'd use a proper PBKDF2 function
                Err(BackupError::UnsupportedAlgorithm("PBKDF2 not yet implemented in CLI".to_string()))
            }
            _ => Err(BackupError::UnsupportedAlgorithm(format!("Unsupported KDF: {}", kdf))),
        }
    }

    fn prepare_key_plaintext(&self, key_pair: &MasterKeyPair) -> Vec<u8> {
        let mut plaintext = Vec::with_capacity(64);
        plaintext.extend_from_slice(&key_pair.secret_key_bytes());
        plaintext.extend_from_slice(&key_pair.public_key_bytes());
        plaintext
    }

    fn extract_key_pair(&self, key_data: &[u8]) -> Result<MasterKeyPair, BackupError> {
        if key_data.len() != 64 {
            return Err(BackupError::InvalidFormat("Invalid key data length in backup".to_string()));
        }

        let mut private_key = [0u8; 32];
        private_key.copy_from_slice(&key_data[0..32]);

        MasterKeyPair::from_secret_bytes(&private_key)
            .map_err(|e| BackupError::InvalidFormat(format!("Failed to create key pair: {}", e)))
    }

    fn encrypt_data(&self, _plaintext: &[u8], _key: &[u8], _nonce: &[u8], algorithm: &str) -> Result<Vec<u8>, BackupError> {
        match algorithm {
            "xchacha20-poly1305" => {
                // For now, fall back to ChaCha20-Poly1305 (not XChaCha20-Poly1305)
                // In a full implementation, you'd use a proper XChaCha20-Poly1305 library
                Err(BackupError::UnsupportedAlgorithm("XChaCha20-Poly1305 not yet implemented".to_string()))
            }
            "aes-gcm" => {
                // Use AES-GCM implementation
                // This would require implementing AES-GCM or using an external crate
                Err(BackupError::UnsupportedAlgorithm("AES-GCM not yet implemented".to_string()))
            }
            _ => Err(BackupError::UnsupportedAlgorithm(format!("Unsupported encryption: {}", algorithm))),
        }
    }

    fn decrypt_data(&self, _ciphertext: &[u8], _key: &[u8], _nonce: &[u8], algorithm: &str) -> Result<Vec<u8>, BackupError> {
        match algorithm {
            "xchacha20-poly1305" => {
                Err(BackupError::UnsupportedAlgorithm("XChaCha20-Poly1305 not yet implemented".to_string()))
            }
            "aes-gcm" => {
                Err(BackupError::UnsupportedAlgorithm("AES-GCM not yet implemented".to_string()))
            }
            _ => Err(BackupError::UnsupportedAlgorithm(format!("Unsupported encryption: {}", algorithm))),
        }
    }

    fn validate_backup_format(&self, backup: &UnifiedBackupFormat) -> Result<(), BackupError> {
        if backup.version != UNIFIED_BACKUP_VERSION {
            return Err(BackupError::UnsupportedVersion(format!("Unsupported backup version: {}", backup.version)));
        }

        if !["argon2id", "pbkdf2"].contains(&backup.kdf.as_str()) {
            return Err(BackupError::UnsupportedAlgorithm(format!("Unsupported KDF: {}", backup.kdf)));
        }

        if !["xchacha20-poly1305", "aes-gcm"].contains(&backup.encryption.as_str()) {
            return Err(BackupError::UnsupportedAlgorithm(format!("Unsupported encryption: {}", backup.encryption)));
        }

        // Validate KDF parameters
        if backup.kdf == "argon2id" && (backup.kdf_params.memory.is_none() || backup.kdf_params.parallelism.is_none()) {
            return Err(BackupError::InvalidFormat("Missing Argon2id parameters (memory, parallelism)".to_string()));
        }

        Ok(())
    }
}

/// Export options for unified backup
#[derive(Debug, Clone, Default)]
pub struct ExportOptions {
    pub label: Option<String>,
    pub kdf: Option<String>,
    pub encryption: Option<String>,
    pub kdf_params: Option<CustomKdfParams>,
}

/// Custom KDF parameters for export
#[derive(Debug, Clone)]
pub struct CustomKdfParams {
    pub iterations: Option<u32>,
    pub memory: Option<u32>,      // For Argon2id
    pub parallelism: Option<u32>, // For Argon2id
}

/// Internal KDF parameters
#[derive(Debug, Clone)]
struct InternalKdfParams {
    iterations: Option<u32>,
    memory: Option<u32>,
    parallelism: Option<u32>,
}

/// Backup errors
#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    #[error("Weak passphrase: {0}")]
    WeakPassphrase(String),
    #[error("Invalid key pair: {0}")]
    InvalidKeyPair(String),
    #[error("Unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Unsupported version: {0}")]
    UnsupportedVersion(String),
    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

// Convenience functions
pub fn export_key_unified(
    key_pair: &MasterKeyPair,
    passphrase: &str,
    options: ExportOptions,
) -> Result<String, BackupError> {
    let manager = UnifiedBackupManager::new();
    manager.export_key(key_pair, passphrase, options)
}

pub fn import_key_unified(
    backup_data: &str,
    passphrase: &str,
) -> Result<(MasterKeyPair, Option<BackupMetadata>), BackupError> {
    let manager = UnifiedBackupManager::new();
    manager.import_key(backup_data, passphrase)
}