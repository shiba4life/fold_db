//! Performance and load tests for the unified transform system.
//!
//! This module contains comprehensive performance tests that validate the unified
//! transform system under various load conditions, stress scenarios, and resource
//! constraints.

use datafold::db_operations::DbOperations;
use datafold::schema::types::Transform;
use datafold::transform_execution::{
    ExecutionContext, JobStatus, TransformConfig, TransformDefinition, TransformInput,
    UnifiedTransformManager,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime};
use tempfile::tempdir;

/// Performance test fixture with configurable parameters.
pub struct PerformanceTestFixture {
    pub manager: UnifiedTransformManager,
    pub db_ops: Arc<DbOperations>,
    _temp_dir: tempfile::TempDir,
}

impl PerformanceTestFixture {
    /// Creates a performance test fixture with optimized configuration.
    pub fn new() -> Self {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let db_path = temp_dir.path().join("performance_test.db");
        let db = sled::open(&db_path).expect("Failed to open test database");
        let db_ops = Arc::new(DbOperations::new(db).expect("Failed to create DbOperations"));
        
        // Performance-optimized configuration
        let mut config = TransformConfig::default();
        config.queue.max_queue_size = 1000;
        config.queue.max_retry_attempts = 1; // Reduced for performance
        config.performance.enable_metrics_collection = true;
        config.performance.enable_performance_monitoring = true;
        config.execution.enable_parallel_execution = true;
        config.execution.max_parallel_jobs = 20;
        
        let manager = UnifiedTransformManager::new(Arc::clone(&db_ops), config)
            .expect("Failed to create UnifiedTransformManager");

        Self {
            manager,
            db_ops,
            _temp_dir: temp_dir,
        }
    }

    /// Creates a lightweight transform for performance testing.
    pub fn create_lightweight_transform(&self, id: &str) -> TransformDefinition {
        TransformDefinition {
            id: id.to_string(),
            transform: Transform::new(
                "return { result: (input.value || 0) + 1, timestamp: Date.now() }".to_string(),
                format!("{}.output", id)
            ),
            inputs: vec![format!("{}.input", id)],
            metadata: HashMap::new(),
        }
    }

    /// Creates a compute-intensive transform for stress testing.
    pub fn create_compute_intensive_transform(&self, id: &str) -> TransformDefinition {
        TransformDefinition {
            id: id.to_string(),
            transform: Transform::new(
                r#"
                // Simulate computational work
                let iterations = input.complexity || 1000;
                let result = 0;
                for (let i = 0; i < iterations; i++) {
                    result += Math.sqrt(i * Math.PI);
                }
                return {
                    result: result,
                    iterations: iterations,
                    complexity_factor: input.complexity || 1000
                };
                "#.to_string(),
                format!("{}.output", id)
            ),
            inputs: vec![format!("{}.input", id)],
            metadata: HashMap::new(),
        }
    }

    /// Creates a memory-intensive transform for memory testing.
    pub fn create_memory_intensive_transform(&self, id: &str) -> TransformDefinition {
        TransformDefinition {
            id: id.to_string(),
            transform: Transform::new(
                r#"
                // Simulate memory-intensive operations
                let size = input.array_size || 1000;
                let data = [];
                for (let i = 0; i < size; i++) {
                    data.push({
                        id: i,
                        value: Math.random() * 1000,
                        metadata: {
                            timestamp: Date.now(),
                            processed: true,
                            iteration: i
                        }
                    });
                }
                
                // Process the data
                let processed = data.map(item => ({
                    ...item,
                    doubled_value: item.value * 2,
                    sqrt_value: Math.sqrt(item.value)
                }));
                
                return {
                    original_size: size,
                    processed_count: processed.length,
                    sample_data: processed.slice(0, 5), // Return sample to avoid huge outputs
                    memory_intensive: true
                };
                "#.to_string(),
                format!("{}.output", id)
            ),
            inputs: vec![format!("{}.input", id)],
            metadata: HashMap::new(),
        }
    }

    /// Waits for queue to be empty or timeout.
    pub fn wait_for_queue_completion(&self, timeout: Duration) -> bool {
        let start_time = Instant::now();
        
        loop {
            let queue_status = self.manager.get_queue_status();
            
            if queue_status.pending == 0 && queue_status.running == 0 {
                return true;
            }
            
            if start_time.elapsed() > timeout {
                return false;
            }
            
            thread::sleep(Duration::from_millis(50));
        }
    }
}

