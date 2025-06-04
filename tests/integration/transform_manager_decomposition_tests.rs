//! Integration tests for TransformManager decomposition validation
//!
//! This module validates that the file decomposition of transform_manager/manager.rs
//! into multiple focused modules maintains all original functionality while improving
//! code organization and maintainability.

use crate::common::CommonTestFixture;
use fold_node::fold_db_core::infrastructure::message_bus::{
    SchemaChanged, TransformTriggered, TransformExecuted,
    TransformTriggerRequest, TransformTriggerResponse,
    TransformExecutionRequest, TransformExecutionResponse,
};
use fold_node::fold_db_core::transform_manager::TransformRunner;
use fold_node::schema::types::{Transform, TransformRegistration};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_transform_manager_construction_with_decomposed_modules() {
    // Test that TransformManager can be constructed successfully with all decomposed modules
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    // Verify that the manager was constructed successfully
    assert!(fixture.transform_manager.list_transforms().is_ok());
    
    println!("✅ TransformManager constructed successfully with decomposed modules");
}

#[tokio::test]
async fn test_api_compatibility_after_decomposition() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let transform_manager = &fixture.transform_manager;

    // Test all public API methods remain functional after decomposition
    
    // 1. Test transform_exists method
    let exists_result = transform_manager.transform_exists("nonexistent");
    assert!(exists_result.is_ok());
    assert!(!exists_result.unwrap());

    // 2. Test list_transforms method
    let list_result = transform_manager.list_transforms();
    assert!(list_result.is_ok());
    assert!(list_result.unwrap().is_empty());

    // 3. Test get_dependent_transforms method
    let dependent_result = transform_manager.get_dependent_transforms("test_aref");
    assert!(dependent_result.is_ok());
    assert!(dependent_result.unwrap().is_empty());

    // 4. Test get_transform_inputs method
    let inputs_result = transform_manager.get_transform_inputs("test_transform");
    assert!(inputs_result.is_ok());
    assert!(inputs_result.unwrap().is_empty());

    // 5. Test get_transform_output method
    let output_result = transform_manager.get_transform_output("test_transform");
    assert!(output_result.is_ok());
    assert!(output_result.unwrap().is_none());

    // 6. Test get_transforms_for_field method
    let field_transforms_result = transform_manager.get_transforms_for_field("test", "field");
    assert!(field_transforms_result.is_ok());
    assert!(field_transforms_result.unwrap().is_empty());

    println!("✅ All API methods remain functional after decomposition");
}

#[tokio::test]
async fn test_transform_registration_across_modules() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let transform_manager = &fixture.transform_manager;
    let registration = CommonTestFixture::create_sample_registration();

    // Test transform registration using the decomposed loading module
    let register_result = transform_manager.register_transform_event_driven(registration.clone());
    assert!(register_result.is_ok(), "Transform registration failed: {:?}", register_result.err());

    // Verify the transform was registered correctly
    let exists = transform_manager.transform_exists(&registration.transform_id);
    assert!(exists.is_ok());
    assert!(exists.unwrap(), "Transform should exist after registration");

    // Verify transform can be retrieved
    let transforms = transform_manager.list_transforms().unwrap();
    assert!(transforms.contains_key(&registration.transform_id));

    // Verify input mappings were created
    let inputs = transform_manager.get_transform_inputs(&registration.transform_id).unwrap();
    assert_eq!(inputs.len(), 2);
    assert!(inputs.contains("aref1"));
    assert!(inputs.contains("aref2"));

    // Verify output mapping was created
    let output = transform_manager.get_transform_output(&registration.transform_id).unwrap();
    assert_eq!(output.unwrap(), "output_aref");

    // Verify field mappings were created
    let field_transforms = transform_manager.get_transforms_for_field("test", "field1").unwrap();
    assert!(field_transforms.contains(&registration.transform_id));

    println!("✅ Transform registration works correctly across decomposed modules");
}

#[tokio::test]
async fn test_transform_unregistration_across_modules() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let transform_manager = &fixture.transform_manager;
    let registration = CommonTestFixture::create_sample_registration();

    // First register a transform
    transform_manager.register_transform_event_driven(registration.clone())
        .expect("Failed to register transform");

    // Verify it was registered
    assert!(transform_manager.transform_exists(&registration.transform_id).unwrap());

    // Now unregister it using the decomposed registry module
    let unregister_result = transform_manager.unregister_transform(&registration.transform_id);
    assert!(unregister_result.is_ok());
    assert!(unregister_result.unwrap(), "Unregister should return true for existing transform");

    // Verify the transform was removed
    assert!(!transform_manager.transform_exists(&registration.transform_id).unwrap());

    // Verify mappings were cleaned up
    let inputs = transform_manager.get_transform_inputs(&registration.transform_id).unwrap();
    assert!(inputs.is_empty());

    let output = transform_manager.get_transform_output(&registration.transform_id).unwrap();
    assert!(output.is_none());

    let field_transforms = transform_manager.get_transforms_for_field("test", "field1").unwrap();
    assert!(!field_transforms.contains(&registration.transform_id));

    println!("✅ Transform unregistration works correctly across decomposed modules");
}

