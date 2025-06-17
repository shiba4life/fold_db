//! JSON parsing, field conversion, and schema loading functionality
//!
//! This module contains the logic for:
//! - Parsing and validating JSON schemas
//! - Converting JSON field definitions to internal types
//! - Loading schemas from JSON strings and files
//! - Field mapping between schemas

use crate::atom::{AtomRef, AtomRefRange};
use crate::schema::core_types::SchemaCore;
use crate::schema::types::{
    Field, FieldVariant, JsonSchemaDefinition, JsonSchemaField, Schema, SchemaError, SingleField,
};
use log::info;
use std::collections::HashMap;
use uuid::Uuid;

/// Parse and validate JSON schema content
pub fn parse_and_validate_json_schema(
    schema_core: &SchemaCore,
    json_content: &str,
) -> Result<JsonSchemaDefinition, SchemaError> {
    let json_schema: JsonSchemaDefinition = serde_json::from_str(json_content)
        .map_err(|e| SchemaError::InvalidField(format!("Invalid JSON schema: {}", e)))?;

    let validator = super::validator::SchemaValidator::new(schema_core);
    validator.validate_json_schema(&json_schema)?;
    info!("JSON schema validation passed for '{}'", json_schema.name);

    Ok(json_schema)
}

/// Converts a JSON schema field to a FieldVariant.
pub fn convert_field(json_field: JsonSchemaField) -> FieldVariant {
    let mut single_field = SingleField::new(
        json_field.permission_policy.into(),
        json_field.payment_config.into(),
        json_field.field_mappers,
    );

    if let Some(ref_atom_uuid) = json_field.ref_atom_uuid {
        single_field.set_ref_atom_uuid(ref_atom_uuid);
    }

    // Add transform if present
    if let Some(json_transform) = json_field.transform {
        single_field.set_transform(json_transform.into());
    }

    // For now, we'll create all fields as Single fields
    // TODO: Handle Collection and Range field types based on json_field.field_type
    FieldVariant::Single(single_field)
}

/// Interprets a JSON schema definition and converts it to a Schema.
pub fn interpret_schema(
    schema_core: &SchemaCore,
    json_schema: JsonSchemaDefinition,
) -> Result<Schema, SchemaError> {
    // First validate the JSON schema
    super::validator::SchemaValidator::new(schema_core).validate_json_schema(&json_schema)?;

    // Convert fields
    let mut fields = HashMap::new();
    for (field_name, json_field) in json_schema.fields {
        fields.insert(field_name, convert_field(json_field));
    }

    // Create the schema
    Ok(Schema {
        name: json_schema.name,
        schema_type: json_schema.schema_type,
        fields,
        payment_config: json_schema.payment_config,
        hash: json_schema.hash,
    })
}

/// Interprets a JSON schema from a string and loads it as Available.
pub fn load_schema_from_json(
    schema_core: &SchemaCore,
    json_str: &str,
) -> Result<(), SchemaError> {
    info!(
        "Parsing JSON schema from string, length: {}",
        json_str.len()
    );
    let json_schema: JsonSchemaDefinition = serde_json::from_str(json_str)
        .map_err(|e| SchemaError::InvalidField(format!("Invalid JSON schema: {e}")))?;

    info!(
        "JSON schema parsed successfully, name: {}, fields: {:?}",
        json_schema.name,
        json_schema.fields.keys().collect::<Vec<_>>()
    );
    let schema = interpret_schema(schema_core, json_schema)?;
    info!(
        "Schema interpreted successfully, name: {}, fields: {:?}",
        schema.name,
        schema.fields.keys().collect::<Vec<_>>()
    );
    schema_core.load_schema_internal(schema)
}

/// Interprets a JSON schema from a file and loads it as Available.
pub fn load_schema_from_file(schema_core: &SchemaCore, path: &str) -> Result<(), SchemaError> {
    let json_str = std::fs::read_to_string(path)
        .map_err(|e| SchemaError::InvalidField(format!("Failed to read schema file: {e}")))?;

    info!(
        "Loading schema from file: {}, content length: {}",
        path,
        json_str.len()
    );
    load_schema_from_json(schema_core, &json_str)
}

