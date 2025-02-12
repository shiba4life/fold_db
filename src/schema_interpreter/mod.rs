//! Schema interpreter module for parsing JSON schema definitions into `FoldDB` schemas.

mod interpreter;
mod types;
mod validator;

pub use interpreter::SchemaInterpreter;
pub use types::JsonSchemaDefinition;

use crate::schema::types::SchemaError;

/// Result type for schema interpretation operations
pub type Result<T> = std::result::Result<T, SchemaError>;
