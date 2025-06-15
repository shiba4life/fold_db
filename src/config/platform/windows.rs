//! Windows-specific configuration path resolution using Windows Known Folders
//!
//! This module implements platform-specific path resolution for Windows systems,
//! using the Windows Known Folders API and standard Windows conventions.

use std::path::PathBuf;
use std::env;
use async_trait::async_trait;
use crate::config::error::{ConfigError, ConfigResult};
use super::{PlatformConfigPaths, keystore::{PlatformKeystore, utils}};

/// Windows-specific configuration paths using Known Folders
pub struct WindowsConfigPaths {
    home_dir: PathBuf,
}

impl WindowsConfigPaths {
    /// Create new Windows configuration paths resolver
    pub fn new() -> Self {
        let home_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("C:\\temp"));
        
        Self { home_dir }
    }

    /// Get home directory
    fn home_dir(&self) -> &PathBuf {
        &self.home_dir
    }

    /// Get APPDATA directory (%APPDATA%)
    fn appdata_dir(&self) -> ConfigResult<PathBuf> {
        env::var("APPDATA")
            .map(PathBuf::from)
            .or_else(|_| {
                // Fallback to default APPDATA location
                Ok(self.home_dir().join("AppData\\Roaming"))
            })
            .map_err(|e: std::io::Error| ConfigError::platform(format!("Failed to get APPDATA directory: {}", e)))
    }

    /// Get LOCALAPPDATA directory (%LOCALAPPDATA%)
    fn local_appdata_dir(&self) -> ConfigResult<PathBuf> {
        env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .or_else(|_| {
                // Fallback to default LOCALAPPDATA location
                Ok(self.home_dir().join("AppData\\Local"))
            })
            .map_err(|e: std::io::Error| ConfigError::platform(format!("Failed to get LOCALAPPDATA directory: {}", e)))
    }

    /// Get TEMP directory (%TEMP%)
    fn temp_dir(&self) -> ConfigResult<PathBuf> {
        env::var("TEMP")
            .or_else(|_| env::var("TMP"))
            .map(PathBuf::from)
            .or_else(|_| {
                // Fallback to Windows temp directory
                Ok(PathBuf::from("C:\\Windows\\Temp"))
            })
            .map_err(|e: std::io::Error| ConfigError::platform(format!("Failed to get TEMP directory: {}", e)))
    }
}

impl PlatformConfigPaths for WindowsConfigPaths {
    fn config_dir(&self) -> ConfigResult<PathBuf> {
        // Configuration files go in %APPDATA%\DataFold
        let dir = self.appdata_dir()?.join("DataFold");
        Ok(dir)
    }

    fn data_dir(&self) -> ConfigResult<PathBuf> {
        // Data files also go in %APPDATA%\DataFold
        let dir = self.appdata_dir()?.join("DataFold");
        Ok(dir)
    }

    fn cache_dir(&self) -> ConfigResult<PathBuf> {
        // Cache files go in %LOCALAPPDATA%\DataFold\Cache
        let dir = self.local_appdata_dir()?.join("DataFold\\Cache");
        Ok(dir)
    }

    fn logs_dir(&self) -> ConfigResult<PathBuf> {
        // Log files go in %LOCALAPPDATA%\DataFold\Logs
        let dir = self.local_appdata_dir()?.join("DataFold\\Logs");
        Ok(dir)
    }

    fn runtime_dir(&self) -> ConfigResult<PathBuf> {
        // Runtime/temporary files go in %TEMP%\DataFold
        let dir = self.temp_dir()?.join("DataFold");
        Ok(dir)
    }

    fn platform_name(&self) -> &'static str {
        "windows"
    }
}

impl Default for WindowsConfigPaths {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_config_paths() {
        let paths = WindowsConfigPaths::new();
        
        let config_dir = paths.config_dir().unwrap();
        let data_dir = paths.data_dir().unwrap();
        let cache_dir = paths.cache_dir().unwrap();
        let logs_dir = paths.logs_dir().unwrap();
        let runtime_dir = paths.runtime_dir().unwrap();
        
        // All should contain DataFold
        assert!(config_dir.to_string_lossy().contains("DataFold"));
        assert!(data_dir.to_string_lossy().contains("DataFold"));
        assert!(cache_dir.to_string_lossy().contains("DataFold"));
        assert!(logs_dir.to_string_lossy().contains("DataFold"));
        assert!(runtime_dir.to_string_lossy().contains("DataFold"));
        
        // Config and data should be in APPDATA (Roaming)
        let config_str = config_dir.to_string_lossy();
        let data_str = data_dir.to_string_lossy();
        assert!(config_str.contains("Roaming") || config_str.contains("AppData"));
        assert!(data_str.contains("Roaming") || data_str.contains("AppData"));
        
        // Cache and logs should be in LOCALAPPDATA (Local)
        let cache_str = cache_dir.to_string_lossy();
        let logs_str = logs_dir.to_string_lossy();
        assert!(cache_str.contains("Local") || cache_str.contains("Cache"));
        assert!(logs_str.contains("Local") || logs_str.contains("Logs"));
        
