//! CLI configuration management for authentication and settings
//!
//! This module handles configuration storage and retrieval for DataFold CLI,
//! including authentication profiles, server settings, and user preferences.
//! Now supports both JSON and TOML configuration formats.

use crate::cli::auth::{CliAuthProfile, CliAuthStatus};
use crate::cli::signing_config::{EnhancedSigningConfig, SigningMode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Errors that can occur during CLI configuration operations
#[derive(Debug, thiserror::Error)]
pub enum CliConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML serialization error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("Configuration validation error: {0}")]
    Validation(String),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Invalid configuration path: {0}")]
    InvalidPath(String),
}

pub type CliConfigResult<T> = Result<T, CliConfigError>;

/// CLI configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// Version of the configuration format
    pub version: String,
    /// Default authentication profile to use
    pub default_profile: Option<String>,
    /// Authentication profiles
    pub profiles: HashMap<String, CliAuthProfile>,
    /// Server configurations
    pub servers: HashMap<String, ServerConfig>,
    /// Global CLI settings
    pub settings: CliSettings,
    /// Enhanced signing configuration
    pub signing: EnhancedSigningConfig,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Server configuration for different DataFold instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server URL
    pub url: String,
    /// Server display name
    pub name: String,
    /// API version to use
    pub api_version: Option<String>,
    /// Custom headers to include in requests
    pub headers: HashMap<String, String>,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Whether to verify SSL certificates
    pub verify_ssl: bool,
}

/// Global CLI settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliSettings {
    /// Default output format (json, yaml, table, etc.)
    pub output_format: String,
    /// Whether to use colored output
    pub colored_output: bool,
    /// Verbosity level (0-3)
    pub verbosity: u8,
    /// Whether to automatically check for CLI updates
    pub auto_update_check: bool,
    /// Default request timeout in seconds
    pub default_timeout_secs: u64,
    /// Default retry configuration
    pub default_max_retries: u32,
    /// Whether to store authentication tokens
    pub store_auth_tokens: bool,
    /// Configuration for signature verification
    pub signature_settings: SignatureSettings,
}

/// Settings for request signing behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureSettings {
    /// Whether to include timestamp in signatures by default
    pub include_timestamp: bool,
    /// Whether to include nonce for replay protection
    pub include_nonce: bool,
    /// Default signature components to include
    pub default_components: Vec<String>,
    /// Maximum request body size for digest calculation
    pub max_body_size_mb: u64,
    /// Whether to verify server responses
    pub verify_responses: bool,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            default_profile: None,
            profiles: HashMap::new(),
            servers: HashMap::new(),
            settings: CliSettings::default(),
            signing: EnhancedSigningConfig::default(),
            updated_at: Utc::now(),
        }
    }
}

impl Default for CliSettings {
    fn default() -> Self {
        Self {
            output_format: "table".to_string(),
            colored_output: true,
            verbosity: 1,
            auto_update_check: true,
            default_timeout_secs: 30,
            default_max_retries: 3,
            store_auth_tokens: true,
            signature_settings: SignatureSettings::default(),
        }
    }
}

impl Default for SignatureSettings {
    fn default() -> Self {
        Self {
            include_timestamp: true,
            include_nonce: true,
            default_components: vec![
                "@method".to_string(),
                "@target-uri".to_string(),
                "content-type".to_string(),
            ],
            max_body_size_mb: 10,
            verify_responses: false,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:8080".to_string(),
            name: "Local DataFold".to_string(),
            api_version: Some("v1".to_string()),
            headers: HashMap::new(),
            timeout_secs: 30,
            max_retries: 3,
            verify_ssl: true,
        }
    }
}

/// CLI configuration manager
pub struct CliConfigManager {
    config_path: PathBuf,
    config: CliConfig,
}

impl CliConfigManager {
    /// Create a new configuration manager with the default config path
    pub fn new() -> CliConfigResult<Self> {
        let config_path = Self::default_config_path()?;
        Self::with_path(config_path)
    }

    /// Create a configuration manager with a specific config path
    pub fn with_path<P: AsRef<Path>>(config_path: P) -> CliConfigResult<Self> {
        let config_path = config_path.as_ref().to_path_buf();

        let config = if config_path.exists() {
            Self::load_config(&config_path)?
        } else {
            CliConfig::default()
        };

        Ok(Self {
            config_path,
            config,
        })
    }

