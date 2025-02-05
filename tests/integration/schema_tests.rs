use fold_db::schema::types::{Schema, SchemaField, PermissionsPolicy, PolicyLevel, Count, FieldType};
use fold_db::schema::security::SecurityManager;
use uuid::Uuid;

#[test]
fn test_schema_field_permissions() {
    let mut security = SecurityManager::new();
    
    // Register test pub keys
    security.register_pub_key(
        "user1".to_string(),
        "key1".to_string(),
    );
    security.register_pub_key(
        "user2".to_string(),
        "key2".to_string(),
    );

    // Create schema
    let mut schema = Schema::new("test".to_string());
    
    // Add field with explicit permissions
    let mut field = SchemaField::new(
        "W1".to_string(),
        Uuid::new_v4().to_string(),
        FieldType::Single,
    );

    // Set explicit access for key1
    field.add_explicit_access(
        "key1".to_string(),
        1, // write count
        2, // read count
    );

    // Set explicit access for key2
    field.add_explicit_access(
        "key2".to_string(),
        3, // write count
        4, // read count
    );

    schema.add_field("test_field".to_string(), field);

    // Test permissions
    let field = schema.fields.get("test_field").unwrap();
    
    // Test key1 access
    let key1_access = field.explicit_access.get("key1").unwrap();
    assert_eq!(key1_access.w, 1);
    assert_eq!(key1_access.r, 2);

    // Test key2 access
    let key2_access = field.explicit_access.get("key2").unwrap();
    assert_eq!(key2_access.w, 3);
    assert_eq!(key2_access.r, 4);
}

#[test]
fn test_schema_transforms() {
    let mut schema = Schema::new("test".to_string());
    
    schema.add_transform("RENAME field1 TO field2".to_string());
    schema.add_transform("MAP field2 TO field3".to_string());

    assert_eq!(schema.transforms.len(), 2);
    assert_eq!(schema.transforms[0], "RENAME field1 TO field2");
    assert_eq!(schema.transforms[1], "MAP field2 TO field3");
}
