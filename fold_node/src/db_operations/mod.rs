use crate::atom::{Atom, AtomRef, AtomRefCollection, AtomStatus};
use crate::schema::SchemaError;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

/// Helper struct for database operations
pub struct DbOperations {
    /// The underlying sled database instance
    pub(crate) db: sled::Db,
}

impl DbOperations {
    /// Creates a new DbOperations instance
    pub fn new(db: sled::Db) -> Self {
        Self { db }
    }

    /// Gets a reference to the underlying database
    pub fn db(&self) -> &sled::Db {
        &self.db
    }

    /// Generic function to store a serializable item in the database
    pub fn store_item<T: Serialize>(&self, key: &str, item: &T) -> Result<(), SchemaError> {
        let bytes = serde_json::to_vec(item)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize item: {}", e)))?;

        self.db
            .insert(key.as_bytes(), bytes)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to store item: {}", e)))?;

        Ok(())
    }

    /// Generic function to retrieve a deserializable item from the database
    pub fn get_item<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, SchemaError> {
        match self.db.get(key.as_bytes()) {
            Ok(Some(bytes)) => {
                let item = serde_json::from_slice(&bytes).map_err(|e| {
                    SchemaError::InvalidData(format!("Failed to deserialize item: {}", e))
                })?;
                Ok(Some(item))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(SchemaError::InvalidData(format!(
                "Failed to retrieve item: {}",
                e
            ))),
        }
    }

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
        self.store_item(atom.uuid(), &atom)?;
        Ok(atom)
    }

    /// Creates or updates a single atom reference
    pub fn update_atom_ref(
        &self,
        aref_uuid: &str,
        atom_uuid: String,
        source_pub_key: String,
    ) -> Result<AtomRef, SchemaError> {
        let mut aref = match self.get_item::<AtomRef>(aref_uuid)? {
            Some(existing_aref) => existing_aref,
            None => AtomRef::new(atom_uuid.clone(), source_pub_key),
        };

        aref.set_atom_uuid(atom_uuid);
        self.store_item(aref_uuid, &aref)?;
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
        let mut aref = match self.get_item::<AtomRefCollection>(aref_uuid)? {
            Some(existing_aref) => existing_aref,
            None => AtomRefCollection::new(source_pub_key),
        };

        aref.set_atom_uuid(key, atom_uuid);
        self.store_item(aref_uuid, &aref)?;
        Ok(aref)
    }
}
