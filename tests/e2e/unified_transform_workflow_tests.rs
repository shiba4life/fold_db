//! End-to-end tests for the unified transform system.
//!
//! This module contains comprehensive end-to-end tests that validate the entire
//! unified transform system from a user perspective, testing real-world scenarios
//! and complete workflows.

use datafold::db_operations::DbOperations;
use datafold::schema::types::Transform;
use datafold::transform_execution::{
    ExecutionContext, JobStatus, QueueStatus, TransformConfig, TransformDefinition,
    TransformInput, TransformOutput, TransformState, TransformStatus, TransformUpdate,
    UnifiedTransformManager,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};
use tempfile::tempdir;
use uuid::Uuid;

/// End-to-end test fixture providing realistic test scenarios.
pub struct E2ETestFixture {
    pub manager: UnifiedTransformManager,
    pub db_ops: Arc<DbOperations>,
    _temp_dir: tempfile::TempDir,
}

impl E2ETestFixture {
    /// Creates a new E2E test fixture with production-like configuration.
    pub fn new() -> Self {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let db_path = temp_dir.path().join("e2e_test.db");
        let db = sled::open(&db_path).expect("Failed to open test database");
        let db_ops = Arc::new(DbOperations::new(db).expect("Failed to create DbOperations"));
        
        // Use production-like configuration
        let mut config = TransformConfig::default();
        config.queue.max_queue_size = 100;
        config.retry.max_attempts = 3;
        config.performance.enable_metrics_collection = true;
        
        let manager = UnifiedTransformManager::new(Arc::clone(&db_ops), config)
            .expect("Failed to create UnifiedTransformManager");

        Self {
            manager,
            db_ops,
            _temp_dir: temp_dir,
        }
    }

    /// Waits for all queued jobs to complete.
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

    /// Creates a realistic transform definition for data processing.
    pub fn create_data_processing_transform(&self, id: &str) -> TransformDefinition {
        TransformDefinition {
            id: id.to_string(),
            transform: Transform::new(
                format!(
                    r#"
                    // Data processing transform for {}
                    if (!input || typeof input !== 'object') {{
                        throw new Error('Invalid input: expected object');
                    }}
                    
                    let result = {{}};
                    result.processed_at = new Date().toISOString();
                    result.transform_id = '{}';
                    
                    if (input.data && Array.isArray(input.data)) {{
                        result.count = input.data.length;
                        result.sum = input.data.reduce((a, b) => a + (typeof b === 'number' ? b : 0), 0);
                        result.average = result.count > 0 ? result.sum / result.count : 0;
                    }} else {{
                        result.count = 0;
                        result.sum = 0;
                        result.average = 0;
                    }}
                    
                    result.metadata = {{
                        version: '1.0',
                        processor: 'unified_transform_system'
                    }};
                    
                    return result;
                    "#,
                    id, id
                ),
                format!("{}.output", id)
            ),
            inputs: vec![format!("{}.input", id)],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("type".to_string(), "data_processing".to_string());
                meta.insert("version".to_string(), "1.0".to_string());
                meta.insert("created_by".to_string(), "e2e_test".to_string());
                meta
            },
        }
    }

    /// Creates test data for processing.
    pub fn create_test_data(&self, size: usize) -> JsonValue {
        let data: Vec<i32> = (1..=size as i32).collect();
        serde_json::json!({
            "data": data,
            "source": "e2e_test",
            "timestamp": SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "metadata": {
                "test_case": "data_processing",
                "size": size
            }
        })
    }
}

// === REAL-WORLD WORKFLOW TESTS ===

