use fold_db::schema_interpreter::SchemaInterpreter;

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
