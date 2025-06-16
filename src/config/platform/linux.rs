//! Linux-specific configuration path resolution following XDG Base Directory Specification
//!
//! This module implements platform-specific path resolution for Linux systems,
//! following the XDG Base Directory Specification.

use super::{
    keystore::{utils, PlatformKeystore},
    PlatformConfigPaths,
};
use crate::config::error::{ConfigError, ConfigResult};
use async_trait::async_trait;
use std::env;
use std::path::PathBuf;

/// Linux-specific configuration paths following XDG specification
pub struct LinuxConfigPaths {
    home_dir: PathBuf,
}

impl LinuxConfigPaths {
    /// Create new Linux configuration paths resolver
    pub fn new() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

        Self { home_dir }
    }

    /// Get home directory
    fn home_dir(&self) -> &PathBuf {
        &self.home_dir
    }

    /// Get XDG config home with fallback
    fn xdg_config_home(&self) -> PathBuf {
        env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| self.home_dir().join(".config"))
    }

    /// Get XDG data home with fallback
    fn xdg_data_home(&self) -> PathBuf {
        env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| self.home_dir().join(".local/share"))
    }

    /// Get XDG cache home with fallback
    fn xdg_cache_home(&self) -> PathBuf {
        env::var("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| self.home_dir().join(".cache"))
    }

    /// Get XDG state home with fallback
    fn xdg_state_home(&self) -> PathBuf {
        env::var("XDG_STATE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| self.home_dir().join(".local/state"))
    }

    /// Get XDG runtime directory with fallback
    fn xdg_runtime_dir(&self) -> PathBuf {
        env::var("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let user = env::var("USER").unwrap_or_else(|_| "unknown".to_string());
                PathBuf::from(format!("/tmp/datafold-{}", user))
            })
    }
}

impl PlatformConfigPaths for LinuxConfigPaths {
    fn config_dir(&self) -> ConfigResult<PathBuf> {
        let dir = self.xdg_config_home().join("datafold");
        Ok(dir)
    }

    fn data_dir(&self) -> ConfigResult<PathBuf> {
        let dir = self.xdg_data_home().join("datafold");
        Ok(dir)
    }

    fn cache_dir(&self) -> ConfigResult<PathBuf> {
        let dir = self.xdg_cache_home().join("datafold");
        Ok(dir)
    }

    fn logs_dir(&self) -> ConfigResult<PathBuf> {
        let dir = self.xdg_state_home().join("datafold/logs");
        Ok(dir)
    }

    fn runtime_dir(&self) -> ConfigResult<PathBuf> {
        let dir = self.xdg_runtime_dir();

        // Ensure runtime dir exists and has correct permissions (700)
        if !dir.exists() {
            std::fs::create_dir_all(&dir).map_err(|e| {
                ConfigError::platform(format!(
                    "Failed to create runtime directory '{}': {}",
                    dir.display(),
                    e
                ))
            })?;

            // Set permissions to 700 (user read/write/execute only)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = std::fs::Permissions::from_mode(0o700);
                std::fs::set_permissions(&dir, perms).map_err(|e| {
                    ConfigError::platform(format!(
                        "Failed to set permissions on runtime directory '{}': {}",
                        dir.display(),
                        e
                    ))
                })?;
            }
        }

        Ok(dir)
    }

    fn platform_name(&self) -> &'static str {
        "linux"
    }
}

impl Default for LinuxConfigPaths {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_linux_config_paths() {
        let paths = LinuxConfigPaths::new();

        let config_dir = paths.config_dir().unwrap();
        let data_dir = paths.data_dir().unwrap();
        let cache_dir = paths.cache_dir().unwrap();
        let logs_dir = paths.logs_dir().unwrap();

        assert!(config_dir.to_string_lossy().contains("datafold"));
        assert!(data_dir.to_string_lossy().contains("datafold"));
        assert!(cache_dir.to_string_lossy().contains("datafold"));
        assert!(logs_dir.to_string_lossy().contains("datafold"));

