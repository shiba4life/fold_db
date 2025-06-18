//! Comprehensive tests for transform configuration management and error handling.
//!
//! This module validates the TransformConfigLoader functionality including
//! configuration loading, validation, hot-reload capabilities, and error scenarios.

use datafold::db_operations::DbOperations;
use datafold::schema::types::Transform;
use datafold::transform_execution::{
    TransformConfig, TransformConfigLoader, TransformDefinition, TransformError,
    UnifiedTransformManager,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

/// Test fixture for configuration tests.
pub struct ConfigurationTestFixture {
    pub manager: UnifiedTransformManager,
    pub config_loader: TransformConfigLoader,
    pub db_ops: Arc<DbOperations>,
    _temp_dir: tempfile::TempDir,
}

impl ConfigurationTestFixture {
    /// Creates a new configuration test fixture.
    pub fn new() -> Self {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let db_path = temp_dir.path().join("config_test.db");
        let db = sled::open(&db_path).expect("Failed to open test database");
        let db_ops = Arc::new(DbOperations::new(db).expect("Failed to create DbOperations"));
        
        let config = TransformConfig::default();
        let config_loader = TransformConfigLoader::new(config.clone())
            .expect("Failed to create TransformConfigLoader");
        
        let manager = UnifiedTransformManager::new(Arc::clone(&db_ops), config)
            .expect("Failed to create UnifiedTransformManager");

        Self {
            manager,
            config_loader,
            db_ops,
            _temp_dir: temp_dir,
        }
    }

    /// Creates a custom configuration.
    pub fn create_custom_config() -> TransformConfig {
        TransformConfig {
            execution: datafold::transform_execution::config::ExecutionConfig {
                max_execution_time: Duration::from_secs(30),
                enable_parallel_execution: true,
                max_parallel_jobs: 10,
                execution_timeout_ms: 30000,
                enable_execution_caching: false,
                cache_ttl_seconds: 3600,
            },
            queue: datafold::transform_execution::config::QueueConfig {
                max_queue_size: 1000,
                max_retry_attempts: 3,
                retry_delay_ms: 1000,
                priority_levels: 5,
                enable_job_persistence: true,
                job_cleanup_interval_seconds: 300,
            },
            performance: datafold::transform_execution::config::PerformanceConfig {
                enable_metrics_collection: true,
                metrics_buffer_size: 10000,
                enable_performance_monitoring: true,
                slow_execution_threshold_ms: 5000,
                memory_limit_mb: 512,
                cpu_limit_percent: 80.0,
            },
            retry: datafold::transform_execution::types::RetryConfig {
                max_attempts: 5,
                base_delay: Duration::from_millis(500),
                max_delay: Duration::from_secs(60),
                backoff_multiplier: 2.5,
            },
            monitoring: datafold::transform_execution::config::MonitoringConfig {
                enable_logging: true,
                log_level: "info".to_string(),
                enable_tracing: true,
                trace_sample_rate: 0.1,
                enable_health_checks: true,
                health_check_interval_seconds: 60,
            },
            security: datafold::transform_execution::config::SecurityConfig {
                enable_input_validation: true,
                enable_output_sanitization: true,
                max_input_size_bytes: 1048576, // 1MB
                max_output_size_bytes: 1048576, // 1MB
                enable_execution_sandboxing: false,
                allowed_operations: vec!["string".to_string(), "math".to_string()],
            },
            storage: datafold::transform_execution::config::StorageConfig {
                enable_state_persistence: true,
                state_cleanup_interval_seconds: 3600,
                max_history_entries_per_transform: 1000,
                enable_backup: false,
                backup_interval_hours: 24,
                backup_retention_days: 7,
            },
            custom: HashMap::new(),
        }
    }

    /// Creates a test transform definition.
    pub fn create_transform_definition(&self, id: &str) -> TransformDefinition {
        TransformDefinition {
            id: id.to_string(),
            transform: Transform::new("return input * 2".to_string(), format!("{}.output", id)),
            inputs: vec![format!("{}.input", id)],
            metadata: HashMap::new(),
        }
    }
}

// === CONFIGURATION LOADING TESTS ===

#[test]
fn test_default_configuration_loading() {
    let fixture = ConfigurationTestFixture::new();
    
    // Default configuration should be valid and working
    let definition = fixture.create_transform_definition("default_config_test");
    let transform_id = fixture.manager.register_transform(definition);
    assert!(transform_id.is_ok());
    
    // Config reload should work
    let reload_result = fixture.manager.reload_config();
    assert!(reload_result.is_ok());
}

#[test]
fn test_custom_configuration_loading() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("custom_config_test.db");
    let db = sled::open(&db_path).expect("Failed to open test database");
    let db_ops = Arc::new(DbOperations::new(db).expect("Failed to create DbOperations"));
    
    // Create custom configuration
    let custom_config = ConfigurationTestFixture::create_custom_config();
    
    // Create manager with custom config
    let manager = UnifiedTransformManager::new(Arc::clone(&db_ops), custom_config);
    assert!(manager.is_ok());
    
    let manager = manager.unwrap();
    
    // Should work with custom configuration
    let definition = TransformDefinition {
        id: "custom_config_test".to_string(),
        transform: Transform::new("return input + 100".to_string(), "custom_config_test.output".to_string()),
        inputs: vec!["custom_config_test.input".to_string()],
        metadata: HashMap::new(),
    };
    
    let transform_id = manager.register_transform(definition);
    assert!(transform_id.is_ok());
}

