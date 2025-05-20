use fold_node::testing::{
    FieldPaymentConfig, FieldType, Mutation, MutationType, PermissionsPolicy, Query, Schema, SchemaField, TrustDistance, TrustDistanceScaling,
};
use fold_node::transform::{Transform, TransformExecutor, TransformParser};
use crate::test_data::test_helpers::create_test_node;
use serde_json::json;
use std::collections::HashMap;


fn create_schema_a() -> Schema {
    let mut schema = Schema::new("SchemaA".to_string());
    let field = SchemaField::new(
        PermissionsPolicy::new(TrustDistance::Distance(1), TrustDistance::Distance(1)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
        Some(FieldType::Single),
    );
    schema.add_field("a_test_field".to_string(), field);
    schema
}

fn create_schema_b(transform: Transform) -> Schema {
    let mut schema = Schema::new("SchemaB".to_string());
    let field = SchemaField::new(
        PermissionsPolicy::new(TrustDistance::Distance(1), TrustDistance::Distance(1)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
        Some(FieldType::Single),
    )
    .with_transform(transform);
    schema.add_field("b_test_field".to_string(), field);
    schema
}

#[test]
fn test_cross_schema_transform_manual_execution() {
    let mut node = create_test_node();

    // Load Schema A and set value
    let schema_a = create_schema_a();
    node.load_schema(schema_a).unwrap();
    node.allow_schema("SchemaA").unwrap();

    let mutation = Mutation {
        mutation_type: MutationType::Create,
        schema_name: "SchemaA".to_string(),
        pub_key: "test_key".to_string(),
        trust_distance: 1,
        fields_and_values: vec![("a_test_field".to_string(), json!(4))]
            .into_iter()
            .collect(),
    };
    node.mutate(mutation).unwrap();

    // Verify value in Schema A
    let query_a = Query {
        schema_name: "SchemaA".to_string(),
        pub_key: "test_key".to_string(),
        fields: vec!["a_test_field".to_string()],
        trust_distance: 1,
    };
    let results = node.query(query_a).unwrap();
    assert_eq!(results[0].as_ref().unwrap(), &json!(4));

    // Create Schema B with transform referencing SchemaA.a_test_field + 5
    let parser = TransformParser::new();
    let expr = parser.parse_expression("SchemaA.a_test_field + 5").unwrap();
    let transform = Transform::new_with_expr(
        "SchemaA.a_test_field + 5".to_string(),
        expr,
        false,
        None,
        false,
        "SchemaB.b_test_field".to_string()
    );
    let schema_b = create_schema_b(transform.clone());
    node.load_schema(schema_b).unwrap();
    node.allow_schema("SchemaB").unwrap();

    // Manually execute transform using value from Schema A
    let mut inputs = HashMap::new();
    inputs.insert("SchemaA.a_test_field".to_string(), json!(4));
    let result = TransformExecutor::execute_transform(&transform, inputs).unwrap();
    assert_eq!(result, json!(9.0));
}

#[test]
fn test_cross_schema_transform_with_inputs() {
    let mut node = create_test_node();

    // Load Schema A and set value
    let schema_a = create_schema_a();
    node.load_schema(schema_a).unwrap();

    let mutation = Mutation {
        mutation_type: MutationType::Create,
        schema_name: "SchemaA".to_string(),
        pub_key: "test_key".to_string(),
        trust_distance: 1,
        fields_and_values: vec![("a_test_field".to_string(), json!(4))]
            .into_iter()
            .collect(),
    };
    node.mutate(mutation).unwrap();

    // Create Schema B with transform referencing SchemaA.a_test_field + 5
    let parser = TransformParser::new();
    let expr = parser.parse_expression("SchemaA.a_test_field + 5").unwrap();
    let mut transform = Transform::new_with_expr(
        "SchemaA.a_test_field + 5".to_string(),
        expr,
        false,
        None,
        false,
        "SchemaB.b_test_field".to_string()
    );
    transform.set_inputs(vec!["SchemaA.a_test_field".to_string()]);
    let schema_b = create_schema_b(transform);
    node.load_schema(schema_b).unwrap();

    // Execute transform through the node
    let result = node.run_transform("SchemaB.b_test_field").unwrap();
    assert_eq!(result, json!(9.0));
}

