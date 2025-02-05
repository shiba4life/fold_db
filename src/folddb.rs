use std::collections::HashMap;
use serde_json::Value;
use sled;

use crate::atom::{Atom, AtomRef};
use crate::schema::{Schema, SchemaError};  // Updated to use re-exported types
use crate::schema::schema_manager::SchemaManager;
use crate::schema::types::{Query, Mutation};    

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
    pub fn query_schema(&self, query: Query) -> Vec<Result<Value, SchemaError>> {
        // get all the field names from the query
        let field_names = query.fields;
        let field_results = field_names.iter().map(|field_name| {
            self.get_field_value(&query.schema_name, field_name)
        });
        let field_values = field_results.collect::<Vec<_>>();

        field_values
    }

    /// Writes data to a schema
    pub fn write_schema(&mut self, mutation: Mutation) -> Result<(), SchemaError> {
        for (field, value) in mutation.fields_and_values {
            self.set_field_value(&mutation.schema_name, &field, value, mutation.pub_key.clone())?;
        }
        Ok(())  
    }

    fn get_latest_atom(&self, aref_uuid: &str) -> Result<Atom, Box<dyn std::error::Error>> {
        // Try in-memory cache first
        if let Some(aref) = self.ref_atoms.get(aref_uuid) {
            if let Some(atom) = self.atoms.get(aref.get_atom_uuid().unwrap()) {
                return Ok(atom.clone());
            }
        }

        // Try from disk
        let aref_bytes = self.db.get(aref_uuid.as_bytes())?
            .ok_or("AtomRef not found")?;
        let aref: AtomRef = serde_json::from_slice(&aref_bytes)?;
        
        let atom_bytes = self.db.get(aref.get_atom_uuid().unwrap().as_bytes())?
            .ok_or("Atom not found")?;
        let atom: Atom = serde_json::from_slice(&atom_bytes)?;
        
        Ok(atom)
    }

    pub fn get_atom_history(&self, aref_uuid: &str) -> Result<Vec<Atom>, Box<dyn std::error::Error>> {
        let mut history = Vec::new();
        let mut current_atom = self.get_latest_atom(aref_uuid)?;
        
        history.push(current_atom.clone());
        
        while let Some(prev_uuid) = current_atom.prev_atom_uuid() {
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
                let content: Value = serde_json::from_str(atom.content().as_str().unwrap_or_default())
                    .map_err(|e| SchemaError::InvalidData(format!("Invalid JSON content: {}", e)))?;
                Ok(content)
            },
            Err(_) => Ok(Value::Null)
        }
    }

    pub fn set_field_value(&mut self, schema_name: &str, field: &str, content: Value, source_pub_key: String) -> Result<(), SchemaError> {
        let schema = self.schema_manager.get_schema(schema_name)?
            .ok_or_else(|| SchemaError::NotFound(format!("Schema {} not found", schema_name)))?;
        
        let field = schema.fields.get(field)
            .ok_or_else(|| SchemaError::InvalidField(format!("Field {} not found", field)))?;
        
        let aref_uuid = field.ref_atom_uuid.clone();
        let prev_atom_uuid = self.ref_atoms.get(&aref_uuid)
            .map(|aref| aref.get_atom_uuid().unwrap().clone());

        
        // Create new atom
        let atom = Atom::new(
            schema_name.to_string(),
            source_pub_key,
            prev_atom_uuid,
            content,
        );

        // Store value
        let atom_bytes = serde_json::to_vec(&atom)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize atom: {}", e)))?;
        self.db.insert(atom.uuid().as_bytes(), atom_bytes)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to store atom: {}", e)))?;
        
        // Update atom ref
        let mut aref = self.ref_atoms.get(&aref_uuid)
            .map(|aref| aref.clone())
            .unwrap_or_else(|| AtomRef::new(atom.uuid().to_string()));
        aref.set_atom_uuid(atom.uuid().to_string());

        self.ref_atoms.insert(aref_uuid, aref);
        
        Ok(())
    }
}