    /// Get the default configuration file path (TOML format)
    pub fn default_config_path() -> CliConfigResult<PathBuf> {
        let home_dir = dirs::home_dir().ok_or_else(|| {
            CliConfigError::InvalidPath("Unable to determine home directory".to_string())
        })?;

        Ok(home_dir.join(".datafold").join("config.toml"))
    }

    /// Get the legacy JSON configuration file path
    pub fn legacy_config_path() -> CliConfigResult<PathBuf> {
        let home_dir = dirs::home_dir().ok_or_else(|| {
            CliConfigError::InvalidPath("Unable to determine home directory".to_string())
        })?;

        Ok(home_dir.join(".datafold").join("config.json"))
    }

    /// Get the default key storage directory
    pub fn default_keys_dir() -> CliConfigResult<PathBuf> {
        let home_dir = dirs::home_dir().ok_or_else(|| {
            CliConfigError::InvalidPath("Unable to determine home directory".to_string())
        })?;

        Ok(home_dir.join(".datafold").join("keys"))
    }

    /// Load configuration from file (supports both TOML and JSON)
    fn load_config(path: &Path) -> CliConfigResult<CliConfig> {
        let content = fs::read_to_string(path)?;

        // Handle empty files gracefully by returning default config
        if content.trim().is_empty() {
            return Ok(CliConfig::default());
        }

        // Determine format by file extension
        let config: CliConfig = if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            toml::from_str(&content)?
        } else {
            // Default to JSON for backward compatibility
            serde_json::from_str(&content)?
        };

