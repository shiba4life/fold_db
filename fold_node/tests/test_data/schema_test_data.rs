use fold_node::testing::{
    Field, FieldPaymentConfig, FieldVariant, PermissionsPolicy, Schema, SingleField, TrustDistance,
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
    let mut schema = Schema::new_range(name.to_string(), "key".to_string());

    // Add the range_key field
    let key_field = SingleField::new(
        PermissionsPolicy::default(),
        create_default_payment_config(),
        HashMap::new(),
    );
    schema.add_field("key".to_string(), FieldVariant::Single(key_field));

    // Add the test field
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

    // Add the range_key field
    let key_field = SingleField::new(
        PermissionsPolicy::default(),
        create_default_payment_config(),
        HashMap::new(),
    );
    schema.add_field("key".to_string(), FieldVariant::Single(key_field));

    // Public fields - basic profile info
    let mut username_field = SingleField::new(
        PermissionsPolicy::default(), // Public read access
        create_default_payment_config(),
        HashMap::new(),
    );
    username_field.set_ref_atom_uuid(Uuid::new_v4().to_string());
    schema.add_field("username".to_string(), FieldVariant::Single(username_field));

    // Protected fields - contact info
    let mut email_field = SingleField::new(
        PermissionsPolicy::new(
            TrustDistance::Distance(1), // Limited read access
            TrustDistance::Distance(1), // Limited write access
        ),
        create_default_payment_config(),
        HashMap::new(),
    );
    email_field.set_ref_atom_uuid(Uuid::new_v4().to_string());
    schema.add_field("email".to_string(), FieldVariant::Single(email_field));

    // Private fields - sensitive info
    let mut payment_field = SingleField::new(
        PermissionsPolicy::new(
            TrustDistance::Distance(3), // Restricted read access
            TrustDistance::Distance(3), // Restricted write access
        ),
        create_default_payment_config(),
        HashMap::new(),
    );
    payment_field.set_ref_atom_uuid(Uuid::new_v4().to_string());
    schema.add_field(
        "payment_info".to_string(),
        FieldVariant::Single(payment_field),
    );

    schema
}

#[allow(dead_code)]
pub fn create_multi_field_schema() -> Schema {
    let mut schema = Schema::new_range("test_schema".to_string(), "key".to_string());

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
        let mut field = SingleField::new(policy, create_default_payment_config(), HashMap::new());
        field.set_ref_atom_uuid(Uuid::new_v4().to_string());
        schema.add_field(name.to_string(), FieldVariant::Single(field));
    }

    schema
}
