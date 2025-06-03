use super::FoldDB;
use crate::schema::types::field::common::Field;
use crate::schema::types::field::FieldVariant;
use crate::schema::types::schema::Schema;
use crate::schema::types::{Mutation, MutationType};
use crate::schema::SchemaError;
use sha2::{Digest, Sha256};

impl FoldDB {
    pub fn write_schema(&mut self, mutation: Mutation) -> Result<(), SchemaError> {
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

        let (schema, mutation, mutation_hash) = self.prepare_mutation_and_schema(mutation)?;

        self.process_field_mutations(&schema, &mutation, &mutation_hash)
    }

    fn prepare_mutation_and_schema(
        &self,
        mutation: Mutation,
    ) -> Result<(Schema, Mutation, String), SchemaError> {
        let mutation_bytes = serde_json::to_vec(&mutation).map_err(|e| {
            SchemaError::InvalidData(format!("Failed to serialize mutation: {}", e))
        })?;
        let mut hasher = Sha256::new();
        hasher.update(mutation_bytes);
        let mutation_hash = format!("{:x}", hasher.finalize());
        log::info!("Generated mutation hash: {}", mutation_hash);

        let schema = self
            .schema_manager
            .get_schema(&mutation.schema_name)?
            .ok_or_else(|| {
                SchemaError::NotFound(format!("Schema {} not found", mutation.schema_name))
            })?;
        log::info!(
            "Retrieved schema: {} with {} fields",
            schema.name,
            schema.fields.len()
        );

        // Validate Range schema mutation before processing
        self.validate_range_schema_mutation(&schema, &mutation)?;

        // Convert mutation to a range_schema_mutation if needed
        let mutation = if schema.range_key().is_some() {
            mutation.to_range_schema_mutation(&schema)?
        } else {
            mutation
        };

        Ok((schema, mutation, mutation_hash))
    }

    /// Validates Range schema mutations to ensure:
    /// - Range schema mutations MUST include a range_key field
    /// - All fields in Range schemas are RangeFields
    /// - All fields have consistent range_key values
    fn validate_range_schema_mutation(
        &self,
        schema: &Schema,
        mutation: &Mutation,
    ) -> Result<(), SchemaError> {
        if let Some(range_key) = schema.range_key() {
            log::info!(
                "🔍 Validating Range schema mutation for schema: {} with range_key: {}",
                schema.name,
                range_key
            );

            // MANDATORY: Range schema mutations MUST include the range_key field
            let range_key_value = mutation.fields_and_values.get(range_key)
                .ok_or_else(|| SchemaError::InvalidData(format!(
                    "Range schema mutation for '{}' is missing required range_key field '{}'. All range schema mutations must provide a value for the range_key.",
                    schema.name, range_key
                )))?;

            log::info!(
                "✅ Range schema mutation contains required range_key '{}' with value: {:?}",
                range_key,
                range_key_value
            );

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

            // Validate all mutation field values have consistent range_key
            for (field_name, field_value) in &mutation.fields_and_values {
                if field_name == range_key {
                    // Skip validation for the range_key field itself
                    continue;
                }

                // Check if field value is an object and contains the range_key
                if let Some(value_obj) = field_value.as_object() {
                    if let Some(field_range_value) = value_obj.get(range_key) {
                        if field_range_value != range_key_value {
                            return Err(SchemaError::InvalidData(format!(
                                "Inconsistent range_key value in field '{}': expected {:?}, got {:?}",
                                field_name, range_key_value, field_range_value
                            )));
                        }
                    } else {
                        log::info!("⚠️ Field '{}' does not contain range_key '{}' - this will be added by to_range_schema_mutation",
                                 field_name, range_key);
                    }
                }
            }

            log::info!(
                "✅ Range schema mutation validation passed for schema: {}",
                schema.name
            );
        }

        Ok(())
    }

    fn process_field_mutations(
        &mut self,
        schema: &Schema,
        mutation: &Mutation,
        mutation_hash: &str,
    ) -> Result<(), SchemaError> {
        // CRITICAL FIX: For range schemas, process all fields together to share AtomRefRange
        if schema.range_key().is_some() {
            self.process_range_schema_mutation(schema, mutation, mutation_hash)
        } else {
            self.process_regular_schema_mutation(schema, mutation, mutation_hash)
        }
    }

