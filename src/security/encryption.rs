//! AES-GCM encryption and decryption for data at rest

use crate::security::{SecurityError, SecurityResult, EncryptedData};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{Engine as _, engine::general_purpose};
use serde_json::Value;
use std::sync::Arc;

/// AES-256-GCM encryption manager
pub struct EncryptionManager {
    cipher: Aes256Gcm,
}

impl EncryptionManager {
    /// Create a new encryption manager with a master key
    pub fn new(master_key: &[u8; 32]) -> Self {
        let key = Key::<Aes256Gcm>::from_slice(master_key);
        let cipher = Aes256Gcm::new(key);
        
        Self { cipher }
    }
    
    /// Generate a random master key
    pub fn generate_master_key() -> [u8; 32] {
        Aes256Gcm::generate_key(&mut OsRng).into()
    }
    
    /// Encrypt data and return an EncryptedData container
    pub fn encrypt(&self, data: &[u8]) -> SecurityResult<EncryptedData> {
        // Generate a random nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        // Encrypt the data
        let ciphertext = self.cipher.encrypt(&nonce, data)
            .map_err(|e| SecurityError::EncryptionFailed(e.to_string()))?;
        
        // Split ciphertext and tag (AES-GCM appends 16-byte tag)
        if ciphertext.len() < 16 {
            return Err(SecurityError::EncryptionFailed(
                "Ciphertext too short".to_string()
            ));
        }
        
        let (encrypted_data, tag) = ciphertext.split_at(ciphertext.len() - 16);
        
        Ok(EncryptedData::new(
            general_purpose::STANDARD.encode(encrypted_data),
            general_purpose::STANDARD.encode(nonce),
            general_purpose::STANDARD.encode(tag),
        ))
    }
    
    /// Encrypt JSON data
    pub fn encrypt_json(&self, json_data: &Value) -> SecurityResult<EncryptedData> {
        let data_bytes = serde_json::to_vec(json_data)
            .map_err(|e| SecurityError::SerializationError(e.to_string()))?;
        
        self.encrypt(&data_bytes)
    }
    
    /// Encrypt a string
    pub fn encrypt_string(&self, text: &str) -> SecurityResult<EncryptedData> {
        self.encrypt(text.as_bytes())
    }
    
    /// Decrypt data from an EncryptedData container
    pub fn decrypt(&self, encrypted_data: &EncryptedData) -> SecurityResult<Vec<u8>> {
        // Decode base64 components
        let data = general_purpose::STANDARD.decode(&encrypted_data.data)
            .map_err(|e| SecurityError::DecryptionFailed(e.to_string()))?;
        
        let nonce_bytes = general_purpose::STANDARD.decode(&encrypted_data.nonce)
            .map_err(|e| SecurityError::DecryptionFailed(e.to_string()))?;
        
        let tag = general_purpose::STANDARD.decode(&encrypted_data.tag)
            .map_err(|e| SecurityError::DecryptionFailed(e.to_string()))?;
        
        // Validate nonce length
        if nonce_bytes.len() != 12 {
            return Err(SecurityError::DecryptionFailed(
                "Invalid nonce length".to_string()
            ));
        }
        
        // Validate tag length
        if tag.len() != 16 {
            return Err(SecurityError::DecryptionFailed(
                "Invalid tag length".to_string()
            ));
        }
        
        // Reconstruct ciphertext with tag
        let mut ciphertext = data;
        ciphertext.extend_from_slice(&tag);
        
        // Create nonce
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Decrypt the data
        let plaintext = self.cipher.decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| SecurityError::DecryptionFailed(e.to_string()))?;
        
        Ok(plaintext)
    }
    
    /// Decrypt to JSON data
    pub fn decrypt_json(&self, encrypted_data: &EncryptedData) -> SecurityResult<Value> {
        let plaintext = self.decrypt(encrypted_data)?;
        
        serde_json::from_slice(&plaintext)
            .map_err(|e| SecurityError::DeserializationError(e.to_string()))
    }
    
    /// Decrypt to string
    pub fn decrypt_string(&self, encrypted_data: &EncryptedData) -> SecurityResult<String> {
        let plaintext = self.decrypt(encrypted_data)?;
        
        String::from_utf8(plaintext)
            .map_err(|e| SecurityError::DecryptionFailed(e.to_string()))
    }
}

/// Wrapper for conditional encryption based on configuration
pub struct ConditionalEncryption {
    manager: Option<Arc<EncryptionManager>>,
    encrypt_at_rest: bool,
}

