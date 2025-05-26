use fold_node::testing::{
    Field, FieldVariant, SingleField, Schema,
};
use fold_node::fees::types::{FieldPaymentConfig, TrustDistanceScaling};
use fold_node::permissions::types::policy::{PermissionsPolicy, TrustDistance};
use fold_node::schema::types::Transform;
use fold_node::transform::parser::TransformParser;
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
    let mut field = SingleField::new(
        PermissionsPolicy::new(TrustDistance::Distance(0), TrustDistance::Distance(0)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
    );
    field.set_transform(transform);
    schema.add_field("calc".to_string(), FieldVariant::Single(field));
    schema
}

#[test]
fn transform_output_updated_on_load() {
    let mut node = create_test_node();
    let schema = create_schema_with_transform();
    node.add_schema_available(schema).unwrap();
    node.approve_schema("OutputSchemaTest").unwrap();

    let loaded_schema = node.get_schema("OutputSchemaTest").unwrap().unwrap();
    let field = loaded_schema.fields.get("calc").unwrap();
    let transform = field.transform().unwrap();
    assert_eq!(transform.get_output(), "OutputSchemaTest.calc");
}
