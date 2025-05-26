use super::FoldDB;
use crate::schema::types::{Mutation, MutationType};
use crate::schema::SchemaError;
use crate::schema::types::field::common::Field;
use sha2::{Digest, Sha256};

impl FoldDB {
    pub fn write_schema(&mut self, mutation: Mutation) -> Result<(), SchemaError> {
        if mutation.fields_and_values.is_empty() {
            return Err(SchemaError::InvalidData("No fields to write".to_string()));
        }

        let mutation_bytes = serde_json::to_vec(&mutation)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize mutation: {}", e)))?;
        let mut hasher = Sha256::new();
        hasher.update(mutation_bytes);
        let mutation_hash = format!("{:x}", hasher.finalize());

        let schema = self
            .schema_manager
            .get_schema(&mutation.schema_name)?
            .ok_or_else(|| {
                SchemaError::NotFound(format!("Schema {} not found", mutation.schema_name))
            })?;

        for (field_name, value) in mutation.fields_and_values.iter() {
            let perm = self.permission_wrapper.check_mutation_field_permission(
                &mutation,
                field_name,
                &self.schema_manager,
            );
            if mutation.trust_distance != 0 && !perm.allowed {
                return Err(perm.error.unwrap_or(SchemaError::InvalidPermission(
                    "Unknown permission error".to_string(),
                )));
            }

            match mutation.mutation_type {
                MutationType::Create => {
                    let mut schema_clone = schema.clone();
                    self.field_manager.set_field_value(
                        &mut schema_clone,
                        field_name,
                        value.clone(),
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
                MutationType::Update => {
                    let mut schema_clone = schema.clone();
                    self.field_manager.update_field(
                        &mut schema_clone,
                        field_name,
                        value.clone(),
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
                        &schema,
                        field_name,
                        value.clone(),
                        mutation.pub_key.clone(),
                        id.clone(),
                    )?;
                }
                MutationType::UpdateToCollection(ref id) => {
                    self.collection_manager.update_collection_field_value(
                        &schema,
                        field_name,
                        value.clone(),
                        mutation.pub_key.clone(),
                        id.clone(),
                    )?;
                }
                MutationType::DeleteFromCollection(ref id) => {
                    self.collection_manager.delete_collection_field_value(
                        &schema,
                        field_name,
                        mutation.pub_key.clone(),
                        id.clone(),
                    )?;
                }
            }

            let _ = self
                .transform_orchestrator
                .add_task(&schema.name, field_name, &mutation_hash);
        }
        Ok(())
    }
}
