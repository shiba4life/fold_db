use fold_node::testing::{
    FieldPaymentConfig, PermissionsPolicy, Schema, FieldVariant, SingleField, TrustDistance, TrustDistanceScaling,
};
use std::collections::HashMap;

#[allow(dead_code)]
pub fn create_default_payment_config() -> FieldPaymentConfig {
    FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap()
}

#[allow(dead_code)]
pub fn create_test_schema(name: &str) -> Schema {
    let mut schema = Schema::new_range(name.to_string(), "key".to_string());
    let field_name = "test_field".to_string();
    let field = FieldVariant::Single(SingleField::new(
        PermissionsPolicy::default(),
        create_default_payment_config(),
        HashMap::new(),
    ));

    schema.add_field(field_name, field);
    schema
}

#[allow(dead_code)]
pub fn create_basic_user_profile_schema() -> Schema {
    let mut schema = Schema::new_range("user_profile".to_string(), "key".to_string());

    let name_field = FieldVariant::Single(SingleField::new(
        PermissionsPolicy::new(TrustDistance::Distance(1), TrustDistance::Distance(1)),
        create_default_payment_config(),
        HashMap::new(),
    ));
    schema.add_field("name".to_string(), name_field);

    let email_field = FieldVariant::Single(SingleField::new(
        PermissionsPolicy::new(TrustDistance::Distance(1), TrustDistance::Distance(1)),
        create_default_payment_config(),
        HashMap::new(),
    ));
    schema.add_field("email".to_string(), email_field);

    schema
}

#[allow(dead_code)]
pub fn create_user_profile_schema() -> Schema {
    let mut schema = Schema::new_range("user_profile".to_string(), "key".to_string());

    // Public fields - basic profile info
    schema.add_field(
        "username".to_string(),
        FieldVariant::Single(
            SingleField::new(
                PermissionsPolicy::default(),
                create_default_payment_config(),
                HashMap::new(),
            )
        )
    );

    // Protected fields - contact info
    schema.add_field(
        "email".to_string(),
        FieldVariant::Single(
            SingleField::new(
            PermissionsPolicy::new(
                TrustDistance::Distance(1), // Limited read access
                TrustDistance::Distance(1), // Limited write access
            ),
            create_default_payment_config(),
            HashMap::new(),
        )));

    // Private fields - sensitive info
    schema.add_field(
        "payment_info".to_string(),
        FieldVariant::Single(
            SingleField::new(
            PermissionsPolicy::new(
                TrustDistance::Distance(3), // Restricted read access
                TrustDistance::Distance(3), // Restricted write access
            ),
            create_default_payment_config(),
            HashMap::new(),
        )));

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
        schema.add_field(
            name.to_string(),
            FieldVariant::Single(
                SingleField::new(
                policy,
                create_default_payment_config(),
                HashMap::new(),
            ))
        );
    }

    schema
}
