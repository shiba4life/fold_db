//! Configuration migration utilities for transitioning existing systems
//!
//! This module provides tools to migrate existing configuration systems to use
//! the new cross-platform configuration management, ensuring backward compatibility
//! and smooth transitions.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::config::cross_platform::{Config, ConfigurationManager};
use crate::config::enhanced::{EnhancedConfig, EnhancedConfigurationManager};
use crate::config::error::{ConfigError, ConfigResult};
use crate::config::platform::{create_platform_resolver, get_platform_info};
use crate::config::value::ConfigValue;

/// Migration result containing information about the migration process
#[derive(Debug, Clone)]
pub struct MigrationResult {
    /// Whether migration was successful
    pub success: bool,

    /// Source configuration path
    pub source_path: PathBuf,

    /// Target configuration path
    pub target_path: PathBuf,

    /// Number of sections migrated
    pub sections_migrated: usize,

    /// Migration warnings
    pub warnings: Vec<String>,

    /// Migration errors (non-fatal)
    pub errors: Vec<String>,

    /// Backup path (if backup was created)
    pub backup_path: Option<PathBuf>,
}

/// Configuration migration strategy
#[derive(Debug, Clone)]
pub enum MigrationStrategy {
    /// Copy existing configuration as-is
    DirectCopy,

    /// Transform configuration to new format
    Transform,

    /// Merge with existing new-format configuration
    Merge,

    /// Custom migration with user-provided function
    Custom(fn(&str) -> ConfigResult<String>),
}

/// Configuration migration manager
pub struct ConfigMigrationManager {
    platform_paths: Box<dyn crate::config::platform::PlatformConfigPaths>,
}

impl ConfigMigrationManager {
    /// Create new migration manager
    pub fn new() -> Self {
        Self {
            platform_paths: create_platform_resolver(),
        }
    }

    /// Migrate CLI configuration to new system
    pub async fn migrate_cli_config(&self) -> ConfigResult<MigrationResult> {
        // Attempt to find existing CLI configuration
        let legacy_paths = vec![
            PathBuf::from(dirs::home_dir().unwrap_or_default()).join(".datafold/config.json"),
            PathBuf::from(dirs::home_dir().unwrap_or_default()).join(".datafold/config.toml"),
        ];

        for legacy_path in legacy_paths {
            if legacy_path.exists() {
                return self
                    .migrate_config_file(
                        &legacy_path,
                        &self.platform_paths.config_file()?,
                        MigrationStrategy::Transform,
                    )
                    .await;
            }
        }

        // No legacy configuration found
        Ok(MigrationResult {
            success: true,
            source_path: PathBuf::new(),
            target_path: self.platform_paths.config_file()?,
            sections_migrated: 0,
            warnings: vec!["No legacy CLI configuration found".to_string()],
            errors: vec![],
            backup_path: None,
        })
    }

    /// Migrate logging configuration to use new paths
    pub async fn migrate_logging_config(&self) -> ConfigResult<MigrationResult> {
        // Default logging config path
        let legacy_path = PathBuf::from("config/logging.toml");
        let target_path = self.platform_paths.config_dir()?.join("logging.toml");

        if legacy_path.exists() {
            self.migrate_logging_config_file(&legacy_path, &target_path)
                .await
        } else {
            Ok(MigrationResult {
                success: true,
                source_path: legacy_path,
                target_path,
                sections_migrated: 0,
                warnings: vec!["No legacy logging configuration found".to_string()],
                errors: vec![],
                backup_path: None,
            })
        }
    }

    /// Migrate unified configuration to enhanced format
    pub async fn migrate_unified_config(&self) -> ConfigResult<MigrationResult> {
        let legacy_path = PathBuf::from("config/unified.json");
        let target_path = self.platform_paths.config_dir()?.join("unified.toml");

        if legacy_path.exists() {
            self.migrate_unified_config_file(&legacy_path, &target_path)
                .await
        } else {
            Ok(MigrationResult {
                success: true,
                source_path: legacy_path,
                target_path,
                sections_migrated: 0,
                warnings: vec!["No legacy unified configuration found".to_string()],
                errors: vec![],
                backup_path: None,
            })
        }
    }