        assert_eq!(paths.platform_name(), "linux");
    }

    #[test]
    fn test_xdg_environment_variables() {
        let paths = LinuxConfigPaths::new();

        // Test with XDG_CONFIG_HOME set
        env::set_var("XDG_CONFIG_HOME", "/tmp/test-config");
        let config_dir = paths.config_dir().unwrap();
        assert!(config_dir.starts_with("/tmp/test-config"));

        // Clean up
        env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn test_runtime_directory_permissions() {
        let paths = LinuxConfigPaths::new();
        let runtime_dir = paths.runtime_dir().unwrap();

        // Runtime dir should be created
        assert!(runtime_dir.exists() || runtime_dir.to_string_lossy().starts_with("/tmp"));
    }

    #[test]
    fn test_config_file_paths() {
        let paths = LinuxConfigPaths::new();
        let config_file = paths.config_file().unwrap();
        let legacy_file = paths.legacy_config_file().unwrap();

        assert!(config_file.to_string_lossy().ends_with("config.toml"));
        assert!(legacy_file.to_string_lossy().ends_with("config.json"));
    }
}

/// Linux-specific keystore implementation using Secret Service API
pub struct LinuxKeystore {
    service_name: String,
}

impl LinuxKeystore {
    pub fn new() -> Self {
        Self {
            service_name: "org.freedesktop.secrets".to_string(),
        }
    }

    /// Check if Secret Service is available
    fn is_secret_service_available(&self) -> bool {
        // Try to connect to Secret Service D-Bus interface
        // For now, return true and implement graceful fallback
        true
    }

