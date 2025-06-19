//! Error handling for the unified transform execution system.
//!
//! This module provides comprehensive error types and handling for all transform
//! operations, including categorization, recovery strategies, and detailed
//! error context for debugging and monitoring.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::SystemTime;
use thiserror::Error;

/// Result type for transform operations.
pub type TransformResult<T> = Result<T, TransformError>;

/// Comprehensive error type for transform execution operations.
#[derive(Debug, Error, Clone)]
pub enum TransformError {
    /// Transform validation errors
    #[error("Transform validation failed: {message}")]
    ValidationError {
        message: String,
        field: Option<String>,
        details: Option<String>,
    },

    /// Transform execution errors
    #[error("Transform execution failed: {message}")]
    ExecutionError {
        message: String,
        transform_id: String,
        input_data: Option<String>,
        stack_trace: Option<String>,
    },

    /// Transform registration errors
    #[error("Transform registration failed: {message}")]
    RegistrationError {
        message: String,
        transform_id: Option<String>,
        conflict_reason: Option<String>,
    },

    /// Transform not found errors
    #[error("Transform not found: {transform_id}")]
    NotFoundError {
        transform_id: String,
        operation: String,
    },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    ConfigurationError {
        message: String,
        config_key: Option<String>,
        suggested_fix: Option<String>,
    },

    /// State management errors
    #[error("State management error: {message}")]
    StateError {
        message: String,
        transform_id: Option<String>,
        operation: String,
    },

    /// Queue management errors
    #[error("Queue operation failed: {message}")]
    QueueError {
        message: String,
        queue_operation: String,
        queue_size: Option<usize>,
    },

    /// Database operation errors
    #[error("Database operation failed: {message}")]
    DatabaseError {
        message: String,
        operation: String,
        table: Option<String>,
    },

    /// Schema-related errors
    #[error("Schema error: {message}")]
    SchemaError {
        message: String,
    },

    /// Serialization/deserialization errors
    #[error("Serialization error: {message}")]
    SerializationError {
        message: String,
        data_type: String,
    },

    /// Permission and security errors
    #[error("Permission denied: {message}")]
    PermissionError {
        message: String,
        required_permission: String,
        operation: String,
    },

    /// Resource exhaustion errors
    #[error("Resource exhausted: {message}")]
    ResourceError {
        message: String,
        resource_type: String,
        current_usage: Option<f64>,
        limit: Option<f64>,
    },

    /// Timeout errors
    #[error("Operation timed out: {message}")]
    TimeoutError {
        message: String,
        operation: String,
        timeout_duration: std::time::Duration,
    },

    /// Dependency errors
    #[error("Dependency error: {message}")]
    DependencyError {
        message: String,
        dependency_name: String,
        dependency_version: Option<String>,
    },

    /// General internal errors
    #[error("Internal error: {message}")]
    InternalError {
        message: String,
        error_code: Option<String>,
        context: Option<String>,
    },
}

