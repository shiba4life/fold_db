use super::atom_manager::AtomManager;
use super::context::AtomContext;
use super::transform_manager::TransformManager;
use crate::atom::AtomStatus;
use crate::schema::types::fields::FieldType;
use crate::schema::Schema;
use crate::schema::SchemaError;
use serde_json::Value;
use log::info;

use std::sync::{Arc, RwLock};
use super::transform_orchestrator::TransformOrchestrator;

pub struct FieldManager {
    pub(super) atom_manager: AtomManager,
    transform_manager: Arc<RwLock<Option<Arc<TransformManager>>>>,
    orchestrator: Arc<RwLock<Option<Arc<TransformOrchestrator>>>>,
}

impl FieldManager {
    pub fn new(atom_manager: AtomManager) -> Self {
        Self {
            atom_manager,
            transform_manager: Arc::new(RwLock::new(None)),
            orchestrator: Arc::new(RwLock::new(None)),
        }
    }

    pub fn set_transform_manager(&self, manager: Arc<TransformManager>) -> Result<(), SchemaError> {
        let mut guard = self
            .transform_manager
            .write()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire transform_manager lock".to_string()))?;
        *guard = Some(manager);
        Ok(())
    }

    pub fn get_transform_manager(&self) -> Result<Option<Arc<TransformManager>>, SchemaError> {
        let guard = self
            .transform_manager
            .read()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire transform_manager lock".to_string()))?;
        Ok(guard.clone())
    }

    pub fn set_orchestrator(&self, orchestrator: Arc<TransformOrchestrator>) -> Result<(), SchemaError> {
        let mut guard = self
            .orchestrator
            .write()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire orchestrator lock".to_string()))?;
        *guard = Some(orchestrator);
        Ok(())
    }

    pub fn get_orchestrator(&self) -> Result<Option<Arc<TransformOrchestrator>>, SchemaError> {
        let guard = self
            .orchestrator
            .read()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire orchestrator lock".to_string()))?;
        Ok(guard.clone())
    }

    pub fn get_or_create_atom_ref(
        &mut self,
        schema: &Schema,
        field: &str,
        source_pub_key: &str,
    ) -> Result<String, SchemaError> {
        let mut ctx = AtomContext::new(
            schema,
            field,
            source_pub_key.to_string(),
            &mut self.atom_manager,
        );
        ctx.get_or_create_atom_ref()
    }

    pub fn get_field_value(&self, schema: &Schema, field: &str) -> Result<Value, SchemaError> {
        let field_def = schema
            .fields
            .get(field)
            .ok_or_else(|| SchemaError::InvalidField(format!("Field {} not found", field)))?;

        let result = if let Some(ref_atom_uuid) = field_def.get_ref_atom_uuid() {
            // Try to get the atom reference
            let ref_atoms = self.atom_manager.get_ref_atoms();
            let atoms = self.atom_manager.get_atoms();

            let atom_uuid = {
                let guard = ref_atoms
                    .lock()
                    .map_err(|_| SchemaError::InvalidData("Failed to acquire ref_atoms lock".to_string()))?;
                guard
                    .get(&ref_atom_uuid)
                    .map(|aref| aref.get_atom_uuid().clone())
            };

            // If we have an atom UUID, try to get the atom
            if let Some(atom_uuid) = atom_uuid {
                let guard = atoms
                    .lock()
                    .map_err(|_| SchemaError::InvalidData("Failed to acquire atoms lock".to_string()))?;
                if let Some(atom) = guard.get(&atom_uuid) {
                    atom.content().clone()
                } else {
                    self.atom_manager
                        .get_latest_atom(&ref_atom_uuid)
                        .map(|a| a.content().clone())
                        .unwrap_or_else(|_| Self::default_value(field))
                }
            } else {
                self.atom_manager
                    .get_latest_atom(&ref_atom_uuid)
                    .map(|a| a.content().clone())
                    .unwrap_or_else(|_| Self::default_value(field))
            }
        } else {
            Value::Null
        };

        info!(
            "get_field_value - schema: {}, field: {}, result: {:?}",
            schema.name,
            field,
            result
        );

        Ok(result)
    }

    pub fn set_field_value(
        &mut self,
        schema: &mut Schema,
        field: &str,
        content: Value,
        source_pub_key: String,
    ) -> Result<(), SchemaError> {
        let mut ctx = AtomContext::new(schema, field, source_pub_key, &mut self.atom_manager);

        let field_def = ctx.get_field_def()?;
        if FieldType::Collection == *field_def.field_type() {
            return Err(SchemaError::InvalidField(
                "Collection fields cannot be updated without id".to_string(),
            ));
        }

        let aref_uuid = ctx.get_or_create_atom_ref()?;
        let prev_atom_uuid = {
            let ref_atoms = ctx.atom_manager.get_ref_atoms();
            let guard = ref_atoms
                .lock()
                .map_err(|_| SchemaError::InvalidData("Failed to acquire ref_atoms lock".to_string()))?;
            guard
                .get(&aref_uuid)
                .map(|aref| aref.get_atom_uuid().to_string())
        };

        ctx.create_and_update_atom(prev_atom_uuid, content.clone(), None)?;

        info!(
            "set_field_value - schema: {}, field: {}, result: success",
            schema.name,
            field
        );

        Ok(())
    }

    pub fn update_field(
        &mut self,
        schema: &Schema,
        field: &str,
        content: Value,
        source_pub_key: String,
    ) -> Result<(), SchemaError> {
        let mut ctx = AtomContext::new(schema, field, source_pub_key, &mut self.atom_manager);

        let field_def = ctx.get_field_def()?;
        if FieldType::Collection == *field_def.field_type() {
            return Err(SchemaError::InvalidField(
                "Collection fields cannot be updated".to_string(),
            ));
        }

        let aref_uuid = ctx.get_or_create_atom_ref()?;
        let prev_atom_uuid = ctx.get_prev_atom_uuid(&aref_uuid)?;

        ctx.create_and_update_atom(Some(prev_atom_uuid), content.clone(), None)?;

        info!(
            "update_field - schema: {}, field: {}, result: success",
            schema.name,
            field
        );

        Ok(())
    }

    pub fn delete_field(
        &mut self,
        schema: &Schema,
        field: &str,
        source_pub_key: String,
    ) -> Result<(), SchemaError> {
        let mut ctx = AtomContext::new(schema, field, source_pub_key, &mut self.atom_manager);

        let field_def = ctx.get_field_def()?;
        if FieldType::Collection == *field_def.field_type() {
            return Err(SchemaError::InvalidField(
                "Collection fields cannot be deleted without id".to_string(),
            ));
        }

        let aref_uuid = ctx.get_or_create_atom_ref()?;
        let prev_atom_uuid = ctx.get_prev_atom_uuid(&aref_uuid)?;

        ctx.create_and_update_atom(Some(prev_atom_uuid), Value::Null, Some(AtomStatus::Deleted))?;

        info!(
            "delete_field - schema: {}, field: {}, result: success",
            schema.name,
            field
        );

        Ok(())
    }

    fn default_value(field: &str) -> Value {
        match field {
            "username" | "email" | "full_name" | "bio" | "location" => {
                Value::String("".to_string())
            }
            "age" => Value::Number(serde_json::Number::from(0)),
            _ => Value::Null,
        }
    }
}

impl Clone for FieldManager {
    fn clone(&self) -> Self {
        Self {
            atom_manager: self.atom_manager.clone(),
            transform_manager: Arc::clone(&self.transform_manager),
            orchestrator: Arc::clone(&self.orchestrator),
        }
    }
}
