use fold_node::testing::{
    Field, FieldVariant, SingleField, Schema,
};
use fold_node::fees::types::{FieldPaymentConfig, TrustDistanceScaling};
use fold_node::permissions::types::policy::{PermissionsPolicy, TrustDistance};
use fold_node::schema::types::Transform;
use fold_node::transform::parser::TransformParser;
use std::collections::HashMap;

#[allow(dead_code)]
fn create_persistence_schema() -> Schema {
    let mut schema = Schema::new("PersistSchema".to_string());
    // Input field
    let input_field = FieldVariant::Single(SingleField::new(
        PermissionsPolicy::new(TrustDistance::Distance(1), TrustDistance::Distance(1)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
    ));
    schema.add_field("input_field".to_string(), input_field);

    // Transform field that depends on input_field
    let parser = TransformParser::new();
    let expr = parser.parse_expression("input_field + 1").unwrap();
    let transform = Transform::new_with_expr(
        "input_field + 1".to_string(),
        expr,
        "PersistSchema.transform_field".to_string(),
    );
    let mut transform_field = SingleField::new(
        PermissionsPolicy::new(TrustDistance::Distance(1), TrustDistance::Distance(1)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
    );
    transform_field.set_transform(transform);
    schema.add_field("transform_field".to_string(), FieldVariant::Single(transform_field));

    schema
}