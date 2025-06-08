//! Cryptographic configuration for DataFold database encryption
//!
//! This module provides configuration structures for master key encryption,
//! key derivation parameters, and crypto initialization settings.

use crate::crypto::{argon2::Argon2Params, error::CryptoResult, CryptoError};
use serde::{Deserialize, Serialize};

/// Top-level cryptographic configuration for database encryption
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CryptoConfig {
    /// Whether database encryption is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Master key configuration for database encryption
    #[serde(default)]
    pub master_key: MasterKeyConfig,
    
    /// Key derivation configuration (when using passphrase-based keys)
    #[serde(default)]
    pub key_derivation: KeyDerivationConfig,
}

fn default_enabled() -> bool {
    false
}



impl CryptoConfig {
    /// Create a new crypto configuration with encryption disabled
    pub fn disabled() -> Self {
        Self::default()
    }
    
    /// Create a new crypto configuration with passphrase-based encryption
    pub fn with_passphrase(passphrase: String) -> Self {
        Self {
            enabled: true,
            master_key: MasterKeyConfig::Passphrase { passphrase },
            key_derivation: KeyDerivationConfig::default(),
        }
    }
    
    /// Create a new crypto configuration with random key generation
    pub fn with_random_key() -> Self {
        Self {
            enabled: true,
            master_key: MasterKeyConfig::Random,
            key_derivation: KeyDerivationConfig::default(),
        }
    }
    
    /// Create a new crypto configuration with enhanced security parameters
    pub fn with_enhanced_security(passphrase: String) -> Self {
        Self {
            enabled: true,
            master_key: MasterKeyConfig::Passphrase { passphrase },
            key_derivation: KeyDerivationConfig::sensitive(),
        }
    }
    
    /// Validate the crypto configuration
    pub fn validate(&self) -> CryptoResult<()> {
        if !self.enabled {
            return Ok(());
        }
        
        // Validate master key configuration
        self.master_key.validate()?;
        
        // Validate key derivation parameters
        self.key_derivation.validate()?;
        
        Ok(())
    }
    
    /// Check if the configuration requires a passphrase
    pub fn requires_passphrase(&self) -> bool {
        self.enabled && matches!(self.master_key, MasterKeyConfig::Passphrase { .. })
    }
    
    /// Check if the configuration uses random key generation
    pub fn uses_random_key(&self) -> bool {
        self.enabled && matches!(self.master_key, MasterKeyConfig::Random)
    }
}

/// Configuration for master key generation and management
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MasterKeyConfig {
    /// Generate a random master key pair (highest security, no password recovery)
    Random,
    
    /// Derive master key from user passphrase (allows password recovery)
    Passphrase {
        /// The passphrase to use for key derivation
        /// Note: In production, this should be provided at runtime, not stored
        passphrase: String,
    },
    
    /// Use an existing key pair from external source (advanced use case)
    External {
        /// Path to the external key file or key identifier
        key_source: String,
    },
}

impl Default for MasterKeyConfig {
    fn default() -> Self {
        Self::Random
    }
}

impl MasterKeyConfig {
    /// Validate the master key configuration
    pub fn validate(&self) -> CryptoResult<()> {
        match self {
            Self::Random => Ok(()),
            Self::Passphrase { passphrase } => {
                if passphrase.is_empty() {
                    return Err(CryptoError::InvalidKey {
                        message: "Passphrase cannot be empty".to_string(),
                    });
                }
                
                if passphrase.len() < 8 {
                    return Err(CryptoError::InvalidKey {
                        message: "Passphrase must be at least 8 characters".to_string(),
                    });
                }
                
                Ok(())
            },
            Self::External { key_source } => {
                if key_source.is_empty() {
                    return Err(CryptoError::InvalidKey {
                        message: "External key source cannot be empty".to_string(),
                    });
                }
                
                Ok(())
            },
        }
    }
}