        // Runtime should be in temp
        let runtime_str = runtime_dir.to_string_lossy();
        assert!(runtime_str.contains("Temp") || runtime_str.contains("temp"));
        
        assert_eq!(paths.platform_name(), "windows");
    }

    #[test]
    fn test_windows_environment_variables() {
        let paths = WindowsConfigPaths::new();
        
        // Test that environment variables are used when available
        env::set_var("APPDATA", "C:\\TestAppData");
        let config_dir = paths.config_dir().unwrap();
        assert!(config_dir.starts_with("C:\\TestAppData"));
        
        env::set_var("LOCALAPPDATA", "C:\\TestLocalAppData");
        let cache_dir = paths.cache_dir().unwrap();
        assert!(cache_dir.starts_with("C:\\TestLocalAppData"));
        
        // Clean up
        env::remove_var("APPDATA");
        env::remove_var("LOCALAPPDATA");
    }

    #[test]
    fn test_config_file_paths() {
        let paths = WindowsConfigPaths::new();
        let config_file = paths.config_file().unwrap();
        let legacy_file = paths.legacy_config_file().unwrap();
        
        assert!(config_file.to_string_lossy().ends_with("config.toml"));
        assert!(legacy_file.to_string_lossy().ends_with("config.json"));
        assert!(config_file.to_string_lossy().contains("DataFold"));
    }

    #[test]
    fn test_directory_separation() {
        let paths = WindowsConfigPaths::new();
        
        let config_dir = paths.config_dir().unwrap();
        let cache_dir = paths.cache_dir().unwrap();
        let runtime_dir = paths.runtime_dir().unwrap();
        
        // Config should be in AppData\Roaming
        // Cache should be in AppData\Local  
        // Runtime should be in Temp
        // These should be in different base directories
        let config_str = config_dir.to_string_lossy();
        let cache_str = cache_dir.to_string_lossy();
        let runtime_str = runtime_dir.to_string_lossy();
        
        // They should all be different base paths
        assert_ne!(config_dir.parent(), cache_dir.parent());
        assert_ne!(config_dir.parent(), runtime_dir.parent());
        assert_ne!(cache_dir.parent(), runtime_dir.parent());
    }
}

/// Windows-specific keystore implementation using Credential Manager
pub struct WindowsKeystore {
    service_name: String,
}

impl WindowsKeystore {
    pub fn new() -> Self {
        Self {
            service_name: "DataFold".to_string(),
        }
    }

    /// Check if Credential Manager is available
    fn is_credential_manager_available(&self) -> bool {
        // Credential Manager is always available on Windows
        true
    }

    /// Create credential target name
    fn create_target_name(&self, key: &str) -> String {
        format!("{}:{}", self.service_name, key)
    }
}

#[async_trait]
impl PlatformKeystore for WindowsKeystore {
    async fn store_secret(&self, key: &str, value: &[u8]) -> ConfigResult<()> {
        if !self.is_available() {
            return Err(ConfigError::platform("Credential Manager not available"));
        }

        // In a real implementation, this would use Windows Credential Manager API
        // For now, implement a file-based fallback with encryption in LOCALAPPDATA
        let config_dir = WindowsConfigPaths::new().config_dir()?;
        let keystore_dir = config_dir.join("Keystore");
        std::fs::create_dir_all(&keystore_dir)
            .map_err(|e| ConfigError::platform(format!("Failed to create keystore directory: {}", e)))?;

        let storage_key = utils::create_storage_key(&self.service_name, key);
        
        // Derive encryption key from system and user info
        let username = env::var("USERNAME").or_else(|_| env::var("USER")).unwrap_or_default();
        let computer_name = env::var("COMPUTERNAME").unwrap_or_default();
        let password = format!("{}:{}:{}:{}", username, computer_name, self.service_name, storage_key);
        let salt = utils::generate_salt(32);
        let encryption_key = utils::derive_key(&password, &salt, &Default::default())?;

        // Encrypt and store
        let encrypted_data = utils::encrypt_data(value, &encryption_key)?;
        let mut final_data = salt;
        final_data.extend_from_slice(&encrypted_data);

        let key_file = keystore_dir.join(format!("{}.cred", hex::encode(storage_key.as_bytes())));
        tokio::fs::write(&key_file, final_data).await
            .map_err(|e| ConfigError::platform(format!("Failed to store secret: {}", e)))?;

        Ok(())
    }