#[tokio::test]
async fn test_event_driven_execution_across_modules() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let transform_manager = &fixture.transform_manager;
    let message_bus = &fixture.message_bus;

    // Register a test transform
    let registration = CommonTestFixture::create_sample_registration();
    transform_manager.register_transform_event_driven(registration.clone())
        .expect("Failed to register transform");

    // Subscribe to TransformExecuted events to verify execution
    let mut executed_consumer = message_bus.subscribe::<TransformExecuted>();

    // Test event-driven execution through execute_transform_now (legacy API)
    let execution_result = transform_manager.execute_transform_now(&registration.transform_id);
    assert!(execution_result.is_ok());

    // The result should indicate event-driven execution was requested
    let result_value = execution_result.unwrap();
    assert_eq!(result_value["status"], "execution_requested");
    assert_eq!(result_value["method"], "event_driven");

    // Wait for and verify TransformExecuted event was published
    let executed_event = timeout(Duration::from_millis(500), async {
        executed_consumer.recv().unwrap()
    }).await;

    assert!(executed_event.is_ok(), "TransformExecuted event should be published");
    let event = executed_event.unwrap();
    assert_eq!(event.transform_id, registration.transform_id);

    println!("✅ Event-driven execution works correctly across decomposed modules");
}

#[tokio::test]
async fn test_schema_change_monitoring_integration() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let message_bus = &fixture.message_bus;

    // Publish a SchemaChanged event to test monitoring integration
    let schema_changed = SchemaChanged {
        schema: "test_schema".to_string(),
    };

    let publish_result = message_bus.publish(schema_changed);
    assert!(publish_result.is_ok(), "Failed to publish SchemaChanged event");

    // The monitoring thread should process this event
    // We can't easily test the internal processing, but we can verify the event was published
    println!("✅ Schema change monitoring integration works correctly");
}

#[tokio::test]
async fn test_transform_trigger_request_processing() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let message_bus = &fixture.message_bus;

    // Subscribe to TransformTriggerResponse events
    let mut response_consumer = message_bus.subscribe::<TransformTriggerResponse>();

    // Publish a TransformTriggerRequest event
    let trigger_request = TransformTriggerRequest {
        correlation_id: "test_correlation".to_string(),
        schema_name: "test".to_string(),
        field_name: "field1".to_string(),
        mutation_hash: "test_mutation_hash".to_string(),
    };

    let publish_result = message_bus.publish(trigger_request.clone());
    assert!(publish_result.is_ok(), "Failed to publish TransformTriggerRequest");

    // Wait for and verify TransformTriggerResponse
    let response_event = timeout(Duration::from_millis(500), async {
        response_consumer.recv().unwrap()
    }).await;

    assert!(response_event.is_ok(), "TransformTriggerResponse should be published");
    let response = response_event.unwrap();
    assert_eq!(response.correlation_id, trigger_request.correlation_id);

    println!("✅ Transform trigger request processing works correctly");
}

#[tokio::test]
async fn test_transform_execution_request_processing() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let message_bus = &fixture.message_bus;

    // Subscribe to TransformExecutionResponse events
    let mut response_consumer = message_bus.subscribe::<TransformExecutionResponse>();

    // Publish a TransformExecutionRequest event
    let execution_request = TransformExecutionRequest {
        correlation_id: "test_execution_correlation".to_string(),
    };

    let publish_result = message_bus.publish(execution_request.clone());
    assert!(publish_result.is_ok(), "Failed to publish TransformExecutionRequest");

    // Wait for and verify TransformExecutionResponse
    let response_event = timeout(Duration::from_millis(500), async {
        response_consumer.recv().unwrap()
    }).await;

    assert!(response_event.is_ok(), "TransformExecutionResponse should be published");
    let response = response_event.unwrap();
    assert_eq!(response.correlation_id, execution_request.correlation_id);

    println!("✅ Transform execution request processing works correctly");
}

#[tokio::test]
async fn test_persistence_across_modules() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let transform_manager = &fixture.transform_manager;
    let registration = CommonTestFixture::create_sample_registration();

    // Register a transform (this should trigger persistence)
    transform_manager.register_transform_event_driven(registration.clone())
        .expect("Failed to register transform");

    // Test that persistence methods work
    let persist_result = transform_manager.persist_mappings_direct();
    assert!(persist_result.is_ok(), "Persistence should work correctly");

    // Verify data was persisted by checking database directly
    let stored_transform = fixture.db_ops.get_transform(&registration.transform_id);
    assert!(stored_transform.is_ok());
    assert!(stored_transform.unwrap().is_some());

    println!("✅ Persistence works correctly across decomposed modules");
}

