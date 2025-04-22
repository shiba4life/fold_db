use fold_node::testing::SchemaCore;

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

    let core = SchemaCore::new("data");
    let result = core.load_schema_from_json(invalid_json_str);
    assert!(result.is_err());
}
