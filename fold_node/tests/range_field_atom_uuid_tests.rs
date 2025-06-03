//! Unit tests for atom_uuid issues in range fields
//!
//! This test suite focuses on reproducing and testing the specific issue where
//! range field deserialization fails due to missing `atom_uuid` fields.

use fold_node::atom::{AtomRefBehavior, AtomRefRange};
use fold_node::db_operations::DbOperations;
use fold_node::fees::types::config::FieldPaymentConfig;
use fold_node::fold_db_core::atom_manager::AtomManager;
use fold_node::fold_db_core::field_manager::FieldManager;
use fold_node::permissions::types::policy::PermissionsPolicy;
use fold_node::schema::types::field::{FieldVariant, RangeField};
use fold_node::schema::Schema;
use serde_json::{json, Value};
use std::collections::HashMap;
use tempfile::tempdir;

/// Test that reproduces the atom_uuid deserialization issue by testing AtomRefRange serialization
#[test]
fn test_atom_ref_range_missing_uuid_deserialization() {
    // Create JSON that might be stored in an atom but lacks the uuid field
    // This simulates what might happen if range data is stored incorrectly
    let problematic_json = json!({
        "atom_uuids": {
            "2024-01-01:daily": "1250",
            "2024-01-01:hourly:00": "45",
            "2024-01-01:hourly:01": "52",
            "2024-01-01:hourly:02": "38",
            "2024-01-02:daily": "1180"
        },
        "updated_at": "2024-01-01T00:00:00Z",
        "status": "Active",
        "update_history": []
        // Missing "uuid" field - this should cause the deserialization error
    });

    // Try to deserialize as AtomRefRange - this should fail
    let result = serde_json::from_value::<AtomRefRange>(problematic_json);
    assert!(
        result.is_err(),
        "Expected deserialization to fail due to missing uuid field"
    );

    let error_msg = result.unwrap_err().to_string();
    println!("Deserialization error: {}", error_msg);
    assert!(
        error_msg.contains("missing field"),
        "Error should mention missing field"
    );
}

/// Test that verifies AtomRefRange requires all necessary fields for proper serialization/deserialization
#[test]
fn test_atom_ref_range_complete_serialization() {
    let source_pub_key = "test_key".to_string();
    let mut atom_ref_range = AtomRefRange::new(source_pub_key);

    // Add some test data similar to EventAnalytics
    atom_ref_range.set_atom_uuid("2024-01-01:daily".to_string(), "atom_uuid_1".to_string());
    atom_ref_range.set_atom_uuid(
        "2024-01-01:hourly:00".to_string(),
        "atom_uuid_2".to_string(),
    );
    atom_ref_range.set_atom_uuid("2024-01-02:daily".to_string(), "atom_uuid_3".to_string());

    // Test serialization
    let serialized = serde_json::to_string(&atom_ref_range).unwrap();
    println!("Serialized AtomRefRange: {}", serialized);

    // Verify the serialized JSON contains all required fields
    let serialized_value: Value = serde_json::from_str(&serialized).unwrap();
    assert!(
        serialized_value.get("uuid").is_some(),
        "uuid field should be present"
    );
    assert!(
        serialized_value.get("atom_uuids").is_some(),
        "atom_uuids field should be present"
    );
    assert!(
        serialized_value.get("updated_at").is_some(),
        "updated_at field should be present"
    );
    assert!(
        serialized_value.get("status").is_some(),
        "status field should be present"
    );
    assert!(
        serialized_value.get("update_history").is_some(),
        "update_history field should be present"
    );

    // Test deserialization
    let deserialized: AtomRefRange = serde_json::from_str(&serialized).unwrap();

    // Verify the data is preserved
    assert_eq!(
        deserialized.get_atom_uuid("2024-01-01:daily"),
        Some(&"atom_uuid_1".to_string())
    );
    assert_eq!(
        deserialized.get_atom_uuid("2024-01-01:hourly:00"),
        Some(&"atom_uuid_2".to_string())
    );
    assert_eq!(
        deserialized.get_atom_uuid("2024-01-02:daily"),
        Some(&"atom_uuid_3".to_string())
    );
    assert_eq!(deserialized.uuid(), atom_ref_range.uuid());
}