impl TransformError {
    /// Creates a validation error.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::ValidationError {
            message: message.into(),
            field: None,
            details: None,
        }
    }

    /// Creates a validation error with field information.
    pub fn validation_with_field(
        message: impl Into<String>,
        field: impl Into<String>,
    ) -> Self {
        Self::ValidationError {
            message: message.into(),
            field: Some(field.into()),
            details: None,
        }
    }

    /// Creates an execution error.
    pub fn execution(message: impl Into<String>, transform_id: impl Into<String>) -> Self {
        Self::ExecutionError {
            message: message.into(),
            transform_id: transform_id.into(),
            input_data: None,
            stack_trace: None,
        }
    }

    /// Creates a registration error.
    pub fn registration(message: impl Into<String>) -> Self {
        Self::RegistrationError {
            message: message.into(),
            transform_id: None,
            conflict_reason: None,
        }
    }

    /// Creates a not found error.
    pub fn not_found(transform_id: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::NotFoundError {
            transform_id: transform_id.into(),
            operation: operation.into(),
        }
    }

    /// Creates a configuration error.
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::ConfigurationError {
            message: message.into(),
            config_key: None,
            suggested_fix: None,
        }
    }

    /// Creates a state error.
    pub fn state(message: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::StateError {
            message: message.into(),
            transform_id: None,
            operation: operation.into(),
        }
    }

    /// Creates a queue error.
    pub fn queue(message: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::QueueError {
            message: message.into(),
            queue_operation: operation.into(),
            queue_size: None,
        }
    }

    /// Creates a database error.
    pub fn database(message: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::DatabaseError {
            message: message.into(),
            operation: operation.into(),
            table: None,
        }
    }

    /// Creates a serialization error.
    pub fn serialization(message: impl Into<String>, data_type: impl Into<String>) -> Self {
        Self::SerializationError {
            message: message.into(),
            data_type: data_type.into(),
        }
    }

    /// Creates a permission error.
    pub fn permission(
        message: impl Into<String>,
        required_permission: impl Into<String>,
        operation: impl Into<String>,
    ) -> Self {
        Self::PermissionError {
            message: message.into(),
            required_permission: required_permission.into(),
            operation: operation.into(),
        }
    }

    /// Creates a timeout error.
    pub fn timeout(
        message: impl Into<String>,
        operation: impl Into<String>,
        timeout_duration: std::time::Duration,
    ) -> Self {
        Self::TimeoutError {
            message: message.into(),
            operation: operation.into(),
            timeout_duration,
        }
    }

    /// Creates an internal error.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::InternalError {
            message: message.into(),
            error_code: None,
            context: None,
        }
    }

    /// Gets the error severity level.
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            TransformError::ValidationError { .. } => ErrorSeverity::Warning,
            TransformError::ExecutionError { .. } => ErrorSeverity::Error,
            TransformError::RegistrationError { .. } => ErrorSeverity::Warning,
            TransformError::NotFoundError { .. } => ErrorSeverity::Warning,
            TransformError::ConfigurationError { .. } => ErrorSeverity::Error,
            TransformError::StateError { .. } => ErrorSeverity::Error,
            TransformError::QueueError { .. } => ErrorSeverity::Warning,
            TransformError::DatabaseError { .. } => ErrorSeverity::Critical,
            TransformError::SchemaError { .. } => ErrorSeverity::Error,
            TransformError::SerializationError { .. } => ErrorSeverity::Warning,
            TransformError::PermissionError { .. } => ErrorSeverity::Error,
            TransformError::ResourceError { .. } => ErrorSeverity::Critical,
            TransformError::TimeoutError { .. } => ErrorSeverity::Warning,
            TransformError::DependencyError { .. } => ErrorSeverity::Error,
            TransformError::InternalError { .. } => ErrorSeverity::Critical,
        }
    }

    /// Gets the error category.
    pub fn category(&self) -> ErrorCategory {
        match self {
            TransformError::ValidationError { .. } => ErrorCategory::Validation,
            TransformError::ExecutionError { .. } => ErrorCategory::Execution,
            TransformError::RegistrationError { .. } => ErrorCategory::Registration,
            TransformError::NotFoundError { .. } => ErrorCategory::NotFound,
            TransformError::ConfigurationError { .. } => ErrorCategory::Configuration,
            TransformError::StateError { .. } => ErrorCategory::State,
            TransformError::QueueError { .. } => ErrorCategory::Queue,
            TransformError::DatabaseError { .. } => ErrorCategory::Database,
            TransformError::SchemaError { .. } => ErrorCategory::Schema,
            TransformError::SerializationError { .. } => ErrorCategory::Serialization,
            TransformError::PermissionError { .. } => ErrorCategory::Permission,
            TransformError::ResourceError { .. } => ErrorCategory::Resource,
            TransformError::TimeoutError { .. } => ErrorCategory::Timeout,
            TransformError::DependencyError { .. } => ErrorCategory::Dependency,
            TransformError::InternalError { .. } => ErrorCategory::Internal,
        }
    }

    /// Determines if the error is retryable.
    pub fn is_retryable(&self) -> bool {
        match self {
            TransformError::ValidationError { .. } => false,
            TransformError::ExecutionError { .. } => true,
            TransformError::RegistrationError { .. } => false,
            TransformError::NotFoundError { .. } => false,
            TransformError::ConfigurationError { .. } => false,
            TransformError::StateError { .. } => true,
            TransformError::QueueError { .. } => true,
            TransformError::DatabaseError { .. } => true,
            TransformError::SchemaError { .. } => false,
            TransformError::SerializationError { .. } => false,
            TransformError::PermissionError { .. } => false,
            TransformError::ResourceError { .. } => true,
            TransformError::TimeoutError { .. } => true,
            TransformError::DependencyError { .. } => true,
            TransformError::InternalError { .. } => true,
        }
    }
}

