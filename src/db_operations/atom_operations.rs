use super::core::DbOperations;
use super::encryption_wrapper::{EncryptionWrapper, contexts};
use crate::atom::{Atom, AtomRef, AtomRefRange, AtomStatus};
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

    /// Creates a new atom and stores it with encryption in the database
    pub fn create_atom_encrypted(
        &self,
        encryption_wrapper: &EncryptionWrapper,
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
        // Store encrypted with the "atom_data" context
        encryption_wrapper.store_encrypted_item(
            &format!("atom:{}", atom.uuid()),
            &atom,
            contexts::ATOM_DATA
        )?;
        Ok(atom)
    }

    /// Retrieves an atom from the database with encryption support
    pub fn get_atom_encrypted(
        &self,
        encryption_wrapper: &EncryptionWrapper,
        atom_uuid: &str,
    ) -> Result<Option<Atom>, SchemaError> {
        encryption_wrapper.get_encrypted_item(&format!("atom:{}", atom_uuid), contexts::ATOM_DATA)
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

    /// Creates or updates a single atom reference with encryption support
    pub fn update_atom_ref_encrypted(
        &self,
        encryption_wrapper: &EncryptionWrapper,
        aref_uuid: &str,
        atom_uuid: String,
        source_pub_key: String,
    ) -> Result<AtomRef, SchemaError> {
        // DIAGNOSTIC: Log the update attempt
        log::info!("🔍 DIAGNOSTIC: update_atom_ref_encrypted called - aref_uuid: {}, atom_uuid: {}", aref_uuid, atom_uuid);
        
        let mut aref = match encryption_wrapper.get_encrypted_item::<AtomRef>(&format!("ref:{}", aref_uuid), contexts::ATOM_DATA)? {
            Some(existing_aref) => {
                log::info!("🔍 DIAGNOSTIC: Found existing encrypted AtomRef - current atom_uuid: {}", existing_aref.get_atom_uuid());
                existing_aref
            }
            None => {
                log::info!("🔍 DIAGNOSTIC: Creating new AtomRef for encryption");
                AtomRef::new(atom_uuid.clone(), source_pub_key)
            }
        };

        // DIAGNOSTIC: Log before update
        log::info!("🔍 DIAGNOSTIC: Before set_atom_uuid - current: {}, new: {}", aref.get_atom_uuid(), atom_uuid);
        
        aref.set_atom_uuid(atom_uuid.clone());
        
        // DIAGNOSTIC: Log after update
        log::info!("🔍 DIAGNOSTIC: After set_atom_uuid - updated to: {}", aref.get_atom_uuid());
        
        // DIAGNOSTIC: Log before persistence
        log::info!("🔍 DIAGNOSTIC: About to persist encrypted AtomRef with key: ref:{}", aref_uuid);
        
        encryption_wrapper.store_encrypted_item(&format!("ref:{}", aref_uuid), &aref, contexts::ATOM_DATA)?;
        
        // DIAGNOSTIC: Verify persistence by reading back
        match encryption_wrapper.get_encrypted_item::<AtomRef>(&format!("ref:{}", aref_uuid), contexts::ATOM_DATA)? {
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

    // TODO: Collection operations are no longer supported - AtomRefCollection has been removed

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

    /// Creates or updates a range of atom references with encryption support
    pub fn update_atom_ref_range_encrypted(
        &self,
        encryption_wrapper: &EncryptionWrapper,
        aref_uuid: &str,
        atom_uuid: String,
        key: String,
        source_pub_key: String,
    ) -> Result<AtomRefRange, SchemaError> {
        let mut aref = match encryption_wrapper.get_encrypted_item::<AtomRefRange>(&format!("ref:{}", aref_uuid), contexts::ATOM_DATA)? {
            Some(existing_aref) => existing_aref,
            None => AtomRefRange::new(source_pub_key),
        };

        aref.set_atom_uuid(key, atom_uuid);
        encryption_wrapper.store_encrypted_item(&format!("ref:{}", aref_uuid), &aref, contexts::ATOM_DATA)?;
        Ok(aref)
    }
}