    fn process_regular_schema_mutation(
        &mut self,
        schema: &Schema,
        mutation: &Mutation,
        mutation_hash: &str,
    ) -> Result<(), SchemaError> {
        for (field_name, value) in mutation.fields_and_values.iter() {
            log::info!("Processing field mutation: {} = {:?}", field_name, value);

            let perm = self.permission_wrapper.check_mutation_field_permission(
                mutation,
                field_name,
                &self.schema_manager,
            );
            log::info!(
                "Permission check for field {}: allowed={}",
                field_name,
                perm.allowed
            );

            if mutation.trust_distance != 0 && !perm.allowed {
                log::error!(
                    "Permission denied for field {} with trust_distance {}",
                    field_name,
                    mutation.trust_distance
                );
                return Err(perm.error.unwrap_or(SchemaError::InvalidPermission(
                    "Unknown permission error".to_string(),
                )));
            }

            // Get current schema to check if atom_refs exist
            let current_schema = self.schema_manager.get_schema(&mutation.schema_name)?
                .ok_or_else(|| SchemaError::NotFound(format!("Schema {} not found", mutation.schema_name)))?;

            match mutation.mutation_type {
                MutationType::Create => {
                    log::info!(
                        "🔧 Executing CREATE mutation for field: {}.{} = {:?}",
                        mutation.schema_name,
                        field_name,
                        value
                    );
                    
                    // Check if atom_ref already exists for this field
                    let field_def = current_schema.fields.get(field_name)
                        .ok_or_else(|| SchemaError::InvalidField(format!("Field {} not found", field_name)))?;
                    
                    let mut working_schema = current_schema.clone();
                    let ref_atom_uuid = if field_def.ref_atom_uuid().is_none() {
                        // First mutation - create atom_ref (normal behavior)
                        log::info!("🆕 First mutation for field {}.{} - creating atom_ref", mutation.schema_name, field_name);
                        self.field_manager.set_field_value(
                            &mut working_schema,
                            field_name,
                            value.clone(),
                            mutation.pub_key.clone(),
                        )?
                    } else {
                        // Subsequent mutation - use existing atom_ref only (prevent fragmentation)
                        log::info!("🔄 Subsequent mutation for field {}.{} - using existing atom_ref", mutation.schema_name, field_name);
                        self.field_manager.update_field_existing_atom_ref(
                            &mut working_schema,
                            field_name,
                            value.clone(),
                            mutation.pub_key.clone(),
                        )?
                    };

                    self.schema_manager.update_field_ref_atom_uuid(
                        &mutation.schema_name,
                        field_name,
                        ref_atom_uuid.clone(),
                    )?;

                    log::info!(
                        "✅ Regular schema field {} processed with existing ref_atom_uuid: {}",
                        field_name,
                        ref_atom_uuid
                    );
                }
                MutationType::Update => {
                    log::info!(
                        "🔄 Executing UPDATE mutation for field: {}.{}",
                        mutation.schema_name,
                        field_name
                    );

                    // Check if atom_ref exists for this field
                    let field_def = current_schema.fields.get(field_name)
                        .ok_or_else(|| SchemaError::InvalidField(format!("Field {} not found", field_name)))?;
                    
                    let mut working_schema = current_schema.clone();
                    let ref_atom_uuid = if field_def.ref_atom_uuid().is_none() {
                        // First mutation (UPDATE can be first mutation too) - create atom_ref
                        log::info!("🆕 First update for field {}.{} - creating atom_ref", mutation.schema_name, field_name);
                        self.field_manager.set_field_value(
                            &mut working_schema,
                            field_name,
                            value.clone(),
                            mutation.pub_key.clone(),
                        )?
                    } else {
                        // Subsequent mutation - use existing atom_ref only
                        log::info!("🔄 Subsequent update for field {}.{} - using existing atom_ref", mutation.schema_name, field_name);
                        self.field_manager.update_field_existing_atom_ref(
                            &mut working_schema,
                            field_name,
                            value.clone(),
                            mutation.pub_key.clone(),
                        )?
                    };

                    log::info!(
                        "✅ Field updated successfully for: {}.{} with ref_atom_uuid: {}",
                        mutation.schema_name,
                        field_name,
                        ref_atom_uuid
                    );

                    // Update the schema manager with the ref_atom_uuid returned from update_field
                    log::info!(
                        "💾 Updating schema manager with ref_atom_uuid: {} for field: {}.{}",
                        ref_atom_uuid,
                        mutation.schema_name,
                        field_name
                    );
                    self.schema_manager.update_field_ref_atom_uuid(
                        &mutation.schema_name,
                        field_name,
                        ref_atom_uuid.clone(),
                    )?;
                    log::info!(
                        "✅ Schema manager updated successfully for {}.{} with ref_atom_uuid: {}",
                        mutation.schema_name,
                        field_name,
                        ref_atom_uuid
                    );
                }
                MutationType::Delete => {
                    let mut schema_clone = schema.clone();
                    self.field_manager.delete_field(
                        &mut schema_clone,
                        field_name,
                        mutation.pub_key.clone(),
                    )?;

                    if let Some(field_def) = schema_clone.fields.get(field_name) {
                        if let Some(ref_atom_uuid) = field_def.ref_atom_uuid() {
                            self.schema_manager.update_field_ref_atom_uuid(
                                &mutation.schema_name,
                                field_name,
                                ref_atom_uuid.clone(),
                            )?;
                        }
                    }
                }
                MutationType::AddToCollection(ref id) => {
                    self.collection_manager.add_collection_field_value(
                        schema,
                        field_name,
                        value.clone(),
                        mutation.pub_key.clone(),
                        id.clone(),
                    )?;
                }
                MutationType::UpdateToCollection(ref id) => {
                    self.collection_manager.update_collection_field_value(
                        schema,
                        field_name,
                        value.clone(),
                        mutation.pub_key.clone(),
                        id.clone(),
                    )?;
                }
                MutationType::DeleteFromCollection(ref id) => {
                    self.collection_manager.delete_collection_field_value(
                        schema,
                        field_name,
                        mutation.pub_key.clone(),
                        id.clone(),
                    )?;
                }
            }

            // Add transform orchestrator task for this field
            let field_key = format!("{}.{}", schema.name, field_name);
            log::info!(
                "Adding transform orchestrator task for field: {}",
                field_key
            );
            log::info!(
                "Transform orchestrator task details - schema: {}, field: {}, mutation_hash: {}",
                schema.name,
                field_name,
                mutation_hash
            );

            let result =
                self.transform_orchestrator
                    .add_task(&schema.name, field_name, mutation_hash);

            match result {
                Ok(_) => log::info!(
                    "Transform orchestrator task added successfully for field: {}",
                    field_key
                ),
                Err(e) => log::error!(
                    "Failed to add transform orchestrator task for field {}: {:?}",
                    field_key,
                    e
                ),
            }
        }

        log::info!(
            "Mutation execution completed successfully for schema: {}",
            mutation.schema_name
        );
        log::info!(
            "Total fields processed: {}",
            mutation.fields_and_values.len()
        );

        // Process the transform queue to execute any queued transforms
        log::info!("Processing transform queue after mutation execution");
        self.transform_orchestrator.process_queue();
        log::info!("Transform queue processing completed");

        Ok(())
    }

