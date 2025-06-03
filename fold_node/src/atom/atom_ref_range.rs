use crate::atom::atom_ref_behavior::AtomRefBehavior;
use crate::atom::atom_ref_types::{AtomRefStatus, AtomRefUpdate};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;
use log::info;

/// A range-based collection of atom references stored in a BTreeMap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomRefRange {
    uuid: String,
    pub(crate) atom_uuids: BTreeMap<String, String>,
    updated_at: DateTime<Utc>,
    status: AtomRefStatus,
    update_history: Vec<AtomRefUpdate>,
}

impl AtomRefRange {
    /// Creates a new empty AtomRefRange.
    #[must_use]
    pub fn new(source_pub_key: String) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            atom_uuids: BTreeMap::new(),
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
    /// If the key already exists, the atom_uuid replaces the existing value.
    pub fn set_atom_uuid(&mut self, key: String, atom_uuid: String) {
        println!("🔑 Setting atom_uuid for aref_uuid: {} -> key: {} -> atom: {}", self.uuid, key, atom_uuid);
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

impl AtomRefBehavior for AtomRefRange {
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
