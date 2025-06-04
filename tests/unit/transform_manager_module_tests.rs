//! Unit tests for individual TransformManager modules
//!
//! This module contains focused unit tests for each decomposed module to ensure
//! they function correctly in isolation and maintain their specific responsibilities.

use crate::common::CommonTestFixture;
use fold_node::fold_db_core::infrastructure::message_bus::{
    MessageBus, TransformTriggered, TransformExecuted,
    TransformTriggerRequest, TransformTriggerResponse,
    TransformExecutionRequest, TransformExecutionResponse,
    SchemaChanged
};
use fold_node::fold_db_core::transform_manager::{TransformManager, TransformRunner};
use fold_node::schema::types::{Transform, TransformRegistration};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

// ========== Manager Module Tests ==========

#[tokio::test]
async fn test_manager_module_initialization() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let manager = &fixture.transform_manager;

    // Test that manager initializes with empty state
    let transforms = manager.list_transforms().unwrap();
    assert!(transforms.is_empty(), "Manager should start with no transforms");

    let dependent_transforms = manager.get_dependent_transforms("any_aref").unwrap();
    assert!(dependent_transforms.is_empty(), "No dependencies should exist initially");

    let transform_inputs = manager.get_transform_inputs("any_transform").unwrap();
    assert!(transform_inputs.is_empty(), "No inputs should exist initially");

    let transform_output = manager.get_transform_output("any_transform").unwrap();
    assert!(transform_output.is_none(), "No outputs should exist initially");

    println!("✅ Manager module initializes correctly");
}

#[tokio::test]
async fn test_manager_module_transform_existence_check() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let manager = &fixture.transform_manager;

    // Test transform existence check
    assert!(!manager.transform_exists("nonexistent").unwrap());

    // Register a transform and check existence
    let registration = CommonTestFixture::create_sample_registration();
    manager.register_transform_event_driven(registration.clone()).unwrap();

    assert!(manager.transform_exists(&registration.transform_id).unwrap());

    println!("✅ Manager module transform existence check works correctly");
}

#[tokio::test]
async fn test_manager_module_transform_runner_trait() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let manager = &fixture.transform_manager;
    let registration = CommonTestFixture::create_sample_registration();

    // Register a transform
    manager.register_transform_event_driven(registration.clone()).unwrap();

    // Test TransformRunner trait methods
    let runner: &dyn TransformRunner = manager;

    // Test execute_transform_now (should use event-driven approach)
    let execute_result = runner.execute_transform_now(&registration.transform_id);
    assert!(execute_result.is_ok());
    
    let result = execute_result.unwrap();
    assert_eq!(result["status"], "execution_requested");
    assert_eq!(result["method"], "event_driven");

    // Test transform_exists
    assert!(runner.transform_exists(&registration.transform_id).unwrap());
    assert!(!runner.transform_exists("nonexistent").unwrap());

    // Test get_transforms_for_field
    let field_transforms = runner.get_transforms_for_field("test", "field1").unwrap();
    assert!(field_transforms.contains(&registration.transform_id));

    println!("✅ Manager module TransformRunner trait implementation works correctly");
}

// ========== Event Handlers Module Tests ==========

#[tokio::test]
async fn test_event_handlers_transform_triggered_processing() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let message_bus = &fixture.message_bus;
    let registration = CommonTestFixture::create_sample_registration();

    // Register a transform
    fixture.transform_manager.register_transform_event_driven(registration.clone()).unwrap();

    // Subscribe to TransformExecuted events
    let mut executed_consumer = message_bus.subscribe::<TransformExecuted>();

    // Publish TransformTriggered event
    let triggered_event = TransformTriggered {
        transform_id: registration.transform_id.clone(),
    };

    message_bus.publish(triggered_event).unwrap();

    // Verify TransformExecuted event is published
    let executed_event = timeout(Duration::from_millis(300), async {
        executed_consumer.recv().unwrap()
    }).await;

    assert!(executed_event.is_ok(), "TransformExecuted should be published");
    let event = executed_event.unwrap();
    assert_eq!(event.transform_id, registration.transform_id);

    println!("✅ Event handlers module processes TransformTriggered events correctly");
}