#[test]
fn test_complete_data_processing_workflow() {
    let fixture = E2ETestFixture::new();
    
    // Step 1: Register data processing transforms
    let preprocessor_def = fixture.create_data_processing_transform("data_preprocessor");
    let analyzer_def = fixture.create_data_processing_transform("data_analyzer");
    let reporter_def = fixture.create_data_processing_transform("data_reporter");
    
    let preprocessor_id = fixture.manager.register_transform(preprocessor_def).unwrap();
    let analyzer_id = fixture.manager.register_transform(analyzer_def).unwrap();
    let reporter_id = fixture.manager.register_transform(reporter_def).unwrap();
    
    // Step 2: Verify all transforms are registered
    let transforms = fixture.manager.list_transforms(None);
    assert_eq!(transforms.len(), 3);
    
    let transform_names: Vec<String> = transforms.iter().map(|t| t.id.clone()).collect();
    assert!(transform_names.contains(&"data_preprocessor".to_string()));
    assert!(transform_names.contains(&"data_analyzer".to_string()));
    assert!(transform_names.contains(&"data_reporter".to_string()));
    
    // Step 3: Process data through the pipeline
    let test_data = fixture.create_test_data(100);
    
    // Preprocess data
    let preprocess_input = TransformInput {
        values: {
            let mut values = HashMap::new();
            values.insert("data_preprocessor.input".to_string(), test_data.clone());
            values
        },
        context: ExecutionContext {
            schema_name: "e2e_data_processing".to_string(),
            field_name: "preprocessed_data".to_string(),
            atom_ref: Some(format!("atom_ref_{}", Uuid::new_v4())),
            timestamp: SystemTime::now(),
            additional_data: HashMap::new(),
        },
    };
    
    let preprocess_result = fixture.manager.execute_transform(preprocessor_id.clone(), preprocess_input).unwrap();
    assert!(preprocess_result.value.is_object());
    
    // Analyze preprocessed data
    let analyze_input = TransformInput {
        values: {
            let mut values = HashMap::new();
            values.insert("data_analyzer.input".to_string(), test_data.clone());
            values
        },
        context: ExecutionContext {
            schema_name: "e2e_data_processing".to_string(),
            field_name: "analyzed_data".to_string(),
            atom_ref: Some(format!("atom_ref_{}", Uuid::new_v4())),
            timestamp: SystemTime::now(),
            additional_data: HashMap::new(),
        },
    };
    
    let analyze_result = fixture.manager.execute_transform(analyzer_id.clone(), analyze_input).unwrap();
    assert!(analyze_result.value.is_object());
    
    // Generate report
    let report_input = TransformInput {
        values: {
            let mut values = HashMap::new();
            values.insert("data_reporter.input".to_string(), test_data);
            values
        },
        context: ExecutionContext {
            schema_name: "e2e_data_processing".to_string(),
            field_name: "data_report".to_string(),
            atom_ref: Some(format!("atom_ref_{}", Uuid::new_v4())),
            timestamp: SystemTime::now(),
            additional_data: HashMap::new(),
        },
    };
    
    let report_result = fixture.manager.execute_transform(reporter_id, report_input).unwrap();
    assert!(report_result.value.is_object());
    
    // Step 4: Verify results contain expected fields
    let result_obj = report_result.value.as_object().unwrap();
    assert!(result_obj.contains_key("count"));
    assert!(result_obj.contains_key("sum"));
    assert!(result_obj.contains_key("average"));
    assert!(result_obj.contains_key("processed_at"));
    assert!(result_obj.contains_key("metadata"));
    
    // Verify calculated values
    assert_eq!(result_obj["count"], JsonValue::Number(serde_json::Number::from(100)));
    assert_eq!(result_obj["sum"], JsonValue::Number(serde_json::Number::from(5050))); // 1+2+...+100 = 5050
    assert_eq!(result_obj["average"], JsonValue::Number(serde_json::Number::from(50))); // 5050/100 = 50.5, but JSON numbers...
    
    // Step 5: Verify execution history
    let preprocess_history = fixture.manager.get_execution_history(preprocessor_id, None).unwrap();
    assert_eq!(preprocess_history.len(), 1);
    assert_eq!(preprocess_history[0].status, JobStatus::Completed);
    
    let analyze_history = fixture.manager.get_execution_history(analyzer_id, None).unwrap();
    assert_eq!(analyze_history.len(), 1);
    assert_eq!(analyze_history[0].status, JobStatus::Completed);
}

