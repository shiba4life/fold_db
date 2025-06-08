//! Enhanced error handling for cryptographic operations
//!
//! This module provides comprehensive error types with detailed context,
//! recovery suggestions, and error classification for all encryption-related
//! operations in DataFold.

use thiserror::Error;
use std::fmt;
use std::time::SystemTime;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Result type alias for enhanced crypto operations
pub type EnhancedCryptoResult<T> = Result<T, EnhancedCryptoError>;

/// Enhanced error classification for error handling strategies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Low severity - operation can continue with degraded functionality
    Low,
    /// Medium severity - operation should be retried or alternative approach used
    Medium,
    /// High severity - operation should be aborted but system can continue
    High,
    /// Critical severity - system integrity is at risk, immediate intervention required
    Critical,
}

/// Error recovery suggestions for automated and manual resolution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryAction {
    /// Retry the operation with the same parameters
    Retry,
    /// Retry with different parameters (e.g., different key derivation params)
    RetryWithDifferentParams,
    /// Generate new cryptographic material (keys, nonces, etc.)
    RegenerateCryptoMaterial,
    /// Fallback to unencrypted operation (if security policy allows)
    FallbackToUnencrypted,
    /// Restore from backup
    RestoreFromBackup,
    /// Manual intervention required - check configuration
    CheckConfiguration,
    /// Manual intervention required - check permissions
    CheckPermissions,
    /// Manual intervention required - check disk space
    CheckDiskSpace,
    /// Manual intervention required - contact administrator
    ContactAdministrator,
    /// No recovery possible - data may be lost
    NoRecovery,
}

/// Detailed context for error occurrence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Unique identifier for this error instance
    pub error_id: Uuid,
    /// Timestamp when the error occurred
    pub timestamp: SystemTime,
    /// Module or component where the error occurred
    pub component: String,
    /// Operation that was being performed
    pub operation: String,
    /// Additional context-specific data
    pub metadata: std::collections::HashMap<String, String>,
    /// Stack trace of error propagation
    pub propagation_chain: Vec<String>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(component: &str, operation: &str) -> Self {
        Self {
            error_id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            component: component.to_string(),
            operation: operation.to_string(),
            metadata: std::collections::HashMap::new(),
            propagation_chain: Vec::new(),
        }
    }

    /// Add metadata to the error context
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Add multiple metadata entries
    pub fn with_multiple_metadata(mut self, metadata: std::collections::HashMap<String, String>) -> Self {
        self.metadata.extend(metadata);
        self
    }

    /// Add a step to the propagation chain
    pub fn propagate(mut self, step: &str) -> Self {
        self.propagation_chain.push(step.to_string());
        self
    }
}

