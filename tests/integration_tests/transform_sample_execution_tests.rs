use crate::test_data::test_helpers::create_test_node;
use fold_node::testing::{Mutation, MutationType};
use fold_node::datafold_node::loader::load_schema_from_file;
use serde_json::json;

#[test]
fn test_sample_transform_execution() {
    let mut node = create_test_node();

    // Load sample schemas
    load_schema_from_file(
        "fold_node/src/datafold_node/samples/data/TransformBase.json",
        &mut node,
    )
    .unwrap();
    load_schema_from_file(
        "fold_node/src/datafold_node/samples/data/TransformSchema.json",
        &mut node,
    )
    .unwrap();

    node.allow_schema("TransformBase").unwrap();
    node.allow_schema("TransformSchema").unwrap();

    // Populate inputs
    for (field, val) in [("value1", json!(2)), ("value2", json!(3))] {
        let mut fields = std::collections::HashMap::new();
        fields.insert(field.to_string(), val);
        let mutation = Mutation {
            mutation_type: MutationType::Create,
            schema_name: "TransformBase".to_string(),
            pub_key: "test_key".to_string(),
            trust_distance: 0,
            fields_and_values: fields,
        };
        node.mutate(mutation).unwrap();
    }

    // Execute transform and verify result
    let result = node.run_transform("TransformSchema.result").unwrap();
    assert_eq!(result, json!(5.0));
}

