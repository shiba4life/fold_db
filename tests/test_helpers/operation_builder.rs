use fold_db::schema::types::{Mutation, Query};
use serde_json::Value;
use std::collections::HashMap;

pub fn create_query(
    schema_name: String,
    fields: Vec<String>,
    pub_key: String,
    trust_distance: u32,
) -> Query {
    Query {
        schema_name,
        fields,
        pub_key,
        trust_distance,
    }
}

pub fn create_mutation(
    schema_name: String,
    fields_and_values: HashMap<String, Value>,
    pub_key: String,
    trust_distance: u32,
) -> Mutation {
    Mutation {
        schema_name,
        fields_and_values,
        pub_key,
        trust_distance,
    }
}

pub fn create_single_field_mutation(
    schema_name: String,
    field: String,
    value: Value,
    pub_key: String,
    trust_distance: u32,
) -> Mutation {
    let mut fields_and_values = HashMap::new();
    fields_and_values.insert(field, value);

    Mutation {
        schema_name,
        fields_and_values,
        pub_key,
        trust_distance,
    }
}
