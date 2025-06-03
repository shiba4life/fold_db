use crate::test_data::test_helpers::create_test_node;
use fold_node::fees::types::{FieldPaymentConfig, TrustDistanceScaling};
use fold_node::permissions::types::policy::{PermissionsPolicy, TrustDistance};
use fold_node::schema::types::Transform;
use fold_node::testing::{Field, FieldVariant, Schema, SingleField};
use fold_node::transform::parser::TransformParser;

#[test]
fn set_unloaded_keeps_transforms() {
    let mut node = create_test_node();

    let mut schema = Schema::new("UnloadSchema".to_string());
    let parser = TransformParser::new();
    let expr = parser.parse_expression("1 + 2").unwrap();
    let transform =
        Transform::new_with_expr("1 + 2".to_string(), expr, "UnloadSchema.calc".to_string());
    let mut field = SingleField::new(
        PermissionsPolicy::new(TrustDistance::Distance(0), TrustDistance::Distance(0)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        std::collections::HashMap::new(),
    );
    field.set_transform(transform);
    schema.add_field("calc".to_string(), FieldVariant::Single(field));

    node.add_schema_available(schema).unwrap();
    node.approve_schema("UnloadSchema").unwrap();

    assert!(node
        .list_transforms()
        .unwrap()
        .contains_key("UnloadSchema.calc"));

    node.unload_schema("UnloadSchema").unwrap();

    assert!(node
        .list_transforms()
        .unwrap()
        .contains_key("UnloadSchema.calc"));
    assert!(node.get_schema("UnloadSchema").unwrap().is_none());
}
