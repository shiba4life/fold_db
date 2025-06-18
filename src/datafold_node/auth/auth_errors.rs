//! Authentication-specific error types and handling for DataFold node
//!
//! This module consolidates all authentication error types, error handling patterns,
//! and error conversion logic to provide comprehensive error management for the
//! authentication system.

use crate::security_types::Severity;
use actix_web::http::StatusCode;
use serde::{Deserialize, Serialize};

/// Enhanced error types for authentication failures with detailed categorization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthenticationError {
    /// Missing required headers
    MissingHeaders {
        missing: Vec<String>,
        correlation_id: String,
    },
    /// Invalid signature format or parsing errors
    InvalidSignatureFormat {
        reason: String,
        correlation_id: String,
    },
    /// Signature verification failures
    SignatureVerificationFailed {
        key_id: String,
        correlation_id: String,
    },
    /// Timestamp validation failures
    TimestampValidationFailed {
        timestamp: u64,
        current_time: u64,
        reason: String,
        correlation_id: String,
    },
    /// Nonce validation failures (replay attempts)
    NonceValidationFailed {
        nonce: String,
        reason: String,
        correlation_id: String,
    },
    /// Public key lookup failures
    PublicKeyLookupFailed {
        key_id: String,
        correlation_id: String,
    },
    /// Configuration errors
    ConfigurationError {
        reason: String,
        correlation_id: String,
    },
    /// Algorithm not supported
    UnsupportedAlgorithm {
        algorithm: String,
        correlation_id: String,
    },
    /// Rate limiting triggered
    RateLimitExceeded {
        client_id: String,
        correlation_id: String,
    },
}

impl AuthenticationError {
    pub fn correlation_id(&self) -> &str {
        match self {
            Self::MissingHeaders { correlation_id, .. } => correlation_id,
            Self::InvalidSignatureFormat { correlation_id, .. } => correlation_id,
            Self::SignatureVerificationFailed { correlation_id, .. } => correlation_id,
            Self::TimestampValidationFailed { correlation_id, .. } => correlation_id,
            Self::NonceValidationFailed { correlation_id, .. } => correlation_id,
            Self::PublicKeyLookupFailed { correlation_id, .. } => correlation_id,
            Self::ConfigurationError { correlation_id, .. } => correlation_id,
            Self::UnsupportedAlgorithm { correlation_id, .. } => correlation_id,
            Self::RateLimitExceeded { correlation_id, .. } => correlation_id,
        }
    }

    pub fn http_status_code(&self) -> StatusCode {
        match self {
            Self::MissingHeaders { .. } => StatusCode::BAD_REQUEST,
            Self::InvalidSignatureFormat { .. } => StatusCode::BAD_REQUEST,
            Self::SignatureVerificationFailed { .. } => StatusCode::UNAUTHORIZED,
            Self::TimestampValidationFailed { .. } => StatusCode::UNAUTHORIZED,
            Self::NonceValidationFailed { .. } => StatusCode::UNAUTHORIZED,
            Self::PublicKeyLookupFailed { .. } => StatusCode::UNAUTHORIZED,
            Self::ConfigurationError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UnsupportedAlgorithm { .. } => StatusCode::BAD_REQUEST,
            Self::RateLimitExceeded { .. } => StatusCode::TOO_MANY_REQUESTS,
        }
    }

