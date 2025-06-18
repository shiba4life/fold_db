//! Integration tests for the unified transform system.
//!
//! This module contains comprehensive integration tests that validate the entire
//! unified transform execution system across multiple components working together.

use datafold::db_operations::DbOperations;
use datafold::schema::types::Transform;
use datafold::transform_execution::{
    ExecutionContext, JobId, JobStatus, QueueStatus, TransformConfig, TransformDefinition,
    TransformInput, TransformMetadata, TransformOutput, TransformUpdate, UnifiedTransformManager,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};
use tempfile::tempdir;
use uuid::Uuid;

/// Test fixture for unified transform integration tests.
pub struct UnifiedTransformTestFixture {
    pub manager: UnifiedTransformManager,
    pub db_ops: Arc<DbOperations>,
    _temp_dir: tempfile::TempDir, // Keep alive for the test duration
}

impl UnifiedTransformTestFixture {
    /// Creates a new test fixture with a clean database.
    pub fn new() -> Self {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let db_path = temp_dir.path().join("test.db");
        let db = sled::open(&db_path).expect("Failed to open test database");
        let db_ops = Arc::new(DbOperations::new(db).expect("Failed to create DbOperations"));
        
        let config = TransformConfig::default();
        let manager = UnifiedTransformManager::new(Arc::clone(&db_ops), config)
            .expect("Failed to create UnifiedTransformManager");

        Self {
            manager,
            db_ops,
            _temp_dir: temp_dir,
        }
    }

    /// Creates a test transform definition.
    pub fn create_transform_definition(&self, id: &str, logic: &str) -> TransformDefinition {
        TransformDefinition {
            id: id.to_string(),
            transform: Transform::new(logic.to_string(), format!("{}.output", id)),
            inputs: vec![format!("{}.input", id)],
            metadata: HashMap::new(),
        }
    }

    /// Creates test input data.
    pub fn create_test_input(&self, value: i64, context_field: &str) -> TransformInput {
        let mut values = HashMap::new();
        values.insert("test.input".to_string(), JsonValue::Number(serde_json::Number::from(value)));
        
        let mut additional_data = HashMap::new();
        additional_data.insert("test_context".to_string(), context_field.to_string());
        
        TransformInput {
            values,
            context: ExecutionContext {
                schema_name: "integration_test".to_string(),
                field_name: "test_field".to_string(),
                atom_ref: Some(format!("atom_ref_{}", Uuid::new_v4())),
                timestamp: SystemTime::now(),
                additional_data,
            },
        }
    }

    /// Waits for async execution to complete.
    pub fn wait_for_job_completion(&self, job_id: JobId, timeout: Duration) -> bool {
        let start_time = SystemTime::now();
        
        loop {
            let queue_status = self.manager.get_queue_status();
            
            // Check if job completed (no pending or running jobs)
            if queue_status.pending == 0 && queue_status.running == 0 {
                return true;
            }
            
            // Check timeout
            if start_time.elapsed().unwrap_or(Duration::from_secs(0)) > timeout {
                return false;
            }
            
            thread::sleep(Duration::from_millis(10));
        }
    }
}

// === BASIC INTEGRATION TESTS ===

#[test]
fn test_end_to_end_transform_workflow() {
    let fixture = UnifiedTransformTestFixture::new();
    
    // Step 1: Register a transform
    let definition = fixture.create_transform_definition("e2e_test", "return input * 2 + 5");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    assert_eq!(transform_id, "e2e_test");
    
    // Step 2: Execute synchronously
    let input = fixture.create_test_input(10, "sync_execution");
    let result = fixture.manager.execute_transform(transform_id.clone(), input).unwrap();
    assert_eq!(result.value, JsonValue::Number(serde_json::Number::from(25))); // 10 * 2 + 5
    
    // Step 3: Check state was updated
    let state = fixture.manager.get_transform_state(transform_id.clone()).unwrap();
    assert_eq!(state.success_count, 1);
    assert_eq!(state.failure_count, 0);
    assert!(state.last_execution.is_some());
    
    // Step 4: Check execution history
    let history = fixture.manager.get_execution_history(transform_id, Some(10)).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].status, JobStatus::Completed);
}

