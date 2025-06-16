//! Error handling for configuration traits
//!
//! This module provides comprehensive error types and context for trait-based
//! configuration operations, extending the base ConfigError with trait-specific
//! error variants and context information.

use std::fmt;
use thiserror::Error;

use crate::config::error::ConfigError;

/// Result type for trait-based configuration operations
pub type TraitConfigResult<T> = Result<T, TraitConfigError>;

/// Comprehensive error types for trait-based configuration operations
#[derive(Debug, Error)]
pub enum TraitConfigError {
    /// Base configuration error
    #[error("Configuration error: {0}")]
    Base(#[from] ConfigError),

    /// Trait validation error with context
    #[error("Trait validation failed: {message}")]
    TraitValidation {
        message: String,
        context: Option<ValidationContext>,
    },

    /// Legacy validation error variant for backward compatibility
    #[error("Validation error: {message}")]
    ValidationError {
        message: String,
        field: String,
        context: ValidationContext,
    },

    /// Legacy configuration error variant for backward compatibility
    #[error("Configuration error: {message}")]
    ConfigurationError {
        message: String,
        field: String,
        context: ValidationContext,
    },

    /// Trait composition error
    #[error("Trait composition error: {0}")]
    TraitComposition(String),

    /// Trait method not implemented
    #[error("Trait method not implemented: {trait_name}::{method_name}")]
    NotImplemented {
        trait_name: &'static str,
        method_name: &'static str,
    },

    /// Trait lifecycle error
    #[error("Trait lifecycle error: {0}")]
    Lifecycle(String),

    /// Trait serialization error
    #[error("Trait serialization error: {0}")]
    Serialization(String),

    /// Trait merge conflict
    #[error("Trait merge conflict: {0}")]
    MergeConflict(String),

    /// Trait reporting error
    #[error("Trait reporting error: {0}")]
    Reporting(String),

    /// Trait event handling error
    #[error("Trait event error: {0}")]
    Event(String),

    /// Cross-platform integration error
    #[error("Cross-platform integration error: {0}")]
    CrossPlatform(String),
}

/// Validation context for trait-specific validation errors
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// The trait that failed validation
    pub trait_name: &'static str,

    /// The specific validation rule that failed
    pub validation_rule: String,

    /// Path to the configuration value that failed
    pub value_path: Option<String>,

    /// Expected value or type
    pub expected: Option<String>,

    /// Actual value found
    pub actual: Option<String>,

    /// Additional context information
    pub additional_context: std::collections::HashMap<String, String>,
}

/// Error context wrapper for providing additional trait operation information
#[derive(Debug)]
pub struct ErrorContext {
    /// The underlying error
    pub error: TraitConfigError,

    /// The trait operation that was being performed
    pub operation: String,

    /// The trait name involved in the operation
    pub trait_name: Option<&'static str>,

    /// Configuration path being operated on
    pub config_path: Option<String>,

    /// Component or service name
    pub component: Option<String>,

    /// Additional context metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl TraitConfigError {
    /// Create a trait validation error with context
    pub fn trait_validation<S: Into<String>>(
        message: S,
        context: Option<ValidationContext>,
    ) -> Self {
        Self::TraitValidation {
            message: message.into(),
            context,
        }
    }

    /// Create a trait composition error
    pub fn trait_composition<S: Into<String>>(message: S) -> Self {
        Self::TraitComposition(message.into())
    }

    /// Create a not implemented error
    pub fn not_implemented(trait_name: &'static str, method_name: &'static str) -> Self {
        Self::NotImplemented {
            trait_name,
            method_name,
        }
    }

    /// Create a lifecycle error
    pub fn lifecycle<S: Into<String>>(message: S) -> Self {
        Self::Lifecycle(message.into())
    }

    /// Create a serialization error
    pub fn serialization<S: Into<String>>(message: S) -> Self {
        Self::Serialization(message.into())
    }

    /// Create a merge conflict error
    pub fn merge_conflict<S: Into<String>>(message: S) -> Self {
        Self::MergeConflict(message.into())
    }

    /// Create a reporting error
    pub fn reporting<S: Into<String>>(message: S) -> Self {
        Self::Reporting(message.into())
    }

    /// Create an event error
    pub fn event<S: Into<String>>(message: S) -> Self {
        Self::Event(message.into())
    }

