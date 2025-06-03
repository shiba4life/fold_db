use fold_node::testing::{
    Field, FieldPaymentConfig, FieldVariant, PermissionsPolicy, RangeField, Schema, SingleField, TrustDistance,
    TrustDistanceScaling,
};
use std::collections::HashMap;
use uuid::Uuid;

#[allow(dead_code)]
pub fn create_default_payment_config() -> FieldPaymentConfig {
    FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap()
}

#[allow(dead_code)]
pub fn create_test_schema(name: &str) -> Schema {
    // Use regular schema for ref_atom_uuid preservation tests (not range schema)
    let mut schema = Schema::new(name.to_string());

    // Add the test field as a Single field (since this test is about ref_atom_uuid preservation)
    let field_name = "test_field".to_string();
    let mut field = SingleField::new(
        PermissionsPolicy::default(),
        create_default_payment_config(),
        HashMap::new(),
    );
    field.set_ref_atom_uuid("test-uuid".to_string());

    schema.add_field(field_name, FieldVariant::Single(field));
    schema
}

#[allow(dead_code)]
pub fn create_user_profile_schema() -> Schema {
    let mut schema = Schema::new_range("user_profile".to_string(), "key".to_string());

    // Add the range_key field as a Range field (required for range schemas)
    let key_field = RangeField::new(
        PermissionsPolicy::default(),
        create_default_payment_config(),
        HashMap::new(),
    );
    schema.add_field("key".to_string(), FieldVariant::Range(key_field));

    // Public fields - basic profile info (converted to Range fields)
    let username_field = RangeField::new(
        PermissionsPolicy::default(), // Public read access
        create_default_payment_config(),
        HashMap::new(),
    );
    schema.add_field("username".to_string(), FieldVariant::Range(username_field));

    // Protected fields - contact info (converted to Range fields)
    let email_field = RangeField::new(
        PermissionsPolicy::new(
            TrustDistance::Distance(1), // Limited read access
            TrustDistance::Distance(1), // Limited write access
        ),
        create_default_payment_config(),
        HashMap::new(),
    );
    schema.add_field("email".to_string(), FieldVariant::Range(email_field));

    // Private fields - sensitive info (converted to Range fields)
    let payment_field = RangeField::new(
        PermissionsPolicy::new(
            TrustDistance::Distance(3), // Restricted read access
            TrustDistance::Distance(3), // Restricted write access
        ),
        create_default_payment_config(),
        HashMap::new(),
    );
    schema.add_field(
        "payment_info".to_string(),
        FieldVariant::Range(payment_field),
    );

    schema
}

#[allow(dead_code)]
pub fn create_multi_field_schema() -> Schema {
    let mut schema = Schema::new_range("test_schema".to_string(), "key".to_string());

    // Add the range_key field as a Range field (required for range schemas)
    let key_field = RangeField::new(
        PermissionsPolicy::default(),
        create_default_payment_config(),
        HashMap::new(),
    );
    schema.add_field("key".to_string(), FieldVariant::Range(key_field));

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
        let field = RangeField::new(policy, create_default_payment_config(), HashMap::new());
        schema.add_field(name.to_string(), FieldVariant::Range(field));
    }

    schema
}
