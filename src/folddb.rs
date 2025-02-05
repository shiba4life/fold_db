use std::collections::HashMap;
use serde_json::Value;
use sled;

use crate::atom::{Atom, AtomRef};
use crate::schema::types::{Schema, SchemaField, SchemaError};
use crate::schema::manager::SchemaManager;

pub struct FoldDB {
    pub db: sled::Db,
    pub atoms: HashMap<String, Atom>,
    pub ref_atoms: HashMap<String, AtomRef>,
    pub schema_manager: SchemaManager,
}

impl FoldDB {
    pub fn new(path: &str) -> sled::Result<Self> {
        let db = sled::open(path)?;
        Ok(Self {
            db,
            atoms: HashMap::new(),
            ref_atoms: HashMap::new(),
            schema_manager: SchemaManager::new(),
        })
    }

    /// Loads and validates a schema, running any transforms
    pub fn load_schema(&mut self, schema: Schema) -> Result<(), SchemaError> {
        self.schema_manager.load_schema(schema)
    }

    /// Makes a schema queriable and writable
    pub fn allow_schema(&mut self, schema_name: &str) -> Result<(), SchemaError> {
        let exists = self.schema_manager.schema_exists(schema_name)?;
        if !exists {
            return Err(SchemaError::NotFound(format!("Schema {} not found", schema_name)));
        }
        Ok(())
    }

    /// Executes a query against a schema
    pub fn query_schema(&self, schema_name: &str, query: Value) -> Result<Value, SchemaError> {
        let schema = self.schema_manager.get_schema(schema_name)?
            .ok_or_else(|| SchemaError::NotFound(format!("Schema {} not found", schema_name)))?;
        
        // TODO: Implement query execution
        Ok(Value::Null)
    }

    /// Writes data to a schema
    pub fn write_schema(&mut self, schema_name: &str, data: Value) -> Result<(), SchemaError> {
        let schema = self.schema_manager.get_schema(schema_name)?
            .ok_or_else(|| SchemaError::NotFound(format!("Schema {} not found", schema_name)))?;
        
        // TODO: Implement schema writing
        Ok(())
    }

    fn get_latest_atom(&self, aref_uuid: &str) -> Result<Atom, Box<dyn std::error::Error>> {
        // Try in-memory cache first
        if let Some(aref) = self.ref_atoms.get(aref_uuid) {
            if let Some(atom) = self.atoms.get(aref.atom_uuid()) {
                return Ok(atom.clone());
            }
        }

        // Try from disk
        let aref_bytes = self.db.get(aref_uuid.as_bytes())?
            .ok_or("AtomRef not found")?;
        let aref: AtomRef = serde_json::from_slice(&aref_bytes)?;
        
        let atom_bytes = self.db.get(aref.atom_uuid().as_bytes())?
            .ok_or("Atom not found")?;
        let atom: Atom = serde_json::from_slice(&atom_bytes)?;
        
        Ok(atom)
    }

    pub fn get_atom_history(&self, aref_uuid: &str) -> Result<Vec<Atom>, Box<dyn std::error::Error>> {
        let mut history = Vec::new();
        let mut current_atom = self.get_latest_atom(aref_uuid)?;
        
        history.push(current_atom.clone());
        
        while let Some(prev_uuid) = current_atom.prev_atom() {
            let atom_bytes = self.db.get(prev_uuid.as_bytes())?
                .ok_or("Previous atom not found")?;
            current_atom = serde_json::from_slice(&atom_bytes)?;
            history.push(current_atom.clone());
        }
        
        Ok(history)
    }

    pub fn get_field_value(&self, schema_name: &str, field: &str) -> Result<Value, SchemaError> {
        let schema = self.schema_manager.get_schema(schema_name)?
            .ok_or_else(|| SchemaError::NotFound(format!("Schema {} not found", schema_name)))?;
        
        let field = schema.fields.get(field)
            .ok_or_else(|| SchemaError::InvalidField(format!("Field {} not found", field)))?;

        // Try getting the atom
        match self.get_latest_atom(&field.ref_atom_uuid) {
            Ok(atom) => {
                let content: Value = serde_json::from_str(atom.content())
                    .map_err(|e| SchemaError::InvalidData(format!("Invalid JSON content: {}", e)))?;
                Ok(content)
            },
            Err(_) => Ok(Value::Null)
        }
    }

    pub fn set_field_value(&mut self, schema_name: &str, field: &str, value: Value, source: String) -> Result<(), SchemaError> {
        let schema = self.schema_manager.get_schema(schema_name)?
            .ok_or_else(|| SchemaError::NotFound(format!("Schema {} not found", schema_name)))?;
        
        let field = schema.fields.get(field)
            .ok_or_else(|| SchemaError::InvalidField(format!("Field {} not found", field)))?;
        
        // Get the current atom to set as prev if it exists
        let prev_uuid = match self.get_latest_atom(&field.ref_atom_uuid) {
            Ok(current_atom) => Some(current_atom.uuid().to_string()),
            Err(_) => None,
        };
        
        // Create new atom
        let atom = Atom::new(
            value.to_string(),
            source.clone(),
            prev_uuid,
        );
        
        // Store the atom
        let atom_uuid = atom.uuid().to_string();
        let atom_bytes = serde_json::to_vec(&atom)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize atom: {}", e)))?;
        self.db.insert(atom_uuid.as_bytes(), atom_bytes)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to store atom: {}", e)))?;
        
        // Update or create the atom ref
        let aref = AtomRef::new(atom_uuid.clone(), source);
        let aref_bytes = serde_json::to_vec(&aref)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize atom ref: {}", e)))?;
        self.db.insert(field.ref_atom_uuid.as_bytes(), aref_bytes)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to store atom ref: {}", e)))?;
        
        // Update in-memory caches
        self.atoms.insert(atom_uuid, atom);
        self.ref_atoms.insert(field.ref_atom_uuid.clone(), aref);
        
        Ok(())
    }
}