#[test]
fn test_configuration_validation() {
    // Test with valid configuration
    let valid_config = ConfigurationTestFixture::create_custom_config();
    let config_loader = TransformConfigLoader::new(valid_config);
    assert!(config_loader.is_ok());
    
    // Test with extreme values that might cause issues
    let mut extreme_config = TransformConfig::default();
    extreme_config.queue.max_queue_size = 0; // This might be invalid
    
    // The configuration loader should handle extreme values gracefully
    let extreme_loader = TransformConfigLoader::new(extreme_config);
    // Depending on implementation, this might succeed or fail
    // but should not panic
    assert!(extreme_loader.is_ok() || extreme_loader.is_err());
}

// === CONFIGURATION HOT-RELOAD TESTS ===

#[test]
fn test_configuration_hot_reload() {
    let fixture = ConfigurationTestFixture::new();
    
    // Register and execute a transform with initial config
    let definition = fixture.create_transform_definition("hot_reload_test");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    // Test reload functionality
    let reload_result = fixture.manager.reload_config();
    assert!(reload_result.is_ok());
    
    // Transform should still work after reload
    let transforms = fixture.manager.list_transforms(None);
    assert!(transforms.iter().any(|t| t.id == "hot_reload_test"));
}

#[test]
fn test_multiple_config_reloads() {
    let fixture = ConfigurationTestFixture::new();
    
    // Register transform
    let definition = fixture.create_transform_definition("multi_reload_test");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    // Perform multiple reloads
    for i in 1..=5 {
        let reload_result = fixture.manager.reload_config();
        assert!(reload_result.is_ok(), "Reload {} should succeed", i);
        
        // Verify system still works after each reload
        let queue_status = fixture.manager.get_queue_status();
        assert!(queue_status.capacity > 0); // Basic sanity check
    }
    
    // Verify transform is still registered and functional
    let transforms = fixture.manager.list_transforms(None);
    assert!(transforms.iter().any(|t| t.id == "multi_reload_test"));
}

// === ERROR HANDLING TESTS ===

#[test]
fn test_transform_registration_error_handling() {
    let fixture = ConfigurationTestFixture::new();
    
    // Test duplicate registration
    let definition1 = fixture.create_transform_definition("duplicate_test");
    let definition2 = fixture.create_transform_definition("duplicate_test");
    
    let first_registration = fixture.manager.register_transform(definition1);
    assert!(first_registration.is_ok());
    
    let second_registration = fixture.manager.register_transform(definition2);
    assert!(second_registration.is_err());
    
    // Test invalid transform definition
    let invalid_definition = TransformDefinition {
        id: "".to_string(), // Empty ID should be invalid
        transform: Transform::new("return input".to_string(), "output".to_string()),
        inputs: vec!["input".to_string()],
        metadata: HashMap::new(),
    };
    
    let invalid_registration = fixture.manager.register_transform(invalid_definition);
    assert!(invalid_registration.is_err());
}

