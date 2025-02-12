use super::fields::SchemaField;
use crate::fees::SchemaPaymentConfig;
use crate::schema::mapper::SchemaMapper;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
    pub fields: HashMap<String, SchemaField>,
    pub schema_mappers: Vec<SchemaMapper>,
    pub payment_config: SchemaPaymentConfig,
}

impl Schema {
    #[must_use]
    pub fn new(name: String) -> Self {
        Self {
            name,
            fields: HashMap::new(),
            schema_mappers: Vec::new(),
            payment_config: SchemaPaymentConfig::default(),
        }
    }

    pub fn add_field(&mut self, field_name: String, field: SchemaField) {
        self.fields.insert(field_name, field);
    }

    pub fn add_schema_mapper(&mut self, mapper: SchemaMapper) {
        self.schema_mappers.push(mapper);
    }
}
