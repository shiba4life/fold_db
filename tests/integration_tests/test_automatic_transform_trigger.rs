use crate::test_data::test_helpers::{create_test_node_with_schema_permissions, load_and_approve_schema};
use fold_node::testing::{Mutation, MutationType};
use fold_node::schema::types::{Query};
use serde_json::json;

#[test]
fn test_automatic_transform_trigger() {
    let mut node = create_test_node_with_schema_permissions(&["TransformBase", "TransformSchema"]);

    // Load and approve sample schemas
    load_and_approve_schema(&mut node, "fold_node/src/datafold_node/samples/data/TransformBase.json", "TransformBase").unwrap();
    load_and_approve_schema(&mut node, "fold_node/src/datafold_node/samples/data/TransformSchema.json", "TransformSchema").unwrap();

    // Mutate TransformBase.value1 - this should automatically trigger and execute TransformSchema.result transform
    let mut fields = std::collections::HashMap::new();
    fields.insert("value1".to_string(), json!(2));
    let mutation = Mutation {
        mutation_type: MutationType::Create,
        schema_name: "TransformBase".to_string(),
        pub_key: "test_key".to_string(),
        trust_distance: 0,
        fields_and_values: fields,
    };
    node.mutate(mutation).unwrap();

    // Mutate TransformBase.value2 - this should also trigger and execute TransformSchema.result transform
    let mut fields = std::collections::HashMap::new();
    fields.insert("value2".to_string(), json!(3));
    let mutation = Mutation {
        mutation_type: MutationType::Create,
        schema_name: "TransformBase".to_string(),
        pub_key: "test_key".to_string(),
        trust_distance: 0,
        fields_and_values: fields,
    };
    node.mutate(mutation).unwrap();

    // Process the transform queue to execute any queued transforms
    node.process_transform_queue().unwrap();

    // Query the result field to verify the transform was executed automatically
    let query = Query::new(
        "TransformSchema".to_string(),
        vec!["result".to_string()],
        "test_key".to_string(),
        0,
    );
    let query_results = node.query(query).unwrap();
    println!("Query results: {:?}", query_results);
    
    // The transform should have been executed automatically, resulting in 2 + 3 = 5
    assert!(!query_results.is_empty(), "Query should return results");
    if let Ok(result_value) = &query_results[0] {
        assert_eq!(*result_value, json!(5.0), "Transform should have calculated 2 + 3 = 5 automatically");
    } else {
        panic!("Query result should be Ok, got: {:?}", query_results[0]);
    }
}