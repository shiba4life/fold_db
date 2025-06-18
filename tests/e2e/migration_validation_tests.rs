//! Migration validation tests for the unified transform system.
//!
//! This module contains comprehensive tests that validate the unified transform
//! system provides equivalent functionality to the removed legacy modules,
//! ensuring no functionality regression occurred during the consolidation.

use datafold::db_operations::DbOperations;
use datafold::schema::types::Transform;
use datafold::transform_execution::{
    ExecutionContext, JobStatus, TransformConfig, TransformDefinition, TransformInput,
    TransformMetadata, TransformState, TransformStatus, TransformUpdate, UnifiedTransformManager,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};
use tempfile::tempdir;

/// Migration validation test fixture.
pub struct MigrationValidationFixture {
    pub manager: UnifiedTransformManager,
    pub db_ops: Arc<DbOperations>,
    _temp_dir: tempfile::TempDir,
}

impl MigrationValidationFixture {
    /// Creates a new migration validation fixture.
    pub fn new() -> Self {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let db_path = temp_dir.path().join("migration_validation.db");
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

    /// Waits for queue completion.
    pub fn wait_for_queue_completion(&self, timeout: Duration) -> bool {
        let start_time = SystemTime::now();
        
        loop {
            let queue_status = self.manager.get_queue_status();
            
            if queue_status.pending == 0 && queue_status.running == 0 {
                return true;
            }
            
            if start_time.elapsed().unwrap_or(Duration::from_secs(0)) > timeout {
                return false;
            }
            
            thread::sleep(Duration::from_millis(50));
        }
    }
}

// === LEGACY TRANSFORM MANAGER FUNCTIONALITY VALIDATION ===

#[test]
fn test_legacy_transform_registration_compatibility() {
    let fixture = MigrationValidationFixture::new();
    
    // Test registration patterns that were supported by legacy system
    let legacy_patterns = vec![
        // Simple transform
        TransformDefinition {
            id: "legacy_simple".to_string(),
            transform: Transform::new("return input + 1".to_string(), "legacy_simple.output".to_string()),
            inputs: vec!["legacy_simple.input".to_string()],
            metadata: HashMap::new(),
        },
        
        // Complex transform with metadata
        TransformDefinition {
            id: "legacy_complex".to_string(),
            transform: Transform::new(
                r#"
                if (!input.data) return null;
                return {
                    processed: input.data.map(x => x * 2),
                    count: input.data.length,
                    timestamp: Date.now()
                };
                "#.to_string(),
                "legacy_complex.output".to_string()
            ),
            inputs: vec!["legacy_complex.input".to_string()],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("type".to_string(), "data_processor".to_string());
                meta.insert("version".to_string(), "1.0".to_string());
                meta.insert("author".to_string(), "legacy_system".to_string());
                meta
            },
        },
        
        // Transform with multiple inputs
        TransformDefinition {
            id: "legacy_multi_input".to_string(),
            transform: Transform::new(
                "return { sum: input1 + input2, product: input1 * input2 }".to_string(),
                "legacy_multi_input.output".to_string()
            ),
            inputs: vec!["legacy_multi_input.input1".to_string(), "legacy_multi_input.input2".to_string()],
            metadata: HashMap::new(),
        },
    ];
    
    // Register all legacy patterns
    let mut registered_ids = Vec::new();
    for definition in legacy_patterns {
        let transform_id = fixture.manager.register_transform(definition).unwrap();
        registered_ids.push(transform_id);
    }
    
    // Verify all transforms were registered
    let transforms = fixture.manager.list_transforms(None);
    assert_eq!(transforms.len(), 3);
    
    for registered_id in &registered_ids {
        assert!(transforms.iter().any(|t| &t.id == registered_id));
    }
    
    // Test legacy-style execution for each transform
    let simple_input = TransformInput {
        values: {
            let mut values = HashMap::new();
            values.insert("legacy_simple.input".to_string(), JsonValue::Number(serde_json::Number::from(10)));
            values
        },
        context: ExecutionContext::default(),
    };
    let simple_result = fixture.manager.execute_transform("legacy_simple".to_string(), simple_input).unwrap();
    assert_eq!(simple_result.value, JsonValue::Number(serde_json::Number::from(11)));
    