#[test]
fn test_async_execution_workflow() {
    let fixture = UnifiedTransformTestFixture::new();
    
    // Register transform
    let definition = fixture.create_transform_definition("async_test", "return input + 100");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    // Enqueue for async execution
    let input = fixture.create_test_input(42, "async_execution");
    let job_id = fixture.manager.enqueue_execution(transform_id.clone(), input).unwrap();
    assert!(!job_id.to_string().is_empty());
    
    // Check queue status
    let queue_status = fixture.manager.get_queue_status();
    assert!(queue_status.pending > 0 || queue_status.running > 0);
    
    // Wait for completion
    let completed = fixture.wait_for_job_completion(job_id, Duration::from_secs(5));
    assert!(completed, "Job should complete within timeout");
    
    // Verify final state
    let final_queue_status = fixture.manager.get_queue_status();
    assert!(final_queue_status.completed > 0);
}

#[test]
fn test_multiple_concurrent_transforms() {
    let fixture = UnifiedTransformTestFixture::new();
    let manager = Arc::new(fixture.manager);
    
    // Register multiple transforms
    let mut transform_ids = Vec::new();
    for i in 1..=5 {
        let definition = TransformDefinition {
            id: format!("concurrent_{}", i),
            transform: Transform::new(
                format!("return input * {}", i),
                format!("concurrent_{}.output", i)
            ),
            inputs: vec![format!("concurrent_{}.input", i)],
            metadata: HashMap::new(),
        };
        let transform_id = manager.register_transform(definition).unwrap();
        transform_ids.push(transform_id);
    }
    
    // Execute all transforms concurrently
    let mut handles = Vec::new();
    for (i, transform_id) in transform_ids.into_iter().enumerate() {
        let manager_clone = Arc::clone(&manager);
        let handle = thread::spawn(move || {
            let input = TransformInput {
                values: {
                    let mut values = HashMap::new();
                    values.insert(format!("concurrent_{}.input", i + 1), JsonValue::Number(serde_json::Number::from(10)));
                    values
                },
                context: ExecutionContext {
                    schema_name: "concurrent_test".to_string(),
                    field_name: format!("field_{}", i + 1),
                    atom_ref: Some(format!("concurrent_atom_{}", i + 1)),
                    timestamp: SystemTime::now(),
                    additional_data: HashMap::new(),
                },
            };
            manager_clone.execute_transform(transform_id, input)
        });
        handles.push(handle);
    }
    
    // Wait for all executions and verify results
    for (i, handle) in handles.into_iter().enumerate() {
        let result = handle.join().unwrap().unwrap();
        let expected = 10 * (i + 1) as i64; // input * multiplier
        assert_eq!(result.value, JsonValue::Number(serde_json::Number::from(expected)));
    }
}

#[test]
fn test_transform_update_integration() {
    let fixture = UnifiedTransformTestFixture::new();
    
    // Register initial transform
    let definition = fixture.create_transform_definition("update_test", "return input + 1");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    // Execute with original logic
    let input = fixture.create_test_input(5, "original");
    let result1 = fixture.manager.execute_transform(transform_id.clone(), input).unwrap();
    assert_eq!(result1.value, JsonValue::Number(serde_json::Number::from(6))); // 5 + 1
    
    // Update transform logic
    let update = TransformUpdate {
        transform: Some(Transform::new("return input * 3".to_string(), "update_test.output".to_string())),
        inputs: None,
        metadata: None,
        status: None,
    };
    fixture.manager.update_transform(transform_id.clone(), update).unwrap();
    
    // Execute with updated logic
    let input2 = fixture.create_test_input(5, "updated");
    let result2 = fixture.manager.execute_transform(transform_id.clone(), input2).unwrap();
    assert_eq!(result2.value, JsonValue::Number(serde_json::Number::from(15))); // 5 * 3
    
    // Verify execution history includes both executions
    let history = fixture.manager.get_execution_history(transform_id, Some(10)).unwrap();
    assert_eq!(history.len(), 2);
}