#[test]
fn test_batch_processing_workflow() {
    let fixture = E2ETestFixture::new();
    
    // Register batch processor
    let batch_def = TransformDefinition {
        id: "batch_processor".to_string(),
        transform: Transform::new(
            r#"
            if (!input.batch_id || !input.items) {
                throw new Error('Invalid batch: missing batch_id or items');
            }
            
            let processed_items = input.items.map((item, index) => ({
                id: item.id || index,
                original_value: item.value,
                processed_value: (item.value || 0) * 2,
                processed_at: new Date().toISOString(),
                batch_id: input.batch_id
            }));
            
            return {
                batch_id: input.batch_id,
                total_items: processed_items.length,
                processed_items: processed_items,
                processing_summary: {
                    successful: processed_items.length,
                    failed: 0,
                    total_processing_time_ms: new Date().getTime() % 1000
                }
            };
            "#.to_string(),
            "batch_processor.output".to_string()
        ),
        inputs: vec!["batch_processor.input".to_string()],
        metadata: HashMap::new(),
    };
    
    let batch_id = fixture.manager.register_transform(batch_def).unwrap();
    
    // Process multiple batches
    let batch_count = 5;
    let mut batch_results = Vec::new();
    
    for batch_num in 1..=batch_count {
        let batch_input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("batch_processor.input".to_string(), serde_json::json!({
                    "batch_id": format!("batch_{}", batch_num),
                    "items": [
                        {"id": format!("item_{}_{}", batch_num, 1), "value": batch_num * 10},
                        {"id": format!("item_{}_{}", batch_num, 2), "value": batch_num * 20},
                        {"id": format!("item_{}_{}", batch_num, 3), "value": batch_num * 30},
                    ]
                }));
                values
            },
            context: ExecutionContext {
                schema_name: "e2e_batch_processing".to_string(),
                field_name: "batch_output".to_string(),
                atom_ref: Some(format!("batch_atom_{}", batch_num)),
                timestamp: SystemTime::now(),
                additional_data: {
                    let mut data = HashMap::new();
                    data.insert("batch_number".to_string(), batch_num.to_string());
                    data
                },
            },
        };
        
        let result = fixture.manager.execute_transform(batch_id.clone(), batch_input).unwrap();
        batch_results.push(result);
        
        // Small delay between batches
        thread::sleep(Duration::from_millis(10));
    }
    
    // Verify all batches processed successfully
    assert_eq!(batch_results.len(), batch_count);
    
    for (i, result) in batch_results.iter().enumerate() {
        let result_obj = result.value.as_object().unwrap();
        assert_eq!(result_obj["batch_id"], JsonValue::String(format!("batch_{}", i + 1)));
        assert_eq!(result_obj["total_items"], JsonValue::Number(serde_json::Number::from(3)));
        
        let processed_items = result_obj["processed_items"].as_array().unwrap();
        assert_eq!(processed_items.len(), 3);
        
        // Verify each item was processed correctly (value doubled)
        for (j, item) in processed_items.iter().enumerate() {
            let item_obj = item.as_object().unwrap();
            let original_value = ((i + 1) * (j + 1) * 10) as i64;
            let expected_processed = original_value * 2;
            
            assert_eq!(item_obj["original_value"], JsonValue::Number(serde_json::Number::from(original_value)));
            assert_eq!(item_obj["processed_value"], JsonValue::Number(serde_json::Number::from(expected_processed)));
        }
    }
    
    // Verify execution history
    let history = fixture.manager.get_execution_history(batch_id, None).unwrap();
    assert_eq!(history.len(), batch_count);
    
    // All executions should be successful
    for record in history {
        assert_eq!(record.status, JobStatus::Completed);
        assert!(record.result.is_some());
    }
}

