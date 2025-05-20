use fold_node::testing::{
    FieldPaymentConfig, FieldType, PermissionsPolicy, Schema, SchemaField,
    TrustDistance, TrustDistanceScaling,
};
use fold_node::transform::{Transform, TransformParser};
use crate::test_data::test_helpers::create_test_node;
use std::collections::HashMap;

fn create_schema_with_transform() -> Schema {
    let mut schema = Schema::new("OutputSchemaTest".to_string());
    let parser = TransformParser::new();
    let expr = parser.parse_expression("1 + 2").unwrap();
    let transform = Transform::new_with_expr(
        "1 + 2".to_string(),
        expr,
        "test.calc".to_string(),
    );
    let field = SchemaField::new(
        PermissionsPolicy::new(TrustDistance::Distance(0), TrustDistance::Distance(0)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
        Some(FieldType::Single),
    )
    .with_transform(transform);
    schema.add_field("calc".to_string(), field);
    schema
}

#[test]
fn transform_output_updated_on_load() {
    let mut node = create_test_node();
    let schema = create_schema_with_transform();
    node.load_schema(schema).unwrap();

    let loaded_schema = node.get_schema("OutputSchemaTest").unwrap().unwrap();
    let field = loaded_schema.fields.get("calc").unwrap();
    let transform = field.get_transform().unwrap();
    assert_eq!(transform.get_output(), "OutputSchemaTest.calc");
}
