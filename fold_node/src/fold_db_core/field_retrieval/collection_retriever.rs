//! Collection Field Retrieval Service
//!
//! Handles value retrieval for Collection fields, including:
//! - Loading AtomRefCollection data from AtomManager
//! - Converting collection data to JSON format
//! - Handling missing ref_atom_uuid cases

use super::{BaseRetriever, FieldRetriever};
use crate::fold_db_core::atom_manager::AtomManager;
use crate::schema::Schema;
use crate::schema::SchemaError;
use log::info;
use serde_json::Value;

pub struct CollectionFieldRetriever<'a> {
    base: BaseRetriever<'a>,
}

impl<'a> CollectionFieldRetriever<'a> {
    pub fn new(atom_manager: &'a AtomManager) -> Self {
        Self {
            base: BaseRetriever::new(atom_manager),
        }
    }

    /// Loads AtomRefCollection data from the AtomManager
    fn load_atom_ref_collection(
        &self,
        ref_atom_uuid: &str,
    ) -> Result<Option<crate::atom::AtomRefCollection>, SchemaError> {
        match self.base.atom_manager.get_ref_collections().lock() {
            Ok(collections_guard) => Ok(collections_guard.get(ref_atom_uuid).cloned()),
            Err(e) => {
                info!("❌ Failed to acquire ref_collections lock: {:?}", e);
                Err(SchemaError::InvalidData(
                    "Failed to access collection data".to_string(),
                ))
            }
        }
    }

    /// Converts AtomRefCollection to JSON format by loading actual atom content
    fn convert_collection_to_json(
        &self,
        _atom_ref_collection: &crate::atom::AtomRefCollection,
    ) -> Result<Value, SchemaError> {
        // TODO: AtomRefCollection doesn't expose a way to iterate over its items
        // For now, return an empty array. This needs to be implemented properly
        // by either adding a public method to AtomRefCollection or using a different approach
        info!("⚠️  Collection field conversion not yet implemented - AtomRefCollection doesn't expose iteration");
        Ok(serde_json::Value::Array(Vec::new()))
    }
}

impl FieldRetriever for CollectionFieldRetriever<'_> {
    fn get_value(&self, schema: &Schema, field: &str) -> Result<Value, SchemaError> {
        self.base
            .retrieve_field_value(schema, field, "Collection", |ref_atom_uuid| {
                match self.load_atom_ref_collection(ref_atom_uuid)? {
                    Some(atom_ref_collection) => {
                        info!("🔍 Found AtomRefCollection");
                        self.convert_collection_to_json(&atom_ref_collection)
                    }
                    None => {
                        info!(
                            "⚠️  No AtomRefCollection found with UUID: {}",
                            ref_atom_uuid
                        );
                        Ok(Value::Null)
                    }
                }
            })
    }

    fn get_value_with_filter(
        &self,
        schema: &Schema,
        field: &str,
        _filter: &Value,
    ) -> Result<Value, SchemaError> {
        info!("⚠️ CollectionFieldRetriever::get_value_with_filter - filtering not yet supported for collection fields, returning regular value");
        // Collection fields don't support filtering yet, so fall back to regular value retrieval
        // In the future, we could add collection-specific filtering here
        self.get_value(schema, field)
    }

    fn supports_filtering(&self) -> bool {
        false // Could be changed to true when collection filtering is implemented
    }
}