    /// Get error code for programmatic error handling
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::MissingHeaders { .. } => "MISSING_HEADERS",
            Self::InvalidSignatureFormat { .. } => "INVALID_SIGNATURE_FORMAT",
            Self::SignatureVerificationFailed { .. } => "SIGNATURE_VERIFICATION_FAILED",
            Self::TimestampValidationFailed { .. } => "TIMESTAMP_VALIDATION_FAILED",
            Self::NonceValidationFailed { .. } => "NONCE_VALIDATION_FAILED",
            Self::PublicKeyLookupFailed { .. } => "PUBLIC_KEY_LOOKUP_FAILED",
            Self::ConfigurationError { .. } => "CONFIGURATION_ERROR",
            Self::UnsupportedAlgorithm { .. } => "UNSUPPORTED_ALGORITHM",
            Self::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
        }
    }

    pub fn public_message(&self) -> String {
        match self {
            Self::MissingHeaders { .. } => "Missing required authentication headers. Please include Signature-Input and Signature headers.".to_string(),
            Self::InvalidSignatureFormat { .. } => "Invalid signature format. Please verify your signature encoding and header format.".to_string(),
            Self::SignatureVerificationFailed { .. } => "Signature verification failed. Please check your signature calculation and key.".to_string(),
            Self::TimestampValidationFailed { .. } => "Request timestamp invalid. Please ensure timestamp is within allowed time window.".to_string(),
            Self::NonceValidationFailed { .. } => "Request validation failed. Please use a unique nonce for each request.".to_string(),
            Self::PublicKeyLookupFailed { .. } => "Authentication failed. Please verify your key ID is registered.".to_string(),
            Self::ConfigurationError { .. } => "Internal server error. Please contact system administrator.".to_string(),
            Self::UnsupportedAlgorithm { .. } => "Unsupported signature algorithm. Please use ed25519.".to_string(),
            Self::RateLimitExceeded { .. } => "Rate limit exceeded. Please reduce request frequency and try again later.".to_string(),
        }
    }

    /// Get user-friendly error message with troubleshooting guidance
    pub fn user_friendly_message(&self, environment: &str) -> String {
        let base_message = self.public_message();
        let troubleshooting = self.get_troubleshooting_guidance();
        let docs_link = self.get_documentation_link();

        if environment == "development" {
            format!(
                "{}\n\nTroubleshooting:\n{}\n\nFor more help: {}",
                base_message, troubleshooting, docs_link
            )
        } else {
            format!(
                "{}. For assistance, reference error ID: {} or visit: {}",
                base_message,
                self.correlation_id(),
                docs_link
            )
        }
    }

    /// Get specific troubleshooting guidance for each error type
    pub fn get_troubleshooting_guidance(&self) -> String {
        match self {
            Self::MissingHeaders { missing, .. } => {
                format!(
                    "Missing headers: {}. \
                • Ensure your HTTP client includes both 'Signature-Input' and 'Signature' headers\n\
                • Verify header names are lowercase\n\
                • Check your signature library configuration",
                    missing.join(", ")
                )
            }
            Self::InvalidSignatureFormat { reason, .. } => {
                format!("Signature format error: {}\n\
                • Verify signature is hex-encoded\n\
                • Check signature-input format: sig1=(\"@method\" \"@target-uri\");created=timestamp;keyid=\"your-key\";alg=\"ed25519\";nonce=\"unique-nonce\"\n\
                • Ensure all required parameters are present: created, keyid, alg, nonce", reason)
            }
            Self::SignatureVerificationFailed { key_id, .. } => {
                format!(
                    "Signature verification failed for key: {}\n\
                • Verify the canonical message construction matches the server\n\
                • Check that covered components match exactly\n\
                • Ensure private key corresponds to registered public key\n\
                • Verify Ed25519 signature calculation",
                    key_id
                )
            }
            Self::TimestampValidationFailed {
                timestamp,
                current_time,
                reason,
                ..
            } => {
                format!(
                    "Timestamp error: {} (request: {}, server: {})\n\
                • Check system clock synchronization (NTP)\n\
                • Ensure timestamp is Unix epoch seconds\n\
                • Verify timestamp is not too old or too far in future\n\
                • Allow for reasonable clock skew",
                    reason, timestamp, current_time
                )
            }
            Self::NonceValidationFailed { nonce, reason, .. } => {
                format!(
                    "Nonce validation failed for '{}': {}\n\
                • Use a unique nonce for every request\n\
                • Ensure nonce follows required format (UUID4 if configured)\n\
                • Check nonce length restrictions\n\
                • Verify nonce contains only allowed characters",
                    nonce, reason
                )
            }
            Self::PublicKeyLookupFailed { key_id, .. } => {
                format!(
                    "Key lookup failed for: {}\n\
                • Verify the key ID is correctly registered\n\
                • Check key registration status (must be 'active')\n\
                • Ensure key ID matches exactly (case-sensitive)\n\
                • Contact administrator if key should be registered",
                    key_id
                )
            }
            Self::ConfigurationError { reason, .. } => {
                format!(
                    "Server configuration error: {}\n\
                • This is a server-side issue\n\
                • Contact system administrator\n\
                • Provide correlation ID for debugging",
                    reason
                )
            }
            Self::UnsupportedAlgorithm { algorithm, .. } => {
                format!(
                    "Unsupported algorithm: {}\n\
                • Use 'ed25519' as the signature algorithm\n\
                • Update signature-input header: alg=\"ed25519\"\n\
                • Verify signature library supports Ed25519",
                    algorithm
                )
            }
            Self::RateLimitExceeded { client_id, .. } => {
                format!(
                    "Rate limit exceeded for client: {}\n\
                • Reduce request frequency\n\
                • Implement exponential backoff retry logic\n\
                • Contact administrator if limits seem too restrictive\n\
                • Check for duplicate or unnecessary requests",
                    client_id
                )
            }
        }
    }

    /// Get documentation link for specific error type
    pub fn get_documentation_link(&self) -> String {
        match self {
            Self::MissingHeaders { .. } | Self::InvalidSignatureFormat { .. } => {
                "https://docs.datafold.dev/signature-auth/setup"
            }
            Self::SignatureVerificationFailed { .. } => {
                "https://docs.datafold.dev/signature-auth/troubleshooting#signature-verification"
            }
            Self::TimestampValidationFailed { .. } => {
                "https://docs.datafold.dev/signature-auth/troubleshooting#timestamp-issues"
            }
            Self::NonceValidationFailed { .. } => {
                "https://docs.datafold.dev/signature-auth/troubleshooting#nonce-validation"
            }
            Self::PublicKeyLookupFailed { .. } => {
                "https://docs.datafold.dev/signature-auth/key-management"
            }
            Self::ConfigurationError { .. } => {
                "https://docs.datafold.dev/signature-auth/server-config"
            }
            Self::UnsupportedAlgorithm { .. } => {
                "https://docs.datafold.dev/signature-auth/algorithms"
            }
            Self::RateLimitExceeded { .. } => {
                "https://docs.datafold.dev/signature-auth/rate-limits"
            }
        }
        .to_string()
    }

    /// Get suggested next steps for resolving the error
    pub fn get_suggested_actions(&self) -> Vec<String> {
        match self {
            Self::MissingHeaders { .. } => vec![
                "Include both Signature-Input and Signature headers in your request".to_string(),
                "Verify your HTTP client configuration".to_string(),
                "Check signature library documentation".to_string(),
            ],
            Self::InvalidSignatureFormat { .. } => vec![
                "Validate signature-input header format".to_string(),
                "Verify signature is properly hex-encoded".to_string(),
                "Test with signature validation endpoint".to_string(),
            ],
            Self::SignatureVerificationFailed { .. } => vec![
                "Test signature generation with validation endpoint".to_string(),
                "Verify private/public key pair consistency".to_string(),
                "Check canonical message construction".to_string(),
            ],
            Self::TimestampValidationFailed { .. } => vec![
                "Synchronize system clock with NTP".to_string(),
                "Check server time window configuration".to_string(),
                "Use current Unix timestamp".to_string(),
            ],
            Self::NonceValidationFailed { .. } => vec![
                "Generate a new unique nonce for each request".to_string(),
                "Verify nonce format requirements".to_string(),
                "Check nonce length and character restrictions".to_string(),
            ],
            Self::PublicKeyLookupFailed { .. } => vec![
                "Register your public key with the server".to_string(),
                "Verify key ID matches registration".to_string(),
                "Check key status is 'active'".to_string(),
            ],
            Self::ConfigurationError { .. } => vec![
                "Contact system administrator".to_string(),
                "Provide correlation ID for support".to_string(),
                "Check server logs if accessible".to_string(),
            ],
            Self::UnsupportedAlgorithm { .. } => vec![
                "Change algorithm to 'ed25519'".to_string(),
                "Update signature library configuration".to_string(),
                "Regenerate signatures with Ed25519".to_string(),
            ],
            Self::RateLimitExceeded { .. } => vec![
                "Implement request throttling".to_string(),
                "Use exponential backoff retry logic".to_string(),
                "Review and optimize request patterns".to_string(),
            ],
        }
    }

    pub fn severity(&self) -> Severity {
        match self {
            Self::MissingHeaders { .. } => Severity::Info,
            Self::InvalidSignatureFormat { .. } => Severity::Warning,
            Self::SignatureVerificationFailed { .. } => Severity::Warning,
            Self::TimestampValidationFailed { .. } => Severity::Warning,
            Self::NonceValidationFailed { .. } => Severity::Critical,
            Self::PublicKeyLookupFailed { .. } => Severity::Warning,
            Self::ConfigurationError { .. } => Severity::Critical,
            Self::UnsupportedAlgorithm { .. } => Severity::Info,
            Self::RateLimitExceeded { .. } => Severity::Critical,
        }
    }
}