/// Test that verifies the structure of serialized AtomRefRange contains all required fields
#[test]
fn test_atom_ref_range_serialized_structure() {
    let source_pub_key = "test_key".to_string();
    let mut atom_ref_range = AtomRefRange::new(source_pub_key);
    atom_ref_range.set_atom_uuid("key1".to_string(), "value1".to_string());

    let serialized_value: Value = serde_json::to_value(&atom_ref_range).unwrap();

    // Verify all required fields are present
    assert!(
        serialized_value.get("uuid").is_some(),
        "uuid field should be present"
    );
    assert!(
        serialized_value.get("atom_uuids").is_some(),
        "atom_uuids field should be present"
    );
    assert!(
        serialized_value.get("updated_at").is_some(),
        "updated_at field should be present"
    );
    assert!(
        serialized_value.get("status").is_some(),
        "status field should be present"
    );
    assert!(
        serialized_value.get("update_history").is_some(),
        "update_history field should be present"
    );

    println!(
        "Complete AtomRefRange structure: {}",
        serde_json::to_string_pretty(&serialized_value).unwrap()
    );
}

/// Test range field with proper atom_ref_range initialization
#[test]
fn test_range_field_with_proper_atom_ref_range() {
    let permission_policy = PermissionsPolicy::default();
    let payment_config = FieldPaymentConfig::default();
    let field_mappers = HashMap::new();
    let source_pub_key = "test_key".to_string();

    let mut range_field = RangeField::new_with_range(
        permission_policy,
        payment_config,
        field_mappers,
        source_pub_key,
    );

    // Verify atom_ref_range is properly initialized
    assert!(range_field.atom_ref_range().is_some());

    // Add some data
    if let Some(atom_ref_range) = range_field.atom_ref_range_mut() {
        atom_ref_range.set_atom_uuid("2024-01-01:daily".to_string(), "atom_uuid_1".to_string());
        atom_ref_range.set_atom_uuid(
            "2024-01-01:hourly:00".to_string(),
            "atom_uuid_2".to_string(),
        );
    }

    // Test serialization of the entire range field
    let serialized = serde_json::to_value(&range_field).unwrap();
    println!(
        "Serialized RangeField: {}",
        serde_json::to_string_pretty(&serialized).unwrap()
    );

    // Test deserialization
    let deserialized: RangeField = serde_json::from_value(serialized).unwrap();

    // Verify the atom_ref_range is preserved
    assert!(deserialized.atom_ref_range().is_some());
    if let Some(atom_ref_range) = deserialized.atom_ref_range() {
        assert_eq!(
            atom_ref_range.get_atom_uuid("2024-01-01:daily"),
            Some(&"atom_uuid_1".to_string())
        );
        assert_eq!(
            atom_ref_range.get_atom_uuid("2024-01-01:hourly:00"),
            Some(&"atom_uuid_2".to_string())
        );
    }
}

