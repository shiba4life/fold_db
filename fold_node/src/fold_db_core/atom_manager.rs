use crate::atom::{Atom, AtomRef, AtomRefCollection, AtomStatus};
use crate::db_operations::DbOperations;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct AtomManager {
    db_ops: Arc<DbOperations>,
    atoms: Arc<Mutex<HashMap<String, Atom>>>,
    ref_atoms: Arc<Mutex<HashMap<String, AtomRef>>>,
    ref_collections: Arc<Mutex<HashMap<String, AtomRefCollection>>>,
}

impl AtomManager {
    pub fn new(db_ops: DbOperations) -> Self {
        let mut atoms = HashMap::new();
        let mut ref_atoms = HashMap::new();
        let mut ref_collections = HashMap::new();

        for result in db_ops.db().iter().flatten() {
            let key_str = String::from_utf8_lossy(result.0.as_ref());

            if key_str.starts_with("atom:") {
                if let Ok(atom) = serde_json::from_slice(result.1.as_ref()) {
                    atoms.insert(key_str.into_owned(), atom);
                }
            } else if key_str.starts_with("ref:") {
                if let Ok(atom_ref) = serde_json::from_slice::<AtomRef>(result.1.as_ref()) {
                    ref_atoms.insert(key_str.into_owned(), atom_ref);
                } else if let Ok(collection) =
                    serde_json::from_slice::<AtomRefCollection>(result.1.as_ref())
                {
                    ref_collections.insert(key_str.into_owned(), collection);
                }
            }
        }

        Self {
            db_ops: Arc::new(db_ops),
            atoms: Arc::new(Mutex::new(atoms)),
            ref_atoms: Arc::new(Mutex::new(ref_atoms)),
            ref_collections: Arc::new(Mutex::new(ref_collections)),
        }
    }

    pub fn get_latest_atom(&self, aref_uuid: &str) -> Result<Atom, Box<dyn std::error::Error>> {
        // Try in-memory cache first
        let ref_atoms = self.ref_atoms.lock().unwrap();
        let atoms = self.atoms.lock().unwrap();

        if let Some(aref) = ref_atoms.get(aref_uuid) {
            if let Some(atom) = atoms.get(aref.get_atom_uuid()) {
                return Ok(atom.clone());
            }
        }

        // Try from disk
        let aref = self
            .db_ops
            .get_item::<AtomRef>(aref_uuid)?
            .ok_or("AtomRef not found")?;

        let atom = self
            .db_ops
            .get_item::<Atom>(aref.get_atom_uuid())?
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
                .get_item::<Atom>(prev_uuid)?
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
        self.atoms
            .lock()
            .unwrap()
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
        self.ref_atoms
            .lock()
            .unwrap()
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
        self.ref_collections
            .lock()
            .unwrap()
            .insert(aref_uuid.to_string(), collection.clone());
        Ok(collection)
    }

    pub fn get_ref_atoms(&self) -> Arc<Mutex<HashMap<String, AtomRef>>> {
        Arc::clone(&self.ref_atoms)
    }

    pub fn get_ref_collections(&self) -> Arc<Mutex<HashMap<String, AtomRefCollection>>> {
        Arc::clone(&self.ref_collections)
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
        }
    }
}