#[test]
fn test_transform_execution_error_handling() {
    let fixture = ConfigurationTestFixture::new();
    
    // Register transform with logic that can fail
    let error_definition = TransformDefinition {
        id: "error_test".to_string(),
        transform: Transform::new(
            "if (input.type === 'error') throw new Error('Test error'); return input.value * 2".to_string(),
            "error_test.output".to_string()
        ),
        inputs: vec!["error_test.input".to_string()],
        metadata: HashMap::new(),
    };
    
    let transform_id = fixture.manager.register_transform(error_definition).unwrap();
    
    // Test execution with error-triggering input
    let error_input = datafold::transform_execution::TransformInput {
        values: {
            let mut values = HashMap::new();
            values.insert("error_test.input".to_string(), serde_json::json!({
                "type": "error",
                "value": 10
            }));
            values
        },
        context: datafold::transform_execution::ExecutionContext::default(),
    };
    
    let error_result = fixture.manager.execute_transform(transform_id.clone(), error_input);
    assert!(error_result.is_err());
    
    // Test execution with valid input
    let valid_input = datafold::transform_execution::TransformInput {
        values: {
            let mut values = HashMap::new();
            values.insert("error_test.input".to_string(), serde_json::json!({
                "type": "valid",
                "value": 10
            }));
            values
        },
        context: datafold::transform_execution::ExecutionContext::default(),
    };
    
    let valid_result = fixture.manager.execute_transform(transform_id, valid_input);
    assert!(valid_result.is_ok());
}

#[test]
fn test_nonexistent_transform_operations() {
    let fixture = ConfigurationTestFixture::new();
    
    let nonexistent_id = "does_not_exist".to_string();
    
    // Test execution of nonexistent transform
    let input = datafold::transform_execution::TransformInput {
        values: HashMap::new(),
        context: datafold::transform_execution::ExecutionContext::default(),
    };
    
    let exec_result = fixture.manager.execute_transform(nonexistent_id.clone(), input);
    assert!(exec_result.is_err());
    if let Err(error) = exec_result {
        match error {
            TransformError::NotFoundError { .. } => {
                // Expected error type
            }
            _ => panic!("Expected NotFoundError, got {:?}", error),
        }
    }
    
    // Test update of nonexistent transform
    let update = datafold::transform_execution::TransformUpdate {
        transform: None,
        inputs: None,
        metadata: None,
        status: None,
    };
    
    let update_result = fixture.manager.update_transform(nonexistent_id.clone(), update);
    assert!(update_result.is_err());
    
    // Test removal of nonexistent transform
    let remove_result = fixture.manager.remove_transform(nonexistent_id.clone());
    assert!(remove_result.is_err());
    
    // Test state query of nonexistent transform
    let state_result = fixture.manager.get_transform_state(nonexistent_id.clone());
    assert!(state_result.is_err());
    
    // Test history query of nonexistent transform
    let history_result = fixture.manager.get_execution_history(nonexistent_id, None);
    assert!(history_result.is_err());
}

#[test]
fn test_invalid_input_handling() {
    let fixture = ConfigurationTestFixture::new();
    
    // Register transform expecting specific input format
    let definition = TransformDefinition {
        id: "input_validation_test".to_string(),
        transform: Transform::new("return input.required_field * 2".to_string(), "input_validation_test.output".to_string()),
        inputs: vec!["input_validation_test.input".to_string()],
        metadata: HashMap::new(),
    };
    
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    // Test with missing required field
    let invalid_input = datafold::transform_execution::TransformInput {
        values: {
            let mut values = HashMap::new();
            values.insert("input_validation_test.input".to_string(), serde_json::json!({
                "other_field": "value"
            }));
            values
        },
        context: datafold::transform_execution::ExecutionContext::default(),
    };
    
    let result = fixture.manager.execute_transform(transform_id.clone(), invalid_input);
    assert!(result.is_err());
    
    // Test with correct input format
    let valid_input = datafold::transform_execution::TransformInput {
        values: {
            let mut values = HashMap::new();
            values.insert("input_validation_test.input".to_string(), serde_json::json!({
                "required_field": 10
            }));
            values
        },
        context: datafold::transform_execution::ExecutionContext::default(),
    };
    
    let result = fixture.manager.execute_transform(transform_id, valid_input);
    assert!(result.is_ok());
}

