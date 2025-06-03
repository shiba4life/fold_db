use fold_node::fees::{SchemaPaymentConfig, TrustDistanceScaling};
use fold_node::permissions::types::policy::TrustDistance;
use fold_node::schema::types::field::FieldType;
use fold_node::schema::types::json_schema::{
    JsonFieldPaymentConfig, JsonPermissionPolicy, JsonSchemaDefinition, JsonSchemaField,
    JsonTransform,
};
use fold_node::schema::types::{SchemaError, SchemaType};
use std::collections::HashMap;

fn build_schema(transform_logic: &str) -> JsonSchemaDefinition {
    let permission = JsonPermissionPolicy {
        read: TrustDistance::Distance(0),
        write: TrustDistance::Distance(0),
        explicit_read: None,
        explicit_write: None,
    };

    let field = JsonSchemaField {
        permission_policy: permission,
        ref_atom_uuid: Some("uuid".to_string()),
        payment_config: JsonFieldPaymentConfig {
            base_multiplier: 1.0,
            trust_distance_scaling: TrustDistanceScaling::None,
            min_payment: None,
        },
        field_mappers: HashMap::new(),
        field_type: FieldType::Single,
        transform: Some(JsonTransform {
            logic: transform_logic.to_string(),
            inputs: Vec::new(),
            output: "test.calc".to_string(),
        }),
    };

    let mut fields = HashMap::new();
    fields.insert("calc".to_string(), field);

    JsonSchemaDefinition {
        name: "test".to_string(),
        schema_type: SchemaType::Single,
        fields,
        payment_config: SchemaPaymentConfig::default(),
        hash: None,
    }
}

#[test]
fn validate_field_with_valid_transform() {
    let schema = build_schema("1 + 1");
    assert!(schema.validate().is_ok());
}

#[test]
fn validate_field_with_invalid_transform() {
    let schema = build_schema("1 +");
    let result = schema.validate();
    assert!(matches!(result, Err(SchemaError::InvalidField(_))));
}
