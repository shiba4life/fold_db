//! Configuration for the ingestion module

use crate::config::traits::{
    AIServiceConfig, ApiClientConfigTrait, BaseConfig, ConfigLifecycle, ConfigValidation,
    EnvironmentConfigTrait, IngestionConfig as IngestionConfigTrait, RetryConfigTrait,
    StandardRetryConfig, TraitConfigError, TraitConfigResult, ValidationContext,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;
use std::time::Duration;

/// Configuration for the ingestion module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionConfig {
    /// OpenRouter API key
    pub openrouter_api_key: String,
    /// OpenRouter model to use
    pub openrouter_model: String,
    /// OpenRouter API base URL
    pub openrouter_base_url: String,
    /// Whether ingestion is enabled
    pub enabled: bool,
    /// Maximum number of retries for API calls
    pub max_retries: u32,
    /// Timeout for API calls in seconds
    pub timeout_seconds: u64,
    /// Whether to auto-execute mutations after generation
    pub auto_execute_mutations: bool,
    /// Default trust distance for mutations
    pub default_trust_distance: u32,
}

impl Default for IngestionConfig {
    fn default() -> Self {
        Self {
            openrouter_api_key: String::new(),
            openrouter_model: "anthropic/claude-3.5-sonnet".to_string(),
            openrouter_base_url: "https://openrouter.ai/api/v1".to_string(),
            enabled: false,
            max_retries: 3,
            timeout_seconds: 30,
            auto_execute_mutations: true,
            default_trust_distance: 0,
        }
    }
}

impl ApiClientConfigTrait for IngestionConfig {
    fn api_key_masked(&self) -> String {
        if self.openrouter_api_key.is_empty() {
            "<not configured>".to_string()
        } else {
            "***configured***".to_string()
        }
    }

    fn has_api_key(&self) -> bool {
        !self.openrouter_api_key.is_empty()
    }

    fn base_url(&self) -> &str {
        &self.openrouter_base_url
    }

    fn model(&self) -> &str {
        &self.openrouter_model
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }

    fn validate(&self) -> TraitConfigResult<()> {
        if self.enabled && self.openrouter_api_key.is_empty() {
            return Err(TraitConfigError::ValidationError {
                field: "openrouter_api_key".to_string(),
                message: "API key is required when ingestion is enabled".to_string(),
                context: ValidationContext::default(),
            });
        }

        if self.openrouter_model.is_empty() {
            return Err(TraitConfigError::ValidationError {
                field: "openrouter_model".to_string(),
                message: "Model identifier is required".to_string(),
                context: ValidationContext::default(),
            });
        }

        if self.openrouter_base_url.is_empty() {
            return Err(TraitConfigError::ValidationError {
                field: "openrouter_base_url".to_string(),
                message: "Base URL is required".to_string(),
                context: ValidationContext::default(),
            });
        }

        if self.timeout_seconds == 0 {
            return Err(TraitConfigError::ValidationError {
                field: "timeout_seconds".to_string(),
                message: "Timeout must be greater than 0".to_string(),
                context: ValidationContext::default(),
            });
        }

        if self.timeout_seconds > 300 {
            return Err(TraitConfigError::ValidationError {
                field: "timeout_seconds".to_string(),
                message: "Timeout should not exceed 300 seconds".to_string(),
                context: ValidationContext::default(),
            });
        }

        Ok(())
    }
}

impl RetryConfigTrait for IngestionConfig {
    fn max_retries(&self) -> u32 {
        self.max_retries
    }

    fn base_delay(&self) -> Duration {
        Duration::from_secs(1) // Default base delay
    }

    fn use_exponential_backoff(&self) -> bool {
        true // Default to exponential backoff
    }

    fn max_delay(&self) -> Duration {
        Duration::from_secs(30) // Default max delay
    }
}

impl EnvironmentConfigTrait for IngestionConfig {
    fn env_prefix() -> &'static str {
        "INGESTION"
    }

    fn apply_env_vars(&mut self) -> TraitConfigResult<()> {
        if let Ok(enabled) = env::var("INGESTION_ENABLED") {
            self.enabled = enabled.parse().unwrap_or(false);
        }

        if let Ok(max_retries) = env::var("INGESTION_MAX_RETRIES") {
            self.max_retries = max_retries.parse().unwrap_or(3);
        }

        if let Ok(timeout) = env::var("INGESTION_TIMEOUT_SECONDS") {
            self.timeout_seconds = timeout.parse().unwrap_or(30);
        }

        if let Ok(auto_execute) = env::var("INGESTION_AUTO_EXECUTE") {
            self.auto_execute_mutations = auto_execute.parse().unwrap_or(true);
        }

        if let Ok(trust_distance) = env::var("INGESTION_DEFAULT_TRUST_DISTANCE") {
            self.default_trust_distance = trust_distance.parse().unwrap_or(0);
        }

        Ok(())
    }
}