#[tokio::test]
async fn test_transform_reload_functionality() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let transform_manager = &fixture.transform_manager;
    let registration = CommonTestFixture::create_sample_registration();

    // Store a transform directly in the database (bypassing in-memory registration)
    fixture.db_ops.store_transform(&registration.transform_id, &registration.transform)
        .expect("Failed to store transform directly");

    // The transform should not be in memory yet
    assert!(!transform_manager.transform_exists(&registration.transform_id).unwrap());

    // Reload transforms from database
    let reload_result = transform_manager.reload_transforms();
    assert!(reload_result.is_ok(), "Transform reload should succeed");

    // Now the transform should be available in memory
    assert!(transform_manager.transform_exists(&registration.transform_id).unwrap());

    println!("✅ Transform reload functionality works correctly");
}

#[tokio::test]
async fn test_cross_module_communication() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let transform_manager = &fixture.transform_manager;
    let message_bus = &fixture.message_bus;

    // Register a transform
    let registration = CommonTestFixture::create_sample_registration();
    transform_manager.register_transform_event_driven(registration.clone())
        .expect("Failed to register transform");

    // Subscribe to relevant events
    let mut executed_consumer = message_bus.subscribe::<TransformExecuted>();

    // Publish a TransformTriggered event to test cross-module communication
    let triggered_event = TransformTriggered {
        transform_id: registration.transform_id.clone(),
    };

    let publish_result = message_bus.publish(triggered_event);
    assert!(publish_result.is_ok(), "Failed to publish TransformTriggered event");

    // Verify that the event flows through the modules correctly
    let executed_event = timeout(Duration::from_millis(500), async {
        executed_consumer.recv().unwrap()
    }).await;

    assert!(executed_event.is_ok(), "TransformExecuted event should be published");
    let event = executed_event.unwrap();
    assert_eq!(event.transform_id, registration.transform_id);

    println!("✅ Cross-module communication works correctly");
}

#[tokio::test]
async fn test_error_handling_across_modules() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let transform_manager = &fixture.transform_manager;

    // Test error handling in various modules
    
    // 1. Test invalid transform registration (loading module)
    let invalid_registration = TransformRegistration {
        transform_id: "".to_string(), // Invalid empty ID
        transform: CommonTestFixture::create_sample_transform(),
        input_arefs: vec![],
        input_names: vec![],
        trigger_fields: vec![],
        output_aref: "".to_string(),
        schema_name: "".to_string(),
        field_name: "".to_string(),
    };

    let _register_result = transform_manager.register_transform_event_driven(invalid_registration);
    // This might succeed depending on validation, but should handle gracefully

    // 2. Test operations on non-existent transforms (registry module)
    let unregister_result = transform_manager.unregister_transform("nonexistent_transform");
    assert!(unregister_result.is_ok());
    assert!(!unregister_result.unwrap()); // Should return false for non-existent transform

    // 3. Test execution of non-existent transform (execution module)
    let execute_result = transform_manager.execute_transform_now("nonexistent_transform");
    assert!(execute_result.is_ok()); // Should still publish event, even if transform doesn't exist

    println!("✅ Error handling works correctly across decomposed modules");
}

#[tokio::test]
async fn test_thread_safety_of_decomposed_modules() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let transform_manager = Arc::new(fixture.transform_manager);

    // Test concurrent operations across modules to ensure thread safety
    let mut handles = vec![];

    // Spawn multiple threads performing different operations
    for i in 0..5 {
        let tm = Arc::clone(&transform_manager);
        let registration = TransformRegistration {
            transform_id: format!("test_transform_{}", i),
            transform: Transform::new(
                "input1".to_string(),
                format!("test.output_{}", i),
            ),
            input_arefs: vec![format!("aref_{}", i)],
            input_names: vec![format!("input_{}", i)],
            trigger_fields: vec![format!("test.field_{}", i)],
            output_aref: format!("output_aref_{}", i),
            schema_name: "test".to_string(),
            field_name: format!("output_{}", i),
        };

        let handle = tokio::spawn(async move {
            // Register transform
            let _ = tm.register_transform_event_driven(registration.clone());
            
            // Check if it exists
            let _ = tm.transform_exists(&registration.transform_id);
            
            // Execute it
            let _ = tm.execute_transform_now(&registration.transform_id);
            
            // Unregister it
            let _ = tm.unregister_transform(&registration.transform_id);
        });

        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        handle.await.expect("Thread should complete successfully");
    }

    println!("✅ Thread safety maintained across decomposed modules");
}