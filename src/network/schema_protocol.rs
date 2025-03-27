use serde::{Serialize, Deserialize};

/// Protocol name for schema operations
#[derive(Debug, Clone)]
pub struct SchemaProtocol;

/// Schema request types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaRequest {
    /// Request to check availability of specific schemas
    CheckSchemas(Vec<String>),
}

/// Schema response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaResponse {
    /// Returns subset of requested schemas that are available
    AvailableSchemas(Vec<String>),
    /// Error response
    Error(String),
}

/// Codec for schema protocol messages
#[derive(Debug, Clone)]
pub struct SchemaCodec;
