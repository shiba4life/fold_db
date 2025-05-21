use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AtomRefStatus {
    Active,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomRefUpdate {
    timestamp: DateTime<Utc>,
    status: AtomRefStatus,
    source_pub_key: String,
}

/// A trait defining the common behavior for atom references.
///
/// This trait provides the interface for both single atom references
/// and collections of atom references.
pub trait AtomRefBehavior {
    /// Returns the unique identifier of this reference
    fn uuid(&self) -> &str;

    /// Returns the timestamp of the last update
    fn updated_at(&self) -> DateTime<Utc>;

    /// Returns the status of this reference
    fn status(&self) -> &AtomRefStatus;

    /// Sets the status of this reference
    fn set_status(&mut self, status: &AtomRefStatus, source_pub_key: String);

    /// Returns the update history
    fn update_history(&self) -> &Vec<AtomRefUpdate>;
}

/// A reference to a single atom version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomRef {
    uuid: String,
    atom_uuid: String,
    updated_at: DateTime<Utc>,
    status: AtomRefStatus,
    update_history: Vec<AtomRefUpdate>,
}

impl AtomRef {
    /// Creates a new AtomRef pointing to the specified Atom.
    #[must_use]
    pub fn new(atom_uuid: String, source_pub_key: String) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            atom_uuid,
            updated_at: Utc::now(),
            status: AtomRefStatus::Active,
            update_history: vec![AtomRefUpdate {
                timestamp: Utc::now(),
                status: AtomRefStatus::Active,
                source_pub_key,
            }],
        }
    }

    /// Updates the reference to point to a new Atom version.
    pub fn set_atom_uuid(&mut self, atom_uuid: String) {
        self.atom_uuid = atom_uuid;
        self.updated_at = Utc::now();
    }

    /// Returns the UUID of the referenced Atom.
    #[must_use]
    pub fn get_atom_uuid(&self) -> &String {
        &self.atom_uuid
    }
}

impl AtomRefBehavior for AtomRef {
    fn uuid(&self) -> &str {
        &self.uuid
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    fn status(&self) -> &AtomRefStatus {
        &self.status
    }

    fn set_status(&mut self, status: &AtomRefStatus, source_pub_key: String) {
        let status_clone = status.clone();
        self.status = status_clone.clone();
        self.updated_at = Utc::now();
        self.update_history.push(AtomRefUpdate {
            timestamp: Utc::now(),
            status: status_clone,
            source_pub_key,
        });
    }

    fn update_history(&self) -> &Vec<AtomRefUpdate> {
        &self.update_history
    }
}

/// A collection of atom references, each identified by a key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomRefCollection {
    uuid: String,
    atom_uuids: HashMap<String, String>,
    updated_at: DateTime<Utc>,
    status: AtomRefStatus,
    update_history: Vec<AtomRefUpdate>,
}

impl AtomRefCollection {
    /// Creates a new empty AtomRefCollection.
    #[must_use]
    pub fn new(source_pub_key: String) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            atom_uuids: HashMap::new(),
            updated_at: Utc::now(),
            status: AtomRefStatus::Active,
            update_history: vec![AtomRefUpdate {
                timestamp: Utc::now(),
                status: AtomRefStatus::Active,
                source_pub_key,
            }],
        }
    }

    /// Updates or adds a reference at the specified key.
    pub fn set_atom_uuid(&mut self, key: String, atom_uuid: String) {
        self.atom_uuids.insert(key, atom_uuid);
        self.updated_at = Utc::now();
    }

    /// Returns the UUID of the Atom referenced by the specified key.
    #[must_use]
    pub fn get_atom_uuid(&self, key: &str) -> Option<&String> {
        self.atom_uuids.get(key)
    }

    /// Removes the reference at the specified key.
    pub fn remove_atom_uuid(&mut self, key: &str) -> Option<String> {
        let result = self.atom_uuids.remove(key);
        if result.is_some() {
            self.updated_at = Utc::now();
        }
        result
    }
}

impl AtomRefBehavior for AtomRefCollection {
    fn uuid(&self) -> &str {
        &self.uuid
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    fn status(&self) -> &AtomRefStatus {
        &self.status
    }

    fn set_status(&mut self, status: &AtomRefStatus, source_pub_key: String) {
        let status_clone = status.clone();
        self.status = status_clone.clone();
        self.updated_at = Utc::now();
        self.update_history.push(AtomRefUpdate {
            timestamp: Utc::now(),
            status: status_clone,
            source_pub_key,
        });
    }

    fn update_history(&self) -> &Vec<AtomRefUpdate> {
        &self.update_history
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atom::Atom;
    use serde_json::json;

    #[test]
    fn test_atom_ref_creation_and_update() {
        use super::AtomRefBehavior;

        let atom = Atom::new(
            "test_schema".to_string(),
            "test_key".to_string(),
            json!({"test": true}),
        );

        // Test single atom ref
        let atom_ref = AtomRef::new(atom.uuid().to_string(), "test_key".to_string());
        assert_eq!(atom_ref.get_atom_uuid(), &atom.uuid().to_string());

        let new_atom = Atom::new(
            "test_schema".to_string(),
            "test_key".to_string(),
            json!({"test": false}),
        );

        let mut updated_ref = atom_ref.clone();
        updated_ref.set_atom_uuid(new_atom.uuid().to_string());

        assert_eq!(updated_ref.get_atom_uuid(), &new_atom.uuid().to_string());
        assert!(updated_ref.updated_at() >= atom_ref.updated_at());
    }

    #[test]
    fn test_atom_ref_collection() {
        use super::AtomRefBehavior;

        let atoms: Vec<_> = (0..3)
            .map(|i| {
                Atom::new(
                    "test_schema".to_string(),
                    "test_key".to_string(),
                    json!({ "index": i }),
                )
            })
            .collect();

        // Test collection usage
        let mut collection = AtomRefCollection::new("test_key".to_string());
        collection.set_atom_uuid("0".to_string(), atoms[0].uuid().to_string());
        collection.set_atom_uuid("1".to_string(), atoms[1].uuid().to_string());
        collection.set_atom_uuid("2".to_string(), atoms[2].uuid().to_string());

        assert_eq!(
            collection.get_atom_uuid("0"),
            Some(&atoms[0].uuid().to_string())
        );
        assert_eq!(
            collection.get_atom_uuid("1"),
            Some(&atoms[1].uuid().to_string())
        );
        assert_eq!(
            collection.get_atom_uuid("2"),
            Some(&atoms[2].uuid().to_string())
        );

        // Test removal
        assert_eq!(
            collection.remove_atom_uuid("1"),
            Some(atoms[1].uuid().to_string())
        );
        assert_eq!(collection.get_atom_uuid("1"), None);

        // Test behavior trait
        assert!(collection.updated_at() > Utc::now() - chrono::Duration::seconds(1));
    }

}
