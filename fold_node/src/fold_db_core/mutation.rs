use super::FoldDB;
use crate::schema::types::{Mutation, MutationType};
use crate::schema::SchemaError;
use crate::schema::types::field::common::Field;
use crate::schema::types::field::FieldVariant;
use crate::schema::types::schema::Schema;
use sha2::{Digest, Sha256};

impl FoldDB {
    pub fn write_schema(&mut self, mutation: Mutation) -> Result<(), SchemaError> {
        log::info!("Starting mutation execution for schema: {}", mutation.schema_name);
        log::info!("Mutation type: {:?}", mutation.mutation_type);
        log::info!("Fields to mutate: {:?}", mutation.fields_and_values.keys().collect::<Vec<_>>());

        if mutation.fields_and_values.is_empty() {
            return Err(SchemaError::InvalidData("No fields to write".to_string()));
        }

        let (schema, mutation, mutation_hash) = self.prepare_mutation_and_schema(mutation)?;
        
        self.process_field_mutations(&schema, &mutation, &mutation_hash)
    }

    fn prepare_mutation_and_schema(&self, mutation: Mutation) -> Result<(Schema, Mutation, String), SchemaError> {
        let mutation_bytes = serde_json::to_vec(&mutation)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize mutation: {}", e)))?;
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
            log::info!("Retrieved schema: {} with {} fields", schema.name, schema.fields.len());
        
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
    /// - All fields in Range schemas are RangeFields
    /// - All fields have consistent range_key values
    fn validate_range_schema_mutation(&self, schema: &Schema, mutation: &Mutation) -> Result<(), SchemaError> {
        if let Some(range_key) = schema.range_key() {
            log::info!("🔍 Validating Range schema mutation for schema: {} with range_key: {}", schema.name, range_key);
            
            // Get the range_key value from mutation
            let range_key_value = mutation.fields_and_values.get(range_key)
                .ok_or_else(|| SchemaError::InvalidData(format!(
                    "Range schema mutation missing range_key field '{}'", range_key
                )))?;
            
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
            
            log::info!("✅ Range schema mutation validation passed for schema: {}", schema.name);
        }
        
        Ok(())
    }

    fn process_field_mutations(&mut self, schema: &Schema, mutation: &Mutation, mutation_hash: &str) -> Result<(), SchemaError> {
        for (field_name, value) in mutation.fields_and_values.iter() {
            log::info!("Processing field mutation: {} = {:?}", field_name, value);
    
            let perm = self.permission_wrapper.check_mutation_field_permission(
                mutation,
                field_name,
                &self.schema_manager,
            );
            log::info!("Permission check for field {}: allowed={}", field_name, perm.allowed);
    
            if mutation.trust_distance != 0 && !perm.allowed {
                log::error!("Permission denied for field {} with trust_distance {}", field_name, mutation.trust_distance);
                return Err(perm.error.unwrap_or(SchemaError::InvalidPermission(
                    "Unknown permission error".to_string(),
                )));
            }
    
            match mutation.mutation_type {
                MutationType::Create => {
                    log::info!("🔧 Executing CREATE mutation for field: {}.{} = {:?}", mutation.schema_name, field_name, value);
                    let mut schema_clone = schema.clone();
    
                    // Check ref_atom_uuid before setting value
                    let before_ref_uuid = schema_clone.fields.get(field_name)
                        .and_then(|f| f.ref_atom_uuid())
                        .map(|uuid| uuid.to_string());
                    log::info!("🔍 ref_atom_uuid BEFORE set_field_value for {}.{}: {:?}",
                              mutation.schema_name, field_name, before_ref_uuid);
    
                    // CRITICAL: Proper ref_atom_uuid Management Pattern
                    // 1. set_field_value creates AtomRef and returns UUID (does NOT set on field)
                    // 2. We use schema_manager to set and persist the UUID on the actual schema
                    // 3. This prevents "ghost ref_atom_uuid" where UUID exists but AtomRef doesn't
                    let ref_atom_uuid = self.field_manager.set_field_value(
                        &mut schema_clone,
                        field_name,
                        value.clone(),
                        mutation.pub_key.clone(),
                    )?;
                    log::info!("✅ Field value set successfully for: {}.{} with ref_atom_uuid: {}",
                              mutation.schema_name, field_name, ref_atom_uuid);
    
                    // Now update the schema manager with the ref_atom_uuid returned from set_field_value
                    // This is the ONLY place where ref_atom_uuid should be set on field definitions
                    log::info!("💾 Updating schema manager with ref_atom_uuid: {} for field: {}.{}",
                              ref_atom_uuid, mutation.schema_name, field_name);
                    self.schema_manager.update_field_ref_atom_uuid(
                        &mutation.schema_name,
                        field_name,
                        ref_atom_uuid.clone(),
                    )?;
                    log::info!("✅ Schema manager updated successfully for {}.{}",
                               mutation.schema_name, field_name);
                }
                MutationType::Update => {
                    log::info!("🔄 Executing UPDATE mutation for field: {}.{}", mutation.schema_name, field_name);
                    let mut schema_clone = schema.clone();

                    let ref_atom_uuid = self.field_manager.update_field(
                        &mut schema_clone,
                        field_name,
                        value.clone(),
                        mutation.pub_key.clone(),
                    )?;
                    log::info!("✅ Field updated successfully for: {}.{} with ref_atom_uuid: {}",
                              mutation.schema_name, field_name, ref_atom_uuid);

                    // Update the schema manager with the ref_atom_uuid returned from update_field
                    log::info!("💾 Updating schema manager with ref_atom_uuid: {} for field: {}.{}",
                              ref_atom_uuid, mutation.schema_name, field_name);
                    self.schema_manager.update_field_ref_atom_uuid(
                        &mutation.schema_name,
                        field_name,
                        ref_atom_uuid.clone(),
                    )?;
                    log::info!("✅ Schema manager updated successfully for {}.{}",
                              mutation.schema_name, field_name);
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
            log::info!("Adding transform orchestrator task for field: {}", field_key);
            log::info!("Transform orchestrator task details - schema: {}, field: {}, mutation_hash: {}",
                schema.name, field_name, mutation_hash);
            
            let result = self
                .transform_orchestrator
                .add_task(&schema.name, field_name, mutation_hash);
            
            match result {
                Ok(_) => log::info!("Transform orchestrator task added successfully for field: {}", field_key),
                Err(e) => log::error!("Failed to add transform orchestrator task for field {}: {:?}", field_key, e),
            }
        }
        
        log::info!("Mutation execution completed successfully for schema: {}", mutation.schema_name);
        log::info!("Total fields processed: {}", mutation.fields_and_values.len());
        
        // Process the transform queue to execute any queued transforms
        log::info!("Processing transform queue after mutation execution");
        self.transform_orchestrator.process_queue();
        log::info!("Transform queue processing completed");
        
        Ok(())
    }
}