/// Enhanced cryptographic error with comprehensive context and recovery information
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum EnhancedCryptoError {
    /// Error during key generation with specific failure details
    #[error("Key generation failed: {message}")]
    KeyGeneration {
        message: String,
        severity: ErrorSeverity,
        recovery_actions: Vec<RecoveryAction>,
        context: ErrorContext,
        underlying_cause: Option<String>,
    },

    /// Error during key derivation with parameter details
    #[error("Key derivation failed: {message}")]
    KeyDerivation {
        message: String,
        severity: ErrorSeverity,
        recovery_actions: Vec<RecoveryAction>,
        context: ErrorContext,
        parameters_used: Option<String>,
        underlying_cause: Option<String>,
    },

    /// Error during encryption operation
    #[error("Encryption failed: {message}")]
    Encryption {
        message: String,
        severity: ErrorSeverity,
        recovery_actions: Vec<RecoveryAction>,
        context: ErrorContext,
        data_size: Option<usize>,
        encryption_context: Option<String>,
        underlying_cause: Option<String>,
    },

    /// Error during decryption operation
    #[error("Decryption failed: {message}")]
    Decryption {
        message: String,
        severity: ErrorSeverity,
        recovery_actions: Vec<RecoveryAction>,
        context: ErrorContext,
        data_size: Option<usize>,
        encryption_context: Option<String>,
        underlying_cause: Option<String>,
    },

    /// Error during signature operations
    #[error("Signature operation failed: {message}")]
    Signature {
        message: String,
        severity: ErrorSeverity,
        recovery_actions: Vec<RecoveryAction>,
        context: ErrorContext,
        operation_type: String, // "sign" or "verify"
        underlying_cause: Option<String>,
    },

    /// Invalid key material provided
    #[error("Invalid key material: {message}")]
    InvalidKey {
        message: String,
        severity: ErrorSeverity,
        recovery_actions: Vec<RecoveryAction>,
        context: ErrorContext,
        key_type: String,
        expected_format: Option<String>,
        underlying_cause: Option<String>,
    },

    /// Random number generation error
    #[error("Random number generation failed: {message}")]
    RandomGeneration {
        message: String,
        severity: ErrorSeverity,
        recovery_actions: Vec<RecoveryAction>,
        context: ErrorContext,
        requested_bytes: Option<usize>,
        underlying_cause: Option<String>,
    },

    /// Error during serialization/deserialization of crypto material
    #[error("Serialization failed: {message}")]
    Serialization {
        message: String,
        severity: ErrorSeverity,
        recovery_actions: Vec<RecoveryAction>,
        context: ErrorContext,
        data_type: String,
        operation: String, // "serialize" or "deserialize"
        underlying_cause: Option<String>,
    },

    /// Configuration-related error
    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        severity: ErrorSeverity,
        recovery_actions: Vec<RecoveryAction>,
        context: ErrorContext,
        config_key: Option<String>,
        expected_value: Option<String>,
        actual_value: Option<String>,
        underlying_cause: Option<String>,
    },

    /// Storage-related error (file I/O, database operations)
    #[error("Storage operation failed: {message}")]
    Storage {
        message: String,
        severity: ErrorSeverity,
        recovery_actions: Vec<RecoveryAction>,
        context: ErrorContext,
        operation: String,
        path: Option<String>,
        underlying_cause: Option<String>,
    },

    /// Security-related error (unauthorized access, tampering detection)
    #[error("Security violation detected: {message}")]
    Security {
        message: String,
        severity: ErrorSeverity,
        recovery_actions: Vec<RecoveryAction>,
        context: ErrorContext,
        security_event: String,
        threat_level: String,
        underlying_cause: Option<String>,
    },

    /// Performance-related error (timeouts, resource exhaustion)
    #[error("Performance issue: {message}")]
    Performance {
        message: String,
        severity: ErrorSeverity,
        recovery_actions: Vec<RecoveryAction>,
        context: ErrorContext,
        operation_duration: Option<std::time::Duration>,
        resource_type: Option<String>,
        underlying_cause: Option<String>,
    },

    /// Network-related error for distributed operations
    #[error("Network operation failed: {message}")]
    Network {
        message: String,
        severity: ErrorSeverity,
        recovery_actions: Vec<RecoveryAction>,
        context: ErrorContext,
        endpoint: Option<String>,
        operation: String,
        underlying_cause: Option<String>,
    },

    /// Validation error for input parameters
    #[error("Validation failed: {message}")]
    Validation {
        message: String,
        severity: ErrorSeverity,
        recovery_actions: Vec<RecoveryAction>,
        context: ErrorContext,
        field_name: Option<String>,
        expected_format: Option<String>,
        actual_value: Option<String>,
        underlying_cause: Option<String>,
    },

    /// Compatibility error (version mismatches, format incompatibilities)
    #[error("Compatibility issue: {message}")]
    Compatibility {
        message: String,
        severity: ErrorSeverity,
        recovery_actions: Vec<RecoveryAction>,
        context: ErrorContext,
        expected_version: Option<String>,
        actual_version: Option<String>,
        underlying_cause: Option<String>,
    },

    /// Resource exhaustion error
    #[error("Resource exhausted: {message}")]
    ResourceExhaustion {
        message: String,
        severity: ErrorSeverity,
        recovery_actions: Vec<RecoveryAction>,
        context: ErrorContext,
        resource_type: String,
        limit: Option<String>,
        current_usage: Option<String>,
        underlying_cause: Option<String>,
    },
}

