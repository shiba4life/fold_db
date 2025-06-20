//! # Unified Cryptographic Error Handling
//!
//! This module provides comprehensive error types and handling for the unified
//! cryptographic system. It follows security-first design principles with
//! careful error information management to prevent information leakage.

use thiserror::Error;
use std::fmt;

/// Result type alias for unified crypto operations
pub type UnifiedCryptoResult<T> = Result<T, UnifiedCryptoError>;

/// Result type alias for security-critical operations
pub type SecurityResult<T> = Result<T, SecurityError>;

/// Comprehensive error types for unified cryptographic operations
///
/// This enum covers all possible error conditions in the unified crypto system
/// while maintaining security by not leaking sensitive information in error messages.
#[derive(Error, Debug)]
pub enum UnifiedCryptoError {
    /// Configuration validation or loading errors
    #[error("Configuration error: {message}")]
    Configuration { message: String },

    /// Invalid configuration detected
    #[error("Invalid configuration: {message}")]
    InvalidConfiguration { message: String },

    /// Key management operation errors
    #[error("Key management error: {operation}")]
    KeyManagement { operation: String },

    /// Key derivation operation errors
    #[error("Key derivation failed: {details}")]
    KeyDerivation { details: String },

    /// Key generation operation errors
    #[error("Key generation failed: {details}")]
    KeyGeneration { details: String },

    /// Crypto validation errors
    #[error("Crypto validation failed: {0}")]
    CryptoValidation(#[from] Box<dyn std::error::Error + Send + Sync>),

    /// Invalid parameter errors
    #[error("Invalid parameter: {message}")]
    InvalidParameter { message: String },

    /// Digital signature operation errors
    #[error("Signature operation failed: {details}")]
    Signature { details: String },

    /// Cryptographic primitive operation errors
    #[error("Cryptographic operation failed: {operation}")]
    CryptographicOperation { operation: String },

    /// Input validation errors (non-sensitive details only)
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    /// Algorithm or cipher suite not supported
    #[error("Unsupported algorithm: {algorithm}")]
    UnsupportedAlgorithm { algorithm: String },

    /// Key format or encoding errors
    #[error("Key format error: {details}")]
    KeyFormat { details: String },

    /// Random number generation errors
    #[error("Random number generation failed")]
    RandomGeneration,

    /// Memory allocation or security errors
    #[error("Memory security error")]
    MemorySecurity,

    /// Audit logging errors (should not interrupt crypto operations)
    #[error("Audit logging error: {details}")]
    AuditLogging { details: String },

    /// Policy enforcement errors
    #[error("Security policy violation: {policy}")]
    PolicyViolation { policy: String },

    /// Hardware security module errors
    #[error("Hardware security module error")]
    HardwareSecurityModule,

    /// Time-based security errors (timing attacks, replay prevention)
    #[error("Timing security error")]
    TimingSecurity,

    /// Access control and authorization errors
    #[error("Access denied: insufficient privileges")]
    AccessDenied,

    /// Resource exhaustion errors
    #[error("Resource exhaustion: {resource}")]
    ResourceExhaustion { resource: String },

    /// Internal system errors (should not happen in normal operation)
    #[error("Internal error: {context}")]
    Internal { context: String },

    /// Authentication failures
    #[error("Authentication failed: {message}")]
    AuthenticationError { message: String },

    /// Concurrency control errors
    #[error("Concurrency error: {message}")]
    ConcurrencyError { message: String },

    /// Data integrity verification errors
    #[error("Integrity error: {message}")]
    IntegrityError { message: String },

    /// Rate limiting errors
    #[error("Rate limit exceeded: {message}")]
    RateLimitExceeded { message: String },

    /// Serialization/deserialization errors
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
}

/// Security-specific error types for high-stakes operations
///
/// These errors are used for security-critical operations where additional
/// context and handling may be required.
#[derive(Error, Debug)]
pub enum SecurityError {
    /// Authentication failures
    #[error("Authentication failed")]
    AuthenticationFailed,

    /// Authorization failures
    #[error("Authorization failed: {context}")]
    AuthorizationFailed { context: String },

    /// Key compromise or security breach detection
    #[error("Security breach detected")]
    SecurityBreach,

    /// Cryptographic verification failures
    #[error("Cryptographic verification failed")]
    VerificationFailed,

    /// Tamper detection in audit logs or key material
    #[error("Tamper detection: {component}")]
    TamperDetection { component: String },

    /// Rate limiting or abuse prevention
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Invalid security context or state
    #[error("Invalid security context")]
    InvalidSecurityContext,

    /// Cryptographic key lifecycle violations
    #[error("Key lifecycle violation: {violation}")]
    KeyLifecycleViolation { violation: String },

    /// Security policy enforcement failures
    #[error("Security policy enforcement failed: {policy}")]
    PolicyEnforcementFailed { policy: String },

