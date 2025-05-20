use fold_node::testing::{
    PermissionsPolicy, FieldPaymentConfig, FieldType, Schema, SchemaField,
    TrustDistance, TrustDistanceScaling,
};
use fold_node::transform::{Transform, TransformParser};
use crate::test_data::test_helpers::create_test_node;

#[test]
fn unload_schema_removes_transforms() {
    let mut node = create_test_node();

    // Build schema with a simple transform
    let mut schema = Schema::new("UnloadSchema".to_string());
    let parser = TransformParser::new();
    let expr = parser.parse_expression("1 + 2").unwrap();
    let transform = Transform::new_with_expr(
        "1 + 2".to_string(),
        expr,
        "UnloadSchema.calc".to_string(),
    );
    let field = SchemaField::new(
        PermissionsPolicy::new(TrustDistance::Distance(0), TrustDistance::Distance(0)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        std::collections::HashMap::new(),
        Some(FieldType::Single),
    )
    .with_transform(transform);
    schema.add_field("calc".to_string(), field);

    node.load_schema(schema).unwrap();
    node.allow_schema("UnloadSchema").unwrap();

    // Ensure transform exists
    let transforms = node.list_transforms().unwrap();
    assert!(transforms.contains_key("UnloadSchema.calc"));

    node.remove_schema("UnloadSchema").unwrap();

    // Transform should be gone
    let transforms = node.list_transforms().unwrap();
    assert!(!transforms.contains_key("UnloadSchema.calc"));
    assert!(node.get_schema("UnloadSchema").unwrap().is_none());
}
