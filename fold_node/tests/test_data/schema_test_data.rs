use fold_node::testing::FieldType;
use fold_node::testing::{
    FieldPaymentConfig, PermissionsPolicy, Schema, SchemaField, TrustDistance, TrustDistanceScaling,
};
use std::collections::HashMap;
use uuid::Uuid;

pub fn create_default_payment_config() -> FieldPaymentConfig {
    FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap()
}

pub fn create_test_schema(name: &str) -> Schema {
    let mut schema = Schema::new(name.to_string());
    let field_name = "test_field".to_string();
    let field = SchemaField::new(
        PermissionsPolicy::default(),
        create_default_payment_config(),
        HashMap::new(),
        Some(FieldType::Single),
    )
    .with_ref_atom_uuid("test-uuid".to_string());

    schema.add_field(field_name, field);
    schema
}

pub fn create_user_profile_schema() -> Schema {
    let mut schema = Schema::new("user_profile".to_string());

    // Public fields - basic profile info
    schema.add_field(
        "username".to_string(),
        SchemaField::new(
            PermissionsPolicy::default(), // Public read access
            create_default_payment_config(),
            HashMap::new(),
            Some(FieldType::Single),
        )
        .with_ref_atom_uuid(Uuid::new_v4().to_string()),
    );

    // Protected fields - contact info
    schema.add_field(
        "email".to_string(),
        SchemaField::new(
            PermissionsPolicy::new(
                TrustDistance::Distance(1), // Limited read access
                TrustDistance::Distance(1), // Limited write access
            ),
            create_default_payment_config(),
            HashMap::new(),
            Some(FieldType::Single),
        )
        .with_ref_atom_uuid(Uuid::new_v4().to_string()),
    );

    // Private fields - sensitive info
    schema.add_field(
        "payment_info".to_string(),
        SchemaField::new(
            PermissionsPolicy::new(
                TrustDistance::Distance(3), // Restricted read access
                TrustDistance::Distance(3), // Restricted write access
            ),
            create_default_payment_config(),
            HashMap::new(),
            Some(FieldType::Single),
        )
        .with_ref_atom_uuid(Uuid::new_v4().to_string()),
    );

    schema
}

pub fn create_multi_field_schema() -> Schema {
    let mut schema = Schema::new("test_schema".to_string());

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
                HashMap::new(),
                Some(FieldType::Single),
            )
            .with_ref_atom_uuid(Uuid::new_v4().to_string()),
        );
    }

    schema
}
