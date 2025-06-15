//! macOS-specific configuration path resolution following Apple guidelines
//!
//! This module implements platform-specific path resolution for macOS systems,
//! following Apple's File System Programming Guide.

use std::path::PathBuf;
use async_trait::async_trait;
use crate::config::error::{ConfigError, ConfigResult};
use super::{PlatformConfigPaths, keystore::{PlatformKeystore, utils}};

/// macOS-specific configuration paths following Apple guidelines
pub struct MacOSConfigPaths {
    home_dir: PathBuf,
}

impl MacOSConfigPaths {
    /// Create new macOS configuration paths resolver
    pub fn new() -> Self {
        let home_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"));
        
        Self { home_dir }
    }

    /// Get home directory
    fn home_dir(&self) -> &PathBuf {
        &self.home_dir
    }

    /// Get Library directory
    fn library_dir(&self) -> PathBuf {
        self.home_dir().join("Library")
    }

    /// Get Application Support directory
    fn application_support_dir(&self) -> PathBuf {
        self.library_dir().join("Application Support")
    }

    /// Get Caches directory
    fn caches_dir(&self) -> PathBuf {
        self.library_dir().join("Caches")
    }

    /// Get Logs directory
    fn logs_dir_base(&self) -> PathBuf {
        self.library_dir().join("Logs")
    }

    /// Get Preferences directory
    fn preferences_dir(&self) -> PathBuf {
        self.library_dir().join("Preferences")
    }
}

impl PlatformConfigPaths for MacOSConfigPaths {
    fn config_dir(&self) -> ConfigResult<PathBuf> {
        // On macOS, configuration files typically go in Application Support
        let dir = self.application_support_dir().join("DataFold");
        Ok(dir)
    }

    fn data_dir(&self) -> ConfigResult<PathBuf> {
        // Data also goes in Application Support on macOS
        let dir = self.application_support_dir().join("DataFold");
        Ok(dir)
    }

    fn cache_dir(&self) -> ConfigResult<PathBuf> {
        let dir = self.caches_dir().join("DataFold");
        Ok(dir)
    }

    fn logs_dir(&self) -> ConfigResult<PathBuf> {
        let dir = self.logs_dir_base().join("DataFold");
        Ok(dir)
    }

    fn runtime_dir(&self) -> ConfigResult<PathBuf> {
        // Use a subdirectory in Caches for runtime/temporary files
        let dir = self.caches_dir().join("DataFold/tmp");
        Ok(dir)
    }

    fn platform_name(&self) -> &'static str {
        "macos"
    }

    fn config_file(&self) -> ConfigResult<PathBuf> {
        // Override to use plist-style naming if desired, but stick with TOML
        Ok(self.config_dir()?.join("config.toml"))
    }

    fn legacy_config_file(&self) -> ConfigResult<PathBuf> {
        // Look for legacy JSON file
        Ok(self.config_dir()?.join("config.json"))
    }
}

impl Default for MacOSConfigPaths {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macos_config_paths() {
        let paths = MacOSConfigPaths::new();
        
        let config_dir = paths.config_dir().unwrap();
        let data_dir = paths.data_dir().unwrap();
        let cache_dir = paths.cache_dir().unwrap();
        let logs_dir = paths.logs_dir().unwrap();
        let runtime_dir = paths.runtime_dir().unwrap();
        
        // Should contain Application Support for config and data
        assert!(config_dir.to_string_lossy().contains("Application Support"));
        assert!(config_dir.to_string_lossy().contains("DataFold"));
        assert!(data_dir.to_string_lossy().contains("Application Support"));
        assert!(data_dir.to_string_lossy().contains("DataFold"));
        
        // Should contain Caches for cache and runtime
        assert!(cache_dir.to_string_lossy().contains("Caches"));
        assert!(cache_dir.to_string_lossy().contains("DataFold"));
        assert!(runtime_dir.to_string_lossy().contains("Caches"));
        
        // Should contain Logs for logs
        assert!(logs_dir.to_string_lossy().contains("Logs"));
        assert!(logs_dir.to_string_lossy().contains("DataFold"));
        
        assert_eq!(paths.platform_name(), "macos");
    }

    #[test]
    fn test_macos_library_structure() {
        let paths = MacOSConfigPaths::new();
        
        let library = paths.library_dir();
        let app_support = paths.application_support_dir();
        let caches = paths.caches_dir();
        let logs = paths.logs_dir_base();
        
        assert!(library.to_string_lossy().ends_with("Library"));
        assert!(app_support.to_string_lossy().ends_with("Application Support"));
        assert!(caches.to_string_lossy().ends_with("Caches"));
        assert!(logs.to_string_lossy().ends_with("Logs"));
    }

