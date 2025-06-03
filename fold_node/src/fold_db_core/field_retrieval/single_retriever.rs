//! Single Field Retrieval Service
//!
//! Handles value retrieval for Single fields, including:
//! - Loading single atom values from AtomManager
//! - Handling missing ref_atom_uuid cases
//! - Providing default values when appropriate

use super::{BaseRetriever, FieldRetriever};
use crate::fold_db_core::atom_manager::AtomManager;
use crate::schema::Schema;
use crate::schema::SchemaError;
use log::info;
use serde_json::Value;

pub struct SingleFieldRetriever<'a> {
    base: BaseRetriever<'a>,
}

impl<'a> SingleFieldRetriever<'a> {
    pub fn new(atom_manager: &'a AtomManager) -> Self {
        Self {
            base: BaseRetriever::new(atom_manager),
        }
    }
}

impl FieldRetriever for SingleFieldRetriever<'_> {
    fn get_value(&self, schema: &Schema, field: &str) -> Result<Value, SchemaError> {
        let _default_value = self.base.default_value_for_field(field);

        self.base
            .retrieve_field_value(schema, field, "Single", |ref_atom_uuid| {
                match self.base.atom_manager.get_latest_atom(ref_atom_uuid) {
                    Ok(atom) => Ok(atom.content().clone()),
                    Err(e) => Err(SchemaError::InvalidData(format!(
                        "Failed to get atom: {}",
                        e
                    ))),
                }
            })
    }

    fn get_value_with_filter(
        &self,
        schema: &Schema,
        field: &str,
        _filter: &Value,
    ) -> Result<Value, SchemaError> {
        info!("⚠️ SingleFieldRetriever::get_value_with_filter - filtering not supported for single fields, returning regular value");
        // Single fields don't support filtering, so fall back to regular value retrieval
        self.get_value(schema, field)
    }

    fn supports_filtering(&self) -> bool {
        false
    }
}
