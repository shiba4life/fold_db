use crate::atom::{Atom, AtomRef, AtomRefCollection, AtomRefRange, AtomStatus};
use crate::db_operations::DbOperations;
use crate::schema::types::SchemaError;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct AtomManager {
    db_ops: Arc<DbOperations>,
    atoms: Arc<Mutex<HashMap<String, Atom>>>,
    ref_atoms: Arc<Mutex<HashMap<String, AtomRef>>>,
    ref_collections: Arc<Mutex<HashMap<String, AtomRefCollection>>>,
    ref_ranges: Arc<Mutex<HashMap<String, AtomRefRange>>>,
}

impl AtomManager {
    pub fn new(db_ops: DbOperations) -> Self {
        let mut atoms = HashMap::new();
        let mut ref_atoms = HashMap::new();
        let mut ref_collections = HashMap::new();
        let mut ref_ranges = HashMap::new();

        for result in db_ops.db().iter().flatten() {
            let key_str = String::from_utf8_lossy(result.0.as_ref());
            let bytes = result.1.as_ref();

            if let Some(stripped) = key_str.strip_prefix("atom:") {
                if let Ok(atom) = serde_json::from_slice(bytes) {
                    atoms.insert(stripped.to_string(), atom);
                }
            } else if let Some(stripped) = key_str.strip_prefix("ref:") {
                if let Ok(atom_ref) = serde_json::from_slice::<AtomRef>(bytes) {
                    ref_atoms.insert(stripped.to_string(), atom_ref);
                } else if let Ok(collection) = serde_json::from_slice::<AtomRefCollection>(bytes) {
                    ref_collections.insert(stripped.to_string(), collection);
                } else if let Ok(range) = serde_json::from_slice::<AtomRefRange>(bytes) {
                    ref_ranges.insert(stripped.to_string(), range);
                }
            } else {
                // Backwards compatibility: entries without prefixes
                if let Ok(atom) = serde_json::from_slice::<Atom>(bytes) {
                    atoms.insert(key_str.into_owned(), atom);
                } else if let Ok(atom_ref) = serde_json::from_slice::<AtomRef>(bytes) {
                    ref_atoms.insert(key_str.into_owned(), atom_ref);
                } else if let Ok(collection) = serde_json::from_slice::<AtomRefCollection>(bytes) {
                    ref_collections.insert(key_str.into_owned(), collection);
                } else if let Ok(range) = serde_json::from_slice::<AtomRefRange>(bytes) {
                    ref_ranges.insert(key_str.into_owned(), range);
                }
            }
        }

        Self {
            db_ops: Arc::new(db_ops),
            atoms: Arc::new(Mutex::new(atoms)),
            ref_atoms: Arc::new(Mutex::new(ref_atoms)),
            ref_collections: Arc::new(Mutex::new(ref_collections)),
            ref_ranges: Arc::new(Mutex::new(ref_ranges)),
        }
    }

    pub fn get_latest_atom(&self, aref_uuid: &str) -> Result<Atom, Box<dyn std::error::Error>> {
        // Try in-memory cache first
        let ref_atoms = self
            .ref_atoms
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire ref_atoms lock".to_string()))?;
        let atoms = self
            .atoms
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire atoms lock".to_string()))?;

        if let Some(aref) = ref_atoms.get(aref_uuid) {
            if let Some(atom) = atoms.get(aref.get_atom_uuid()) {
                return Ok(atom.clone());
            }
        }

        // Try from disk
        let aref = self
            .db_ops
            .get_item::<AtomRef>(&format!("ref:{}", aref_uuid))?
            .ok_or("AtomRef not found")?;

        let atom = self
            .db_ops
            .get_item::<Atom>(&format!("atom:{}", aref.get_atom_uuid()))?
            .ok_or("Atom not found")?;

        Ok(atom)
    }

    pub fn get_atom_history(
        &self,
        aref_uuid: &str,
    ) -> Result<Vec<Atom>, Box<dyn std::error::Error>> {
        let mut history = Vec::new();
        let mut current_atom = self.get_latest_atom(aref_uuid)?;

        history.push(current_atom.clone());

        while let Some(prev_uuid) = current_atom.prev_atom_uuid() {
            current_atom = self
                .db_ops
                .get_item::<Atom>(&format!("atom:{}", prev_uuid))?
                .ok_or("Previous atom not found")?;
            history.push(current_atom.clone());
        }

        Ok(history)
    }

    pub fn create_atom(
        &self,
        schema_name: &str,
        source_pub_key: String,
        prev_atom_uuid: Option<String>,
        content: Value,
        status: Option<AtomStatus>,
    ) -> Result<Atom, Box<dyn std::error::Error>> {
        let atom = self.db_ops.create_atom(
            schema_name,
            source_pub_key,
            prev_atom_uuid,
            content,
            status,
        )?;
        self
            .atoms
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire atoms lock".to_string()))?
            .insert(atom.uuid().to_string(), atom.clone());
        Ok(atom)
    }

    pub fn update_atom_ref(
        &self,
        aref_uuid: &str,
        atom_uuid: String,
        source_pub_key: String,
    ) -> Result<AtomRef, Box<dyn std::error::Error>> {
        let aref = self
            .db_ops
            .update_atom_ref(aref_uuid, atom_uuid, source_pub_key)?;
        self
            .ref_atoms
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire ref_atoms lock".to_string()))?
            .insert(aref_uuid.to_string(), aref.clone());
        
        Ok(aref)
    }

    pub fn update_atom_ref_collection(
        &self,
        aref_uuid: &str,
        atom_uuid: String,
        id: String,
        source_pub_key: String,
    ) -> Result<AtomRefCollection, Box<dyn std::error::Error>> {
        let collection =
            self.db_ops
                .update_atom_ref_collection(aref_uuid, atom_uuid, id, source_pub_key)?;
        self
            .ref_collections
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire ref_collections lock".to_string()))?
            .insert(aref_uuid.to_string(), collection.clone());
        Ok(collection)
    }

    pub fn update_atom_ref_range(
        &self,
        aref_uuid: &str,
        atom_uuid: String,
        key: String,
        source_pub_key: String,
    ) -> Result<AtomRefRange, Box<dyn std::error::Error>> {
        let range = self
            .db_ops
            .update_atom_ref_range(aref_uuid, atom_uuid, key, source_pub_key)?;
        self
            .ref_ranges
            .lock()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire ref_ranges lock".to_string()))?
            .insert(aref_uuid.to_string(), range.clone());
        Ok(range)
    }

    pub fn get_ref_atoms(&self) -> Arc<Mutex<HashMap<String, AtomRef>>> {
        Arc::clone(&self.ref_atoms)
    }

    pub fn get_ref_collections(&self) -> Arc<Mutex<HashMap<String, AtomRefCollection>>> {
        Arc::clone(&self.ref_collections)
    }

    pub fn get_ref_ranges(&self) -> Arc<Mutex<HashMap<String, AtomRefRange>>> {
        Arc::clone(&self.ref_ranges)
    }

    pub fn get_atoms(&self) -> Arc<Mutex<HashMap<String, Atom>>> {
        Arc::clone(&self.atoms)
    }

}

impl Clone for AtomManager {
    fn clone(&self) -> Self {
        Self {
            db_ops: Arc::clone(&self.db_ops),
            atoms: Arc::clone(&self.atoms),
            ref_atoms: Arc::clone(&self.ref_atoms),
            ref_collections: Arc::clone(&self.ref_collections),
            ref_ranges: Arc::clone(&self.ref_ranges),
        }
    }
}
