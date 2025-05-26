use crate::test_data::test_helpers::{setup_test_db, cleanup_test_db};
use fold_node::testing::{
    Field, FieldVariant, SingleField, Schema, Mutation, MutationType,
};
use fold_node::fees::types::{FieldPaymentConfig, TrustDistanceScaling};
use fold_node::permissions::types::policy::{PermissionsPolicy, TrustDistance};
use fold_node::schema::types::Transform;
use fold_node::transform::parser::TransformParser;
use serde_json::json;

#[test]
fn mutation_enqueues_transform() {
    let (mut db, path) = setup_test_db();

    // Build schema with a field that has a simple transform
    let mut schema = Schema::new("EnqueueSchema".to_string());
    let parser = TransformParser::new();
    let expr = parser.parse_expression("1 + 1").unwrap();
    let transform = Transform::new_with_expr(
        "1 + 1".to_string(),
        expr,
        "EnqueueSchema.calc".to_string()
    );
    let mut field = SingleField::new(
        PermissionsPolicy::new(TrustDistance::Distance(0), TrustDistance::Distance(0)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        std::collections::HashMap::new(),
    );
    field.set_transform(transform);
    schema.add_field("calc".to_string(), FieldVariant::Single(field));

    db.add_schema_available(schema).unwrap();
    db.approve_schema("EnqueueSchema").unwrap();

    let mutation = Mutation {
        mutation_type: MutationType::Create,
        schema_name: "EnqueueSchema".to_string(),
        pub_key: "test_key".to_string(),
        trust_distance: 0,
        fields_and_values: vec![("calc".to_string(), json!(123))]
            .into_iter()
            .collect(),
    };

    db.write_schema(mutation).unwrap();

    // Verify that a transform task was queued
    assert_eq!(db.orchestrator_len().unwrap(), 1);

    cleanup_test_db(&path);
}
