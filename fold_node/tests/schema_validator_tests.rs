use fold_node::testing::{
    Field, FieldPaymentConfig, PermissionsPolicy, Schema, SingleField, FieldVariant,
    SchemaCore, SchemaValidator, SchemaError, TrustDistance, TrustDistanceScaling,
};
use fold_node::transform::{Transform, TransformParser};
use tempfile::tempdir;
use std::collections::HashMap;
use uuid::Uuid;

fn create_core() -> SchemaCore {
    let dir = tempdir().unwrap();
    SchemaCore::new(dir.path().to_str().unwrap()).unwrap()
}

fn base_field() -> FieldVariant {
    let mut field = SingleField::new(
        PermissionsPolicy::new(TrustDistance::Distance(1), TrustDistance::Distance(1)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
    );
    field.set_ref_atom_uuid(Uuid::new_v4().to_string());
    FieldVariant::Single(field)
}

#[test]
fn validator_accepts_valid_transform() {
    let core = create_core();
    let mut source = Schema::new("Src".to_string());
    source.add_field("value".to_string(), base_field());
    core.add_schema_available(source).unwrap();
    core.approve_schema("Src").unwrap();

    let parser = TransformParser::new();
    let expr = parser.parse_expression("Src.value + 1").unwrap();
    let mut transform = Transform::new_with_expr(
        "Src.value + 1".to_string(),
        expr,
        "Target.result".to_string(),
    );
    transform.set_inputs(vec!["Src.value".to_string()]);

    let mut field = SingleField::new(
        PermissionsPolicy::new(TrustDistance::Distance(1), TrustDistance::Distance(1)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
    );
    field.set_transform(transform);
    let field = FieldVariant::Single(field);

    let mut target = Schema::new("Target".to_string());
    target.add_field("result".to_string(), field);

    let validator = SchemaValidator::new(&core);
    assert!(validator.validate(&target).is_ok());
}

#[test]
fn validator_rejects_self_input() {
    let core = create_core();
    let parser = TransformParser::new();
    let expr = parser.parse_expression("Self.result + 1").unwrap();
    let mut transform = Transform::new_with_expr(
        "Self.result + 1".to_string(),
        expr,
        "Self.result".to_string(),
    );
    transform.set_inputs(vec!["Self.result".to_string()]);

    let mut field = SingleField::new(
        PermissionsPolicy::default(),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
    );
    field.set_transform(transform);
    let field = FieldVariant::Single(field);

    let mut schema = Schema::new("Self".to_string());
    schema.add_field("result".to_string(), field);

    let validator = SchemaValidator::new(&core);
    let res = validator.validate(&schema);
    assert!(matches!(res, Err(SchemaError::InvalidTransform(_))));
}

#[test]
fn validator_rejects_wrong_output() {
    let core = create_core();
    let parser = TransformParser::new();
    let expr = parser.parse_expression("1 + 2").unwrap();
    let transform = Transform::new_with_expr(
        "1 + 2".to_string(),
        expr,
        "Other.field".to_string(),
    );

    let mut field = SingleField::new(
        PermissionsPolicy::default(),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
    );
    field.set_transform(transform);
    let field = FieldVariant::Single(field);

    let mut schema = Schema::new("Test".to_string());
    schema.add_field("calc".to_string(), field);

    let validator = SchemaValidator::new(&core);
    let res = validator.validate(&schema);
    assert!(matches!(res, Err(SchemaError::InvalidTransform(_))));
}

