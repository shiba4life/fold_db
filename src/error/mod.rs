//! Unified error handling module
//!
//! This module consolidates error types, error handling utilities, and error creation patterns
//! that were previously split between `error.rs` and `error_handling/`.

use crate::schema::types::SchemaError;
use std::fmt;
use std::io;

// Re-export all error handling utilities
pub mod error_factory;
pub mod iterator_utils;
pub mod parser_utils;
pub mod regex_utils;
pub mod string_utils;

// Main error types (previously in error.rs)
#[derive(Debug)]
pub enum FoldDbError {
    Schema(SchemaError),
    Io(io::Error),
    Serde(serde_json::Error),
    Sled(sled::Error),
    Network(NetworkErrorKind),
    InvalidData(String),
    InvalidField(String),
    FieldNotFound(String),
    SchemaNotFound(String),
    InvalidTransform(String),
    SerializationError(String),
    DeserializationError(String),
    ConfigurationError(String),
    DatabaseError(String),
    NetworkError(String),
    PermissionDenied(String),
    PaymentRequired(String),
    // Additional variants needed by the codebase
    Config(String),
    Permission(String),
    Validation(String),
    Timeout(String),
    Protocol(String),
    Database(String),
}

impl std::error::Error for FoldDbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FoldDbError::Schema(err) => Some(err),
            FoldDbError::Io(err) => Some(err),
            FoldDbError::Serde(err) => Some(err),
            FoldDbError::Sled(err) => Some(err),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum NetworkErrorKind {
    ConnectionFailed(String),
    Timeout(String),
    InvalidMessage(String),
    PeerNotFound(String),
    ProtocolError(String),
    SecurityError(String),
    ConfigurationError(String),
    ResourceExhausted(String),
    InternalError(String),
    NetworkError(crate::network::NetworkError),
    Protocol(String),
    Connection(String),
}

impl fmt::Display for NetworkErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkErrorKind::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            NetworkErrorKind::Timeout(msg) => write!(f, "Network timeout: {}", msg),
            NetworkErrorKind::InvalidMessage(msg) => write!(f, "Invalid message: {}", msg),
            NetworkErrorKind::PeerNotFound(msg) => write!(f, "Peer not found: {}", msg),
            NetworkErrorKind::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
            NetworkErrorKind::SecurityError(msg) => write!(f, "Security error: {}", msg),
            NetworkErrorKind::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            NetworkErrorKind::ResourceExhausted(msg) => write!(f, "Resource exhausted: {}", msg),
            NetworkErrorKind::InternalError(msg) => write!(f, "Internal error: {}", msg),
            NetworkErrorKind::NetworkError(err) => write!(f, "Network error: {}", err),
            NetworkErrorKind::Protocol(msg) => write!(f, "Protocol error: {}", msg),
            NetworkErrorKind::Connection(msg) => write!(f, "Connection error: {}", msg),
        }
    }
}