/// Error severity levels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Informational messages
    Info,
    /// Warning conditions
    Warning,
    /// Error conditions
    Error,
    /// Critical system errors
    Critical,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Info => write!(f, "INFO"),
            ErrorSeverity::Warning => write!(f, "WARNING"),
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Error categories for classification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Validation errors
    Validation,
    /// Execution errors
    Execution,
    /// Registration errors
    Registration,
    /// Resource not found
    NotFound,
    /// Configuration errors
    Configuration,
    /// State management errors
    State,
    /// Queue management errors
    Queue,
    /// Database errors
    Database,
    /// Schema errors
    Schema,
    /// Serialization errors
    Serialization,
    /// Permission errors
    Permission,
    /// Resource exhaustion
    Resource,
    /// Timeout errors
    Timeout,
    /// Dependency errors
    Dependency,
    /// Internal system errors
    Internal,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCategory::Validation => write!(f, "VALIDATION"),
            ErrorCategory::Execution => write!(f, "EXECUTION"),
            ErrorCategory::Registration => write!(f, "REGISTRATION"),
            ErrorCategory::NotFound => write!(f, "NOT_FOUND"),
            ErrorCategory::Configuration => write!(f, "CONFIGURATION"),
            ErrorCategory::State => write!(f, "STATE"),
            ErrorCategory::Queue => write!(f, "QUEUE"),
            ErrorCategory::Database => write!(f, "DATABASE"),
            ErrorCategory::Schema => write!(f, "SCHEMA"),
            ErrorCategory::Serialization => write!(f, "SERIALIZATION"),
            ErrorCategory::Permission => write!(f, "PERMISSION"),
            ErrorCategory::Resource => write!(f, "RESOURCE"),
            ErrorCategory::Timeout => write!(f, "TIMEOUT"),
            ErrorCategory::Dependency => write!(f, "DEPENDENCY"),
            ErrorCategory::Internal => write!(f, "INTERNAL"),
        }
    }
}

/// Error context for enhanced debugging and monitoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Error ID for tracking
    pub error_id: String,
    /// Timestamp when error occurred
    pub timestamp: SystemTime,
    /// Component where error occurred
    pub component: String,
    /// Operation being performed
    pub operation: String,
    /// Additional context data
    pub context_data: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    /// Creates a new error context.
    pub fn new(component: impl Into<String>, operation: impl Into<String>) -> Self {
        Self {
            error_id: uuid::Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            component: component.into(),
            operation: operation.into(),
            context_data: std::collections::HashMap::new(),
        }
    }

    /// Adds context data.
    pub fn with_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context_data.insert(key.into(), value.into());
        self
    }
}

/// Error action suggestions for recovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorAction {
    /// Retry the operation
    Retry,
    /// Skip the operation
    Skip,
    /// Abort the operation
    Abort,
    /// Escalate to administrator
    Escalate,
    /// Use fallback value
    Fallback,
}

