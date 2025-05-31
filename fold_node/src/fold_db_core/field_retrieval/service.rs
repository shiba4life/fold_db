//! Field Retrieval Service Coordinator
//!
//! Provides a unified interface for field value retrieval by delegating to
//! appropriate specialized retrievers based on field type. This replaces the
//! complex branching logic in FieldManager.

use super::{FieldRetriever, SingleFieldRetriever, RangeFieldRetriever, CollectionFieldRetriever};
use crate::fold_db_core::atom_manager::AtomManager;
use crate::schema::types::field::FieldVariant;
use crate::schema::Schema;
use crate::schema::SchemaError;
use serde_json::Value;
use log::info;

pub struct FieldRetrievalService;

impl Default for FieldRetrievalService {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldRetrievalService {
    pub fn new() -> Self {
        Self
    }

    /// Retrieves a field value without filtering
    pub fn get_field_value(&self, atom_manager: &AtomManager, schema: &Schema, field: &str) -> Result<Value, SchemaError> {
        info!("🔍 FieldRetrievalService::get_field_value - schema: {}, field: {}", schema.name, field);
        
        let field_def = schema
            .fields
            .get(field)
            .ok_or_else(|| SchemaError::InvalidField(format!("Field {} not found", field)))?;

        let result = match field_def {
            FieldVariant::Single(_) => {
                let retriever = SingleFieldRetriever::new(atom_manager);
                FieldRetriever::get_value(&retriever, schema, field)?
            }
            FieldVariant::Range(_) => {
                let retriever = RangeFieldRetriever::new(atom_manager);
                FieldRetriever::get_value(&retriever, schema, field)?
            }
            FieldVariant::Collection(_) => {
                let retriever = CollectionFieldRetriever::new(atom_manager);
                FieldRetriever::get_value(&retriever, schema, field)?
            }
        };
        
        info!("✅ FieldRetrievalService::get_field_value COMPLETE - schema: {}, field: {}, result: {:?}",
              schema.name, field, result);
        
        Ok(result)
    }

    /// Retrieves a field value with optional filtering
    pub fn get_field_value_with_filter(&self, atom_manager: &AtomManager, schema: &Schema, field: &str, filter: &Value) -> Result<Value, SchemaError> {
        info!("🔄 FieldRetrievalService::get_field_value_with_filter - schema: {}, field: {}", schema.name, field);
        
        let field_def = schema
            .fields
            .get(field)
            .ok_or_else(|| SchemaError::InvalidField(format!("Field {} not found", field)))?;

        let result = match field_def {
            FieldVariant::Single(_) => {
                let retriever = SingleFieldRetriever::new(atom_manager);
                info!("⚠️ Single field does not support filtering, returning regular value");
                FieldRetriever::get_value(&retriever, schema, field)?
            }
            FieldVariant::Range(_) => {
                let retriever = RangeFieldRetriever::new(atom_manager);
                info!("✅ Range field supports filtering, applying filter");
                FieldRetriever::get_value_with_filter(&retriever, schema, field, filter)?
            }
            FieldVariant::Collection(_) => {
                let retriever = CollectionFieldRetriever::new(atom_manager);
                info!("⚠️ Collection field does not support filtering yet, returning regular value");
                FieldRetriever::get_value(&retriever, schema, field)?
            }
        };
        
        info!("✅ FieldRetrievalService::get_field_value_with_filter COMPLETE - schema: {}, field: {}, result: {:?}",
              schema.name, field, result);
        
        Ok(result)
    }

    /// Checks if a field supports filtering
    pub fn supports_filtering(&self, schema: &Schema, field: &str) -> Result<bool, SchemaError> {
        let field_def = schema
            .fields
            .get(field)
            .ok_or_else(|| SchemaError::InvalidField(format!("Field {} not found", field)))?;

        let supports = match field_def {
            FieldVariant::Single(_) => false,
            FieldVariant::Range(_) => true,
            FieldVariant::Collection(_) => false, // Could be changed to true when collection filtering is implemented
        };
        
        Ok(supports)
    }
}