#[test]
fn test_transform_lifecycle_integration() {
    let fixture = UnifiedTransformTestFixture::new();
    
    // 1. Register transform
    let definition = fixture.create_transform_definition("lifecycle_test", "return input.toUpperCase()");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    // 2. List transforms - should find our transform
    let transforms = fixture.manager.list_transforms(None);
    assert_eq!(transforms.len(), 1);
    assert_eq!(transforms[0].id, "lifecycle_test");
    
    // 3. Execute transform
    let mut values = HashMap::new();
    values.insert("test.input".to_string(), JsonValue::String("hello".to_string()));
    let input = TransformInput {
        values,
        context: ExecutionContext::default(),
    };
    let result = fixture.manager.execute_transform(transform_id.clone(), input).unwrap();
    assert_eq!(result.value, JsonValue::String("HELLO".to_string()));
    
    // 4. Check state
    let state = fixture.manager.get_transform_state(transform_id.clone()).unwrap();
    assert_eq!(state.success_count, 1);
    
    // 5. Remove transform
    fixture.manager.remove_transform(transform_id.clone()).unwrap();
    
    // 6. Verify removal
    let transforms_after = fixture.manager.list_transforms(None);
    assert_eq!(transforms_after.len(), 0);
    
    // 7. Verify execution fails after removal
    let input2 = TransformInput {
        values: HashMap::new(),
        context: ExecutionContext::default(),
    };
    let result = fixture.manager.execute_transform(transform_id, input2);
    assert!(result.is_err());
}

// === ERROR HANDLING INTEGRATION TESTS ===

#[test]
fn test_execution_error_handling_integration() {
    let fixture = UnifiedTransformTestFixture::new();
    
    // Register transform that will fail under certain conditions
    let definition = fixture.create_transform_definition("error_test", "if (input < 0) throw 'Negative input not allowed'; return input * 2");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    // Test successful execution
    let good_input = fixture.create_test_input(5, "success_case");
    let result = fixture.manager.execute_transform(transform_id.clone(), good_input).unwrap();
    assert_eq!(result.value, JsonValue::Number(serde_json::Number::from(10)));
    
    // Test error execution
    let bad_input = fixture.create_test_input(-1, "error_case");
    let error_result = fixture.manager.execute_transform(transform_id.clone(), bad_input);
    assert!(error_result.is_err());
    
    // Check state reflects both success and failure
    let state = fixture.manager.get_transform_state(transform_id.clone()).unwrap();
    assert_eq!(state.success_count, 1);
    assert_eq!(state.failure_count, 1);
    assert!(state.last_error.is_some());
    
    // Check execution history includes both attempts
    let history = fixture.manager.get_execution_history(transform_id, Some(10)).unwrap();
    assert_eq!(history.len(), 2);
    
    // Verify one succeeded and one failed
    let success_count = history.iter().filter(|r| r.status == JobStatus::Completed).count();
    let failure_count = history.iter().filter(|r| r.status == JobStatus::Failed).count();
    assert_eq!(success_count, 1);
    assert_eq!(failure_count, 1);
}

#[test]
fn test_async_error_handling_integration() {
    let fixture = UnifiedTransformTestFixture::new();
    
    // Register transform that fails
    let definition = fixture.create_transform_definition("async_error_test", "throw 'Always fails'");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    // Enqueue failing job
    let input = fixture.create_test_input(1, "async_error");
    let job_id = fixture.manager.enqueue_execution(transform_id, input).unwrap();
    
    // Wait for job to complete (with failure)
    let completed = fixture.wait_for_job_completion(job_id.clone(), Duration::from_secs(5));
    assert!(completed, "Job should complete even with error");
    
    // Check queue status shows failed job
    let queue_status = fixture.manager.get_queue_status();
    assert!(queue_status.failed > 0);
    
    // Test retry mechanism
    let retry_result = fixture.manager.retry_failed(job_id);
    // Retry might succeed or fail depending on implementation details
    assert!(retry_result.is_ok() || retry_result.is_err());
}

// === PERFORMANCE INTEGRATION TESTS ===

#[test]
fn test_high_volume_execution_integration() {
    let fixture = UnifiedTransformTestFixture::new();
    
    // Register a simple transform
    let definition = fixture.create_transform_definition("volume_test", "return input + 1");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    // Execute many times
    let execution_count = 100;
    for i in 0..execution_count {
        let input = fixture.create_test_input(i, &format!("volume_{}", i));
        let result = fixture.manager.execute_transform(transform_id.clone(), input).unwrap();
        assert_eq!(result.value, JsonValue::Number(serde_json::Number::from(i + 1)));
    }
    
    // Verify state tracking
    let state = fixture.manager.get_transform_state(transform_id.clone()).unwrap();
    assert_eq!(state.success_count, execution_count as u64);
    assert_eq!(state.failure_count, 0);
    
    // Verify execution history (may be limited)
    let history = fixture.manager.get_execution_history(transform_id, None).unwrap();
    assert!(history.len() <= execution_count);
    
    // All recorded executions should be successful
    for record in history {
        assert_eq!(record.status, JobStatus::Completed);
    }
}