impl ConditionalEncryption {
    /// Create a new conditional encryption wrapper
    pub fn new(encrypt_at_rest: bool, master_key: Option<[u8; 32]>) -> SecurityResult<Self> {
        let manager = if encrypt_at_rest {
            match master_key {
                Some(key) => Some(Arc::new(EncryptionManager::new(&key))),
                None => return Err(SecurityError::EncryptionFailed(
                    "Master key required for encryption".to_string()
                )),
            }
        } else {
            None
        };
        
        Ok(Self {
            manager,
            encrypt_at_rest,
        })
    }
    
    /// Conditionally encrypt data based on configuration
    pub fn maybe_encrypt(&self, data: &[u8]) -> SecurityResult<Option<EncryptedData>> {
        if self.encrypt_at_rest {
            if let Some(manager) = &self.manager {
                Ok(Some(manager.encrypt(data)?))
            } else {
                Err(SecurityError::EncryptionFailed(
                    "Encryption enabled but no manager available".to_string()
                ))
            }
        } else {
            Ok(None)
        }
    }
    
    /// Conditionally encrypt JSON data
    pub fn maybe_encrypt_json(&self, json_data: &Value) -> SecurityResult<Option<EncryptedData>> {
        if self.encrypt_at_rest {
            if let Some(manager) = &self.manager {
                Ok(Some(manager.encrypt_json(json_data)?))
            } else {
                Err(SecurityError::EncryptionFailed(
                    "Encryption enabled but no manager available".to_string()
                ))
            }
        } else {
            Ok(None)
        }
    }
    
    /// Conditionally decrypt data
    pub fn maybe_decrypt(&self, encrypted_data: &EncryptedData) -> SecurityResult<Vec<u8>> {
        if let Some(manager) = &self.manager {
            manager.decrypt(encrypted_data)
        } else {
            Err(SecurityError::DecryptionFailed(
                "No encryption manager available for decryption".to_string()
            ))
        }
    }
    
    /// Conditionally decrypt JSON data
    pub fn maybe_decrypt_json(&self, encrypted_data: &EncryptedData) -> SecurityResult<Value> {
        if let Some(manager) = &self.manager {
            manager.decrypt_json(encrypted_data)
        } else {
            Err(SecurityError::DecryptionFailed(
                "No encryption manager available for decryption".to_string()
            ))
        }
    }
    
    /// Check if encryption is enabled
    pub fn is_encryption_enabled(&self) -> bool {
        self.encrypt_at_rest && self.manager.is_some()
    }
}

/// Utility functions for encryption operations
pub struct EncryptionUtils;

impl EncryptionUtils {
    /// Generate a master key from a password using PBKDF2
    pub fn derive_key_from_password(password: &str, salt: &[u8]) -> SecurityResult<[u8; 32]> {
        use ring::pbkdf2;
        use std::num::NonZeroU32;
        
        const CREDENTIAL_LEN: usize = 32;
        let mut key = [0u8; CREDENTIAL_LEN];
        
        let iterations = NonZeroU32::new(100_000).unwrap();
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            iterations,
            salt,
            password.as_bytes(),
            &mut key,
        );
        