/// Performance metrics collection structure.
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_executions: usize,
    pub successful_executions: usize,
    pub failed_executions: usize,
    pub total_duration: Duration,
    pub average_execution_time: Duration,
    pub min_execution_time: Duration,
    pub max_execution_time: Duration,
    pub throughput_per_second: f64,
    pub memory_usage_estimate: usize,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            total_duration: Duration::from_millis(0),
            average_execution_time: Duration::from_millis(0),
            min_execution_time: Duration::from_secs(999),
            max_execution_time: Duration::from_millis(0),
            throughput_per_second: 0.0,
            memory_usage_estimate: 0,
        }
    }

    pub fn update(&mut self, execution_time: Duration, success: bool) {
        self.total_executions += 1;
        
        if success {
            self.successful_executions += 1;
        } else {
            self.failed_executions += 1;
        }
        
        self.total_duration += execution_time;
        
        if execution_time < self.min_execution_time {
            self.min_execution_time = execution_time;
        }
        
        if execution_time > self.max_execution_time {
            self.max_execution_time = execution_time;
        }
        
        if self.total_executions > 0 {
            self.average_execution_time = self.total_duration / self.total_executions as u32;
        }
        
        if self.total_duration.as_secs() > 0 {
            self.throughput_per_second = self.successful_executions as f64 / self.total_duration.as_secs_f64();
        }
    }

    pub fn finalize(&mut self, wall_clock_time: Duration) {
        if wall_clock_time.as_secs() > 0 {
            self.throughput_per_second = self.successful_executions as f64 / wall_clock_time.as_secs_f64();
        }
    }
}

// === THROUGHPUT PERFORMANCE TESTS ===

#[test]
fn test_sequential_execution_performance() {
    let fixture = PerformanceTestFixture::new();
    
    // Register lightweight transform
    let definition = fixture.create_lightweight_transform("sequential_perf_test");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    let execution_count = 100;
    let mut metrics = PerformanceMetrics::new();
    let overall_start = Instant::now();
    
    for i in 1..=execution_count {
        let input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("sequential_perf_test.input".to_string(), JsonValue::Number(serde_json::Number::from(i)));
                values
            },
            context: ExecutionContext {
                schema_name: "performance_test".to_string(),
                field_name: "sequential_output".to_string(),
                atom_ref: Some(format!("seq_atom_{}", i)),
                timestamp: SystemTime::now(),
                additional_data: HashMap::new(),
            },
        };
        
        let execution_start = Instant::now();
        let result = fixture.manager.execute_transform(transform_id.clone(), input);
        let execution_time = execution_start.elapsed();
        
        metrics.update(execution_time, result.is_ok());
    }
    
    let total_wall_time = overall_start.elapsed();
    metrics.finalize(total_wall_time);
    
    // Performance assertions
    assert_eq!(metrics.total_executions, execution_count);
    assert!(metrics.successful_executions >= execution_count * 95 / 100, 
            "Should have >95% success rate");
    assert!(metrics.average_execution_time < Duration::from_millis(100),
            "Average execution time should be < 100ms");
    assert!(metrics.throughput_per_second > 10.0,
            "Should achieve > 10 executions per second");
    
    println!("Sequential Performance Metrics: {:?}", metrics);
}