impl IngestionConfig {
    /// Create a new ingestion config from environment variables and saved config file
    pub fn from_env() -> Result<Self, crate::ingestion::IngestionError> {
        // Try to get API key from environment first, then from saved config
        let mut api_key = env::var("FOLD_OPENROUTER_API_KEY").unwrap_or_default();
        let mut model = env::var("OPENROUTER_MODEL")
            .unwrap_or_else(|_| "anthropic/claude-3.5-sonnet".to_string());

        // If no API key in environment, try to load from saved config
        if api_key.is_empty() {
            if let Ok(saved_config) = Self::load_saved_config() {
                if !saved_config.api_key.is_empty() {
                    api_key = saved_config.api_key;
                }
                model = saved_config.model;
            }
        }

        // If still no API key, return error
        if api_key.is_empty() {
            return Err(crate::ingestion::IngestionError::configuration_error(
                "FOLD_OPENROUTER_API_KEY not set in environment or saved config",
            ));
        }

        let base_url = env::var("OPENROUTER_BASE_URL")
            .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

        let enabled = env::var("INGESTION_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let max_retries = env::var("INGESTION_MAX_RETRIES")
            .unwrap_or_else(|_| "3".to_string())
            .parse()
            .unwrap_or(3);

        let timeout_seconds = env::var("INGESTION_TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);

        let auto_execute_mutations = env::var("INGESTION_AUTO_EXECUTE")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let default_trust_distance = env::var("INGESTION_DEFAULT_TRUST_DISTANCE")
            .unwrap_or_else(|_| "0".to_string())
            .parse()
            .unwrap_or(0);

        Ok(Self {
            openrouter_api_key: api_key,
            openrouter_model: model,
            openrouter_base_url: base_url,
            enabled,
            max_retries,
            timeout_seconds,
            auto_execute_mutations,
            default_trust_distance,
        })
    }

    /// Create a new ingestion config allowing empty API key (for configuration endpoints)
    pub fn from_env_allow_empty() -> Self {
        // Try to get API key from environment first, then from saved config
        let mut api_key = env::var("FOLD_OPENROUTER_API_KEY").unwrap_or_default();
        let mut model = env::var("OPENROUTER_MODEL")
            .unwrap_or_else(|_| "anthropic/claude-3.5-sonnet".to_string());

        // If no API key in environment, try to load from saved config
        if api_key.is_empty() {
            if let Ok(saved_config) = Self::load_saved_config() {
                if !saved_config.api_key.is_empty() {
                    api_key = saved_config.api_key;
                }
                model = saved_config.model;
            }
        }

        let base_url = env::var("OPENROUTER_BASE_URL")
            .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

        let enabled = env::var("INGESTION_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let max_retries = env::var("INGESTION_MAX_RETRIES")
            .unwrap_or_else(|_| "3".to_string())
            .parse()
            .unwrap_or(3);

        let timeout_seconds = env::var("INGESTION_TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);

        let auto_execute_mutations = env::var("INGESTION_AUTO_EXECUTE")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let default_trust_distance = env::var("INGESTION_DEFAULT_TRUST_DISTANCE")
            .unwrap_or_else(|_| "0".to_string())
            .parse()
            .unwrap_or(0);

        Self {
            openrouter_api_key: api_key,
            openrouter_model: model,
            openrouter_base_url: base_url,
            enabled,
            max_retries,
            timeout_seconds,
            auto_execute_mutations,
            default_trust_distance,
        }
    }

    /// Load saved OpenRouter configuration from file
    fn load_saved_config() -> Result<SavedOpenRouterConfig, Box<dyn std::error::Error>> {
        use std::fs;
        use std::path::Path;

        let config_dir = env::var("DATAFOLD_CONFIG_DIR").unwrap_or_else(|_| "./config".to_string());

        let config_path = Path::new(&config_dir).join("openrouter_config.json");

        if !config_path.exists() {
            return Err("Config file does not exist".into());
        }

        let content = fs::read_to_string(&config_path)?;
        let config: SavedOpenRouterConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), crate::ingestion::IngestionError> {
        if self.openrouter_api_key.is_empty() {
            return Err(crate::ingestion::IngestionError::configuration_error(
                "OpenRouter API key is required",
            ));
        }

        if self.openrouter_model.is_empty() {
            return Err(crate::ingestion::IngestionError::configuration_error(
                "OpenRouter model is required",
            ));
        }

        if self.openrouter_base_url.is_empty() {
            return Err(crate::ingestion::IngestionError::configuration_error(
                "OpenRouter base URL is required",
            ));
        }

        Ok(())
    }

    /// Check if ingestion is enabled and properly configured
    pub fn is_ready(&self) -> bool {
        self.enabled && self.validate().is_ok()
    }
}

/// Saved OpenRouter configuration structure
#[derive(Debug, Serialize, Deserialize)]
struct SavedOpenRouterConfig {
    pub api_key: String,
    pub model: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = IngestionConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.openrouter_model, "anthropic/claude-3.5-sonnet");
        assert_eq!(config.openrouter_base_url, "https://openrouter.ai/api/v1");
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.auto_execute_mutations);
        assert_eq!(config.default_trust_distance, 0);
    }

    #[test]
    fn test_validation_fails_without_api_key() {
        let config = IngestionConfig::default();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_succeeds_with_api_key() {
        let config = IngestionConfig {
            openrouter_api_key: "test-key".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_is_ready() {
        let mut config = IngestionConfig::default();
        assert!(!config.is_ready());

        config.enabled = true;
        config.openrouter_api_key = "test-key".to_string();
        assert!(config.is_ready());
    }
}

// Implement shared traits for IngestionConfig
#[async_trait]
impl BaseConfig for IngestionConfig {
    type Error = crate::ingestion::IngestionError;
    type Event = ();
    type TransformTarget = ();

    async fn load(path: &Path) -> Result<Self, Self::Error> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::ingestion::IngestionError::configuration_error(format!(
                "Failed to read config file: {}",
                e
            ))
        })?;

        let config: Self = serde_json::from_str(&content).map_err(|e| {
            crate::ingestion::IngestionError::configuration_error(format!(
                "Failed to parse config file: {}",
                e
            ))
        })?;

        Ok(config)
    }

    fn validate(&self) -> Result<(), Self::Error> {
        self.validate()
    }

    async fn save(&self, path: &Path) -> Result<(), Self::Error> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                crate::ingestion::IngestionError::configuration_error(format!(
                    "Failed to create config directory: {}",
                    e
                ))
            })?;
        }

        let content = serde_json::to_string_pretty(self).map_err(|e| {
            crate::ingestion::IngestionError::configuration_error(format!(
                "Failed to serialize config: {}",
                e
            ))
        })?;

        std::fs::write(path, content).map_err(|e| {
            crate::ingestion::IngestionError::configuration_error(format!(
                "Failed to write config file: {}",
                e
            ))
        })?;

        Ok(())
    }

    fn metadata(&self) -> std::collections::HashMap<String, String> {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("enabled".to_string(), self.enabled.to_string());
        metadata.insert("model".to_string(), self.openrouter_model.clone());
        metadata.insert("has_api_key".to_string(), self.has_api_key().to_string());
        metadata.insert("max_retries".to_string(), self.max_retries.to_string());
        metadata.insert(
            "timeout_seconds".to_string(),
            self.timeout_seconds.to_string(),
        );
        metadata
    }
}

