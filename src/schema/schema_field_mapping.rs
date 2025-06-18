//! Schema field mapping operations
//!
//! This module handles field mapping between schemas including:
//! - Field mapping between schemas
//! - Atom reference creation and management
//! - Cross-schema field relationships

use crate::schema::core_types::{SchemaCore, SchemaState};
use crate::schema::types::{Field, SchemaError};
use log::info;

impl SchemaCore {
    /// Map fields for a schema and ensure proper atom references are assigned
    /// This is a wrapper around the parsing module's map_fields function
    pub fn map_schema_fields(&self, schema_name: &str) -> Result<Vec<String>, SchemaError> {
        info!("Mapping fields for schema '{}'", schema_name);
        
        // Delegate to the parsing module's map_fields function
        match super::parsing::map_fields(self, schema_name) {
            Ok(atom_refs) => {
                info!(
                    "Field mapping successful for schema '{}': created {} atom references",
                    schema_name,
                    atom_refs.len()
                );
                // Return count as string representation for now
                Ok(vec![format!("Created {} atom references", atom_refs.len())])
            }
            Err(e) => {
                info!("Field mapping failed for schema '{}': {}", schema_name, e);
                Err(e)
            }
        }
    }

    /// Check if a schema has properly mapped fields (all fields have atom references)
    pub fn validate_field_mappings(&self, schema_name: &str) -> Result<bool, SchemaError> {
        let schema = self.get_schema(schema_name)?
            .ok_or_else(|| SchemaError::NotFound(format!("Schema '{}' not found", schema_name)))?;

        let all_mapped = schema.fields.values().all(|field| field.ref_atom_uuid().is_some());
        
        info!(
            "Schema '{}' field mapping validation: {}",
            schema_name,
            if all_mapped { "all fields mapped" } else { "some fields unmapped" }
        );

        Ok(all_mapped)
    }

    /// Get field mapping statistics for a schema
    pub fn get_field_mapping_stats(&self, schema_name: &str) -> Result<(usize, usize), SchemaError> {
        let schema = self.get_schema(schema_name)?
            .ok_or_else(|| SchemaError::NotFound(format!("Schema '{}' not found", schema_name)))?;

        let total_fields = schema.fields.len();
        let mapped_fields = schema.fields.values()
            .filter(|field| field.ref_atom_uuid().is_some())
            .count();

        info!(
            "Schema '{}' field mapping stats: {}/{} fields mapped",
            schema_name,
            mapped_fields,
            total_fields
        );

        Ok((mapped_fields, total_fields))
    }

    /// Remap fields for a schema (useful after schema updates)
    pub fn remap_schema_fields(&self, schema_name: &str) -> Result<(), SchemaError> {
        info!("Remapping fields for schema '{}'", schema_name);

        // Check if schema is approved by looking in available schemas
        let is_approved = {
            let available = self.available.lock().map_err(|_| {
                SchemaError::InvalidData("Failed to acquire schema lock".to_string())
            })?;
            available.get(schema_name)
                .map(|(_, state)| *state == SchemaState::Approved)
                .unwrap_or(false)
        };

        if is_approved {
            self.ensure_approved_schema_in_schemas(schema_name)?;
        }

        // Perform the field mapping
        self.map_schema_fields(schema_name)?;

        // Persist the updated schema with field assignments
        if let Ok(Some(updated_schema)) = self.get_schema(schema_name) {
            self.persist_schema(&updated_schema)?;
            info!("Schema '{}' field remapping completed and persisted", schema_name);
        }

        Ok(())
    }

    /// Get schemas that have unmapped fields
    pub fn get_schemas_with_unmapped_fields(&self) -> Result<Vec<String>, SchemaError> {
        let mut schemas_with_unmapped = Vec::new();

        let available = self.available.lock().map_err(|_| {
            SchemaError::InvalidData("Failed to acquire schema lock".to_string())
        })?;

        for (schema_name, (schema, _)) in available.iter() {
            let has_unmapped = schema.fields.values().any(|field| field.ref_atom_uuid().is_none());
            if has_unmapped {
                schemas_with_unmapped.push(schema_name.clone());
            }
        }

        info!(
            "Found {} schemas with unmapped fields: {:?}",
            schemas_with_unmapped.len(),
            schemas_with_unmapped
        );

        Ok(schemas_with_unmapped)
    }

    /// Validate field mappings across all schemas
    pub fn validate_all_field_mappings(&self) -> Result<Vec<(String, bool)>, SchemaError> {
        let mut validation_results = Vec::new();

        let available = self.available.lock().map_err(|_| {
            SchemaError::InvalidData("Failed to acquire schema lock".to_string())
        })?;

        for (schema_name, _) in available.iter() {
            let is_valid = self.validate_field_mappings(schema_name)?;
            validation_results.push((schema_name.clone(), is_valid));
        }

        let invalid_count = validation_results.iter().filter(|(_, valid)| !valid).count();
        info!(
            "Field mapping validation completed: {}/{} schemas have valid mappings",
            validation_results.len() - invalid_count,
            validation_results.len()
        );

        Ok(validation_results)
    }
}