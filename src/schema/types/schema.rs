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

    pub fn with_fields(mut self, fields: HashMap<String, SchemaField>) -> Self {
        self.fields = fields;
        self
    }

    pub fn with_schema_mappers(mut self, schema_mappers: Vec<SchemaMapper>) -> Self {
        self.schema_mappers = schema_mappers;
        self
    }

    pub fn with_payment_config(mut self, payment_config: SchemaPaymentConfig) -> Self {
        self.payment_config = payment_config;
        self
    }

    pub fn add_field(&mut self, field_name: String, field: SchemaField) {
        self.fields.insert(field_name, field);
    }

    pub fn add_schema_mapper(&mut self, mapper: SchemaMapper) {
        // check that the mapper's source schema is not this schema name.
        if mapper.source_schema_name == self.name {
            panic!("Mapper source schema cannot be the same as the target schema");
        }
        self.schema_mappers.push(mapper);
    }
}
