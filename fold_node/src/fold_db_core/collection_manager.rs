use super::context::AtomContext;
use super::field_manager::FieldManager;
use crate::atom::AtomStatus;
use crate::schema::types::field::FieldType;
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
        let mut ctx = AtomContext::new(
            schema,
            field,
            source_pub_key,
            &mut self.field_manager.atom_manager,
        );
        ctx.validate_field_type(FieldType::Collection)?;
        let aref_uuid = ctx.get_or_create_atom_ref()?;
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
        let mut ctx = AtomContext::new(
            schema,
            field,
            source_pub_key,
            &mut self.field_manager.atom_manager,
        );
        ctx.validate_field_type(FieldType::Collection)?;

        let aref_uuid = ctx.get_or_create_atom_ref()?;
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
        let mut ctx = AtomContext::new(
            schema,
            field,
            source_pub_key,
            &mut self.field_manager.atom_manager,
        );
        ctx.validate_field_type(FieldType::Collection)?;

        let aref_uuid = ctx.get_or_create_atom_ref()?;
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
