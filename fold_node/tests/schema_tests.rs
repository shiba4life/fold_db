use fold_node::testing::{Schema, SchemaCore};
use tempfile::tempdir;
mod test_data;
use test_data::schema_test_data::*;

#[test]
fn test_schema_creation() {
    let schema_name = "test_schema".to_string();
    let schema = Schema::new(schema_name.clone());

    assert_eq!(schema.name, schema_name);
    assert!(schema.fields.is_empty());
}

#[test]
fn test_schema_field_management() {
    let schema = create_test_schema("test_schema");
    let field_name = "test_field";

    // Verify field was added
    assert!(schema.fields.contains_key(field_name));
    let stored_field = schema.fields.get(field_name).unwrap();
    assert_eq!(stored_field.get_ref_atom_uuid(), Some("test-uuid".to_string()));
    assert!(stored_field.field_mappers.is_empty());
}

#[test]
fn test_schema_field_permissions() {
    let schema = create_multi_field_schema();

    // Verify permissions for each field
    let public_field = schema.fields.get("public_field").unwrap();
    match public_field.permission_policy.read_policy {
        fold_node::testing::TrustDistance::Distance(d) => assert_eq!(d, 0),
        _ => panic!("Expected Distance variant"),
    }

    let protected_field = schema.fields.get("protected_field").unwrap();
    match protected_field.permission_policy.read_policy {
        fold_node::testing::TrustDistance::Distance(d) => assert_eq!(d, 1),
        _ => panic!("Expected Distance variant"),
    }
    match protected_field.permission_policy.write_policy {
        fold_node::testing::TrustDistance::Distance(d) => assert_eq!(d, 2),
        _ => panic!("Expected Distance variant"),
    }

    let private_field = schema.fields.get("private_field").unwrap();
    match private_field.permission_policy.read_policy {
        fold_node::testing::TrustDistance::Distance(d) => assert_eq!(d, 3),
        _ => panic!("Expected Distance variant"),
    }
}

#[test]
fn test_user_profile_schema() {
    let schema = create_user_profile_schema();

    // Verify schema structure
    assert_eq!(schema.name, "user_profile");
    assert_eq!(schema.fields.len(), 3);

    // Verify field permissions
    let username_field = schema.fields.get("username").unwrap();
    match username_field.permission_policy.read_policy {
        fold_node::testing::TrustDistance::Distance(d) => assert_eq!(d, 0), // Public read
        _ => panic!("Expected Distance variant"),
    }

    let email_field = schema.fields.get("email").unwrap();
    match email_field.permission_policy.read_policy {
        fold_node::testing::TrustDistance::Distance(d) => assert_eq!(d, 1), // Limited read
        _ => panic!("Expected Distance variant"),
    }

    let payment_field = schema.fields.get("payment_info").unwrap();
    match payment_field.permission_policy.read_policy {
        fold_node::testing::TrustDistance::Distance(d) => assert_eq!(d, 3), // Restricted read
        _ => panic!("Expected Distance variant"),
    }
}

#[test]
fn test_schema_persistence() {
    // Create a temporary directory for test
    let test_dir = tempdir().unwrap();
    let manager = SchemaCore::new(test_dir.path().to_str().unwrap());

    // Create and load a test schema
    let schema = create_test_schema("test_persistence");
    manager.load_schema(schema.clone()).unwrap();
    
    // Test schema retrieval
    let loaded_schema = manager.get_schema("test_persistence").unwrap().unwrap();
    assert_eq!(loaded_schema.name, "test_persistence");
    assert_eq!(
        loaded_schema.fields.get("test_field").unwrap().get_ref_atom_uuid(),
        Some("test-uuid".to_string())
    );

    // Create a new manager instance to verify disk persistence
    let new_manager = SchemaCore::new(test_dir.path().to_str().unwrap());
    new_manager.load_schemas_from_disk().unwrap();
    let reloaded_schema = new_manager.get_schema("test_persistence").unwrap().unwrap();
    assert_eq!(reloaded_schema.name, "test_persistence");
    
    // Test schema unloading
    assert!(manager.unload_schema("test_persistence").unwrap());
    
    // Verify schema was removed
    let removed_schema = manager.get_schema("test_persistence").unwrap();
    assert!(removed_schema.is_none());
}

#[test]
fn test_schema_disk_loading() {
    // Create a temporary directory for test
    let test_dir = tempdir().unwrap();
    let manager = SchemaCore::new(test_dir.path().to_str().unwrap());

    // Create and save multiple schemas
    let schemas = vec![
        ("schema1", "field1"),
        ("schema2", "field2"),
        ("schema3", "field3"),
    ];

    for (schema_name, _) in &schemas {
        let schema = create_test_schema(schema_name);
        manager.load_schema(schema).unwrap();
    }

    // Create a new manager instance and load schemas from disk
    let new_manager = SchemaCore::new(test_dir.path().to_str().unwrap());
    new_manager.load_schemas_from_disk().unwrap();

    // Verify all schemas were loaded
    for (schema_name, _) in schemas {
        let schema = new_manager.get_schema(schema_name).unwrap().unwrap();
        assert_eq!(schema.name, schema_name);
    }
}
