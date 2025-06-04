//! Test to reproduce and diagnose the UUID mismatch issue
//! 
//! This test reproduces the exact problem described:
//! - Mutation creates atoms with UUIDs: 4338b8fc-dd04-4033-ad8a-f0f2cbdeafd3, 815d0069-e5c3-43e0-ad9c-f00be8e9e3ae
//! - Query looks for atoms with different UUIDs: 544410db-7e5a-4683-af18-3ffc0e534c2c, 2495f451-9180-45b4-846d-ce7dba341dd7
//! - Root cause: FieldManager creates new AtomRefs but doesn't update schema field definitions

use std::sync::Arc;
use datafold::fold_db_core::FoldDB;
use datafold::schema::types::{Mutation, Query};
use serde_json::{json, Value};
use log::info;

#[tokio::test]
async fn test_uuid_mismatch_diagnosis() {
    // Initialize logging to see our diagnostic messages
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init()
        .ok();

    info!("🧪 Starting UUID mismatch diagnosis test");

    // Create test database
    let test_db_path = "test_uuid_mismatch_db";
    let _ = std::fs::remove_dir_all(test_db_path);
    
    let fold_db = FoldDB::new(test_db_path).expect("Failed to create FoldDB");

    // 1. CREATE AND APPROVE TRANSFORMBASE SCHEMA
    let schema_json = json!({
        "name": "TransformBase",
        "fields": {
            "value1": {
                "field_type": "Single",
                "permission_policy": {
                    "read_policy": {"NoRequirement": null},
                    "write_policy": {"Distance": 0}
                },
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": {"None": null}
                },
                "field_mappers": {}
            },
            "value2": {
                "field_type": "Single",
                "permission_policy": {
                    "read_policy": {"NoRequirement": null},
                    "write_policy": {"Distance": 0}
                },
                "payment_config": {
                    "base_multiplier": 1.0,
                    "trust_distance_scaling": {"None": null}
                },
                "field_mappers": {}
            }
        },
        "payment_config": {
            "base_multiplier": 1.0,
            "min_payment_threshold": 0
        }
    });

    info!("📋 Loading TransformBase schema");
    fold_db.load_schema_from_json(&schema_json.to_string())
        .expect("Failed to load schema");
    
    info!("✅ Approving TransformBase schema");
    fold_db.approve_schema("TransformBase")
        .expect("Failed to approve schema");

    // Give time for field mapping to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // 2. CHECK INITIAL SCHEMA FIELD UUIDS
    info!("🔍 Checking initial schema field UUIDs");
    if let Ok(Some(schema)) = fold_db.get_schema("TransformBase") {
        if let Some(value1_field) = schema.fields.get("value1") {
            if let Some(uuid) = value1_field.ref_atom_uuid() {
                info!("📋 INITIAL STATE: value1 ref_atom_uuid = {}", uuid);
            } else {
                info!("📋 INITIAL STATE: value1 has NO ref_atom_uuid");
            }
        }
        if let Some(value2_field) = schema.fields.get("value2") {
            if let Some(uuid) = value2_field.ref_atom_uuid() {
                info!("📋 INITIAL STATE: value2 ref_atom_uuid = {}", uuid);
            } else {
                info!("📋 INITIAL STATE: value2 has NO ref_atom_uuid");
            }
        }
    }

    // 3. PERFORM MUTATION (creates new AtomRefs)
    info!("🔧 MUTATION PHASE: Creating mutation that will generate new AtomRef UUIDs");
    let mutation = Mutation {
        schema: "TransformBase".to_string(),
        fields_and_values: {
            let mut map = std::collections::HashMap::new();
            map.insert("value1".to_string(), json!("test_value_1"));
            map.insert("value2".to_string(), json!("test_value_2"));
            map
        },
    };

    info!("🚀 Executing mutation...");
    match fold_db.execute_mutation(&mutation).await {
        Ok(_) => info!("✅ Mutation executed successfully"),
        Err(e) => {
            info!("❌ Mutation failed: {}", e);
            panic!("Mutation should succeed");
        }
    }

    // Give time for mutation processing
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // 4. CHECK SCHEMA FIELD UUIDS AFTER MUTATION
    info!("🔍 Checking schema field UUIDs AFTER mutation");
    if let Ok(Some(schema)) = fold_db.get_schema("TransformBase") {
        if let Some(value1_field) = schema.fields.get("value1") {
            if let Some(uuid) = value1_field.ref_atom_uuid() {
                info!("📋 AFTER MUTATION: value1 ref_atom_uuid = {}", uuid);
            } else {
                info!("📋 AFTER MUTATION: value1 STILL has NO ref_atom_uuid!");
            }
        }
        if let Some(value2_field) = schema.fields.get("value2") {
            if let Some(uuid) = value2_field.ref_atom_uuid() {
                info!("📋 AFTER MUTATION: value2 ref_atom_uuid = {}", uuid);
            } else {
                info!("📋 AFTER MUTATION: value2 STILL has NO ref_atom_uuid!");
            }
        }
    }

    // 5. PERFORM QUERY (will show mismatch via diagnostic logs)
    info!("🔍 QUERY PHASE: Attempting to query field values");
    let query = Query {
        schema: "TransformBase".to_string(),
        fields: vec!["value1".to_string(), "value2".to_string()],
        filter: None,
    };

    info!("🚀 Executing query...");
    match fold_db.execute_query(&query).await {
        Ok(result) => {
            info!("✅ Query completed with result: {:?}", result);
        }
        Err(e) => {
            info!("❌ Query failed: {}", e);
            info!("🚨 This failure is expected due to UUID mismatch!");
        }
    }

    // 6. SUMMARY
    info!("📊 UUID MISMATCH DIAGNOSIS COMPLETE");
    info!("🔍 Check the logs above for:");
    info!("   - 'UUID MISMATCH DIAGNOSIS' messages from FieldManager");
    info!("   - 'QUERY DIAGNOSIS' messages from FieldRetrievalService");
    info!("   - The actual UUIDs being created vs. looked up");

    // Cleanup
    let _ = std::fs::remove_dir_all(test_db_path);
}