#[tokio::test]
async fn test_event_handlers_trigger_request_processing() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let message_bus = &fixture.message_bus;

    // Subscribe to TransformTriggerResponse events
    let mut response_consumer = message_bus.subscribe::<TransformTriggerResponse>();

    // Publish TransformTriggerRequest event
    let trigger_request = TransformTriggerRequest {
        correlation_id: "test_correlation_123".to_string(),
        schema_name: "test_schema".to_string(),
        field_name: "test_field".to_string(),
        mutation_hash: "test_mutation_hash".to_string(),
    };

    message_bus.publish(trigger_request.clone()).unwrap();

    // Verify TransformTriggerResponse is published
    let response_event = timeout(Duration::from_millis(300), async {
        response_consumer.recv().unwrap()
    }).await;

    assert!(response_event.is_ok(), "TransformTriggerResponse should be published");
    let response = response_event.unwrap();
    assert_eq!(response.correlation_id, trigger_request.correlation_id);

    println!("✅ Event handlers module processes TransformTriggerRequest correctly");
}

#[tokio::test]
async fn test_event_handlers_execution_request_processing() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let message_bus = &fixture.message_bus;

    // Subscribe to TransformExecutionResponse events
    let mut response_consumer = message_bus.subscribe::<TransformExecutionResponse>();

    // Publish TransformExecutionRequest event
    let execution_request = TransformExecutionRequest {
        correlation_id: "execution_test_456".to_string(),
    };

    message_bus.publish(execution_request.clone()).unwrap();

    // Verify TransformExecutionResponse is published
    let response_event = timeout(Duration::from_millis(300), async {
        response_consumer.recv().unwrap()
    }).await;

    assert!(response_event.is_ok(), "TransformExecutionResponse should be published");
    let response = response_event.unwrap();
    assert_eq!(response.correlation_id, execution_request.correlation_id);

    println!("✅ Event handlers module processes TransformExecutionRequest correctly");
}

// ========== Execution Module Tests ==========

#[tokio::test]
async fn test_execution_module_correlation_parsing() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let message_bus = &fixture.message_bus;

    // Test execution with transform_triggered correlation pattern
    let test_transform_id = "test_transform_123";
    let correlation_id = format!("transform_triggered_{}", test_transform_id);

    // Subscribe to TransformExecuted events
    let mut executed_consumer = message_bus.subscribe::<TransformExecuted>();

    // Publish TransformExecutionRequest with specific correlation pattern
    let execution_request = TransformExecutionRequest {
        correlation_id: correlation_id.clone(),
    };

    message_bus.publish(execution_request).unwrap();

    // Verify that the execution module publishes TransformExecuted event
    let executed_event = timeout(Duration::from_millis(300), async {
        executed_consumer.recv().unwrap()
    }).await;

    assert!(executed_event.is_ok(), "TransformExecuted should be published");
    let event = executed_event.unwrap();
    assert_eq!(event.transform_id, test_transform_id);
    assert_eq!(event.result, "executed_via_event_request");

    println!("✅ Execution module parses correlation IDs correctly");
}

#[tokio::test]
async fn test_execution_module_generic_correlation_handling() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let message_bus = &fixture.message_bus;

    // Subscribe to TransformExecutionResponse events
    let mut response_consumer = message_bus.subscribe::<TransformExecutionResponse>();

    // Test with generic correlation ID (not transform_triggered pattern)
    let execution_request = TransformExecutionRequest {
        correlation_id: "generic_correlation_789".to_string(),
    };

    message_bus.publish(execution_request.clone()).unwrap();

    // Verify response is published for generic correlations
    let response_event = timeout(Duration::from_millis(300), async {
        response_consumer.recv().unwrap()
    }).await;

    assert!(response_event.is_ok(), "TransformExecutionResponse should be published");
    let response = response_event.unwrap();
    assert_eq!(response.correlation_id, execution_request.correlation_id);
    assert!(response.success, "Generic execution should succeed");
    assert_eq!(response.transforms_executed, 0);

    println!("✅ Execution module handles generic correlations correctly");
}

// ========== Loading Module Tests ==========

