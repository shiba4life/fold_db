use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub schema_name: String,
    pub fields: Vec<String>,
    pub pub_key: String,
    pub trust_distance: u32,
}

impl Query {
    pub fn new(schema_name: String, fields: Vec<String>, pub_key: String, trust_distance: u32) -> Self {
        Self {
            schema_name,
            fields,
            pub_key,
            trust_distance,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mutation {
    pub schema_name: String,
    pub fields_and_values: HashMap<String, Value>,
    pub pub_key: String,
    pub trust_distance: u32,
}

impl Mutation {
    pub fn new(schema_name: String, fields_and_values: HashMap<String, Value>, pub_key: String, trust_distance: u32) -> Self {
        Self {
            schema_name,
            fields_and_values,
            pub_key,
            trust_distance,
        }
    }
}
