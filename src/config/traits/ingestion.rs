//! Ingestion-specific configuration traits for the shared traits system
//!
//! This module provides domain-specific traits for ingestion configurations,
//! implementing common patterns found across ingestion service configurations
//! while maintaining type safety and validation consistency.

use crate::config::traits::{
    BaseConfig, ConfigLifecycle, ConfigValidation, TraitConfigError, TraitConfigResult,
    ValidationContext,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

/// Domain-specific trait for ingestion configurations
#[async_trait]
pub trait IngestionConfig: BaseConfig + ConfigValidation + ConfigLifecycle {
    /// Associated type for API client configuration
    type ApiClientConfig: Clone + std::fmt::Debug;

    /// Associated type for service-specific settings
    type ServiceSettings: Clone + std::fmt::Debug + Default;

    /// Associated type for retry configuration
    type RetryConfig: Clone + std::fmt::Debug;

    /// Get the API client configuration
    fn api_client_config(&self) -> &Self::ApiClientConfig;

    /// Get service-specific settings
    fn service_settings(&self) -> &Self::ServiceSettings;

    /// Get retry configuration
    fn retry_config(&self) -> &Self::RetryConfig;

    /// Check if ingestion is enabled and properly configured
    fn is_ready(&self) -> bool;

    /// Validate API credentials and connectivity
    async fn validate_api_connectivity(&self) -> TraitConfigResult<()>;

    /// Apply environment variable overrides to the configuration
    async fn apply_env_overrides(&mut self) -> TraitConfigResult<()>;

    /// Validate timeout configurations
    fn validate_timeouts(&self) -> TraitConfigResult<()>;

    /// Validate retry settings
    fn validate_retry_settings(&self) -> TraitConfigResult<()>;
}

/// Trait for API client configuration
pub trait ApiClientConfigTrait: Clone + std::fmt::Debug {
    /// Get the API key (masked for logging)
    fn api_key_masked(&self) -> String;

    /// Check if API key is configured
    fn has_api_key(&self) -> bool;

    /// Get the base URL for the API
    fn base_url(&self) -> &str;

    /// Get the model or service identifier
    fn model(&self) -> &str;

    /// Get request timeout
    fn timeout(&self) -> Duration;

    /// Validate API client configuration
    fn validate(&self) -> TraitConfigResult<()>;
}

/// Trait for retry configuration
pub trait RetryConfigTrait: Clone + std::fmt::Debug {
    /// Maximum number of retries
    fn max_retries(&self) -> u32;

    /// Base delay between retries
    fn base_delay(&self) -> Duration;

    /// Whether to use exponential backoff
    fn use_exponential_backoff(&self) -> bool;

    /// Maximum delay between retries
    fn max_delay(&self) -> Duration;

    /// Validate retry configuration
    fn validate(&self) -> TraitConfigResult<()> {
        if self.max_retries() > 20 {
            return Err(TraitConfigError::ValidationError {
                field: "max_retries".to_string(),
                message: "Maximum retries should not exceed 20".to_string(),
                context: ValidationContext::default(),
            });
        }

        if self.base_delay().as_secs() > 60 {
            return Err(TraitConfigError::ValidationError {
                field: "base_delay".to_string(),
                message: "Base delay should not exceed 60 seconds".to_string(),
                context: ValidationContext::default(),
            });
        }

        if self.max_delay() < self.base_delay() {
            return Err(TraitConfigError::ValidationError {
                field: "max_delay".to_string(),
                message: "Maximum delay must be greater than or equal to base delay".to_string(),
                context: ValidationContext::default(),
            });
        }

        Ok(())
    }
}

/// Configuration for AI service integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIServiceConfig {
    /// API key for the AI service
    pub api_key: String,
    /// Model to use for AI operations
    pub model: String,
    /// Base URL for the AI service API
    pub base_url: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Whether the service is enabled
    pub enabled: bool,
}

impl Default for AIServiceConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "default-model".to_string(),
            base_url: "https://api.example.com".to_string(),
            timeout_seconds: 30,
            enabled: false,
        }
    }
}

impl ApiClientConfigTrait for AIServiceConfig {
    fn api_key_masked(&self) -> String {
        if self.api_key.is_empty() {
            "<not configured>".to_string()
        } else {
            "***configured***".to_string()
        }
    }

    fn has_api_key(&self) -> bool {
        !self.api_key.is_empty()
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }

    fn validate(&self) -> TraitConfigResult<()> {
        if self.enabled && self.api_key.is_empty() {
            return Err(TraitConfigError::ValidationError {
                field: "api_key".to_string(),
                message: "API key is required when service is enabled".to_string(),
                context: ValidationContext::default(),
            });
        }

        if self.model.is_empty() {
            return Err(TraitConfigError::ValidationError {
                field: "model".to_string(),
                message: "Model identifier is required".to_string(),
                context: ValidationContext::default(),
            });
        }

        if self.base_url.is_empty() {
            return Err(TraitConfigError::ValidationError {
                field: "base_url".to_string(),
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

/// Standard retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardRetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Base delay in seconds between retries
    pub base_delay_seconds: u64,
    /// Whether to use exponential backoff
    pub exponential_backoff: bool,
    /// Maximum delay in seconds between retries
    pub max_delay_seconds: u64,
}

impl Default for StandardRetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_seconds: 1,
            exponential_backoff: true,
            max_delay_seconds: 30,
        }
    }
}

impl RetryConfigTrait for StandardRetryConfig {
    fn max_retries(&self) -> u32 {
        self.max_retries
    }

    fn base_delay(&self) -> Duration {
        Duration::from_secs(self.base_delay_seconds)
    }

