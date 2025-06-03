use super::context::AtomContext;
use super::field_manager::FieldManager;
use crate::atom::AtomStatus;
use crate::schema::types::field::FieldType;
use crate::schema::types::field::common::Field;
use crate::schema::Schema;
use crate::schema::SchemaError;
use log::info;
use serde_json::Value;

pub struct CollectionManager {
    pub(super) field_manager: FieldManager,
}

impl CollectionManager {
    pub fn new(field_manager: FieldManager) -> Self {
        Self { field_manager }
    }

    pub fn add_collection_field_value(
        &mut self,
        schema: &Schema,
        field: &str,
        content: Value,
        source_pub_key: String,
        id: String,
    ) -> Result<(), SchemaError> {
        // Check if field has an existing ref_atom_uuid
        let existing_ref_uuid = schema
            .fields
            .get(field)
            .and_then(|f| f.ref_atom_uuid())
            .ok_or_else(|| {
                SchemaError::InvalidData(format!(
                    "No existing atom_ref found for collection field {}.{}. Atom_refs must be created during field creation.",
                    schema.name, field
                ))
            })?;

        let mut ctx = AtomContext::new(
            schema,
            field,
            source_pub_key,
            &mut self.field_manager.atom_manager,
        );
        ctx.validate_field_type(FieldType::Collection)?;
        
        // Set the context to use the existing aref_uuid instead of creating a new one
        let aref_uuid = existing_ref_uuid.to_string();
        ctx.set_ref_atom_uuid(aref_uuid.clone());
        
        ctx.create_and_update_collection_atom(None, content.clone(), None, id.clone())?;

        info!(
            "add_collection_field_value - schema: {}, field: {}, id: {}, aref_uuid: {}, result: success",
            schema.name,
            field,
            id,
            aref_uuid
        );

        Ok(())
    }

    pub fn update_collection_field_value(
        &mut self,
        schema: &Schema,
        field: &str,
        content: Value,
        source_pub_key: String,
        id: String,
    ) -> Result<(), SchemaError> {
        // Check if field has an existing ref_atom_uuid
        let existing_ref_uuid = schema
            .fields
            .get(field)
            .and_then(|f| f.ref_atom_uuid());

        let mut ctx = AtomContext::new(
            schema,
            field,
            source_pub_key,
            &mut self.field_manager.atom_manager,
        );
        ctx.validate_field_type(FieldType::Collection)?;

        let aref_uuid = if let Some(existing_uuid) = existing_ref_uuid {
            // Subsequent mutation - use existing atom_ref
            log::info!("🔄 Using existing atom_ref for collection {}.{}", schema.name, field);
            ctx.set_ref_atom_uuid(existing_uuid.to_string());
            existing_uuid.to_string()
        } else {
            // First mutation - create atom_ref
            log::info!("🆕 Creating new atom_ref for collection {}.{}", schema.name, field);
            ctx.get_or_create_atom_ref()?
        };
        
        let prev_atom_uuid = ctx.get_prev_collection_atom_uuid(&aref_uuid, &id)?;

        ctx.create_and_update_collection_atom(
            Some(prev_atom_uuid),
            content.clone(),
            None,
            id.clone(),
        )?;

        info!(
            "update_collection_field_value - schema: {}, field: {}, id: {}, aref_uuid: {}, result: success",
            schema.name,
            field,
            id,
            aref_uuid
        );

        Ok(())
    }

    pub fn delete_collection_field_value(
        &mut self,
        schema: &Schema,
        field: &str,
        source_pub_key: String,
        id: String,
    ) -> Result<(), SchemaError> {
        // Check if field has an existing ref_atom_uuid
        let existing_ref_uuid = schema
            .fields
            .get(field)
            .and_then(|f| f.ref_atom_uuid())
            .ok_or_else(|| {
                SchemaError::InvalidData(format!(
                    "Cannot delete from collection field {}.{} - no atom_ref exists.",
                    schema.name, field
                ))
            })?;

        let mut ctx = AtomContext::new(
            schema,
            field,
            source_pub_key,
            &mut self.field_manager.atom_manager,
        );
        ctx.validate_field_type(FieldType::Collection)?;

        // For delete operations, atom_ref must already exist
        let aref_uuid = existing_ref_uuid.to_string();
        ctx.set_ref_atom_uuid(aref_uuid.clone());
        
        let prev_atom_uuid = ctx.get_prev_collection_atom_uuid(&aref_uuid, &id)?;

        ctx.create_and_update_collection_atom(
            Some(prev_atom_uuid),
            Value::Null,
            Some(AtomStatus::Deleted),
            id.clone(),
        )?;

        info!(
            "delete_collection_field_value - schema: {}, field: {}, id: {}, aref_uuid: {}, result: success",
            schema.name,
            field,
            id,
            aref_uuid
        );

        Ok(())
    }
}

impl Clone for CollectionManager {
    fn clone(&self) -> Self {
        Self {
            field_manager: self.field_manager.clone(),
        }
    }
}
