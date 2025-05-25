use fold_node::testing::{SchemaCore, Schema, FieldVariant, SingleField, Field, FieldPaymentConfig, SchemaPaymentConfig, TrustDistanceScaling, PermissionsPolicy, TrustDistance, SchemaError, FieldType};
use fold_node::schema::types::{JsonSchemaDefinition, JsonSchemaField};
use fold_node::schema::types::json_schema::{JsonFieldPaymentConfig, JsonPermissionPolicy};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;

    fn cleanup_test_schema(name: &str) {
        let path = PathBuf::from("data/schemas").join(format!("{}.json", name));
        let _ = fs::remove_file(path);
    }

    fn create_test_field(
        ref_atom_uuid: Option<String>,
        field_mappers: HashMap<String, String>,
    ) -> FieldVariant {
        let mut single_field = SingleField::new(
            PermissionsPolicy::default(),
            FieldPaymentConfig::default(),
            field_mappers,
        );
        if let Some(uuid) = ref_atom_uuid {
            single_field.set_ref_atom_uuid(uuid);
        }
        FieldVariant::Single(single_field)
    }

    fn build_json_schema(name: &str) -> JsonSchemaDefinition {
        let permission_policy = JsonPermissionPolicy {
            read: TrustDistance::Distance(0),
            write: TrustDistance::Distance(0),
            explicit_read: None,
            explicit_write: None,
        };
        let field = JsonSchemaField {
            permission_policy,
            ref_atom_uuid: Some("uuid".to_string()),
            payment_config: JsonFieldPaymentConfig {
                base_multiplier: 1.0,
                trust_distance_scaling: TrustDistanceScaling::None,
                min_payment: None,
            },
            field_mappers: HashMap::new(),
            field_type: FieldType::Single,
            transform: None,
        };
        let mut fields = HashMap::new();
        fields.insert("field".to_string(), field);
        JsonSchemaDefinition {
            name: name.to_string(),
            fields,
            payment_config: SchemaPaymentConfig::default(),
        }
    }

    #[test]
    fn test_schema_persistence() {
        let test_schema_name = "test_persistence_schema";
        cleanup_test_schema(test_schema_name); // Cleanup any leftover test files

        let core = SchemaCore::new("data").unwrap();

        // Create a test schema
        let mut fields = HashMap::new();
        fields.insert(
            "test_field".to_string(),
            create_test_field(Some("test_uuid".to_string()), HashMap::new()),
        );
        let schema = Schema::new(test_schema_name.to_string()).with_fields(fields);

        // Load and persist schema
        core.load_schema(schema.clone()).unwrap();

        // Verify file exists
        let schema_path = PathBuf::from("data/schemas").join(format!("{}.json", test_schema_name));

        // Read and verify content
        let content = fs::read_to_string(&schema_path).unwrap();
        let loaded_schema: Schema = serde_json::from_str(&content).unwrap();
        assert_eq!(loaded_schema.name, test_schema_name);
        assert_eq!(
            loaded_schema
                .fields
                .get("test_field")
                .unwrap()
                .ref_atom_uuid(),
            Some(&"test_uuid".to_string())
        );

        // Unload schema should keep file on disk
        core.unload_schema(test_schema_name).unwrap();
        assert!(schema_path.exists());

        cleanup_test_schema(test_schema_name);
    }

    #[test]
    fn test_map_fields_success() {
        let core = SchemaCore::new("data").unwrap();

        // Create source schema with a field that has a ref_atom_uuid
        let mut source_fields = HashMap::new();
        source_fields.insert(
            "source_field".to_string(),
            create_test_field(Some("test_uuid".to_string()), HashMap::new()),
        );
        let source_schema = Schema::new("source_schema".to_string()).with_fields(source_fields);
        core.load_schema(source_schema).unwrap();

        // Create target schema with a field that maps to the source field
        let mut field_mappers = HashMap::new();
        field_mappers.insert("source_schema".to_string(), "source_field".to_string());
        let mut target_fields = HashMap::new();
        target_fields.insert(
            "target_field".to_string(),
            create_test_field(None, field_mappers),
        );
        let target_schema = Schema::new("target_schema".to_string()).with_fields(target_fields);
        core.load_schema(target_schema).unwrap();

        // Map fields
        core.map_fields("target_schema").unwrap();

        // Verify the mapping
        let mapped_schema = core.get_schema("target_schema").unwrap().unwrap();
        let mapped_field = mapped_schema.fields.get("target_field").unwrap();
        assert_eq!(
            mapped_field.ref_atom_uuid(),
            Some(&"test_uuid".to_string())
        );
    }

    #[test]
    fn test_validate_schema_valid() {
        let core = SchemaCore::new("data").unwrap();
        let schema = build_json_schema("valid");
        assert!(core.interpret_schema(schema).is_ok());
    }

    #[test]
    fn test_validate_schema_empty_name() {
        let core = SchemaCore::new("data").unwrap();
        let schema = build_json_schema("");
        let result = core.interpret_schema(schema);
        assert!(matches!(result, Err(SchemaError::InvalidField(msg)) if msg == "Schema name cannot be empty"));
    }

    #[test]
    fn test_validate_schema_empty_field_name() {
        let core = SchemaCore::new("data").unwrap();
        let mut schema = build_json_schema("valid");
        let field = schema.fields.remove("field").unwrap();
        schema.fields.insert("".to_string(), field);
        let result = core.interpret_schema(schema);
        assert!(matches!(result, Err(SchemaError::InvalidField(msg)) if msg == "Field name cannot be empty"));
    }

    #[test]
    fn test_validate_schema_invalid_mapper() {
        let core = SchemaCore::new("data").unwrap();
        let mut schema = build_json_schema("valid");
        if let Some(field) = schema.fields.get_mut("field") {
            field.field_mappers.insert(String::new(), "v".to_string());
        }
        let result = core.interpret_schema(schema);
        assert!(matches!(result, Err(SchemaError::InvalidField(msg)) if msg.contains("invalid field mapper")));
    }

    #[test]
    fn test_validate_schema_min_payment_zero() {
        let core = SchemaCore::new("data").unwrap();
        let mut schema = build_json_schema("valid");
        if let Some(field) = schema.fields.get_mut("field") {
            field.payment_config.min_payment = Some(0);
        }
        let result = core.interpret_schema(schema);
        assert!(matches!(result, Err(SchemaError::InvalidField(msg)) if msg.contains("min_payment cannot be zero")));
    }
