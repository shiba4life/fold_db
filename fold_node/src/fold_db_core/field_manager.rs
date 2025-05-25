use super::atom_manager::AtomManager;
use super::context::AtomContext;
use super::transform_manager::TransformManager;
use crate::atom::AtomStatus;
use crate::schema::types::Field;
use crate::schema::types::field::FieldVariant;
use crate::schema::Schema;
use crate::schema::SchemaError;
use serde_json::Value;
use log::info;

use std::sync::{Arc, RwLock};
use super::transform_orchestrator::TransformOrchestrator;

pub struct FieldManager {
    pub(super) atom_manager: AtomManager,
    transform_manager: Arc<RwLock<Option<Arc<TransformManager>>>>,
    orchestrator: Arc<RwLock<Option<Arc<TransformOrchestrator>>>>,
}

impl FieldManager {
    pub fn new(atom_manager: AtomManager) -> Self {
        Self {
            atom_manager,
            transform_manager: Arc::new(RwLock::new(None)),
            orchestrator: Arc::new(RwLock::new(None)),
        }
    }

    pub fn set_transform_manager(&self, manager: Arc<TransformManager>) -> Result<(), SchemaError> {
        let mut guard = self
            .transform_manager
            .write()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire transform_manager lock".to_string()))?;
        *guard = Some(manager);
        Ok(())
    }

    pub fn get_transform_manager(&self) -> Result<Option<Arc<TransformManager>>, SchemaError> {
        let guard = self
            .transform_manager
            .read()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire transform_manager lock".to_string()))?;
        Ok(guard.clone())
    }

    pub fn set_orchestrator(&self, orchestrator: Arc<TransformOrchestrator>) -> Result<(), SchemaError> {
        let mut guard = self
            .orchestrator
            .write()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire orchestrator lock".to_string()))?;
        *guard = Some(orchestrator);
        Ok(())
    }

    pub fn get_orchestrator(&self) -> Result<Option<Arc<TransformOrchestrator>>, SchemaError> {
        let guard = self
            .orchestrator
            .read()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire orchestrator lock".to_string()))?;
        Ok(guard.clone())
    }

    pub fn get_or_create_atom_ref(
        &mut self,
        schema: &Schema,
        field: &str,
        source_pub_key: &str,
    ) -> Result<String, SchemaError> {
        let mut ctx = AtomContext::new(
            schema,
            field,
            source_pub_key.to_string(),
            &mut self.atom_manager,
        );
        ctx.get_or_create_atom_ref()
    }

    pub fn get_field_value(&self, schema: &Schema, field: &str) -> Result<Value, SchemaError> {
        let field_def = schema
            .fields
            .get(field)
            .ok_or_else(|| SchemaError::InvalidField(format!("Field {} not found", field)))?;

        let result = match field_def {
            crate::schema::types::field::FieldVariant::Range(_range_field) => {
                // For Range fields, treat them like Single fields for now
                // The data is stored as a single JSON object atom
                info!("get_field_value - Range field: {}, ref_atom_uuid: {:?}", field, field_def.ref_atom_uuid());
                if let Some(ref_atom_uuid) = field_def.ref_atom_uuid() {
                    // Try to get the atom reference
                    let ref_atoms = self.atom_manager.get_ref_atoms();
                    let atoms = self.atom_manager.get_atoms();

                    let atom_uuid = {
                        let guard = ref_atoms
                            .lock()
                            .map_err(|_| SchemaError::InvalidData("Failed to acquire ref_atoms lock".to_string()))?;
                        guard
                            .get(ref_atom_uuid.as_str())
                            .map(|aref| aref.get_atom_uuid().clone())
                    };

                    // If we have an atom UUID, try to get the atom
                    if let Some(atom_uuid) = atom_uuid {
                        let guard = atoms
                            .lock()
                            .map_err(|_| SchemaError::InvalidData("Failed to acquire atoms lock".to_string()))?;
                        if let Some(atom) = guard.get(&atom_uuid) {
                            atom.content().clone()
                        } else {
                            self.atom_manager
                                .get_latest_atom(ref_atom_uuid)
                                .map(|a| a.content().clone())
                                .unwrap_or_else(|_| Self::default_value(field))
                        }
                    } else {
                        self.atom_manager
                            .get_latest_atom(ref_atom_uuid)
                            .map(|a| a.content().clone())
                            .unwrap_or_else(|_| Self::default_value(field))
                    }
                } else {
                    Value::Null
                }
            }
            _ => {
                // For Single and Collection fields, use the existing logic
                if let Some(ref_atom_uuid) = field_def.ref_atom_uuid() {
                    // Try to get the atom reference
                    let ref_atoms = self.atom_manager.get_ref_atoms();
                    let atoms = self.atom_manager.get_atoms();

                    let atom_uuid = {
                        let guard = ref_atoms
                            .lock()
                            .map_err(|_| SchemaError::InvalidData("Failed to acquire ref_atoms lock".to_string()))?;
                        guard
                            .get(ref_atom_uuid.as_str())
                            .map(|aref| aref.get_atom_uuid().clone())
                    };

                    // If we have an atom UUID, try to get the atom
                    if let Some(atom_uuid) = atom_uuid {
                        let guard = atoms
                            .lock()
                            .map_err(|_| SchemaError::InvalidData("Failed to acquire atoms lock".to_string()))?;
                        if let Some(atom) = guard.get(&atom_uuid) {
                            atom.content().clone()
                        } else {
                            self.atom_manager
                                .get_latest_atom(ref_atom_uuid)
                                .map(|a| a.content().clone())
                                .unwrap_or_else(|_| Self::default_value(field))
                        }
                    } else {
                        self.atom_manager
                            .get_latest_atom(ref_atom_uuid)
                            .map(|a| a.content().clone())
                            .unwrap_or_else(|_| Self::default_value(field))
                    }
                } else {
                    Value::Null
                }
            }
        };

        info!(
            "get_field_value - schema: {}, field: {}, aref_uuid: {:?}, result: {:?}",
            schema.name,
            field,
            field_def.ref_atom_uuid(),
            result
        );

        Ok(result)
    }

