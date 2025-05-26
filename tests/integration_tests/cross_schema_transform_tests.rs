use fold_node::testing::{
    Field, FieldVariant, SingleField, Mutation, MutationType, Query, Schema,
    Transform, TransformParser,
};
use fold_node::fees::types::{FieldPaymentConfig, TrustDistanceScaling};
use fold_node::permissions::types::policy::{PermissionsPolicy, TrustDistance};
use fold_node::transform::TransformExecutor;
use crate::test_data::test_helpers::create_test_node_with_schema_permissions;
use serde_json::json;
use std::collections::HashMap;


fn create_schema_a() -> Schema {
    let mut schema = Schema::new("SchemaA".to_string());
    let field = FieldVariant::Single(SingleField::new(
        PermissionsPolicy::new(TrustDistance::Distance(1), TrustDistance::Distance(1)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
    ));
    schema.add_field("a_test_field".to_string(), field);
    schema
}

fn create_schema_b(transform: Transform) -> Schema {
    let mut schema = Schema::new("SchemaB".to_string());
    let mut field = SingleField::new(
        PermissionsPolicy::new(TrustDistance::Distance(1), TrustDistance::Distance(1)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
    );
    field.set_transform(transform);
    schema.add_field("b_test_field".to_string(), FieldVariant::Single(field));
    schema
}

#[test]
fn test_cross_schema_transform_manual_execution() {
    let mut node = create_test_node_with_schema_permissions(&["SchemaA", "SchemaB"]);

    // Load Schema A and set value
    let schema_a = create_schema_a();
    node.add_schema_available(schema_a).unwrap();
    node.approve_schema("SchemaA").unwrap();

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
        filter: None,
    };
    let results = node.query(query_a).unwrap();
    assert_eq!(results[0].as_ref().unwrap(), &json!(4));

    // Create Schema B with transform referencing SchemaA.a_test_field + 5
    let parser = TransformParser::new();
    let expr = parser.parse_expression("SchemaA.a_test_field + 5").unwrap();
    let transform = Transform::new_with_expr(
        "SchemaA.a_test_field + 5".to_string(),
        expr,
        "SchemaB.b_test_field".to_string()
    );
    let schema_b = create_schema_b(transform.clone());
    node.add_schema_available(schema_b).unwrap();
    node.approve_schema("SchemaB").unwrap();

    // Manually execute transform using value from Schema A
    let mut inputs = HashMap::new();
    inputs.insert("SchemaA.a_test_field".to_string(), json!(4));
    let result = TransformExecutor::execute_transform(&transform, inputs).unwrap();
    assert_eq!(result, json!(9.0));
}

#[test]
fn test_cross_schema_transform_with_inputs() {
    let mut node = create_test_node_with_schema_permissions(&["SchemaA", "SchemaB"]);

    // Load Schema A and set value
    let schema_a = create_schema_a();
    node.add_schema_available(schema_a).unwrap();
    node.approve_schema("SchemaA").unwrap();

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
        "SchemaB.b_test_field".to_string()
    );
    transform.set_inputs(vec!["SchemaA.a_test_field".to_string()]);
    let schema_b = create_schema_b(transform);
    node.add_schema_available(schema_b).unwrap();
    node.approve_schema("SchemaB").unwrap();

    // Execute transform through the node
    let result = node.run_transform("SchemaB.b_test_field").unwrap();
    assert_eq!(result, json!(9.0));
}

#[test]
fn test_cross_schema_transform_auto_inputs() {
    let mut node = create_test_node_with_schema_permissions(&["SchemaA", "SchemaB"]);

    // Load Schema A and set value
    let schema_a = create_schema_a();
    node.add_schema_available(schema_a).unwrap();
    node.approve_schema("SchemaA").unwrap();

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
    let transform = Transform::new_with_expr(
        "SchemaA.a_test_field + 5".to_string(),
        expr,
        "SchemaB.b_test_field".to_string(),
    );
    let schema_b = create_schema_b(transform);
    node.add_schema_available(schema_b).unwrap();
    node.approve_schema("SchemaB").unwrap();

    // Execute transform through the node
    let result = node.run_transform("SchemaB.b_test_field").unwrap();
    assert_eq!(result, json!(9.0));
}

#[test]
fn test_transform_persists_to_output_field() {
    let mut node = create_test_node_with_schema_permissions(&["SchemaA", "SchemaB"]);

    // Create Schema B with an output field
    let mut schema_b = Schema::new("SchemaB".to_string());
    let output_field = FieldVariant::Single(SingleField::new(
        PermissionsPolicy::new(TrustDistance::Distance(1), TrustDistance::Distance(1)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
    ));
    schema_b.add_field("b_out".to_string(), output_field);
    node.add_schema_available(schema_b).unwrap();
    node.approve_schema("SchemaB").unwrap();

    // Create Schema A with input field and transform writing to SchemaB.b_out
    let mut schema_a = Schema::new("SchemaA".to_string());
    let input_field = FieldVariant::Single(SingleField::new(
        PermissionsPolicy::new(TrustDistance::Distance(1), TrustDistance::Distance(1)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
    ));
    schema_a.add_field("a_in".to_string(), input_field);

    let parser = TransformParser::new();
    let expr = parser.parse_expression("a_in + 2").unwrap();
    let mut transform = Transform::new_with_expr(
        "a_in + 2".to_string(),
        expr,
        "SchemaB.b_out".to_string(),
    );
    transform.set_inputs(vec!["SchemaA.a_in".to_string()]);
    let mut t_field = SingleField::new(
        PermissionsPolicy::new(TrustDistance::Distance(1), TrustDistance::Distance(1)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
    );
    t_field.set_transform(transform);
    schema_a.add_field("calc".to_string(), FieldVariant::Single(t_field));

    node.add_schema_available(schema_a).unwrap();
    node.approve_schema("SchemaA").unwrap();

    // Set input value
    let mutation = Mutation {
        mutation_type: MutationType::Create,
        schema_name: "SchemaA".to_string(),
        pub_key: "test_key".to_string(),
        trust_distance: 1,
        fields_and_values: vec![("a_in".to_string(), json!(3))]
            .into_iter()
            .collect(),
    };
    node.mutate(mutation).unwrap();

    // Execute the transform which should persist to SchemaB.b_out
    node.run_transform("SchemaA.calc").unwrap();

    let query = Query {
        schema_name: "SchemaB".to_string(),
        pub_key: "test_key".to_string(),
        fields: vec!["b_out".to_string()],
        trust_distance: 1,
        filter: None,
    };
    let results = node.query(query).unwrap();
    assert_eq!(results[0].as_ref().unwrap(), &json!(5.0));
}

