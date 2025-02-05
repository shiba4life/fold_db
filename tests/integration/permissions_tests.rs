use std::sync::Arc;
use fold_db::folddb::FoldDB;
use fold_db::schema::types::{Schema, SchemaField, PolicyLevel, PermissionsPolicy};
use serde_json::json;
use uuid::Uuid;

#[test]
fn test_field_permissions() {
    let mut db = FoldDB::new(&crate::get_test_db_path("permissions")).unwrap();

    // Create schema with explicit permissions
    let mut schema = Schema::new("test".to_string());
    
    // Add field with explicit permissions
    let mut field = SchemaField::new(
        "W1".to_string(),
        Uuid::new_v4().to_string(),
    );

    // Set explicit access for test users
    field.add_explicit_access("user1".to_string(), 1, 2); // 1 write, 2 reads
    field.add_explicit_access("user2".to_string(), 3, 4); // 3 writes, 4 reads

    schema.add_field("test_field".to_string(), field);

    // Load schema
    db.load_schema(schema).unwrap();

    // Write initial value
    db.set_field_value(
        "test",
        "test_field",
        json!("test value"),
        "user1".to_string(),
    ).unwrap();

    // Read value
    let value = db.get_field_value("test", "test_field").unwrap();
    assert_eq!(value, json!("test value"));
}

#[test]
fn test_schema_transforms() {
    let mut db = FoldDB::new(&crate::get_test_db_path("transforms")).unwrap();

    // Create schema with transforms
    let mut schema = Schema::new("test".to_string());
    
    // Add transforms
    schema.add_transform("RENAME old_field TO new_field".to_string());
    schema.add_transform("MAP field1 TO field2".to_string());

    // Add field
    let field = SchemaField::new(
        "W1".to_string(),
        Uuid::new_v4().to_string(),
    );
    schema.add_field("test_field".to_string(), field);

    // Load schema
    db.load_schema(schema).unwrap();

    // Write and read value
    db.set_field_value(
        "test",
        "test_field",
        json!("test value"),
        "test".to_string(),
    ).unwrap();

    let value = db.get_field_value("test", "test_field").unwrap();
    assert_eq!(value, json!("test value"));
}
