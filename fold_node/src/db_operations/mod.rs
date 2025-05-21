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

        // Ensure the data is durably written to disk
        self.db
            .flush()
            .map_err(|e| SchemaError::InvalidData(format!("Failed to flush db: {}", e)))?;

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
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::atom::AtomRefBehavior;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        value: String,
    }

    fn create_temp_db() -> DbOperations {
        let db = sled::Config::new().temporary(true).open().unwrap();
        DbOperations::new(db)
    }

    #[test]
    fn test_store_and_get_item() {
        let db_ops = create_temp_db();
        let item = TestStruct { value: "hello".to_string() };
        db_ops.store_item("key1", &item).unwrap();
        let retrieved: Option<TestStruct> = db_ops.get_item("key1").unwrap();
        assert_eq!(retrieved, Some(item));
    }

    #[test]
    fn test_create_atom_persists() {
        let db_ops = create_temp_db();
        let content = json!({"field": 1});
        let atom = db_ops
            .create_atom("TestSchema", "owner".to_string(), None, content.clone(), None)
            .unwrap();
        let stored: Option<Atom> = db_ops.get_item(&format!("atom:{}", atom.uuid())).unwrap();
        assert!(stored.is_some());
        assert_eq!(stored.unwrap().content(), &content);
    }

    #[test]
    fn test_update_atom_ref_persists() {
        let db_ops = create_temp_db();
        let atom1 = db_ops
            .create_atom("TestSchema", "owner".to_string(), None, json!({"v": 1}), None)
            .unwrap();
        let mut aref = db_ops
            .update_atom_ref("ref1", atom1.uuid().to_string(), "owner".to_string())
            .unwrap();
        assert_eq!(aref.get_atom_uuid(), &atom1.uuid().to_string());

        let atom2 = db_ops
            .create_atom("TestSchema", "owner".to_string(), None, json!({"v": 2}), None)
            .unwrap();
        aref = db_ops
            .update_atom_ref("ref1", atom2.uuid().to_string(), "owner".to_string())
            .unwrap();

        let stored: Option<AtomRef> = db_ops.get_item("ref:ref1").unwrap();
        let stored = stored.unwrap();
        assert_eq!(stored.uuid(), aref.uuid());
        assert_eq!(stored.get_atom_uuid(), &atom2.uuid().to_string());
    }

    #[test]
    fn test_update_atom_ref_collection_persists() {
        let db_ops = create_temp_db();
        let atom1 = db_ops
            .create_atom("TestSchema", "owner".to_string(), None, json!({"idx": 1}), None)
            .unwrap();
        let atom2 = db_ops
            .create_atom("TestSchema", "owner".to_string(), None, json!({"idx": 2}), None)
            .unwrap();

        let mut collection = db_ops
            .update_atom_ref_collection(
                "col1",
                atom1.uuid().to_string(),
                "a".to_string(),
                "owner".to_string(),
            )
            .unwrap();
        assert_eq!(collection.get_atom_uuid("a"), Some(&atom1.uuid().to_string()));

        collection = db_ops
            .update_atom_ref_collection(
                "col1",
                atom2.uuid().to_string(),
                "b".to_string(),
                "owner".to_string(),
            )
            .unwrap();

        let stored: Option<AtomRefCollection> = db_ops.get_item("ref:col1").unwrap();
        let stored = stored.unwrap();
        assert_eq!(stored.uuid(), collection.uuid());
        assert_eq!(stored.get_atom_uuid("a"), Some(&atom1.uuid().to_string()));
        assert_eq!(stored.get_atom_uuid("b"), Some(&atom2.uuid().to_string()));
    }

    #[test]
    fn test_persistence_across_reopen() {
        // Use a temporary directory so the DB persists across instances
        let dir = tempfile::tempdir().unwrap();
        let db = sled::open(dir.path()).unwrap();
        let db_ops = DbOperations::new(db);

        let atom = db_ops
            .create_atom("TestSchema", "owner".to_string(), None, json!({"v": 1}), None)
            .unwrap();
        let _aref = db_ops
            .update_atom_ref("ref_persist", atom.uuid().to_string(), "owner".to_string())
            .unwrap();

        // Drop first instance to close the database
        drop(db_ops);

        // Re-open the database and verify the items exist
        let db2 = sled::open(dir.path()).unwrap();
        let db_ops2 = DbOperations::new(db2);
        let stored_atom: Option<Atom> = db_ops2
            .get_item(&format!("atom:{}", atom.uuid()))
            .unwrap();
        let stored_aref: Option<AtomRef> = db_ops2.get_item("ref:ref_persist").unwrap();

        assert!(stored_atom.is_some());
        assert!(stored_aref.is_some());
        assert_eq!(stored_aref.unwrap().get_atom_uuid(), &atom.uuid().to_string());
    }
}
