//! Configuration error types and handling
//!
//! This module provides comprehensive error types for all configuration operations,
//! with proper error context and user-friendly messages.

use std::fmt;

/// Comprehensive error types for configuration operations
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// IO-related errors (file access, permissions, etc.)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML parsing/serialization errors
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    /// TOML serialization errors
    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    /// JSON parsing errors (for legacy migration)
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    /// Configuration validation errors
    #[error("Configuration validation error: {0}")]
    Validation(String),

    /// Platform detection/resolution errors
    #[error("Platform error: {0}")]
    Platform(String),

    /// Path resolution errors
    #[error("Path resolution error: {0}")]
    PathResolution(String),

    /// Configuration not found errors
    #[error("Configuration not found: {0}")]
    NotFound(String),

    /// Access permission errors
    #[error("Access denied: {0}")]
    AccessDenied(String),

    /// Configuration format version mismatch
    #[error("Format version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: String, found: String },

    /// Encryption/decryption errors
    #[error("Encryption error: {0}")]
    Encryption(String),

    /// Configuration merge conflicts
    #[error("Merge conflict: {0}")]
    MergeConflict(String),

    /// Runtime configuration errors
    #[error("Runtime error: {0}")]
    Runtime(String),
}

impl ConfigError {
    /// Create a validation error with context
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a platform error with context
    pub fn platform<S: Into<String>>(msg: S) -> Self {
        Self::Platform(msg.into())
    }

    /// Create a path resolution error with context
    pub fn path_resolution<S: Into<String>>(msg: S) -> Self {
        Self::PathResolution(msg.into())
    }

    /// Create a not found error with context
    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create an access denied error with context
    pub fn access_denied<S: Into<String>>(msg: S) -> Self {
        Self::AccessDenied(msg.into())
    }

    /// Create an encryption error with context
    pub fn encryption<S: Into<String>>(msg: S) -> Self {
        Self::Encryption(msg.into())
    }

    /// Create a merge conflict error with context
    pub fn merge_conflict<S: Into<String>>(msg: S) -> Self {
        Self::MergeConflict(msg.into())
    }

    /// Create a runtime error with context
    pub fn runtime<S: Into<String>>(msg: S) -> Self {
        Self::Runtime(msg.into())
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            ConfigError::Io(_) => false,
            ConfigError::Toml(_) => false,
            ConfigError::TomlSer(_) => false,
            ConfigError::Json(_) => false,
            ConfigError::Validation(_) => true,
            ConfigError::Platform(_) => false,
            ConfigError::PathResolution(_) => false,
            ConfigError::NotFound(_) => true,
            ConfigError::AccessDenied(_) => false,
            ConfigError::VersionMismatch { .. } => true,
            ConfigError::Encryption(_) => false,
            ConfigError::MergeConflict(_) => true,
            ConfigError::Runtime(_) => true,
        }
    }

    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            ConfigError::Io(e) => format!("Failed to access configuration file: {}", e),
            ConfigError::Toml(e) => format!("Configuration file format error: {}", e),
            ConfigError::TomlSer(e) => format!("Failed to save configuration: {}", e),
            ConfigError::Json(e) => format!("Legacy configuration format error: {}", e),
            ConfigError::Validation(msg) => format!("Configuration validation failed: {}", msg),
            ConfigError::Platform(msg) => format!("Platform detection failed: {}", msg),
            ConfigError::PathResolution(msg) => format!("Could not resolve configuration path: {}", msg),
            ConfigError::NotFound(msg) => format!("Configuration not found: {}", msg),
            ConfigError::AccessDenied(msg) => format!("Access denied: {}", msg),
            ConfigError::VersionMismatch { expected, found } => {
                format!("Configuration version mismatch: expected {}, found {}", expected, found)
            }
            ConfigError::Encryption(msg) => format!("Encryption error: {}", msg),
            ConfigError::MergeConflict(msg) => format!("Configuration merge conflict: {}", msg),
            ConfigError::Runtime(msg) => format!("Runtime configuration error: {}", msg),
        }
    }
}

/// Result type for configuration operations
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Context wrapper for providing additional error information
#[derive(Debug)]
pub struct ConfigErrorContext {
    pub error: ConfigError,
    pub operation: String,
    pub path: Option<String>,
    pub component: Option<String>,
}

impl ConfigErrorContext {
    /// Create new error context
    pub fn new(error: ConfigError, operation: String) -> Self {
        Self {
            error,
            operation,
            path: None,
            component: None,
        }
    }

    /// Add path context
    pub fn with_path<S: Into<String>>(mut self, path: S) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Add component context
    pub fn with_component<S: Into<String>>(mut self, component: S) -> Self {
        self.component = Some(component.into());
        self
    }
}

impl fmt::Display for ConfigErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Configuration error during '{}': {}", self.operation, self.error)?;
        
        if let Some(path) = &self.path {
            write!(f, " (path: {})", path)?;
        }
        
        if let Some(component) = &self.component {
            write!(f, " (component: {})", component)?;
        }
        
        Ok(())
    }
}

impl std::error::Error for ConfigErrorContext {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = ConfigError::validation("test validation error");
        assert!(matches!(err, ConfigError::Validation(_)));
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_error_context() {
        let err = ConfigError::not_found("config.toml");
        let ctx = ConfigErrorContext::new(err, "load_config".to_string())
            .with_path("/home/user/.config/datafold/config.toml")
            .with_component("ConfigManager");
        
        let msg = format!("{}", ctx);
        assert!(msg.contains("load_config"));
        assert!(msg.contains("Configuration not found"));
    }

    #[test]
    fn test_user_messages() {
        let err = ConfigError::access_denied("insufficient permissions");
        let user_msg = err.user_message();
        assert!(user_msg.contains("Access denied"));
        assert!(!err.is_recoverable());
    }
}