    pub fn get_filtered_field_value(&self, schema: &Schema, field: &str, filter: &Value) -> Result<Value, SchemaError> {
        let field_def = schema
            .fields
            .get(field)
            .ok_or_else(|| SchemaError::InvalidField(format!("Field {} not found", field)))?;

        // Check if this is a RangeField that supports filtering
        match field_def {
            FieldVariant::Range(_range_field) => {
                // Extract the range_filter from the filter object
                let range_filter = if let Some(filter_obj) = filter.as_object() {
                    if let Some(target_field) = filter_obj.get("field") {
                        if target_field.as_str() == Some(field) {
                            filter_obj.get("range_filter")
                                .ok_or_else(|| SchemaError::InvalidData("Missing range_filter in filter".to_string()))?
                        } else {
                            // This filter is not for this field, return regular field value
                            return self.get_field_value(schema, field);
                        }
                    } else {
                        // Assume the entire filter is the range filter (for backward compatibility)
                        filter
                    }
                } else {
                    return Err(SchemaError::InvalidData("Filter must be an object".to_string()));
                };

                // Get the full Range field data first
                info!("get_filtered_field_value - About to call get_field_value for Range field: {}", field);
                let range_data = self.get_field_value(schema, field)?;
                info!("get_filtered_field_value - Retrieved range_data: {:?}", range_data);
                
                // Apply filtering to the data
                if let Value::Object(data_map) = range_data {
                    let mut matches = std::collections::HashMap::new();
                    
                    // Parse the range filter
                    if let Some(filter_obj) = range_filter.as_object() {
                        if let Some(key_prefix) = filter_obj.get("KeyPrefix").and_then(|v| v.as_str()) {
                            // Filter by key prefix
                            for (key, value) in data_map {
                                if key.starts_with(key_prefix) {
                                    matches.insert(key, value.as_str().unwrap_or("").to_string());
                                }
                            }
                        } else if let Some(exact_key) = filter_obj.get("Key").and_then(|v| v.as_str()) {
                            // Filter by exact key
                            if let Some(value) = data_map.get(exact_key) {
                                matches.insert(exact_key.to_string(), value.as_str().unwrap_or("").to_string());
                            }
                        } else if let Some(pattern) = filter_obj.get("KeyPattern").and_then(|v| v.as_str()) {
                            // Filter by key pattern (simple glob matching)
                            for (key, value) in data_map {
                                // Simple glob pattern matching
                                let is_match = if pattern.ends_with('*') {
                                    let prefix = &pattern[..pattern.len() - 1];
                                    key.starts_with(prefix)
                                } else if pattern.starts_with('*') {
                                    let suffix = &pattern[1..];
                                    key.ends_with(suffix)
                                } else {
                                    key == pattern
                                };
                                
                                if is_match {
                                    matches.insert(key, value.as_str().unwrap_or("").to_string());
                                }
                            }
                        }
                    }
                    
                    let result = serde_json::json!({
                        "matches": matches,
                        "total_count": matches.len()
                    });
                    Ok(result)
                } else {
                    // No data or invalid data format
                    let result = serde_json::json!({
                        "matches": {},
                        "total_count": 0
                    });
                    Ok(result)
                }
            }
            _ => {
                // For non-range fields, fall back to regular field value retrieval
                // In the future, we could add filtering support for other field types
                self.get_field_value(schema, field)
            }
        }
    }

