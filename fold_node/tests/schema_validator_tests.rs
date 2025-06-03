use fold_node::schema::types::field::FieldType;
use fold_node::schema::types::json_schema::{
    JsonFieldPaymentConfig, JsonPermissionPolicy, JsonSchemaDefinition, JsonSchemaField,
};
use fold_node::schema::types::schema::SchemaType;
use fold_node::testing::{
    CollectionField, Field, FieldPaymentConfig, FieldVariant, PermissionsPolicy, RangeField,
    Schema, SchemaCore, SchemaError, SchemaValidator, SingleField, TrustDistance,
    TrustDistanceScaling,
};
use fold_node::transform::{Transform, TransformParser};
use std::collections::HashMap;
use tempfile::tempdir;
use uuid::Uuid;

fn create_core() -> SchemaCore {
    let dir = tempdir().unwrap();
    SchemaCore::new_for_testing(dir.path().to_str().unwrap()).unwrap()
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

fn base_range_field() -> FieldVariant {
    let field = RangeField::new(
        PermissionsPolicy::new(TrustDistance::Distance(1), TrustDistance::Distance(1)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
    );
    FieldVariant::Range(field)
}

fn base_collection_field() -> FieldVariant {
    let field = CollectionField::new(
        PermissionsPolicy::new(TrustDistance::Distance(1), TrustDistance::Distance(1)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
    );
    FieldVariant::Collection(field)
}

fn create_json_field_def(field_type: FieldType) -> JsonSchemaField {
    JsonSchemaField {
        permission_policy: JsonPermissionPolicy {
            read: TrustDistance::Distance(0),
            write: TrustDistance::Distance(1),
            explicit_read: None,
            explicit_write: None,
        },
        payment_config: JsonFieldPaymentConfig {
            base_multiplier: 1.0,
            trust_distance_scaling: TrustDistanceScaling::None,
            min_payment: None,
        },
        ref_atom_uuid: None,
        field_type,
        field_mappers: HashMap::new(),
        transform: None,
    }
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
    let transform = Transform::new_with_expr("1 + 2".to_string(), expr, "Other.field".to_string());

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

// ===================== RANGE SCHEMA DB-LEVEL VALIDATION TESTS =====================

#[test]
fn validator_accepts_valid_range_schema_all_range_fields() {
    let core = create_core();
    let validator = SchemaValidator::new(&core);

    // Create a valid Range schema with all Range fields
    let mut schema = Schema::new_range("ValidRangeSchema".to_string(), "user_id".to_string());
    schema.add_field("user_id".to_string(), base_range_field());
    schema.add_field("score".to_string(), base_range_field());
    schema.add_field("achievements".to_string(), base_range_field());

    let result = validator.validate(&schema);
    assert!(
        result.is_ok(),
        "Valid Range schema with all Range fields should pass validation: {:?}",
        result
    );
}

#[test]
fn validator_rejects_range_schema_with_single_field() {
    let core = create_core();
    let validator = SchemaValidator::new(&core);

    // Create Range schema with mixed field types (includes Single field)
    let mut schema = Schema::new_range("InvalidRangeSchema".to_string(), "user_id".to_string());
    schema.add_field("user_id".to_string(), base_range_field());
    schema.add_field("score".to_string(), base_range_field());
    schema.add_field("bad_single_field".to_string(), base_field()); // Single field in Range schema

    let result = validator.validate(&schema);
    assert!(
        result.is_err(),
        "Range schema with Single field should be rejected"
    );

    match result.unwrap_err() {
        SchemaError::InvalidField(msg) => {
            assert!(msg.contains("RangeSchema"));
            assert!(msg.contains("Single field"));
            assert!(msg.contains("bad_single_field"));
            assert!(msg.contains("ALL fields must be Range fields"));
        }
        _ => panic!("Expected InvalidField error for Single field in Range schema"),
    }
}

#[test]
fn validator_rejects_range_schema_with_collection_field() {
    let core = create_core();
    let validator = SchemaValidator::new(&core);

    // Create Range schema with Collection field
    let mut schema = Schema::new_range("InvalidRangeSchema".to_string(), "user_id".to_string());
    schema.add_field("user_id".to_string(), base_range_field());
    schema.add_field("score".to_string(), base_range_field());
    schema.add_field("bad_collection_field".to_string(), base_collection_field()); // Collection field in Range schema

    let result = validator.validate(&schema);
    assert!(
        result.is_err(),
        "Range schema with Collection field should be rejected"
    );

    match result.unwrap_err() {
        SchemaError::InvalidField(msg) => {
            assert!(msg.contains("RangeSchema"));
            assert!(msg.contains("Collection field"));
            assert!(msg.contains("bad_collection_field"));
            assert!(msg.contains("ALL fields must be Range fields"));
        }
        _ => panic!("Expected InvalidField error for Collection field in Range schema"),
    }
}

#[test]
fn validator_rejects_range_schema_missing_range_key_field() {
    let core = create_core();
    let validator = SchemaValidator::new(&core);

    // Create Range schema without the range_key field
    let mut schema = Schema::new_range("InvalidRangeSchema".to_string(), "missing_key".to_string());
    schema.add_field("score".to_string(), base_range_field());
    schema.add_field("achievements".to_string(), base_range_field());
    // Missing the "missing_key" field

    let result = validator.validate(&schema);
    assert!(
        result.is_err(),
        "Range schema missing range_key field should be rejected"
    );

    match result.unwrap_err() {
        SchemaError::InvalidField(msg) => {
            assert!(msg.contains("range_key"));
            assert!(msg.contains("missing_key"));
            assert!(msg.contains("must be one of the schema's fields"));
        }
        _ => panic!("Expected InvalidField error for missing range_key field"),
    }
}

#[test]
fn validator_rejects_range_schema_range_key_not_range_field() {
    let core = create_core();
    let validator = SchemaValidator::new(&core);

    // Create Range schema where range_key field is not a Range field
    let mut schema = Schema::new_range("InvalidRangeSchema".to_string(), "user_id".to_string());
    schema.add_field("user_id".to_string(), base_field()); // Single field as range_key
    schema.add_field("score".to_string(), base_range_field());

    let result = validator.validate(&schema);
    assert!(
        result.is_err(),
        "Range schema with non-Range range_key field should be rejected"
    );

    match result.unwrap_err() {
        SchemaError::InvalidField(msg) => {
            assert!(msg.contains("range_key field"));
            assert!(msg.contains("user_id"));
            assert!(msg.contains("Single field"));
            assert!(msg.contains("range_key must be a Range field"));
        }
        _ => panic!("Expected InvalidField error for non-Range range_key field"),
    }
}

#[test]
fn validator_rejects_range_schema_empty_fields() {
    let core = create_core();
    let validator = SchemaValidator::new(&core);

    // Create Range schema with no fields
    let schema = Schema::new_range("EmptyRangeSchema".to_string(), "user_id".to_string());
    // No fields added

    let result = validator.validate(&schema);
    assert!(
        result.is_err(),
        "Range schema with no fields should be rejected"
    );

    match result.unwrap_err() {
        SchemaError::InvalidField(msg) => {
            assert!(msg.contains("range_key"));
            assert!(msg.contains("user_id"));
            assert!(msg.contains("must be one of the schema's fields"));
        }
        _ => panic!("Expected InvalidField error for empty Range schema"),
    }
}

#[test]
fn validator_accepts_regular_schema_with_mixed_fields() {
    let core = create_core();
    let validator = SchemaValidator::new(&core);

    // Create regular (non-Range) schema with mixed field types - should be fine
    let mut schema = Schema::new("MixedSchema".to_string());
    schema.add_field("single_field".to_string(), base_field());
    schema.add_field("range_field".to_string(), base_range_field());
    schema.add_field("collection_field".to_string(), base_collection_field());

    let result = validator.validate(&schema);
    assert!(
        result.is_ok(),
        "Regular schema with mixed field types should pass validation: {:?}",
        result
    );
}

// ===================== JSON RANGE SCHEMA VALIDATION TESTS =====================

#[test]
fn validator_accepts_valid_json_range_schema() {
    let core = create_core();
    let validator = SchemaValidator::new(&core);

    // Create valid JSON Range schema definition
    let mut fields = HashMap::new();
    fields.insert(
        "user_id".to_string(),
        create_json_field_def(FieldType::Range),
    );
    fields.insert("score".to_string(), create_json_field_def(FieldType::Range));
    fields.insert(
        "achievements".to_string(),
        create_json_field_def(FieldType::Range),
    );

    let json_schema = JsonSchemaDefinition {
        name: "ValidJsonRangeSchema".to_string(),
        schema_type: SchemaType::Range {
            range_key: "user_id".to_string(),
        },
        fields,
        payment_config: fold_node::fees::payment_config::SchemaPaymentConfig::default(),
        hash: None,
    };

    let result = validator.validate_json_schema(&json_schema);
    assert!(
        result.is_ok(),
        "Valid JSON Range schema should pass validation: {:?}",
        result
    );
}

#[test]
fn validator_rejects_json_range_schema_with_single_field() {
    let core = create_core();
    let validator = SchemaValidator::new(&core);

    // Create JSON Range schema with Single field
    let mut fields = HashMap::new();
    fields.insert(
        "user_id".to_string(),
        create_json_field_def(FieldType::Range),
    );
    fields.insert("score".to_string(), create_json_field_def(FieldType::Range));
    fields.insert(
        "bad_single".to_string(),
        create_json_field_def(FieldType::Single),
    ); // Single field

    let json_schema = JsonSchemaDefinition {
        name: "InvalidJsonRangeSchema".to_string(),
        schema_type: SchemaType::Range {
            range_key: "user_id".to_string(),
        },
        fields,
        payment_config: fold_node::fees::payment_config::SchemaPaymentConfig::default(),
        hash: None,
    };

    let result = validator.validate_json_schema(&json_schema);
    assert!(
        result.is_err(),
        "JSON Range schema with Single field should be rejected"
    );

    match result.unwrap_err() {
        SchemaError::InvalidField(msg) => {
            assert!(msg.contains("JSON RangeSchema"));
            assert!(msg.contains("Single field"));
            assert!(msg.contains("bad_single"));
            assert!(msg.contains("ALL fields must be Range fields"));
        }
        _ => panic!("Expected InvalidField error for Single field in JSON Range schema"),
    }
}

#[test]
fn validator_rejects_json_range_schema_with_collection_field() {
    let core = create_core();
    let validator = SchemaValidator::new(&core);

    // Create JSON Range schema with Collection field
    let mut fields = HashMap::new();
    fields.insert(
        "user_id".to_string(),
        create_json_field_def(FieldType::Range),
    );
    fields.insert("score".to_string(), create_json_field_def(FieldType::Range));
    fields.insert(
        "bad_collection".to_string(),
        create_json_field_def(FieldType::Collection),
    ); // Collection field

    let json_schema = JsonSchemaDefinition {
        name: "InvalidJsonRangeSchema".to_string(),
        schema_type: SchemaType::Range {
            range_key: "user_id".to_string(),
        },
        fields,
        payment_config: fold_node::fees::payment_config::SchemaPaymentConfig::default(),
        hash: None,
    };

    let result = validator.validate_json_schema(&json_schema);
    assert!(
        result.is_err(),
        "JSON Range schema with Collection field should be rejected"
    );

    match result.unwrap_err() {
        SchemaError::InvalidField(msg) => {
            assert!(msg.contains("JSON RangeSchema"));
            assert!(msg.contains("Collection field"));
            assert!(msg.contains("bad_collection"));
            assert!(msg.contains("ALL fields must be Range fields"));
        }
        _ => panic!("Expected InvalidField error for Collection field in JSON Range schema"),
    }
}

#[test]
fn validator_rejects_json_range_schema_missing_range_key_field() {
    let core = create_core();
    let validator = SchemaValidator::new(&core);

    // Create JSON Range schema without range_key field
    let mut fields = HashMap::new();
    fields.insert("score".to_string(), create_json_field_def(FieldType::Range));
    fields.insert(
        "achievements".to_string(),
        create_json_field_def(FieldType::Range),
    );
    // Missing "user_id" field

    let json_schema = JsonSchemaDefinition {
        name: "InvalidJsonRangeSchema".to_string(),
        schema_type: SchemaType::Range {
            range_key: "user_id".to_string(),
        },
        fields,
        payment_config: fold_node::fees::payment_config::SchemaPaymentConfig::default(),
        hash: None,
    };

    let result = validator.validate_json_schema(&json_schema);
    assert!(
        result.is_err(),
        "JSON Range schema missing range_key field should be rejected"
    );

    match result.unwrap_err() {
        SchemaError::InvalidField(msg) => {
            assert!(msg.contains("JSON RangeSchema"));
            assert!(msg.contains("missing the range_key field"));
            assert!(msg.contains("user_id"));
        }
        _ => panic!("Expected InvalidField error for missing range_key field in JSON Range schema"),
    }
}

#[test]
fn validator_rejects_json_range_schema_range_key_not_range_field() {
    let core = create_core();
    let validator = SchemaValidator::new(&core);

    // Create JSON Range schema where range_key field is not Range type
    let mut fields = HashMap::new();
    fields.insert(
        "user_id".to_string(),
        create_json_field_def(FieldType::Single),
    ); // Single field as range_key
    fields.insert("score".to_string(), create_json_field_def(FieldType::Range));

    let json_schema = JsonSchemaDefinition {
        name: "InvalidJsonRangeSchema".to_string(),
        schema_type: SchemaType::Range {
            range_key: "user_id".to_string(),
        },
        fields,
        payment_config: fold_node::fees::payment_config::SchemaPaymentConfig {
            base_multiplier: 1.0,
            min_payment_threshold: 1,
        },
        hash: None,
    };

    let result = validator.validate_json_schema(&json_schema);
    assert!(
        result.is_err(),
        "JSON Range schema with non-Range range_key field should be rejected"
    );

    match result.unwrap_err() {
        SchemaError::InvalidField(msg) => {
            assert!(msg.contains("JSON RangeSchema"));
            assert!(msg.contains("range_key field"));
            assert!(msg.contains("user_id"));
            assert!(msg.contains("Single field"));
            assert!(msg.contains("must be a Range field"));
        }
        _ => {
            panic!("Expected InvalidField error for non-Range range_key field in JSON Range schema")
        }
    }
}

#[test]
fn validator_rejects_json_range_schema_empty_fields() {
    let core = create_core();
    let validator = SchemaValidator::new(&core);

    // Create JSON Range schema with no fields
    let fields = HashMap::new();

    let json_schema = JsonSchemaDefinition {
        name: "EmptyJsonRangeSchema".to_string(),
        schema_type: SchemaType::Range {
            range_key: "user_id".to_string(),
        },
        fields,
        payment_config: fold_node::fees::payment_config::SchemaPaymentConfig {
            base_multiplier: 1.0,
            min_payment_threshold: 1,
        },
        hash: None,
    };

    let result = validator.validate_json_schema(&json_schema);
    assert!(
        result.is_err(),
        "JSON Range schema with no fields should be rejected"
    );

    match result.unwrap_err() {
        SchemaError::InvalidField(msg) => {
            assert!(msg.contains("JSON RangeSchema"));
            assert!(msg.contains("missing the range_key field"));
            assert!(msg.contains("user_id"));
        }
        _ => panic!("Expected InvalidField error for empty JSON Range schema"),
    }
}

#[test]
fn validator_accepts_json_regular_schema_with_mixed_fields() {
    let core = create_core();
    let validator = SchemaValidator::new(&core);

    // Create regular JSON schema with mixed field types - should be fine
    let mut fields = HashMap::new();
    fields.insert(
        "single_field".to_string(),
        create_json_field_def(FieldType::Single),
    );
    fields.insert(
        "range_field".to_string(),
        create_json_field_def(FieldType::Range),
    );
    fields.insert(
        "collection_field".to_string(),
        create_json_field_def(FieldType::Collection),
    );

    let json_schema = JsonSchemaDefinition {
        name: "MixedJsonSchema".to_string(),
        schema_type: SchemaType::Single,
        fields,
        payment_config: fold_node::fees::payment_config::SchemaPaymentConfig {
            base_multiplier: 1.0,
            min_payment_threshold: 1,
        },
        hash: None,
    };

    let result = validator.validate_json_schema(&json_schema);
    assert!(
        result.is_ok(),
        "Regular JSON schema with mixed field types should pass validation: {:?}",
        result
    );
}

// ===================== ERROR MESSAGE VALIDATION TESTS =====================

#[test]
fn validator_provides_helpful_error_messages_for_range_schema_violations() {
    let core = create_core();
    let validator = SchemaValidator::new(&core);

    // Test detailed error message for mixed field types
    let mut schema = Schema::new_range("UserScores".to_string(), "user_id".to_string());
    schema.add_field("user_id".to_string(), base_range_field());
    schema.add_field("profile_data".to_string(), base_field()); // Single field

    let result = validator.validate(&schema);
    assert!(result.is_err());

    if let Err(SchemaError::InvalidField(msg)) = result {
        // Verify the error message contains helpful guidance
        assert!(msg.contains("RangeSchema 'UserScores'"));
        assert!(msg.contains("Single field 'profile_data'"));
        assert!(msg.contains("ALL fields must be Range fields"));
        assert!(msg.contains("Consider using a regular Schema"));
        assert!(msg.contains("convert 'profile_data' to a Range field"));
    } else {
        panic!("Expected detailed error message for field type violation");
    }
}

#[test]
fn validator_provides_helpful_error_messages_for_json_schema_violations() {
    let core = create_core();
    let validator = SchemaValidator::new(&core);

    // Test detailed error message for JSON schema violations
    let mut fields = HashMap::new();
    fields.insert(
        "user_id".to_string(),
        create_json_field_def(FieldType::Range),
    );
    fields.insert(
        "metadata".to_string(),
        create_json_field_def(FieldType::Collection),
    ); // Collection field

    let json_schema = JsonSchemaDefinition {
        name: "UserProfiles".to_string(),
        schema_type: SchemaType::Range {
            range_key: "user_id".to_string(),
        },
        fields,
        payment_config: fold_node::fees::payment_config::SchemaPaymentConfig {
            base_multiplier: 1.0,
            min_payment_threshold: 1,
        },
        hash: None,
    };

    let result = validator.validate_json_schema(&json_schema);
    assert!(result.is_err());

    if let Err(SchemaError::InvalidField(msg)) = result {
        // Verify the error message contains helpful guidance
        assert!(msg.contains("JSON RangeSchema 'UserProfiles'"));
        assert!(msg.contains("Collection field 'metadata'"));
        assert!(msg.contains("ALL fields must be Range fields"));
        assert!(msg.contains("Consider using a regular Schema"));
        assert!(msg.contains("change 'metadata' to field_type: \"Range\""));
    } else {
        panic!("Expected detailed error message for JSON field type violation");
    }
}

// ===================== BOUNDARY CONDITION TESTS =====================

#[test]
fn validator_handles_range_schema_with_single_range_key_field() {
    let core = create_core();
    let validator = SchemaValidator::new(&core);

    // Create Range schema with only the range_key field
    let mut schema = Schema::new_range("MinimalRangeSchema".to_string(), "id".to_string());
    schema.add_field("id".to_string(), base_range_field());

    let result = validator.validate(&schema);
    assert!(
        result.is_ok(),
        "Range schema with only range_key field should pass validation: {:?}",
        result
    );
}

#[test]
fn validator_handles_complex_range_schema_structure() {
    let core = create_core();
    let validator = SchemaValidator::new(&core);

    // Create complex Range schema with many Range fields
    let mut schema = Schema::new_range("ComplexRangeSchema".to_string(), "entity_id".to_string());

    for i in 0..10 {
        let field_name = if i == 0 {
            "entity_id".to_string()
        } else {
            format!("range_field_{}", i)
        };
        schema.add_field(field_name, base_range_field());
    }

    let result = validator.validate(&schema);
    assert!(
        result.is_ok(),
        "Complex Range schema with many Range fields should pass validation: {:?}",
        result
    );
}

#[test]
fn validator_validates_range_schema_with_special_characters_in_range_key() {
    let core = create_core();
    let validator = SchemaValidator::new(&core);

    // Create Range schema with special characters in range_key
    let mut schema = Schema::new_range(
        "SpecialRangeSchema".to_string(),
        "user:id_with-special.chars".to_string(),
    );
    schema.add_field("user:id_with-special.chars".to_string(), base_range_field());
    schema.add_field("data_field".to_string(), base_range_field());

    let result = validator.validate(&schema);
    assert!(
        result.is_ok(),
        "Range schema with special characters in range_key should pass validation: {:?}",
        result
    );
}