#[test]
fn test_concurrent_execution_performance() {
    let fixture = PerformanceTestFixture::new();
    
    // Register lightweight transform
    let definition = fixture.create_lightweight_transform("concurrent_perf_test");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    let thread_count = 10;
    let executions_per_thread = 50;
    let total_executions = thread_count * executions_per_thread;
    
    let manager = Arc::new(fixture.manager);
    let metrics = Arc::new(Mutex::new(PerformanceMetrics::new()));
    let barrier = Arc::new(Barrier::new(thread_count));
    
    let overall_start = Instant::now();
    let mut handles = Vec::new();
    
    for thread_id in 0..thread_count {
        let manager_clone = Arc::clone(&manager);
        let metrics_clone = Arc::clone(&metrics);
        let barrier_clone = Arc::clone(&barrier);
        let transform_id_clone = transform_id.clone();
        
        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            barrier_clone.wait();
            
            let thread_start = Instant::now();
            let mut thread_metrics = PerformanceMetrics::new();
            
            for i in 1..=executions_per_thread {
                let input = TransformInput {
                    values: {
                        let mut values = HashMap::new();
                        values.insert("concurrent_perf_test.input".to_string(), 
                                    JsonValue::Number(serde_json::Number::from(thread_id * executions_per_thread + i)));
                        values
                    },
                    context: ExecutionContext {
                        schema_name: "concurrent_performance_test".to_string(),
                        field_name: "concurrent_output".to_string(),
                        atom_ref: Some(format!("conc_atom_{}_{}", thread_id, i)),
                        timestamp: SystemTime::now(),
                        additional_data: HashMap::new(),
                    },
                };
                
                let execution_start = Instant::now();
                let result = manager_clone.execute_transform(transform_id_clone.clone(), input);
                let execution_time = execution_start.elapsed();
                
                thread_metrics.update(execution_time, result.is_ok());
            }
            
            // Merge thread metrics into global metrics
            let mut global_metrics = metrics_clone.lock().unwrap();
            global_metrics.total_executions += thread_metrics.total_executions;
            global_metrics.successful_executions += thread_metrics.successful_executions;
            global_metrics.failed_executions += thread_metrics.failed_executions;
            global_metrics.total_duration += thread_metrics.total_duration;
            
            if thread_metrics.min_execution_time < global_metrics.min_execution_time {
                global_metrics.min_execution_time = thread_metrics.min_execution_time;
            }
            
            if thread_metrics.max_execution_time > global_metrics.max_execution_time {
                global_metrics.max_execution_time = thread_metrics.max_execution_time;
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    let total_wall_time = overall_start.elapsed();
    
    let mut final_metrics = metrics.lock().unwrap();
    final_metrics.finalize(total_wall_time);
    
    if final_metrics.total_executions > 0 {
        final_metrics.average_execution_time = final_metrics.total_duration / final_metrics.total_executions as u32;
    }
    
    // Performance assertions for concurrent execution
    assert_eq!(final_metrics.total_executions, total_executions);
    assert!(final_metrics.successful_executions >= total_executions * 90 / 100,
            "Should have >90% success rate under concurrency");
    assert!(final_metrics.throughput_per_second > 50.0,
            "Should achieve > 50 executions per second with concurrency");
    
    println!("Concurrent Performance Metrics: {:?}", *final_metrics);
}

// === STRESS TESTS ===

#[test]
fn test_high_load_stress() {
    let fixture = PerformanceTestFixture::new();
    
    // Register compute-intensive transform
    let definition = fixture.create_compute_intensive_transform("stress_test");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    let execution_count = 20; // Smaller count for compute-intensive operations
    let mut metrics = PerformanceMetrics::new();
    let overall_start = Instant::now();
    
    for i in 1..=execution_count {
        let input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("stress_test.input".to_string(), serde_json::json!({
                    "complexity": 5000 + (i * 100) // Increasing complexity
                }));
                values
            },
            context: ExecutionContext {
                schema_name: "stress_test".to_string(),
                field_name: "stress_output".to_string(),
                atom_ref: Some(format!("stress_atom_{}", i)),
                timestamp: SystemTime::now(),
                additional_data: HashMap::new(),
            },
        };
        
        let execution_start = Instant::now();
        let result = fixture.manager.execute_transform(transform_id.clone(), input);
        let execution_time = execution_start.elapsed();
        
        metrics.update(execution_time, result.is_ok());
        
        // Verify result structure for compute-intensive transform
        if let Ok(output) = result {
            let result_obj = output.value.as_object().unwrap();
            assert!(result_obj.contains_key("result"));
            assert!(result_obj.contains_key("iterations"));
            assert!(result_obj.contains_key("complexity_factor"));
        }
    }
    
    let total_wall_time = overall_start.elapsed();
    metrics.finalize(total_wall_time);
    
    // Stress test assertions
    assert_eq!(metrics.total_executions, execution_count);
    assert!(metrics.successful_executions >= execution_count * 95 / 100,
            "Should maintain >95% success rate under computational stress");
    assert!(metrics.average_execution_time < Duration::from_secs(5),
            "Average execution time should be < 5s even under stress");
    
    println!("Stress Test Metrics: {:?}", metrics);
}

