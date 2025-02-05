use std::collections::HashMap;
use serde_json::Value;
use sled;

use crate::atom::{Atom, AtomRef};
use crate::schema::{Schema, SchemaError};  // Updated to use re-exported types
use crate::schema::schema_manager::SchemaManager;
use crate::schema::types::{Query, Mutation};    
use crate::permissions::types::policy::PermissionsPolicy;

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
            self.get_field_value(&query.schema_name, field_name, &query.pub_key, query.trust_distance)
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

    fn check_read_permission(&self, policy: &PermissionsPolicy, pub_key: &str, trust_distance: u32) -> bool {
        // Check explicit read policy first
        if let Some(explicit_policy) = &policy.explicit_read_policy {
            if explicit_policy.counts_by_pub_key.contains_key(pub_key) {
                return true;
            }
        }
        // Then check trust distance - lower trust_distance means higher trust
        trust_distance <= policy.read_policy
    }

    fn check_write_permission(&self, policy: &PermissionsPolicy, pub_key: &str) -> bool {
        // Check explicit write policy
        if let Some(explicit_policy) = &policy.explicit_write_policy {
            explicit_policy.counts_by_pub_key.contains_key(pub_key)
        } else {
            false // No explicit write permission means no write access
        }
    }

    pub fn get_field_value(&self, schema_name: &str, field: &str, pub_key: &str, trust_distance: u32) -> Result<Value, SchemaError> {
        let schema = self.schema_manager.get_schema(schema_name)?
            .ok_or_else(|| SchemaError::NotFound(format!("Schema {} not found", schema_name)))?;
        
        let field = schema.fields.get(field)
            .ok_or_else(|| SchemaError::InvalidField(format!("Field {} not found", field)))?;

        // Try getting the atom
        match self.get_latest_atom(&field.ref_atom_uuid) {
            Ok(atom) => {
                // Check if the reader has permission based on trust distance
                if !self.check_read_permission(&field.permission_policy, &pub_key, trust_distance) {
                    Err(SchemaError::InvalidPermission("Read access denied".to_string()))
                } else {
                    Ok(atom.content().clone())
                }
            },
            Err(_) => Ok(Value::Null)
        }
    }

    pub fn set_field_value(&mut self, schema_name: &str, field: &str, content: Value, source_pub_key: String) -> Result<(), SchemaError> {
        let schema = self.schema_manager.get_schema(schema_name)?
            .ok_or_else(|| SchemaError::NotFound(format!("Schema {} not found", schema_name)))?;
        
        let field = schema.fields.get(field)
            .ok_or_else(|| SchemaError::InvalidField(format!("Field {} not found", field)))?;

        // Check write permission
        if !self.check_write_permission(&field.permission_policy, &source_pub_key) {
            return Err(SchemaError::InvalidPermission("Write access denied".to_string()));
        }
        
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

        // Store value and update in-memory cache
        let atom_bytes = serde_json::to_vec(&atom)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize atom: {}", e)))?;
        self.db.insert(atom.uuid().as_bytes(), atom_bytes)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to store atom: {}", e)))?;
        self.atoms.insert(atom.uuid().to_string(), atom.clone());

        // Update atom ref with new atom UUID
        let mut aref = self.ref_atoms.get(&aref_uuid)
            .map(|aref| aref.clone())
            .unwrap_or_else(|| AtomRef::new(atom.uuid().to_string()));
        
        // Set the new atom UUID
        aref.set_atom_uuid(atom.uuid().to_string());
        self.ref_atoms.insert(aref_uuid.clone(), aref.clone());

        // Store atom ref
        let aref_bytes = serde_json::to_vec(&aref)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize atom ref: {}", e)))?;
        self.db.insert(aref_uuid.as_bytes(), aref_bytes)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to store atom ref: {}", e)))?;
        
        Ok(())
    }
}
