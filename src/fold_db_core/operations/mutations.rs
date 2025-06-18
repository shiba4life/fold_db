//! Mutation processing logic
//!
//! This module handles event-driven mutation processing, validation,
//! and execution. It manages state changes and coordinates with the
//! mutation service for field updates.

use crate::fold_db_core::infrastructure::message_bus::{MutationExecuted, MessageBus};
use crate::fold_db_core::services;
use crate::permissions::PermissionWrapper;
use crate::schema::core::SchemaCore;
use crate::schema::types::Mutation;
use crate::schema::{Schema, SchemaError};
use log::warn;
use serde_json::Value;
use sha2::Digest;
use std::sync::Arc;
use std::time::Instant;

/// Mutation operations coordinator
pub struct MutationOperations {
    schema_manager: Arc<SchemaCore>,
    permission_wrapper: PermissionWrapper,
    message_bus: Arc<MessageBus>,
}

impl MutationOperations {
    /// Create a new mutation operations coordinator
    pub fn new(
        schema_manager: Arc<SchemaCore>,
        permission_wrapper: PermissionWrapper,
        message_bus: Arc<MessageBus>,
    ) -> Self {
        Self {
            schema_manager,
            permission_wrapper,
            message_bus,
        }
    }

    /// Write schema operation - main orchestration method for mutations
    pub fn write_schema(&mut self, mutation: Mutation) -> Result<(), SchemaError> {
        let start_time = Instant::now();
        let fields_count = mutation.fields_and_values.len();
        let operation_type = format!("{:?}", mutation.mutation_type);
        let schema_name = mutation.schema_name.clone();

        log::info!(
            "Starting mutation execution for schema: {}",
            mutation.schema_name
        );
        log::info!("Mutation type: {:?}", mutation.mutation_type);
        log::info!(
            "Fields to mutate: {:?}",
            mutation.fields_and_values.keys().collect::<Vec<_>>()
        );

        if mutation.fields_and_values.is_empty() {
            return Err(SchemaError::InvalidData("No fields to write".to_string()));
        }

        // 1. Prepare mutation and validate schema
        let (schema, processed_mutation, mutation_hash) =
            self.prepare_mutation_and_schema(mutation)?;

        // 2. Create mutation service and delegate field updates
        let mutation_service =
            services::mutation::MutationService::new(Arc::clone(&self.message_bus));
        let result = self.process_field_mutations_via_service(
            &mutation_service,
            &schema,
            &processed_mutation,
            &mutation_hash,
        );

        // 3. Publish MutationExecuted event
        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        let mutation_event =
            MutationExecuted::new(operation_type, schema_name, execution_time_ms, fields_count);

        if let Err(e) = self.message_bus.publish(mutation_event) {
            warn!("Failed to publish MutationExecuted event: {}", e);
        }

        result
    }

    /// Prepare mutation and schema - extract and validate components
    fn prepare_mutation_and_schema(
        &self,
        mutation: Mutation,
    ) -> Result<(Schema, Mutation, String), SchemaError> {
        // Get schema
        let schema = match self.schema_manager.get_schema(&mutation.schema_name)? {
            Some(schema) => schema,
            None => {
                return Err(SchemaError::InvalidData(format!(
                    "Schema '{}' not found",
                    mutation.schema_name
                )));
            }
        };

        // Check field-level permissions for each field in the mutation
        for field_name in mutation.fields_and_values.keys() {
            let permission_result = self.permission_wrapper.check_mutation_field_permission(
                &mutation,
                field_name,
                &self.schema_manager,
            );

            if !permission_result.allowed {
                return Err(permission_result.error.unwrap_or_else(|| {
                    SchemaError::InvalidData(format!(
                        "Permission denied for field '{}' in schema '{}' with trust distance {}",
                        field_name, mutation.schema_name, mutation.trust_distance
                    ))
                }));
            }
        }

        // Generate mutation hash for tracking
        let mut hasher = <sha2::Sha256 as Digest>::new();
        hasher.update(format!("{:?}", mutation).as_bytes());
        let mutation_hash = format!("{:x}", hasher.finalize());

        Ok((schema, mutation, mutation_hash))
    }