        Ok(key)
    }
    
    /// Generate a random salt for key derivation
    pub fn generate_salt() -> [u8; 16] {
        use rand::RngCore;
        
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);
        salt
    }
    
    /// Create an encryption manager from a password
    pub fn create_manager_from_password(
        password: &str,
        salt: &[u8],
    ) -> SecurityResult<EncryptionManager> {
        let key = Self::derive_key_from_password(password, salt)?;
        Ok(EncryptionManager::new(&key))
    }
    
    /// Validate encrypted data format
    pub fn validate_encrypted_data(encrypted_data: &EncryptedData) -> SecurityResult<()> {
        // Check algorithm
        if encrypted_data.algorithm != "AES-256-GCM" {
            return Err(SecurityError::DecryptionFailed(
                format!("Unsupported algorithm: {}", encrypted_data.algorithm)
            ));
        }
        
        // Validate base64 encoding
        general_purpose::STANDARD.decode(&encrypted_data.data)
            .map_err(|e| SecurityError::DecryptionFailed(
                format!("Invalid data encoding: {}", e)
            ))?;
        
        let nonce = general_purpose::STANDARD.decode(&encrypted_data.nonce)
            .map_err(|e| SecurityError::DecryptionFailed(
                format!("Invalid nonce encoding: {}", e)
            ))?;
        
        let tag = general_purpose::STANDARD.decode(&encrypted_data.tag)
            .map_err(|e| SecurityError::DecryptionFailed(
                format!("Invalid tag encoding: {}", e)
            ))?;
        
        // Validate lengths
        if nonce.len() != 12 {
            return Err(SecurityError::DecryptionFailed(
                "Invalid nonce length".to_string()
            ));
        }
        
        if tag.len() != 16 {
            return Err(SecurityError::DecryptionFailed(
                "Invalid tag length".to_string()
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encryption_decryption() {
        let key = EncryptionManager::generate_master_key();
        let manager = EncryptionManager::new(&key);
        
        let original_data = b"Hello, world! This is a test message.";
        
        // Encrypt the data
        let encrypted = manager.encrypt(original_data).unwrap();
        
        // Decrypt the data
        let decrypted = manager.decrypt(&encrypted).unwrap();
        
        assert_eq!(original_data, decrypted.as_slice());
    }
    
    #[test]
    fn test_json_encryption() {
        let key = EncryptionManager::generate_master_key();
        let manager = EncryptionManager::new(&key);
        
        let original_json = serde_json::json!({
            "user": "alice",
            "action": "login",
            "timestamp": 1234567890,
            "data": {
                "ip": "192.168.1.1",
                "browser": "Chrome"
            }
        });
        
        // Encrypt the JSON
        let encrypted = manager.encrypt_json(&original_json).unwrap();
        
        // Decrypt the JSON
        let decrypted = manager.decrypt_json(&encrypted).unwrap();
        
        assert_eq!(original_json, decrypted);
    }
    
    #[test]
    fn test_string_encryption() {
        let key = EncryptionManager::generate_master_key();
        let manager = EncryptionManager::new(&key);
        
        let original_string = "This is a secret message";
        
        // Encrypt the string
        let encrypted = manager.encrypt_string(original_string).unwrap();
        
        // Decrypt the string
        let decrypted = manager.decrypt_string(&encrypted).unwrap();
        
        assert_eq!(original_string, decrypted);
    }
    
    #[test]
    fn test_conditional_encryption() {
        let key = EncryptionManager::generate_master_key();
        
        // With encryption enabled
        let conditional = ConditionalEncryption::new(true, Some(key)).unwrap();
        assert!(conditional.is_encryption_enabled());
        
        let data = b"test data";
        let encrypted = conditional.maybe_encrypt(data).unwrap();
        assert!(encrypted.is_some());
        
        let decrypted = conditional.maybe_decrypt(&encrypted.unwrap()).unwrap();
        assert_eq!(data, decrypted.as_slice());
        
        // With encryption disabled
        let conditional = ConditionalEncryption::new(false, None).unwrap();
        assert!(!conditional.is_encryption_enabled());
        
        let encrypted = conditional.maybe_encrypt(data).unwrap();
        assert!(encrypted.is_none());
    }
    
    #[test]
    fn test_key_derivation() {
        let password = "super_secret_password";
        let salt = EncryptionUtils::generate_salt();
        
        let key1 = EncryptionUtils::derive_key_from_password(password, &salt).unwrap();
        let key2 = EncryptionUtils::derive_key_from_password(password, &salt).unwrap();
        
        // Same password and salt should produce same key
        assert_eq!(key1, key2);
        
        // Different salt should produce different key
        let different_salt = EncryptionUtils::generate_salt();
        let key3 = EncryptionUtils::derive_key_from_password(password, &different_salt).unwrap();
        assert_ne!(key1, key3);
    }
    
    #[test]
    fn test_encrypted_data_validation() {
        let key = EncryptionManager::generate_master_key();
        let manager = EncryptionManager::new(&key);
        
        let data = b"test data";
        let encrypted = manager.encrypt(data).unwrap();
        
        // Valid encrypted data should pass validation
        assert!(EncryptionUtils::validate_encrypted_data(&encrypted).is_ok());
        
        // Invalid algorithm should fail
        let mut invalid = encrypted.clone();
        invalid.algorithm = "AES-128-GCM".to_string();
        assert!(EncryptionUtils::validate_encrypted_data(&invalid).is_err());
        
        // Invalid base64 should fail
        let mut invalid = encrypted.clone();
        invalid.data = "invalid_base64!".to_string();
        assert!(EncryptionUtils::validate_encrypted_data(&invalid).is_err());
    }
}