/// Maps fields between schemas based on their defined relationships.
/// Returns a list of AtomRefs that need to be persisted in FoldDB.
pub fn map_fields(schema_core: &SchemaCore, schema_name: &str) -> Result<Vec<AtomRef>, SchemaError> {
    info!("🔧 Starting field mapping for schema '{}'", schema_name);

    let schemas = schema_core
        .schemas
        .lock()
        .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;

    // First collect all the source field ref_atom_uuids we need
    let mut field_mappings = Vec::new();
    if let Some(schema) = schemas.get(schema_name) {
        for (field_name, field) in &schema.fields {
            for (source_schema_name, source_field_name) in field.field_mappers() {
                if let Some(source_schema) = schemas.get(source_schema_name) {
                    if let Some(source_field) = source_schema.fields.get(source_field_name) {
                        if let Some(ref_atom_uuid) = source_field.ref_atom_uuid() {
                            field_mappings.push((field_name.clone(), ref_atom_uuid.clone()));
                        }
                    }
                }
            }
        }
    }
    drop(schemas); // Release the immutable lock

    // Now get a mutable lock to update the fields
    let mut schemas = schema_core
        .schemas
        .lock()
        .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;

    let schema = schemas
        .get_mut(schema_name)
        .ok_or_else(|| SchemaError::InvalidData(format!("Schema {schema_name} not found")))?;

    // Apply the collected mappings
    for (field_name, ref_atom_uuid) in field_mappings {
        if let Some(field) = schema.fields.get_mut(&field_name) {
            field.set_ref_atom_uuid(ref_atom_uuid);
        }
    }

    let mut atom_refs = Vec::new();

    // For unmapped fields, create a new ref_atom_uuid and AtomRef
    // Only create new ARefs for fields that truly don't have them (None or empty)
    for field in schema.fields.values_mut() {
        let needs_new_aref = match field.ref_atom_uuid() {
            None => true,
            Some(uuid) => uuid.is_empty(),
        };

        if needs_new_aref {
            let ref_atom_uuid = Uuid::new_v4().to_string();

            // Create and store the appropriate atom reference type based on field type
            let key = format!("ref:{}", ref_atom_uuid);

            match field {
                // TODO: Collection fields are no longer supported - CollectionField has been removed
                FieldVariant::Range(_) => {
                    // For range fields, create AtomRefRange
                    let atom_ref_range = AtomRefRange::new(ref_atom_uuid.clone());
                    if let Err(e) = schema_core.db_ops.store_item(&key, &atom_ref_range) {
                        info!("Failed to persist AtomRefRange '{}': {}", ref_atom_uuid, e);
                    } else {
                        info!("✅ Persisted AtomRefRange: {}", key);
                    }
                    // Create a corresponding AtomRef for the return list
                    atom_refs.push(AtomRef::new(
                        Uuid::new_v4().to_string(),
                        "system".to_string(),
                    ));
                }
                FieldVariant::Single(_) => {
                    // For single fields, create AtomRef
                    let atom_ref =
                        AtomRef::new(Uuid::new_v4().to_string(), "system".to_string());
                    if let Err(e) = schema_core.db_ops.store_item(&key, &atom_ref) {
                        info!("Failed to persist AtomRef '{}': {}", ref_atom_uuid, e);
                    } else {
                        info!("✅ Persisted AtomRef: {}", key);
                    }
                    atom_refs.push(atom_ref);
                }
            };

            // Set the ref_atom_uuid in the field - this will be used as the key to find the AtomRef
            field.set_ref_atom_uuid(ref_atom_uuid);
        }
    }

    // Persist the updated schema
    schema_core.persist_schema(schema)?;

    // Also update the available HashMap to keep it in sync
    let updated_schema = schema.clone();
    drop(schemas); // Release the schemas lock

    let mut available = schema_core
        .available
        .lock()
        .map_err(|_| SchemaError::InvalidData("Failed to acquire schema lock".to_string()))?;

    if let Some((_, state)) = available.get(schema_name) {
        let state = *state;
        available.insert(schema_name.to_string(), (updated_schema, state));
    }

    Ok(atom_refs)
}

impl SchemaCore {
    /// Interprets a JSON schema from a string and loads it as Available.
    pub fn load_schema_from_json(&self, json_str: &str) -> Result<(), SchemaError> {
        load_schema_from_json(self, json_str)
    }

    /// Interprets a JSON schema from a file and loads it as Available.
    pub fn load_schema_from_file(&self, path: &str) -> Result<(), SchemaError> {
        load_schema_from_file(self, path)
    }

    /// Maps fields between schemas based on their defined relationships.
    /// Returns a list of AtomRefs that need to be persisted in FoldDB.
    pub fn map_fields(&self, schema_name: &str) -> Result<Vec<AtomRef>, SchemaError> {
        map_fields(self, schema_name)
    }

    /// Interprets a JSON schema definition and converts it to a Schema.
    pub fn interpret_schema(&self, json_schema: JsonSchemaDefinition) -> Result<Schema, SchemaError> {
        interpret_schema(self, json_schema)
    }
}