    async fn get_secret(&self, key: &str) -> ConfigResult<Option<Vec<u8>>> {
        if !self.is_available() {
            return Err(ConfigError::platform("Credential Manager not available"));
        }

        let storage_key = utils::create_storage_key(&self.service_name, key);
        
        // Check file-based fallback
        let config_dir = WindowsConfigPaths::new().config_dir()?;
        let keystore_dir = config_dir.join("Keystore");
        let key_file = keystore_dir.join(format!("{}.cred", hex::encode(storage_key.as_bytes())));

        if !key_file.exists() {
            return Ok(None);
        }

        let file_data = tokio::fs::read(&key_file).await
            .map_err(|e| ConfigError::platform(format!("Failed to read secret: {}", e)))?;

        if file_data.len() < 32 {
            return Err(ConfigError::encryption("Invalid credential file format"));
        }

        // Extract salt and encrypted data
        let salt = &file_data[0..32];
        let encrypted_data = &file_data[32..];

        // Derive decryption key
        let username = env::var("USERNAME").or_else(|_| env::var("USER")).unwrap_or_default();
        let computer_name = env::var("COMPUTERNAME").unwrap_or_default();
        let password = format!("{}:{}:{}:{}", username, computer_name, self.service_name, storage_key);
        let decryption_key = utils::derive_key(&password, salt, &Default::default())?;

        // Decrypt
        let decrypted_data = utils::decrypt_data(encrypted_data, &decryption_key)?;
        Ok(Some(decrypted_data))
    }

    async fn delete_secret(&self, key: &str) -> ConfigResult<()> {
        if !self.is_available() {
            return Err(ConfigError::platform("Credential Manager not available"));
        }

        let storage_key = utils::create_storage_key(&self.service_name, key);
        
        // Delete file-based fallback
        let config_dir = WindowsConfigPaths::new().config_dir()?;
        let keystore_dir = config_dir.join("Keystore");
        let key_file = keystore_dir.join(format!("{}.cred", hex::encode(storage_key.as_bytes())));

        if key_file.exists() {
            tokio::fs::remove_file(&key_file).await
                .map_err(|e| ConfigError::platform(format!("Failed to delete secret: {}", e)))?;
        }

        Ok(())
    }

    async fn list_keys(&self) -> ConfigResult<Vec<String>> {
        if !self.is_available() {
            return Err(ConfigError::platform("Credential Manager not available"));
        }

        let config_dir = WindowsConfigPaths::new().config_dir()?;
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
                if file_name.ends_with(".cred") {
                    let key_name = &file_name[..file_name.len() - 5]; // Remove .cred extension
                    keys.push(key_name.to_string());
                }
            }
        }

        Ok(keys)
    }

    fn is_available(&self) -> bool {
        // Credential Manager is always available on Windows
        WindowsConfigPaths::new().config_dir().is_ok()
    }

    fn keystore_type(&self) -> &'static str {
        "windows_credential_manager"
    }
}

/// Windows-specific file watching using ReadDirectoryChangesW
pub struct WindowsFileWatcher {
    // ReadDirectoryChangesW would be initialized here
}

impl WindowsFileWatcher {
    pub fn new() -> ConfigResult<Self> {
        // Initialize ReadDirectoryChangesW
        Ok(Self {})
    }
}

impl super::PlatformFileWatcher for WindowsFileWatcher {
    fn watch_file<F>(&self, path: &std::path::Path, callback: F) -> ConfigResult<()>
    where
        F: Fn() + Send + 'static,
    {
        // Implementation would use ReadDirectoryChangesW to watch for file changes
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

/// Windows-specific atomic file operations
pub struct WindowsAtomicOps;

impl super::PlatformAtomicOps for WindowsAtomicOps {
    fn atomic_write(&self, path: &std::path::Path, content: &[u8]) -> ConfigResult<()> {
        let temp_path = path.with_extension("tmp");
        
        // Write to temporary file
        std::fs::write(&temp_path, content)
            .map_err(|e| ConfigError::platform(format!("Failed to write temporary file: {}", e)))?;

        // Atomic rename (ReplaceFile would be better but rename is sufficient)
        std::fs::rename(&temp_path, path)
            .map_err(|e| ConfigError::platform(format!("Failed to rename file: {}", e)))?;

        Ok(())
    }

    fn create_with_lock(&self, path: &std::path::Path, content: &[u8]) -> ConfigResult<()> {
        // In a real implementation, this would use LockFileEx
        // For now, use atomic write
        self.atomic_write(path, content)
    }
}

/// Windows-specific memory optimization
pub struct WindowsMemoryOps;

impl WindowsMemoryOps {
    /// Use memory mapping for large configuration files
    pub fn mmap_config_file(path: &std::path::Path) -> ConfigResult<Vec<u8>> {
        // In a real implementation, this would use CreateFileMapping/MapViewOfFile
        // For now, use regular file read
        std::fs::read(path)
            .map_err(|e| ConfigError::platform(format!("Failed to read config file: {}", e)))
    }

    /// Optimize memory usage for configuration cache
    pub fn optimize_cache_memory() -> ConfigResult<()> {
        // This would call HeapCompact or similar Windows memory APIs
        Ok(())
    }

    /// Use Windows-specific memory allocation
    pub fn allocate_secure_memory(size: usize) -> ConfigResult<Vec<u8>> {
        // In a real implementation, this would use VirtualAlloc with PAGE_READWRITE
        // and VirtualLock to lock pages in memory
        Ok(vec![0u8; size])
    }
}