impl std::fmt::Display for AuthenticationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingHeaders {
                missing,
                correlation_id,
            } => {
                write!(
                    f,
                    "Missing required headers: {} (correlation_id: {})",
                    missing.join(", "),
                    correlation_id
                )
            }
            Self::InvalidSignatureFormat {
                reason,
                correlation_id,
            } => {
                write!(
                    f,
                    "Invalid signature format: {} (correlation_id: {})",
                    reason, correlation_id
                )
            }
            Self::SignatureVerificationFailed {
                key_id,
                correlation_id,
            } => {
                write!(
                    f,
                    "Signature verification failed for key_id: {} (correlation_id: {})",
                    key_id, correlation_id
                )
            }
            Self::TimestampValidationFailed {
                timestamp,
                current_time,
                reason,
                correlation_id,
            } => {
                write!(
                    f,
                    "Timestamp validation failed: {} at {} (current: {}) (correlation_id: {})",
                    reason, timestamp, current_time, correlation_id
                )
            }
            Self::NonceValidationFailed {
                nonce,
                reason,
                correlation_id,
            } => {
                write!(
                    f,
                    "Nonce validation failed for {}: {} (correlation_id: {})",
                    nonce, reason, correlation_id
                )
            }
            Self::PublicKeyLookupFailed {
                key_id,
                correlation_id,
            } => {
                write!(
                    f,
                    "Public key lookup failed for key_id: {} (correlation_id: {})",
                    key_id, correlation_id
                )
            }
            Self::ConfigurationError {
                reason,
                correlation_id,
            } => {
                write!(
                    f,
                    "Configuration error: {} (correlation_id: {})",
                    reason, correlation_id
                )
            }
            Self::UnsupportedAlgorithm {
                algorithm,
                correlation_id,
            } => {
                write!(
                    f,
                    "Unsupported algorithm: {} (correlation_id: {})",
                    algorithm, correlation_id
                )
            }
            Self::RateLimitExceeded {
                client_id,
                correlation_id,
            } => {
                write!(
                    f,
                    "Rate limit exceeded for client: {} (correlation_id: {})",
                    client_id, correlation_id
                )
            }
        }
    }
}

