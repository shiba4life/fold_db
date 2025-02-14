use super::fields::SchemaField;
use crate::fees::SchemaPaymentConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
    pub fields: HashMap<String, SchemaField>,
    pub payment_config: SchemaPaymentConfig,
}

impl Schema {
    #[must_use]
    pub fn new(name: String) -> Self {
        Self {
            name,
            fields: HashMap::new(),
            payment_config: SchemaPaymentConfig::default(),
        }
    }

    pub fn with_fields(mut self, fields: HashMap<String, SchemaField>) -> Self {
        self.fields = fields;
        self
    }

    pub fn with_payment_config(mut self, payment_config: SchemaPaymentConfig) -> Self {
        self.payment_config = payment_config;
        self
    }

    pub fn add_field(&mut self, field_name: String, field: SchemaField) {
        self.fields.insert(field_name, field);
    }
}