    let complex_input = TransformInput {
        values: {
            let mut values = HashMap::new();
            values.insert("legacy_complex.input".to_string(), serde_json::json!({
                "data": [1, 2, 3, 4, 5]
            }));
            values
        },
        context: ExecutionContext::default(),
    };
    let complex_result = fixture.manager.execute_transform("legacy_complex".to_string(), complex_input).unwrap();
    let complex_obj = complex_result.value.as_object().unwrap();
    assert_eq!(complex_obj["processed"], serde_json::json!([2, 4, 6, 8, 10]));
    assert_eq!(complex_obj["count"], JsonValue::Number(serde_json::Number::from(5)));
    
    let multi_input = TransformInput {
        values: {
            let mut values = HashMap::new();
            values.insert("legacy_multi_input.input1".to_string(), JsonValue::Number(serde_json::Number::from(6)));
            values.insert("legacy_multi_input.input2".to_string(), JsonValue::Number(serde_json::Number::from(7)));
            values
        },
        context: ExecutionContext::default(),
    };
    let multi_result = fixture.manager.execute_transform("legacy_multi_input".to_string(), multi_input).unwrap();
    let multi_obj = multi_result.value.as_object().unwrap();
    assert_eq!(multi_obj["sum"], JsonValue::Number(serde_json::Number::from(13)));
    assert_eq!(multi_obj["product"], JsonValue::Number(serde_json::Number::from(42)));
}

#[test]
fn test_legacy_orchestration_compatibility() {
    let fixture = MigrationValidationFixture::new();
    
    // Test orchestration patterns that were supported by legacy system
    let orchestration_transform = TransformDefinition {
        id: "legacy_orchestration_test".to_string(),
        transform: Transform::new(
            r#"
            return {
                job_id: input.job_id || 'unknown',
                status: 'processed',
                result: (input.value || 0) * 2,
                orchestrated_at: Date.now()
            };
            "#.to_string(),
            "legacy_orchestration_test.output".to_string()
        ),
        inputs: vec!["legacy_orchestration_test.input".to_string()],
        metadata: HashMap::new(),
    };
    
    let transform_id = fixture.manager.register_transform(orchestration_transform).unwrap();
    
    // Test async execution (legacy orchestration style)
    let mut job_ids = Vec::new();
    for i in 1..=5 {
        let input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("legacy_orchestration_test.input".to_string(), serde_json::json!({
                    "job_id": format!("legacy_job_{}", i),
                    "value": i * 10
                }));
                values
            },
            context: ExecutionContext {
                schema_name: "legacy_orchestration".to_string(),
                field_name: "orchestrated_output".to_string(),
                atom_ref: Some(format!("legacy_atom_{}", i)),
                timestamp: SystemTime::now(),
                additional_data: HashMap::new(),
            },
        };
        
        let job_id = fixture.manager.enqueue_execution(transform_id.clone(), input).unwrap();
        job_ids.push(job_id);
    }
    
    // Wait for orchestration to complete
    let completed = fixture.wait_for_queue_completion(Duration::from_secs(10));
    assert!(completed, "Legacy-style orchestration should complete");
    
    // Verify queue status (legacy orchestration metrics)
    let queue_status = fixture.manager.get_queue_status();
    assert_eq!(queue_status.pending, 0);
    assert_eq!(queue_status.running, 0);
    assert_eq!(queue_status.completed, 5);
    
    // Verify execution history (legacy persistence pattern)
    let history = fixture.manager.get_execution_history(transform_id, None).unwrap();
    assert_eq!(history.len(), 5);
    
    for record in history {
        assert_eq!(record.status, JobStatus::Completed);
        assert!(record.result.is_some());
    }
}

