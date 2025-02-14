use fold_db::fees::types::{FieldPaymentConfig, TrustDistanceScaling};
use fold_db::permissions::types::policy::{PermissionsPolicy, TrustDistance};
use fold_db::schema::types::fields::SchemaField;
use fold_db::schema::{Schema, SchemaManager};
use uuid::Uuid;

fn create_default_payment_config() -> FieldPaymentConfig {
    FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap()
}

#[test]
fn test_schema_creation() {
    let schema_name = "test_schema".to_string();
    let schema = Schema::new(schema_name.clone());

    assert_eq!(schema.name, schema_name);
    assert!(schema.fields.is_empty());
}

#[test]
fn test_schema_field_management() {
    let mut schema = Schema::new("test_schema".to_string());
    let field_name = "test_field".to_string();
    let field = SchemaField::new(
        PermissionsPolicy::default(),
        create_default_payment_config(),
    ).with_ref_atom_uuid("test-uuid".to_string());

    // Add field
    schema.add_field(field_name.clone(), field.clone());

    // Verify field was added
    assert!(schema.fields.contains_key(&field_name));
    assert_eq!(
        schema.fields.get(&field_name).unwrap().ref_atom_uuid,
        Some("test-uuid".to_string())
    );
}

#[test]
fn test_schema_field_permissions() {
    let mut schema = Schema::new("test_schema".to_string());
    let field_name = "protected_field".to_string();

    // Create field with custom permissions
    let field = SchemaField::new(
        PermissionsPolicy::new(
            TrustDistance::Distance(2),
            TrustDistance::Distance(3),
        ),
        create_default_payment_config(),
    ).with_ref_atom_uuid(Uuid::new_v4().to_string());

    schema.add_field(field_name.clone(), field.clone());

    // Verify permissions
    let stored_field = schema.fields.get(&field_name).unwrap();
    match stored_field.permission_policy.read_policy {
        TrustDistance::Distance(d) => assert_eq!(d, 2),
        _ => panic!("Expected Distance variant"),
    }
    match stored_field.permission_policy.write_policy {
        TrustDistance::Distance(d) => assert_eq!(d, 3),
        _ => panic!("Expected Distance variant"),
    }
}

#[test]
fn test_schema_with_multiple_fields() {
    let mut schema = Schema::new("test_schema".to_string());

    // Add multiple fields with different permissions
    let fields = vec![
        ("public_field", PermissionsPolicy::default()),
        (
            "protected_field",
            PermissionsPolicy::new(TrustDistance::Distance(1), TrustDistance::Distance(2)),
        ),
        (
            "private_field",
            PermissionsPolicy::new(TrustDistance::Distance(3), TrustDistance::Distance(3)),
        ),
    ];

    for (name, policy) in fields {
        schema.add_field(
            name.to_string(),
            SchemaField::new(
                policy,
                create_default_payment_config(),
            ).with_ref_atom_uuid(Uuid::new_v4().to_string()),
        );
    }

    // Verify all fields were added with correct permissions
    assert_eq!(schema.fields.len(), 3);
    match &schema
        .fields
        .get("public_field")
        .unwrap()
        .permission_policy
        .read_policy
    {
        TrustDistance::Distance(d) => assert_eq!(*d, 0),
        _ => panic!("Expected Distance variant"),
    }
    match &schema
        .fields
        .get("protected_field")
        .unwrap()
        .permission_policy
        .read_policy
    {
        TrustDistance::Distance(d) => assert_eq!(*d, 1),
        _ => panic!("Expected Distance variant"),
    }
    match &schema
        .fields
        .get("private_field")
        .unwrap()
        .permission_policy
        .read_policy
    {
        TrustDistance::Distance(d) => assert_eq!(*d, 3),
        _ => panic!("Expected Distance variant"),
    }
}