#[test]
fn test_memory_intensive_stress() {
    let fixture = PerformanceTestFixture::new();
    
    // Register memory-intensive transform
    let definition = fixture.create_memory_intensive_transform("memory_stress_test");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    let execution_count = 10; // Fewer executions for memory-intensive operations
    let mut metrics = PerformanceMetrics::new();
    let overall_start = Instant::now();
    
    for i in 1..=execution_count {
        let input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("memory_stress_test.input".to_string(), serde_json::json!({
                    "array_size": 10000 + (i * 1000) // Increasing memory usage
                }));
                values
            },
            context: ExecutionContext {
                schema_name: "memory_stress_test".to_string(),
                field_name: "memory_output".to_string(),
                atom_ref: Some(format!("memory_atom_{}", i)),
                timestamp: SystemTime::now(),
                additional_data: HashMap::new(),
            },
        };
        
        let execution_start = Instant::now();
        let result = fixture.manager.execute_transform(transform_id.clone(), input);
        let execution_time = execution_start.elapsed();
        
        metrics.update(execution_time, result.is_ok());
        
        // Verify result structure for memory-intensive transform
        if let Ok(output) = result {
            let result_obj = output.value.as_object().unwrap();
            assert!(result_obj.contains_key("original_size"));
            assert!(result_obj.contains_key("processed_count"));
            assert!(result_obj.contains_key("sample_data"));
            assert_eq!(result_obj["memory_intensive"], JsonValue::Bool(true));
        }
        
        // Small delay to allow memory cleanup
        thread::sleep(Duration::from_millis(100));
    }
    
    let total_wall_time = overall_start.elapsed();
    metrics.finalize(total_wall_time);
    
    // Memory stress test assertions
    assert_eq!(metrics.total_executions, execution_count);
    assert!(metrics.successful_executions >= execution_count * 90 / 100,
            "Should maintain >90% success rate under memory stress");
    
    println!("Memory Stress Test Metrics: {:?}", metrics);
}

// === QUEUE PERFORMANCE TESTS ===

#[test]
fn test_async_queue_performance() {
    let fixture = PerformanceTestFixture::new();
    
    // Register lightweight transform for queue testing
    let definition = fixture.create_lightweight_transform("queue_perf_test");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    let job_count = 100;
    let mut job_ids = Vec::new();
    let enqueue_start = Instant::now();
    
    // Enqueue jobs rapidly
    for i in 1..=job_count {
        let input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("queue_perf_test.input".to_string(), JsonValue::Number(serde_json::Number::from(i)));
                values
            },
            context: ExecutionContext {
                schema_name: "queue_performance_test".to_string(),
                field_name: "queue_output".to_string(),
                atom_ref: Some(format!("queue_atom_{}", i)),
                timestamp: SystemTime::now(),
                additional_data: HashMap::new(),
            },
        };
        
        let job_result = fixture.manager.enqueue_execution(transform_id.clone(), input);
        if let Ok(job_id) = job_result {
            job_ids.push(job_id);
        }
    }
    
    let enqueue_duration = enqueue_start.elapsed();
    
    // Wait for all jobs to complete
    let completion_start = Instant::now();
    let completed = fixture.wait_for_queue_completion(Duration::from_secs(30));
    let completion_duration = completion_start.elapsed();
    
    assert!(completed, "All queued jobs should complete within timeout");
    
    // Verify queue performance metrics
    let final_queue_status = fixture.manager.get_queue_status();
    assert_eq!(final_queue_status.pending, 0);
    assert_eq!(final_queue_status.running, 0);
    assert!(final_queue_status.completed >= job_count * 90 / 100);
    
    // Performance assertions
    assert!(enqueue_duration < Duration::from_secs(5),
            "Should enqueue {} jobs in < 5 seconds", job_count);
    assert!(completion_duration < Duration::from_secs(20),
            "Should complete {} jobs in < 20 seconds", job_count);
    
    let enqueue_rate = job_ids.len() as f64 / enqueue_duration.as_secs_f64();
    assert!(enqueue_rate > 20.0, "Should enqueue > 20 jobs per second");
    
    println!("Queue Performance: Enqueued {} jobs in {:?} ({:.2} jobs/sec)", 
             job_ids.len(), enqueue_duration, enqueue_rate);
    println!("Queue Completion: Completed in {:?}", completion_duration);
}

// === SCALABILITY TESTS ===

