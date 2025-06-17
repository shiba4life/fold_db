//! Input request handling and validation for ingestion

use crate::ingestion::{IngestionError, IngestionResult};
use serde_json::Value;

/// Request for processing JSON ingestion
#[derive(Debug, serde::Deserialize)]
pub struct IngestionRequest {
    /// JSON data to ingest
    pub data: Value,
    /// Whether to auto-execute mutations after generation
    pub auto_execute: Option<bool>,
    /// Trust distance for mutations
    pub trust_distance: Option<u32>,
    /// Public key for mutations
    pub pub_key: Option<String>,
}

impl IngestionRequest {
    /// Validate JSON input
    pub fn validate_input(&self) -> IngestionResult<()> {
        if self.data.is_null() {
            return Err(IngestionError::invalid_input("Input data cannot be null"));
        }

        if !self.data.is_object() && !self.data.is_array() {
            return Err(IngestionError::invalid_input(
                "Input data must be a JSON object or array",
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_input() {
        // Valid inputs
        let request = IngestionRequest {
            data: serde_json::json!({"key": "value"}),
            auto_execute: None,
            trust_distance: None,
            pub_key: None,
        };
        assert!(request.validate_input().is_ok());

        let request = IngestionRequest {
            data: serde_json::json!([1, 2, 3]),
            auto_execute: None,
            trust_distance: None,
            pub_key: None,
        };
        assert!(request.validate_input().is_ok());

        // Invalid inputs
        let request = IngestionRequest {
            data: serde_json::json!(null),
            auto_execute: None,
            trust_distance: None,
            pub_key: None,
        };
        assert!(request.validate_input().is_err());

        let request = IngestionRequest {
            data: serde_json::json!("string"),
            auto_execute: None,
            trust_distance: None,
            pub_key: None,
        };
        assert!(request.validate_input().is_err());

        let request = IngestionRequest {
            data: serde_json::json!(42),
            auto_execute: None,
            trust_distance: None,
            pub_key: None,
        };
        assert!(request.validate_input().is_err());
    }
}