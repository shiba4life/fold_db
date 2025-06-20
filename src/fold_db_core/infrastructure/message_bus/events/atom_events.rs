use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::EventType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldValueSet {
    pub field: String,
    pub value: Value,
    pub source: String,
}

impl EventType for FieldValueSet {
    fn type_id() -> &'static str {
        "FieldValueSet"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomCreated {
    pub atom_id: String,
    pub data: Value,
}

impl EventType for AtomCreated {
    fn type_id() -> &'static str {
        "AtomCreated"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomUpdated {
    pub atom_id: String,
    pub data: Value,
}

impl EventType for AtomUpdated {
    fn type_id() -> &'static str {
        "AtomUpdated"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomRefCreated {
    pub aref_uuid: String,
    pub aref_type: String,
    pub field_path: String,
}

impl EventType for AtomRefCreated {
    fn type_id() -> &'static str {
        "AtomRefCreated"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomRefUpdated {
    pub aref_uuid: String,
    pub field_path: String,
    pub operation: String,
}

impl EventType for AtomRefUpdated {
    fn type_id() -> &'static str {
        "AtomRefUpdated"
    }
}

