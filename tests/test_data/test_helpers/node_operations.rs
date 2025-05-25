use fold_node::testing::{Mutation, MutationType, Query, Schema};
use fold_node::{DataFoldNode, FoldDbResult};
use serde_json::Value;
use std::collections::HashMap;

/// Load a schema into the node and allow access.
#[allow(dead_code)]
pub fn load_and_allow(node: &mut DataFoldNode, schema: Schema) -> FoldDbResult<()> {
    let name = schema.name.clone();
    node.load_schema(schema)?;
    node.allow_schema(&name)?;
    Ok(())
}

/// Create a new node with the given schema loaded and allowed.
#[allow(dead_code)]
pub fn create_node_with_schema(schema: Schema) -> DataFoldNode {
    let mut node = super::create_test_node();
    load_and_allow(&mut node, schema).expect("failed to load schema");
    node
}

/// Execute a single field create mutation on the node.
#[allow(dead_code)]
pub fn insert_value(
    node: &mut DataFoldNode,
    schema: &str,
    field: &str,
    value: Value,
) -> FoldDbResult<()> {
    let mut fields = HashMap::new();
    fields.insert(field.to_string(), value);
    let mutation = Mutation {
        mutation_type: MutationType::Create,
        schema_name: schema.to_string(),
        pub_key: "test_key".to_string(),
        trust_distance: 1,
        fields_and_values: fields,
    };
    node.mutate(mutation)
}

/// Query a single field from the node and return the value.
#[allow(dead_code)]
pub fn query_value(
    node: &mut DataFoldNode,
    schema: &str,
    field: &str,
) -> FoldDbResult<Value> {
    let query = Query {
        schema_name: schema.to_string(),
        fields: vec![field.to_string()],
        pub_key: "test_key".to_string(),
        trust_distance: 1,
        filter: None,
    };
    let mut results = node.query(query)?;
    results
        .remove(0)
        .map_err(|e| e.into())
}