/// Configuration for key derivation parameters (Argon2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationConfig {
    /// Memory cost in KB (minimum 8 KB, recommended 64 MB+)
    #[serde(default = "default_memory_cost")]
    pub memory_cost: u32,
    
    /// Time cost (iterations, minimum 1, recommended 3+)
    #[serde(default = "default_time_cost")]
    pub time_cost: u32,
    
    /// Parallelism degree (threads, minimum 1, recommended 4)
    #[serde(default = "default_parallelism")]
    pub parallelism: u32,
    
    /// Security level preset (overrides individual parameters if set)
    #[serde(default)]
    pub preset: Option<SecurityLevel>,
}

fn default_memory_cost() -> u32 {
    65536 // 64 MB
}

fn default_time_cost() -> u32 {
    3
}

fn default_parallelism() -> u32 {
    4
}

impl Default for KeyDerivationConfig {
    fn default() -> Self {
        Self {
            memory_cost: default_memory_cost(),
            time_cost: default_time_cost(),
            parallelism: default_parallelism(),
            preset: None,
        }
    }
}

impl KeyDerivationConfig {
    /// Create configuration optimized for interactive use (faster)
    pub fn interactive() -> Self {
        Self {
            memory_cost: 32768, // 32 MB
            time_cost: 2,
            parallelism: 2,
            preset: Some(SecurityLevel::Interactive),
        }
    }
    
    /// Create configuration optimized for sensitive operations (slower, more secure)
    pub fn sensitive() -> Self {
        Self {
            memory_cost: 131072, // 128 MB
            time_cost: 4,
            parallelism: 8,
            preset: Some(SecurityLevel::Sensitive),
        }
    }
    
    /// Create configuration with custom parameters
    pub fn custom(memory_cost: u32, time_cost: u32, parallelism: u32) -> CryptoResult<Self> {
        let config = Self {
            memory_cost,
            time_cost,
            parallelism,
            preset: None,
        };
        
        config.validate()?;
        Ok(config)
    }
    
    /// Validate the key derivation configuration
    pub fn validate(&self) -> CryptoResult<()> {
        // Use preset parameters if specified
        let (memory_cost, time_cost, parallelism) = if let Some(preset) = &self.preset {
            match preset {
                SecurityLevel::Interactive => (32768, 2, 2),
                SecurityLevel::Balanced => (65536, 3, 4),
                SecurityLevel::Sensitive => (131072, 4, 8),
            }
        } else {
            (self.memory_cost, self.time_cost, self.parallelism)
        };
        
        // Validate using Argon2Params validation
        Argon2Params::new(memory_cost, time_cost, parallelism)?;
        
        Ok(())
    }
    
    /// Convert to Argon2Params for use with crypto module
    pub fn to_argon2_params(&self) -> CryptoResult<Argon2Params> {
        let (memory_cost, time_cost, parallelism) = if let Some(preset) = &self.preset {
            match preset {
                SecurityLevel::Interactive => return Ok(Argon2Params::interactive()),
                SecurityLevel::Balanced => return Ok(Argon2Params::default()),
                SecurityLevel::Sensitive => return Ok(Argon2Params::sensitive()),
            }
        } else {
            (self.memory_cost, self.time_cost, self.parallelism)
        };
        
        Argon2Params::new(memory_cost, time_cost, parallelism)
    }
}

/// Security level presets for key derivation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SecurityLevel {
    /// Fast parameters for interactive use (32 MB, 2 iterations, 2 threads)
    Interactive,
    
    /// Balanced parameters for general use (64 MB, 3 iterations, 4 threads)
    Balanced,
    
    /// High security parameters for sensitive operations (128 MB, 4 iterations, 8 threads)
    Sensitive,
}