/// Error handler for managing transform errors.
pub struct TransformErrorHandler {
    /// Error policies for different error types
    policies: std::collections::HashMap<ErrorCategory, ErrorAction>,
}

impl TransformErrorHandler {
    /// Creates a new error handler with default policies.
    pub fn new() -> Self {
        let mut policies = std::collections::HashMap::new();
        
        // Default error handling policies
        policies.insert(ErrorCategory::Validation, ErrorAction::Abort);
        policies.insert(ErrorCategory::Execution, ErrorAction::Retry);
        policies.insert(ErrorCategory::Registration, ErrorAction::Abort);
        policies.insert(ErrorCategory::NotFound, ErrorAction::Abort);
        policies.insert(ErrorCategory::Configuration, ErrorAction::Escalate);
        policies.insert(ErrorCategory::State, ErrorAction::Retry);
        policies.insert(ErrorCategory::Queue, ErrorAction::Retry);
        policies.insert(ErrorCategory::Database, ErrorAction::Escalate);
        policies.insert(ErrorCategory::Schema, ErrorAction::Abort);
        policies.insert(ErrorCategory::Serialization, ErrorAction::Abort);
        policies.insert(ErrorCategory::Permission, ErrorAction::Escalate);
        policies.insert(ErrorCategory::Resource, ErrorAction::Retry);
        policies.insert(ErrorCategory::Timeout, ErrorAction::Retry);
        policies.insert(ErrorCategory::Dependency, ErrorAction::Escalate);
        policies.insert(ErrorCategory::Internal, ErrorAction::Escalate);

        Self { policies }
    }

    /// Handles an error and returns the suggested action.
    pub fn handle(&self, error: &TransformError, context: ErrorContext) -> ErrorAction {
        let category = error.category();
        let action = self.policies.get(&category).unwrap_or(&ErrorAction::Escalate);
        
        // Log the error with context
        log::error!(
            "Transform error [{}] in {} during {}: {} (Action: {:?})",
            context.error_id,
            context.component,
            context.operation,
            error,
            action
        );

        action.clone()
    }

    /// Sets a custom error policy for a category.
    pub fn set_policy(&mut self, category: ErrorCategory, action: ErrorAction) {
        self.policies.insert(category, action);
    }

    /// Gets the current policy for a category.
    pub fn get_policy(&self, category: &ErrorCategory) -> Option<&ErrorAction> {
        self.policies.get(category)
    }
}

impl Default for TransformErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}
/// Centralized error conversion utilities to eliminate duplicate error handling patterns.
/// 
/// This module provides standardized error conversion functions to replace the 81+ duplicate
/// error handling patterns found throughout the codebase.
pub mod error_conversion {
    
    use crate::schema::types::errors::SchemaError;

    /// Converts any error to SchemaError::InvalidData with consistent formatting.
    pub fn to_schema_error<E: std::fmt::Display>(error: E, operation: &str) -> SchemaError {
        SchemaError::InvalidData(format!("{}: {}", operation, error))
    }

    /// Converts serialization errors with consistent messaging.
    pub fn serialization_error<E: std::fmt::Display>(error: E) -> SchemaError {
        SchemaError::InvalidData(format!("Failed to serialize item: {}", error))
    }

    /// Converts deserialization errors with consistent messaging.
    pub fn deserialization_error<E: std::fmt::Display>(error: E) -> SchemaError {
        SchemaError::InvalidData(format!("Failed to deserialize item: {}", error))
    }

    /// Converts database operation errors with consistent messaging.
    pub fn database_error<E: std::fmt::Display>(error: E, operation: &str) -> SchemaError {
        SchemaError::InvalidData(format!("Database {} failed: {}", operation, error))
    }

    /// Converts file operation errors with consistent messaging.
    pub fn file_error<E: std::fmt::Display>(error: E, file_path: &str, operation: &str) -> SchemaError {
        SchemaError::InvalidData(format!("Failed to {} file {}: {}", operation, file_path, error))
    }

