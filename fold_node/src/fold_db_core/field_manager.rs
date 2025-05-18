use super::atom_manager::AtomManager;
use super::context::AtomContext;
use super::transform_manager::TransformManager;
use crate::atom::AtomStatus;
use crate::schema::types::fields::FieldType;
use crate::schema::Schema;
use crate::schema::SchemaError;
use serde_json::Value;

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

    pub fn set_transform_manager(&self, manager: Arc<TransformManager>) {
        let mut guard = self.transform_manager.write().unwrap();
        *guard = Some(manager);
    }

    pub fn get_transform_manager(&self) -> Option<Arc<TransformManager>> {
        self.transform_manager.read().unwrap().clone()
    }

    pub fn set_orchestrator(&self, orchestrator: Arc<TransformOrchestrator>) {
        let mut guard = self.orchestrator.write().unwrap();
        *guard = Some(orchestrator);
    }

    pub fn get_orchestrator(&self) -> Option<Arc<TransformOrchestrator>> {
        self.orchestrator.read().unwrap().clone()
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

        let Some(ref_atom_uuid) = field_def.get_ref_atom_uuid() else {
            return Ok(Value::Null);
        };

        // Try to get the atom reference
        let ref_atoms = self.atom_manager.get_ref_atoms();
        let atoms = self.atom_manager.get_atoms();

        let atom_uuid = {
            let guard = ref_atoms.lock().unwrap();
            guard
                .get(&ref_atom_uuid)
                .map(|aref| aref.get_atom_uuid().clone())
        };

        // If we have an atom UUID, try to get the atom
        if let Some(atom_uuid) = atom_uuid {
            let guard = atoms.lock().unwrap();
            if let Some(atom) = guard.get(&atom_uuid) {
                return Ok(atom.content().clone());
            }
        }

        // If we couldn't find the atom in memory, try from disk
        match self.atom_manager.get_latest_atom(&ref_atom_uuid) {
            Ok(atom) => Ok(atom.content().clone()),
            Err(_) => {
                // If we couldn't find the atom, return a default value based on the field name
                // This is a temporary solution until we implement proper default values
                match field {
                    "username" => Ok(Value::String("".to_string())),
                    "email" => Ok(Value::String("".to_string())),
                    "full_name" => Ok(Value::String("".to_string())),
                    "bio" => Ok(Value::String("".to_string())),
                    "age" => Ok(Value::Number(serde_json::Number::from(0))),
                    "location" => Ok(Value::String("".to_string())),
                    _ => Ok(Value::Null),
                }
            }
        }
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
            let guard = ref_atoms.lock().unwrap();
            guard
                .get(&aref_uuid)
                .map(|aref| aref.get_atom_uuid().to_string())
        };

        ctx.create_and_update_atom(prev_atom_uuid, content.clone(), None)?;

        if let Some(orc) = self.get_orchestrator() {
            orc.add_task(&schema.name, field);
        }

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

        if let Some(orc) = self.get_orchestrator() {
            orc.add_task(&schema.name, field);
        }

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

        if let Some(orc) = self.get_orchestrator() {
            orc.add_task(&schema.name, field);
        }

        Ok(())
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