    fn process_range_schema_mutation(
        &mut self,
        schema: &Schema,
        mutation: &Mutation,
        mutation_hash: &str,
    ) -> Result<(), SchemaError> {
        log::info!("🎯 Processing range schema mutation - mutations only update existing atom_refs");
        
        // Get the range_key and its value
        let range_key = schema.range_key().unwrap();
        let range_key_value = mutation.fields_and_values.get(range_key)
            .ok_or_else(|| SchemaError::InvalidData(format!("Range key '{}' not found in mutation", range_key)))?;
        
        // Convert range_key value to string for AtomRefRange key
        let range_key_str = match range_key_value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            _ => serde_json::to_string(range_key_value)
                .map_err(|e| SchemaError::InvalidData(e.to_string()))?
                .trim_matches('"')
                .to_string(),
        };
        
        log::info!("🔑 Processing range schema for range_key_value: {}", range_key_str);
        
        // Get current schema state to find existing AtomRefRange UUID
        let current_schema = self.schema_manager.get_schema(&mutation.schema_name)?
            .ok_or_else(|| SchemaError::NotFound(format!("Schema {} not found", mutation.schema_name)))?;
            
        // Find existing AtomRefRange UUID from any field (they should all share the same one)
        let shared_aref_uuid: Option<String> = current_schema.fields.iter()
            .find_map(|(field_name, field)| {
                if let Some(ref_atom_uuid) = field.ref_atom_uuid() {
                    log::info!("📋 Found existing AtomRefRange UUID from field {}: {}", field_name, ref_atom_uuid);
                    Some(ref_atom_uuid.clone())
                } else {
                    None
                }
            });
        