#[async_trait]
impl ConfigLifecycle for IngestionConfig {
    async fn reload(&mut self, path: &Path) -> Result<(), Self::Error> {
        let new_config = Self::load(path).await?;
        *self = new_config;
        Ok(())
    }

    async fn backup(&self, backup_path: &Path) -> Result<(), Self::Error> {
        self.save(backup_path).await
    }

    async fn merge(&mut self, other: Self) -> Result<(), Self::Error> {
        // Merge logic: prefer non-default values from other
        if other.enabled {
            self.enabled = other.enabled;
        }

        if !other.openrouter_api_key.is_empty() {
            self.openrouter_api_key = other.openrouter_api_key;
        }

        if other.openrouter_model != "anthropic/claude-3.5-sonnet" {
            self.openrouter_model = other.openrouter_model;
        }

        if other.openrouter_base_url != "https://openrouter.ai/api/v1" {
            self.openrouter_base_url = other.openrouter_base_url;
        }

        if other.max_retries != 3 {
            self.max_retries = other.max_retries;
        }

        if other.timeout_seconds != 30 {
            self.timeout_seconds = other.timeout_seconds;
        }

        if !other.auto_execute_mutations {
            self.auto_execute_mutations = other.auto_execute_mutations;
        }

        if other.default_trust_distance != 0 {
            self.default_trust_distance = other.default_trust_distance;
        }

        Ok(())
    }
}

