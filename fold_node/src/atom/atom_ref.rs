use crate::atom::atom_ref_behavior::AtomRefBehavior;
use crate::atom::atom_ref_types::{AtomRefStatus, AtomRefUpdate};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
