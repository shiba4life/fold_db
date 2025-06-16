//! Logging-specific configuration traits for the shared traits system
//!
//! This module provides domain-specific traits for logging configurations,
//! implementing common patterns found across logging configurations while
//! maintaining type safety and validation consistency.

use crate::config::traits::{
    BaseConfig, ConfigLifecycle, ConfigValidation, TraitConfigError, TraitConfigResult,
    ValidationContext,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Domain-specific trait for logging configurations
#[async_trait]
pub trait LoggingConfig: BaseConfig + ConfigValidation + ConfigLifecycle {
    /// Associated type for log level enumeration
    type LogLevel: Clone + std::fmt::Debug + PartialEq;

    /// Associated type for output configuration
    type OutputConfig: Clone + std::fmt::Debug;

    /// Associated type for platform-specific settings
    type PlatformSettings: Clone + std::fmt::Debug + Default;

    /// Get the default log level for the configuration
    fn default_log_level(&self) -> Self::LogLevel;

    /// Get all configured output settings
    fn output_configs(&self) -> Vec<Self::OutputConfig>;

    /// Get platform-specific logging settings
    fn platform_settings(&self) -> &Self::PlatformSettings;

    /// Validate a log level string and convert to typed level
    fn parse_log_level(&self, level: &str) -> TraitConfigResult<Self::LogLevel>;

    /// Validate file size format and convert to bytes
    fn parse_file_size(&self, size_str: &str) -> TraitConfigResult<u64> {
        let size_str = size_str.to_uppercase();

        if let Some(num_str) = size_str.strip_suffix("GB") {
            let num: u64 = num_str
                .parse()
                .map_err(|_| TraitConfigError::ValidationError {
                    field: "file_size".to_string(),
                    message: format!("Invalid file size format: {}", size_str),
                    context: ValidationContext::default(),
                })?;
            Ok(num * 1024 * 1024 * 1024)
        } else if let Some(num_str) = size_str.strip_suffix("MB") {
            let num: u64 = num_str
                .parse()
                .map_err(|_| TraitConfigError::ValidationError {
                    field: "file_size".to_string(),
                    message: format!("Invalid file size format: {}", size_str),
                    context: ValidationContext::default(),
                })?;
            Ok(num * 1024 * 1024)
        } else if let Some(num_str) = size_str.strip_suffix("KB") {
            let num: u64 = num_str
                .parse()
                .map_err(|_| TraitConfigError::ValidationError {
                    field: "file_size".to_string(),
                    message: format!("Invalid file size format: {}", size_str),
                    context: ValidationContext::default(),
                })?;
            Ok(num * 1024)
        } else if let Some(num_str) = size_str.strip_suffix("B") {
            num_str
                .parse()
                .map_err(|_| TraitConfigError::ValidationError {
                    field: "file_size".to_string(),
                    message: format!("Invalid file size format: {}", size_str),
                    context: ValidationContext::default(),
                })
        } else {
            // Default to bytes if no suffix
            size_str
                .parse()
                .map_err(|_| TraitConfigError::ValidationError {
                    field: "file_size".to_string(),
                    message: format!("Invalid file size format: {}", size_str),
                    context: ValidationContext::default(),
                })
        }
    }

    /// Apply environment variable overrides to the configuration
    async fn apply_env_overrides(&mut self) -> TraitConfigResult<()>;

    /// Validate log level consistency across all outputs
    fn validate_log_levels(&self) -> TraitConfigResult<()>;

    /// Validate output configuration consistency
    fn validate_outputs(&self) -> TraitConfigResult<()>;
}

/// Trait for log level validation and parsing
pub trait LogLevelTrait:
    Clone + std::fmt::Debug + PartialEq + Serialize + for<'de> Deserialize<'de>
{
    /// Parse a string into a log level
    fn from_str(s: &str) -> Result<Self, String>;

    /// Convert log level to string
    fn to_string(&self) -> String;

    /// Get all valid log level strings
    fn valid_levels() -> Vec<&'static str>;

    /// Check if this level is valid
    fn is_valid(&self) -> bool;

    /// Get numeric priority for comparison (lower = more verbose)
    fn priority(&self) -> u8;
}

/// Standard log level implementation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StandardLogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevelTrait for StandardLogLevel {
    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_uppercase().as_str() {
            "TRACE" => Ok(Self::Trace),
            "DEBUG" => Ok(Self::Debug),
            "INFO" => Ok(Self::Info),
            "WARN" => Ok(Self::Warn),
            "ERROR" => Ok(Self::Error),
            _ => Err(format!("Invalid log level: {}", s)),
        }
    }

    fn to_string(&self) -> String {
        match self {
            Self::Trace => "TRACE".to_string(),
            Self::Debug => "DEBUG".to_string(),
            Self::Info => "INFO".to_string(),
            Self::Warn => "WARN".to_string(),
            Self::Error => "ERROR".to_string(),
        }
    }

    fn valid_levels() -> Vec<&'static str> {
        vec!["TRACE", "DEBUG", "INFO", "WARN", "ERROR"]
    }

    fn is_valid(&self) -> bool {
        true // All enum variants are valid
    }

    fn priority(&self) -> u8 {
        match self {
            Self::Trace => 0,
            Self::Debug => 1,
            Self::Info => 2,
            Self::Warn => 3,
            Self::Error => 4,
        }
    }
}

/// Trait for output-specific configuration
pub trait OutputConfigTrait: std::fmt::Debug {
    /// Output type identifier
    fn output_type(&self) -> &str;

    /// Whether this output is enabled
    fn is_enabled(&self) -> bool;

    /// Get the log level for this output
    fn log_level(&self) -> &str;

    /// Validate output-specific settings
    fn validate(&self) -> TraitConfigResult<()>;