        Ok(config)
    }

    /// Load configuration with automatic migration from JSON to TOML
    pub fn load_with_migration() -> CliConfigResult<Self> {
        let toml_path = Self::default_config_path()?;
        let json_path = Self::legacy_config_path()?;

        // Try TOML first
        if toml_path.exists() {
            return Self::with_path(toml_path);
        }

        // Try JSON and migrate if found
        if json_path.exists() {
            let mut manager = Self::with_path(&json_path)?;

            // Migrate to TOML
            manager.config_path = toml_path;
            manager.save()?;

            // Remove old JSON file
            let _ = fs::remove_file(&json_path);

            return Ok(manager);
        }

        // Create new TOML config
        Self::with_path(toml_path)
    }

    /// Save configuration to file (TOML or JSON based on file extension)
    pub fn save(&mut self) -> CliConfigResult<()> {
        // Update timestamp
        self.config.updated_at = Utc::now();

        // Ensure parent directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write configuration in appropriate format
        let content = if self.config_path.extension().and_then(|s| s.to_str()) == Some("toml") {
            toml::to_string_pretty(&self.config)?
        } else {
            serde_json::to_string_pretty(&self.config)?
        };

        fs::write(&self.config_path, content)?;

        Ok(())
    }

    /// Get the current configuration
    pub fn config(&self) -> &CliConfig {
        &self.config
    }

    /// Get mutable access to the configuration
    pub fn config_mut(&mut self) -> &mut CliConfig {
        &mut self.config
    }

    /// Add or update an authentication profile
    pub fn add_profile(&mut self, name: String, profile: CliAuthProfile) -> CliConfigResult<()> {
        self.config.profiles.insert(name.clone(), profile);

        // Set as default if no default is set
        if self.config.default_profile.is_none() {
            self.config.default_profile = Some(name);
        }

        Ok(())
    }

    /// Remove an authentication profile
    pub fn remove_profile(&mut self, name: &str) -> CliConfigResult<()> {
        if !self.config.profiles.contains_key(name) {
            return Err(CliConfigError::ProfileNotFound(name.to_string()));
        }

        self.config.profiles.remove(name);

        // Clear default if this was the default profile
        if self.config.default_profile.as_ref() == Some(&name.to_string()) {
            self.config.default_profile = None;
        }

        Ok(())
    }

    /// Get an authentication profile by name
    pub fn get_profile(&self, name: &str) -> Option<&CliAuthProfile> {
        self.config.profiles.get(name)
    }

    /// Get the default authentication profile
    pub fn get_default_profile(&self) -> Option<&CliAuthProfile> {
        self.config
            .default_profile
            .as_ref()
            .and_then(|name| self.config.profiles.get(name))
    }

    /// Set the default profile
    pub fn set_default_profile(&mut self, name: String) -> CliConfigResult<()> {
        if !self.config.profiles.contains_key(&name) {
            return Err(CliConfigError::ProfileNotFound(name));
        }

        self.config.default_profile = Some(name);
        Ok(())
    }

    /// List all profile names
    pub fn list_profiles(&self) -> Vec<&String> {
        self.config.profiles.keys().collect()
    }

    /// Add or update a server configuration
    pub fn add_server(&mut self, name: String, config: ServerConfig) {
        self.config.servers.insert(name, config);
    }

    /// Get a server configuration by name
    pub fn get_server(&self, name: &str) -> Option<&ServerConfig> {
        self.config.servers.get(name)
    }

    /// Remove a server configuration
    pub fn remove_server(&mut self, name: &str) {
        self.config.servers.remove(name);
    }

    /// List all server names
    pub fn list_servers(&self) -> Vec<&String> {
        self.config.servers.keys().collect()
    }

    /// Update CLI settings
    pub fn update_settings<F>(&mut self, updater: F) -> CliConfigResult<()>
    where
        F: FnOnce(&mut CliSettings),
    {
        updater(&mut self.config.settings);
        Ok(())
    }

    /// Get CLI authentication status
    pub fn auth_status(&self) -> CliAuthStatus {
        match self.get_default_profile() {
            Some(profile) => CliAuthStatus {
                configured: true,
                client_id: Some(profile.client_id.clone()),
                key_id: Some(profile.key_id.clone()),
                server_url: Some(profile.server_url.clone()),
                last_attempt: None,
                last_success: None,
            },
            None => CliAuthStatus::default(),
        }
    }

    /// Validate the current configuration
    pub fn validate(&self) -> CliConfigResult<()> {
        // Validate that default profile exists
        if let Some(default_name) = &self.config.default_profile {
            if !self.config.profiles.contains_key(default_name) {
                return Err(CliConfigError::Validation(format!(
                    "Default profile '{}' does not exist",
                    default_name
                )));
            }
        }

        // Validate profiles
        for (name, profile) in &self.config.profiles {
            if profile.client_id.is_empty() {
                return Err(CliConfigError::Validation(format!(
                    "Profile '{}' has empty client_id",
                    name
                )));
            }

            if profile.key_id.is_empty() {
                return Err(CliConfigError::Validation(format!(
                    "Profile '{}' has empty key_id",
                    name
                )));
            }

            if profile.server_url.is_empty() {
                return Err(CliConfigError::Validation(format!(
                    "Profile '{}' has empty server_url",
                    name
                )));
            }
        }

        // Validate servers
        for (name, server) in &self.config.servers {
            if server.url.is_empty() {
                return Err(CliConfigError::Validation(format!(
                    "Server '{}' has empty URL",
                    name
                )));
            }

            if server.timeout_secs == 0 {
                return Err(CliConfigError::Validation(format!(
                    "Server '{}' has invalid timeout",
                    name
                )));
            }
        }

        Ok(())
    }

    /// Reset configuration to defaults
    pub fn reset(&mut self) {
        self.config = CliConfig::default();
    }

    /// Get configuration file path
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Get signing configuration
    pub fn signing_config(&self) -> &EnhancedSigningConfig {
        &self.config.signing
    }

    /// Get mutable signing configuration
    pub fn signing_config_mut(&mut self) -> &mut EnhancedSigningConfig {
        &mut self.config.signing
    }

    /// Update signing mode for a command
    pub fn set_command_signing_mode(
        &mut self,
        command: String,
        mode: SigningMode,
    ) -> CliConfigResult<()> {
        self.config
            .signing
            .auto_signing
            .set_command_mode(command, mode);
        Ok(())
    }

    /// Get effective signing mode for a command
    pub fn get_command_signing_mode(&self, command: &str) -> SigningMode {
        self.config.signing.auto_signing.get_command_mode(command)
    }

    /// Enable or disable automatic signing globally
    pub fn set_auto_signing_enabled(&mut self, enabled: bool) {
        self.config.signing.auto_signing.enabled = enabled;
    }

    /// Set default signing mode
    pub fn set_default_signing_mode(&mut self, mode: SigningMode) {
        self.config.signing.auto_signing.default_mode = mode;
    }

    /// Enable or disable debug mode for signing
    pub fn set_signing_debug(&mut self, enabled: bool) {
        self.config.signing.debug.enabled = enabled;
    }

    /// Get signing status for a command
    pub fn get_signing_status(&self, command: &str) -> bool {
        self.config
            .signing
            .auto_signing
            .is_effective_auto_signing(command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_test_profile() -> CliAuthProfile {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());

        CliAuthProfile {
            client_id: "test-client-123".to_string(),
            key_id: "test-key".to_string(),
            user_id: Some("test-user".to_string()),
            server_url: "https://api.example.com".to_string(),
            metadata,
        }
    }

    #[test]
    fn test_cli_config_defaults() {
        let config = CliConfig::default();
        assert_eq!(config.version, "1.0");
        assert!(config.default_profile.is_none());
        assert!(config.profiles.is_empty());
        assert!(config.servers.is_empty());
        assert_eq!(config.settings.output_format, "table");
        assert!(config.settings.colored_output);
        assert_eq!(config.settings.verbosity, 1);
    }

    #[test]
    fn test_config_manager_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_path = temp_file.path();

        let manager = CliConfigManager::with_path(config_path).unwrap();
        assert_eq!(manager.config_path(), config_path);
        assert!(manager.config().profiles.is_empty());
    }

    #[test]
    fn test_profile_management() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut manager = CliConfigManager::with_path(temp_file.path()).unwrap();

        let profile = create_test_profile();
        manager
            .add_profile("test".to_string(), profile.clone())
            .unwrap();

        assert_eq!(manager.list_profiles().len(), 1);
        assert!(manager.list_profiles().contains(&&"test".to_string()));

        let retrieved = manager.get_profile("test").unwrap();
        assert_eq!(retrieved.client_id, profile.client_id);
        assert_eq!(retrieved.key_id, profile.key_id);

        // Should be set as default since it's the first profile
        assert_eq!(manager.config().default_profile.as_ref().unwrap(), "test");

        let default = manager.get_default_profile().unwrap();
        assert_eq!(default.client_id, profile.client_id);
    }

    #[test]
    fn test_server_management() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut manager = CliConfigManager::with_path(temp_file.path()).unwrap();

        let server_config = ServerConfig {
            url: "https://test.example.com".to_string(),
            name: "Test Server".to_string(),
            api_version: Some("v2".to_string()),
            headers: HashMap::new(),
            timeout_secs: 60,
            max_retries: 5,
            verify_ssl: false,
        };

        manager.add_server("test".to_string(), server_config.clone());

        assert_eq!(manager.list_servers().len(), 1);
        assert!(manager.list_servers().contains(&&"test".to_string()));

        let retrieved = manager.get_server("test").unwrap();
        assert_eq!(retrieved.url, server_config.url);
        assert_eq!(retrieved.name, server_config.name);
        assert_eq!(retrieved.timeout_secs, server_config.timeout_secs);
    }

    #[test]
    fn test_config_persistence() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_path = temp_file.path().to_path_buf();

        // Create and configure manager
        {
            let mut manager = CliConfigManager::with_path(&config_path).unwrap();
            let profile = create_test_profile();
            manager.add_profile("test".to_string(), profile).unwrap();
            manager.save().unwrap();
        }

        // Load from saved file
        {
            let manager = CliConfigManager::with_path(&config_path).unwrap();
            assert_eq!(manager.list_profiles().len(), 1);

            let profile = manager.get_profile("test").unwrap();
            assert_eq!(profile.client_id, "test-client-123");
        }
    }

    #[test]
    fn test_config_validation() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut manager = CliConfigManager::with_path(temp_file.path()).unwrap();

        // Valid configuration should pass
        let profile = create_test_profile();
        manager.add_profile("test".to_string(), profile).unwrap();
        assert!(manager.validate().is_ok());

        // Invalid profile should fail validation
        let mut invalid_profile = create_test_profile();
        invalid_profile.client_id = String::new();
        manager
            .add_profile("invalid".to_string(), invalid_profile)
            .unwrap();
        assert!(manager.validate().is_err());
    }
}