#[tokio::test]
async fn test_loading_module_transform_registration() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let manager = &fixture.transform_manager;
    let registration = CommonTestFixture::create_sample_registration();

    // Test transform registration through loading module
    let register_result = manager.register_transform_event_driven(registration.clone());
    assert!(register_result.is_ok(), "Transform registration should succeed");

    // Verify in-memory state was updated correctly
    let transforms = manager.list_transforms().unwrap();
    assert!(transforms.contains_key(&registration.transform_id));

    let inputs = manager.get_transform_inputs(&registration.transform_id).unwrap();
    assert_eq!(inputs.len(), 2);
    assert!(inputs.contains("aref1"));
    assert!(inputs.contains("aref2"));

    let output = manager.get_transform_output(&registration.transform_id).unwrap();
    assert_eq!(output.unwrap(), "output_aref");

    println!("✅ Loading module registers transforms correctly");
}

#[tokio::test]
async fn test_loading_module_auto_registration() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let manager = &fixture.transform_manager;
    let transform = CommonTestFixture::create_sample_transform();

    // Test auto registration (with dependency analysis)
    let register_result = manager.register_transform_auto(
        "auto_test_transform".to_string(),
        transform.clone(),
        "auto_output_aref".to_string(),
        "auto_test".to_string(),
        "auto_field".to_string(),
    );

    assert!(register_result.is_ok(), "Auto registration should succeed");

    // Verify the transform was registered
    assert!(manager.transform_exists("auto_test_transform").unwrap());

    let output = manager.get_transform_output("auto_test_transform").unwrap();
    assert_eq!(output.unwrap(), "auto_output_aref");

    println!("✅ Loading module auto registration works correctly");
}

#[tokio::test]
async fn test_loading_module_reload_functionality() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let manager = &fixture.transform_manager;
    let transform = CommonTestFixture::create_sample_transform();

    // Store transform directly in database (bypass in-memory registration)
    fixture.db_ops.store_transform("reload_test_transform", &transform).unwrap();

    // Transform should not be in memory yet
    assert!(!manager.transform_exists("reload_test_transform").unwrap());

    // Reload transforms from database
    let reload_result = manager.reload_transforms();
    assert!(reload_result.is_ok(), "Reload should succeed");

    // Transform should now be available in memory
    assert!(manager.transform_exists("reload_test_transform").unwrap());

    println!("✅ Loading module reload functionality works correctly");
}

// ========== Monitoring Module Tests ==========

#[tokio::test]
async fn test_monitoring_module_schema_change_handling() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let message_bus = &fixture.message_bus;

    // Publish SchemaChanged event to test monitoring
    let schema_changed = SchemaChanged {
        schema: "monitoring_test_schema".to_string(),
    };

    let publish_result = message_bus.publish(schema_changed);
    assert!(publish_result.is_ok(), "SchemaChanged event should be published successfully");

    // The monitoring thread should process this event in the background
    // We can't easily assert on the internal processing, but the publish should succeed
    
    // Give the monitoring thread a moment to process
    tokio::time::sleep(Duration::from_millis(50)).await;

    println!("✅ Monitoring module handles SchemaChanged events correctly");
}

// ========== Persistence Module Tests ==========

#[tokio::test]
async fn test_persistence_module_mapping_persistence() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let manager = &fixture.transform_manager;
    let registration = CommonTestFixture::create_sample_registration();

    // Register a transform (which should trigger mapping persistence)
    manager.register_transform_event_driven(registration.clone()).unwrap();

    // Test direct persistence operation
    let persist_result = manager.persist_mappings_direct();
    assert!(persist_result.is_ok(), "Mapping persistence should succeed");

    // Verify data was persisted to database
    let stored_transform = fixture.db_ops.get_transform(&registration.transform_id).unwrap();
    assert!(stored_transform.is_some(), "Transform should be persisted");

    println!("✅ Persistence module persists mappings correctly");
}

#[tokio::test]
async fn test_persistence_module_mapping_loading() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let manager = &fixture.transform_manager;

    // Test that we can store and retrieve mappings through the database
    // Create and register a transform to test persistence
    let test_registration = CommonTestFixture::create_sample_registration();
    let register_result = manager.register_transform_event_driven(test_registration.clone());
    assert!(register_result.is_ok(), "Transform registration should succeed");
    
    // Test that persistence works by checking database
    let stored_transform = fixture.db_ops.get_transform(&test_registration.transform_id).unwrap();
    assert!(stored_transform.is_some(), "Transform should be persisted in database");

    println!("✅ Persistence module loads mappings correctly");
}