    /// Underlying unified crypto error
    #[error("Cryptographic error: {0}")]
    CryptographicError(#[from] UnifiedCryptoError),
}

/// Error context for enhanced error reporting and debugging
///
/// Provides additional context while maintaining security by not exposing
/// sensitive information in error traces.
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Operation that failed
    pub operation: String,
    /// Component where error occurred
    pub component: String,
    /// Error severity level
    pub severity: ErrorSeverity,
    /// Timestamp of error occurrence
    pub timestamp: std::time::SystemTime,
    /// Correlation ID for audit trail
    pub correlation_id: String,
}

/// Error severity levels for prioritization and handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Low severity - logging/monitoring issues
    Low,
    /// Medium severity - degraded functionality
    Medium,
    /// High severity - security implications
    High,
    /// Critical severity - system compromise risk
    Critical,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(operation: &str, component: &str, severity: ErrorSeverity) -> Self {
        Self {
            operation: operation.to_string(),
            component: component.to_string(),
            severity,
            timestamp: std::time::SystemTime::now(),
            correlation_id: generate_correlation_id(),
        }
    }

    /// Create error context for cryptographic operations
    pub fn crypto_operation(operation: &str, severity: ErrorSeverity) -> Self {
        Self::new(operation, "unified_crypto", severity)
    }

    /// Create error context for key management operations
    pub fn key_management(operation: &str, severity: ErrorSeverity) -> Self {
        Self::new(operation, "key_manager", severity)
    }

    /// Create error context for security violations
    pub fn security_violation(violation: &str) -> Self {
        Self::new(violation, "security_monitor", ErrorSeverity::High)
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} in {} (severity: {:?}, correlation: {})",
            format_timestamp(self.timestamp),
            self.operation,
            self.component,
            self.severity,
            self.correlation_id
        )
    }
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Low => write!(f, "LOW"),
            ErrorSeverity::Medium => write!(f, "MEDIUM"),
            ErrorSeverity::High => write!(f, "HIGH"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

// Note: Legacy crypto error conversions removed to avoid conflicts
// Individual modules can implement specific conversions as needed

// Standard library error conversions
impl From<std::io::Error> for UnifiedCryptoError {
    fn from(err: std::io::Error) -> Self {
        UnifiedCryptoError::Internal {
            context: format!("I/O error: {}", err),
        }
    }
}

impl From<serde_json::Error> for UnifiedCryptoError {
    fn from(err: serde_json::Error) -> Self {
        UnifiedCryptoError::Configuration {
            message: format!("JSON serialization error: {}", err),
        }
    }
}

/// Generate a unique correlation ID for error tracking
fn generate_correlation_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    
    // Simple correlation ID - in production, consider using UUID or similar
    format!("crypto-{:x}", timestamp)
}

/// Format timestamp for error display
fn format_timestamp(timestamp: std::time::SystemTime) -> String {
    use std::time::UNIX_EPOCH;
    
    let duration = timestamp.duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    
    format!("{}.{:03}", duration.as_secs(), duration.subsec_millis())
}

/// Security-aware error handling utilities
pub mod security {
    use super::*;
    
    /// Check if an error should trigger security alerting
    pub fn requires_security_alert(error: &UnifiedCryptoError) -> bool {
        matches!(
            error,
            UnifiedCryptoError::PolicyViolation { .. }
                | UnifiedCryptoError::AccessDenied
                | UnifiedCryptoError::MemorySecurity
                | UnifiedCryptoError::TimingSecurity
        )
    }
    
    /// Check if an error should trigger audit logging
    pub fn requires_audit_logging(error: &UnifiedCryptoError) -> bool {
        match error {
            UnifiedCryptoError::AuditLogging { .. } => false, // Avoid recursion
            _ => true, // All other errors should be audited
        }
    }
    
    /// Sanitize error message for external display
    /// 
    /// Removes potentially sensitive information from error messages
    /// before displaying to users or logging to external systems.
    pub fn sanitize_error_message(error: &UnifiedCryptoError) -> String {
        match error {
            UnifiedCryptoError::Configuration { .. } => {
                "Configuration error occurred".to_string()
            }
            UnifiedCryptoError::KeyManagement { .. } => {
                "Key management operation failed".to_string()
            }
            UnifiedCryptoError::CryptographicOperation { .. } => {
                "Cryptographic operation failed".to_string()
            }
            UnifiedCryptoError::InvalidInput { .. } => {
                "Invalid input provided".to_string()
            }
            UnifiedCryptoError::AccessDenied => {
                "Access denied".to_string()
            }
            _ => "Cryptographic error occurred".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context_creation() {
        let ctx = ErrorContext::crypto_operation("test_op", ErrorSeverity::Medium);
        assert_eq!(ctx.operation, "test_op");
        assert_eq!(ctx.component, "unified_crypto");
        assert_eq!(ctx.severity, ErrorSeverity::Medium);
    }

    #[test]
    fn test_error_severity_display() {
        assert_eq!(format!("{}", ErrorSeverity::Low), "LOW");
        assert_eq!(format!("{}", ErrorSeverity::Critical), "CRITICAL");
    }


    #[test]
    fn test_security_alert_detection() {
        let error = UnifiedCryptoError::AccessDenied;
        assert!(security::requires_security_alert(&error));
        
        let normal_error = UnifiedCryptoError::Configuration {
            message: "test".to_string(),
        };
        assert!(!security::requires_security_alert(&normal_error));
    }

    #[test]
    fn test_error_message_sanitization() {
        let error = UnifiedCryptoError::KeyManagement {
            operation: "sensitive key operation with private data".to_string(),
        };
        
        let sanitized = security::sanitize_error_message(&error);
        assert_eq!(sanitized, "Key management operation failed");
        assert!(!sanitized.contains("sensitive"));
        assert!(!sanitized.contains("private"));
    }
}