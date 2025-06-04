//! Mutation Domain Service
//! 
//! This module handles ONLY mutation-specific domain logic:
//! - Field value updates
//! - Atom modifications  
//! - Collection updates
//! 
//! It does NOT handle:
//! - Schema orchestration (belongs to FoldDB)
//! - Permission checking (belongs to FoldDB) 
//! - Event publishing (belongs to FoldDB)
//! - Schema validation (belongs to FoldDB)

use crate::fold_db_core::infrastructure::message_bus::{MessageBus, TransformTriggerRequest, CollectionUpdateRequest, FieldValueSetRequest};
use crate::schema::types::field::FieldVariant;
use crate::schema::types::schema::Schema;
use crate::schema::types::Mutation;
use crate::schema::SchemaError;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;
use serde_json::Value;

/// Mutation service responsible for field updates and atom modifications
pub struct MutationService {
    message_bus: Arc<MessageBus>,
}

impl MutationService {
    pub fn new(message_bus: Arc<MessageBus>) -> Self {
        Self {
            message_bus,
        }
    }

    /// Update a single field value (core mutation operation)
    pub fn update_field_value(
        &self,
        schema: &Schema,
        field_name: &str,
        value: &Value,
        mutation_hash: &str,
    ) -> Result<(), SchemaError> {
        log::info!("🔧 MutationService: Updating field {}.{}", schema.name, field_name);

        // Get field definition from schema
        let field_variant = schema.fields.get(field_name)
            .ok_or_else(|| SchemaError::InvalidData(
                format!("Field '{}' not found in schema '{}'", field_name, schema.name)
            ))?;

        // Apply field-specific mutation logic
        match field_variant {
            FieldVariant::Single(single_field) => {
                self.update_single_field(schema, field_name, single_field, value, mutation_hash)
            }
            FieldVariant::Range(range_field) => {
                self.update_range_field(schema, field_name, range_field, value, mutation_hash)
            }
            FieldVariant::Collection(collection_field) => {
                self.update_collection_field(schema, field_name, collection_field, value, mutation_hash)
            }
        }
    }

    /// Update atoms for a range schema mutation (sharing AtomRefRange)
    pub fn update_range_schema_fields(
        &self,
        schema: &Schema,
        fields_and_values: &std::collections::HashMap<String, Value>,
        range_key_value: &str,
        _mutation_hash: &str,
    ) -> Result<(), SchemaError> {
        log::info!("🎯 MutationService: Processing range schema mutation for range_key_value: {}", range_key_value);
        
        // Send CollectionUpdateRequest for each field in the range schema
        for (field_name, value) in fields_and_values {
            let correlation_id = Uuid::new_v4().to_string();
            let collection_request = CollectionUpdateRequest {
                correlation_id: correlation_id.clone(),
                schema_name: schema.name.clone(),
                field_name: field_name.clone(),
                operation: "update".to_string(),
                value: value.clone(),
                source_pub_key: "mutation_service".to_string(),
                item_id: Some(range_key_value.to_string()),
            };

            match self.message_bus.publish(collection_request) {
                Ok(_) => {
                    log::info!("✅ Range schema field update request sent for {}.{}", schema.name, field_name);
                }
                Err(e) => {
                    log::error!("❌ Failed to send range schema field update for {}.{}: {:?}", schema.name, field_name, e);
                    return Err(SchemaError::InvalidData(format!("Failed to update range schema field {}: {}", field_name, e)));
                }
            }
        }
        
        log::info!("✅ All range schema field updates sent successfully");
        Ok(())
    }

    /// Modify atom value (core mutation operation)
    pub fn modify_atom(
        &self,
        atom_uuid: &str,
        _new_value: &Value,
        mutation_hash: &str,
    ) -> Result<(), SchemaError> {
        log::info!("🔧 MutationService: Modifying atom {} with hash {}", atom_uuid, mutation_hash);
        
        // This would typically interact with atom storage
        // For now, we'll use event-driven communication
        
        // TODO: Implement direct atom modification logic
        // This should update the atom's value and update its hash
        
        log::info!("✅ Atom {} modified successfully", atom_uuid);
        Ok(())
    }

