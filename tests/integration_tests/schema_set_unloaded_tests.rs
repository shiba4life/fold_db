use fold_node::testing::{
    PermissionsPolicy, FieldPaymentConfig, FieldType, Schema, SchemaField,
    TrustDistance, TrustDistanceScaling,
};
use fold_node::transform::{Transform, TransformParser};
use crate::test_data::test_helpers::create_test_node;

#[test]
fn set_unloaded_keeps_transforms() {
    let mut node = create_test_node();

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

    assert!(node.list_transforms().unwrap().contains_key("UnloadSchema.calc"));

    node.set_schema_unloaded("UnloadSchema").unwrap();

    assert!(node.list_transforms().unwrap().contains_key("UnloadSchema.calc"));
    assert!(node.get_schema("UnloadSchema").unwrap().is_none());
}
