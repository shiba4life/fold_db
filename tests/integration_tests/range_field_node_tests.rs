use fold_node::testing::{Mutation, MutationType};
use serde_json::json;
use crate::test_data::schema_test_data::create_range_field_schema;
use crate::test_data::test_helpers::create_test_node;

#[test]
fn test_range_field_mutations_and_queries() {
    let mut node = create_test_node();
    let schema = create_range_field_schema();
    crate::test_data::test_helpers::node_operations::load_and_allow(&mut node, schema).unwrap();

    // insert first value
    let mut fields = std::collections::HashMap::new();
    fields.insert("numbers".to_string(), json!({"key": "a", "value": 1}));
    let mut1 = Mutation::new("range_schema".to_string(), fields, String::new(), 0, MutationType::Create);
    node.mutate(mut1).unwrap();

    let val = crate::test_data::test_helpers::node_operations::query_value(&mut node, "range_schema", "numbers").unwrap();
    assert_eq!(val, json!({"a": 1}));

    // update existing key
    let mut fields = std::collections::HashMap::new();
    fields.insert("numbers".to_string(), json!({"key": "a", "value": 2}));
    let mut2 = Mutation::new("range_schema".to_string(), fields, String::new(), 0, MutationType::Update);
    node.mutate(mut2).unwrap();

    let val = crate::test_data::test_helpers::node_operations::query_value(&mut node, "range_schema", "numbers").unwrap();
    assert_eq!(val, json!({"a": 2}));

    // add second key
    let mut fields = std::collections::HashMap::new();
    fields.insert("numbers".to_string(), json!({"key": "b", "value": 3}));
    let mut3 = Mutation::new("range_schema".to_string(), fields, String::new(), 0, MutationType::Create);
    node.mutate(mut3).unwrap();

    let val = crate::test_data::test_helpers::node_operations::query_value(&mut node, "range_schema", "numbers").unwrap();
    assert_eq!(val, json!({"a": 2, "b": 3}));

}

