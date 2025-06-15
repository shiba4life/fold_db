//! Platform-specific keystore integration for secure configuration storage
//!
//! This module provides unified access to platform-specific secure storage:
//! - Linux: Secret Service API (libsecret)
//! - macOS: Keychain Services
//! - Windows: Credential Manager

use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::config::error::{ConfigError, ConfigResult};

/// Trait for platform-specific keystore operations
#[async_trait]
pub trait PlatformKeystore: Send + Sync {
    /// Store a secret in the platform keystore
    async fn store_secret(&self, key: &str, value: &[u8]) -> ConfigResult<()>;
    
    /// Retrieve a secret from the platform keystore
    async fn get_secret(&self, key: &str) -> ConfigResult<Option<Vec<u8>>>;
    
    /// Delete a secret from the platform keystore
    async fn delete_secret(&self, key: &str) -> ConfigResult<()>;
    
    /// List all keys stored by this application
    async fn list_keys(&self) -> ConfigResult<Vec<String>>;
    
    /// Check if keystore is available on this platform
    fn is_available(&self) -> bool;
    
    /// Get platform-specific keystore identifier
    fn keystore_type(&self) -> &'static str;
}

/// Secure configuration entry stored in keystore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureConfigEntry {
    /// Entry identifier
    pub key: String,
    /// Encrypted data
    pub encrypted_data: Vec<u8>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last access timestamp
    pub accessed_at: chrono::DateTime<chrono::Utc>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Configuration for keystore integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystoreConfig {
    /// Enable keystore integration
    pub enabled: bool,
    /// Service name for keystore entries
    pub service_name: String,
    /// Encryption key derivation settings
    pub key_derivation: KeyDerivationConfig,
    /// Auto-backup encrypted sections
    pub auto_backup: bool,
    /// Maximum entry lifetime in seconds
    pub max_lifetime_secs: Option<u64>,
}

/// Key derivation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationConfig {
    /// Key derivation algorithm
    pub algorithm: String,
    /// Salt length in bytes
    pub salt_length: usize,
    /// Number of iterations
    pub iterations: u32,
    /// Derived key length
    pub key_length: usize,
}

impl Default for KeystoreConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            service_name: "DataFold".to_string(),
            key_derivation: KeyDerivationConfig::default(),
            auto_backup: true,
            max_lifetime_secs: Some(86400 * 30), // 30 days
        }
    }
}

impl Default for KeyDerivationConfig {
    fn default() -> Self {
        Self {
            algorithm: "argon2id".to_string(),
            salt_length: 32,
            iterations: 3,
            key_length: 32,
        }
    }
}

/// Create platform-specific keystore implementation
pub fn create_platform_keystore() -> Box<dyn PlatformKeystore> {
    #[cfg(target_os = "linux")]
    {
        Box::new(super::linux::LinuxKeystore::new())
    }
    #[cfg(target_os = "macos")]
    {
        Box::new(super::macos::MacOSKeystore::new())
    }
    #[cfg(target_os = "windows")]
    {
        Box::new(super::windows::WindowsKeystore::new())
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Box::new(FallbackKeystore::new())
    }
}

/// Fallback keystore implementation for unsupported platforms
pub struct FallbackKeystore {
    storage: tokio::sync::RwLock<HashMap<String, Vec<u8>>>,
}

impl FallbackKeystore {
    pub fn new() -> Self {
        Self {
            storage: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl PlatformKeystore for FallbackKeystore {
    async fn store_secret(&self, key: &str, value: &[u8]) -> ConfigResult<()> {
        let mut storage = self.storage.write().await;
        storage.insert(key.to_string(), value.to_vec());
        Ok(())
    }
    
    async fn get_secret(&self, key: &str) -> ConfigResult<Option<Vec<u8>>> {
        let storage = self.storage.read().await;
        Ok(storage.get(key).cloned())
    }
    
    async fn delete_secret(&self, key: &str) -> ConfigResult<()> {
        let mut storage = self.storage.write().await;
        storage.remove(key);
        Ok(())
    }
    
    async fn list_keys(&self) -> ConfigResult<Vec<String>> {
        let storage = self.storage.read().await;
        Ok(storage.keys().cloned().collect())
    }
    
    fn is_available(&self) -> bool {
        true
    }
    
    fn keystore_type(&self) -> &'static str {
        "fallback"
    }
}

/// Utility functions for keystore operations
pub mod utils {
    use super::*;
    use blake3::Hasher;
    use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, Algorithm, Version, Params};
    use argon2::password_hash::{SaltString, rand_core::OsRng};

    /// Generate a secure key from password using Argon2
    pub fn derive_key(password: &str, salt: &[u8], config: &KeyDerivationConfig) -> ConfigResult<Vec<u8>> {
        let params = Params::new(
            Params::DEFAULT_M_COST,
            config.iterations,
            Params::DEFAULT_P_COST,
            Some(config.key_length),
        ).map_err(|e| ConfigError::encryption(format!("Invalid Argon2 params: {}", e)))?;

        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let salt_string = SaltString::encode_b64(salt)
            .map_err(|e| ConfigError::encryption(format!("Salt encoding failed: {}", e)))?;
        
        let hash = argon2.hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| ConfigError::encryption(format!("Key derivation failed: {}", e)))?;
        
        Ok(hash.hash.unwrap().as_bytes().to_vec())
    }

