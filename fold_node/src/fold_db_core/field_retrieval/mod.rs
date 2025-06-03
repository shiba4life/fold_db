//! Field Retrieval Services
//!
//! This module provides specialized field value retrieval services for different field types.
//! It replaces the complex branching logic in FieldManager with dedicated retrievers that
//! handle both regular value retrieval and filtering for their specific field types.

use crate::schema::Schema;
use crate::schema::SchemaError;
use serde_json::Value;

pub mod base_retriever;
pub mod collection_retriever;
pub mod range_retriever;
pub mod service;
pub mod single_retriever;

pub use base_retriever::BaseRetriever;
pub use collection_retriever::CollectionFieldRetriever;
pub use range_retriever::RangeFieldRetriever;
pub use service::FieldRetrievalService;
pub use single_retriever::SingleFieldRetriever;

/// Trait for field value retrieval services
pub trait FieldRetriever {
    /// Retrieves the value of a field without any filtering
    fn get_value(&self, schema: &Schema, field: &str) -> Result<Value, SchemaError>;

    /// Retrieves the value of a field with optional filtering applied
    fn get_value_with_filter(
        &self,
        schema: &Schema,
        field: &str,
        filter: &Value,
    ) -> Result<Value, SchemaError>;

    /// Returns true if this retriever supports filtering operations
    fn supports_filtering(&self) -> bool;
}