impl fmt::Display for FoldDbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FoldDbError::Schema(err) => write!(f, "Schema error: {}", err),
            FoldDbError::Io(err) => write!(f, "IO error: {}", err),
            FoldDbError::Serde(err) => write!(f, "Serialization error: {}", err),
            FoldDbError::Sled(err) => write!(f, "Database error: {}", err),
            FoldDbError::Network(err) => write!(f, "Network error: {}", err),
            FoldDbError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            FoldDbError::InvalidField(msg) => write!(f, "Invalid field: {}", msg),
            FoldDbError::FieldNotFound(msg) => write!(f, "Field not found: {}", msg),
            FoldDbError::SchemaNotFound(msg) => write!(f, "Schema not found: {}", msg),
            FoldDbError::InvalidTransform(msg) => write!(f, "Invalid transform: {}", msg),
            FoldDbError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            FoldDbError::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
            FoldDbError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            FoldDbError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            FoldDbError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            FoldDbError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            FoldDbError::PaymentRequired(msg) => write!(f, "Payment required: {}", msg),
            FoldDbError::Config(msg) => write!(f, "Configuration error: {}", msg),
            FoldDbError::Permission(msg) => write!(f, "Permission error: {}", msg),
            FoldDbError::Validation(msg) => write!(f, "Validation error: {}", msg),
            FoldDbError::Timeout(msg) => write!(f, "Timeout error: {}", msg),
            FoldDbError::Protocol(msg) => write!(f, "Protocol error: {}", msg),
            FoldDbError::Database(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

// Error conversions
impl From<SchemaError> for FoldDbError {
    fn from(error: SchemaError) -> Self {
        FoldDbError::Schema(error)
    }
}

impl From<io::Error> for FoldDbError {
    fn from(error: io::Error) -> Self {
        FoldDbError::Io(error)
    }
}

impl From<serde_json::Error> for FoldDbError {
    fn from(error: serde_json::Error) -> Self {
        FoldDbError::Serde(error)
    }
}

impl From<sled::Error> for FoldDbError {
    fn from(error: sled::Error) -> Self {
        FoldDbError::Sled(error)
    }
}

impl From<crate::network::NetworkError> for NetworkErrorKind {
    fn from(error: crate::network::NetworkError) -> Self {
        match error {
            crate::network::NetworkError::ConnectionError(msg) => NetworkErrorKind::ConnectionFailed(msg),
            crate::network::NetworkError::Timeout(msg) => NetworkErrorKind::Timeout(msg),
            crate::network::NetworkError::TimeoutError => NetworkErrorKind::Timeout("Network timeout".to_string()),
            crate::network::NetworkError::InvalidPeerId(msg) => NetworkErrorKind::PeerNotFound(msg),
            crate::network::NetworkError::ProtocolError(msg) => NetworkErrorKind::ProtocolError(msg),
            crate::network::NetworkError::ConfigurationError(msg) => NetworkErrorKind::ConfigurationError(msg),
            crate::network::NetworkError::RequestFailed(msg) => NetworkErrorKind::InternalError(msg),
            crate::network::NetworkError::RemoteError(msg) => NetworkErrorKind::InternalError(msg),
            crate::network::NetworkError::Libp2pError(msg) => NetworkErrorKind::InternalError(msg),
            crate::network::NetworkError::OperationNotFound(msg) => NetworkErrorKind::InternalError(msg),
            crate::network::NetworkError::ConflictDetected(msg) => NetworkErrorKind::InternalError(msg),
        }
    }
}

impl From<&str> for NetworkErrorKind {
    fn from(msg: &str) -> Self {
        NetworkErrorKind::InternalError(msg.to_string())
    }
}

// Error handling utilities (previously in error_handling/mod.rs)

/// Utility trait for safe unwrapping with context
pub trait SafeUnwrap<T> {
    /// Safely unwrap with a custom error message
    fn safe_unwrap(self, context: &str) -> Result<T, SchemaError>;
}

impl<T> SafeUnwrap<T> for Option<T> {
    fn safe_unwrap(self, context: &str) -> Result<T, SchemaError> {
        self.ok_or_else(|| SchemaError::InvalidData(format!("Unexpected None: {}", context)))
    }
}

impl<T, E: std::fmt::Display> SafeUnwrap<T> for Result<T, E> {
    fn safe_unwrap(self, context: &str) -> Result<T, SchemaError> {
        self.map_err(|e| SchemaError::InvalidData(format!("{}: {}", context, e)))
    }
}

/// Utility for safe iterator operations
pub struct SafeIterator;

impl SafeIterator {
    /// Safely get the next item from an iterator with context
    pub fn next_with_context<T, I>(iter: &mut I, context: &str) -> Result<T, SchemaError>
    where
        I: Iterator<Item = T>,
    {
        iter.next()
            .ok_or_else(|| SchemaError::InvalidData(format!("Iterator exhausted: {}", context)))
    }

    /// Safely get the first item from an iterator with context
    pub fn first_with_context<T, I>(mut iter: I, context: &str) -> Result<T, SchemaError>
    where
        I: Iterator<Item = T>,
    {
        iter.next()
            .ok_or_else(|| SchemaError::InvalidData(format!("Empty iterator: {}", context)))
    }
}

/// Utility for safe string operations
pub struct SafeString;

impl SafeString {
    /// Safely get the first character of a string
    pub fn first_char(s: &str, context: &str) -> Result<char, SchemaError> {
        s.chars()
            .next()
            .ok_or_else(|| SchemaError::InvalidData(format!("Empty string: {}", context)))
    }

    /// Safely check if string starts with numeric character
    pub fn starts_with_numeric(s: &str) -> bool {
        s.chars().next().is_some_and(|c| c.is_numeric())
    }
}

// Type aliases for convenience
pub type FoldDbResult<T> = Result<T, FoldDbError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_unwrap_option() {
        let some_val: Option<i32> = Some(42);
        let none_val: Option<i32> = None;

        assert_eq!(some_val.safe_unwrap("test context").unwrap(), 42);
        assert!(none_val.safe_unwrap("test context").is_err());
    }

    #[test]
    fn test_safe_iterator() {
        let mut iter = vec![1, 2, 3].into_iter();
        assert_eq!(
            SafeIterator::next_with_context(&mut iter, "test").unwrap(),
            1
        );

        let empty_iter = std::iter::empty::<i32>();
        assert!(SafeIterator::first_with_context(empty_iter, "test").is_err());
    }

    #[test]
    fn test_safe_string() {
        assert_eq!(SafeString::first_char("hello", "test").unwrap(), 'h');
        assert!(SafeString::first_char("", "test").is_err());
        assert!(SafeString::starts_with_numeric("123abc"));
        assert!(!SafeString::starts_with_numeric("abc123"));
    }
}