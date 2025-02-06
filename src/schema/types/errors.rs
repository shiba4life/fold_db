use std::fmt;

#[derive(Debug, Clone)]
pub enum SchemaError {
    NotFound(String),
    InvalidField(String),
    InvalidPermission(String),
    InvalidTransform(String),
    InvalidData(String),
    InvalidDSL(String),
    MappingError(String),
}

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SchemaError::NotFound(msg) => write!(f, "Schema not found: {}", msg),
            SchemaError::InvalidField(msg) => write!(f, "Invalid field: {}", msg),
            SchemaError::InvalidPermission(msg) => write!(f, "Invalid permission: {}", msg),
            SchemaError::InvalidTransform(msg) => write!(f, "Invalid transform: {}", msg),
            SchemaError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            SchemaError::InvalidDSL(msg) => write!(f, "Invalid DSL: {}", msg),
            SchemaError::MappingError(msg) => write!(f, "Mapping error: {}", msg),
        }
    }
}

impl std::error::Error for SchemaError {}
