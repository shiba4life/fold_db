use super::core::DbOperations;
use crate::atom::{Atom, AtomRef, AtomRefRange, AtomStatus, CollectionOperation, AtomRefCollection};
use crate::schema::SchemaError;
use serde_json::Value;

impl DbOperations {
    /// Creates a new atom and stores it in the database
    pub fn create_atom(
        &self,
        schema_name: &str,
        source_pub_key: String,
        prev_atom_uuid: Option<String>,
        content: Value,
        status: Option<AtomStatus>,
    ) -> Result<Atom, SchemaError> {
        let mut atom = Atom::new(schema_name.to_string(), source_pub_key, content);

        // Only set prev_atom_uuid if it's Some
        if let Some(prev_uuid) = prev_atom_uuid {
            if !prev_uuid.is_empty() {
                atom = atom.with_prev_version(prev_uuid);
            }
        }

        atom = atom.with_status(status.unwrap_or(AtomStatus::Active));
        // Persist with an "atom:" prefix so we can easily distinguish entries
        // when reloading from disk
        self.store_item(&format!("atom:{}", atom.uuid()), &atom)?;
        Ok(atom)
    }

    /// Creates or updates a single atom reference
    pub fn update_atom_ref(
        &self,
        aref_uuid: &str,
        atom_uuid: String,
        source_pub_key: String,
    ) -> Result<AtomRef, SchemaError> {
        // DIAGNOSTIC: Log the update attempt
        log::info!("🔍 DIAGNOSTIC: update_atom_ref called - aref_uuid: {}, atom_uuid: {}", aref_uuid, atom_uuid);
        
        let mut aref = match self.get_item::<AtomRef>(&format!("ref:{}", aref_uuid))? {
            Some(existing_aref) => {
                log::info!("🔍 DIAGNOSTIC: Found existing AtomRef - current atom_uuid: {}", existing_aref.get_atom_uuid());
                existing_aref
            }
            None => {
                log::info!("🔍 DIAGNOSTIC: Creating new AtomRef");
                AtomRef::new(atom_uuid.clone(), source_pub_key)
            }
        };

        // DIAGNOSTIC: Log before update
        log::info!("🔍 DIAGNOSTIC: Before set_atom_uuid - current: {}, new: {}", aref.get_atom_uuid(), atom_uuid);
        
        aref.set_atom_uuid(atom_uuid.clone());
        
        // DIAGNOSTIC: Log after update
        log::info!("🔍 DIAGNOSTIC: After set_atom_uuid - updated to: {}", aref.get_atom_uuid());
        
        // DIAGNOSTIC: Log before persistence
        log::info!("🔍 DIAGNOSTIC: About to persist AtomRef with key: ref:{}", aref_uuid);
        
        self.store_item(&format!("ref:{}", aref_uuid), &aref)?;
        
        // DIAGNOSTIC: Verify persistence by reading back
        match self.get_item::<AtomRef>(&format!("ref:{}", aref_uuid))? {
            Some(persisted_aref) => {
                log::info!("🔍 DIAGNOSTIC: Persistence verification - stored atom_uuid: {}", persisted_aref.get_atom_uuid());
                if persisted_aref.get_atom_uuid() != &atom_uuid {
                    log::error!("❌ DIAGNOSTIC: PERSISTENCE MISMATCH! Expected: {}, Got: {}", atom_uuid, persisted_aref.get_atom_uuid());
                }
            }
            None => {
                log::error!("❌ DIAGNOSTIC: PERSISTENCE FAILED! Could not read back stored AtomRef");
            }
        }
        
        Ok(aref)
    }

    /// Creates or updates a collection of atom references
    pub fn update_atom_ref_collection(
        &self,
        aref_uuid: &str,
        operation: CollectionOperation,
        source_pub_key: String,
    ) -> Result<AtomRefCollection, SchemaError> {
        let mut collection = match self.get_item::<AtomRefCollection>(&format!("ref:{}", aref_uuid))? {
            Some(existing_collection) => existing_collection,
            None => AtomRefCollection::new(source_pub_key.clone()),
        };

        match operation {
            CollectionOperation::Add { atom_uuid } => {
                collection.add_atom_uuid(atom_uuid, source_pub_key);
            }
            CollectionOperation::Remove { atom_uuid } => {
                collection.remove_atom_uuid(&atom_uuid, source_pub_key);
            }
            CollectionOperation::Insert { index, atom_uuid } => {
                collection.insert_atom_uuid(index, atom_uuid, source_pub_key)
                    .map_err(|e| SchemaError::InvalidData(e))?;
            }
            CollectionOperation::UpdateByIndex { index, atom_uuid } => {
                collection.set_atom_uuid(index, atom_uuid, source_pub_key)
                    .map_err(|e| SchemaError::InvalidData(e))?;
            }
            CollectionOperation::Clear => {
                collection.clear(source_pub_key);
            }
        }

        self.store_item(&format!("ref:{}", aref_uuid), &collection)?;
        Ok(collection)
    }

    /// Creates or updates a range of atom references
    pub fn update_atom_ref_range(
        &self,
        aref_uuid: &str,
        atom_uuid: String,
        key: String,
        source_pub_key: String,
    ) -> Result<AtomRefRange, SchemaError> {
        let mut aref = match self.get_item::<AtomRefRange>(&format!("ref:{}", aref_uuid))? {
            Some(existing_aref) => existing_aref,
            None => AtomRefRange::new(source_pub_key),
        };

        aref.set_atom_uuid(key, atom_uuid);
        self.store_item(&format!("ref:{}", aref_uuid), &aref)?;
        Ok(aref)
    }
}
