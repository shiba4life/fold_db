use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use serde_json::Value;
#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct Atom {
    uuid: String,
    source_schema_name: String,
    source_pub_key: String,
    created_at: DateTime<Utc>,
    prev_atom_uuid: Option<String>,
    content: Value,
}

impl Atom {
    pub fn new(source_schema_name: String, source_pub_key: String, prev_atom_uuid: Option<String>, content: Value) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            source_schema_name,
            source_pub_key,
            created_at: Utc::now(),
            prev_atom_uuid,
            content,
        }
    }

    pub fn content(&self) -> &Value {
        &self.content
    }

    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    pub fn source_schema_name(&self) -> &str {
        &self.source_schema_name
    }

    pub fn source_pub_key(&self) -> &str {
        &self.source_pub_key
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn prev_atom_uuid(&self) -> Option<&String> {
        self.prev_atom_uuid.as_ref()
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomRef {
    uuid: String,
    atom_uuid: Option<String>,
    updated_at: DateTime<Utc>,
}

impl AtomRef {
    pub fn new(atom_uuid: String) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            atom_uuid: Some(atom_uuid),
            updated_at: Utc::now(),
        }
    }

    pub fn set_atom_uuid(&mut self, atom_uuid: String) {
        self.atom_uuid = Some(atom_uuid);
    }

    pub fn get_atom_uuid(&self) -> Option<&String> {
        self.atom_uuid.as_ref()
    }

    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