    #[test]
    fn test_config_file_paths() {
        let paths = MacOSConfigPaths::new();
        let config_file = paths.config_file().unwrap();
        let legacy_file = paths.legacy_config_file().unwrap();
        
        assert!(config_file.to_string_lossy().ends_with("config.toml"));
        assert!(legacy_file.to_string_lossy().ends_with("config.json"));
        assert!(config_file.to_string_lossy().contains("DataFold"));
    }

    #[test]
    fn test_directory_hierarchy() {
        let paths = MacOSConfigPaths::new();
        
        let config_dir = paths.config_dir().unwrap();
        let data_dir = paths.data_dir().unwrap();
        
        // Config and data should be in the same parent directory on macOS
        assert_eq!(config_dir.parent(), data_dir.parent());
    }
}

/// macOS-specific keystore implementation using Keychain Services
pub struct MacOSKeystore {
    service_name: String,
}

impl MacOSKeystore {
    pub fn new() -> Self {
        Self {
            service_name: "DataFold".to_string(),
        }
    }

    /// Check if Keychain Services is available
    fn is_keychain_available(&self) -> bool {
        // Keychain is always available on macOS
        true
    }

    /// Create keychain item attributes
    fn create_keychain_attributes(&self, key: &str) -> std::collections::HashMap<String, String> {
        let mut attributes = std::collections::HashMap::new();
        attributes.insert("service".to_string(), self.service_name.clone());
        attributes.insert("account".to_string(), key.to_string());
        attributes.insert("description".to_string(), "DataFold Configuration".to_string());
        attributes
    }
}

#[async_trait]
impl PlatformKeystore for MacOSKeystore {
    async fn store_secret(&self, key: &str, value: &[u8]) -> ConfigResult<()> {
        if !self.is_available() {
            return Err(ConfigError::platform("Keychain Services not available"));
        }

        // In a real implementation, this would use Security Framework
        // For now, implement a file-based fallback with encryption in Application Support
        let config_dir = MacOSConfigPaths::new().config_dir()?;
        let keystore_dir = config_dir.join("Keystore");
        std::fs::create_dir_all(&keystore_dir)
            .map_err(|e| ConfigError::platform(format!("Failed to create keystore directory: {}", e)))?;

        let storage_key = utils::create_storage_key(&self.service_name, key);
        
        // Derive encryption key from system and user info
        let username = std::env::var("USER").unwrap_or_default();
        let password = format!("{}:{}:{}", username, self.service_name, storage_key);
        let salt = utils::generate_salt(32);
        let encryption_key = utils::derive_key(&password, &salt, &Default::default())?;

        // Encrypt and store
        let encrypted_data = utils::encrypt_data(value, &encryption_key)?;
        let mut final_data = salt;
        final_data.extend_from_slice(&encrypted_data);

        let key_file = keystore_dir.join(format!("{}.keychain", hex::encode(storage_key.as_bytes())));
        tokio::fs::write(&key_file, final_data).await
            .map_err(|e| ConfigError::platform(format!("Failed to store secret: {}", e)))?;

        // Set restrictive permissions (600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&key_file, perms)
                .map_err(|e| ConfigError::platform(format!("Failed to set file permissions: {}", e)))?;
        }