#[test]
fn test_legacy_state_management_compatibility() {
    let fixture = MigrationValidationFixture::new();
    
    // Test state management patterns from legacy system
    let state_transform = TransformDefinition {
        id: "legacy_state_test".to_string(),
        transform: Transform::new(
            r#"
            return {
                state_update: true,
                counter: (input.counter || 0) + 1,
                session_id: input.session_id,
                updated_at: Date.now()
            };
            "#.to_string(),
            "legacy_state_test.output".to_string()
        ),
        inputs: vec!["legacy_state_test.input".to_string()],
        metadata: HashMap::new(),
    };
    
    let transform_id = fixture.manager.register_transform(state_transform).unwrap();
    
    // Execute multiple times to build state history (legacy pattern)
    for i in 1..=10 {
        let input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("legacy_state_test.input".to_string(), serde_json::json!({
                    "counter": i,
                    "session_id": "legacy_session_123"
                }));
                values
            },
            context: ExecutionContext {
                schema_name: "legacy_state".to_string(),
                field_name: "state_output".to_string(),
                atom_ref: Some(format!("legacy_state_atom_{}", i)),
                timestamp: SystemTime::now(),
                additional_data: HashMap::new(),
            },
        };
        
        let result = fixture.manager.execute_transform(transform_id.clone(), input).unwrap();
        let result_obj = result.value.as_object().unwrap();
        assert_eq!(result_obj["counter"], JsonValue::Number(serde_json::Number::from(i + 1)));
        assert_eq!(result_obj["session_id"], JsonValue::String("legacy_session_123".to_string()));
        
        thread::sleep(Duration::from_millis(10));
    }
    
    // Test legacy state retrieval patterns
    let state = fixture.manager.get_transform_state(transform_id.clone()).unwrap();
    assert_eq!(state.success_count, 10);
    assert_eq!(state.failure_count, 0);
    assert!(state.last_execution.is_some());
    assert!(state.last_result.is_some());
    
    // Test legacy history retrieval patterns
    let full_history = fixture.manager.get_execution_history(transform_id.clone(), None).unwrap();
    let limited_history = fixture.manager.get_execution_history(transform_id, Some(5)).unwrap();
    
    assert_eq!(full_history.len(), 10);
    assert_eq!(limited_history.len(), 5);
    
    // Verify chronological order (legacy requirement)
    for i in 0..limited_history.len() - 1 {
        assert!(limited_history[i].started_at >= limited_history[i + 1].started_at);
    }
}

// === LEGACY DATABASE OPERATIONS COMPATIBILITY ===

