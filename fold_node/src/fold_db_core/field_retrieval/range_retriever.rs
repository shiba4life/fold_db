//! Range Field Retrieval Service
//!
//! Handles value retrieval and filtering for Range fields, including:
//! - Loading AtomRefRange data from AtomManager
//! - Converting AtomRefRange to JSON format
//! - Delegating filtering to RangeField's native apply_filter method

use super::{FieldRetriever, BaseRetriever};
use crate::fold_db_core::atom_manager::AtomManager;
use crate::schema::types::field::FieldVariant;
use crate::schema::types::field::range_filter::RangeFilter;
use crate::schema::Schema;
use crate::schema::SchemaError;
use serde_json::Value;
use log::info;

pub struct RangeFieldRetriever<'a> {
    base: BaseRetriever<'a>,
}

impl<'a> RangeFieldRetriever<'a> {
    pub fn new(atom_manager: &'a AtomManager) -> Self {
        Self {
            base: BaseRetriever::new(atom_manager),
        }
    }

    /// Loads AtomRefRange data from the AtomManager
    fn load_atom_ref_range(&self, ref_atom_uuid: &str) -> Result<Option<crate::atom::AtomRefRange>, SchemaError> {
        match self.base.atom_manager.get_ref_ranges().lock() {
            Ok(ranges_guard) => {
                Ok(ranges_guard.get(ref_atom_uuid).cloned())
            }
            Err(e) => {
                info!("❌ Failed to acquire ref_ranges lock: {:?}", e);
                Err(SchemaError::InvalidData("Failed to access range data".to_string()))
            }
        }
    }

    /// Converts AtomRefRange to JSON format by loading actual atom content
    fn convert_range_to_json(&self, atom_ref_range: &crate::atom::AtomRefRange) -> Result<Value, SchemaError> {
        let mut result_obj = serde_json::Map::new();
        
        for (key, atom_uuid) in &atom_ref_range.atom_uuids {
            info!("🔑 Processing key: {} -> atom_uuid: {}", key, atom_uuid);
            
            // Access atoms directly since these are individual atoms, not AtomRefs
            match self.base.atom_manager.get_atoms().lock() {
                Ok(atoms_guard) => {
                    if let Some(atom) = atoms_guard.get(atom_uuid) {
                        info!("✅ Retrieved atom for key: {} -> content: {:?}", key, atom.content());
                        // For range field atoms, the content is the direct value
                        result_obj.insert(key.clone(), atom.content().clone());
                    } else {
                        info!("⚠️  Atom not found in atoms collection for key: {} -> atom_uuid: {}", key, atom_uuid);
                    }
                }
                Err(e) => {
                    info!("⚠️  Failed to acquire atoms lock for key {}: {:?}", key, e);
                }
            }
        }
        
        Ok(serde_json::Value::Object(result_obj))
    }

    /// Applies range filter using RangeField's native filtering
    fn apply_range_filter(&self, range_field: &mut crate::schema::types::field::RangeField, filter: &Value) -> Result<Value, SchemaError> {
        // Extract the range_filter field from the filter object
        let range_filter_value = filter.get("range_filter")
            .ok_or_else(|| SchemaError::InvalidData("Filter must contain 'range_filter' field".to_string()))?;
        
        // Parse the range_filter into a RangeFilter
        let range_filter: RangeFilter = serde_json::from_value(range_filter_value.clone())
            .map_err(|e| SchemaError::InvalidData(format!("Invalid range filter format: {}", e)))?;

        // Load AtomRefRange data into the RangeField before filtering
        if let Some(ref_atom_uuid) = &range_field.inner.ref_atom_uuid {
            info!("🔍 Loading AtomRefRange data for ref_atom_uuid: {}", ref_atom_uuid);
            
            if let Some(atom_ref_range) = self.load_atom_ref_range(ref_atom_uuid)? {
                info!("✅ Found AtomRefRange with {} keys", atom_ref_range.atom_uuids.len());
                // Populate the RangeField's atom_ref_range
                range_field.atom_ref_range = Some(atom_ref_range);
            } else {
                info!("❌ No AtomRefRange found for ref_atom_uuid: {}", ref_atom_uuid);
                return Ok(serde_json::json!({
                    "matches": {},
                    "total_count": 0
                }));
            }
        } else {
            info!("❌ No ref_atom_uuid set on RangeField");
            return Ok(serde_json::json!({
                "matches": {},
                "total_count": 0
            }));
        }
        
        info!("🔍 Applying range filter to field with {} keys",
              range_field.atom_ref_range.as_ref().map(|r| r.atom_uuids.len()).unwrap_or(0));
        
        // Use the RangeField's native apply_filter method with populated data
        let filter_result = range_field.apply_filter(&range_filter);
        info!("✅ RangeField native filtering successful: {} matches", filter_result.total_count);
        
        // Convert RangeFilterResult to the expected JSON format
        Ok(serde_json::json!({
            "matches": filter_result.matches,
            "total_count": filter_result.total_count
        }))
    }

    /// Gets the default value for a range field
    fn default_range_value() -> Value {
        serde_json::json!({})
    }
}

impl FieldRetriever for RangeFieldRetriever<'_> {
    fn get_value(&self, schema: &Schema, field: &str) -> Result<Value, SchemaError> {
        self.base.retrieve_field_value(
            schema,
            field,
            "Range",
            |ref_atom_uuid| {
                match self.load_atom_ref_range(ref_atom_uuid)? {
                    Some(atom_ref_range) => {
                        info!("🔍 Found AtomRefRange with {} entries", atom_ref_range.atom_uuids.len());
                        self.convert_range_to_json(&atom_ref_range)
                    }
                    None => {
                        info!("⚠️  No AtomRefRange found with UUID: {}", ref_atom_uuid);
                        Ok(Self::default_range_value())
                    }
                }
            },
        )
    }

    fn get_value_with_filter(&self, schema: &Schema, field: &str, filter: &Value) -> Result<Value, SchemaError> {
        info!("🔄 RangeFieldRetriever::get_value_with_filter - field: {}", field);
        
        let field_def = self.base.get_field_def(schema, field)?;
        self.base.validate_field_type(field_def, "Range", field)?;

        // Get a mutable copy for filtering
        let FieldVariant::Range(range_field) = field_def else {
            return Err(SchemaError::InvalidField(format!("Field {} is not a Range field", field)));
        };

        let mut range_field_with_data = range_field.clone();
        self.apply_range_filter(&mut range_field_with_data, filter)
    }

    fn supports_filtering(&self) -> bool {
        true
    }
}