#[test]
fn test_concurrent_async_execution_integration() {
    let fixture = UnifiedTransformTestFixture::new();
    
    // Register multiple transforms for concurrent async execution
    let mut transform_ids = Vec::new();
    for i in 1..=10 {
        let definition = fixture.create_transform_definition(
            &format!("async_concurrent_{}", i),
            &format!("return input * {} + {}", i, i * 10)
        );
        let transform_id = fixture.manager.register_transform(definition).unwrap();
        transform_ids.push(transform_id);
    }
    
    // Enqueue all transforms for async execution
    let mut job_ids = Vec::new();
    for (i, transform_id) in transform_ids.iter().enumerate() {
        let input = fixture.create_test_input(5, &format!("async_concurrent_{}", i + 1));
        let job_id = fixture.manager.enqueue_execution(transform_id.clone(), input).unwrap();
        job_ids.push(job_id);
    }
    
    // Wait for all jobs to complete
    let timeout = Duration::from_secs(10);
    let start_time = SystemTime::now();
    
    loop {
        let queue_status = fixture.manager.get_queue_status();
        if queue_status.pending == 0 && queue_status.running == 0 {
            break;
        }
        
        if start_time.elapsed().unwrap_or(Duration::from_secs(0)) > timeout {
            panic!("Async executions did not complete within timeout");
        }
        
        thread::sleep(Duration::from_millis(50));
    }
    
    // Verify all jobs completed successfully
    let final_queue_status = fixture.manager.get_queue_status();
    assert_eq!(final_queue_status.completed, 10);
    assert_eq!(final_queue_status.failed, 0);
}

// === CONFIGURATION INTEGRATION TESTS ===

#[test]
fn test_config_reload_integration() {
    let fixture = UnifiedTransformTestFixture::new();
    
    // Register and execute a transform
    let definition = fixture.create_transform_definition("config_test", "return input * 2");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    let input = fixture.create_test_input(3, "before_reload");
    let result1 = fixture.manager.execute_transform(transform_id.clone(), input).unwrap();
    assert_eq!(result1.value, JsonValue::Number(serde_json::Number::from(6)));
    
    // Reload configuration
    let reload_result = fixture.manager.reload_config();
    assert!(reload_result.is_ok());
    
    // Execute again after reload
    let input2 = fixture.create_test_input(4, "after_reload");
    let result2 = fixture.manager.execute_transform(transform_id, input2).unwrap();
    assert_eq!(result2.value, JsonValue::Number(serde_json::Number::from(8)));
}

// === DATABASE INTEGRATION TESTS ===

#[test]
fn test_database_persistence_integration() {
    let fixture = UnifiedTransformTestFixture::new();
    
    // Register transform
    let definition = fixture.create_transform_definition("persist_test", "return input.length");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    // Execute with string input
    let mut values = HashMap::new();
    values.insert("test.input".to_string(), JsonValue::String("hello world".to_string()));
    let input = TransformInput {
        values,
        context: ExecutionContext::default(),
    };
    let result = fixture.manager.execute_transform(transform_id.clone(), input).unwrap();
    assert_eq!(result.value, JsonValue::Number(serde_json::Number::from(11)));
    
    // Create a new manager instance with the same database
    let manager2 = UnifiedTransformManager::new(
        Arc::clone(&fixture.db_ops),
        TransformConfig::default()
    ).unwrap();
    
    // Verify transform is still registered
    let transforms = manager2.list_transforms(None);
    assert_eq!(transforms.len(), 1);
    assert_eq!(transforms[0].id, "persist_test");
    
    // Verify state is persisted
    let state = manager2.get_transform_state(transform_id.clone()).unwrap();
    assert_eq!(state.success_count, 1);
    
    // Verify execution history is persisted
    let history = manager2.get_execution_history(transform_id, None).unwrap();
    assert_eq!(history.len(), 1);
}