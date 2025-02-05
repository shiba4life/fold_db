use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Atom {
    uuid: String,
    content: String,
    source: String,
    created_at: DateTime<Utc>,
    prev_atom: Option<String>,
}

impl Atom {
    pub fn new(content: String, source: String, prev_atom: Option<String>) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            content,
            source,
            created_at: Utc::now(),
            prev_atom,
        }
    }

    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn prev_atom(&self) -> Option<&String> {
        self.prev_atom.as_ref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomRef {
    uuid: String,
    atom_uuid: String,
    updated_at: DateTime<Utc>,
    update_source: String,
}

impl AtomRef {
    pub fn new(atom_uuid: String, update_source: String) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            atom_uuid,
            updated_at: Utc::now(),
            update_source,
        }
    }

    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    pub fn atom_uuid(&self) -> &str {
        &self.atom_uuid
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn update_source(&self) -> &str {
        &self.update_source
    }
}
