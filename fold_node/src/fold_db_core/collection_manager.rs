use serde_json::Value;
use crate::schema::types::fields::FieldType;
use crate::schema::SchemaError;
use crate::schema::Schema;
use crate::atom::AtomStatus;
use super::field_manager::FieldManager;
use super::context::AtomContext;

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
        let mut ctx = AtomContext::new(schema, field, source_pub_key, &mut self.field_manager.atom_manager);
        ctx.validate_field_type(FieldType::Collection)?;
        ctx.create_and_update_collection_atom(None, content, None, id)
    }

    pub fn update_collection_field_value(
        &mut self,
        schema: &Schema,
        field: &str,
        content: Value,
        source_pub_key: String,
        id: String,
    ) -> Result<(), SchemaError> {
        let mut ctx = AtomContext::new(schema, field, source_pub_key, &mut self.field_manager.atom_manager);
        ctx.validate_field_type(FieldType::Collection)?;
        
        let aref_uuid = ctx.get_or_create_atom_ref()?;
        let prev_atom_uuid = ctx.get_prev_collection_atom_uuid(&aref_uuid, &id)?;
        
        ctx.create_and_update_collection_atom(Some(prev_atom_uuid), content, None, id)
    }

    pub fn delete_collection_field_value(
        &mut self,
        schema: &Schema,
        field: &str,
        source_pub_key: String,
        id: String,
    ) -> Result<(), SchemaError> {
        let mut ctx = AtomContext::new(schema, field, source_pub_key, &mut self.field_manager.atom_manager);
        ctx.validate_field_type(FieldType::Collection)?;
        
        let aref_uuid = ctx.get_or_create_atom_ref()?;
        let prev_atom_uuid = ctx.get_prev_collection_atom_uuid(&aref_uuid, &id)?;
        
        ctx.create_and_update_collection_atom(Some(prev_atom_uuid), Value::Null, Some(AtomStatus::Deleted), id)
    }

}

impl Clone for CollectionManager {
    fn clone(&self) -> Self {
        Self {
            field_manager: self.field_manager.clone(),
        }
    }
}