#[async_trait]
impl ConfigValidation for IngestionConfig {
    fn validate_field(&self, field_name: &str) -> Result<(), Self::Error> {
        match field_name {
            "openrouter_api_key" => {
                if self.enabled && self.openrouter_api_key.is_empty() {
                    return Err(crate::ingestion::IngestionError::configuration_error(
                        "OpenRouter API key is required when ingestion is enabled",
                    ));
                }
            }
            "openrouter_model" => {
                if self.openrouter_model.is_empty() {
                    return Err(crate::ingestion::IngestionError::configuration_error(
                        "OpenRouter model is required",
                    ));
                }
            }
            "openrouter_base_url" => {
                if self.openrouter_base_url.is_empty() {
                    return Err(crate::ingestion::IngestionError::configuration_error(
                        "OpenRouter base URL is required",
                    ));
                }
            }
            "timeout_seconds" => {
                if self.timeout_seconds == 0 {
                    return Err(crate::ingestion::IngestionError::configuration_error(
                        "Timeout must be greater than 0",
                    ));
                }
                if self.timeout_seconds > 300 {
                    return Err(crate::ingestion::IngestionError::configuration_error(
                        "Timeout should not exceed 300 seconds",
                    ));
                }
            }
            "max_retries" => {
                if self.max_retries > 20 {
                    return Err(crate::ingestion::IngestionError::configuration_error(
                        "Max retries should not exceed 20",
                    ));
                }
            }
            _ => {
                return Err(crate::ingestion::IngestionError::configuration_error(
                    format!("Unknown field: {}", field_name),
                ))
            }
        }
        Ok(())
    }

    fn get_validation_rules(&self) -> std::collections::HashMap<String, String> {
        let mut rules = std::collections::HashMap::new();
        rules.insert(
            "openrouter_api_key".to_string(),
            "Required when ingestion is enabled".to_string(),
        );
        rules.insert(
            "openrouter_model".to_string(),
            "Must be a valid model identifier".to_string(),
        );
        rules.insert(
            "openrouter_base_url".to_string(),
            "Must be a valid URL".to_string(),
        );
        rules.insert(
            "timeout_seconds".to_string(),
            "Must be between 1 and 300 seconds".to_string(),
        );
        rules.insert(
            "max_retries".to_string(),
            "Must be between 0 and 20".to_string(),
        );
        rules.insert(
            "default_trust_distance".to_string(),
            "Must be a non-negative integer".to_string(),
        );
        rules
    }
}

#[async_trait]
impl IngestionConfigTrait for IngestionConfig {
    type ApiClientConfig = Self;
    type ServiceSettings = IngestionServiceSettings;
    type RetryConfig = Self;

    fn api_client_config(&self) -> &Self::ApiClientConfig {
        self
    }

    fn service_settings(&self) -> &Self::ServiceSettings {
        // Return a static reference to avoid lifetime issues
        use once_cell::sync::Lazy;
        static DEFAULT_SERVICE_SETTINGS: Lazy<IngestionServiceSettings> =
            Lazy::new(|| IngestionServiceSettings::default());
        &*DEFAULT_SERVICE_SETTINGS
    }

    fn retry_config(&self) -> &Self::RetryConfig {
        self
    }

    fn is_ready(&self) -> bool {
        self.is_ready()
    }

    async fn validate_api_connectivity(&self) -> TraitConfigResult<()> {
        if !self.enabled {
            return Ok(());
        }

        // Basic validation - in a real implementation, this could make a test API call
        if self.openrouter_api_key.is_empty() {
            return Err(TraitConfigError::ValidationError {
                field: "openrouter_api_key".to_string(),
                message: "API key is required for connectivity validation".to_string(),
                context: ValidationContext::default(),
            });
        }

        Ok(())
    }

    async fn apply_env_overrides(&mut self) -> TraitConfigResult<()> {
        self.apply_env_vars()
    }

    fn validate_timeouts(&self) -> TraitConfigResult<()> {
        if self.timeout_seconds == 0 {
            return Err(TraitConfigError::ValidationError {
                field: "timeout_seconds".to_string(),
                message: "Timeout must be greater than 0".to_string(),
                context: ValidationContext::default(),
            });
        }

        if self.timeout_seconds > 300 {
            return Err(TraitConfigError::ValidationError {
                field: "timeout_seconds".to_string(),
                message: "Timeout should not exceed 300 seconds".to_string(),
                context: ValidationContext::default(),
            });
        }

        Ok(())
    }

    fn validate_retry_settings(&self) -> TraitConfigResult<()> {
        self.validate()
            .map_err(|e| TraitConfigError::ValidationError {
                field: "retry_settings".to_string(),
                message: format!("Retry validation failed: {}", e),
                context: ValidationContext::default(),
            })
    }
}

/// Service-specific settings for ingestion
#[derive(Debug, Clone, Default)]
pub struct IngestionServiceSettings {
    /// Whether to auto-execute mutations
    pub auto_execute_mutations: bool,
    /// Default trust distance
    pub default_trust_distance: u32,
}
