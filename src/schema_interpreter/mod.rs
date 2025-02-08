//! Schema interpreter module for parsing JSON schema definitions into FoldDB schemas.

mod types;
mod validator;
mod interpreter;

pub use interpreter::SchemaInterpreter;
pub use types::JsonSchemaDefinition;

use crate::schema::types::{Schema, SchemaError};

/// Result type for schema interpretation operations
pub type Result<T> = std::result::Result<T, SchemaError>;
