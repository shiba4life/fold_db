//! Comprehensive tests for transform state persistence and retrieval.
//!
//! This module validates the TransformStateStore functionality including
//! state persistence, execution history tracking, and database integration.

use datafold::db_operations::DbOperations;
use datafold::schema::types::Transform;
use datafold::transform_execution::{
    ExecutionContext, ExecutionRecord, JobStatus, TransformConfig, TransformDefinition,
    TransformInput, TransformState, TransformStateStore, UnifiedTransformManager,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};
use tempfile::tempdir;

/// Test fixture for persistence tests.
pub struct PersistenceTestFixture {
    pub state_store: TransformStateStore,
    pub manager: UnifiedTransformManager,
    pub db_ops: Arc<DbOperations>,
    _temp_dir: tempfile::TempDir,
}

impl PersistenceTestFixture {
    /// Creates a new persistence test fixture.
    pub fn new() -> Self {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let db_path = temp_dir.path().join("persistence_test.db");
        let db = sled::open(&db_path).expect("Failed to open test database");
        let db_ops = Arc::new(DbOperations::new(db).expect("Failed to create DbOperations"));
        
        let config = TransformConfig::default();
        let manager = UnifiedTransformManager::new(Arc::clone(&db_ops), config)
            .expect("Failed to create UnifiedTransformManager");
        
        let state_store = TransformStateStore::new(Arc::clone(&db_ops))
            .expect("Failed to create TransformStateStore");

        Self {
            state_store,
            manager,
            db_ops,
            _temp_dir: temp_dir,
        }
    }

    /// Creates a test transform definition.
    pub fn create_transform_definition(&self, id: &str) -> TransformDefinition {
        TransformDefinition {
            id: id.to_string(),
            transform: Transform::new("return input + 1".to_string(), format!("{}.output", id)),
            inputs: vec![format!("{}.input", id)],
            metadata: HashMap::new(),
        }
    }

    /// Creates test execution context.
    pub fn create_execution_context(&self, field_name: &str) -> ExecutionContext {
        ExecutionContext {
            schema_name: "persistence_test".to_string(),
            field_name: field_name.to_string(),
            atom_ref: Some(format!("atom_ref_{}", field_name)),
            timestamp: SystemTime::now(),
            additional_data: HashMap::new(),
        }
    }
}

// === BASIC STATE PERSISTENCE TESTS ===

#[test]
fn test_transform_state_creation_and_retrieval() {
    let fixture = PersistenceTestFixture::new();
    
    // Register a transform (this should create initial state)
    let definition = fixture.create_transform_definition("state_test");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    // Get the state
    let state = fixture.manager.get_transform_state(transform_id.clone()).unwrap();
    
    // Verify initial state properties
    assert_eq!(state.transform_id, "state_test");
    assert_eq!(state.success_count, 0);
    assert_eq!(state.failure_count, 0);
    assert!(state.last_execution.is_none());
    assert!(state.last_result.is_none());
    assert!(state.last_error.is_none());
    assert!(state.created_at <= SystemTime::now());
    assert!(state.updated_at >= state.created_at);
}

#[test]
fn test_state_updates_after_execution() {
    let fixture = PersistenceTestFixture::new();
    
    // Register and execute transform
    let definition = fixture.create_transform_definition("execution_state_test");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    let input = TransformInput {
        values: {
            let mut values = HashMap::new();
            values.insert("execution_state_test.input".to_string(), JsonValue::Number(serde_json::Number::from(10)));
            values
        },
        context: fixture.create_execution_context("test_field"),
    };
    
    let execution_time = SystemTime::now();
    let result = fixture.manager.execute_transform(transform_id.clone(), input).unwrap();
    
    // Get updated state
    let state = fixture.manager.get_transform_state(transform_id).unwrap();
    
    // Verify state was updated
    assert_eq!(state.success_count, 1);
    assert_eq!(state.failure_count, 0);
    assert!(state.last_execution.is_some());
    assert!(state.last_execution.unwrap() >= execution_time);
    assert!(state.last_result.is_some());
    assert!(state.last_error.is_none());
    assert_eq!(result.value, JsonValue::Number(serde_json::Number::from(11))); // 10 + 1
}

