use crate::atom::atom_ref_behavior::AtomRefBehavior;
use crate::atom::atom_ref_types::{AtomRefStatus, AtomRefUpdate};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomRefHash {
    uuid: String,
    pub atom_uuids: HashMap<String, String>,
    updated_at: DateTime<Utc>,
    status: AtomRefStatus,
    update_history: Vec<AtomRefUpdate>,
}

impl AtomRefHash {
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

    pub fn set_atom_uuid(&mut self, key: String, atom_uuid: String) {
        self.atom_uuids.insert(key, atom_uuid);
        self.updated_at = Utc::now();
    }

    pub fn get_atom_uuid(&self, key: &str) -> Option<&String> {
        self.atom_uuids.get(key)
    }

    pub fn remove_atom_uuid(&mut self, key: &str) -> Option<String> {
        let res = self.atom_uuids.remove(key);
        if res.is_some() {
            self.updated_at = Utc::now();
        }
        res
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.atom_uuids.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item=&String> {
        self.atom_uuids.keys()
    }

    pub fn len(&self) -> usize {
        self.atom_uuids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.atom_uuids.is_empty()
    }

    pub fn clear(&mut self) {
        self.atom_uuids.clear();
        self.updated_at = Utc::now();
    }
}

impl AtomRefBehavior for AtomRefHash {
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