impl EnhancedCryptoError {
    /// Get the error severity level
    pub fn severity(&self) -> &ErrorSeverity {
        match self {
            Self::KeyGeneration { severity, .. } => severity,
            Self::KeyDerivation { severity, .. } => severity,
            Self::Encryption { severity, .. } => severity,
            Self::Decryption { severity, .. } => severity,
            Self::Signature { severity, .. } => severity,
            Self::InvalidKey { severity, .. } => severity,
            Self::RandomGeneration { severity, .. } => severity,
            Self::Serialization { severity, .. } => severity,
            Self::Configuration { severity, .. } => severity,
            Self::Storage { severity, .. } => severity,
            Self::Security { severity, .. } => severity,
            Self::Performance { severity, .. } => severity,
            Self::Network { severity, .. } => severity,
            Self::Validation { severity, .. } => severity,
            Self::Compatibility { severity, .. } => severity,
            Self::ResourceExhaustion { severity, .. } => severity,
        }
    }

    /// Get recovery actions for this error
    pub fn recovery_actions(&self) -> &[RecoveryAction] {
        match self {
            Self::KeyGeneration { recovery_actions, .. } => recovery_actions,
            Self::KeyDerivation { recovery_actions, .. } => recovery_actions,
            Self::Encryption { recovery_actions, .. } => recovery_actions,
            Self::Decryption { recovery_actions, .. } => recovery_actions,
            Self::Signature { recovery_actions, .. } => recovery_actions,
            Self::InvalidKey { recovery_actions, .. } => recovery_actions,
            Self::RandomGeneration { recovery_actions, .. } => recovery_actions,
            Self::Serialization { recovery_actions, .. } => recovery_actions,
            Self::Configuration { recovery_actions, .. } => recovery_actions,
            Self::Storage { recovery_actions, .. } => recovery_actions,
            Self::Security { recovery_actions, .. } => recovery_actions,
            Self::Performance { recovery_actions, .. } => recovery_actions,
            Self::Network { recovery_actions, .. } => recovery_actions,
            Self::Validation { recovery_actions, .. } => recovery_actions,
            Self::Compatibility { recovery_actions, .. } => recovery_actions,
            Self::ResourceExhaustion { recovery_actions, .. } => recovery_actions,
        }
    }

    /// Get error context
    pub fn context(&self) -> &ErrorContext {
        match self {
            Self::KeyGeneration { context, .. } => context,
            Self::KeyDerivation { context, .. } => context,
            Self::Encryption { context, .. } => context,
            Self::Decryption { context, .. } => context,
            Self::Signature { context, .. } => context,
            Self::InvalidKey { context, .. } => context,
            Self::RandomGeneration { context, .. } => context,
            Self::Serialization { context, .. } => context,
            Self::Configuration { context, .. } => context,
            Self::Storage { context, .. } => context,
            Self::Security { context, .. } => context,
            Self::Performance { context, .. } => context,
            Self::Network { context, .. } => context,
            Self::Validation { context, .. } => context,
            Self::Compatibility { context, .. } => context,
            Self::ResourceExhaustion { context, .. } => context,
        }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        !self.recovery_actions().is_empty() && 
        !self.recovery_actions().contains(&RecoveryAction::NoRecovery)
    }

    /// Check if this error should trigger an immediate alert
    pub fn should_alert(&self) -> bool {
        matches!(self.severity(), ErrorSeverity::High | ErrorSeverity::Critical) ||
        matches!(self, Self::Security { .. })
    }

