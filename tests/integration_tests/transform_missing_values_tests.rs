use fold_node::testing::{
    FieldPaymentConfig, FieldType, PermissionsPolicy, Schema, SchemaField,
    TrustDistance, TrustDistanceScaling,
};
use fold_node::transform::{Transform, TransformParser};
use crate::test_data::test_helpers::create_test_node;

#[test]
fn transform_without_values_no_missing_aref() {
    let mut node = create_test_node();

    // Build schema
    let mut schema = Schema::new("EmptyTransform".to_string());
    let input_field = SchemaField::new(
        PermissionsPolicy::new(TrustDistance::Distance(0), TrustDistance::Distance(0)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        std::collections::HashMap::new(),
        Some(FieldType::Single),
    );
    schema.add_field("input".to_string(), input_field);

    let parser = TransformParser::new();
    let expr = parser.parse_expression("input + 1").unwrap();
    let mut transform = Transform::new_with_expr(
        "input + 1".to_string(),
        expr,
        "EmptyTransform.output".to_string(),
    );
    transform.set_inputs(vec!["EmptyTransform.input".to_string()]);
    let t_field = SchemaField::new(
        PermissionsPolicy::new(TrustDistance::Distance(0), TrustDistance::Distance(0)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        std::collections::HashMap::new(),
        Some(FieldType::Single),
    )
    .with_transform(transform);
    schema.add_field("output".to_string(), t_field);

    node.load_schema(schema).unwrap();

    let err = node.run_transform("EmptyTransform.output").unwrap_err();
    assert!(
        !err.to_string().contains("AtomRef not found"),
        "Error should not mention missing AtomRef, got: {}",
        err
    );
}
