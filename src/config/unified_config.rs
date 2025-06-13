//! Unified configuration management across all DataFold components
//!
//! This module provides a unified configuration format that works across
//! Rust CLI, JavaScript SDK, and Python SDK. It supports environment-specific
//! configurations as the primary configuration system.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Errors that can occur during unified configuration operations
#[derive(Debug, thiserror::Error)]
pub enum UnifiedConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Configuration validation error: {0}")]
    Validation(String),
    
    #[error("Environment not found: {0}")]
    EnvironmentNotFound(String),
    
    #[error("Invalid configuration path: {0}")]
    InvalidPath(String),
}

pub type UnifiedConfigResult<T> = Result<T, UnifiedConfigError>;

/// Unified configuration structure supporting all platforms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedConfig {
    /// Configuration format version
    pub config_format_version: String,
    /// Environment-specific configurations
    pub environments: HashMap<String, EnvironmentConfig>,
    /// Security profiles available across platforms
    pub security_profiles: HashMap<String, SecurityProfile>,
    /// Default configuration values
    pub defaults: DefaultConfig,
}

/// Environment-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Signing configuration for this environment
    pub signing: SigningConfig,
    /// Verification configuration for this environment
    pub verification: VerificationConfig,
    /// Logging configuration for this environment
    pub logging: LoggingConfig,
    /// Authentication configuration for this environment
    pub authentication: AuthenticationConfig,
    /// Performance configuration for this environment
    pub performance: PerformanceConfig,
}

/// Signing configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningConfig {
    /// Security policy to use (minimal, standard, strict)
    pub policy: String,
    /// Signing timeout in milliseconds
    pub timeout_ms: u64,
    /// Required signature components
    pub required_components: Vec<String>,
    /// Whether to include content digest
    pub include_content_digest: bool,
    /// Whether to include timestamp
    pub include_timestamp: bool,
    /// Whether to include nonce
    pub include_nonce: bool,
    /// Maximum body size for digest calculation (MB)
    pub max_body_size_mb: u64,
    /// Debug configuration
    pub debug: DebugConfig,
}

/// Verification configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    /// Whether to enforce strict timing checks
    pub strict_timing: bool,
    /// Allowed clock skew in seconds
    pub allow_clock_skew_seconds: u64,
    /// Whether to require nonce in signatures
    pub require_nonce: bool,
    /// Maximum signature age in seconds
    pub max_signature_age_seconds: u64,
}

/// Logging configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (debug, info, warn, error)
    pub level: String,
    /// Whether to use colored output
    pub colored_output: bool,
    /// Whether to use structured logging
    pub structured: bool,
}

/// Authentication configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    /// Whether to store authentication tokens
    pub store_tokens: bool,
    /// Whether to automatically check for updates
    pub auto_update_check: bool,
    /// Whether to prompt on first signature
    pub prompt_on_first_sign: bool,
}

/// Performance configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Whether to cache keys
    pub cache_keys: bool,
    /// Maximum concurrent signing operations
    pub max_concurrent_signs: usize,
    /// Default timeout in seconds
    pub default_timeout_secs: u64,
    /// Default maximum retries
    pub default_max_retries: u32,
}

/// Debug configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    /// Whether debug is enabled
    pub enabled: bool,
    /// Whether to log canonical strings
    pub log_canonical_strings: bool,
    /// Whether to log components
    pub log_components: bool,
    /// Whether to log timing
    pub log_timing: bool,
}

/// Security profile definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityProfile {
    /// Profile description
    pub description: String,
    /// Required signature components
    pub required_components: Vec<String>,
    /// Whether to include content digest
    pub include_content_digest: bool,
    /// Digest algorithm to use
    pub digest_algorithm: String,
    /// Whether to validate nonces
    pub validate_nonces: bool,
    /// Whether to allow custom nonces
    pub allow_custom_nonces: bool,
}

/// Default configuration values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultConfig {
    /// Default environment to use
    pub environment: String,
    /// Default signing mode
    pub signing_mode: String,
    /// Default output format
    pub output_format: String,
    /// Default verbosity level
    pub verbosity: u8,
}

/// Unified configuration manager
#[derive(Clone)]
pub struct UnifiedConfigManager {
    config: UnifiedConfig,
    current_environment: String,
}

impl UnifiedConfigManager {
    /// Load unified configuration from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> UnifiedConfigResult<Self> {
        let content = fs::read_to_string(path)?;
        let config: UnifiedConfig = serde_json::from_str(&content)?;
        
        let current_environment = config.defaults.environment.clone();
        
        let manager = Self {
            config,
            current_environment,
        };
        