    /// Get a detailed description for logging
    pub fn detailed_description(&self) -> String {
        format!(
            "Error ID: {}, Component: {}, Operation: {}, Severity: {:?}, Message: {}, Recovery Actions: {:?}",
            self.context().error_id,
            self.context().component,
            self.context().operation,
            self.severity(),
            self,
            self.recovery_actions()
        )
    }

    /// Convert to a structured format for JSON logging
    pub fn to_structured_log(&self) -> serde_json::Value {
        serde_json::json!({
            "error_id": self.context().error_id,
            "timestamp": self.context().timestamp.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_secs(),
            "component": self.context().component,
            "operation": self.context().operation,
            "severity": self.severity(),
            "message": self.to_string(),
            "recovery_actions": self.recovery_actions(),
            "metadata": self.context().metadata,
            "propagation_chain": self.context().propagation_chain,
            "error_type": self.error_type_name(),
        })
    }

    /// Get the error type name for categorization
    pub fn error_type_name(&self) -> &'static str {
        match self {
            Self::KeyGeneration { .. } => "key_generation",
            Self::KeyDerivation { .. } => "key_derivation",
            Self::Encryption { .. } => "encryption",
            Self::Decryption { .. } => "decryption",
            Self::Signature { .. } => "signature",
            Self::InvalidKey { .. } => "invalid_key",
            Self::RandomGeneration { .. } => "random_generation",
            Self::Serialization { .. } => "serialization",
            Self::Configuration { .. } => "configuration",
            Self::Storage { .. } => "storage",
            Self::Security { .. } => "security",
            Self::Performance { .. } => "performance",
            Self::Network { .. } => "network",
            Self::Validation { .. } => "validation",
            Self::Compatibility { .. } => "compatibility",
            Self::ResourceExhaustion { .. } => "resource_exhaustion",
        }
    }
}