/// Test field manager with range field to verify proper atom creation and retrieval
#[test]
fn test_field_manager_range_field_save_fetch_cycle() {
    let temp_dir = tempdir().unwrap();
    let db = sled::open(temp_dir.path().join("test_db")).unwrap();
    let db_ops = DbOperations::new(db).unwrap();
    let atom_manager = AtomManager::new(db_ops);
    let mut field_manager = FieldManager::new(atom_manager);

    // Create a schema with a range field
    let mut schema = Schema::new("EventAnalytics".to_string());

    let permission_policy = PermissionsPolicy::default();
    let payment_config = FieldPaymentConfig::default();
    let field_mappers = HashMap::new();

    let range_field = RangeField::new(permission_policy, payment_config, field_mappers);
    schema.fields.insert(
        "metrics_by_timeframe".to_string(),
        FieldVariant::Range(range_field),
    );

    // Test setting field value (save operation)
    let test_data = json!({
        "2024-01-01:daily": "1250",
        "2024-01-01:hourly:00": "45",
        "2024-01-01:hourly:01": "52",
        "2024-01-02:daily": "1180"
    });

    let source_pub_key = "test_source_key".to_string();
    let set_result = field_manager.set_field_value(
        &mut schema,
        "metrics_by_timeframe",
        test_data.clone(),
        source_pub_key,
    );
    println!("Set field result: {:?}", set_result);
    assert!(set_result.is_ok(), "Setting field value should succeed");

    // Get the atom UUID that was created
    let atom_uuid = set_result.unwrap();
    println!("Created atom UUID: {}", atom_uuid);

    // Test getting field value WITHOUT setting ref_atom_uuid (this should return Null)
    let get_result_without_ref = field_manager.get_field_value(&schema, "metrics_by_timeframe");
    println!(
        "Get field result WITHOUT ref_atom_uuid: {:?}",
        get_result_without_ref
    );
    assert_eq!(
        get_result_without_ref.unwrap(),
        Value::Null,
        "Should return Null when ref_atom_uuid is not set"
    );

    // Update the schema to include the ref_atom_uuid (simulating what the mutation logic does)
    if let Some(FieldVariant::Range(ref mut range_field)) =
        schema.fields.get_mut("metrics_by_timeframe")
    {
        println!("Setting ref_atom_uuid to: {}", atom_uuid);
        range_field.inner.ref_atom_uuid = Some(atom_uuid.clone());
        println!(
            "ref_atom_uuid after setting: {:?}",
            range_field.inner.ref_atom_uuid
        );
    }

    // Verify the ref_atom_uuid was set correctly
    if let Some(FieldVariant::Range(range_field)) = schema.fields.get("metrics_by_timeframe") {
        println!(
            "Verified ref_atom_uuid: {:?}",
            range_field.inner.ref_atom_uuid
        );
    }

    // Test getting field value WITH ref_atom_uuid set (this should return the data)
    let get_result = field_manager.get_field_value(&schema, "metrics_by_timeframe");
    println!("Get field result WITH ref_atom_uuid: {:?}", get_result);
    assert!(get_result.is_ok(), "Getting field value should succeed");

    let retrieved_data = get_result.unwrap();

    println!("Expected data: {:?}", test_data);
    println!("Retrieved data: {:?}", retrieved_data);

    // Now that we've fixed the retrieval logic, this should work correctly
    println!("✅ TESTING FIXED BEHAVIOR: Range field save/fetch cycle");
    println!("   - Atom creation: SUCCESS (UUID: {})", atom_uuid);
    println!("   - ref_atom_uuid setting: SUCCESS");
    println!("   - Data retrieval: Testing...");

    // Test that the retrieved data matches the original data
    assert_eq!(
        retrieved_data, test_data,
        "Retrieved data should match original data after fix"
    );

    println!("   - Data retrieval: SUCCESS! ✅");
    println!("🎉 Range field save/fetch cycle is now working correctly!");
}

/// Test that verifies range field with proper atom_ref_range initialization and persistence
#[test]
fn test_range_field_with_atom_ref_range_persistence() {
    let permission_policy = PermissionsPolicy::default();
    let payment_config = FieldPaymentConfig::default();
    let field_mappers = HashMap::new();
    let source_pub_key = "test_key".to_string();

    let mut range_field = RangeField::new_with_range(
        permission_policy,
        payment_config,
        field_mappers,
        source_pub_key,
    );

    // Verify atom_ref_range is properly initialized
    assert!(range_field.atom_ref_range().is_some());

    // Add some data to the atom_ref_range
    if let Some(atom_ref_range) = range_field.atom_ref_range_mut() {
        atom_ref_range.set_atom_uuid("2024-01-01:daily".to_string(), "atom_uuid_1".to_string());
        atom_ref_range.set_atom_uuid(
            "2024-01-01:hourly:00".to_string(),
            "atom_uuid_2".to_string(),
        );
        atom_ref_range.set_atom_uuid("2024-01-02:daily".to_string(), "atom_uuid_3".to_string());
    }

    // Test serialization of the entire range field
    let serialized = serde_json::to_value(&range_field).unwrap();
    println!(
        "Serialized RangeField: {}",
        serde_json::to_string_pretty(&serialized).unwrap()
    );

    // Verify the atom_ref_range is included in serialization
    assert!(
        serialized.get("atom_ref_range").is_some(),
        "atom_ref_range should be serialized"
    );

    // Test deserialization
    let deserialized: RangeField = serde_json::from_value(serialized).unwrap();

    // Verify the atom_ref_range is preserved
    assert!(deserialized.atom_ref_range().is_some());
    if let Some(atom_ref_range) = deserialized.atom_ref_range() {
        assert_eq!(
            atom_ref_range.get_atom_uuid("2024-01-01:daily"),
            Some(&"atom_uuid_1".to_string())
        );
        assert_eq!(
            atom_ref_range.get_atom_uuid("2024-01-01:hourly:00"),
            Some(&"atom_uuid_2".to_string())
        );
        assert_eq!(
            atom_ref_range.get_atom_uuid("2024-01-02:daily"),
            Some(&"atom_uuid_3".to_string())
        );
    }
}

