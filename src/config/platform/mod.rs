//! Platform-specific configuration path resolution
//!
//! This module provides platform-aware configuration path resolution following
//! OS-specific conventions and best practices.

use std::path::PathBuf;
use crate::config::error::{ConfigError, ConfigResult};

pub mod linux;
pub mod macos;
pub mod windows;
pub mod keystore;

/// Core trait for platform-specific path resolution
pub trait PlatformConfigPaths: Send + Sync {
    /// Get the configuration directory path
    /// - Linux: $XDG_CONFIG_HOME/datafold or ~/.config/datafold
    /// - macOS: ~/Library/Application Support/DataFold
    /// - Windows: %APPDATA%\DataFold
    fn config_dir(&self) -> ConfigResult<PathBuf>;

    /// Get the data directory path
    /// - Linux: $XDG_DATA_HOME/datafold or ~/.local/share/datafold
    /// - macOS: ~/Library/Application Support/DataFold
    /// - Windows: %APPDATA%\DataFold
    fn data_dir(&self) -> ConfigResult<PathBuf>;

    /// Get the cache directory path
    /// - Linux: $XDG_CACHE_HOME/datafold or ~/.cache/datafold
    /// - macOS: ~/Library/Caches/DataFold
    /// - Windows: %LOCALAPPDATA%\DataFold\Cache
    fn cache_dir(&self) -> ConfigResult<PathBuf>;

    /// Get the logs directory path
    /// - Linux: $XDG_STATE_HOME/datafold/logs or ~/.local/state/datafold/logs
    /// - macOS: ~/Library/Logs/DataFold
    /// - Windows: %LOCALAPPDATA%\DataFold\Logs
    fn logs_dir(&self) -> ConfigResult<PathBuf>;

    /// Get the runtime/temporary directory path
    /// - Linux: $XDG_RUNTIME_DIR/datafold or /tmp/datafold-$USER
    /// - macOS: ~/Library/Caches/DataFold/tmp
    /// - Windows: %TEMP%\DataFold
    fn runtime_dir(&self) -> ConfigResult<PathBuf>;

    /// Get the main configuration file path
    fn config_file(&self) -> ConfigResult<PathBuf> {
        Ok(self.config_dir()?.join("config.toml"))
    }

    /// Get the legacy configuration file path (for migration)
    fn legacy_config_file(&self) -> ConfigResult<PathBuf> {
        Ok(self.config_dir()?.join("config.json"))
    }

    /// Get platform identifier
    fn platform_name(&self) -> &'static str;

    /// Check if all directories exist and are accessible
    fn validate_paths(&self) -> ConfigResult<()> {
        let dirs = vec![
            ("config", self.config_dir()?),
            ("data", self.data_dir()?),
            ("cache", self.cache_dir()?),
            ("logs", self.logs_dir()?),
            ("runtime", self.runtime_dir()?),
        ];

        for (name, dir) in dirs {
            if !dir.exists() {
                std::fs::create_dir_all(&dir).map_err(|e| {
                    ConfigError::platform(format!(
                        "Failed to create {} directory '{}': {}",
                        name,
                        dir.display(),
                        e
                    ))
                })?;
            }

            // Check if directory is accessible
            if !dir.is_dir() {
                return Err(ConfigError::platform(format!(
                    "{} path '{}' exists but is not a directory",
                    name,
                    dir.display()
                )));
            }

            // Test write access
            let test_file = dir.join(".datafold_access_test");
            match std::fs::write(&test_file, b"test") {
                Ok(_) => {
                    let _ = std::fs::remove_file(&test_file);
                }
                Err(e) => {
                    return Err(ConfigError::access_denied(format!(
                        "No write access to {} directory '{}': {}",
                        name,
                        dir.display(),
                        e
                    )));
                }
            }
        }

        Ok(())
    }

    /// Create all necessary directories
    fn ensure_directories(&self) -> ConfigResult<()> {
        let dirs = vec![
            self.config_dir()?,
            self.data_dir()?,
            self.cache_dir()?,
            self.logs_dir()?,
            self.runtime_dir()?,
        ];

        for dir in dirs {
            std::fs::create_dir_all(&dir).map_err(|e| {
                ConfigError::platform(format!(
                    "Failed to create directory '{}': {}",
                    dir.display(),
                    e
                ))
            })?;
        }

        Ok(())
    }
}

/// Enhanced platform capabilities with keystore support
pub struct EnhancedPlatformInfo {
    pub basic_info: PlatformInfo,
    pub keystore_available: bool,
    pub file_watching_available: bool,
    pub atomic_operations_available: bool,
    pub memory_mapping_available: bool,
}

impl EnhancedPlatformInfo {
    /// Detect enhanced platform capabilities
    pub fn detect() -> Self {
        let basic_info = PlatformInfo::detect();
        
        Self {
            keystore_available: basic_info.supports_keyring,
            file_watching_available: basic_info.supports_file_watching,
            atomic_operations_available: true, // All platforms support this
            memory_mapping_available: true,    // All platforms support this
            basic_info,
        }
    }
}

