use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::fields::SchemaField;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
    pub fields: HashMap<String, SchemaField>,
    pub transforms: Vec<String>, // Transform names/identifiers
}

impl Schema {
    pub fn new(name: String) -> Self {
        Self {
            name,
            fields: HashMap::new(),
            transforms: Vec::new(),
        }
    }

    pub fn add_field(&mut self, field_name: String, field: SchemaField) {
        self.fields.insert(field_name, field);
    }

    pub fn add_transform(&mut self, transform: String) {
        self.transforms.push(transform);
    }
}