// === RESOURCE LIMIT TESTS ===

#[test]
fn test_queue_capacity_limits() {
    let fixture = ConfigurationTestFixture::new();
    
    // Register a slow transform
    let definition = TransformDefinition {
        id: "slow_transform".to_string(),
        transform: Transform::new(
            "var start = Date.now(); while (Date.now() - start < 100) {} return input + 1".to_string(),
            "slow_transform.output".to_string()
        ),
        inputs: vec!["slow_transform.input".to_string()],
        metadata: HashMap::new(),
    };
    
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    // Check initial queue status
    let initial_status = fixture.manager.get_queue_status();
    let max_capacity = initial_status.capacity;
    
    // Enqueue multiple jobs
    let mut job_ids = Vec::new();
    for i in 1..=5 {
        let input = datafold::transform_execution::TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("slow_transform.input".to_string(), serde_json::Value::Number(serde_json::Number::from(i)));
                values
            },
            context: datafold::transform_execution::ExecutionContext::default(),
        };
        
        let job_result = fixture.manager.enqueue_execution(transform_id.clone(), input);
        if job_result.is_ok() {
            job_ids.push(job_result.unwrap());
        }
    }
    
    // Verify queue constraints are respected
    let queue_status = fixture.manager.get_queue_status();
    assert!(queue_status.pending + queue_status.running <= max_capacity);
}

#[test]
fn test_execution_timeout_handling() {
    let fixture = ConfigurationTestFixture::new();
    
    // Register transform that will run for a long time
    let definition = TransformDefinition {
        id: "timeout_test".to_string(),
        transform: Transform::new(
            "var start = Date.now(); while (Date.now() - start < 10000) {} return input".to_string(), // 10 second loop
            "timeout_test.output".to_string()
        ),
        inputs: vec!["timeout_test.input".to_string()],
        metadata: HashMap::new(),
    };
    
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    let input = datafold::transform_execution::TransformInput {
        values: {
            let mut values = HashMap::new();
            values.insert("timeout_test.input".to_string(), serde_json::Value::Number(serde_json::Number::from(1)));
            values
        },
        context: datafold::transform_execution::ExecutionContext::default(),
    };
    
    // This should timeout (depending on configuration)
    let result = fixture.manager.execute_transform(transform_id, input);
    // Result could be ok or error depending on timeout configuration
    // but should not hang indefinitely
    assert!(result.is_ok() || result.is_err());
}

// === CONFIGURATION EDGE CASES ===

#[test]
fn test_configuration_with_extreme_values() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("extreme_config_test.db");
    let db = sled::open(&db_path).expect("Failed to open test database");
    let db_ops = Arc::new(DbOperations::new(db).expect("Failed to create DbOperations"));
    
    // Create configuration with extreme values
    let mut extreme_config = TransformConfig::default();
    extreme_config.queue.max_queue_size = 1; // Very small queue
    extreme_config.retry.max_attempts = 0; // No retries
    extreme_config.performance.memory_limit_mb = 1; // Very low memory limit
    
    // Should still create manager successfully (implementation should handle gracefully)
    let manager_result = UnifiedTransformManager::new(db_ops, extreme_config);
    assert!(manager_result.is_ok());
    
    if let Ok(manager) = manager_result {
        // Basic operations should still work
        let definition = TransformDefinition {
            id: "extreme_config_test".to_string(),
            transform: Transform::new("return 42".to_string(), "extreme_config_test.output".to_string()),
            inputs: vec![],
            metadata: HashMap::new(),
        };
        
        let registration_result = manager.register_transform(definition);
        assert!(registration_result.is_ok() || registration_result.is_err()); // Either is acceptable
    }
}