/// Create platform-specific file watcher
pub fn create_platform_file_watcher() -> ConfigResult<Box<dyn PlatformFileWatcher>> {
    #[cfg(target_os = "linux")]
    {
        Ok(Box::new(linux::LinuxFileWatcher::new()?))
    }
    #[cfg(target_os = "macos")]
    {
        Ok(Box::new(macos::MacOSFileWatcher::new()?))
    }
    #[cfg(target_os = "windows")]
    {
        Ok(Box::new(windows::WindowsFileWatcher::new()?))
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Ok(Box::new(FallbackFileWatcher::new()))
    }
}

/// Trait for platform-specific file watching
pub trait PlatformFileWatcher: Send + Sync {
    /// Watch a file for changes and call callback when changed
    fn watch_file<F>(&self, path: &std::path::Path, callback: F) -> ConfigResult<()>
    where
        F: Fn() + Send + 'static;
}

/// Fallback file watcher for unsupported platforms
pub struct FallbackFileWatcher;

impl FallbackFileWatcher {
    pub fn new() -> Self {
        Self
    }
}

impl PlatformFileWatcher for FallbackFileWatcher {
    fn watch_file<F>(&self, _path: &std::path::Path, _callback: F) -> ConfigResult<()>
    where
        F: Fn() + Send + 'static,
    {
        // No-op for unsupported platforms
        Ok(())
    }
}

/// Create platform-specific atomic operations handler
pub fn create_platform_atomic_ops() -> Box<dyn PlatformAtomicOps> {
    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxAtomicOps)
    }
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacOSAtomicOps)
    }
    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsAtomicOps)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Box::new(FallbackAtomicOps)
    }
}

/// Trait for platform-specific atomic operations
pub trait PlatformAtomicOps: Send + Sync {
    /// Perform atomic write operation
    fn atomic_write(&self, path: &std::path::Path, content: &[u8]) -> ConfigResult<()>;
    
    /// Create file with lock
    fn create_with_lock(&self, path: &std::path::Path, content: &[u8]) -> ConfigResult<()>;
}

/// Fallback atomic operations for unsupported platforms
pub struct FallbackAtomicOps;

impl PlatformAtomicOps for FallbackAtomicOps {
    fn atomic_write(&self, path: &std::path::Path, content: &[u8]) -> ConfigResult<()> {
        std::fs::write(path, content)
            .map_err(|e| ConfigError::platform(format!("Failed to write file: {}", e)))
    }
    
    fn create_with_lock(&self, path: &std::path::Path, content: &[u8]) -> ConfigResult<()> {
        self.atomic_write(path, content)
    }
}

/// Platform information and capabilities
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub name: String,
    pub version: String,
    pub arch: String,
    pub supports_xdg: bool,
    pub supports_keyring: bool,
    pub supports_file_watching: bool,
}

impl PlatformInfo {
    /// Detect current platform information
    pub fn detect() -> Self {
        let name = if cfg!(target_os = "linux") {
            "linux".to_string()
        } else if cfg!(target_os = "macos") {
            "macos".to_string()
        } else if cfg!(target_os = "windows") {
            "windows".to_string()
        } else {
            "unknown".to_string()
        };

        let arch = std::env::consts::ARCH.to_string();
        let version = "unknown".to_string(); // Could be enhanced with OS version detection

        let supports_xdg = cfg!(target_os = "linux");
        let supports_keyring = cfg!(any(target_os = "linux", target_os = "macos", target_os = "windows"));
        let supports_file_watching = true; // Most platforms support this

        Self {
            name,
            version,
            arch,
            supports_xdg,
            supports_keyring,
            supports_file_watching,
        }
    }

    /// Check if this is a Unix-like platform
    pub fn is_unix(&self) -> bool {
        matches!(self.name.as_str(), "linux" | "macos")
    }

    /// Check if this is Windows
    pub fn is_windows(&self) -> bool {
        self.name == "windows"
    }
}

/// Create platform-specific path resolver
pub fn create_platform_resolver() -> Box<dyn PlatformConfigPaths> {
    if cfg!(target_os = "linux") {
        Box::new(linux::LinuxConfigPaths::new())
    } else if cfg!(target_os = "macos") {
        Box::new(macos::MacOSConfigPaths::new())
    } else if cfg!(target_os = "windows") {
        Box::new(windows::WindowsConfigPaths::new())
    } else {
        // Fallback to Unix-like behavior for unknown platforms
        Box::new(linux::LinuxConfigPaths::new())
    }
}

/// Get platform information
pub fn get_platform_info() -> PlatformInfo {
    PlatformInfo::detect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let info = get_platform_info();
        assert!(!info.name.is_empty());
        assert!(!info.arch.is_empty());
    }

    #[test]
    fn test_platform_resolver_creation() {
        let resolver = create_platform_resolver();
        let config_dir = resolver.config_dir().unwrap();
        assert!(config_dir.to_string_lossy().contains("datafold") || 
                config_dir.to_string_lossy().contains("DataFold"));
    }

    #[test]
    fn test_config_file_paths() {
        let resolver = create_platform_resolver();
        let config_file = resolver.config_file().unwrap();
        let legacy_file = resolver.legacy_config_file().unwrap();
        
        assert!(config_file.to_string_lossy().ends_with("config.toml"));
        assert!(legacy_file.to_string_lossy().ends_with("config.json"));
    }
}