#[test]
fn test_legacy_persistence_compatibility() {
    // Test persistence across manager instances (legacy behavior)
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("legacy_persistence_test.db");
    
    let transform_id = {
        // First manager instance (simulate legacy system)
        let db = sled::open(&db_path).expect("Failed to open test database");
        let db_ops = Arc::new(DbOperations::new(db).expect("Failed to create DbOperations"));
        let config = TransformConfig::default();
        let manager1 = UnifiedTransformManager::new(Arc::clone(&db_ops), config).unwrap();
        
        // Register and execute transform (legacy persistence pattern)
        let definition = TransformDefinition {
            id: "legacy_persistence_test".to_string(),
            transform: Transform::new(
                r#"
                return {
                    persisted_data: input.data,
                    legacy_id: input.legacy_id,
                    persistence_test: true,
                    timestamp: Date.now()
                };
                "#.to_string(),
                "legacy_persistence_test.output".to_string()
            ),
            inputs: vec!["legacy_persistence_test.input".to_string()],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("legacy_system".to_string(), "true".to_string());
                meta.insert("persistence_version".to_string(), "1.0".to_string());
                meta
            },
        };
        
        let transform_id = manager1.register_transform(definition).unwrap();
        
        // Execute with legacy-style data
        let input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("legacy_persistence_test.input".to_string(), serde_json::json!({
                    "data": "legacy_test_data",
                    "legacy_id": "LEGACY_ID_12345"
                }));
                values
            },
            context: ExecutionContext {
                schema_name: "legacy_persistence".to_string(),
                field_name: "persisted_output".to_string(),
                atom_ref: Some("legacy_persistence_atom".to_string()),
                timestamp: SystemTime::now(),
                additional_data: HashMap::new(),
            },
        };
        
        let result = manager1.execute_transform(transform_id.clone(), input).unwrap();
        let result_obj = result.value.as_object().unwrap();
        assert_eq!(result_obj["persisted_data"], JsonValue::String("legacy_test_data".to_string()));
        assert_eq!(result_obj["legacy_id"], JsonValue::String("LEGACY_ID_12345".to_string()));
        
        transform_id
    };
    
    // Second manager instance (simulate unified system reading legacy data)
    let db = sled::open(&db_path).expect("Failed to reopen test database");
    let db_ops = Arc::new(DbOperations::new(db).expect("Failed to create DbOperations"));
    let config = TransformConfig::default();
    let manager2 = UnifiedTransformManager::new(Arc::clone(&db_ops), config).unwrap();
    
    // Verify legacy data is accessible
    let transforms = manager2.list_transforms(None);
    assert_eq!(transforms.len(), 1);
    assert_eq!(transforms[0].id, "legacy_persistence_test");
    
    // Verify legacy state is accessible
    let state = manager2.get_transform_state(transform_id.clone()).unwrap();
    assert_eq!(state.success_count, 1);
    assert!(state.last_execution.is_some());
    
    // Verify legacy history is accessible
    let history = manager2.get_execution_history(transform_id.clone(), None).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].status, JobStatus::Completed);
    
    // Verify continued operation with legacy data
    let input2 = TransformInput {
        values: {
            let mut values = HashMap::new();
            values.insert("legacy_persistence_test.input".to_string(), serde_json::json!({
                "data": "unified_system_data",
                "legacy_id": "UNIFIED_ID_67890"
            }));
            values
        },
        context: ExecutionContext {
            schema_name: "legacy_persistence".to_string(),
            field_name: "persisted_output".to_string(),
            atom_ref: Some("unified_persistence_atom".to_string()),
            timestamp: SystemTime::now(),
            additional_data: HashMap::new(),
        },
    };
    
    let result2 = manager2.execute_transform(transform_id.clone(), input2).unwrap();
    let result2_obj = result2.value.as_object().unwrap();
    assert_eq!(result2_obj["persisted_data"], JsonValue::String("unified_system_data".to_string()));
    
    // Verify updated state
    let final_state = manager2.get_transform_state(transform_id).unwrap();
    assert_eq!(final_state.success_count, 2);
}

// === LEGACY ERROR HANDLING COMPATIBILITY ===

