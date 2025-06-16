//! Core cross-platform configuration management system
//!
//! This module provides the foundational configuration management system that works
//! across all platforms, implementing the core traits and abstractions from the
//! architecture design.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::error::{ConfigError, ConfigResult};
use crate::config::platform::{create_platform_resolver, get_platform_info, PlatformConfigPaths};
use crate::config::value::{ConfigValue, ConfigValueSchema};

/// Core trait for configuration providers (format-agnostic)
#[async_trait]
pub trait ConfigurationProvider: Send + Sync {
    /// Load configuration from the canonical location
    async fn load(&self) -> ConfigResult<Config>;

    /// Save configuration to the canonical location
    async fn save(&self, config: &Config) -> ConfigResult<()>;

    /// Reload configuration (for runtime reloadability)
    async fn reload(&self) -> ConfigResult<Config>;

    /// Validate configuration without loading
    async fn validate(&self, config: &Config) -> ConfigResult<()>;

    /// Check if configuration file exists
    async fn exists(&self) -> ConfigResult<bool>;

    /// Get configuration file path
    fn config_path(&self) -> ConfigResult<PathBuf>;

    /// Get provider type identifier
    fn provider_type(&self) -> &'static str;
}

/// TOML-based configuration provider
pub struct TomlConfigProvider {
    platform_paths: Box<dyn PlatformConfigPaths>,
    config_file_override: Option<PathBuf>,
}

impl TomlConfigProvider {
    /// Create new TOML configuration provider
    pub fn new() -> Self {
        Self {
            platform_paths: create_platform_resolver(),
            config_file_override: None,
        }
    }

    /// Create with custom configuration file path
    pub fn with_path<P: AsRef<Path>>(path: P) -> Self {
        Self {
            platform_paths: create_platform_resolver(),
            config_file_override: Some(path.as_ref().to_path_buf()),
        }
    }

    /// Get the actual configuration file path
    fn get_config_path(&self) -> ConfigResult<PathBuf> {
        if let Some(override_path) = &self.config_file_override {
            Ok(override_path.clone())
        } else {
            self.platform_paths.config_file()
        }
    }

    /// Ensure directories exist
    async fn ensure_directories(&self) -> ConfigResult<()> {
        self.platform_paths.ensure_directories()
    }