        // Ensure atom_refs exist for range schema mutations
        if shared_aref_uuid.is_none() {
            return Err(SchemaError::InvalidData(format!(
                "No existing atom_refs found for range schema {}. Atom_refs must be created during field creation.",
                mutation.schema_name
            )));
        }
        
        // Process each field using the shared AtomRefRange
        for (field_name, value) in mutation.fields_and_values.iter() {
            log::info!("🔧 Processing range schema field: {} = {:?}", field_name, value);

            // Permission check
            let perm = self.permission_wrapper.check_mutation_field_permission(
                mutation,
                field_name,
                &self.schema_manager,
            );

            if mutation.trust_distance != 0 && !perm.allowed {
                return Err(perm.error.unwrap_or(SchemaError::InvalidPermission(
                    "Unknown permission error".to_string(),
                )));
            }

            // Use current schema state (not clone) to avoid ghost UUIDs
            let mut working_schema = self.schema_manager.get_schema(&mutation.schema_name)?
                .ok_or_else(|| SchemaError::NotFound(format!("Schema {} not found", mutation.schema_name)))?;

            match mutation.mutation_type {
                MutationType::Create | MutationType::Update => {
                    let action = if matches!(mutation.mutation_type, MutationType::Create) { "CREATE" } else { "UPDATE" };
                    log::info!("🔧 Executing {} mutation for range field: {}.{}", action, mutation.schema_name, field_name);

                    let ref_atom_uuid = self.field_manager.update_field_existing_atom_ref(
                        &mut working_schema,
                        field_name,
                        value.clone(),
                        mutation.pub_key.clone(),
                    )?;

                    // Store a copy for logging before moving
                    let ref_atom_uuid_for_log = ref_atom_uuid.clone();

                    // Update schema manager with the ref_atom_uuid
                    self.schema_manager.update_field_ref_atom_uuid(
                        &mutation.schema_name,
                        field_name,
                        ref_atom_uuid,
                    )?;

                    log::info!("✅ Range schema field {} {} with ref_atom_uuid: {}", field_name, action.to_lowercase(), ref_atom_uuid_for_log);
                }
                MutationType::Delete => {
                    // For range schemas, delete operations should remove entries from the shared AtomRefRange
                    log::info!("🗑️ Delete mutation for range schema field: {}", field_name);
                    // Implementation would depend on specific delete semantics
                }
                _ => {
                    return Err(SchemaError::InvalidData(format!(
                        "Unsupported mutation type {:?} for range schema field {}",
                        mutation.mutation_type, field_name
                    )));
                }
            }

            // Add transform orchestrator task for this field
            let field_key = format!("{}.{}", schema.name, field_name);
            log::info!(
                "Adding transform orchestrator task for field: {}",
                field_key
            );

            let result = self.transform_orchestrator.add_task(&schema.name, field_name, mutation_hash);

            match result {
                Ok(_) => log::info!(
                    "Transform orchestrator task added successfully for field: {}",
                    field_key
                ),
                Err(e) => log::error!(
                    "Failed to add transform orchestrator task for field {}: {:?}",
                    field_key,
                    e
                ),
            }
        }
        
        log::info!(
            "Range schema mutation execution completed successfully for schema: {}",
            mutation.schema_name
        );

        // Process the transform queue to execute any queued transforms
        log::info!("Processing transform queue after range schema mutation execution");
        self.transform_orchestrator.process_queue();
        log::info!("Transform queue processing completed");
        
        Ok(())
    }
}