#[test]
fn test_async_workflow_with_dependencies() {
    let fixture = E2ETestFixture::new();
    
    // Register multiple interdependent transforms
    let data_fetcher_def = TransformDefinition {
        id: "data_fetcher".to_string(),
        transform: Transform::new(
            r#"
            // Simulate data fetching
            return {
                fetched_data: input.query ? input.query.toUpperCase() + "_DATA" : "DEFAULT_DATA",
                fetch_timestamp: new Date().toISOString(),
                status: "success"
            };
            "#.to_string(),
            "data_fetcher.output".to_string()
        ),
        inputs: vec!["data_fetcher.input".to_string()],
        metadata: HashMap::new(),
    };
    
    let data_validator_def = TransformDefinition {
        id: "data_validator".to_string(),
        transform: Transform::new(
            r#"
            if (!input.fetched_data) {
                throw new Error('No data to validate');
            }
            
            return {
                original_data: input.fetched_data,
                is_valid: input.fetched_data.includes("_DATA"),
                validation_timestamp: new Date().toISOString(),
                validation_rules_applied: ["format_check", "content_check"]
            };
            "#.to_string(),
            "data_validator.output".to_string()
        ),
        inputs: vec!["data_validator.input".to_string()],
        metadata: HashMap::new(),
    };
    
    let data_processor_def = TransformDefinition {
        id: "data_processor".to_string(),
        transform: Transform::new(
            r#"
            if (!input.is_valid) {
                throw new Error('Cannot process invalid data');
            }
            
            return {
                processed_data: input.original_data + "_PROCESSED",
                processing_timestamp: new Date().toISOString(),
                processing_steps: ["validation", "transformation", "enhancement"]
            };
            "#.to_string(),
            "data_processor.output".to_string()
        ),
        inputs: vec!["data_processor.input".to_string()],
        metadata: HashMap::new(),
    };
    
    // Register all transforms
    let fetcher_id = fixture.manager.register_transform(data_fetcher_def).unwrap();
    let validator_id = fixture.manager.register_transform(data_validator_def).unwrap();
    let processor_id = fixture.manager.register_transform(data_processor_def).unwrap();
    
    // Execute workflow asynchronously
    let mut job_ids = Vec::new();
    
    // Step 1: Fetch data (can run independently)
    for i in 1..=3 {
        let fetch_input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("data_fetcher.input".to_string(), serde_json::json!({
                    "query": format!("query_{}", i)
                }));
                values
            },
            context: ExecutionContext {
                schema_name: "async_workflow".to_string(),
                field_name: "fetched_data".to_string(),
                atom_ref: Some(format!("fetch_atom_{}", i)),
                timestamp: SystemTime::now(),
                additional_data: HashMap::new(),
            },
        };
        
        let job_id = fixture.manager.enqueue_execution(fetcher_id.clone(), fetch_input).unwrap();
        job_ids.push(job_id);
    }
    
    // Wait for fetch jobs to complete
    let fetch_completed = fixture.wait_for_queue_completion(Duration::from_secs(10));
    assert!(fetch_completed, "Fetch jobs should complete within timeout");
    
    // Step 2: Validate fetched data (depends on fetch results)
    // In a real system, this would use the fetch results as input
    // For this test, we'll simulate with known data
    for i in 1..=3 {
        let validate_input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("data_validator.input".to_string(), serde_json::json!({
                    "fetched_data": format!("QUERY_{}_DATA", i)
                }));
                values
            },
            context: ExecutionContext {
                schema_name: "async_workflow".to_string(),
                field_name: "validated_data".to_string(),
                atom_ref: Some(format!("validate_atom_{}", i)),
                timestamp: SystemTime::now(),
                additional_data: HashMap::new(),
            },
        };
        
        let job_id = fixture.manager.enqueue_execution(validator_id.clone(), validate_input).unwrap();
        job_ids.push(job_id);
    }
    
    // Wait for validation jobs to complete
    let validate_completed = fixture.wait_for_queue_completion(Duration::from_secs(10));
    assert!(validate_completed, "Validation jobs should complete within timeout");
    
    // Step 3: Process validated data (depends on validation results)
    for i in 1..=3 {
        let process_input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("data_processor.input".to_string(), serde_json::json!({
                    "original_data": format!("QUERY_{}_DATA", i),
                    "is_valid": true
                }));
                values
            },
            context: ExecutionContext {
                schema_name: "async_workflow".to_string(),
                field_name: "processed_data".to_string(),
                atom_ref: Some(format!("process_atom_{}", i)),
                timestamp: SystemTime::now(),
                additional_data: HashMap::new(),
            },
        };
        
        let job_id = fixture.manager.enqueue_execution(processor_id.clone(), process_input).unwrap();
        job_ids.push(job_id);
    }
    
    // Wait for all processing jobs to complete
    let process_completed = fixture.wait_for_queue_completion(Duration::from_secs(10));
    assert!(process_completed, "Processing jobs should complete within timeout");
    
    // Verify final queue status
    let final_queue_status = fixture.manager.get_queue_status();
    assert_eq!(final_queue_status.pending, 0);
    assert_eq!(final_queue_status.running, 0);
    assert_eq!(final_queue_status.completed, 9); // 3 + 3 + 3 jobs
    
    // Verify execution histories
    let fetcher_history = fixture.manager.get_execution_history(fetcher_id, None).unwrap();
    let validator_history = fixture.manager.get_execution_history(validator_id, None).unwrap();
    let processor_history = fixture.manager.get_execution_history(processor_id, None).unwrap();
    
    assert_eq!(fetcher_history.len(), 3);
    assert_eq!(validator_history.len(), 3);
    assert_eq!(processor_history.len(), 3);
    
    // All executions should be successful
    for record in fetcher_history.iter().chain(validator_history.iter()).chain(processor_history.iter()) {
        assert_eq!(record.status, JobStatus::Completed);
    }
}

// === ERROR RECOVERY WORKFLOW TESTS ===

