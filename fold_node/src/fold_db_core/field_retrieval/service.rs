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
use std::collections::HashMap;
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

    /// Query Range schema and group results by range_key value
    /// Returns map of range_key -> field_name -> field_value
    pub fn query_range_schema(&self,
        atom_manager: &AtomManager,
        schema: &Schema,
        fields: &[String],
        range_filter: &Value
    ) -> Result<HashMap<String, HashMap<String, Value>>, SchemaError> {
        info!("🎯 FieldRetrievalService::query_range_schema - schema: {}, fields: {:?}", schema.name, fields);
        
        // Validate this is a Range schema
        let range_key = schema.range_key()
            .ok_or_else(|| SchemaError::InvalidData(format!("Schema '{}' is not a Range schema", schema.name)))?;
        
        // Extract range_filter object
        let range_filter_obj = range_filter.as_object()
            .ok_or_else(|| SchemaError::InvalidData("range_filter must be an object".to_string()))?;
        
        // Get the range_key value from the filter
        let range_key_value = range_filter_obj.get(range_key)
            .ok_or_else(|| SchemaError::InvalidData(format!("range_filter missing key '{}'", range_key)))?;
        
        info!("🔍 Range key '{}' filtering for value: {:?}", range_key, range_key_value);
        
        let mut result: HashMap<String, HashMap<String, Value>> = HashMap::new();
        let range_key_str = range_key_value.to_string().trim_matches('"').to_string();
        
        // For each requested field, get its value with the range filter
        for field_name in fields {
            info!("🔄 Processing field: {}", field_name);
            
            // Validate field exists in schema
            if !schema.fields.contains_key(field_name) {
                return Err(SchemaError::InvalidField(format!("Field '{}' not found in schema '{}'", field_name, schema.name)));
            }
            
            // Wrap the range filter in the expected format for individual field processing
            let wrapped_filter = serde_json::json!({
                "range_filter": range_filter
            });
            
            // Get field value with the wrapped range filter
            match self.get_field_value_with_filter(atom_manager, schema, field_name, &wrapped_filter) {
                Ok(field_value) => {
                    // Ensure the range_key entry exists in result
                    let range_entry = result.entry(range_key_str.clone()).or_default();
                    range_entry.insert(field_name.clone(), field_value);
                    info!("✅ Added field '{}' to range key '{}'", field_name, range_key_str);
                }
                Err(e) => {
                    info!("⚠️ Failed to get field '{}': {:?}", field_name, e);
                    return Err(e);
                }
            }
        }
        
        info!("✅ FieldRetrievalService::query_range_schema COMPLETE - schema: {}, result keys: {:?}",
              schema.name, result.keys().collect::<Vec<_>>());
        
        Ok(result)
    }
}