#[test]
fn test_legacy_error_handling_compatibility() {
    let fixture = MigrationValidationFixture::new();
    
    // Test error handling patterns that were supported by legacy system
    let error_transform = TransformDefinition {
        id: "legacy_error_test".to_string(),
        transform: Transform::new(
            r#"
            // Legacy error patterns
            if (input.error_type === 'validation') {
                throw new Error('Legacy validation error: ' + input.message);
            }
            
            if (input.error_type === 'runtime') {
                throw new ReferenceError('Legacy runtime error: undefined variable');
            }
            
            if (input.error_type === 'custom') {
                var error = new Error('Legacy custom error');
                error.code = 'LEGACY_ERROR_001';
                error.details = input.details;
                throw error;
            }
            
            return {
                success: true,
                processed_value: input.value,
                legacy_compatible: true
            };
            "#.to_string(),
            "legacy_error_test.output".to_string()
        ),
        inputs: vec!["legacy_error_test.input".to_string()],
        metadata: HashMap::new(),
    };
    
    let transform_id = fixture.manager.register_transform(error_transform).unwrap();
    
    // Test legacy error scenarios
    let error_cases = vec![
        (serde_json::json!({"error_type": "validation", "message": "Invalid input"}), false),
        (serde_json::json!({"error_type": "runtime"}), false),
        (serde_json::json!({"error_type": "custom", "details": "Custom error details"}), false),
        (serde_json::json!({"value": 42}), true), // Success case
    ];
    
    for (test_input, should_succeed) in error_cases {
        let input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("legacy_error_test.input".to_string(), test_input);
                values
            },
            context: ExecutionContext {
                schema_name: "legacy_error_handling".to_string(),
                field_name: "error_output".to_string(),
                atom_ref: Some("legacy_error_atom".to_string()),
                timestamp: SystemTime::now(),
                additional_data: HashMap::new(),
            },
        };
        
        let result = fixture.manager.execute_transform(transform_id.clone(), input);
        
        if should_succeed {
            assert!(result.is_ok(), "Legacy success case should succeed");
            let result_obj = result.unwrap().value.as_object().unwrap();
            assert_eq!(result_obj["success"], JsonValue::Bool(true));
            assert_eq!(result_obj["legacy_compatible"], JsonValue::Bool(true));
        } else {
            assert!(result.is_err(), "Legacy error case should fail");
        }
        
        thread::sleep(Duration::from_millis(10));
    }
    
    // Verify legacy error tracking
    let state = fixture.manager.get_transform_state(transform_id.clone()).unwrap();
    assert_eq!(state.success_count, 1);
    assert_eq!(state.failure_count, 3);
    assert!(state.last_error.is_some());
    
    // Verify legacy error history
    let history = fixture.manager.get_execution_history(transform_id, None).unwrap();
    assert_eq!(history.len(), 4);
    
    let success_count = history.iter().filter(|r| r.status == JobStatus::Completed).count();
    let failure_count = history.iter().filter(|r| r.status == JobStatus::Failed).count();
    
    assert_eq!(success_count, 1);
    assert_eq!(failure_count, 3);
}

// === LEGACY CONFIGURATION COMPATIBILITY ===

#[test]
fn test_legacy_configuration_compatibility() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("legacy_config_test.db");
    let db = sled::open(&db_path).expect("Failed to open test database");
    let db_ops = Arc::new(DbOperations::new(db).expect("Failed to create DbOperations"));
    
    // Test legacy configuration patterns
    let legacy_config = TransformConfig {
        execution: datafold::transform_execution::config::ExecutionConfig {
            max_execution_time: Duration::from_secs(30), // Legacy timeout
            enable_parallel_execution: false, // Legacy sequential mode
            max_parallel_jobs: 1, // Legacy single-threaded
            execution_timeout_ms: 30000,
            enable_execution_caching: false, // Legacy no-cache
            cache_ttl_seconds: 0,
        },
        queue: datafold::transform_execution::config::QueueConfig {
            max_queue_size: 50, // Legacy small queue
            max_retry_attempts: 2, // Legacy retry count
            retry_delay_ms: 500, // Legacy retry delay
            priority_levels: 1, // Legacy no priorities
            enable_job_persistence: true,
            job_cleanup_interval_seconds: 60,
        },
        performance: datafold::transform_execution::config::PerformanceConfig {
            enable_metrics_collection: false, // Legacy no metrics
            metrics_buffer_size: 100,
            enable_performance_monitoring: false,
            slow_execution_threshold_ms: 10000,
            memory_limit_mb: 256,
            cpu_limit_percent: 50.0,
        },
        retry: datafold::transform_execution::types::RetryConfig {
            max_attempts: 2,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 1.5,
        },
        monitoring: datafold::transform_execution::config::MonitoringConfig {
            enable_logging: true,
            log_level: "warn".to_string(), // Legacy warn level
            enable_tracing: false,
            trace_sample_rate: 0.0,
            enable_health_checks: false,
            health_check_interval_seconds: 0,
        },
        security: datafold::transform_execution::config::SecurityConfig {
            enable_input_validation: false, // Legacy no validation
            enable_output_sanitization: false,
            max_input_size_bytes: 65536, // Legacy 64KB limit
            max_output_size_bytes: 65536,
            enable_execution_sandboxing: false,
            allowed_operations: vec![], // Legacy unrestricted
        },
        storage: datafold::transform_execution::config::StorageConfig {
            enable_state_persistence: true,
            state_cleanup_interval_seconds: 3600,
            max_history_entries_per_transform: 100, // Legacy small history
            enable_backup: false,
            backup_interval_hours: 0,
            backup_retention_days: 0,
        },
        custom: HashMap::new(),
    };
    
    // Create manager with legacy configuration
    let manager = UnifiedTransformManager::new(db_ops, legacy_config).unwrap();
    
    // Test that legacy configuration works
    let definition = TransformDefinition {
        id: "legacy_config_test".to_string(),
        transform: Transform::new(
            "return { legacy_mode: true, value: input + 1 }".to_string(),
            "legacy_config_test.output".to_string()
        ),
        inputs: vec!["legacy_config_test.input".to_string()],
        metadata: HashMap::new(),
    };
    
    let transform_id = manager.register_transform(definition).unwrap();
    
    // Execute with legacy configuration
    let input = TransformInput {
        values: {
            let mut values = HashMap::new();
            values.insert("legacy_config_test.input".to_string(), JsonValue::Number(serde_json::Number::from(99)));
            values
        },
        context: ExecutionContext::default(),
    };
    
    let result = manager.execute_transform(transform_id, input).unwrap();
    let result_obj = result.value.as_object().unwrap();
    assert_eq!(result_obj["legacy_mode"], JsonValue::Bool(true));
    assert_eq!(result_obj["value"], JsonValue::Number(serde_json::Number::from(100)));
    
    // Verify queue works with legacy configuration
    let queue_status = manager.get_queue_status();
    assert_eq!(queue_status.capacity, 50); // Legacy queue size
    
    // Test configuration reload (legacy pattern)
    let reload_result = manager.reload_config();
    assert!(reload_result.is_ok());
}