impl std::error::Error for AuthenticationError {}

/// Standardized error response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: bool,
    pub error_code: String,
    pub message: String,
    pub correlation_id: Option<String>,
    pub timestamp: u64,
    pub details: Option<ErrorDetails>,
}

/// Detailed error information for development environments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub error_type: String,
    pub troubleshooting: String,
    pub suggested_actions: Vec<String>,
    pub documentation_link: String,
}

/// Custom error type for authentication failures that implements ResponseError
#[derive(Debug)]
pub struct CustomAuthError {
    pub auth_error: AuthenticationError,
    pub error_message: String,
}

impl std::fmt::Display for CustomAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_message)
    }
}

impl actix_web::ResponseError for CustomAuthError {
    fn status_code(&self) -> StatusCode {
        self.auth_error.http_status_code()
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        let status = self.status_code();
        match status {
            StatusCode::BAD_REQUEST => actix_web::HttpResponse::BadRequest(),
            StatusCode::UNAUTHORIZED => actix_web::HttpResponse::Unauthorized(),
            StatusCode::TOO_MANY_REQUESTS => actix_web::HttpResponse::TooManyRequests(),
            StatusCode::INTERNAL_SERVER_ERROR => actix_web::HttpResponse::InternalServerError(),
            _ => actix_web::HttpResponse::Unauthorized(),
        }
        .json(serde_json::json!({
            "error": true,
            "error_code": self.auth_error.error_code(),
            "message": self.error_message,
            "correlation_id": self.auth_error.correlation_id()
        }))
    }
}