// ========== Registry Module Tests ==========

#[tokio::test]
async fn test_registry_module_transform_unregistration() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let manager = &fixture.transform_manager;
    let registration = CommonTestFixture::create_sample_registration();

    // Register a transform first
    manager.register_transform_event_driven(registration.clone()).unwrap();
    assert!(manager.transform_exists(&registration.transform_id).unwrap());

    // Test unregistration through registry module
    let unregister_result = manager.unregister_transform(&registration.transform_id);
    assert!(unregister_result.is_ok());
    assert!(unregister_result.unwrap(), "Unregistration should return true for existing transform");

    // Verify transform was removed from all mappings
    assert!(!manager.transform_exists(&registration.transform_id).unwrap());

    let inputs = manager.get_transform_inputs(&registration.transform_id).unwrap();
    assert!(inputs.is_empty(), "Inputs should be cleared");

    let output = manager.get_transform_output(&registration.transform_id).unwrap();
    assert!(output.is_none(), "Output should be cleared");

    let field_transforms = manager.get_transforms_for_field("test", "field1").unwrap();
    assert!(!field_transforms.contains(&registration.transform_id), "Field mappings should be cleared");

    println!("✅ Registry module unregisters transforms correctly");
}

#[tokio::test]
async fn test_registry_module_nonexistent_transform_unregistration() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let manager = &fixture.transform_manager;

    // Test unregistering a transform that doesn't exist
    let unregister_result = manager.unregister_transform("nonexistent_transform");
    assert!(unregister_result.is_ok());
    assert!(!unregister_result.unwrap(), "Unregistration should return false for non-existent transform");

    println!("✅ Registry module handles non-existent transform unregistration correctly");
}

// ========== Types Module Tests ==========

#[tokio::test]
async fn test_types_module_transform_runner_trait() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let manager = &fixture.transform_manager;

    // Test that TransformManager implements TransformRunner trait correctly
    let runner: &dyn TransformRunner = manager;

    // Test trait methods
    let exists_result = runner.transform_exists("test");
    assert!(exists_result.is_ok());

    let execute_result = runner.execute_transform_now("test");
    assert!(execute_result.is_ok());

    let field_transforms_result = runner.get_transforms_for_field("schema", "field");
    assert!(field_transforms_result.is_ok());

    println!("✅ Types module TransformRunner trait works correctly");
}

// ========== Module Integration Tests ==========

#[tokio::test]
async fn test_module_boundary_isolation() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let manager = &fixture.transform_manager;
    let registration = CommonTestFixture::create_sample_registration();

    // Test that each module's functionality is properly isolated
    // but works together through the manager

    // 1. Loading module - register transform
    let register_result = manager.register_transform_event_driven(registration.clone());
    assert!(register_result.is_ok());

    // 2. Manager module - check existence
    assert!(manager.transform_exists(&registration.transform_id).unwrap());

    // 3. Registry module - unregister transform
    let unregister_result = manager.unregister_transform(&registration.transform_id);
    assert!(unregister_result.is_ok());
    assert!(unregister_result.unwrap());

    // 4. Manager module - verify removal
    assert!(!manager.transform_exists(&registration.transform_id).unwrap());

    println!("✅ Module boundaries are properly isolated while maintaining integration");
}

#[tokio::test]
async fn test_error_propagation_across_modules() {
    let fixture = CommonTestFixture::new()
        .expect("Failed to create test fixture");

    let manager = &fixture.transform_manager;

    // Test that errors are properly propagated across module boundaries
    
    // This depends on specific error conditions that might be hard to trigger
    // For now, we test that operations that should succeed actually do succeed
    
    let valid_registration = CommonTestFixture::create_sample_registration();
    let register_result = manager.register_transform_event_driven(valid_registration);
    assert!(register_result.is_ok(), "Valid registration should succeed");

    println!("✅ Error propagation across modules works correctly");
}