//! Schema interpreter module for parsing JSON schema definitions into `FoldDB` schemas.

mod interpreter;
pub mod types;
mod validator;

use crate::schema::types::SchemaError;

/// Result type for schema interpretation operations
pub(crate) type Result<T> = std::result::Result<T, SchemaError>;

// Re-export types needed by lib.rs
pub use interpreter::SchemaInterpreter;
pub use types::JsonSchemaDefinition;