#[test]
fn test_state_updates_after_failure() {
    let fixture = PersistenceTestFixture::new();
    
    // Register transform that will fail
    let definition = TransformDefinition {
        id: "failure_state_test".to_string(),
        transform: Transform::new("throw 'Test error'".to_string(), "failure_state_test.output".to_string()),
        inputs: vec!["failure_state_test.input".to_string()],
        metadata: HashMap::new(),
    };
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    let input = TransformInput {
        values: {
            let mut values = HashMap::new();
            values.insert("failure_state_test.input".to_string(), JsonValue::Number(serde_json::Number::from(5)));
            values
        },
        context: fixture.create_execution_context("failure_field"),
    };
    
    let execution_time = SystemTime::now();
    let result = fixture.manager.execute_transform(transform_id.clone(), input);
    assert!(result.is_err());
    
    // Get updated state
    let state = fixture.manager.get_transform_state(transform_id).unwrap();
    
    // Verify failure was recorded
    assert_eq!(state.success_count, 0);
    assert_eq!(state.failure_count, 1);
    assert!(state.last_execution.is_some());
    assert!(state.last_execution.unwrap() >= execution_time);
    assert!(state.last_result.is_none());
    assert!(state.last_error.is_some());
}

#[test]
fn test_multiple_execution_state_tracking() {
    let fixture = PersistenceTestFixture::new();
    
    // Register transform
    let definition = fixture.create_transform_definition("multi_exec_test");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    // Execute successfully multiple times
    for i in 1..=5 {
        let input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("multi_exec_test.input".to_string(), JsonValue::Number(serde_json::Number::from(i)));
                values
            },
            context: fixture.create_execution_context(&format!("execution_{}", i)),
        };
        
        let result = fixture.manager.execute_transform(transform_id.clone(), input).unwrap();
        assert_eq!(result.value, JsonValue::Number(serde_json::Number::from(i + 1)));
    }
    
    // Check final state
    let state = fixture.manager.get_transform_state(transform_id).unwrap();
    assert_eq!(state.success_count, 5);
    assert_eq!(state.failure_count, 0);
}

// === EXECUTION HISTORY TESTS ===

#[test]
fn test_execution_history_recording() {
    let fixture = PersistenceTestFixture::new();
    
    // Register transform
    let definition = fixture.create_transform_definition("history_test");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    // Execute multiple times
    let execution_count = 3;
    for i in 1..=execution_count {
        let input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("history_test.input".to_string(), JsonValue::Number(serde_json::Number::from(i * 10)));
                values
            },
            context: fixture.create_execution_context(&format!("history_{}", i)),
        };
        
        let _result = fixture.manager.execute_transform(transform_id.clone(), input).unwrap();
        
        // Small delay to ensure distinct timestamps
        thread::sleep(Duration::from_millis(10));
    }
    
    // Get execution history
    let history = fixture.manager.get_execution_history(transform_id, None).unwrap();
    
    // Verify history
    assert_eq!(history.len(), execution_count);
    
    // Verify all executions are recorded as completed
    for record in &history {
        assert_eq!(record.status, JobStatus::Completed);
        assert!(record.completed_at.is_some());
        assert!(record.result.is_some());
        assert!(record.error_message.is_none());
    }
    
    // Verify chronological order (most recent first)
    for i in 0..history.len() - 1 {
        assert!(history[i].started_at >= history[i + 1].started_at);
    }
}

#[test]
fn test_execution_history_with_failures() {
    let fixture = PersistenceTestFixture::new();
    
    // Register transform that conditionally fails
    let definition = TransformDefinition {
        id: "conditional_failure_test".to_string(),
        transform: Transform::new(
            "if (input < 0) throw 'Negative input'; return input * 2".to_string(),
            "conditional_failure_test.output".to_string()
        ),
        inputs: vec!["conditional_failure_test.input".to_string()],
        metadata: HashMap::new(),
    };
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    // Execute with mix of success and failure
    let test_inputs = vec![5, -1, 10, -2, 15]; // 3 success, 2 failure
    
    for (i, input_value) in test_inputs.iter().enumerate() {
        let input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("conditional_failure_test.input".to_string(), JsonValue::Number(serde_json::Number::from(*input_value)));
                values
            },
            context: fixture.create_execution_context(&format!("mixed_{}", i)),
        };
        
        let result = fixture.manager.execute_transform(transform_id.clone(), input);
        
        if *input_value >= 0 {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err());
        }
        
        thread::sleep(Duration::from_millis(10));
    }
    
    // Get execution history
    let history = fixture.manager.get_execution_history(transform_id.clone(), None).unwrap();
    assert_eq!(history.len(), 5);
    
    // Count successes and failures
    let success_count = history.iter().filter(|r| r.status == JobStatus::Completed).count();
    let failure_count = history.iter().filter(|r| r.status == JobStatus::Failed).count();
    
    assert_eq!(success_count, 3);
    assert_eq!(failure_count, 2);
    
    // Verify state matches history
    let state = fixture.manager.get_transform_state(transform_id).unwrap();
    assert_eq!(state.success_count, 3);
    assert_eq!(state.failure_count, 2);
}