#[test]
fn test_transform_registration_scalability() {
    let fixture = PerformanceTestFixture::new();
    
    let transform_count = 100;
    let registration_start = Instant::now();
    
    // Register many transforms
    for i in 1..=transform_count {
        let definition = TransformDefinition {
            id: format!("scale_test_{}", i),
            transform: Transform::new(
                format!("return {{ id: '{}', result: input + {} }}", i, i),
                format!("scale_test_{}.output", i)
            ),
            inputs: vec![format!("scale_test_{}.input", i)],
            metadata: HashMap::new(),
        };
        
        let result = fixture.manager.register_transform(definition);
        assert!(result.is_ok(), "Transform registration {} should succeed", i);
    }
    
    let registration_duration = registration_start.elapsed();
    
    // Test listing performance with many transforms
    let list_start = Instant::now();
    let transforms = fixture.manager.list_transforms(None);
    let list_duration = list_start.elapsed();
    
    assert_eq!(transforms.len(), transform_count);
    
    // Performance assertions
    assert!(registration_duration < Duration::from_secs(10),
            "Should register {} transforms in < 10 seconds", transform_count);
    assert!(list_duration < Duration::from_secs(1),
            "Should list {} transforms in < 1 second", transform_count);
    
    let registration_rate = transform_count as f64 / registration_duration.as_secs_f64();
    assert!(registration_rate > 10.0, "Should register > 10 transforms per second");
    
    println!("Scalability: Registered {} transforms in {:?} ({:.2} reg/sec)", 
             transform_count, registration_duration, registration_rate);
    println!("List Performance: Listed {} transforms in {:?}", 
             transform_count, list_duration);
}

#[test]
fn test_execution_history_scalability() {
    let fixture = PerformanceTestFixture::new();
    
    // Register transform
    let definition = fixture.create_lightweight_transform("history_scale_test");
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    let execution_count = 200;
    
    // Execute many times to build up history
    for i in 1..=execution_count {
        let input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("history_scale_test.input".to_string(), JsonValue::Number(serde_json::Number::from(i)));
                values
            },
            context: ExecutionContext {
                schema_name: "history_scalability_test".to_string(),
                field_name: "history_output".to_string(),
                atom_ref: Some(format!("history_atom_{}", i)),
                timestamp: SystemTime::now(),
                additional_data: HashMap::new(),
            },
        };
        
        let _result = fixture.manager.execute_transform(transform_id.clone(), input).unwrap();
        
        // No delay - test rapid execution
    }
    
    // Test history retrieval performance with different limits
    let full_history_start = Instant::now();
    let full_history = fixture.manager.get_execution_history(transform_id.clone(), None).unwrap();
    let full_history_duration = full_history_start.elapsed();
    
    let limited_history_start = Instant::now();
    let limited_history = fixture.manager.get_execution_history(transform_id.clone(), Some(50)).unwrap();
    let limited_history_duration = limited_history_start.elapsed();
    
    // Verify history integrity
    assert!(full_history.len() <= execution_count);
    assert_eq!(limited_history.len(), 50);
    
    // Performance assertions
    assert!(full_history_duration < Duration::from_secs(2),
            "Should retrieve full history in < 2 seconds");
    assert!(limited_history_duration < Duration::from_millis(500),
            "Should retrieve limited history in < 500ms");
    
    println!("History Scalability: Full history ({} records) in {:?}", 
             full_history.len(), full_history_duration);
    println!("Limited history (50 records) in {:?}", limited_history_duration);
}

// === RESOURCE CLEANUP TESTS ===