        manager.validate()?;
        Ok(manager)
    }
    
    /// Load from default unified config location
    pub fn load_default() -> UnifiedConfigResult<Self> {
        let config_path = Path::new("config/unified-datafold-config.json");
        Self::load_from_file(config_path)
    }
    
    /// Set current environment
    pub fn set_environment(&mut self, env: String) -> UnifiedConfigResult<()> {
        if !self.config.environments.contains_key(&env) {
            return Err(UnifiedConfigError::EnvironmentNotFound(env));
        }
        self.current_environment = env;
        Ok(())
    }
    
    /// Get current environment configuration
    pub fn current_environment_config(&self) -> UnifiedConfigResult<&EnvironmentConfig> {
        self.config.environments.get(&self.current_environment)
            .ok_or_else(|| UnifiedConfigError::EnvironmentNotFound(self.current_environment.clone()))
    }
    
    
    /// Get security profile by name
    pub fn get_security_profile(&self, name: &str) -> Option<&SecurityProfile> {
        self.config.security_profiles.get(name)
    }
    
    /// List available environments
    pub fn list_environments(&self) -> Vec<&String> {
        self.config.environments.keys().collect()
    }
    
    /// List available security profiles
    pub fn list_security_profiles(&self) -> Vec<&String> {
        self.config.security_profiles.keys().collect()
    }
    
    /// Validate the configuration
    fn validate(&self) -> UnifiedConfigResult<()> {
        // Validate default environment exists
        if !self.config.environments.contains_key(&self.config.defaults.environment) {
            return Err(UnifiedConfigError::Validation(
                format!("Default environment '{}' not found", self.config.defaults.environment)
            ));
        }
        
        // Validate each environment configuration
        for (env_name, env_config) in &self.config.environments {
            // Validate signing policy references exist
            if !self.config.security_profiles.contains_key(&env_config.signing.policy) {
                return Err(UnifiedConfigError::Validation(
                    format!("Environment '{}' references unknown security profile '{}'", 
                           env_name, env_config.signing.policy)
                ));
            }
            
            // Validate performance settings
            if env_config.performance.max_concurrent_signs == 0 {
                return Err(UnifiedConfigError::Validation(
                    format!("Environment '{}' has invalid max_concurrent_signs", env_name)
                ));
            }
            
            if env_config.signing.timeout_ms == 0 {
                return Err(UnifiedConfigError::Validation(
                    format!("Environment '{}' has invalid signing timeout", env_name)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Get current environment name
    pub fn current_environment(&self) -> &str {
        &self.current_environment
    }
    
    /// Get the full unified configuration
    pub fn config(&self) -> &UnifiedConfig {
        &self.config
    }
    
}


#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_config() -> UnifiedConfig {
        let mut environments = HashMap::new();
        environments.insert("test".to_string(), EnvironmentConfig {
            signing: SigningConfig {
                policy: "standard".to_string(),
                timeout_ms: 5000,
                required_components: vec!["@method".to_string(), "@target-uri".to_string()],
                include_content_digest: true,
                include_timestamp: true,
                include_nonce: true,
                max_body_size_mb: 10,
                debug: DebugConfig {
                    enabled: false,
                    log_canonical_strings: false,
                    log_components: false,
                    log_timing: false,
                },
            },
            verification: VerificationConfig {
                strict_timing: false,
                allow_clock_skew_seconds: 300,
                require_nonce: true,
                max_signature_age_seconds: 3600,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                colored_output: true,
                structured: false,
            },
            authentication: AuthenticationConfig {
                store_tokens: true,
                auto_update_check: true,
                prompt_on_first_sign: true,
            },
            performance: PerformanceConfig {
                cache_keys: true,
                max_concurrent_signs: 10,
                default_timeout_secs: 30,
                default_max_retries: 3,
            },
        });
        
        let mut security_profiles = HashMap::new();
        security_profiles.insert("standard".to_string(), SecurityProfile {
            description: "Standard security profile".to_string(),
            required_components: vec!["@method".to_string(), "@target-uri".to_string()],
            include_content_digest: true,
            digest_algorithm: "sha-256".to_string(),
            validate_nonces: true,
            allow_custom_nonces: true,
        });
        
        UnifiedConfig {
            config_format_version: "1.0".to_string(),
            environments,
            security_profiles,
            defaults: DefaultConfig {
                environment: "test".to_string(),
                signing_mode: "manual".to_string(),
                output_format: "table".to_string(),
                verbosity: 1,
            },
        }
    }
    
    #[test]
    fn test_unified_config_validation() {
        let config = create_test_config();
        let manager = UnifiedConfigManager {
            config,
            current_environment: "test".to_string(),
        };
        
        assert!(manager.validate().is_ok());
    }
    
    #[test]
    fn test_environment_switching() {
        let config = create_test_config();
        let manager = UnifiedConfigManager {
            config,
            current_environment: "test".to_string(),
        };
        
        assert_eq!(manager.current_environment(), "test");
        
        // Test environment configuration access
        let env_config = manager.current_environment_config().unwrap();
        assert_eq!(env_config.signing.policy, "standard");
        assert_eq!(env_config.signing.timeout_ms, 5000);
        assert_eq!(env_config.logging.level, "info");
    }
    
    #[test]
    fn test_security_profile_access() {
        let config = create_test_config();
        let manager = UnifiedConfigManager {
            config,
            current_environment: "test".to_string(),
        };
        
        let profile = manager.get_security_profile("standard").unwrap();
        assert_eq!(profile.description, "Standard security profile");
        assert!(profile.include_content_digest);
        assert_eq!(profile.digest_algorithm, "sha-256");
    }
}