/// Test that simulates the EventAnalytics scenario from the error message
#[test]
fn test_event_analytics_range_field_scenario() {
    // Create a schema similar to EventAnalytics
    let mut schema = Schema::new("EventAnalytics".to_string());

    // Add the metrics_by_timeframe range field
    let permission_policy = PermissionsPolicy::default();
    let payment_config = FieldPaymentConfig::default();
    let field_mappers = HashMap::new();
    let source_pub_key = "test_key".to_string();

    let range_field = RangeField::new_with_range(
        permission_policy,
        payment_config,
        field_mappers,
        source_pub_key,
    );

    schema.fields.insert(
        "metrics_by_timeframe".to_string(),
        FieldVariant::Range(range_field),
    );

    // Test that the field can be accessed
    let field_def = schema.fields.get("metrics_by_timeframe").unwrap();
    match field_def {
        FieldVariant::Range(range_field) => {
            assert!(range_field.atom_ref_range().is_some());

            // Test serialization to ensure all fields are present
            let serialized = serde_json::to_value(range_field).unwrap();
            println!(
                "EventAnalytics range field serialized: {}",
                serde_json::to_string_pretty(&serialized).unwrap()
            );

            // Verify atom_ref_range structure
            if let Some(atom_ref_range_value) = serialized.get("atom_ref_range") {
                assert!(
                    atom_ref_range_value.get("uuid").is_some(),
                    "atom_ref_range should have uuid"
                );
                assert!(
                    atom_ref_range_value.get("atom_uuids").is_some(),
                    "atom_ref_range should have atom_uuids"
                );
                assert!(
                    atom_ref_range_value.get("updated_at").is_some(),
                    "atom_ref_range should have updated_at"
                );
                assert!(
                    atom_ref_range_value.get("status").is_some(),
                    "atom_ref_range should have status"
                );
                assert!(
                    atom_ref_range_value.get("update_history").is_some(),
                    "atom_ref_range should have update_history"
                );
            }
        }
        _ => panic!("Expected Range field"),
    }
}

/// Test that reproduces potential data corruption scenarios
#[test]
fn test_range_field_data_corruption_scenarios() {
    // Test scenario 1: AtomRefRange with missing fields
    let incomplete_atom_ref_range = json!({
        "atom_uuids": {
            "2024-01-01:daily": "1250"
        },
        "updated_at": "2024-01-01T00:00:00Z",
        "status": "Active"
        // Missing uuid and update_history
    });

    let result = serde_json::from_value::<AtomRefRange>(incomplete_atom_ref_range);
    assert!(result.is_err(), "Should fail with missing fields");

    // Test scenario 2: RangeField with corrupted atom_ref_range
    let corrupted_range_field = json!({
        "inner": {
            "permission_policy": {
                "read_policy": {"NoRequirement": null},
                "write_policy": {"NoRequirement": null},
                "explicit_read_policy": null,
                "explicit_write_policy": null
            },
            "payment_config": {
                "base_multiplier": 1.0,
                "trust_distance_scaling": {"None": null},
                "min_payment": null
            },
            "field_mappers": {},
            "ref_atom_uuid": null
        },
        "atom_ref_range": {
            "atom_uuids": {"key": "value"},
            "updated_at": "2024-01-01T00:00:00Z",
            "status": "Active"
            // Missing uuid and update_history
        }
    });

    let result = serde_json::from_value::<RangeField>(corrupted_range_field);
    assert!(result.is_err(), "Should fail with corrupted atom_ref_range");

    println!("Successfully detected data corruption scenarios");
}
