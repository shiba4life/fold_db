use fold_db::fees::types::config::TrustDistanceScaling;
use fold_db::permissions::types::policy::TrustDistance;
use fold_db::schema_interpreter::SchemaInterpreter;

#[test]
fn test_interpret_user_profile_schema() {
    let json_str = r#"{
        "name": "UserProfile",
        "fields": {
            "username": {
                "permission_policy": {
                    "read": {
                        "NoRequirement": null
                    },
                    "write": {
                        "Distance": 0
                    },
                    "explicit_read": null,
                    "explicit_write": null
                },
                "ref_atom_uuid": "username_atom_123",
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": {
                        "None": null
                    },
                    "min_payment": null
                }
            },
            "email": {
                "permission_policy": {
                    "read": {
                        "Distance": 1
                    },
                    "write": {
                        "Distance": 0
                    },
                    "explicit_read": {
                        "counts_by_pub_key": {
                            "trusted_service_key": 1
                        }
                    },
                    "explicit_write": null
                },
                "ref_atom_uuid": "email_atom_456",
                "payment_config": {
                    "base_multiplier": 2.0,
                    "trust_distance_scaling": {
                        "Linear": {
                            "slope": 0.5,
                            "intercept": 1.0,
                            "min_factor": 1.0
                        }
                    },
                    "min_payment": 1000
                }
            }
        },
        "schema_mappers": [
            {
                "source_schemas": ["LegacyUser"],
                "target_schema": "UserProfile",
                "rules": [
                    {
                        "Rename": {
                            "source_field": "user_name",
                            "target_field": "username"
                        }
                    },
                    {
                        "Map": {
                            "source_field": "email_address",
                            "target_field": "email"
                        }
                    }
                ]
            }
        ],
        "payment_config": {
            "base_multiplier": 1.5,
            "min_payment_threshold": 500
        }
    }"#;

    let interpreter = SchemaInterpreter::new();
    let result = interpreter.interpret_str(json_str);
    assert!(result.is_ok());

    let schema = result.unwrap();
    assert_eq!(schema.name, "UserProfile");
    assert_eq!(schema.payment_config.base_multiplier, 1.5);
    assert_eq!(schema.payment_config.min_payment_threshold, 500);

    // Verify username field
    let username_field = schema.fields.get("username").unwrap();
    assert!(matches!(
        username_field.permission_policy.read_policy,
        TrustDistance::NoRequirement
    ));
    assert!(matches!(
        username_field.permission_policy.write_policy,
        TrustDistance::Distance(0)
    ));
    assert!(username_field
        .permission_policy
        .explicit_read_policy
        .is_none());
    assert!(username_field
        .permission_policy
        .explicit_write_policy
        .is_none());
    assert_eq!(username_field.ref_atom_uuid, Some("username_atom_123".to_string()));
    assert_eq!(username_field.payment_config.base_multiplier, 1.0);
    assert!(matches!(
        username_field.payment_config.trust_distance_scaling,
        TrustDistanceScaling::None
    ));
    assert!(username_field.payment_config.min_payment.is_none());

    // Verify email field
    let email_field = schema.fields.get("email").unwrap();
    assert!(matches!(
        email_field.permission_policy.read_policy,
        TrustDistance::Distance(1)
    ));
    assert!(matches!(
        email_field.permission_policy.write_policy,
        TrustDistance::Distance(0)
    ));
    assert!(email_field
        .permission_policy
        .explicit_write_policy
        .is_none());

    // Verify explicit read policy
    let explicit_read = email_field
        .permission_policy
        .explicit_read_policy
        .as_ref()
        .unwrap();
    assert_eq!(
        explicit_read.counts_by_pub_key.get("trusted_service_key"),
        Some(&1)
    );

    assert_eq!(email_field.ref_atom_uuid, Some("email_atom_456".to_string()));
    assert_eq!(email_field.payment_config.base_multiplier, 2.0);
    assert!(matches!(
        email_field.payment_config.trust_distance_scaling,
        TrustDistanceScaling::Linear {
            slope: 0.5,
            intercept: 1.0,
            min_factor: 1.0
        }
    ));
    assert_eq!(email_field.payment_config.min_payment, Some(1000));
}

#[test]
fn test_invalid_schema_validation() {
    let invalid_json_str = r#"{
        "name": "InvalidSchema",
        "fields": {
            "field1": {
                "permission_policy": {
                    "read": {
                        "Distance": -1
                    },
                    "write": {
                        "Distance": 0
                    },
                    "explicit_read": null,
                    "explicit_write": null
                },
                "ref_atom_uuid": "field1_atom",
                "payment_config": {
                    "base_multiplier": 0.0,
                    "trust_distance_scaling": {
                        "None": null
                    },
                    "min_payment": null
                }
            }
        },
        "schema_mappers": [],
        "payment_config": {
            "base_multiplier": 1.0,
            "min_payment_threshold": 0
        }
    }"#;

    let interpreter = SchemaInterpreter::new();
    let result = interpreter.interpret_str(invalid_json_str);
    assert!(result.is_err());
}

#[test]
fn test_schema_from_file() {
    let interpreter = SchemaInterpreter::new();
    let result = interpreter.interpret_file("src/schema_interpreter/examples/user_profile.json");
    assert!(result.is_ok());
}
