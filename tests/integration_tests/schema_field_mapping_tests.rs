use fold_db::{DataFoldNode, NodeConfig};
use fold_db::schema::Schema;
use serde_json::json;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_field_mappers_share_aref_uuid() {
    // Create a temporary directory for the test
    let dir = tempdir().unwrap();
    let config = NodeConfig {
        storage_path: dir.path().to_path_buf(),
        default_trust_distance: 1,
    };
    
    // Create a new DataFoldNode
    let mut node = DataFoldNode::new(config).unwrap();
    
    // Create source schema (UserProfile)
    let source_schema_json = r#"{
        "name": "UserProfile",
        "fields": {
            "username": {
                "field_type": "Single",
                "permission_policy": {
                    "read_policy": { "NoRequirement": null },
                    "write_policy": { "Distance": 0 },
                    "explicit_read_policy": null,
                    "explicit_write_policy": null
                },
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": { "None": null },
                    "min_payment": null
                },
                "field_mappers": {},
                "ref_atom_uuid": "test-uuid-123"
            }
        },
        "payment_config": {
            "base_multiplier": 1.5,
            "min_payment_threshold": 500
        }
    }"#;
    
    let source_schema: Schema = serde_json::from_str(source_schema_json).unwrap();
    node.load_schema(source_schema).unwrap();
    
    // Create target schema (UserProfile2) with field_mappers
    let target_schema_json = r#"{
        "name": "UserProfile2",
        "fields": {
            "user_name": {
                "field_type": "Single",
                "permission_policy": {
                    "read_policy": { "NoRequirement": null },
                    "write_policy": { "Distance": 0 },
                    "explicit_read_policy": null,
                    "explicit_write_policy": null
                },
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": { "None": null },
                    "min_payment": null
                },
                "field_mappers": {
                    "UserProfile": "username"
                }
            }
        },
        "payment_config": {
            "base_multiplier": 1.5,
            "min_payment_threshold": 500
        }
    }"#;
    
    let target_schema: Schema = serde_json::from_str(target_schema_json).unwrap();
    node.load_schema(target_schema).unwrap();
    
    // Get both schemas after loading
    let source_schema = node.get_schema("UserProfile").unwrap().unwrap();
    let target_schema = node.get_schema("UserProfile2").unwrap().unwrap();
    
    // Get the ref_atom_uuid from the source field
    let source_field = source_schema.fields.get("username").unwrap();
    let source_ref_atom_uuid = source_field.get_ref_atom_uuid();
    
    // Get the ref_atom_uuid from the target field
    let target_field = target_schema.fields.get("user_name").unwrap();
    let target_ref_atom_uuid = target_field.get_ref_atom_uuid();
    
    // Verify that both fields have ref_atom_uuid values
    assert!(source_ref_atom_uuid.is_some(), "Source field should have a ref_atom_uuid");
    assert!(target_ref_atom_uuid.is_some(), "Target field should have a ref_atom_uuid");
    
    // Verify that they are the same
    assert_eq!(source_ref_atom_uuid, target_ref_atom_uuid, 
        "Field mappers should ensure target field uses the same ref_atom_uuid as the source field");
    
    // Test with a mutation to verify data is shared
    let mutation = json!({
        "type": "mutation",
        "schema": "UserProfile",
        "mutation_type": "create",
        "data": {
            "username": "testuser"
        }
    });
    
    // Execute the mutation
    node.execute_operation(serde_json::from_value(mutation).unwrap()).unwrap();
    
    // Query both schemas to verify data is shared
    let query1 = json!({
        "type": "query",
        "schema": "UserProfile",
        "fields": ["username"],
        "filter": null
    });
    
    let query2 = json!({
        "type": "query",
        "schema": "UserProfile2",
        "fields": ["user_name"],
        "filter": null
    });
    
    let result1 = node.execute_operation(serde_json::from_value(query1).unwrap()).unwrap();
    let result2 = node.execute_operation(serde_json::from_value(query2).unwrap()).unwrap();
    
    // Extract the values from the results
    let value1 = result1.as_array().unwrap()[0].as_str().unwrap();
    let value2 = result2.as_array().unwrap()[0].as_str().unwrap();
    
    // Verify that they are the same
    assert_eq!(value1, "testuser", "Source field should contain the value we set");
    assert_eq!(value2, "testuser", "Target field should contain the same value as the source field");
}
