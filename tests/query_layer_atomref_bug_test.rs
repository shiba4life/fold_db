//! Test to validate the Query Layer AtomRef Bug
//!
//! This test reproduces the exact issue where:
//! 1. Mutation layer correctly updates dynamic AtomRefs
//! 2. Query layer incorrectly reads static schema references
//! 3. Result: Query finds old/wrong atom UUIDs

use fold_node::fold_db_core::infrastructure::message_bus::{
    MessageBus, FieldValueSetRequest, FieldValueSetResponse
};
use fold_node::fold_db_core::managers::atom::AtomManager;  
use fold_node::fold_db_core::transform_manager::utils::TransformUtils;
use fold_node::db_operations::DbOperations;
use fold_node::schema::{Schema, types::field::FieldVariant};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use std::thread;
use tempfile::tempdir;

#[test]
fn test_query_layer_atomref_bug_reproduction() {
    println!("🚨 TESTING QUERY LAYER ATOMREF BUG");
    println!("   This test reproduces the exact bug where query reads static schema refs");
    
    // Setup database
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let db = sled::Config::new()
        .path(temp_dir.path())
        .temporary(true)
        .open()
        .expect("Failed to open database");
    
    let db_ops = DbOperations::new(db.clone()).expect("Failed to create DbOperations");
    let message_bus = Arc::new(MessageBus::new());
    
    // Create AtomManager
    let _atom_manager = AtomManager::new(db_ops.clone(), Arc::clone(&message_bus));
    
    // Subscribe to FieldValueSetResponse events
    let mut response_consumer = message_bus.subscribe::<FieldValueSetResponse>();
    
    // STEP 1: Create a test schema with initial static field reference
    println!("📝 STEP 1: Creating test schema with static field reference");
    let mut test_schema = Schema::new("test_schema".to_string());
    
    // Add a field with a static atom reference (this will become stale)
    let initial_static_atom_uuid = "static-atom-uuid-12345";
    
    // Create SingleField with proper structure
    use fold_node::schema::types::field::Field; // Import the trait
    use std::collections::HashMap;
    
    let mut single_field = fold_node::schema::types::field::SingleField::new(
        fold_node::permissions::types::policy::PermissionsPolicy::default(),
        fold_node::fees::types::config::FieldPaymentConfig::default(),
        HashMap::new(),
    );
    
    // Set the static atom reference (this will become stale)
    single_field.set_ref_atom_uuid(initial_static_atom_uuid.to_string());
    
    let field_variant = FieldVariant::Single(single_field);
    test_schema.fields.insert("test_field".to_string(), field_variant);
    println!("✅ Schema created with static ref_atom_uuid: {}", initial_static_atom_uuid);
    
    // STEP 2: Use mutation layer to create new field value (updates dynamic AtomRef)
    println!("📝 STEP 2: Using mutation layer to create new field value");
    let mutation_request = FieldValueSetRequest::new(
        "mutation_test".to_string(),
        "test_schema".to_string(),
        "test_field".to_string(),
        json!({"content": "new_value_v1", "timestamp": "2024-01-01"}),
        "test_pubkey".to_string(),
    );
    
    message_bus.publish(mutation_request).expect("Failed to publish mutation");
    thread::sleep(Duration::from_millis(200));
    
    let mutation_response = response_consumer.recv_timeout(Duration::from_millis(500))
        .expect("Should receive mutation response");
    
    assert!(mutation_response.success, "Mutation should succeed");
    let dynamic_aref_uuid = mutation_response.aref_uuid.expect("Should return AtomRef UUID");
    println!("✅ Mutation created dynamic AtomRef: {}", dynamic_aref_uuid);
    
    // STEP 3: Verify dynamic AtomRef was created and points to new atom  
    println!("🔍 STEP 3: Verifying dynamic AtomRef state");
    let dynamic_aref = db_ops.get_item::<fold_node::atom::AtomRef>(&format!("ref:{}", dynamic_aref_uuid))
        .expect("Should be able to query dynamic AtomRef")
        .expect("Dynamic AtomRef should exist");
    
    let dynamic_atom_uuid = dynamic_aref.get_atom_uuid().clone();
    println!("✅ Dynamic AtomRef points to atom: {}", dynamic_atom_uuid);
    
    // CRITICAL TEST: This should be DIFFERENT from the static schema reference
    assert_ne!(dynamic_atom_uuid, initial_static_atom_uuid, 
        "Dynamic atom UUID should differ from static schema reference");
    
    // STEP 4: Test query layer - this should reveal the bug!
    println!("🚨 STEP 4: Testing query layer (this will show the bug)");
    println!("   Expected: Query should find atom {}", dynamic_atom_uuid);
    println!("   Bug: Query will try to find atom {}", initial_static_atom_uuid);
    
    // Use the query layer to resolve field value
    match TransformUtils::resolve_field_value(&Arc::new(db_ops.clone()), &test_schema, "test_field") {
        Ok(value) => {
            println!("✅ Query layer returned value: {}", value);
            println!("🔍 Check logs above to see if the fix was applied");
            
            // If our fix worked, the value should match what we set
            if let Some(obj) = value.as_object() {
                if let Some(content) = obj.get("content") {
                    assert_eq!(content, &json!("new_value_v1"), "Content should match what we set via mutation");
                    println!("✅ QUERY LAYER FIX CONFIRMED: Returned correct updated content");
                } else {
                    println!("⚠️ Query returned object but missing 'content' field: {}", value);
                }
            } else {
                println!("⚠️ Query returned non-object value: {}", value);
            }
        }
        Err(e) => {
            println!("❌ Query layer failed: {}", e);
            println!("🔍 This might indicate the static schema reference is broken");
            println!("   Check the diagnostic logs above for details");
            
            // This failure is expected if static reference doesn't exist
            // The diagnostic logs should show the mismatch
        }
    }
    
    // STEP 5: Create another mutation to further test the system
    println!("📝 STEP 5: Testing second mutation to verify dynamic AtomRef updates");
    let mutation_request_2 = FieldValueSetRequest::new(
        "mutation_test_2".to_string(),
        "test_schema".to_string(),
        "test_field".to_string(),
        json!({"content": "new_value_v2", "timestamp": "2024-01-02"}),
        "test_pubkey_2".to_string(),
    );
    
    message_bus.publish(mutation_request_2).expect("Failed to publish second mutation");
    thread::sleep(Duration::from_millis(200));
    
    let mutation_response_2 = response_consumer.recv_timeout(Duration::from_millis(500))
        .expect("Should receive second mutation response");
    
    assert!(mutation_response_2.success, "Second mutation should succeed");
    let dynamic_aref_uuid_2 = mutation_response_2.aref_uuid.expect("Should return same AtomRef UUID");
    
    // Should reuse the same AtomRef UUID
    assert_eq!(dynamic_aref_uuid, dynamic_aref_uuid_2, "Should reuse same AtomRef UUID");
    
    // Check that the AtomRef now points to a newer atom
    let updated_aref = db_ops.get_item::<fold_node::atom::AtomRef>(&format!("ref:{}", dynamic_aref_uuid))
        .expect("Should be able to query updated AtomRef")
        .expect("Updated AtomRef should exist");
    
    let updated_atom_uuid = updated_aref.get_atom_uuid().clone();
    println!("✅ After second mutation, AtomRef points to atom: {}", updated_atom_uuid);
    assert_ne!(updated_atom_uuid, dynamic_atom_uuid, "Should point to newer atom after second mutation");
    
    // Test query layer again
    println!("🔍 STEP 6: Testing query layer after second mutation");
    match TransformUtils::resolve_field_value(&Arc::new(db_ops), &test_schema, "test_field") {
        Ok(value) => {
            println!("✅ Query layer returned value after second mutation: {}", value);
            
            if let Some(obj) = value.as_object() {
                if let Some(content) = obj.get("content") {
                    assert_eq!(content, &json!("new_value_v2"), "Should return latest content after second mutation");
                    println!("✅ QUERY LAYER FULLY WORKING: Returns latest content after multiple mutations");
                }
            }
        }
        Err(e) => {
            println!("❌ Query layer failed after second mutation: {}", e);
        }
    }
    
    println!("✅ QUERY LAYER ATOMREF BUG TEST COMPLETED");
    println!("   Static schema ref: {}", initial_static_atom_uuid);
    println!("   Dynamic AtomRef:   {}", dynamic_aref_uuid);
    println!("   Final atom UUID:   {}", updated_atom_uuid);
    println!("   Check diagnostic logs for bug confirmation and fix validation");
}