#[test]
fn test_error_recovery_workflow() {
    let fixture = E2ETestFixture::new();
    
    // Register transform that fails under certain conditions
    let resilient_transform_def = TransformDefinition {
        id: "resilient_processor".to_string(),
        transform: Transform::new(
            r#"
            if (!input.data) {
                throw new Error('Missing required data field');
            }
            
            if (input.data.error_trigger) {
                throw new Error('Simulated processing error');
            }
            
            if (input.data.value < 0) {
                throw new Error('Negative values not supported');
            }
            
            return {
                processed_value: input.data.value * 2,
                processing_status: "success",
                timestamp: new Date().toISOString()
            };
            "#.to_string(),
            "resilient_processor.output".to_string()
        ),
        inputs: vec!["resilient_processor.input".to_string()],
        metadata: HashMap::new(),
    };
    
    let transform_id = fixture.manager.register_transform(resilient_transform_def).unwrap();
    
    // Test data with mix of valid and invalid inputs
    let test_cases = vec![
        (serde_json::json!({"data": {"value": 10}}), true),           // Should succeed
        (serde_json::json!({"data": {"error_trigger": true}}), false), // Should fail
        (serde_json::json!({"data": {"value": 20}}), true),           // Should succeed
        (serde_json::json!({"data": {"value": -5}}), false),          // Should fail (negative)
        (serde_json::json!({"missing_data": true}), false),           // Should fail (missing data)
        (serde_json::json!({"data": {"value": 30}}), true),           // Should succeed
    ];
    
    let mut execution_results = Vec::new();
    
    for (i, (test_input, should_succeed)) in test_cases.iter().enumerate() {
        let input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("resilient_processor.input".to_string(), test_input.clone());
                values
            },
            context: ExecutionContext {
                schema_name: "error_recovery_test".to_string(),
                field_name: "test_output".to_string(),
                atom_ref: Some(format!("error_test_atom_{}", i)),
                timestamp: SystemTime::now(),
                additional_data: HashMap::new(),
            },
        };
        
        let result = fixture.manager.execute_transform(transform_id.clone(), input);
        execution_results.push((result, *should_succeed));
        
        thread::sleep(Duration::from_millis(10));
    }
    
    // Verify results match expectations
    for (result, should_succeed) in execution_results {
        if should_succeed {
            assert!(result.is_ok(), "Expected successful execution");
        } else {
            assert!(result.is_err(), "Expected failed execution");
        }
    }
    
    // Verify execution history reflects both successes and failures
    let history = fixture.manager.get_execution_history(transform_id.clone(), None).unwrap();
    assert_eq!(history.len(), 6);
    
    let success_count = history.iter().filter(|r| r.status == JobStatus::Completed).count();
    let failure_count = history.iter().filter(|r| r.status == JobStatus::Failed).count();
    
    assert_eq!(success_count, 3);
    assert_eq!(failure_count, 3);
    
    // Verify state reflects the mixed results
    let state = fixture.manager.get_transform_state(transform_id).unwrap();
    assert_eq!(state.success_count, 3);
    assert_eq!(state.failure_count, 3);
    assert!(state.last_error.is_some()); // Last execution was a failure
}

// === SYSTEM STRESS WORKFLOW TESTS ===

#[test]
fn test_high_load_workflow() {
    let fixture = E2ETestFixture::new();
    
    // Register high-throughput transform
    let throughput_transform_def = TransformDefinition {
        id: "throughput_processor".to_string(),
        transform: Transform::new(
            r#"
            return {
                input_id: input.id,
                processed_value: (input.value || 0) + 1,
                batch_info: {
                    size: input.batch_size || 1,
                    timestamp: new Date().toISOString()
                }
            };
            "#.to_string(),
            "throughput_processor.output".to_string()
        ),
        inputs: vec!["throughput_processor.input".to_string()],
        metadata: HashMap::new(),
    };
    
    let transform_id = fixture.manager.register_transform(throughput_transform_def).unwrap();
    
    // Execute many transforms rapidly
    let execution_count = 50;
    let mut successful_executions = 0;
    
    for i in 1..=execution_count {
        let input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("throughput_processor.input".to_string(), serde_json::json!({
                    "id": format!("load_test_{}", i),
                    "value": i,
                    "batch_size": execution_count
                }));
                values
            },
            context: ExecutionContext {
                schema_name: "high_load_test".to_string(),
                field_name: "load_output".to_string(),
                atom_ref: Some(format!("load_atom_{}", i)),
                timestamp: SystemTime::now(),
                additional_data: HashMap::new(),
            },
        };
        
        let result = fixture.manager.execute_transform(transform_id.clone(), input);
        if result.is_ok() {
            successful_executions += 1;
        }
        
        // No delay - test system under rapid fire
    }
    
    // Should handle high load gracefully
    assert!(successful_executions >= execution_count * 8 / 10, 
            "Should successfully execute at least 80% of requests under load");
    
    // Verify system state after high load
    let final_state = fixture.manager.get_transform_state(transform_id.clone()).unwrap();
    assert_eq!(final_state.success_count as usize, successful_executions);
    
    let history = fixture.manager.get_execution_history(transform_id, Some(execution_count)).unwrap();
    assert!(history.len() <= execution_count);
    
    // Queue should be manageable
    let queue_status = fixture.manager.get_queue_status();
    assert!(queue_status.pending <= queue_status.capacity);
}