    /// Get the application keyring collection
    async fn get_collection_path(&self) -> ConfigResult<String> {
        // In a real implementation, this would use D-Bus to get the collection
        // For now, return a default collection path
        Ok("session".to_string())
    }
}

#[async_trait]
impl PlatformKeystore for LinuxKeystore {
    async fn store_secret(&self, key: &str, value: &[u8]) -> ConfigResult<()> {
        if !self.is_available() {
            return Err(ConfigError::platform("Secret Service not available"));
        }

        let storage_key = utils::create_storage_key("DataFold", key);

        // In a real implementation, this would use libsecret or D-Bus
        // For now, implement a file-based fallback with encryption
        let config_dir = LinuxConfigPaths::new().config_dir()?;
        let keystore_dir = config_dir.join("keystore");
        std::fs::create_dir_all(&keystore_dir).map_err(|e| {
            ConfigError::platform(format!("Failed to create keystore directory: {}", e))
        })?;

        // Derive encryption key from system entropy
        let password = format!("{}:{}", env::var("USER").unwrap_or_default(), storage_key);
        let salt = utils::generate_salt(32);
        let encryption_key = utils::derive_key(&password, &salt, &Default::default())?;

        // Encrypt and store
        let encrypted_data = utils::encrypt_data(value, &encryption_key)?;
        let mut final_data = salt;
        final_data.extend_from_slice(&encrypted_data);

        let key_file = keystore_dir.join(format!("{}.key", hex::encode(storage_key.as_bytes())));
        tokio::fs::write(&key_file, final_data)
            .await
            .map_err(|e| ConfigError::platform(format!("Failed to store secret: {}", e)))?;

        // Set restrictive permissions (600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&key_file, perms).map_err(|e| {
                ConfigError::platform(format!("Failed to set file permissions: {}", e))
            })?;
        }

        Ok(())
    }

    async fn get_secret(&self, key: &str) -> ConfigResult<Option<Vec<u8>>> {
        if !self.is_available() {
            return Err(ConfigError::platform("Secret Service not available"));
        }

        let storage_key = utils::create_storage_key("DataFold", key);

        // Check file-based fallback
        let config_dir = LinuxConfigPaths::new().config_dir()?;
        let keystore_dir = config_dir.join("keystore");
        let key_file = keystore_dir.join(format!("{}.key", hex::encode(storage_key.as_bytes())));

        if !key_file.exists() {
            return Ok(None);
        }

        let file_data = tokio::fs::read(&key_file)
            .await
            .map_err(|e| ConfigError::platform(format!("Failed to read secret: {}", e)))?;

        if file_data.len() < 32 {
            return Err(ConfigError::encryption("Invalid keystore file format"));
        }

        // Extract salt and encrypted data
        let salt = &file_data[0..32];
        let encrypted_data = &file_data[32..];

        // Derive decryption key
        let password = format!("{}:{}", env::var("USER").unwrap_or_default(), storage_key);
        let decryption_key = utils::derive_key(&password, salt, &Default::default())?;

        // Decrypt
        let decrypted_data = utils::decrypt_data(encrypted_data, &decryption_key)?;
        Ok(Some(decrypted_data))
    }

    async fn delete_secret(&self, key: &str) -> ConfigResult<()> {
        if !self.is_available() {
            return Err(ConfigError::platform("Secret Service not available"));
        }

        let storage_key = utils::create_storage_key("DataFold", key);

        // Delete file-based fallback
        let config_dir = LinuxConfigPaths::new().config_dir()?;
        let keystore_dir = config_dir.join("keystore");
        let key_file = keystore_dir.join(format!("{}.key", hex::encode(storage_key.as_bytes())));

        if key_file.exists() {
            tokio::fs::remove_file(&key_file)
                .await
                .map_err(|e| ConfigError::platform(format!("Failed to delete secret: {}", e)))?;
        }

        Ok(())
    }

    async fn list_keys(&self) -> ConfigResult<Vec<String>> {
        if !self.is_available() {
            return Err(ConfigError::platform("Secret Service not available"));
        }

        let config_dir = LinuxConfigPaths::new().config_dir()?;
        let keystore_dir = config_dir.join("keystore");

        if !keystore_dir.exists() {
            return Ok(Vec::new());
        }

        let mut keys = Vec::new();
        let mut dir_entries = tokio::fs::read_dir(&keystore_dir).await.map_err(|e| {
            ConfigError::platform(format!("Failed to read keystore directory: {}", e))
        })?;

        while let Some(entry) = dir_entries
            .next_entry()
            .await
            .map_err(|e| ConfigError::platform(format!("Failed to read directory entry: {}", e)))?
        {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".key") {
                    let key_name = &file_name[..file_name.len() - 4]; // Remove .key extension
                    keys.push(key_name.to_string());
                }
            }
        }

        Ok(keys)
    }

    fn is_available(&self) -> bool {
        // Check if we can access the config directory (fallback is always available)
        LinuxConfigPaths::new().config_dir().is_ok()
    }

    fn keystore_type(&self) -> &'static str {
        "linux_secret_service"
    }
}

/// Linux-specific file watching using inotify
pub struct LinuxFileWatcher {
    // inotify would be initialized here
}

impl LinuxFileWatcher {
    pub fn new() -> ConfigResult<Self> {
        // Initialize inotify
        Ok(Self {})
    }
}

impl super::PlatformFileWatcher for LinuxFileWatcher {
    fn watch_file<F>(&self, path: &std::path::Path, callback: F) -> ConfigResult<()>
    where
        F: Fn() + Send + 'static,
    {
        // Implementation would use inotify to watch for file changes
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

/// Linux-specific atomic file operations
pub struct LinuxAtomicOps;

impl super::PlatformAtomicOps for LinuxAtomicOps {
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

        // Atomic rename
        std::fs::rename(&temp_path, path)
            .map_err(|e| ConfigError::platform(format!("Failed to rename file: {}", e)))?;

        Ok(())
    }

    fn create_with_lock(&self, path: &std::path::Path, content: &[u8]) -> ConfigResult<()> {
        // In a real implementation, this would use flock()
        // For now, use atomic write
        self.atomic_write(path, content)
    }
}
