//! Range Field Retrieval Service
//!
//! Handles value retrieval and filtering for Range fields, including:
//! - Loading AtomRefRange data from AtomManager
//! - Converting AtomRefRange to JSON format
//! - Delegating filtering to RangeField's native apply_filter method

use super::{BaseRetriever, FieldRetriever};
use crate::fold_db_core::atom_manager::AtomManager;
use crate::schema::types::field::range_filter::RangeFilter;
use crate::schema::types::field::FieldVariant;
use crate::schema::Schema;
use crate::schema::SchemaError;
use log::info;
use serde_json::Value;

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
    fn load_atom_ref_range(
        &self,
        ref_atom_uuid: &str,
    ) -> Result<Option<crate::atom::AtomRefRange>, SchemaError> {
        match self.base.atom_manager.get_ref_ranges().lock() {
            Ok(ranges_guard) => Ok(ranges_guard.get(ref_atom_uuid).cloned()),
            Err(e) => {
                info!("❌ Failed to acquire ref_ranges lock: {:?}", e);
                Err(SchemaError::InvalidData(
                    "Failed to access range data".to_string(),
                ))
            }
        }
    }

    /// Converts AtomRefRange to JSON format by loading actual atom content
    fn convert_range_to_json(
        &self,
        atom_ref_range: &crate::atom::AtomRefRange,
    ) -> Result<Value, SchemaError> {
        let mut result_obj = serde_json::Map::new();

        for (key, atom_uuid) in &atom_ref_range.atom_uuids {
            info!("🔑 Processing key: {} -> atom_uuid: {}", key, atom_uuid);

            // Single atom per key - store the content directly
            match self.base.atom_manager.get_atoms().lock() {
                Ok(atoms_guard) => {
                    if let Some(atom) = atoms_guard.get(atom_uuid) {
                        result_obj.insert(key.clone(), atom.content().clone());
                        info!(
                            "✅ Added atom content for key: {} -> value: {:?}",
                            key,
                            atom.content()
                        );
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
    fn apply_range_filter(
        &self,
        range_field: &mut crate::schema::types::field::RangeField,
        filter: &Value,
    ) -> Result<Value, SchemaError> {
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

        // For range schemas, extract the range_key value from the filter
        // Format: {"range_filter": {"range_key_field": {"Key": "abc"}}} or {"range_filter": {"range_key_field": "abc"}}
        let range_filter = if let Some(obj) = range_filter_value.as_object() {
            if obj.len() == 1 {
                // Get the single key-value pair (range_key_field -> filter_spec)
                let (range_key_field, filter_spec) = obj.iter().next().unwrap();
                info!("🔍 Range filter for field '{}': {:?}", range_key_field, filter_spec);

                // Try to parse filter_spec as a RangeFilter (e.g., {"Key": "abc"})
                if let Ok(range_filter) = serde_json::from_value::<RangeFilter>(filter_spec.clone()) {
                    info!("✅ Parsed structured range filter: {:?}", range_filter);
                    range_filter
                } else {
                    // Fallback: convert the filter_spec value to string and create RangeFilter::Key
                    let value_str = match filter_spec {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        _ => serde_json::to_string(filter_spec)
                            .map_err(|e| {
                                SchemaError::InvalidData(format!(
                                    "Failed to convert range filter value to string: {}",
                                    e
                                ))
                            })?
                            .trim_matches('"')
                            .to_string(), // Remove quotes from JSON strings
                    };

                    info!("✅ Using fallback Key filter for range_key value: '{}'", value_str);
                    RangeFilter::Key(value_str)
                }
            } else {
                return Err(SchemaError::InvalidData(format!(
                    "range_filter should contain exactly one key-value pair, found {} keys",
                    obj.len()
                )));
            }
        } else {
            // Try direct parsing as RangeFilter
            serde_json::from_value::<RangeFilter>(range_filter_value.clone()).map_err(|e| {
                SchemaError::InvalidData(format!(
                    "Invalid range filter format: expected object with range_key field or valid RangeFilter enum, got: {:?}, error: {}",
                    range_filter_value, e
                ))
            })?
        };

        // Load AtomRefRange data into the RangeField before filtering
        if let Some(ref_atom_uuid) = &range_field.inner.ref_atom_uuid {
            info!(
                "🔍 Loading AtomRefRange data for ref_atom_uuid: {}",
                ref_atom_uuid
            );

            if let Some(atom_ref_range) = self.load_atom_ref_range(ref_atom_uuid)? {
                info!(
                    "✅ Found AtomRefRange with {} keys",
                    atom_ref_range.atom_uuids.len()
                );
                // Populate the RangeField's atom_ref_range
                range_field.atom_ref_range = Some(atom_ref_range);
            } else {
                info!(
                    "🔧 AtomRefRange not in memory for ref_atom_uuid: {}, creating it (normal for schemas loaded from disk)",
                    ref_atom_uuid
                );
                
                // Create missing AtomRefRange (same logic as mutations)
                let new_range = crate::atom::AtomRefRange::new("system".to_string());
                
                // Store it in the AtomManager
                match self.base.atom_manager.get_ref_ranges().lock() {
                    Ok(mut ranges_guard) => {
                        ranges_guard.insert(ref_atom_uuid.to_string(), new_range.clone());
                        range_field.atom_ref_range = Some(new_range);
                        info!("✅ Created missing AtomRefRange for ref_atom_uuid: {}", ref_atom_uuid);
                    }
                    Err(e) => {
                        info!("❌ Failed to acquire ref_ranges lock for creation: {:?}", e);
                        return Ok(serde_json::json!({
                            "matches": {},
                            "total_count": 0
                        }));
                    }
                }
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
        let mut grouped_by_original_key =
            std::collections::HashMap::<String, Vec<serde_json::Value>>::new();

        for (match_key, atom_uuid) in &filter_result.matches {
            // Extract original key (remove _N suffix if present, where N is a single digit)
            // This only strips suffixes we added ourselves for multiple atoms per key
            let original_key = if let Some(underscore_pos) = match_key.rfind('_') {
                let suffix = &match_key[underscore_pos + 1..];
                // Only strip if it's a single digit (0-9) - these are suffixes we add for multiple atoms
                if suffix.len() == 1 && suffix.chars().all(|c| c.is_ascii_digit()) {
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
                        info!(
                            "✅ Loaded atom content for key: {} -> atom_uuid: {} -> content: {:?}",
                            original_key,
                            atom_uuid,
                            atom.content()
                        );
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
        self.base
            .retrieve_field_value(schema, field, "Range", |ref_atom_uuid| {
                match self.load_atom_ref_range(ref_atom_uuid)? {
                    Some(atom_ref_range) => {
                        info!(
                            "🔍 Found AtomRefRange with {} entries",
                            atom_ref_range.atom_uuids.len()
                        );
                        self.convert_range_to_json(&atom_ref_range)
                    }
                    None => {
                        info!("⚠️  No AtomRefRange found with UUID: {}", ref_atom_uuid);
                        Ok(Self::default_range_value())
                    }
                }
            })
    }

    fn get_value_with_filter(
        &self,
        schema: &Schema,
        field: &str,
        filter: &Value,
    ) -> Result<Value, SchemaError> {
        info!(
            "🔄 RangeFieldRetriever::get_value_with_filter - field: {}",
            field
        );

        let field_def = self.base.get_field_def(schema, field)?;
        self.base.validate_field_type(field_def, "Range", field)?;

        // Get a mutable copy for filtering
        let FieldVariant::Range(range_field) = field_def else {
            return Err(SchemaError::InvalidField(format!(
                "Field {} is not a Range field",
                field
            )));
        };

        let mut range_field_with_data = range_field.clone();
        self.apply_range_filter(&mut range_field_with_data, filter)
    }

    fn supports_filtering(&self) -> bool {
        true
    }
}
