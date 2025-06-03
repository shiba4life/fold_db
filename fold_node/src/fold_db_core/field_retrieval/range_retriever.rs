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
        
        for (key, atom_uuids) in &atom_ref_range.atom_uuids {
            info!("🔑 Processing key: {} -> atom_uuids: {:?}", key, atom_uuids);
            
            // Handle multiple atoms per key
            match atom_uuids.len().cmp(&1) {
                std::cmp::Ordering::Equal => {
                    // Single atom - maintain backward compatibility by storing the content directly
                    let atom_uuid = &atom_uuids[0];
                    match self.base.atom_manager.get_atoms().lock() {
                        Ok(atoms_guard) => {
                            if let Some(atom) = atoms_guard.get(atom_uuid) {
                                result_obj.insert(key.clone(), atom.content().clone());
                                info!("✅ Added single atom content for key: {} -> value: {:?}", key, atom.content());
                            } else {
                                info!("⚠️  Atom not found in atoms collection for key: {} -> atom_uuid: {}", key, atom_uuid);
                            }
                        }
                        Err(e) => {
                            info!("⚠️  Failed to acquire atoms lock for key {}: {:?}", key, e);
                        }
                    }
                }
                std::cmp::Ordering::Greater => {
                    // Multiple atoms - store as an array
                    let mut atom_contents = Vec::new();
                    match self.base.atom_manager.get_atoms().lock() {
                        Ok(atoms_guard) => {
                            for atom_uuid in atom_uuids {
                                if let Some(atom) = atoms_guard.get(atom_uuid) {
                                    atom_contents.push(atom.content().clone());
                                    info!("✅ Added atom content for key: {} -> atom_uuid: {} -> value: {:?}", key, atom_uuid, atom.content());
                                } else {
                                    info!("⚠️  Atom not found in atoms collection for key: {} -> atom_uuid: {}", key, atom_uuid);
                                }
                            }
                            result_obj.insert(key.clone(), serde_json::Value::Array(atom_contents));
                            info!("✅ Added {} atoms for key: {}", atom_uuids.len(), key);
                        }
                        Err(e) => {
                            info!("⚠️  Failed to acquire atoms lock for key {}: {:?}", key, e);
                        }
                    }
                }
                std::cmp::Ordering::Less => {
                    // Empty atom_uuids - skip this key
                }
            }
            // If atom_uuids is empty, we skip this key (no content to add)
        }
        
        Ok(serde_json::Value::Object(result_obj))
    }

    /// Applies range filter using RangeField's native filtering
    fn apply_range_filter(&self, range_field: &mut crate::schema::types::field::RangeField, filter: &Value) -> Result<Value, SchemaError> {
        // Check if the filter contains range_filter - if not, return empty result
        let range_filter_value = match filter.get("range_filter") {
            Some(value) => value,
            None => {
                info!("🔄 Filter does not contain 'range_filter', returning empty result for range field");
                return Ok(serde_json::json!({
                    "matches": {},
                    "total_count": 0
                }));
            }
        };
        
        // Convert range_filter to RangeFilter enum
        let range_filter = if let Ok(range_filter) = serde_json::from_value::<RangeFilter>(range_filter_value.clone()) {
            range_filter
        } else if let Some(obj) = range_filter_value.as_object() {
            if obj.len() == 1 {
                // Get the single key-value pair
                let (_key, value) = obj.iter().next().unwrap();
                
                // Convert the value to string and create RangeFilter::Key
                let value_str = match value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => serde_json::to_string(value)
                        .map_err(|e| SchemaError::InvalidData(format!("Failed to convert range filter value to string: {}", e)))?
                        .trim_matches('"').to_string(), // Remove quotes from JSON strings
                };
                
                RangeFilter::Key(value_str)
            } else {
                return Err(SchemaError::InvalidData(format!(
                    "range_filter should contain exactly one key-value pair, found {} keys", 
                    obj.len()
                )));
            }
        } else {
            return Err(SchemaError::InvalidData(format!(
                "Invalid range filter format: expected object with single key-value pair or valid RangeFilter enum, got: {:?}", 
                range_filter_value
            )));
        };

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
            info!("❌ No ref_atom_uuid found in RangeField");
            return Ok(serde_json::json!({
                "matches": {},
                "total_count": 0
            }));
        }

        // Apply the filter using RangeField's native apply_filter method
        let filter_result = range_field.apply_filter(&range_filter);
        info!("🔍 Filter result: {} matches", filter_result.matches.len());

        // If no matches found, return empty result
        if filter_result.matches.is_empty() {
            return Ok(serde_json::json!({
                "matches": {},
                "total_count": 0
            }));
        }

        // Convert UUIDs back to actual atom content
        let mut content_matches = std::collections::HashMap::new();
        let mut grouped_by_original_key = std::collections::HashMap::<String, Vec<serde_json::Value>>::new();
        
        for (match_key, atom_uuid) in &filter_result.matches {
            // Extract original key (remove _N suffix if present)
            let original_key = if let Some(underscore_pos) = match_key.rfind('_') {
                if match_key[underscore_pos + 1..].chars().all(|c| c.is_ascii_digit()) {
                    &match_key[..underscore_pos]
                } else {
                    match_key
                }
            } else {
                match_key
            };
            
            // Load actual atom content
            match self.base.atom_manager.get_atoms().lock() {
                Ok(atoms_guard) => {
                    if let Some(atom) = atoms_guard.get(atom_uuid) {
                        grouped_by_original_key
                            .entry(original_key.to_string())
                            .or_default()
                            .push(atom.content().clone());
                        info!("✅ Loaded atom content for key: {} -> atom_uuid: {} -> content: {:?}",
                              original_key, atom_uuid, atom.content());
                    } else {
                        info!("⚠️  Atom not found for UUID: {}", atom_uuid);
                    }
                }
                Err(e) => {
                    info!("⚠️  Failed to acquire atoms lock: {:?}", e);
                }
            }
        }
        
        // Convert grouped content to the final format
        for (key, contents) in grouped_by_original_key {
            if contents.len() == 1 {
                content_matches.insert(key, contents[0].clone());
            } else {
                content_matches.insert(key, serde_json::Value::Array(contents));
            }
        }

        Ok(serde_json::json!({
            "matches": content_matches,
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