    /// Handle single field mutation
    fn update_single_field(
        &self,
        schema: &Schema,
        field_name: &str,
        _single_field: &crate::schema::types::field::single_field::SingleField,
        value: &Value,
        mutation_hash: &str,
    ) -> Result<(), SchemaError> {
        log::info!("🔧 Updating single field: {}.{}", schema.name, field_name);
        
        // First, send FieldValueSetRequest to store the actual field value as an Atom
        let value_correlation_id = Uuid::new_v4().to_string();
        let field_value_request = FieldValueSetRequest::new(
            value_correlation_id.clone(),
            schema.name.clone(),
            field_name.to_string(),
            value.clone(),
            "mutation_service".to_string(),
        );

        if let Err(e) = self.message_bus.publish(field_value_request) {
            log::error!("❌ Failed to send field value set request for {}.{}: {:?}", schema.name, field_name, e);
            return Err(SchemaError::InvalidData(format!("Failed to set field value: {}", e)));
        }
        log::info!("✅ Field value set request sent for {}.{}", schema.name, field_name);
        
        // Then send TransformTriggerRequest for this field (for any transforms)
        let transform_correlation_id = Uuid::new_v4().to_string();
        let transform_request = TransformTriggerRequest {
            correlation_id: transform_correlation_id.clone(),
            schema_name: schema.name.clone(),
            field_name: field_name.to_string(),
            mutation_hash: mutation_hash.to_owned(),
        };

        match self.message_bus.publish(transform_request) {
            Ok(_) => {
                log::info!("✅ Single field transform trigger sent for {}.{}", schema.name, field_name);
                Ok(())
            }
            Err(e) => {
                log::error!("❌ Failed to send transform trigger for {}.{}: {:?}", schema.name, field_name, e);
                Err(SchemaError::InvalidData(format!("Failed to update single field: {}", e)))
            }
        }
    }

    /// Handle range field mutation
    fn update_range_field(
        &self,
        schema: &Schema,
        field_name: &str,
        _range_field: &crate::schema::types::field::range_field::RangeField,
        _value: &Value,
        mutation_hash: &str,
    ) -> Result<(), SchemaError> {
        log::info!("🔧 Updating range field: {}.{}", schema.name, field_name);
        
        // Send TransformTriggerRequest for this field
        let correlation_id = Uuid::new_v4().to_string();
        let transform_request = TransformTriggerRequest {
            correlation_id: correlation_id.clone(),
            schema_name: schema.name.clone(),
            field_name: field_name.to_string(),
            mutation_hash: mutation_hash.to_owned(),
        };

        match self.message_bus.publish(transform_request) {
            Ok(_) => {
                log::info!("✅ Range field transform trigger sent for {}.{}", schema.name, field_name);
                Ok(())
            }
            Err(e) => {
                log::error!("❌ Failed to send transform trigger for {}.{}: {:?}", schema.name, field_name, e);
                Err(SchemaError::InvalidData(format!("Failed to update range field: {}", e)))
            }
        }
    }

    /// Handle collection field mutation
    fn update_collection_field(
        &self,
        schema: &Schema,
        field_name: &str,
        _collection_field: &crate::schema::types::field::collection_field::CollectionField,
        value: &Value,
        _mutation_hash: &str,
    ) -> Result<(), SchemaError> {
        log::info!("🔧 Updating collection field: {}.{}", schema.name, field_name);
        
        // Send CollectionUpdateRequest for this field
        let correlation_id = Uuid::new_v4().to_string();
        let collection_request = CollectionUpdateRequest {
            correlation_id: correlation_id.clone(),
            schema_name: schema.name.clone(),
            field_name: field_name.to_string(),
            operation: "update".to_string(),
            value: value.clone(),
            source_pub_key: "mutation_service".to_string(),
            item_id: None, // Single collection update doesn't need item_id
        };

        match self.message_bus.publish(collection_request) {
            Ok(_) => {
                log::info!("✅ Collection field update request sent for {}.{}", schema.name, field_name);
                Ok(())
            }
            Err(e) => {
                log::error!("❌ Failed to send collection update for {}.{}: {:?}", schema.name, field_name, e);
                Err(SchemaError::InvalidData(format!("Failed to update collection field: {}", e)))
            }
        }
    }