/// Conversion from the original CryptoError to EnhancedCryptoError
impl From<crate::crypto::error::CryptoError> for EnhancedCryptoError {
    fn from(err: crate::crypto::error::CryptoError) -> Self {
        let context = ErrorContext::new("crypto", "legacy_operation");
        
        match err {
            crate::crypto::error::CryptoError::KeyGeneration { message } => {
                Self::KeyGeneration {
                    message,
                    severity: ErrorSeverity::High,
                    recovery_actions: vec![RecoveryAction::Retry, RecoveryAction::RegenerateCryptoMaterial],
                    context,
                    underlying_cause: None,
                }
            }
            crate::crypto::error::CryptoError::Serialization { message } => {
                Self::Serialization {
                    message,
                    severity: ErrorSeverity::Medium,
                    recovery_actions: vec![RecoveryAction::Retry, RecoveryAction::CheckConfiguration],
                    context,
                    data_type: "unknown".to_string(),
                    operation: "unknown".to_string(),
                    underlying_cause: None,
                }
            }
            crate::crypto::error::CryptoError::Deserialization { message } => {
                Self::Serialization {
                    message,
                    severity: ErrorSeverity::Medium,
                    recovery_actions: vec![RecoveryAction::CheckConfiguration, RecoveryAction::RestoreFromBackup],
                    context,
                    data_type: "unknown".to_string(),
                    operation: "deserialize".to_string(),
                    underlying_cause: None,
                }
            }
            crate::crypto::error::CryptoError::InvalidKey { message } => {
                Self::InvalidKey {
                    message,
                    severity: ErrorSeverity::High,
                    recovery_actions: vec![RecoveryAction::RegenerateCryptoMaterial, RecoveryAction::CheckConfiguration],
                    context,
                    key_type: "unknown".to_string(),
                    expected_format: None,
                    underlying_cause: None,
                }
            }
            crate::crypto::error::CryptoError::Signature { message } => {
                Self::Signature {
                    message,
                    severity: ErrorSeverity::High,
                    recovery_actions: vec![RecoveryAction::Retry, RecoveryAction::RegenerateCryptoMaterial],
                    context,
                    operation_type: "unknown".to_string(),
                    underlying_cause: None,
                }
            }
            crate::crypto::error::CryptoError::KeyDerivation { message } => {
                Self::KeyDerivation {
                    message,
                    severity: ErrorSeverity::High,
                    recovery_actions: vec![RecoveryAction::RetryWithDifferentParams, RecoveryAction::CheckConfiguration],
                    context,
                    parameters_used: None,
                    underlying_cause: None,
                }
            }
            crate::crypto::error::CryptoError::RandomGeneration { message } => {
                Self::RandomGeneration {
                    message,
                    severity: ErrorSeverity::Critical,
                    recovery_actions: vec![RecoveryAction::Retry, RecoveryAction::ContactAdministrator],
                    context,
                    requested_bytes: None,
                    underlying_cause: None,
                }
            }
            crate::crypto::error::CryptoError::SignatureVerification => {
                Self::Signature {
                    message: "Signature verification failed".to_string(),
                    severity: ErrorSeverity::High,
                    recovery_actions: vec![RecoveryAction::Retry, RecoveryAction::CheckConfiguration],
                    context,
                    operation_type: "verify".to_string(),
                    underlying_cause: None,
                }
            }
            crate::crypto::error::CryptoError::InvalidSignature { message } => {
                Self::Signature {
                    message,
                    severity: ErrorSeverity::Medium,
                    recovery_actions: vec![RecoveryAction::CheckConfiguration, RecoveryAction::Retry],
                    context,
                    operation_type: "validate".to_string(),
                    underlying_cause: None,
                }
            }
            crate::crypto::error::CryptoError::InvalidInput(message) => {
                Self::Validation {
                    message,
                    severity: ErrorSeverity::Medium,
                    recovery_actions: vec![RecoveryAction::CheckConfiguration, RecoveryAction::Retry],
                    context,
                    field_name: None,
                    expected_format: None,
                    actual_value: None,
                    underlying_cause: None,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context_creation() {
        let context = ErrorContext::new("test_component", "test_operation")
            .with_metadata("key1", "value1")
            .propagate("step1");

        assert_eq!(context.component, "test_component");
        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(context.propagation_chain, vec!["step1"]);
    }

    #[test]
    fn test_enhanced_crypto_error_properties() {
        let context = ErrorContext::new("test", "test_op");
        let error = EnhancedCryptoError::KeyGeneration {
            message: "Test error".to_string(),
            severity: ErrorSeverity::High,
            recovery_actions: vec![RecoveryAction::Retry],
            context,
            underlying_cause: None,
        };

        assert_eq!(*error.severity(), ErrorSeverity::High);
        assert_eq!(error.recovery_actions(), &[RecoveryAction::Retry]);
        assert!(error.is_recoverable());
        assert!(error.should_alert());
        assert_eq!(error.error_type_name(), "key_generation");
    }

    #[test]
    fn test_error_conversion_from_legacy() {
        let legacy_error = crate::crypto::error::CryptoError::KeyGeneration {
            message: "Legacy error".to_string(),
        };
        let enhanced_error = EnhancedCryptoError::from(legacy_error);

        assert_eq!(enhanced_error.error_type_name(), "key_generation");
        assert!(enhanced_error.is_recoverable());
    }

    #[test]
    fn test_structured_log_format() {
        let context = ErrorContext::new("test", "test_op");
        let error = EnhancedCryptoError::Security {
            message: "Security test".to_string(),
            severity: ErrorSeverity::Critical,
            recovery_actions: vec![RecoveryAction::ContactAdministrator],
            context,
            security_event: "unauthorized_access".to_string(),
            threat_level: "high".to_string(),
            underlying_cause: None,
        };

        let log = error.to_structured_log();
        assert_eq!(log["error_type"], "security");
        assert_eq!(log["severity"], "Critical");
        assert_eq!(log["component"], "test");
    }
}