use fold_node::db_operations::core::DbOperations;
use fold_node::testing::SchemaCore;
use std::sync::Arc;
use tempfile::tempdir;

fn create_test_db_ops() -> Arc<DbOperations> {
    let db = sled::Config::new().temporary(true).open().unwrap();
    Arc::new(DbOperations::new(db).unwrap())
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

    let temp_dir = tempdir().unwrap();
    let db_ops = create_test_db_ops();
    let core = SchemaCore::new(temp_dir.path().to_str().unwrap(), db_ops).unwrap();
    let result = core.load_schema_from_json(invalid_json_str);
    assert!(result.is_err());
}