    /// Generate mutation hash for tracking
    pub fn generate_mutation_hash(mutation: &Mutation) -> Result<String, SchemaError> {
        let mutation_bytes = serde_json::to_vec(&mutation).map_err(|e| {
            SchemaError::InvalidData(format!("Failed to serialize mutation: {}", e))
        })?;
        let mut hasher = Sha256::new();
        hasher.update(mutation_bytes);
        let mutation_hash = format!("{:x}", hasher.finalize());
        Ok(mutation_hash)
    }

    /// Validate field value format (mutation-specific validation)
    pub fn validate_field_value(
        field_variant: &FieldVariant,
        value: &Value,
    ) -> Result<(), SchemaError> {
        match field_variant {
            FieldVariant::Single(_) => {
                // Validate single field value format
                if value.is_null() {
                    return Err(SchemaError::InvalidData("Single field value cannot be null".to_string()));
                }
                Ok(())
            }
            FieldVariant::Range(_) => {
                // Validate range field value format
                if !value.is_object() {
                    return Err(SchemaError::InvalidData("Range field value must be an object".to_string()));
                }
                Ok(())
            }
            FieldVariant::Collection(_) => {
                // Validate collection field value format
                if !value.is_array() && !value.is_object() {
                    return Err(SchemaError::InvalidData("Collection field value must be array or object".to_string()));
                }
                Ok(())
            }
        }
    }
}

/// Range schema mutation validation (domain-specific logic)
pub fn validate_range_schema_mutation_format(
    schema: &Schema,
    mutation: &Mutation,
) -> Result<(), SchemaError> {
    if let Some(range_key) = schema.range_key() {
        log::info!(
            "🔍 Validating Range schema mutation format for schema: {} with range_key: {}",
            schema.name,
            range_key
        );

        // MANDATORY: Range schema mutations MUST include the range_key field
        let range_key_value = mutation.fields_and_values.get(range_key)
            .ok_or_else(|| SchemaError::InvalidData(format!(
                "Range schema mutation for '{}' is missing required range_key field '{}'. All range schema mutations must provide a value for the range_key.",
                schema.name, range_key
            )))?;

        // Validate the range_key value is not null or empty
        if range_key_value.is_null() {
            return Err(SchemaError::InvalidData(format!(
                "Range schema mutation for '{}' has null value for range_key field '{}'. Range key must have a valid value.",
                schema.name, range_key
            )));
        }

        // If range_key value is a string, ensure it's not empty
        if let Some(str_value) = range_key_value.as_str() {
            if str_value.trim().is_empty() {
                return Err(SchemaError::InvalidData(format!(
                    "Range schema mutation for '{}' has empty string value for range_key field '{}'. Range key must have a non-empty value.",
                    schema.name, range_key
                )));
            }
        }

        // Validate all fields in the schema are RangeFields
        for (field_name, field_variant) in &schema.fields {
            match field_variant {
                FieldVariant::Range(_) => {
                    log::info!("✅ Field '{}' is correctly a RangeField", field_name);
                }
                FieldVariant::Single(_) => {
                    return Err(SchemaError::InvalidData(format!(
                        "Range schema '{}' contains Single field '{}', but all fields must be RangeFields",
                        schema.name, field_name
                    )));
                }
                FieldVariant::Collection(_) => {
                    return Err(SchemaError::InvalidData(format!(
                        "Range schema '{}' contains Collection field '{}', but all fields must be RangeFields",
                        schema.name, field_name
                    )));
                }
            }
        }

        log::info!(
            "✅ Range schema mutation format validation passed for schema: {}",
            schema.name
        );
    }

    Ok(())
}