    /// Converts JSON parsing errors with consistent messaging.
    pub fn json_error<E: std::fmt::Display>(error: E) -> SchemaError {
        SchemaError::InvalidData(format!("Invalid JSON: {}", error))
    }

    /// Converts encryption errors with consistent messaging.
    pub fn encryption_error<E: std::fmt::Display>(error: E) -> SchemaError {
        SchemaError::InvalidData(format!("Encryption failed: {}", error))
    }

    /// Converts decryption errors with consistent messaging.
    pub fn decryption_error<E: std::fmt::Display>(error: E) -> SchemaError {
        SchemaError::InvalidData(format!("Decryption failed: {}", error))
    }

    /// Converts async task errors with consistent messaging.
    pub fn async_task_error<E: std::fmt::Display>(error: E) -> SchemaError {
        SchemaError::InvalidData(format!("Async task failed: {}", error))
    }

    /// Converts directory creation errors with consistent messaging.
    pub fn directory_error<E: std::fmt::Display>(error: E, path: &str) -> SchemaError {
        SchemaError::InvalidData(format!("Failed to create directory {}: {}", path, error))
    }

    /// Converts transform execution errors with consistent messaging.
    pub fn transform_execution_error<E: std::fmt::Display>(error: E) -> SchemaError {
        SchemaError::InvalidData(format!("Unified transform execution failed: {}", error))
    }

    /// Converts tree operation errors with consistent messaging.
    pub fn tree_error<E: std::fmt::Display>(error: E, operation: &str) -> SchemaError {
        SchemaError::InvalidData(format!("Tree {} failed: {}", operation, error))
    }

    /// Converts flush operation errors with consistent messaging.
    pub fn flush_error<E: std::fmt::Display>(error: E) -> SchemaError {
        SchemaError::InvalidData(format!("Flush failed: {}", error))
    }

    /// Extension trait to add convenient error conversion methods to Result types.
    pub trait ErrorConversion<T> {
        fn map_to_schema_error(self, operation: &str) -> Result<T, SchemaError>;
        fn map_serialization_error(self) -> Result<T, SchemaError>;
        fn map_deserialization_error(self) -> Result<T, SchemaError>;
        fn map_database_error(self, operation: &str) -> Result<T, SchemaError>;
        fn map_file_error(self, file_path: &str, operation: &str) -> Result<T, SchemaError>;
        fn map_json_error(self) -> Result<T, SchemaError>;
        fn map_encryption_error(self) -> Result<T, SchemaError>;
        fn map_decryption_error(self) -> Result<T, SchemaError>;
        fn map_async_task_error(self) -> Result<T, SchemaError>;
        fn map_directory_error(self, path: &str) -> Result<T, SchemaError>;
        fn map_transform_execution_error(self) -> Result<T, SchemaError>;
        fn map_tree_error(self, operation: &str) -> Result<T, SchemaError>;
        fn map_flush_error(self) -> Result<T, SchemaError>;
    }

    impl<T, E: std::fmt::Display> ErrorConversion<T> for Result<T, E> {
        fn map_to_schema_error(self, operation: &str) -> Result<T, SchemaError> {
            self.map_err(|e| to_schema_error(e, operation))
        }

        fn map_serialization_error(self) -> Result<T, SchemaError> {
            self.map_err(serialization_error)
        }

        fn map_deserialization_error(self) -> Result<T, SchemaError> {
            self.map_err(deserialization_error)
        }

        fn map_database_error(self, operation: &str) -> Result<T, SchemaError> {
            self.map_err(|e| database_error(e, operation))
        }

        fn map_file_error(self, file_path: &str, operation: &str) -> Result<T, SchemaError> {
            self.map_err(|e| file_error(e, file_path, operation))
        }

        fn map_json_error(self) -> Result<T, SchemaError> {
            self.map_err(json_error)
        }