#[test]
fn test_resource_cleanup_under_load() {
    let fixture = PerformanceTestFixture::new();
    
    // Register and remove transforms repeatedly
    let cycle_count = 20;
    let transforms_per_cycle = 10;
    
    for cycle in 1..=cycle_count {
        let cycle_start = Instant::now();
        let mut transform_ids = Vec::new();
        
        // Register transforms
        for i in 1..=transforms_per_cycle {
            let definition = TransformDefinition {
                id: format!("cleanup_test_{}_{}", cycle, i),
                transform: Transform::new(
                    "return { cycle: input.cycle, value: input.value + 1 }".to_string(),
                    format!("cleanup_test_{}_{}.output", cycle, i)
                ),
                inputs: vec![format!("cleanup_test_{}_{}.input", cycle, i)],
                metadata: HashMap::new(),
            };
            
            let transform_id = fixture.manager.register_transform(definition).unwrap();
            transform_ids.push(transform_id);
        }
        
        // Execute each transform
        for (i, transform_id) in transform_ids.iter().enumerate() {
            let input = TransformInput {
                values: {
                    let mut values = HashMap::new();
                    values.insert(format!("cleanup_test_{}_{}.input", cycle, i + 1), serde_json::json!({
                        "cycle": cycle,
                        "value": i + 1
                    }));
                    values
                },
                context: ExecutionContext {
                    schema_name: "cleanup_test".to_string(),
                    field_name: "cleanup_output".to_string(),
                    atom_ref: Some(format!("cleanup_atom_{}_{}", cycle, i + 1)),
                    timestamp: SystemTime::now(),
                    additional_data: HashMap::new(),
                },
            };
            
            let _result = fixture.manager.execute_transform(transform_id.clone(), input).unwrap();
        }
        
        // Remove all transforms
        for transform_id in transform_ids {
            let remove_result = fixture.manager.remove_transform(transform_id);
            assert!(remove_result.is_ok());
        }
        
        let cycle_duration = cycle_start.elapsed();
        assert!(cycle_duration < Duration::from_secs(5),
                "Cycle {} should complete in < 5 seconds", cycle);
        
        // Verify cleanup
        let transforms = fixture.manager.list_transforms(None);
        assert_eq!(transforms.len(), 0, "All transforms should be cleaned up after cycle {}", cycle);
    }
    
    println!("Resource cleanup test completed {} cycles successfully", cycle_count);
}

// === BASELINE PERFORMANCE BENCHMARKS ===

#[test]
fn test_baseline_performance_benchmark() {
    let fixture = PerformanceTestFixture::new();
    
    // Register simple benchmark transform
    let definition = TransformDefinition {
        id: "benchmark_transform".to_string(),
        transform: Transform::new(
            "return { input: input, processed_at: Date.now(), benchmark: true }".to_string(),
            "benchmark_transform.output".to_string()
        ),
        inputs: vec!["benchmark_transform.input".to_string()],
        metadata: HashMap::new(),
    };
    
    let transform_id = fixture.manager.register_transform(definition).unwrap();
    
    // Warm up the system
    for i in 1..=10 {
        let input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("benchmark_transform.input".to_string(), JsonValue::Number(serde_json::Number::from(i)));
                values
            },
            context: ExecutionContext::default(),
        };
        let _result = fixture.manager.execute_transform(transform_id.clone(), input).unwrap();
    }
    
    // Run benchmark
    let benchmark_count = 1000;
    let benchmark_start = Instant::now();
    let mut successful_executions = 0;
    
    for i in 1..=benchmark_count {
        let input = TransformInput {
            values: {
                let mut values = HashMap::new();
                values.insert("benchmark_transform.input".to_string(), JsonValue::Number(serde_json::Number::from(i)));
                values
            },
            context: ExecutionContext {
                schema_name: "benchmark_test".to_string(),
                field_name: "benchmark_output".to_string(),
                atom_ref: Some(format!("benchmark_atom_{}", i)),
                timestamp: SystemTime::now(),
                additional_data: HashMap::new(),
            },
        };
        
        let result = fixture.manager.execute_transform(transform_id.clone(), input);
        if result.is_ok() {
            successful_executions += 1;
        }
    }
    
    let benchmark_duration = benchmark_start.elapsed();
    let throughput = successful_executions as f64 / benchmark_duration.as_secs_f64();
    let avg_execution_time = benchmark_duration / successful_executions as u32;
    
    // Baseline performance assertions
    assert!(successful_executions >= benchmark_count * 99 / 100,
            "Should achieve >99% success rate in baseline benchmark");
    assert!(throughput > 100.0,
            "Should achieve > 100 executions per second in baseline");
    assert!(avg_execution_time < Duration::from_millis(10),
            "Average execution time should be < 10ms in baseline");
    
    println!("Baseline Benchmark Results:");
    println!("  Executions: {}/{}", successful_executions, benchmark_count);
    println!("  Duration: {:?}", benchmark_duration);
    println!("  Throughput: {:.2} executions/second", throughput);
    println!("  Average execution time: {:?}", avg_execution_time);
    
    // Store baseline for comparison in CI/CD
    assert!(throughput > 100.0, "Performance regression detected: throughput {} < 100.0", throughput);
}