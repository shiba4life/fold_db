//! Core verification types and errors

use crate::cli::auth::CliAuthError;
use crate::error::FoldDbError;
use std::fmt;

/// Context for verification requests
#[derive(Debug)]
pub struct VerificationRequestContext<'a> {
    #[allow(dead_code)]
    pub url: &'a str,
    #[allow(dead_code)]
    pub method: &'a str,
    pub body: Option<&'a [u8]>,
}

/// Errors that can occur during signature verification
#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("Signature format error: {0}")]
    SignatureFormat(String),

    #[error("Missing signature component: {0}")]
    MissingComponent(String),

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Timestamp verification failed: {0}")]
    TimestampVerification(String),

    #[error("Content digest mismatch: {0}")]
    ContentDigestMismatch(String),

    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    #[error("Key management error: {0}")]
    KeyManagement(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("CLI auth error: {0}")]
    CliAuth(#[from] CliAuthError),

    #[error("FoldDb error: {0}")]
    FoldDb(#[from] FoldDbError),
}

/// Result type for verification operations
pub type VerificationResult<T> = Result<T, VerificationError>;

/// Verification status for signature validation
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum VerificationStatus {
    /// Signature is valid
    Valid,
    /// Signature is invalid
    Invalid,
    /// Verification status unknown (insufficient information)
    Unknown,
    /// Error occurred during verification
    Error,
}

impl fmt::Display for VerificationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Valid => write!(f, "VALID"),
            Self::Invalid => write!(f, "INVALID"),
            Self::Unknown => write!(f, "UNKNOWN"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}