    /// Generate random salt
    pub fn generate_salt(length: usize) -> Vec<u8> {
        use rand::RngCore;
        let mut salt = vec![0u8; length];
        OsRng.fill_bytes(&mut salt);
        salt
    }

    /// Create secure storage key for configuration entry
    pub fn create_storage_key(service: &str, key: &str) -> String {
        let mut hasher = Hasher::new();
        hasher.update(service.as_bytes());
        hasher.update(b"::");
        hasher.update(key.as_bytes());
        format!("datafold:{}", hex::encode(hasher.finalize().as_bytes()))
    }

    /// Encrypt data using AES-GCM
    pub fn encrypt_data(data: &[u8], key: &[u8]) -> ConfigResult<Vec<u8>> {
        use aes_gcm::{Aes256Gcm, KeyInit, Nonce, AeadInPlace};
        use rand::RngCore;

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| ConfigError::encryption(format!("Invalid encryption key: {}", e)))?;
        
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let mut buffer = data.to_vec();
        let tag = cipher.encrypt_in_place_detached(nonce, b"", &mut buffer)
            .map_err(|e| ConfigError::encryption(format!("Encryption failed: {}", e)))?;

        // Prepend nonce and append tag
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&buffer);
        result.extend_from_slice(&tag);

        Ok(result)
    }

    /// Decrypt data using AES-GCM
    pub fn decrypt_data(encrypted_data: &[u8], key: &[u8]) -> ConfigResult<Vec<u8>> {
        use aes_gcm::{Aes256Gcm, KeyInit, Nonce, AeadInPlace, Tag};

        if encrypted_data.len() < 28 { // 12 (nonce) + 16 (tag) = minimum
            return Err(ConfigError::encryption("Invalid encrypted data length"));
        }

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| ConfigError::encryption(format!("Invalid decryption key: {}", e)))?;

        let nonce = Nonce::from_slice(&encrypted_data[0..12]);
        let tag = Tag::from_slice(&encrypted_data[encrypted_data.len()-16..]);
        let mut ciphertext = encrypted_data[12..encrypted_data.len()-16].to_vec();

        cipher.decrypt_in_place_detached(nonce, b"", &mut ciphertext, tag)
            .map_err(|e| ConfigError::encryption(format!("Decryption failed: {}", e)))?;

        Ok(ciphertext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fallback_keystore() {
        let keystore = FallbackKeystore::new();
        assert!(keystore.is_available());
        assert_eq!(keystore.keystore_type(), "fallback");

        let key = "test_key";
        let value = b"test_value";

        // Store secret
        keystore.store_secret(key, value).await.unwrap();

        // Retrieve secret
        let retrieved = keystore.get_secret(key).await.unwrap();
        assert_eq!(retrieved, Some(value.to_vec()));

        // List keys
        let keys = keystore.list_keys().await.unwrap();
        assert!(keys.contains(&key.to_string()));

        // Delete secret
        keystore.delete_secret(key).await.unwrap();
        let retrieved = keystore.get_secret(key).await.unwrap();
        assert_eq!(retrieved, None);
    }

    #[test]
    fn test_key_derivation() {
        let config = KeyDerivationConfig::default();
        let password = "test_password";
        let salt = utils::generate_salt(config.salt_length);

        let key1 = utils::derive_key(password, &salt, &config).unwrap();
        let key2 = utils::derive_key(password, &salt, &config).unwrap();

        assert_eq!(key1, key2);
        assert_eq!(key1.len(), config.key_length);
    }

    #[test]
    fn test_encryption_decryption() {
        let key = utils::generate_salt(32); // 256-bit key
        let data = b"Hello, World!";

        let encrypted = utils::encrypt_data(data, &key).unwrap();
        let decrypted = utils::decrypt_data(&encrypted, &key).unwrap();

        assert_eq!(data, &decrypted[..]);
        assert_ne!(data.to_vec(), encrypted);
    }
}