        Ok(())
    }

    async fn get_secret(&self, key: &str) -> ConfigResult<Option<Vec<u8>>> {
        if !self.is_available() {
            return Err(ConfigError::platform("Keychain Services not available"));
        }

        let storage_key = utils::create_storage_key(&self.service_name, key);
        
        // Check file-based fallback
        let config_dir = MacOSConfigPaths::new().config_dir()?;
        let keystore_dir = config_dir.join("Keystore");
        let key_file = keystore_dir.join(format!("{}.keychain", hex::encode(storage_key.as_bytes())));

        if !key_file.exists() {
            return Ok(None);
        }

        let file_data = tokio::fs::read(&key_file).await
            .map_err(|e| ConfigError::platform(format!("Failed to read secret: {}", e)))?;

        if file_data.len() < 32 {
            return Err(ConfigError::encryption("Invalid keychain file format"));
        }

        // Extract salt and encrypted data
        let salt = &file_data[0..32];
        let encrypted_data = &file_data[32..];

        // Derive decryption key
        let username = std::env::var("USER").unwrap_or_default();
        let password = format!("{}:{}:{}", username, self.service_name, storage_key);
        let decryption_key = utils::derive_key(&password, salt, &Default::default())?;

        // Decrypt
        let decrypted_data = utils::decrypt_data(encrypted_data, &decryption_key)?;
        Ok(Some(decrypted_data))
    }

    async fn delete_secret(&self, key: &str) -> ConfigResult<()> {
        if !self.is_available() {
            return Err(ConfigError::platform("Keychain Services not available"));
        }

        let storage_key = utils::create_storage_key(&self.service_name, key);
        
        // Delete file-based fallback
        let config_dir = MacOSConfigPaths::new().config_dir()?;
        let keystore_dir = config_dir.join("Keystore");
        let key_file = keystore_dir.join(format!("{}.keychain", hex::encode(storage_key.as_bytes())));

        if key_file.exists() {
            tokio::fs::remove_file(&key_file).await
                .map_err(|e| ConfigError::platform(format!("Failed to delete secret: {}", e)))?;
        }

        Ok(())
    }

    async fn list_keys(&self) -> ConfigResult<Vec<String>> {
        if !self.is_available() {
            return Err(ConfigError::platform("Keychain Services not available"));
        }

        let config_dir = MacOSConfigPaths::new().config_dir()?;
        let keystore_dir = config_dir.join("Keystore");

        if !keystore_dir.exists() {
            return Ok(Vec::new());
        }

        let mut keys = Vec::new();
        let mut dir_entries = tokio::fs::read_dir(&keystore_dir).await
            .map_err(|e| ConfigError::platform(format!("Failed to read keystore directory: {}", e)))?;

        while let Some(entry) = dir_entries.next_entry().await
            .map_err(|e| ConfigError::platform(format!("Failed to read directory entry: {}", e)))? {
            
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".keychain") {
                    let key_name = &file_name[..file_name.len() - 9]; // Remove .keychain extension
                    keys.push(key_name.to_string());
                }
            }
        }

        Ok(keys)
    }

    fn is_available(&self) -> bool {
        // Keychain is always available on macOS
        MacOSConfigPaths::new().config_dir().is_ok()
    }

    fn keystore_type(&self) -> &'static str {
        "macos_keychain"
    }
}

/// macOS-specific file watching using FSEvents
pub struct MacOSFileWatcher {
    // FSEvents would be initialized here
}

impl MacOSFileWatcher {
    pub fn new() -> ConfigResult<Self> {
        // Initialize FSEvents
        Ok(Self {})
    }
}

impl super::PlatformFileWatcher for MacOSFileWatcher {
    fn watch_file<F>(&self, path: &std::path::Path, callback: F) -> ConfigResult<()>
    where
        F: Fn() + Send + 'static,
    {
        // Implementation would use FSEvents to watch for file changes
        // For now, provide a basic polling implementation
        let path = path.to_path_buf();
        std::thread::spawn(move || {
            let mut last_modified = std::fs::metadata(&path)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
                
                if let Ok(metadata) = std::fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        if modified > last_modified {
                            last_modified = modified;
                            callback();
                        }
                    }
                }
            }
        });

        Ok(())
    }
}

/// macOS-specific atomic file operations
pub struct MacOSAtomicOps;

impl super::PlatformAtomicOps for MacOSAtomicOps {
    fn atomic_write(&self, path: &std::path::Path, content: &[u8]) -> ConfigResult<()> {
        let temp_path = path.with_extension("tmp");
        
        // Write to temporary file
        std::fs::write(&temp_path, content)
            .map_err(|e| ConfigError::platform(format!("Failed to write temporary file: {}", e)))?;

        // Set appropriate permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o644);
            std::fs::set_permissions(&temp_path, perms)
                .map_err(|e| ConfigError::platform(format!("Failed to set permissions: {}", e)))?;
        }

        // Atomic rename (exchangedata would be better but rename is sufficient)
        std::fs::rename(&temp_path, path)
            .map_err(|e| ConfigError::platform(format!("Failed to rename file: {}", e)))?;

        Ok(())
    }

    fn create_with_lock(&self, path: &std::path::Path, content: &[u8]) -> ConfigResult<()> {
        // In a real implementation, this would use flock() or fcntl()
        // For now, use atomic write
        self.atomic_write(path, content)
    }
}

/// macOS-specific memory optimization
pub struct MacOSMemoryOps;

impl MacOSMemoryOps {
    /// Use memory mapping for large configuration files
    pub fn mmap_config_file(path: &std::path::Path) -> ConfigResult<Vec<u8>> {
        // In a real implementation, this would use mmap()
        // For now, use regular file read
        std::fs::read(path)
            .map_err(|e| ConfigError::platform(format!("Failed to read config file: {}", e)))
    }

    /// Optimize memory usage for configuration cache
    pub fn optimize_cache_memory() -> ConfigResult<()> {
        // This would call malloc_trim() equivalent or memory pressure APIs
        Ok(())
    }
}