#[test]
fn test_execution_history_limit() {
    let fixture = PersistenceTestFixture::new();
    
    // Register transform
    let definition = fixture.create_transform_definition("limit_test");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    // Execute many times
    let total_executions = 10;
    for i in 1..=total_executions {
        let input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("limit_test.input".to_string(), JsonValue::Number(serde_json::Number::from(i)));
                values
            },
            context: fixture.create_execution_context(&format!("limit_{}", i)),
        };
        
        let _result = fixture.manager.execute_transform(transform_id.clone(), input).unwrap();
        thread::sleep(Duration::from_millis(5));
    }
    
    // Test various limits
    let limited_history_3 = fixture.manager.get_execution_history(transform_id.clone(), Some(3)).unwrap();
    assert_eq!(limited_history_3.len(), 3);
    
    let limited_history_5 = fixture.manager.get_execution_history(transform_id.clone(), Some(5)).unwrap();
    assert_eq!(limited_history_5.len(), 5);
    
    let unlimited_history = fixture.manager.get_execution_history(transform_id.clone(), None).unwrap();
    assert_eq!(unlimited_history.len(), total_executions);
    
    // Verify we get the most recent executions
    for i in 0..limited_history_3.len() - 1 {
        assert!(limited_history_3[i].started_at >= limited_history_3[i + 1].started_at);
    }
}

// === DATABASE PERSISTENCE TESTS ===

#[test]
fn test_state_persistence_across_manager_instances() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("cross_instance_test.db");
    
    let transform_id = {
        // Create first manager instance
        let db = sled::open(&db_path).expect("Failed to open test database");
        let db_ops = Arc::new(DbOperations::new(db).expect("Failed to create DbOperations"));
        let config = TransformConfig::default();
        let manager1 = UnifiedTransformManager::new(Arc::clone(&db_ops), config)
            .expect("Failed to create first manager");
        
        // Register and execute transform
        let definition = TransformDefinition {
            id: "cross_instance_test".to_string(),
            transform: Transform::new("return input.toUpperCase()".to_string(), "cross_instance_test.output".to_string()),
            inputs: vec!["cross_instance_test.input".to_string()],
            metadata: HashMap::new(),
        };
        let transform_id = manager1.register_transform(definition).unwrap();
        
        let input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("cross_instance_test.input".to_string(), JsonValue::String("hello".to_string()));
                values
            },
            context: ExecutionContext::default(),
        };
        let result = manager1.execute_transform(transform_id.clone(), input).unwrap();
        assert_eq!(result.value, JsonValue::String("HELLO".to_string()));
        
        transform_id
    };
    
    // Create second manager instance with same database
    let db = sled::open(&db_path).expect("Failed to reopen test database");
    let db_ops = Arc::new(DbOperations::new(db).expect("Failed to create DbOperations"));
    let config = TransformConfig::default();
    let manager2 = UnifiedTransformManager::new(Arc::clone(&db_ops), config)
        .expect("Failed to create second manager");
    
    // Verify transform is still registered
    let transforms = manager2.list_transforms(None);
    assert_eq!(transforms.len(), 1);
    assert_eq!(transforms[0].id, "cross_instance_test");
    
    // Verify state is persisted
    let state = manager2.get_transform_state(transform_id.clone()).unwrap();
    assert_eq!(state.success_count, 1);
    assert_eq!(state.failure_count, 0);
    
    // Verify execution history is persisted
    let history = manager2.get_execution_history(transform_id.clone(), None).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].status, JobStatus::Completed);
    
    // Execute another transform with second manager
    let input2 = TransformInput {
        values: {
            let mut values = HashMap::new();
            values.insert("cross_instance_test.input".to_string(), JsonValue::String("world".to_string()));
            values
        },
        context: ExecutionContext::default(),
    };
    let result2 = manager2.execute_transform(transform_id.clone(), input2).unwrap();
    assert_eq!(result2.value, JsonValue::String("WORLD".to_string()));
    
    // Verify updated state
    let final_state = manager2.get_transform_state(transform_id.clone()).unwrap();
    assert_eq!(final_state.success_count, 2);
    
    // Verify updated history
    let final_history = manager2.get_execution_history(transform_id, None).unwrap();
    assert_eq!(final_history.len(), 2);
}