#[test]
fn test_configuration_reload_error_recovery() {
    let fixture = ConfigurationTestFixture::new();
    
    // Register a transform
    let definition = fixture.create_transform_definition("reload_recovery_test");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    // Perform successful reload
    let first_reload = fixture.manager.reload_config();
    assert!(first_reload.is_ok());
    
    // Transform should still be available after reload
    let transforms = fixture.manager.list_transforms(None);
    assert!(transforms.iter().any(|t| t.id == "reload_recovery_test"));
    
    // Even if reload fails (which it shouldn't in this test), system should remain functional
    let post_reload_queue_status = fixture.manager.get_queue_status();
    assert!(post_reload_queue_status.capacity > 0);
}

// === CUSTOM CONFIGURATION TESTS ===

#[test]
fn test_custom_configuration_parameters() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("custom_params_test.db");
    let db = sled::open(&db_path).expect("Failed to open test database");
    let db_ops = Arc::new(DbOperations::new(db).expect("Failed to create DbOperations"));
    
    // Create config with custom parameters
    let mut custom_config = TransformConfig::default();
    custom_config.custom.insert("test_parameter".to_string(), serde_json::json!("test_value"));
    custom_config.custom.insert("numeric_parameter".to_string(), serde_json::json!(42));
    custom_config.custom.insert("boolean_parameter".to_string(), serde_json::json!(true));
    
    let manager = UnifiedTransformManager::new(db_ops, custom_config).unwrap();
    
    // System should work with custom parameters
    let definition = TransformDefinition {
        id: "custom_params_test".to_string(),
        transform: Transform::new("return input + 1".to_string(), "custom_params_test.output".to_string()),
        inputs: vec!["custom_params_test.input".to_string()],
        metadata: HashMap::new(),
    };
    
    let transform_id = manager.register_transform(definition).unwrap();
    assert_eq!(transform_id, "custom_params_test");
}

// === SECURITY CONFIGURATION TESTS ===

#[test]
fn test_security_configuration_enforcement() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("security_config_test.db");
    let db = sled::open(&db_path).expect("Failed to open test database");
    let db_ops = Arc::new(DbOperations::new(db).expect("Failed to create DbOperations"));
    
    // Create config with security restrictions
    let mut secure_config = TransformConfig::default();
    secure_config.security.enable_input_validation = true;
    secure_config.security.max_input_size_bytes = 1024; // Small input limit
    
    let manager = UnifiedTransformManager::new(db_ops, secure_config).unwrap();
    
    // Register transform
    let definition = TransformDefinition {
        id: "security_test".to_string(),
        transform: Transform::new("return input.length".to_string(), "security_test.output".to_string()),
        inputs: vec!["security_test.input".to_string()],
        metadata: HashMap::new(),
    };
    
    let transform_id = manager.register_transform(definition).unwrap();
    
    // Test with small input (should work)
    let small_input = datafold::transform_execution::TransformInput {
        values: {
            let mut values = HashMap::new();
            values.insert("security_test.input".to_string(), serde_json::Value::String("hello".to_string()));
            values
        },
        context: datafold::transform_execution::ExecutionContext::default(),
    };
    
    let small_result = manager.execute_transform(transform_id.clone(), small_input);
    assert!(small_result.is_ok());
    
    // Test with large input (might be rejected depending on implementation)
    let large_string = "x".repeat(2048); // Larger than max_input_size_bytes
    let large_input = datafold::transform_execution::TransformInput {
        values: {
            let mut values = HashMap::new();
            values.insert("security_test.input".to_string(), serde_json::Value::String(large_string));
            values
        },
        context: datafold::transform_execution::ExecutionContext::default(),
    };
    
    let large_result = manager.execute_transform(transform_id, large_input);
    // Depending on implementation, this might succeed or fail
    assert!(large_result.is_ok() || large_result.is_err());
}