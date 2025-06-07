use crate::atom::atom_ref_behavior::AtomRefBehavior;
use crate::atom::atom_ref_types::{AtomRefStatus, AtomRefUpdate};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// AtomRefCollection stores a list of atom UUIDs for collection-type fields.
/// It provides an ordered list of references to atoms, suitable for arrays,
/// lists, or other sequential data structures in schemas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomRefCollection {
    uuid: String,
    /// Ordered list of atom UUIDs in this collection
    pub atom_uuids: Vec<String>,
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
            atom_uuids: Vec::new(),
            updated_at: Utc::now(),
            status: AtomRefStatus::Active,
            update_history: vec![AtomRefUpdate {
                timestamp: Utc::now(),
                status: AtomRefStatus::Active,
                source_pub_key,
            }],
        }
    }

    /// Adds an atom UUID to the end of the collection.
    pub fn add_atom_uuid(&mut self, atom_uuid: String, source_pub_key: String) {
        self.atom_uuids.push(atom_uuid);
        self.updated_at = Utc::now();
        self.update_history.push(AtomRefUpdate {
            timestamp: Utc::now(),
            status: self.status.clone(),
            source_pub_key,
        });
    }

    /// Removes the first occurrence of the specified atom UUID from the collection.
    pub fn remove_atom_uuid(&mut self, atom_uuid: &str, source_pub_key: String) -> bool {
        if let Some(pos) = self.atom_uuids.iter().position(|x| x == atom_uuid) {
            self.atom_uuids.remove(pos);
            self.updated_at = Utc::now();
            self.update_history.push(AtomRefUpdate {
                timestamp: Utc::now(),
                status: self.status.clone(),
                source_pub_key,
            });
            true
        } else {
            false
        }
    }

    /// Inserts an atom UUID at the specified index.
    pub fn insert_atom_uuid(&mut self, index: usize, atom_uuid: String, source_pub_key: String) -> Result<(), String> {
        if index > self.atom_uuids.len() {
            return Err(format!("Index {} out of bounds for collection of length {}", index, self.atom_uuids.len()));
        }
        
        self.atom_uuids.insert(index, atom_uuid);
        self.updated_at = Utc::now();
        self.update_history.push(AtomRefUpdate {
            timestamp: Utc::now(),
            status: self.status.clone(),
            source_pub_key,
        });
        Ok(())
    }

    /// Replaces the atom UUID at the specified index.
    pub fn set_atom_uuid(&mut self, index: usize, atom_uuid: String, source_pub_key: String) -> Result<(), String> {
        if index >= self.atom_uuids.len() {
            return Err(format!("Index {} out of bounds for collection of length {}", index, self.atom_uuids.len()));
        }
        
        self.atom_uuids[index] = atom_uuid;
        self.updated_at = Utc::now();
        self.update_history.push(AtomRefUpdate {
            timestamp: Utc::now(),
            status: self.status.clone(),
            source_pub_key,
        });
        Ok(())
    }

    /// Gets the atom UUID at the specified index.
    pub fn get_atom_uuid_at(&self, index: usize) -> Option<&String> {
        self.atom_uuids.get(index)
    }

    /// Returns the number of atom UUIDs in the collection.
    pub fn len(&self) -> usize {
        self.atom_uuids.len()
    }

    /// Returns true if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.atom_uuids.is_empty()
    }

    /// Clears all atom UUIDs from the collection.
    pub fn clear(&mut self, source_pub_key: String) {
        self.atom_uuids.clear();
        self.updated_at = Utc::now();
        self.update_history.push(AtomRefUpdate {
            timestamp: Utc::now(),
            status: self.status.clone(),
            source_pub_key,
        });
    }

    /// Returns an iterator over the atom UUIDs.
    pub fn iter(&self) -> std::slice::Iter<String> {
        self.atom_uuids.iter()
    }

    /// Returns true if the collection contains the specified atom UUID.
    pub fn contains(&self, atom_uuid: &str) -> bool {
        self.atom_uuids.contains(&atom_uuid.to_string())
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

impl AtomRefCollection {
    /// Get the first atom UUID in the collection (for compatibility with AtomRef interface)
    pub fn get_atom_uuid(&self) -> &str {
        // For collections, return the first atom UUID if available
        self.atom_uuids.first().map(|s| s.as_str()).unwrap_or("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_collection() {
        let collection = AtomRefCollection::new("test_key".to_string());
        assert!(collection.is_empty());
        assert_eq!(collection.len(), 0);
        assert_eq!(collection.status(), &AtomRefStatus::Active);
    }

    #[test]
    fn test_add_atom_uuid() {
        let mut collection = AtomRefCollection::new("test_key".to_string());
        collection.add_atom_uuid("atom1".to_string(), "test_key".to_string());
        collection.add_atom_uuid("atom2".to_string(), "test_key".to_string());
        
        assert_eq!(collection.len(), 2);
        assert_eq!(collection.get_atom_uuid_at(0), Some(&"atom1".to_string()));
        assert_eq!(collection.get_atom_uuid_at(1), Some(&"atom2".to_string()));
        assert!(collection.contains("atom1"));
        assert!(collection.contains("atom2"));
    }

    #[test]
    fn test_remove_atom_uuid() {
        let mut collection = AtomRefCollection::new("test_key".to_string());
        collection.add_atom_uuid("atom1".to_string(), "test_key".to_string());
        collection.add_atom_uuid("atom2".to_string(), "test_key".to_string());
        
        assert!(collection.remove_atom_uuid("atom1", "test_key".to_string()));
        assert_eq!(collection.len(), 1);
        assert_eq!(collection.get_atom_uuid_at(0), Some(&"atom2".to_string()));
        assert!(!collection.contains("atom1"));
        assert!(collection.contains("atom2"));
        
        // Try to remove non-existent atom
        assert!(!collection.remove_atom_uuid("atom3", "test_key".to_string()));
    }

    #[test]
    fn test_insert_atom_uuid() {
        let mut collection = AtomRefCollection::new("test_key".to_string());
        collection.add_atom_uuid("atom1".to_string(), "test_key".to_string());
        collection.add_atom_uuid("atom3".to_string(), "test_key".to_string());
        
        // Insert at index 1
        assert!(collection.insert_atom_uuid(1, "atom2".to_string(), "test_key".to_string()).is_ok());
        assert_eq!(collection.len(), 3);
        assert_eq!(collection.get_atom_uuid_at(0), Some(&"atom1".to_string()));
        assert_eq!(collection.get_atom_uuid_at(1), Some(&"atom2".to_string()));
        assert_eq!(collection.get_atom_uuid_at(2), Some(&"atom3".to_string()));
        
        // Try to insert at invalid index
        assert!(collection.insert_atom_uuid(10, "atom4".to_string(), "test_key".to_string()).is_err());
    }

    #[test]
    fn test_set_atom_uuid() {
        let mut collection = AtomRefCollection::new("test_key".to_string());
        collection.add_atom_uuid("atom1".to_string(), "test_key".to_string());
        collection.add_atom_uuid("atom2".to_string(), "test_key".to_string());
        
        // Replace at index 1
        assert!(collection.set_atom_uuid(1, "atom_new".to_string(), "test_key".to_string()).is_ok());
        assert_eq!(collection.len(), 2);
        assert_eq!(collection.get_atom_uuid_at(0), Some(&"atom1".to_string()));
        assert_eq!(collection.get_atom_uuid_at(1), Some(&"atom_new".to_string()));
        
        // Try to set at invalid index
        assert!(collection.set_atom_uuid(10, "atom4".to_string(), "test_key".to_string()).is_err());
    }

    #[test]
    fn test_clear() {
        let mut collection = AtomRefCollection::new("test_key".to_string());
        collection.add_atom_uuid("atom1".to_string(), "test_key".to_string());
        collection.add_atom_uuid("atom2".to_string(), "test_key".to_string());
        
        collection.clear("test_key".to_string());
        assert!(collection.is_empty());
        assert_eq!(collection.len(), 0);
    }

    #[test]
    fn test_update_history() {
        let mut collection = AtomRefCollection::new("test_key".to_string());
        let initial_history_len = collection.update_history().len();
        
        collection.add_atom_uuid("atom1".to_string(), "test_key".to_string());
        assert_eq!(collection.update_history().len(), initial_history_len + 1);
        
        collection.set_status(&AtomRefStatus::Active, "test_key".to_string());
        assert_eq!(collection.update_history().len(), initial_history_len + 2);
    }
}