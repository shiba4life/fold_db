use super::atom_manager::AtomManager;
use crate::atom::{AtomRef, AtomRefCollection, AtomRefRange, AtomStatus};
use crate::schema::types::field::FieldType;
use crate::schema::types::Field;
use crate::schema::Schema;
use crate::schema::SchemaError;
use serde_json::Value;
use uuid::Uuid;

pub struct AtomContext<'a> {
    schema: &'a Schema,
    field: &'a str,
    source_pub_key: String,
    pub(super) atom_manager: &'a mut AtomManager,
}

impl<'a> AtomContext<'a> {
    pub fn new(
        schema: &'a Schema,
        field: &'a str,
        source_pub_key: String,
        atom_manager: &'a mut AtomManager,
    ) -> Self {
        Self {
            schema,
            field,
            source_pub_key,
            atom_manager,
        }
    }

    pub fn get_field_def(
        &self,
    ) -> Result<&'a crate::schema::types::FieldVariant, SchemaError> {
        self.schema
            .fields
            .get(self.field)
            .ok_or_else(|| SchemaError::InvalidField(format!("Field {} not found", self.field)))
    }

    pub fn get_or_create_atom_ref(&mut self) -> Result<String, SchemaError> {
        let field_def = self.get_field_def()?;

        let aref_uuid = if let Some(uuid) = field_def.ref_atom_uuid() {
            uuid.clone()
        } else {
            let aref_uuid = Uuid::new_v4().to_string();
            match field_def {
                crate::schema::types::FieldVariant::Single(_) => {
                    let aref = AtomRef::new(aref_uuid.clone(), self.source_pub_key.clone());
                    let ref_atoms = self.atom_manager.get_ref_atoms();
                    let mut guard = ref_atoms
                        .lock()
                        .map_err(|_| SchemaError::InvalidData("Failed to acquire ref_atoms lock".to_string()))?;
                    guard.insert(aref_uuid.clone(), aref);
                }
                crate::schema::types::FieldVariant::Collection(_) => {
                    let collection = AtomRefCollection::new(self.source_pub_key.clone());
                    let ref_collections = self.atom_manager.get_ref_collections();
                    let mut guard = ref_collections
                        .lock()
                        .map_err(|_| SchemaError::InvalidData("Failed to acquire ref_collections lock".to_string()))?;
                    guard.insert(aref_uuid.clone(), collection);
                }
                crate::schema::types::FieldVariant::Range(_) => {
                    let range = AtomRefRange::new(self.source_pub_key.clone());
                    let ref_ranges = self.atom_manager.get_ref_ranges();
                    let mut guard = ref_ranges
                        .lock()
                        .map_err(|_| SchemaError::InvalidData("Failed to acquire ref_ranges lock".to_string()))?;
                    guard.insert(aref_uuid.clone(), range);
                }
            }
            aref_uuid
        };

        Ok(aref_uuid)
    }

    pub fn get_prev_atom_uuid(&self, aref_uuid: &str) -> Result<String, SchemaError> {
        let ref_atoms = self.atom_manager.get_ref_atoms();
        let guard = ref_atoms
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire ref_atoms lock".to_string()))?;
        let aref = guard
            .get(aref_uuid)
            .ok_or_else(|| SchemaError::InvalidData("AtomRef not found".to_string()))?;
        Ok(aref.get_atom_uuid().to_string())
    }

    pub fn get_prev_collection_atom_uuid(
        &self,
        aref_uuid: &str,
        id: &str,
    ) -> Result<String, SchemaError> {
        let ref_collections = self.atom_manager.get_ref_collections();
        let guard = ref_collections
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire ref_collections lock".to_string()))?;
        let aref = guard
            .get(aref_uuid)
            .ok_or_else(|| SchemaError::InvalidData("AtomRefCollection not found".to_string()))?;
        aref.get_atom_uuid(id)
            .ok_or_else(|| SchemaError::InvalidData("Atom not found".to_string()))
            .map(|uuid| uuid.to_string())
    }

    pub fn create_and_update_atom(
        &mut self,
        prev_atom_uuid: Option<String>,
        content: Value,
        status: Option<AtomStatus>,
    ) -> Result<(), SchemaError> {
        let atom = self
            .atom_manager
            .create_atom(
                &self.schema.name,
                self.source_pub_key.clone(),
                prev_atom_uuid,
                content,
                status,
            )
            .map_err(|e| SchemaError::InvalidData(e.to_string()))?;

        let aref_uuid = self.get_or_create_atom_ref()?;
        let field_def = self.get_field_def()?;

        match field_def {
            crate::schema::types::FieldVariant::Single(_) => {
                self.atom_manager
                    .update_atom_ref(
                        &aref_uuid,
                        atom.uuid().to_string(),
                        self.source_pub_key.clone(),
                    )
                    .map_err(|e| SchemaError::InvalidData(e.to_string()))?;
            }
            crate::schema::types::FieldVariant::Collection(_) => {
                self.atom_manager
                    .update_atom_ref_collection(
                        &aref_uuid,
                        atom.uuid().to_string(),
                        "0".to_string(),
                        self.source_pub_key.clone(),
                    )
                    .map_err(|e| SchemaError::InvalidData(e.to_string()))?;
            }
            crate::schema::types::FieldVariant::Range(_) => {
                self.atom_manager
                    .update_atom_ref_range(
                        &aref_uuid,
                        atom.uuid().to_string(),
                        "0".to_string(),
                        self.source_pub_key.clone(),
                    )
                    .map_err(|e| SchemaError::InvalidData(e.to_string()))?;
            }
        }

        Ok(())
    }

    pub fn create_and_update_collection_atom(
        &mut self,
        prev_atom_uuid: Option<String>,
        content: Value,
        status: Option<AtomStatus>,
        id: String,
    ) -> Result<(), SchemaError> {
        let atom = self
            .atom_manager
            .create_atom(
                &self.schema.name,
                self.source_pub_key.clone(),
                prev_atom_uuid,
                content,
                status,
            )
            .map_err(|e| SchemaError::InvalidData(e.to_string()))?;

        let aref_uuid = self.get_or_create_atom_ref()?;

        self.atom_manager
            .update_atom_ref_collection(
                &aref_uuid,
                atom.uuid().to_string(),
                id,
                self.source_pub_key.clone(),
            )
            .map_err(|e| SchemaError::InvalidData(e.to_string()))?;

        Ok(())
    }

    pub fn validate_field_type(&self, expected_type: FieldType) -> Result<(), SchemaError> {
        let field_def = self.get_field_def()?;
        let matches = matches!((field_def, &expected_type),
            (crate::schema::types::FieldVariant::Single(_), &FieldType::Single) |
            (crate::schema::types::FieldVariant::Collection(_), &FieldType::Collection) |
            (crate::schema::types::FieldVariant::Range(_), &FieldType::Range)
        );
        
        if !matches {
            let msg = match &expected_type {
                FieldType::Single => "Collection fields cannot be updated without id",
                FieldType::Collection => "Single fields cannot be updated with collection id",
                FieldType::Range => "Incorrect field type for range operation",
            };
            return Err(SchemaError::InvalidField(msg.to_string()));
        }
        Ok(())
    }
}