// === SYSTEM UPGRADE SIMULATION TESTS ===

#[test]
fn test_system_upgrade_workflow() {
    let fixture = E2ETestFixture::new();
    
    // Register initial version of transform
    let v1_transform_def = TransformDefinition {
        id: "versioned_processor".to_string(),
        transform: Transform::new(
            r#"
            return {
                version: "1.0",
                result: input.value * 2,
                features: ["basic_processing"]
            };
            "#.to_string(),
            "versioned_processor.output".to_string()
        ),
        inputs: vec!["versioned_processor.input".to_string()],
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("version".to_string(), "1.0".to_string());
            meta
        },
    };
    
    let transform_id = fixture.manager.register_transform(v1_transform_def).unwrap();
    
    // Execute with v1
    let v1_input = TransformInput {
        values: {
            let mut values = HashMap::new();
            values.insert("versioned_processor.input".to_string(), serde_json::json!({
                "value": 10
            }));
            values
        },
        context: ExecutionContext {
            schema_name: "upgrade_test".to_string(),
            field_name: "v1_output".to_string(),
            atom_ref: Some("upgrade_atom_v1".to_string()),
            timestamp: SystemTime::now(),
            additional_data: HashMap::new(),
        },
    };
    
    let v1_result = fixture.manager.execute_transform(transform_id.clone(), v1_input).unwrap();
    let v1_output = v1_result.value.as_object().unwrap();
    assert_eq!(v1_output["version"], JsonValue::String("1.0".to_string()));
    assert_eq!(v1_output["result"], JsonValue::Number(serde_json::Number::from(20)));
    
    // Simulate system upgrade by updating the transform
    let v2_update = TransformUpdate {
        transform: Some(Transform::new(
            r#"
            return {
                version: "2.0",
                result: input.value * 3, // Enhanced processing
                enhanced_result: Math.pow(input.value, 2),
                features: ["basic_processing", "enhanced_math", "power_operations"]
            };
            "#.to_string(),
            "versioned_processor.output".to_string()
        )),
        inputs: None,
        metadata: Some({
            let mut meta = HashMap::new();
            meta.insert("version".to_string(), "2.0".to_string());
            meta.insert("upgrade_date".to_string(), SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string());
            meta
        }),
        status: None,
    };
    
    fixture.manager.update_transform(transform_id.clone(), v2_update).unwrap();
    
    // Execute with v2
    let v2_input = TransformInput {
        values: {
            let mut values = HashMap::new();
            values.insert("versioned_processor.input".to_string(), serde_json::json!({
                "value": 10
            }));
            values
        },
        context: ExecutionContext {
            schema_name: "upgrade_test".to_string(),
            field_name: "v2_output".to_string(),
            atom_ref: Some("upgrade_atom_v2".to_string()),
            timestamp: SystemTime::now(),
            additional_data: HashMap::new(),
        },
    };
    
    let v2_result = fixture.manager.execute_transform(transform_id.clone(), v2_input).unwrap();
    let v2_output = v2_result.value.as_object().unwrap();
    assert_eq!(v2_output["version"], JsonValue::String("2.0".to_string()));
    assert_eq!(v2_output["result"], JsonValue::Number(serde_json::Number::from(30))); // 10 * 3
    assert_eq!(v2_output["enhanced_result"], JsonValue::Number(serde_json::Number::from(100))); // 10^2
    
    // Verify execution history shows both versions
    let history = fixture.manager.get_execution_history(transform_id, None).unwrap();
    assert_eq!(history.len(), 2);
    
    // Both executions should be successful
    for record in history {
        assert_eq!(record.status, JobStatus::Completed);
    }
    
    // Verify final state
    let final_state = fixture.manager.get_transform_state(transform_id).unwrap();
    assert_eq!(final_state.success_count, 2);
    assert_eq!(final_state.failure_count, 0);
}