/// Configuration validation errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Crypto configuration validation failed: {0}")]
    CryptoValidation(#[from] CryptoError),
    
    #[error("Invalid configuration parameter: {message}")]
    InvalidParameter { message: String },
    
    #[error("Configuration serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for configuration operations
pub type ConfigResult<T> = Result<T, ConfigError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_config_default() {
        let config = CryptoConfig::default();
        assert!(!config.enabled);
        assert!(matches!(config.master_key, MasterKeyConfig::Random));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_crypto_config_with_passphrase() {
        let config = CryptoConfig::with_passphrase("test-passphrase-123".to_string());
        assert!(config.enabled);
        assert!(config.requires_passphrase());
        assert!(!config.uses_random_key());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_crypto_config_with_random_key() {
        let config = CryptoConfig::with_random_key();
        assert!(config.enabled);
        assert!(!config.requires_passphrase());
        assert!(config.uses_random_key());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_crypto_config_enhanced_security() {
        let config = CryptoConfig::with_enhanced_security("strong-passphrase".to_string());
        assert!(config.enabled);
        assert!(config.requires_passphrase());
        assert_eq!(config.key_derivation.preset, Some(SecurityLevel::Sensitive));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_master_key_config_validation() {
        // Valid passphrase
        let config = MasterKeyConfig::Passphrase {
            passphrase: "valid-passphrase".to_string(),
        };
        assert!(config.validate().is_ok());

        // Empty passphrase should fail
        let config = MasterKeyConfig::Passphrase {
            passphrase: "".to_string(),
        };
        assert!(config.validate().is_err());

        // Short passphrase should fail
        let config = MasterKeyConfig::Passphrase {
            passphrase: "short".to_string(),
        };
        assert!(config.validate().is_err());

        // Random key should always be valid
        let config = MasterKeyConfig::Random;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_key_derivation_config_presets() {
        let interactive = KeyDerivationConfig::interactive();
        assert_eq!(interactive.preset, Some(SecurityLevel::Interactive));
        assert!(interactive.validate().is_ok());

        let sensitive = KeyDerivationConfig::sensitive();
        assert_eq!(sensitive.preset, Some(SecurityLevel::Sensitive));
        assert!(sensitive.validate().is_ok());

        let default = KeyDerivationConfig::default();
        assert_eq!(default.preset, None);
        assert!(default.validate().is_ok());
    }

    #[test]
    fn test_key_derivation_config_to_argon2_params() {
        let config = KeyDerivationConfig::interactive();
        let params = config.to_argon2_params().expect("Should convert to Argon2Params");
        assert_eq!(params.memory_cost, 32768);
        assert_eq!(params.time_cost, 2);
        assert_eq!(params.parallelism, 2);

        let config = KeyDerivationConfig::sensitive();
        let params = config.to_argon2_params().expect("Should convert to Argon2Params");
        assert_eq!(params.memory_cost, 131072);
        assert_eq!(params.time_cost, 4);
        assert_eq!(params.parallelism, 8);
    }

    #[test]
    fn test_key_derivation_config_custom() {
        let config = KeyDerivationConfig::custom(1024, 2, 2)
            .expect("Should create custom config");
        assert_eq!(config.memory_cost, 1024);
        assert_eq!(config.time_cost, 2);
        assert_eq!(config.parallelism, 2);
        assert!(config.validate().is_ok());

        // Invalid parameters should fail
        assert!(KeyDerivationConfig::custom(7, 2, 2).is_err()); // Memory too low
        assert!(KeyDerivationConfig::custom(1024, 0, 2).is_err()); // Time too low
        assert!(KeyDerivationConfig::custom(1024, 2, 0).is_err()); // Parallelism too low
    }

    #[test]
    fn test_config_serialization() {
        let config = CryptoConfig::with_passphrase("test-passphrase".to_string());
        
        let json = serde_json::to_string(&config).expect("Should serialize");
        let deserialized: CryptoConfig = serde_json::from_str(&json)
            .expect("Should deserialize");
        
        assert_eq!(config.enabled, deserialized.enabled);
        assert!(matches!(deserialized.master_key, MasterKeyConfig::Passphrase { .. }));
    }

    #[test]
    fn test_disabled_config_validation() {
        let config = CryptoConfig::disabled();
        assert!(!config.enabled);
        assert!(config.validate().is_ok());
        
        // Even with invalid passphrase, disabled config should pass validation
        let config = CryptoConfig {
            master_key: MasterKeyConfig::Passphrase {
                passphrase: "".to_string(), // Invalid but should be ignored when disabled
            },
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }
} 