    /// Process field mutations via service delegation
    fn process_field_mutations_via_service(
        &self,
        mutation_service: &services::mutation::MutationService,
        schema: &Schema,
        mutation: &Mutation,
        mutation_hash: &str,
    ) -> Result<(), SchemaError> {
        // Check if this is a range schema
        if let Some(range_key) = schema.range_key() {
            log::info!(
                "🎯 DEBUG: Processing range schema mutation for schema '{}' with range_key '{}'",
                schema.name,
                range_key
            );

            // Extract the range key value from the mutation data
            let range_key_value = mutation.fields_and_values.get(range_key).ok_or_else(|| {
                SchemaError::InvalidData(format!(
                    "Range schema mutation missing range_key field '{}'",
                    range_key
                ))
            })?;

            let range_key_str = match range_key_value {
                Value::String(s) => s.clone(),
                _ => range_key_value.to_string().trim_matches('"').to_string(),
            };

            log::info!("🎯 DEBUG: Range key value: '{}'", range_key_str);

            // Use the specialized range schema mutation method
            return mutation_service.update_range_schema_fields(
                schema,
                &mutation.fields_and_values,
                &range_key_str,
                mutation_hash,
            );
        } else {
            log::info!(
                "🔍 DEBUG: Processing regular schema mutation for schema '{}'",
                schema.name
            );
        }

        // For non-range schemas, process fields individually
        for (field_name, field_value) in &mutation.fields_and_values {
            // Get field definition
            let _field = schema.fields.get(field_name).ok_or_else(|| {
                SchemaError::InvalidData(format!("Field '{}' not found in schema", field_name))
            })?;

            // Delegate to mutation service
            mutation_service.update_field_value(
                schema,
                field_name.as_str(),
                field_value,
                mutation_hash,
            )?;
        }

        Ok(())
    }

    /// Validate mutation against schema constraints
    pub fn validate_mutation(&self, mutation: &Mutation) -> Result<(), SchemaError> {
        // Get schema first
        let schema = match self.schema_manager.get_schema(&mutation.schema_name)? {
            Some(schema) => schema,
            None => {
                return Err(SchemaError::InvalidData(format!(
                    "Schema '{}' not found",
                    mutation.schema_name
                )));
            }
        };

        // Validate range schema format if applicable
        if schema.range_key().is_some() {
            services::mutation::validate_range_schema_mutation_format(&schema, mutation)?;
        }

        // Validate individual field values
        for (field_name, field_value) in &mutation.fields_and_values {
            if let Some(field_variant) = schema.fields.get(field_name) {
                services::mutation::MutationService::validate_field_value(
                    field_variant,
                    field_value,
                )?;
            } else {
                return Err(SchemaError::InvalidData(format!(
                    "Field '{}' not found in schema '{}'",
                    field_name, schema.name
                )));
            }
        }

        Ok(())
    }

    /// Check permissions for mutation operation
    pub fn check_mutation_permissions(&self, mutation: &Mutation) -> Result<(), SchemaError> {
        for field_name in mutation.fields_and_values.keys() {
            let permission_result = self.permission_wrapper.check_mutation_field_permission(
                mutation,
                field_name,
                &self.schema_manager,
            );

            if !permission_result.allowed {
                return Err(permission_result.error.unwrap_or_else(|| {
                    SchemaError::InvalidData(format!(
                        "Permission denied for field '{}' in schema '{}' with trust distance {}",
                        field_name, mutation.schema_name, mutation.trust_distance
                    ))
                }));
            }
        }
        Ok(())
    }

    /// Generate mutation hash for tracking and deduplication
    pub fn generate_mutation_hash(&self, mutation: &Mutation) -> String {
        let mut hasher = <sha2::Sha256 as Digest>::new();
        hasher.update(format!("{:?}", mutation).as_bytes());
        format!("{:x}", hasher.finalize())
    }
}