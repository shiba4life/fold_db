use fold_node::datafold_node::{loader::load_schema_from_file, DataFoldNode, config::NodeConfig};
use tempfile::tempdir;
use fold_node::schema::types::field::Field;
use std::fs;

fn create_node(path: &std::path::Path) -> Result<DataFoldNode, Box<dyn std::error::Error>> {
    let config = NodeConfig {
        storage_path: path.into(),
        default_trust_distance: 1,
        network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
    };
    Ok(DataFoldNode::new(config)?)
}

#[test]
fn test_load_schema_from_config() -> Result<(), Box<dyn std::error::Error>> {
    let test_dir = tempdir()?;
    let db_path = test_dir.path().join("test_db");
    let schema_path = test_dir.path().join("test_schema.json");
    let test_schema = r#"{
        "name": "test_schema",
        "fields": {},
        "payment_config": {
            "base_multiplier": 1.0,
            "min_payment_threshold": 0
        }
    }"#;
    fs::write(&schema_path, test_schema)?;

    let mut node = create_node(&db_path)?;
    load_schema_from_file(&schema_path, &mut node)?;
    Ok(())
}

#[test]
fn test_load_schema_with_transform_object() -> Result<(), Box<dyn std::error::Error>> {
    let test_dir = tempdir()?;
    let db_path = test_dir.path().join("test_db");
    let schema_path = test_dir.path().join("transform_schema.json");
    let test_schema = r#"{
        "name": "transform_schema",
        "fields": {
            "computed": {
                "permission_policy": {
                    "read_policy": { "Distance": 0 },
                    "write_policy": { "Distance": 0 },
                    "explicit_read_policy": null,
                    "explicit_write_policy": null
                },
                "ref_atom_uuid": "calc_uuid",
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": { "None": null },
                    "min_payment": null
                },
                "field_mappers": {},
                "field_type": "Single",
                "transform": {
                    "logic": "4 + 5",
                    "inputs": [],
                    "output": "transform_schema.computed"
                }
            }
        },
        "payment_config": {
            "base_multiplier": 1.0,
            "min_payment_threshold": 0
        }
    }"#;
    fs::write(&schema_path, test_schema)?;

    let mut node = create_node(&db_path)?;
    load_schema_from_file(&schema_path, &mut node)?;

    let loaded_schema = node
        .get_schema("transform_schema")?
        .expect("schema not found");
    let field = loaded_schema
        .fields
        .get("computed")
        .expect("field not found");
    assert!(field.transform().is_some());
    assert_eq!(field.transform().unwrap().logic, "4 + 5");

    Ok(())
}

#[test]
fn test_load_schema_invalid_fails() -> Result<(), Box<dyn std::error::Error>> {
    let test_dir = tempdir()?;
    let db_path = test_dir.path().join("test_db");
    let schema_path = test_dir.path().join("invalid_schema.json");
    let invalid_schema = r#"{
        "name": "invalid_schema",
        "fields": {},
        "payment_config": {
            "base_multiplier": 0.0,
            "min_payment_threshold": 0
        }
    }"#;
    fs::write(&schema_path, invalid_schema)?;

    let mut node = create_node(&db_path)?;
    let res = load_schema_from_file(&schema_path, &mut node);
    assert!(res.is_err(), "invalid schema should fail to load");
    Ok(())
}