    /// Clone this configuration (for object safety)
    fn clone_config(&self) -> Box<dyn OutputConfigTrait>;
}

/// Trait for platform-specific logging settings
pub trait PlatformLogSettingsTrait: Clone + std::fmt::Debug + Default {
    /// Whether to use platform-specific log directories
    fn use_platform_paths(&self) -> bool;

    /// Whether platform optimizations are enabled
    fn optimizations_enabled(&self) -> bool;

    /// Apply platform-specific configuration updates
    fn apply_platform_defaults(&mut self) -> TraitConfigResult<()>;
}

/// Configuration for log rotation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    /// Maximum file size before rotation
    pub max_size: String,
    /// Maximum number of files to keep
    pub max_files: u32,
    /// Use compression for rotated files
    pub use_compression: bool,
    /// Auto-cleanup old files
    pub auto_cleanup: bool,
}

impl Default for LogRotationConfig {
    fn default() -> Self {
        Self {
            max_size: "10MB".to_string(),
            max_files: 5,
            use_compression: true,
            auto_cleanup: true,
        }
    }
}

impl LogRotationConfig {
    /// Validate rotation configuration
    pub fn validate(&self) -> TraitConfigResult<()> {
        // Validate max_size format
        self.parse_file_size(&self.max_size)?;

        // Validate max_files
        if self.max_files == 0 {
            return Err(TraitConfigError::ValidationError {
                field: "max_files".to_string(),
                message: "Maximum files must be at least 1".to_string(),
                context: ValidationContext::default(),
            });
        }

        Ok(())
    }

    /// Parse file size string to bytes
    fn parse_file_size(&self, size_str: &str) -> TraitConfigResult<u64> {
        let size_str = size_str.to_uppercase();

        if let Some(num_str) = size_str.strip_suffix("GB") {
            let num: u64 = num_str
                .parse()
                .map_err(|_| TraitConfigError::ValidationError {
                    field: "max_size".to_string(),
                    message: format!("Invalid file size format: {}", size_str),
                    context: ValidationContext::default(),
                })?;
            Ok(num * 1024 * 1024 * 1024)
        } else if let Some(num_str) = size_str.strip_suffix("MB") {
            let num: u64 = num_str
                .parse()
                .map_err(|_| TraitConfigError::ValidationError {
                    field: "max_size".to_string(),
                    message: format!("Invalid file size format: {}", size_str),
                    context: ValidationContext::default(),
                })?;
            Ok(num * 1024 * 1024)
        } else if let Some(num_str) = size_str.strip_suffix("KB") {
            let num: u64 = num_str
                .parse()
                .map_err(|_| TraitConfigError::ValidationError {
                    field: "max_size".to_string(),
                    message: format!("Invalid file size format: {}", size_str),
                    context: ValidationContext::default(),
                })?;
            Ok(num * 1024)
        } else if let Some(num_str) = size_str.strip_suffix("B") {
            num_str
                .parse()
                .map_err(|_| TraitConfigError::ValidationError {
                    field: "max_size".to_string(),
                    message: format!("Invalid file size format: {}", size_str),
                    context: ValidationContext::default(),
                })
        } else {
            // Default to bytes if no suffix
            size_str
                .parse()
                .map_err(|_| TraitConfigError::ValidationError {
                    field: "max_size".to_string(),
                    message: format!("Invalid file size format: {}", size_str),
                    context: ValidationContext::default(),
                })
        }
    }
}

/// Configuration for log formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFormatConfig {
    /// Include timestamps in log output
    pub include_timestamp: bool,
    /// Include module path in log output
    pub include_module: bool,
    /// Include thread information in log output
    pub include_thread: bool,
    /// Enable colored output
    pub enable_colors: bool,
    /// Custom format string (if supported by output)
    pub custom_format: Option<String>,
}

impl Default for LogFormatConfig {
    fn default() -> Self {
        Self {
            include_timestamp: true,
            include_module: true,
            include_thread: false,
            enable_colors: true,
            custom_format: None,
        }
    }
}

impl LogFormatConfig {
    /// Validate format configuration
    pub fn validate(&self) -> TraitConfigResult<()> {
        // Format configuration is generally valid by construction
        // Could add custom format string validation if needed
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_log_level() {
        assert_eq!(
            StandardLogLevel::from_str("INFO").unwrap(),
            StandardLogLevel::Info
        );
        assert_eq!(
            StandardLogLevel::from_str("info").unwrap(),
            StandardLogLevel::Info
        );
        assert!(StandardLogLevel::from_str("INVALID").is_err());

        assert_eq!(StandardLogLevel::Info.to_string(), "INFO");
        assert_eq!(StandardLogLevel::Info.priority(), 2);
        assert!(StandardLogLevel::Info.is_valid());
    }

    #[test]
    fn test_log_rotation_config() {
        let config = LogRotationConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = config.clone();
        invalid_config.max_files = 0;
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_file_size_parsing() {
        let config = LogRotationConfig::default();

        assert_eq!(config.parse_file_size("10MB").unwrap(), 10 * 1024 * 1024);
        assert_eq!(config.parse_file_size("1GB").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(config.parse_file_size("512KB").unwrap(), 512 * 1024);
        assert_eq!(config.parse_file_size("1024B").unwrap(), 1024);
        assert_eq!(config.parse_file_size("1024").unwrap(), 1024);

        assert!(config.parse_file_size("invalid").is_err());
        assert!(config.parse_file_size("10XB").is_err());
    }

    #[test]
    fn test_log_format_config() {
        let config = LogFormatConfig::default();
        assert!(config.validate().is_ok());
        assert!(config.include_timestamp);
        assert!(config.include_module);
        assert!(!config.include_thread);
        assert!(config.enable_colors);
    }
}
