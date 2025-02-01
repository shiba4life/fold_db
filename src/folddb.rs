use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sled;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Atom {
    pub uuid: String,
    pub content: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub prev: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AtomRef {
    pub latest_atom: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InternalSchema {
    pub fields: HashMap<String, String>,
}

pub struct FoldDB {
    pub db: sled::Db,
    pub internal_schemas: HashMap<String, InternalSchema>,
}

impl FoldDB {
    pub fn new(path: &str, internal_schemas: HashMap<String, InternalSchema>) -> sled::Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db, internal_schemas })
    }

    pub fn create_atom(&self, content: String, type_field: String, source: String, prev: Option<String>) -> Result<Atom, Box<dyn std::error::Error>> {
        let atom = Atom {
            uuid: Uuid::new_v4().to_string(),
            content,
            type_field,
            source,
            created_at: Utc::now(),
            prev,
        };
        
        let atom_bytes = serde_json::to_vec(&atom)?;
        self.db.insert(atom.uuid.as_bytes(), atom_bytes)?;
        Ok(atom)
    }

    pub fn create_atom_ref(&self, atom: &Atom) -> Result<String, Box<dyn std::error::Error>> {
        let aref_uuid = Uuid::new_v4().to_string();
        let aref = AtomRef {
            latest_atom: atom.uuid.clone(),
        };
        
        let aref_bytes = serde_json::to_vec(&aref)?;
        self.db.insert(aref_uuid.as_bytes(), aref_bytes)?;
        Ok(aref_uuid)
    }

    pub fn update_atom_ref(&self, aref_uuid: &str, new_atom: &Atom) -> Result<(), Box<dyn std::error::Error>> {
        let aref = AtomRef {
            latest_atom: new_atom.uuid.clone(),
        };
        
        let aref_bytes = serde_json::to_vec(&aref)?;
        self.db.insert(aref_uuid.as_bytes(), aref_bytes)?;
        Ok(())
    }

    pub fn get_latest_atom(&self, aref_uuid: &str) -> Result<Atom, Box<dyn std::error::Error>> {
        let aref_bytes = self
            .db
            .get(aref_uuid.as_bytes())?
            .ok_or("AtomRef not found")?;
        let aref: AtomRef = serde_json::from_slice(&aref_bytes)?;
        let atom_bytes = self
            .db
            .get(aref.latest_atom.as_bytes())?
            .ok_or("Atom not found")?;
        let atom: Atom = serde_json::from_slice(&atom_bytes)?;
        Ok(atom)
    }

    pub fn get_atom_history(&self, aref_uuid: &str) -> Result<Vec<Atom>, Box<dyn std::error::Error>> {
        let mut history = Vec::new();
        let mut current_atom = self.get_latest_atom(aref_uuid)?;
        
        history.push(current_atom.clone());
        
        while let Some(prev_uuid) = current_atom.prev {
            let atom_bytes = self
                .db
                .get(prev_uuid.as_bytes())?
                .ok_or("Previous atom not found")?;
            current_atom = serde_json::from_slice(&atom_bytes)?;
            history.push(current_atom.clone());
        }
        
        Ok(history)
    }

    pub fn get_field_value(&self, schema_name: &str, field: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let internal_schema = self.internal_schemas
            .get(schema_name)
            .ok_or("Internal schema not found")?;
        let aref_uuid = internal_schema.fields
            .get(field)
            .ok_or("Field not found in internal schema")?;
        let atom = self.get_latest_atom(aref_uuid)?;
        let content: Value = serde_json::from_str(&atom.content)?;
        Ok(content)
    }

    pub fn update_field_value(&self, schema_name: &str, field: &str, value: Value, source: String) -> Result<(), Box<dyn std::error::Error>> {
        let internal_schema = self.internal_schemas
            .get(schema_name)
            .ok_or("Internal schema not found")?;
        let aref_uuid = internal_schema.fields
            .get(field)
            .ok_or("Field not found in internal schema")?;
        
        // Get the current atom to set as prev
        let current_atom = self.get_latest_atom(aref_uuid)?;
        
        // Create new atom with the updated value
        let new_atom = self.create_atom(
            value.to_string(),
            "field_update".to_string(),
            source,
            Some(current_atom.uuid),
        )?;
        
        // Update the atom ref to point to the new atom
        self.update_atom_ref(aref_uuid, &new_atom)?;
        
        Ok(())
    }
}
