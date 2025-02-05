use fold_db::schema::Schema;
use fold_db::schema::types::fields::SchemaField;
use fold_db::permissions::types::policy::PermissionsPolicy;
use uuid::Uuid;

#[test]
fn test_schema_creation() {
    let schema_name = "test_schema".to_string();
    let schema = Schema::new(schema_name.clone());
    
    assert_eq!(schema.name, schema_name);
    assert!(schema.fields.is_empty());
    assert!(schema.transforms.is_empty());
}

#[test]
fn test_schema_field_management() {
    let mut schema = Schema::new("test_schema".to_string());
    let field_name = "test_field".to_string();
    let field = SchemaField {
        ref_atom_uuid: Uuid::new_v4().to_string(),
        permission_policy: PermissionsPolicy::default(),
    };

    // Add field
    schema.add_field(field_name.clone(), field.clone());
    
    // Verify field was added
    assert!(schema.fields.contains_key(&field_name));
    assert_eq!(schema.fields.get(&field_name).unwrap().ref_atom_uuid, field.ref_atom_uuid);
}

#[test]
fn test_schema_transform_management() {
    let mut schema = Schema::new("test_schema".to_string());
    let transform = "test_transform".to_string();

    // Add transform
    schema.add_transform(transform.clone());
    
    // Verify transform was added
    assert!(schema.transforms.contains(&transform));
    assert_eq!(schema.transforms.len(), 1);
}

#[test]
fn test_schema_field_permissions() {
    let mut schema = Schema::new("test_schema".to_string());
    let field_name = "protected_field".to_string();
    
    // Create field with custom permissions
    let field = SchemaField {
        ref_atom_uuid: Uuid::new_v4().to_string(),
        permission_policy: PermissionsPolicy::new(2, 3), // read_policy: 2, write_policy: 3
    };

    schema.add_field(field_name.clone(), field.clone());
    
    // Verify permissions
    let stored_field = schema.fields.get(&field_name).unwrap();
    assert_eq!(stored_field.permission_policy.read_policy, 2);
    assert_eq!(stored_field.permission_policy.write_policy, 3);
}

#[test]
fn test_schema_with_multiple_fields() {
    let mut schema = Schema::new("test_schema".to_string());
    
    // Add multiple fields with different permissions
    let fields = vec![
        ("public_field", PermissionsPolicy::default()),
        ("protected_field", PermissionsPolicy::new(1, 2)),
        ("private_field", PermissionsPolicy::new(3, 3)),
    ];

    for (name, policy) in fields {
        schema.add_field(
            name.to_string(),
            SchemaField {
                ref_atom_uuid: Uuid::new_v4().to_string(),
                permission_policy: policy,
            }
        );
    }

    // Verify all fields were added with correct permissions
    assert_eq!(schema.fields.len(), 3);
    assert_eq!(schema.fields.get("public_field").unwrap().permission_policy.read_policy, 0);
    assert_eq!(schema.fields.get("protected_field").unwrap().permission_policy.read_policy, 1);
    assert_eq!(schema.fields.get("private_field").unwrap().permission_policy.read_policy, 3);
}
