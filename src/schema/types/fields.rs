use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    Single,    // Regular field with a single value
    Collection // Field containing multiple values
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaField {
    pub permission_setting: String, // X:0, W1, etc
    pub ref_atom_uuid: String,
    pub field_type: FieldType,
    pub explicit_access: HashMap<String, AccessCounts>, // pub_key -> access counts
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessCounts {
    pub w: u32,
    pub r: u32,
}

impl SchemaField {
    pub fn new(permission_setting: String, ref_atom_uuid: String, field_type: FieldType) -> Self {
        Self {
            permission_setting,
            ref_atom_uuid,
            field_type,
            explicit_access: HashMap::new(),
        }
    }

    pub fn add_explicit_access(&mut self, pub_key: String, w: u32, r: u32) {
        self.explicit_access.insert(pub_key, AccessCounts { w, r });
    }
}
