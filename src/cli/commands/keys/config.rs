//! Configuration and setup for key operations
//! 
//! This module handles configuration management for key operations,
//! including storage paths, security settings, and default parameters.

use crate::cli::args::CliSecurityLevel;
use crate::cli::commands::keys::error::{KeyError, KeyResult};
use crate::cli::utils::key_utils::{ensure_storage_dir, get_default_storage_dir};
use crate::crypto::Argon2Params;
use std::path::{Path, PathBuf};

/// Configuration for key management operations
#[derive(Debug, Clone)]
pub struct KeyManagementConfig {
    /// Default storage directory for keys
    pub storage_dir: PathBuf,
    /// Default security level for new keys
    pub default_security_level: CliSecurityLevel,
    /// Whether to create backups by default during rotation
    pub auto_backup_on_rotation: bool,
    /// Maximum number of backup versions to keep
    pub max_backup_versions: u32,
    /// Default export format preference
    pub default_export_format: String,
    /// Whether to include metadata in exports by default
    pub include_metadata_in_exports: bool,
}

impl Default for KeyManagementConfig {
    fn default() -> Self {
        Self {
            storage_dir: get_default_storage_dir().unwrap_or_else(|_| PathBuf::from(".datafold/keys")),
            default_security_level: CliSecurityLevel::Balanced,
            auto_backup_on_rotation: true,
            max_backup_versions: 5,
            default_export_format: "json".to_string(),
            include_metadata_in_exports: true,
        }
    }
}

impl KeyManagementConfig {
    /// Create a new configuration with custom storage directory
    pub fn with_storage_dir(storage_dir: PathBuf) -> Self {
        Self {
            storage_dir,
            ..Default::default()
        }
    }

    /// Create a new configuration with custom security level
    pub fn with_security_level(security_level: CliSecurityLevel) -> Self {
        Self {
            default_security_level: security_level,
            ..Default::default()
        }
    }

    /// Validate and prepare the configuration
    pub fn validate(&self) -> KeyResult<()> {
        // Ensure storage directory exists and has proper permissions
        ensure_storage_dir(&self.storage_dir)
            .map_err(|e| KeyError::ConfigurationError(format!("Invalid storage directory: {}", e)))?;

        // Validate backup version limits
        if self.max_backup_versions > 100 {
            return Err(KeyError::ConfigurationError(
                "Maximum backup versions cannot exceed 100".to_string()
            ));
        }

        // Validate export format
        if !matches!(self.default_export_format.as_str(), "json" | "binary") {
            return Err(KeyError::ConfigurationError(
                "Invalid default export format. Must be 'json' or 'binary'".to_string()
            ));
        }

        Ok(())
    }

    /// Get Argon2 parameters for the configured security level
    pub fn get_argon2_params(&self) -> Argon2Params {
        match self.default_security_level {
            CliSecurityLevel::Interactive => Argon2Params::interactive(),
            CliSecurityLevel::Balanced => Argon2Params::default(),
            CliSecurityLevel::Sensitive => Argon2Params::sensitive(),
        }
    }

    /// Load configuration from file (simplified implementation)
    pub fn load_from_file(_config_path: &Path) -> KeyResult<Self> {
        // For now, just return default config
        // TODO: Implement proper serialization when CliSecurityLevel supports serde
        Ok(Self::default())
    }

    /// Save configuration to file (simplified implementation)
    pub fn save_to_file(&self, _config_path: &Path) -> KeyResult<()> {
        self.validate()?;
        // TODO: Implement proper serialization when CliSecurityLevel supports serde
        Ok(())
    }
}

/// Get the default configuration file path
pub fn get_default_config_path() -> KeyResult<PathBuf> {
    let storage_dir = get_default_storage_dir()
        .map_err(|e| KeyError::ConfigurationError(format!("Failed to get storage directory: {}", e)))?;
    Ok(storage_dir.join("config.json"))
}

/// Initialize key management with default configuration
pub fn initialize_key_management() -> KeyResult<KeyManagementConfig> {
    let config = KeyManagementConfig::default();
    config.validate()?;
    Ok(config)
}

/// Initialize key management with custom storage directory
pub fn initialize_with_storage_dir(storage_dir: PathBuf) -> KeyResult<KeyManagementConfig> {
    let config = KeyManagementConfig::with_storage_dir(storage_dir);
    config.validate()?;
    Ok(config)
}