#[test]
fn test_concurrent_state_updates() {
    let fixture = PersistenceTestFixture::new();
    
    // Register transform
    let definition = fixture.create_transform_definition("concurrent_state_test");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    let manager = Arc::new(fixture.manager);
    let mut handles = Vec::new();
    
    // Execute concurrently from multiple threads
    for i in 1..=5 {
        let manager_clone = Arc::clone(&manager);
        let transform_id_clone = transform_id.clone();
        
        let handle = thread::spawn(move || {
            let input = TransformInput {
                values: {
                    let mut values = HashMap::new();
                    values.insert("concurrent_state_test.input".to_string(), JsonValue::Number(serde_json::Number::from(i * 10)));
                    values
                },
                context: ExecutionContext {
                    schema_name: "concurrent_test".to_string(),
                    field_name: format!("concurrent_field_{}", i),
                    atom_ref: Some(format!("concurrent_atom_{}", i)),
                    timestamp: SystemTime::now(),
                    additional_data: HashMap::new(),
                },
            };
            
            manager_clone.execute_transform(transform_id_clone, input)
        });
        
        handles.push(handle);
    }
    
    // Wait for all executions
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_ok());
    }
    
    // Verify final state
    let state = manager.get_transform_state(transform_id.clone()).unwrap();
    assert_eq!(state.success_count, 5);
    assert_eq!(state.failure_count, 0);
    
    // Verify all executions are in history
    let history = manager.get_execution_history(transform_id, None).unwrap();
    assert_eq!(history.len(), 5);
    
    // All should be completed
    for record in history {
        assert_eq!(record.status, JobStatus::Completed);
    }
}

// === STATE CONSISTENCY TESTS ===

#[test]
fn test_state_consistency_after_mixed_operations() {
    let fixture = PersistenceTestFixture::new();
    
    // Register transform
    let definition = TransformDefinition {
        id: "consistency_test".to_string(),
        transform: Transform::new(
            "if (input === 'error') throw 'Test error'; return input + '_processed'".to_string(),
            "consistency_test.output".to_string()
        ),
        inputs: vec!["consistency_test.input".to_string()],
        metadata: HashMap::new(),
    };
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    // Mix of successful and failed executions
    let test_cases = vec![
        ("success1", true),
        ("error", false),
        ("success2", true),
        ("error", false),
        ("success3", true),
    ];
    
    for (input_str, should_succeed) in test_cases {
        let input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("consistency_test.input".to_string(), JsonValue::String(input_str.to_string()));
                values
            },
            context: fixture.create_execution_context("consistency_field"),
        };
        
        let result = fixture.manager.execute_transform(transform_id.clone(), input);
        
        if should_succeed {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err());
        }
        
        thread::sleep(Duration::from_millis(10));
    }
    
    // Verify state consistency
    let state = fixture.manager.get_transform_state(transform_id.clone()).unwrap();
    assert_eq!(state.success_count, 3);
    assert_eq!(state.failure_count, 2);
    
    // Verify history consistency
    let history = fixture.manager.get_execution_history(transform_id, None).unwrap();
    assert_eq!(history.len(), 5);
    
    let history_success_count = history.iter().filter(|r| r.status == JobStatus::Completed).count();
    let history_failure_count = history.iter().filter(|r| r.status == JobStatus::Failed).count();
    
    assert_eq!(history_success_count, 3);
    assert_eq!(history_failure_count, 2);
    
    // State and history should match
    assert_eq!(state.success_count as usize, history_success_count);
    assert_eq!(state.failure_count as usize, history_failure_count);
}

// === ERROR HANDLING TESTS ===

#[test]
fn test_get_state_for_nonexistent_transform() {
    let fixture = PersistenceTestFixture::new();
    
    let result = fixture.manager.get_transform_state("nonexistent_transform".to_string());
    assert!(result.is_err());
}

#[test]
fn test_get_history_for_nonexistent_transform() {
    let fixture = PersistenceTestFixture::new();
    
    let result = fixture.manager.get_execution_history("nonexistent_transform".to_string(), None);
    assert!(result.is_err());
}

// === CLEANUP AND MEMORY TESTS ===

#[test]
fn test_state_cleanup_after_transform_removal() {
    let fixture = PersistenceTestFixture::new();
    
    // Register and execute transform
    let definition = fixture.create_transform_definition("cleanup_test");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    let input = TransformInput {
        values: {
            let mut values = HashMap::new();
            values.insert("cleanup_test.input".to_string(), JsonValue::Number(serde_json::Number::from(42)));
            values
        },
        context: fixture.create_execution_context("cleanup_field"),
    };
    let _result = fixture.manager.execute_transform(transform_id.clone(), input).unwrap();
    
    // Verify state exists
    let state = fixture.manager.get_transform_state(transform_id.clone()).unwrap();
    assert_eq!(state.success_count, 1);
    
    // Remove transform
    fixture.manager.remove_transform(transform_id.clone()).unwrap();
    
    // Verify state is cleaned up
    let state_result = fixture.manager.get_transform_state(transform_id.clone());
    assert!(state_result.is_err());
    
    // Verify history is cleaned up
    let history_result = fixture.manager.get_execution_history(transform_id, None);
    assert!(history_result.is_err());
}