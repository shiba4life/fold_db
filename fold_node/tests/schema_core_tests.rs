use fold_node::testing::{
    Field, FieldPaymentConfig, PermissionsPolicy, Schema, SingleField, FieldVariant,
    SchemaCore, SchemaValidator, TrustDistance, TrustDistanceScaling,
};
use fold_node::schema::types::{JsonSchemaDefinition, JsonSchemaField};
use fold_node::schema::types::field::{FieldType};
use fold_node::schema::types::json_schema::{JsonFieldPaymentConfig, JsonPermissionPolicy};
use fold_node::fees::SchemaPaymentConfig;
use tempfile::tempdir;
use std::collections::HashMap;
use std::fs;

fn cleanup_test_schema(name: &str) {
    let path = std::path::PathBuf::from("data/schemas").join(format!("{}.json", name));
    let _ = fs::remove_file(path);
}

fn create_test_field(ref_atom_uuid: Option<String>, field_mappers: HashMap<String, String>) -> FieldVariant {
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
    cleanup_test_schema(test_schema_name);

    let temp_dir = tempdir().unwrap();
    let core = SchemaCore::new(temp_dir.path().to_str().unwrap()).unwrap();

    let mut fields = HashMap::new();
    fields.insert(
        "test_field".to_string(),
        create_test_field(Some("test_uuid".to_string()), HashMap::new()),
    );
    let schema = Schema::new(test_schema_name.to_string()).with_fields(fields);

    core.load_schema(schema.clone()).unwrap();
    let schema_path = core.schema_path(test_schema_name);
    assert!(schema_path.exists());

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

    core.unload_schema(test_schema_name).unwrap();
    assert!(schema_path.exists());

    cleanup_test_schema(test_schema_name);
}

#[test]
fn test_map_fields_success() {
    let temp_dir = tempdir().unwrap();
    let core = SchemaCore::new(temp_dir.path().to_str().unwrap()).unwrap();

    let mut source_fields = HashMap::new();
    source_fields.insert(
        "source_field".to_string(),
        create_test_field(Some("test_uuid".to_string()), HashMap::new()),
    );
    let source_schema = Schema::new("source_schema".to_string()).with_fields(source_fields);
    core.load_schema(source_schema).unwrap();

    let mut field_mappers = HashMap::new();
    field_mappers.insert("source_schema".to_string(), "source_field".to_string());
    let mut target_fields = HashMap::new();
    target_fields.insert(
        "target_field".to_string(),
        create_test_field(None, field_mappers),
    );
    let target_schema = Schema::new("target_schema".to_string()).with_fields(target_fields);
    core.load_schema(target_schema).unwrap();

    core.map_fields("target_schema").unwrap();
    let mapped_schema = core.get_schema("target_schema").unwrap().unwrap();
    let mapped_field = mapped_schema.fields.get("target_field").unwrap();
    assert_eq!(
        mapped_field.ref_atom_uuid(),
        Some(&"test_uuid".to_string())
    );
}

#[test]
fn test_validate_schema_valid() {
    let temp_dir = tempdir().unwrap();
    let core = SchemaCore::new(temp_dir.path().to_str().unwrap()).unwrap();
    let schema = build_json_schema("valid");
    let validator = SchemaValidator::new(&core);
    assert!(validator.validate_json_schema(&schema).is_ok());
}

#[test]
fn test_validate_schema_empty_name() {
    let temp_dir = tempdir().unwrap();
    let core = SchemaCore::new(temp_dir.path().to_str().unwrap()).unwrap();
    let schema = build_json_schema("");
    let validator = SchemaValidator::new(&core);
    let result = validator.validate_json_schema(&schema);
    assert!(matches!(result, Err(fold_node::schema::types::errors::SchemaError::InvalidField(msg)) if msg == "Schema name cannot be empty"));
}

#[test]
fn test_validate_schema_empty_field_name() {
    let temp_dir = tempdir().unwrap();
    let core = SchemaCore::new(temp_dir.path().to_str().unwrap()).unwrap();
    let mut schema = build_json_schema("valid");
    let field = schema.fields.remove("field").unwrap();
    schema.fields.insert("".to_string(), field);
    let validator = SchemaValidator::new(&core);
    let result = validator.validate_json_schema(&schema);
    assert!(matches!(result, Err(fold_node::schema::types::errors::SchemaError::InvalidField(msg)) if msg == "Field name cannot be empty"));
}

#[test]
fn test_validate_schema_invalid_mapper() {
    let temp_dir = tempdir().unwrap();
    let core = SchemaCore::new(temp_dir.path().to_str().unwrap()).unwrap();
    let mut schema = build_json_schema("valid");
    if let Some(field) = schema.fields.get_mut("field") {
        field.field_mappers.insert(String::new(), "v".to_string());
    }
    let validator = SchemaValidator::new(&core);
    let result = validator.validate_json_schema(&schema);
    assert!(matches!(result, Err(fold_node::schema::types::errors::SchemaError::InvalidField(msg)) if msg.contains("invalid field mapper")));
}

#[test]
fn test_validate_schema_min_payment_zero() {
    let temp_dir = tempdir().unwrap();
    let core = SchemaCore::new(temp_dir.path().to_str().unwrap()).unwrap();
    let mut schema = build_json_schema("valid");
    if let Some(field) = schema.fields.get_mut("field") {
        field.payment_config.min_payment = Some(0);
    }
    let validator = SchemaValidator::new(&core);
    let result = validator.validate_json_schema(&schema);
    assert!(matches!(result, Err(fold_node::schema::types::errors::SchemaError::InvalidField(msg)) if msg.contains("min_payment cannot be zero")));
}

#[test]
fn test_load_schemas_from_disk_persists_state() {
    let dir = tempdir().unwrap();
    let schema_dir = dir.path().join("schemas");
    std::fs::create_dir_all(&schema_dir).unwrap();
    let schema_path = schema_dir.join("disk.json");

    let schema = Schema::new("disk".to_string());
    std::fs::write(&schema_path, serde_json::to_string_pretty(&schema).unwrap()).unwrap();

    let core = SchemaCore::new(dir.path().to_str().unwrap()).unwrap();
    core.load_schemas_from_disk().unwrap();
    drop(core);

    let db = sled::open(dir.path()).unwrap();
    let tree = db.open_tree("schema_states").unwrap();
    assert!(tree.get("disk").unwrap().is_some());
}

#[test]
fn test_new_schema_default_unloaded() {
    let dir = tempdir().unwrap();
    let schema_dir = dir.path().join("schemas");
    std::fs::create_dir_all(&schema_dir).unwrap();
    let schema_path = schema_dir.join("fresh.json");

    let schema = Schema::new("fresh".to_string());
    std::fs::write(&schema_path, serde_json::to_string_pretty(&schema).unwrap()).unwrap();

    let core = SchemaCore::new(dir.path().to_str().unwrap()).unwrap();
    core.load_schemas_from_disk().unwrap();
    assert!(!core.list_loaded_schemas().unwrap().contains(&"fresh".to_string()));
    drop(core);
    let db = sled::open(dir.path()).unwrap();
    let tree = db.open_tree("schema_states").unwrap();
    let value = tree.get("fresh").unwrap().unwrap();
    let state: String = serde_json::from_slice(&value).unwrap();
    assert_eq!(state, "Unloaded");
}