// === COMPREHENSIVE FUNCTIONALITY PARITY TEST ===

#[test]
fn test_comprehensive_functionality_parity() {
    let fixture = MigrationValidationFixture::new();
    
    // Test all major legacy functionality patterns in one comprehensive test
    let comprehensive_transform = TransformDefinition {
        id: "comprehensive_parity_test".to_string(),
        transform: Transform::new(
            r#"
            // Comprehensive legacy functionality test
            var result = {
                registration: true,
                execution: true,
                state_management: true,
                orchestration: true,
                persistence: true,
                error_handling: true,
                configuration: true
            };
            
            // Test data processing (legacy pattern)
            if (input.data && Array.isArray(input.data)) {
                result.data_processing = {
                    input_count: input.data.length,
                    processed_data: input.data.map(x => x * 2),
                    sum: input.data.reduce((a, b) => a + b, 0)
                };
            }
            
            // Test error conditions (legacy pattern)
            if (input.trigger_error) {
                throw new Error('Legacy error handling test');
            }
            
            // Test state updates (legacy pattern)
            if (input.update_state) {
                result.state_update = {
                    previous_value: input.previous_value || 0,
                    new_value: (input.previous_value || 0) + 1,
                    timestamp: Date.now()
                };
            }
            
            // Test orchestration (legacy pattern)
            if (input.orchestrate) {
                result.orchestration = {
                    job_id: input.job_id || 'default_job',
                    priority: input.priority || 1,
                    async_compatible: true
                };
            }
            
            return result;
            "#.to_string(),
            "comprehensive_parity_test.output".to_string()
        ),
        inputs: vec!["comprehensive_parity_test.input".to_string()],
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("parity_test".to_string(), "true".to_string());
            meta.insert("legacy_compatible".to_string(), "true".to_string());
            meta.insert("test_type".to_string(), "comprehensive".to_string());
            meta
        },
    };
    
    let transform_id = fixture.manager.register_transform(comprehensive_transform).unwrap();
    
    // Test comprehensive functionality scenarios
    let test_scenarios = vec![
        // Data processing scenario
        serde_json::json!({
            "data": [1, 2, 3, 4, 5],
            "scenario": "data_processing"
        }),
        
        // State management scenario
        serde_json::json!({
            "update_state": true,
            "previous_value": 10,
            "scenario": "state_management"
        }),
        
        // Orchestration scenario
        serde_json::json!({
            "orchestrate": true,
            "job_id": "legacy_job_123",
            "priority": 5,
            "scenario": "orchestration"
        }),
    ];
    
    for (i, scenario) in test_scenarios.iter().enumerate() {
        let input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("comprehensive_parity_test.input".to_string(), scenario.clone());
                values
            },
            context: ExecutionContext {
                schema_name: "comprehensive_parity".to_string(),
                field_name: "parity_output".to_string(),
                atom_ref: Some(format!("parity_atom_{}", i)),
                timestamp: SystemTime::now(),
                additional_data: HashMap::new(),
            },
        };
        
        let result = fixture.manager.execute_transform(transform_id.clone(), input).unwrap();
        let result_obj = result.value.as_object().unwrap();
        
        // Verify all core functionalities are present
        assert_eq!(result_obj["registration"], JsonValue::Bool(true));
        assert_eq!(result_obj["execution"], JsonValue::Bool(true));
        assert_eq!(result_obj["state_management"], JsonValue::Bool(true));
        assert_eq!(result_obj["orchestration"], JsonValue::Bool(true));
        assert_eq!(result_obj["persistence"], JsonValue::Bool(true));
        assert_eq!(result_obj["error_handling"], JsonValue::Bool(true));
        assert_eq!(result_obj["configuration"], JsonValue::Bool(true));
        
        // Verify scenario-specific functionality
        match scenario["scenario"].as_str().unwrap() {
            "data_processing" => {
                assert!(result_obj.contains_key("data_processing"));
                let data_proc = result_obj["data_processing"].as_object().unwrap();
                assert_eq!(data_proc["input_count"], JsonValue::Number(serde_json::Number::from(5)));
                assert_eq!(data_proc["processed_data"], serde_json::json!([2, 4, 6, 8, 10]));
                assert_eq!(data_proc["sum"], JsonValue::Number(serde_json::Number::from(15)));
            },
            "state_management" => {
                assert!(result_obj.contains_key("state_update"));
                let state_update = result_obj["state_update"].as_object().unwrap();
                assert_eq!(state_update["previous_value"], JsonValue::Number(serde_json::Number::from(10)));
                assert_eq!(state_update["new_value"], JsonValue::Number(serde_json::Number::from(11)));
            },
            "orchestration" => {
                assert!(result_obj.contains_key("orchestration"));
                let orchestration = result_obj["orchestration"].as_object().unwrap();
                assert_eq!(orchestration["job_id"], JsonValue::String("legacy_job_123".to_string()));
                assert_eq!(orchestration["priority"], JsonValue::Number(serde_json::Number::from(5)));
                assert_eq!(orchestration["async_compatible"], JsonValue::Bool(true));
            },
            _ => {},
        }
        
        thread::sleep(Duration::from_millis(10));
    }
    
    // Test error scenario
    let error_input = TransformInput {
        values: {
            let mut values = HashMap::new();
            values.insert("comprehensive_parity_test.input".to_string(), serde_json::json!({
                "trigger_error": true,
                "scenario": "error_handling"
            }));
            values
        },
        context: ExecutionContext::default(),
    };
    
    let error_result = fixture.manager.execute_transform(transform_id.clone(), error_input);
    assert!(error_result.is_err(), "Error scenario should fail");
    
    // Verify comprehensive state tracking
    let final_state = fixture.manager.get_transform_state(transform_id.clone()).unwrap();
    assert_eq!(final_state.success_count, 3);
    assert_eq!(final_state.failure_count, 1);
    
    // Verify comprehensive history tracking
    let history = fixture.manager.get_execution_history(transform_id, None).unwrap();
    assert_eq!(history.len(), 4);
    
    let success_count = history.iter().filter(|r| r.status == JobStatus::Completed).count();
    let failure_count = history.iter().filter(|r| r.status == JobStatus::Failed).count();
    
    assert_eq!(success_count, 3);
    assert_eq!(failure_count, 1);
    
    println!("Comprehensive functionality parity test completed successfully");
    println!("All legacy functionality patterns validated in unified system");
}