    fn use_exponential_backoff(&self) -> bool {
        self.exponential_backoff
    }

    fn max_delay(&self) -> Duration {
        Duration::from_secs(self.max_delay_seconds)
    }
}

/// Configuration for data processing settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProcessingConfig {
    /// Whether to auto-execute generated operations
    pub auto_execute: bool,
    /// Default trust distance for operations
    pub default_trust_distance: u32,
    /// Maximum data size to process (in bytes)
    pub max_data_size: usize,
    /// Whether to validate input data
    pub validate_input: bool,
    /// Whether to enable data transformation
    pub enable_transformation: bool,
}

impl Default for DataProcessingConfig {
    fn default() -> Self {
        Self {
            auto_execute: true,
            default_trust_distance: 0,
            max_data_size: 10 * 1024 * 1024, // 10MB
            validate_input: true,
            enable_transformation: true,
        }
    }
}

impl DataProcessingConfig {
    /// Validate data processing configuration
    pub fn validate(&self) -> TraitConfigResult<()> {
        if self.max_data_size == 0 {
            return Err(TraitConfigError::ValidationError {
                field: "max_data_size".to_string(),
                message: "Maximum data size must be greater than 0".to_string(),
                context: ValidationContext::default(),
            });
        }

        if self.max_data_size > 100 * 1024 * 1024 {
            return Err(TraitConfigError::ValidationError {
                field: "max_data_size".to_string(),
                message: "Maximum data size should not exceed 100MB".to_string(),
                context: ValidationContext::default(),
            });
        }

        Ok(())
    }
}

/// Configuration for ingestion security settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionSecurityConfig {
    /// Enable rate limiting
    pub enable_rate_limiting: bool,
    /// Maximum requests per minute
    pub max_requests_per_minute: u32,
    /// Enable input sanitization
    pub enable_input_sanitization: bool,
    /// Enable output validation
    pub enable_output_validation: bool,
    /// Allowed content types
    pub allowed_content_types: Vec<String>,
}

impl Default for IngestionSecurityConfig {
    fn default() -> Self {
        Self {
            enable_rate_limiting: true,
            max_requests_per_minute: 60,
            enable_input_sanitization: true,
            enable_output_validation: true,
            allowed_content_types: vec!["application/json".to_string(), "text/plain".to_string()],
        }
    }
}

impl IngestionSecurityConfig {
    /// Validate security configuration
    pub fn validate(&self) -> TraitConfigResult<()> {
        if self.enable_rate_limiting && self.max_requests_per_minute == 0 {
            return Err(TraitConfigError::ValidationError {
                field: "max_requests_per_minute".to_string(),
                message: "Maximum requests per minute must be greater than 0 when rate limiting is enabled".to_string(),
                context: ValidationContext::default(),
            });
        }

        if self.max_requests_per_minute > 10000 {
            return Err(TraitConfigError::ValidationError {
                field: "max_requests_per_minute".to_string(),
                message: "Maximum requests per minute should not exceed 10000".to_string(),
                context: ValidationContext::default(),
            });
        }

        if self.allowed_content_types.is_empty() {
            return Err(TraitConfigError::ValidationError {
                field: "allowed_content_types".to_string(),
                message: "At least one content type must be allowed".to_string(),
                context: ValidationContext::default(),
            });
        }

        Ok(())
    }
}

/// Trait for environment variable mapping
pub trait EnvironmentConfigTrait {
    /// Get environment variable prefix for this configuration
    fn env_prefix() -> &'static str;

    /// Apply environment variables with the given prefix
    fn apply_env_vars(&mut self) -> TraitConfigResult<()>;

    /// Get environment variable name for a field
    fn env_var_name(field: &str) -> String {
        format!("{}_{}", Self::env_prefix(), field.to_uppercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_service_config() {
        let mut config = AIServiceConfig::default();
        config.enabled = true;

        // Should fail validation without API key
        assert!(config.validate().is_err());

        config.api_key = "test-key".to_string();
        assert!(config.validate().is_ok());

        assert!(config.has_api_key());
        assert_eq!(config.api_key_masked(), "***configured***");
        assert_eq!(config.timeout(), Duration::from_secs(30));
    }

    #[test]
    fn test_retry_config() {
        let config = StandardRetryConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.max_retries(), 3);
        assert_eq!(config.base_delay(), Duration::from_secs(1));
        assert!(config.use_exponential_backoff());

        let mut invalid_config = config.clone();
        invalid_config.max_retries = 25;
        assert!(invalid_config.validate().is_err());

        let mut invalid_config = config.clone();
        invalid_config.max_delay_seconds = 0;
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_data_processing_config() {
        let config = DataProcessingConfig::default();
        assert!(config.validate().is_ok());
        assert!(config.auto_execute);
        assert_eq!(config.default_trust_distance, 0);

        let mut invalid_config = config.clone();
        invalid_config.max_data_size = 0;
        assert!(invalid_config.validate().is_err());

        let mut invalid_config = config.clone();
        invalid_config.max_data_size = 200 * 1024 * 1024; // 200MB
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_security_config() {
        let config = IngestionSecurityConfig::default();
        assert!(config.validate().is_ok());
        assert!(config.enable_rate_limiting);
        assert_eq!(config.max_requests_per_minute, 60);

        let mut invalid_config = config.clone();
        invalid_config.max_requests_per_minute = 0;
        assert!(invalid_config.validate().is_err());

        let mut invalid_config = config.clone();
        invalid_config.allowed_content_types.clear();
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_empty_api_key_handling() {
        let config = AIServiceConfig::default();
        assert!(!config.has_api_key());
        assert_eq!(config.api_key_masked(), "<not configured>");
    }
}