    /// Create a cross-platform error
    pub fn cross_platform<S: Into<String>>(message: S) -> Self {
        Self::CrossPlatform(message.into())
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            TraitConfigError::Base(e) => e.is_recoverable(),
            TraitConfigError::TraitValidation { .. } => true,
            TraitConfigError::ValidationError { .. } => true,
            TraitConfigError::ConfigurationError { .. } => true,
            TraitConfigError::TraitComposition(_) => false,
            TraitConfigError::NotImplemented { .. } => false,
            TraitConfigError::Lifecycle(_) => true,
            TraitConfigError::Serialization(_) => false,
            TraitConfigError::MergeConflict(_) => true,
            TraitConfigError::Reporting(_) => true,
            TraitConfigError::Event(_) => true,
            TraitConfigError::CrossPlatform(_) => false,
        }
    }

    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            TraitConfigError::Base(e) => e.user_message(),
            TraitConfigError::TraitValidation { message, context } => {
                let mut msg = format!("Configuration validation failed: {}", message);
                if let Some(ctx) = context {
                    msg.push_str(&format!(" (in {})", ctx.trait_name));
                    if let Some(path) = &ctx.value_path {
                        msg.push_str(&format!(" at path '{}'", path));
                    }
                }
                msg
            }
            TraitConfigError::ValidationError { message, field, .. } => {
                format!("Validation error in field '{}': {}", field, message)
            }
            TraitConfigError::ConfigurationError { message, field, .. } => {
                format!("Configuration error in field '{}': {}", field, message)
            }
            TraitConfigError::TraitComposition(msg) => {
                format!("Configuration trait composition failed: {}", msg)
            }
            TraitConfigError::NotImplemented {
                trait_name,
                method_name,
            } => {
                format!(
                    "Configuration method {}::{} is not implemented",
                    trait_name, method_name
                )
            }
            TraitConfigError::Lifecycle(msg) => {
                format!("Configuration lifecycle error: {}", msg)
            }
            TraitConfigError::Serialization(msg) => {
                format!("Configuration serialization failed: {}", msg)
            }
            TraitConfigError::MergeConflict(msg) => {
                format!("Configuration merge conflict: {}", msg)
            }
            TraitConfigError::Reporting(msg) => {
                format!("Configuration reporting failed: {}", msg)
            }
            TraitConfigError::Event(msg) => {
                format!("Configuration event error: {}", msg)
            }
            TraitConfigError::CrossPlatform(msg) => {
                format!("Cross-platform configuration error: {}", msg)
            }
        }
    }
}

impl ValidationContext {
    /// Create a new validation context
    pub fn new(trait_name: &'static str, validation_rule: String) -> Self {
        Self {
            trait_name,
            validation_rule,
            value_path: None,
            expected: None,
            actual: None,
            additional_context: std::collections::HashMap::new(),
        }
    }

    /// Add value path context
    pub fn with_path<S: Into<String>>(mut self, path: S) -> Self {
        self.value_path = Some(path.into());
        self
    }

    /// Add expected value context
    pub fn with_expected<S: Into<String>>(mut self, expected: S) -> Self {
        self.expected = Some(expected.into());
        self
    }

    /// Add actual value context
    pub fn with_actual<S: Into<String>>(mut self, actual: S) -> Self {
        self.actual = Some(actual.into());
        self
    }

    /// Add additional context
    pub fn with_context<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.additional_context.insert(key.into(), value.into());
        self
    }
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self::new("UnknownTrait", "default_validation".to_string())
    }
}

impl ErrorContext {
    /// Create new error context
    pub fn new(error: TraitConfigError, operation: String) -> Self {
        Self {
            error,
            operation,
            trait_name: None,
            config_path: None,
            component: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Add trait name context
    pub fn with_trait(mut self, trait_name: &'static str) -> Self {
        self.trait_name = Some(trait_name);
        self
    }

    /// Add configuration path context
    pub fn with_config_path<S: Into<String>>(mut self, path: S) -> Self {
        self.config_path = Some(path.into());
        self
    }

    /// Add component context
    pub fn with_component<S: Into<String>>(mut self, component: S) -> Self {
        self.component = Some(component.into());
        self
    }

    /// Add metadata
    pub fn with_metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Configuration trait error during '{}': {}",
            self.operation, self.error
        )?;

        if let Some(trait_name) = self.trait_name {
            write!(f, " (trait: {})", trait_name)?;
        }

        if let Some(path) = &self.config_path {
            write!(f, " (path: {})", path)?;
        }

        if let Some(component) = &self.component {
            write!(f, " (component: {})", component)?;
        }

        Ok(())
    }
}

impl std::error::Error for ErrorContext {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

// Conversion from base ConfigError to TraitConfigError
impl From<TraitConfigError> for ConfigError {
    fn from(error: TraitConfigError) -> Self {
        match error {
            TraitConfigError::Base(e) => e,
            _ => ConfigError::runtime(error.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trait_validation_error() {
        let context = ValidationContext::new("TestTrait", "required_field".to_string())
            .with_path("section.field")
            .with_expected("non-empty string")
            .with_actual("empty string");

        let error = TraitConfigError::trait_validation("Field validation failed", Some(context));
        assert!(error.is_recoverable());

        let user_msg = error.user_message();
        assert!(user_msg.contains("TestTrait"));
        assert!(user_msg.contains("section.field"));
    }

    #[test]
    fn test_error_context() {
        let error = TraitConfigError::lifecycle("Failed to load configuration");
        let ctx = ErrorContext::new(error, "load_config".to_string())
            .with_trait("BaseConfig")
            .with_config_path("/etc/datafold/config.toml")
            .with_component("ConfigManager");

        let msg = format!("{}", ctx);
        assert!(msg.contains("load_config"));
        assert!(msg.contains("BaseConfig"));
        assert!(msg.contains("/etc/datafold/config.toml"));
    }

    #[test]
    fn test_not_implemented_error() {
        let error = TraitConfigError::not_implemented("TestTrait", "test_method");
        assert!(!error.is_recoverable());

        let user_msg = error.user_message();
        assert!(user_msg.contains("TestTrait::test_method"));
        assert!(user_msg.contains("not implemented"));
    }
}