    pub fn set_field_value(
        &mut self,
        schema: &mut Schema,
        field: &str,
        content: Value,
        source_pub_key: String,
    ) -> Result<(), SchemaError> {
        let mut ctx = AtomContext::new(schema, field, source_pub_key.clone(), &mut self.atom_manager);

        let field_def = ctx.get_field_def()?;
        if matches!(field_def, crate::schema::types::FieldVariant::Collection(_)) {
            return Err(SchemaError::InvalidField(
                "Collection fields cannot be updated without id".to_string(),
            ));
        }

        // Special handling for Range fields
        if let crate::schema::types::FieldVariant::Range(_range_field) = field_def {
            return self.set_range_field_value(schema, field, content, source_pub_key);
        }

        let aref_uuid = ctx.get_or_create_atom_ref()?;
        let prev_atom_uuid = {
            let ref_atoms = ctx.atom_manager.get_ref_atoms();
            let guard = ref_atoms
                .lock()
                .map_err(|_| SchemaError::InvalidData("Failed to acquire ref_atoms lock".to_string()))?;
            guard
                .get(&aref_uuid)
                .map(|aref| aref.get_atom_uuid().to_string())
        };

        ctx.create_and_update_atom(prev_atom_uuid, content.clone(), None)?;

        info!(
            "set_field_value - schema: {}, field: {}, aref_uuid: {}, result: success",
            schema.name,
            field,
            aref_uuid
        );

        Ok(())
    }

    fn set_range_field_value(
        &mut self,
        schema: &mut Schema,
        field: &str,
        content: Value,
        source_pub_key: String,
    ) -> Result<(), SchemaError> {
        // For now, let's store the Range field data as a single atom like other fields
        // but mark it specially so we can retrieve it correctly
        let aref_uuid = {
            let mut ctx = AtomContext::new(schema, field, source_pub_key.clone(), &mut self.atom_manager);
            let aref_uuid = ctx.get_or_create_atom_ref()?;
            
            let prev_atom_uuid = {
                let ref_atoms = ctx.atom_manager.get_ref_atoms();
                let guard = ref_atoms
                    .lock()
                    .map_err(|_| SchemaError::InvalidData("Failed to acquire ref_atoms lock".to_string()))?;
                guard
                    .get(&aref_uuid)
                    .map(|aref| aref.get_atom_uuid().to_string())
            };

            ctx.create_and_update_atom(prev_atom_uuid, content.clone(), None)?;
            aref_uuid
        }; // ctx is dropped here
        
        // Update the field definition with the aref_uuid if it doesn't have one
        // We do this after dropping the context to avoid borrow conflicts
        if let Some(field_def) = schema.fields.get_mut(field) {
            if field_def.ref_atom_uuid().is_none() {
                field_def.set_ref_atom_uuid(aref_uuid.clone());
            }
        }

        info!(
            "set_field_value - schema: {}, field: {}, aref_uuid: {}, result: success",
            schema.name,
            field,
            aref_uuid
        );

        Ok(())
    }

    pub fn update_field(
        &mut self,
        schema: &Schema,
        field: &str,
        content: Value,
        source_pub_key: String,
    ) -> Result<(), SchemaError> {
        let mut ctx = AtomContext::new(schema, field, source_pub_key, &mut self.atom_manager);

        let field_def = ctx.get_field_def()?;
        if matches!(field_def, crate::schema::types::FieldVariant::Collection(_)) {
            return Err(SchemaError::InvalidField(
                "Collection fields cannot be updated".to_string(),
            ));
        }

        let aref_uuid = ctx.get_or_create_atom_ref()?;
        let prev_atom_uuid = ctx.get_prev_atom_uuid(&aref_uuid)?;

        ctx.create_and_update_atom(Some(prev_atom_uuid), content.clone(), None)?;

        info!(
            "update_field - schema: {}, field: {}, aref_uuid: {}, result: success",
            schema.name,
            field,
            aref_uuid
        );

        Ok(())
    }

    pub fn delete_field(
        &mut self,
        schema: &Schema,
        field: &str,
        source_pub_key: String,
    ) -> Result<(), SchemaError> {
        let mut ctx = AtomContext::new(schema, field, source_pub_key, &mut self.atom_manager);

        let field_def = ctx.get_field_def()?;
        if matches!(field_def, crate::schema::types::FieldVariant::Collection(_)) {
            return Err(SchemaError::InvalidField(
                "Collection fields cannot be deleted without id".to_string(),
            ));
        }

        let aref_uuid = ctx.get_or_create_atom_ref()?;
        let prev_atom_uuid = ctx.get_prev_atom_uuid(&aref_uuid)?;

        ctx.create_and_update_atom(Some(prev_atom_uuid), Value::Null, Some(AtomStatus::Deleted))?;

        info!(
            "delete_field - schema: {}, field: {}, aref_uuid: {}, result: success",
            schema.name,
            field,
            aref_uuid
        );

        Ok(())
    }

    fn default_value(field: &str) -> Value {
        match field {
            "username" | "email" | "full_name" | "bio" | "location" => {
                Value::String("".to_string())
            }
            "age" => Value::Number(serde_json::Number::from(0)),
            _ => Value::Null,
        }
    }
}

impl Clone for FieldManager {
    fn clone(&self) -> Self {
        Self {
            atom_manager: self.atom_manager.clone(),
            transform_manager: Arc::clone(&self.transform_manager),
            orchestrator: Arc::clone(&self.orchestrator),
        }
    }
}
