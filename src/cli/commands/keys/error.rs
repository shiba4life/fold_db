//! Error types and error handling specific to key management
//! 
//! This module provides specialized error types for key management operations
//! including storage, retrieval, encryption, decryption, and validation errors.

use std::fmt;

/// Key management specific error types
#[derive(Debug)]
pub enum KeyError {
    /// Invalid key format or content
    InvalidKey(String),
    /// Key not found in storage
    KeyNotFound(String),
    /// Key already exists (for operations requiring uniqueness)
    KeyExists(String),
    /// Encryption/decryption failures
    CryptographicError(String),
    /// Storage-related errors (file I/O, permissions)
    StorageError(String),
    /// Invalid passphrase or authentication failure
    AuthenticationError(String),
    /// Configuration or parameter errors
    ConfigurationError(String),
    /// Backup/restore operation errors
    BackupError(String),
    /// Import/export operation errors
    ImportExportError(String),
    /// Key rotation errors
    RotationError(String),
    /// Validation or integrity check failures
    ValidationError(String),
}

impl fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyError::InvalidKey(msg) => write!(f, "Invalid key: {}", msg),
            KeyError::KeyNotFound(msg) => write!(f, "Key not found: {}", msg),
            KeyError::KeyExists(msg) => write!(f, "Key already exists: {}", msg),
            KeyError::CryptographicError(msg) => write!(f, "Cryptographic error: {}", msg),
            KeyError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            KeyError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            KeyError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            KeyError::BackupError(msg) => write!(f, "Backup error: {}", msg),
            KeyError::ImportExportError(msg) => write!(f, "Import/export error: {}", msg),
            KeyError::RotationError(msg) => write!(f, "Rotation error: {}", msg),
            KeyError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for KeyError {}

/// Result type for key management operations
pub type KeyResult<T> = Result<T, KeyError>;

/// Convert generic errors to KeyError types
impl From<std::io::Error> for KeyError {
    fn from(err: std::io::Error) -> Self {
        KeyError::StorageError(format!("I/O error: {}", err))
    }
}

impl From<serde_json::Error> for KeyError {
    fn from(err: serde_json::Error) -> Self {
        KeyError::StorageError(format!("JSON serialization error: {}", err))
    }
}
