use super::core::DbOperations;
use crate::atom::{Atom, AtomRef, AtomRefCollection, AtomRefRange, AtomStatus};
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
        let mut aref = match self.get_item::<AtomRef>(&format!("ref:{}", aref_uuid))? {
            Some(existing_aref) => existing_aref,
            None => AtomRef::new(atom_uuid.clone(), source_pub_key),
        };

        aref.set_atom_uuid(atom_uuid);
        self.store_item(&format!("ref:{}", aref_uuid), &aref)?;
        Ok(aref)
    }

    /// Creates or updates a collection of atom references
    pub fn update_atom_ref_collection(
        &self,
        aref_uuid: &str,
        atom_uuid: String,
        key: String,
        source_pub_key: String,
    ) -> Result<AtomRefCollection, SchemaError> {
        let mut aref = match self.get_item::<AtomRefCollection>(&format!("ref:{}", aref_uuid))? {
            Some(existing_aref) => existing_aref,
            None => AtomRefCollection::new(source_pub_key),
        };

        aref.set_atom_uuid(key, atom_uuid);
        self.store_item(&format!("ref:{}", aref_uuid), &aref)?;
        Ok(aref)
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