    /// Migrate a configuration file with specified strategy
    pub async fn migrate_config_file(
        &self,
        source_path: &Path,
        target_path: &Path,
        strategy: MigrationStrategy,
    ) -> ConfigResult<MigrationResult> {
        let mut result = MigrationResult {
            success: false,
            source_path: source_path.to_path_buf(),
            target_path: target_path.to_path_buf(),
            sections_migrated: 0,
            warnings: vec![],
            errors: vec![],
            backup_path: None,
        };

        // Create backup of target if it exists
        if target_path.exists() {
            let backup_path = target_path.with_extension("bak");
            fs::copy(target_path, &backup_path)
                .await
                .map_err(|e| ConfigError::platform(format!("Failed to create backup: {}", e)))?;
            result.backup_path = Some(backup_path);
        }

        // Ensure target directory exists
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                ConfigError::platform(format!("Failed to create target directory: {}", e))
            })?;
        }

        match strategy {
            MigrationStrategy::DirectCopy => {
                fs::copy(source_path, target_path).await.map_err(|e| {
                    ConfigError::platform(format!("Failed to copy configuration: {}", e))
                })?;
                result.success = true;
                result.sections_migrated = 1;
            }

            MigrationStrategy::Transform => {
                let content = fs::read_to_string(source_path).await.map_err(|e| {
                    ConfigError::platform(format!("Failed to read source configuration: {}", e))
                })?;

                let transformed = self
                    .transform_config_content(&content, source_path, target_path)
                    .await?;

                fs::write(target_path, transformed).await.map_err(|e| {
                    ConfigError::platform(format!(
                        "Failed to write transformed configuration: {}",
                        e
                    ))
                })?;

                result.success = true;
                result.sections_migrated = 1;
            }

            MigrationStrategy::Merge => {
                // Load existing target configuration if it exists
                let mut target_config = if target_path.exists() {
                    let manager = ConfigurationManager::with_toml_file(target_path);
                    manager.get().await?
                } else {
                    std::sync::Arc::new(Config::default())
                };

                // Load source configuration
                let source_content = fs::read_to_string(source_path).await.map_err(|e| {
                    ConfigError::platform(format!("Failed to read source configuration: {}", e))
                })?;

                let source_config = self.parse_legacy_config(&source_content, source_path)?;

                // Merge configurations
                let mut merged_config = (*target_config).clone();
                merged_config.merge(source_config)?;

                // Save merged configuration
                let manager = ConfigurationManager::with_toml_file(target_path);
                manager.set(merged_config).await?;

                result.success = true;
                result.sections_migrated = 1;
            }

            MigrationStrategy::Custom(transform_fn) => {
                let content = fs::read_to_string(source_path).await.map_err(|e| {
                    ConfigError::platform(format!("Failed to read source configuration: {}", e))
                })?;

                let transformed = transform_fn(&content)?;

                fs::write(target_path, transformed).await.map_err(|e| {
                    ConfigError::platform(format!(
                        "Failed to write custom transformed configuration: {}",
                        e
                    ))
                })?;

                result.success = true;
                result.sections_migrated = 1;
            }
        }

        Ok(result)
    }

    /// Transform configuration content from legacy format to new format
    async fn transform_config_content(
        &self,
        content: &str,
        source_path: &Path,
        _target_path: &Path,
    ) -> ConfigResult<String> {
        // Determine source format by extension
        let source_extension = source_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("json");

        match source_extension {
            "json" => {
                // Parse JSON and convert to TOML
                let json_value: serde_json::Value = serde_json::from_str(content)
                    .map_err(|e| ConfigError::validation(format!("Invalid JSON: {}", e)))?;

                let config = self.json_to_config(json_value)?;

                toml::to_string_pretty(&config)
                    .map_err(|e| ConfigError::platform(format!("Failed to serialize TOML: {}", e)))
            }

            "toml" => {
                // Validate TOML and ensure it matches new schema
                let _: Config = toml::from_str(content)
                    .map_err(|e| ConfigError::validation(format!("Invalid TOML: {}", e)))?;

                Ok(content.to_string())
            }

            _ => Err(ConfigError::validation(format!(
                "Unsupported configuration format: {}",
                source_extension
            ))),
        }
    }

    /// Convert JSON value to Config structure
    fn json_to_config(&self, json: serde_json::Value) -> ConfigResult<Config> {
        let mut config = Config::new();

        if let serde_json::Value::Object(obj) = json {
            for (key, value) in obj {
                let config_value = self.json_value_to_config_value(value)?;
                config.set_section(key, config_value);
            }
        }

        Ok(config)
    }

    /// Convert JSON value to ConfigValue
    fn json_value_to_config_value(&self, value: serde_json::Value) -> ConfigResult<ConfigValue> {
        match value {
            serde_json::Value::Null => Ok(ConfigValue::String("".to_string())),
            serde_json::Value::Bool(b) => Ok(ConfigValue::boolean(b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(ConfigValue::integer(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(ConfigValue::float(f))
                } else {
                    Ok(ConfigValue::string(n.to_string()))
                }
            }
            serde_json::Value::String(s) => Ok(ConfigValue::string(s)),
            serde_json::Value::Array(arr) => {
                let mut config_arr = Vec::new();
                for item in arr {
                    config_arr.push(self.json_value_to_config_value(item)?);
                }
                Ok(ConfigValue::array(config_arr))
            }
            serde_json::Value::Object(obj) => {
                let mut config_obj = HashMap::new();
                for (key, value) in obj {
                    config_obj.insert(key, self.json_value_to_config_value(value)?);
                }
                Ok(ConfigValue::object(config_obj))
            }
        }
    }

    /// Parse legacy configuration from string content
    fn parse_legacy_config(&self, content: &str, source_path: &Path) -> ConfigResult<Config> {
        let extension = source_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("json");

        match extension {
            "json" => {
                let json_value: serde_json::Value = serde_json::from_str(content)?;
                self.json_to_config(json_value)
            }
            "toml" => toml::from_str(content)
                .map_err(|e| ConfigError::validation(format!("Invalid TOML: {}", e))),
            _ => Err(ConfigError::validation(format!(
                "Unsupported format: {}",
                extension
            ))),
        }
    }

    /// Migrate logging configuration file
    async fn migrate_logging_config_file(
        &self,
        source_path: &Path,
        target_path: &Path,
    ) -> ConfigResult<MigrationResult> {
        let content = fs::read_to_string(source_path)
            .await
            .map_err(|e| ConfigError::platform(format!("Failed to read logging config: {}", e)))?;

        // Parse existing logging config
        let logging_config: crate::logging::config::LogConfig = toml::from_str(&content)
            .map_err(|e| ConfigError::validation(format!("Invalid logging config: {}", e)))?;

        // Update file paths to use new platform-specific paths
        let mut updated_config = logging_config.clone();

        // Update file output path
        if updated_config.outputs.file.enabled {
            let logs_dir = self.platform_paths.logs_dir()?;
            let log_file_name = std::path::Path::new(&updated_config.outputs.file.path)
                .file_name()
                .unwrap_or(std::ffi::OsStr::new("datafold.log"));
            updated_config.outputs.file.path =
                logs_dir.join(log_file_name).to_string_lossy().to_string();
        }

        // Update structured output path if present
        if let Some(ref path) = updated_config.outputs.structured.path {
            let logs_dir = self.platform_paths.logs_dir()?;
            let log_file_name = std::path::Path::new(path)
                .file_name()
                .unwrap_or(std::ffi::OsStr::new("datafold-structured.json"));
            updated_config.outputs.structured.path =
                Some(logs_dir.join(log_file_name).to_string_lossy().to_string());
        }

        // Ensure target directory exists
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                ConfigError::platform(format!("Failed to create target directory: {}", e))
            })?;
        }

        // Write updated configuration
        let updated_content = toml::to_string_pretty(&updated_config).map_err(|e| {
            ConfigError::platform(format!("Failed to serialize logging config: {}", e))
        })?;

        fs::write(target_path, updated_content)
            .await
            .map_err(|e| ConfigError::platform(format!("Failed to write logging config: {}", e)))?;

        Ok(MigrationResult {
            success: true,
            source_path: source_path.to_path_buf(),
            target_path: target_path.to_path_buf(),
            sections_migrated: 1,
            warnings: vec![],
            errors: vec![],
            backup_path: None,
        })
    }

    /// Migrate unified configuration file
    async fn migrate_unified_config_file(
        &self,
        source_path: &Path,
        target_path: &Path,
    ) -> ConfigResult<MigrationResult> {
        let content = fs::read_to_string(source_path)
            .await
            .map_err(|e| ConfigError::platform(format!("Failed to read unified config: {}", e)))?;

        // Parse existing unified config
        let unified_config: crate::config::unified_config::UnifiedConfig =
            serde_json::from_str(&content)
                .map_err(|e| ConfigError::validation(format!("Invalid unified config: {}", e)))?;

        // Convert to enhanced configuration format
        let enhanced_config = self.convert_unified_to_enhanced(unified_config.clone())?;

        // Ensure target directory exists
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                ConfigError::platform(format!("Failed to create target directory: {}", e))
            })?;
        }

        // Write enhanced configuration
        let enhanced_content = toml::to_string_pretty(&enhanced_config).map_err(|e| {
            ConfigError::platform(format!("Failed to serialize enhanced config: {}", e))
        })?;

        fs::write(target_path, enhanced_content)
            .await
            .map_err(|e| {
                ConfigError::platform(format!("Failed to write enhanced config: {}", e))
            })?;

        Ok(MigrationResult {
            success: true,
            source_path: source_path.to_path_buf(),
            target_path: target_path.to_path_buf(),
            sections_migrated: unified_config.environments.len(),
            warnings: vec![],
            errors: vec![],
            backup_path: None,
        })
    }

    /// Convert unified config to enhanced config
    fn convert_unified_to_enhanced(
        &self,
        unified: crate::config::unified_config::UnifiedConfig,
    ) -> ConfigResult<EnhancedConfig> {
        let mut enhanced = EnhancedConfig::default();

        // Set version
        enhanced.base.version = unified.config_format_version;

        // Convert environment configurations to sections
        for (env_name, env_config) in unified.environments {
            let mut env_section = HashMap::new();

            // Add signing configuration
            env_section.insert(
                "signing".to_string(),
                ConfigValue::object({
                    let mut signing = HashMap::new();
                    signing.insert(
                        "policy".to_string(),
                        ConfigValue::string(env_config.signing.policy),
                    );
                    signing.insert(
                        "timeout_ms".to_string(),
                        ConfigValue::integer(env_config.signing.timeout_ms as i64),
                    );
                    signing.insert(
                        "include_content_digest".to_string(),
                        ConfigValue::boolean(env_config.signing.include_content_digest),
                    );
                    signing
                }),
            );

            // Add verification configuration
            env_section.insert(
                "verification".to_string(),
                ConfigValue::object({
                    let mut verification = HashMap::new();
                    verification.insert(
                        "strict_timing".to_string(),
                        ConfigValue::boolean(env_config.verification.strict_timing),
                    );
                    verification.insert(
                        "allow_clock_skew_seconds".to_string(),
                        ConfigValue::integer(
                            env_config.verification.allow_clock_skew_seconds as i64,
                        ),
                    );
                    verification
                }),
            );

            enhanced
                .base
                .set_section(env_name, ConfigValue::object(env_section));
        }

        Ok(enhanced)
    }

    /// Perform comprehensive migration of all configuration systems
    pub async fn migrate_all(&self) -> ConfigResult<Vec<MigrationResult>> {
        let mut results = Vec::new();

        // Migrate CLI configuration
        match self.migrate_cli_config().await {
            Ok(result) => results.push(result),
            Err(e) => {
                results.push(MigrationResult {
                    success: false,
                    source_path: PathBuf::new(),
                    target_path: PathBuf::new(),
                    sections_migrated: 0,
                    warnings: vec![],
                    errors: vec![format!("CLI migration failed: {}", e)],
                    backup_path: None,
                });
            }
        }

        // Migrate logging configuration
        match self.migrate_logging_config().await {
            Ok(result) => results.push(result),
            Err(e) => {
                results.push(MigrationResult {
                    success: false,
                    source_path: PathBuf::new(),
                    target_path: PathBuf::new(),
                    sections_migrated: 0,
                    warnings: vec![],
                    errors: vec![format!("Logging migration failed: {}", e)],
                    backup_path: None,
                });
            }
        }

        // Migrate unified configuration
        match self.migrate_unified_config().await {
            Ok(result) => results.push(result),
            Err(e) => {
                results.push(MigrationResult {
                    success: false,
                    source_path: PathBuf::new(),
                    target_path: PathBuf::new(),
                    sections_migrated: 0,
                    warnings: vec![],
                    errors: vec![format!("Unified migration failed: {}", e)],
                    backup_path: None,
                });
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_migration_manager_creation() {
        let manager = ConfigMigrationManager::new();
        // Should not panic
    }

    #[tokio::test]
    async fn test_json_to_config_conversion() {
        let manager = ConfigMigrationManager::new();

        let json = serde_json::json!({
            "app": {
                "name": "test",
                "version": "1.0.0"
            },
            "debug": true
        });

        let config = manager.json_to_config(json).unwrap();
        assert!(config.get_section("app").is_ok());
        assert!(config.get_section("debug").is_ok());
    }

    #[tokio::test]
    async fn test_migration_result_creation() {
        let result = MigrationResult {
            success: true,
            source_path: PathBuf::from("source.json"),
            target_path: PathBuf::from("target.toml"),
            sections_migrated: 2,
            warnings: vec![],
            errors: vec![],
            backup_path: None,
        };

        assert!(result.success);
        assert_eq!(result.sections_migrated, 2);
    }
}
