//! Configuration for the ingestion module

use serde::{Deserialize, Serialize};
use std::env;

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
        let mut config = IngestionConfig::default();
        config.openrouter_api_key = "test-key".to_string();
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