        fn map_encryption_error(self) -> Result<T, SchemaError> {
            self.map_err(encryption_error)
        }

        fn map_decryption_error(self) -> Result<T, SchemaError> {
            self.map_err(decryption_error)
        }

        fn map_async_task_error(self) -> Result<T, SchemaError> {
            self.map_err(async_task_error)
        }

        fn map_directory_error(self, path: &str) -> Result<T, SchemaError> {
            self.map_err(|e| directory_error(e, path))
        }

        fn map_transform_execution_error(self) -> Result<T, SchemaError> {
            self.map_err(transform_execution_error)
        }

        fn map_tree_error(self, operation: &str) -> Result<T, SchemaError> {
            self.map_err(|e| tree_error(e, operation))
        }

        fn map_flush_error(self) -> Result<T, SchemaError> {
            self.map_err(flush_error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = TransformError::validation("Invalid input");
        assert_eq!(error.severity(), ErrorSeverity::Warning);
        assert_eq!(error.category(), ErrorCategory::Validation);
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_error_with_field() {
        let error = TransformError::validation_with_field("Invalid input", "test_field");
        match error {
            TransformError::ValidationError { field, .. } => {
                assert_eq!(field, Some("test_field".to_string()));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_execution_error() {
        let error = TransformError::execution("Execution failed", "test_transform");
        assert_eq!(error.severity(), ErrorSeverity::Error);
        assert_eq!(error.category(), ErrorCategory::Execution);
        assert!(error.is_retryable());
    }

    #[test]
    fn test_error_handler() {
        let handler = TransformErrorHandler::new();
        let error = TransformError::validation("Test error");
        let context = ErrorContext::new("test_component", "test_operation");
        
        let action = handler.handle(&error, context);
        assert_eq!(action, ErrorAction::Abort);
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("test_component", "test_operation")
            .with_data("key1", "value1")
            .with_data("key2", "value2");
        
        assert_eq!(context.component, "test_component");
        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.context_data.len(), 2);
        assert_eq!(context.context_data.get("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_error_conversion_utilities() {
        use error_conversion::*;
        
        // Test serialization error conversion
        let result: Result<(), &str> = Err("test error");
        let schema_error = result.map_serialization_error().unwrap_err();
        assert!(schema_error.to_string().contains("Failed to serialize item"));
        
        // Test database error conversion
        let result: Result<(), &str> = Err("connection failed");
        let schema_error = result.map_database_error("insert").unwrap_err();
        assert!(schema_error.to_string().contains("Database insert failed"));
        
        // Test file error conversion
        let result: Result<(), &str> = Err("permission denied");
        let schema_error = result.map_file_error("/test/path", "read").unwrap_err();
        assert!(schema_error.to_string().contains("Failed to read file /test/path"));
        
        // Test JSON error conversion
        let result: Result<(), &str> = Err("invalid syntax");
        let schema_error = result.map_json_error().unwrap_err();
        assert!(schema_error.to_string().contains("Invalid JSON"));
        
        // Test transform execution error conversion
        let result: Result<(), &str> = Err("execution failed");
        let schema_error = result.map_transform_execution_error().unwrap_err();
        assert!(schema_error.to_string().contains("Unified transform execution failed"));
    }

    #[test]
    fn test_specific_error_conversion_functions() {
        use error_conversion::*;
        
        // Test individual conversion functions
        let error = serialization_error("test error");
        assert!(error.to_string().contains("Failed to serialize item: test error"));
        
        let error = database_error("connection timeout", "select");
        assert!(error.to_string().contains("Database select failed: connection timeout"));
        
        let error = file_error("not found", "/path/to/file", "write");
        assert!(error.to_string().contains("Failed to write file /path/to/file: not found"));
        
        let error = encryption_error("key invalid");
        assert!(error.to_string().contains("Encryption failed: key invalid"));
        
        let error = async_task_error("task panicked");
        assert!(error.to_string().contains("Async task failed: task panicked"));
    }
}