    /// Migrate legacy JSON configuration if it exists
    async fn migrate_legacy_config(&self) -> ConfigResult<Option<Config>> {
        let legacy_path = self.platform_paths.legacy_config_file()?;

        if legacy_path.exists() {
            // Read legacy JSON config
            let content = tokio::fs::read_to_string(&legacy_path)
                .await
                .map_err(ConfigError::from)?;

            // Parse as JSON first
            let json_value: serde_json::Value = serde_json::from_str(&content)?;

            // Convert to our Config structure (this would need custom logic)
            // For now, create a basic config and store the JSON data
            let mut config = Config::default();
            config.metadata.insert(
                "migrated_from".to_string(),
                ConfigValue::string(legacy_path.to_string_lossy()),
            );
            config.metadata.insert(
                "migration_timestamp".to_string(),
                ConfigValue::string(chrono::Utc::now().to_rfc3339()),
            );

            // Store the original JSON data for manual migration
            config
                .metadata
                .insert("legacy_data".to_string(), ConfigValue::string(content));

            Ok(Some(config))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl ConfigurationProvider for TomlConfigProvider {
    async fn load(&self) -> ConfigResult<Config> {
        let config_path = self.get_config_path()?;

        // Ensure directories exist
        self.ensure_directories().await?;

        // Check if config file exists
        if !config_path.exists() {
            // Try to migrate legacy configuration
            if let Some(migrated_config) = self.migrate_legacy_config().await? {
                // Save the migrated config as TOML
                self.save(&migrated_config).await?;
                return Ok(migrated_config);
            }

            // No config file exists, return default
            let default_config = Config::default();

            // Save default config for future use
            self.save(&default_config).await?;

            return Ok(default_config);
        }

        // Read and parse TOML file
        let content = tokio::fs::read_to_string(&config_path)
            .await
            .map_err(ConfigError::from)?;

        let config: Config = toml::from_str(&content).map_err(ConfigError::from)?;

        // Validate the loaded configuration
        self.validate(&config).await?;

        Ok(config)
    }

    async fn save(&self, config: &Config) -> ConfigResult<()> {
        let config_path = self.get_config_path()?;

        // Ensure directories exist
        self.ensure_directories().await?;

        // Validate before saving
        self.validate(config).await?;

        // Serialize to TOML
        let toml_content = toml::to_string_pretty(config).map_err(ConfigError::from)?;

        // Write atomically by writing to temp file first
        let temp_path = config_path.with_extension("toml.tmp");

        tokio::fs::write(&temp_path, toml_content)
            .await
            .map_err(ConfigError::from)?;

        // Atomic rename
        tokio::fs::rename(&temp_path, &config_path)
            .await
            .map_err(ConfigError::from)?;

        Ok(())
    }

    async fn reload(&self) -> ConfigResult<Config> {
        // For now, reload is the same as load
        // Could be enhanced with file watching and caching
        self.load().await
    }

    async fn validate(&self, config: &Config) -> ConfigResult<()> {
        // Basic validation
        if config.version.is_empty() {
            return Err(ConfigError::validation(
                "Configuration version cannot be empty",
            ));
        }

        // Validate platform compatibility
        let platform_info = get_platform_info();
        if let Some(platform_reqs) = &config.platform_requirements {
            if !platform_reqs.is_compatible(&platform_info.name) {
                return Err(ConfigError::validation(format!(
                    "Configuration not compatible with platform '{}'",
                    platform_info.name
                )));
            }
        }

        // Validate sections
        for (section_name, section_data) in &config.sections {
            if section_data.is_null() {
                return Err(ConfigError::validation(format!(
                    "Configuration section '{}' cannot be null",
                    section_name
                )));
            }
        }

        Ok(())
    }

    async fn exists(&self) -> ConfigResult<bool> {
        let config_path = self.get_config_path()?;
        Ok(config_path.exists())
    }

    fn config_path(&self) -> ConfigResult<PathBuf> {
        self.get_config_path()
    }

    fn provider_type(&self) -> &'static str {
        "toml"
    }
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Configuration format version
    pub version: String,

    /// Configuration creation timestamp
    pub created_at: String,

    /// Last modification timestamp
    pub updated_at: String,

    /// Platform-specific requirements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_requirements: Option<PlatformRequirements>,

    /// Configuration sections (organized by feature/component)
    pub sections: HashMap<String, ConfigValue>,

    /// Metadata and administrative information
    pub metadata: HashMap<String, ConfigValue>,

    /// Security configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<SecurityConfig>,
}

impl Config {
    /// Create a new configuration with defaults
    pub fn new() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        let platform_info = get_platform_info();

        let mut metadata = HashMap::new();
        metadata.insert(
            "platform".to_string(),
            ConfigValue::string(platform_info.name),
        );
        metadata.insert(
            "architecture".to_string(),
            ConfigValue::string(platform_info.arch),
        );

        Self {
            version: "1.0.0".to_string(),
            created_at: now.clone(),
            updated_at: now,
            platform_requirements: None,
            sections: HashMap::new(),
            metadata,
            security: None,
        }
    }

    /// Get a configuration section
    pub fn get_section(&self, name: &str) -> ConfigResult<&ConfigValue> {
        self.sections
            .get(name)
            .ok_or_else(|| ConfigError::not_found(format!("Section '{}'", name)))
    }

    /// Get a mutable configuration section
    pub fn get_section_mut(&mut self, name: &str) -> ConfigResult<&mut ConfigValue> {
        self.sections
            .get_mut(name)
            .ok_or_else(|| ConfigError::not_found(format!("Section '{}'", name)))
    }

    /// Set a configuration section
    pub fn set_section(&mut self, name: String, value: ConfigValue) {
        self.sections.insert(name, value);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// Remove a configuration section
    pub fn remove_section(&mut self, name: &str) -> Option<ConfigValue> {
        let result = self.sections.remove(name);
        if result.is_some() {
            self.updated_at = chrono::Utc::now().to_rfc3339();
        }
        result
    }

    /// Merge with another configuration
    pub fn merge(&mut self, other: Config) -> ConfigResult<()> {
        // Merge sections
        for (name, value) in other.sections {
            match self.sections.get_mut(&name) {
                Some(existing) => {
                    let merged = existing.clone().merge(value)?;
                    self.sections.insert(name, merged);
                }
                None => {
                    self.sections.insert(name, value);
                }
            }
        }

        // Merge metadata
        for (name, value) in other.metadata {
            self.metadata.insert(name, value);
        }

        self.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }

    /// Get a value from a specific section by path (e.g., "logging.level")
    pub fn get_value(&self, path: &str) -> ConfigResult<&ConfigValue> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return Err(ConfigError::validation("Empty path"));
        }

        let section = self.get_section(parts[0])?;
        let mut current = section;

        for part in &parts[1..] {
            current = current.get(part)?;
        }

        Ok(current)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Platform requirements for configuration compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformRequirements {
    /// Supported platforms
    pub supported_platforms: Vec<String>,

    /// Minimum versions by platform
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub minimum_versions: HashMap<String, String>,

    /// Platform-specific features required
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub required_features: HashMap<String, Vec<String>>,
}

impl PlatformRequirements {
    /// Check if this configuration is compatible with the given platform
    pub fn is_compatible(&self, platform: &str) -> bool {
        self.supported_platforms
            .iter()
            .any(|p| p == platform || p == "*")
    }
}

/// Security configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Whether to encrypt sensitive configuration data
    pub encrypt_sensitive_data: bool,

    /// Key derivation parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_derivation: Option<HashMap<String, ConfigValue>>,

    /// Access control settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_control: Option<HashMap<String, ConfigValue>>,
}

/// Unified configuration API
pub struct ConfigurationManager {
    provider: Arc<dyn ConfigurationProvider>,
    cache: Arc<RwLock<Option<Arc<Config>>>>,
    // Event hooks for change tracking would go here
}

impl ConfigurationManager {
    /// Create new configuration manager with default TOML provider
    pub fn new() -> Self {
        Self {
            provider: Arc::new(TomlConfigProvider::new()),
            cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with custom provider
    pub fn with_provider(provider: Arc<dyn ConfigurationProvider>) -> Self {
        Self {
            provider,
            cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with custom TOML config file path
    pub fn with_toml_file<P: AsRef<Path>>(path: P) -> Self {
        Self {
            provider: Arc::new(TomlConfigProvider::with_path(path)),
            cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Get current configuration (cached or loaded)
    pub async fn get(&self) -> ConfigResult<Arc<Config>> {
        // Check cache first
        {
            let cache_read = self.cache.read().await;
            if let Some(cached_config) = cache_read.as_ref() {
                return Ok(cached_config.clone());
            }
        }

        // Load configuration
        let config = self.provider.load().await?;
        let config_arc = Arc::new(config);

        // Update cache
        {
            let mut cache_write = self.cache.write().await;
            *cache_write = Some(config_arc.clone());
        }

        Ok(config_arc)
    }

    /// Set new configuration
    pub async fn set(&self, new_config: Config) -> ConfigResult<()> {
        // Save configuration
        self.provider.save(&new_config).await?;

        // Update cache
        {
            let mut cache_write = self.cache.write().await;
            *cache_write = Some(Arc::new(new_config));
        }

        // TODO: Emit configuration change event

        Ok(())
    }

    /// Reload configuration from storage
    pub async fn reload(&self) -> ConfigResult<Arc<Config>> {
        let config = self.provider.reload().await?;
        let config_arc = Arc::new(config);

        // Update cache
        {
            let mut cache_write = self.cache.write().await;
            *cache_write = Some(config_arc.clone());
        }

        // TODO: Emit configuration reload event

        Ok(config_arc)
    }

    /// Clear cache (force reload on next get)
    pub async fn clear_cache(&self) {
        let mut cache_write = self.cache.write().await;
        *cache_write = None;
    }

    /// Check if configuration exists
    pub async fn exists(&self) -> ConfigResult<bool> {
        self.provider.exists().await
    }

    /// Get configuration file path
    pub fn config_path(&self) -> ConfigResult<PathBuf> {
        self.provider.config_path()
    }

    /// Get provider type
    pub fn provider_type(&self) -> &'static str {
        self.provider.provider_type()
    }
}

impl Default for ConfigurationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_toml_provider_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        let provider = TomlConfigProvider::with_path(&config_path);

        // Test save and load
        let mut config = Config::new();
        config.set_section("test".to_string(), ConfigValue::string("value"));

        provider.save(&config).await.unwrap();
        assert!(provider.exists().await.unwrap());

        let loaded_config = provider.load().await.unwrap();
        assert_eq!(
            loaded_config
                .get_section("test")
                .unwrap()
                .as_string()
                .unwrap(),
            "value"
        );
    }

    #[tokio::test]
    async fn test_configuration_manager() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        let manager = ConfigurationManager::with_toml_file(&config_path);

        // Test get (should create default config)
        let config = manager.get().await.unwrap();
        assert_eq!(config.version, "1.0.0");

        // Test set
        let mut new_config = Config::new();
        new_config.set_section("app".to_string(), ConfigValue::string("test_app"));

        manager.set(new_config).await.unwrap();

        // Test reload
        manager.clear_cache().await;
        let reloaded_config = manager.get().await.unwrap();
        assert_eq!(
            reloaded_config
                .get_section("app")
                .unwrap()
                .as_string()
                .unwrap(),
            "test_app"
        );
    }

    #[test]
    fn test_config_value_operations() {
        let mut config = Config::new();

        // Test section operations
        config.set_section(
            "logging".to_string(),
            ConfigValue::object({
                let mut obj = HashMap::new();
                obj.insert("level".to_string(), ConfigValue::string("info"));
                obj.insert("enabled".to_string(), ConfigValue::boolean(true));
                obj
            }),
        );

        // Test path-based access
        let level = config.get_value("logging.level").unwrap();
        assert_eq!(level.as_string().unwrap(), "info");

        let enabled = config.get_value("logging.enabled").unwrap();
        assert_eq!(enabled.as_bool().unwrap(), true);
    }

    #[test]
    fn test_config_merge() {
        let mut config1 = Config::new();
        config1.set_section("section1".to_string(), ConfigValue::string("value1"));

        let mut config2 = Config::new();
        config2.set_section("section2".to_string(), ConfigValue::string("value2"));
        config2.set_section("section1".to_string(), ConfigValue::string("new_value1"));

        config1.merge(config2).unwrap();

        assert_eq!(
            config1
                .get_section("section1")
                .unwrap()
                .as_string()
                .unwrap(),
            "new_value1"
        );
        assert_eq!(
            config1
                .get_section("section2")
                .unwrap()
                .as_string()
                .unwrap(),
            "value2"
        );
    }
}
