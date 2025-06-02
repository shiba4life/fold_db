//! Performance monitoring examples for DataFold logging system
//! 
//! This file demonstrates advanced performance monitoring techniques,
//! metrics collection, and optimization strategies using the logging system.

use fold_node::logging::{LoggingSystem, config::LogConfig};
use fold_node::logging::features::{PerformanceTimer, LogFeature};
use fold_node::{log_transform_info, log_network_info, log_http_info};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;

/// Performance metrics collector
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub operation_counts: HashMap<String, u64>,
    pub operation_durations: HashMap<String, Vec<Duration>>,
    pub error_counts: HashMap<String, u64>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            operation_counts: HashMap::new(),
            operation_durations: HashMap::new(),
            error_counts: HashMap::new(),
        }
    }

    pub fn record_operation(&mut self, operation: &str, duration: Duration) {
        *self.operation_counts.entry(operation.to_string()).or_insert(0) += 1;
        self.operation_durations
            .entry(operation.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
    }

    pub fn record_error(&mut self, operation: &str) {
        *self.error_counts.entry(operation.to_string()).or_insert(0) += 1;
    }

    pub fn get_average_duration(&self, operation: &str) -> Option<Duration> {
        self.operation_durations.get(operation).map(|durations| {
            let total: Duration = durations.iter().sum();
            total / durations.len() as u32
        })
    }

    pub fn get_percentile(&self, operation: &str, percentile: f64) -> Option<Duration> {
        self.operation_durations.get(operation).map(|durations| {
            let mut sorted_durations = durations.clone();
            sorted_durations.sort();
            let index = ((sorted_durations.len() as f64 - 1.0) * percentile / 100.0) as usize;
            sorted_durations[index]
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging system with performance-focused configuration
    setup_performance_logging().await?;
    
    println!("Running DataFold performance monitoring examples...\n");
    
    // Performance monitoring examples
    basic_performance_monitoring().await?;
    advanced_timing_techniques().await?;
    batch_operation_monitoring().await?;
    memory_and_resource_monitoring().await?;
    distributed_operation_tracking().await?;
    performance_analysis().await?;
    
    println!("\nPerformance monitoring examples completed!");
    Ok(())
}

/// Set up logging configuration optimized for performance monitoring
async fn setup_performance_logging() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = LogConfig::default();
    
    // Configure for performance monitoring
    config.general.default_level = "INFO".to_string();
    config.general.enable_correlation_ids = true;
    
    // Enable structured logging for metrics
    config.outputs.structured.enabled = true;
    config.outputs.structured.level = "INFO".to_string();
    config.outputs.structured.path = Some("examples/logs/performance-metrics.json".to_string());
    config.outputs.structured.include_metrics = true;
    config.outputs.structured.include_context = true;
    
    // Console for real-time monitoring
    config.outputs.console.enabled = true;
    config.outputs.console.level = "INFO".to_string();
    
    // File for historical analysis
    config.outputs.file.enabled = true;
    config.outputs.file.path = "examples/logs/performance.log".to_string();
    config.outputs.file.level = "DEBUG".to_string();
    
    LoggingSystem::init_with_config(config).await?;
    
    log::info!("Performance monitoring logging initialized");
    Ok(())
}

/// Demonstrate basic performance monitoring techniques
async fn basic_performance_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Performance Monitoring ===");
    
    // Simple operation timing
    simple_operation_timing().await;
    
    // Function-level performance monitoring
    function_performance_monitoring().await;
    
    // Resource usage monitoring
    resource_usage_monitoring().await;
    
    Ok(())
}

async fn simple_operation_timing() {
    println!("--- Simple Operation Timing ---");
    
    // Using PerformanceTimer for automatic logging
    let timer = PerformanceTimer::new(
        LogFeature::Transform,
        "data_validation".to_string()
    );
    
    // Simulate data validation work
    sleep(Duration::from_millis(50)).await;
    
    timer.finish(); // Automatically logs completion time
    
    // Manual timing with custom metrics
    let start = Instant::now();
    
    // Simulate complex calculation
    sleep(Duration::from_millis(75)).await;
    
    let duration = start.elapsed();
    log_transform_info!(
        "Complex calculation completed",
        operation = "score_calculation",
        duration_ms = duration.as_millis(),
        complexity_factor = 2.5,
        input_size = 1000,
        cache_hits = 150,
        cache_misses = 25
    );
}

async fn function_performance_monitoring() {
    println!("--- Function-Level Performance Monitoring ---");
    
    // Monitor database operation
    let db_timer = Instant::now();
    
    // Simulate database query
    sleep(Duration::from_millis(30)).await;
    
    let db_duration = db_timer.elapsed();
    log::info!(
        target: "datafold_node::database",
        "Database query executed",
        operation = "user_lookup",
        query_type = "indexed_search",
        duration_ms = db_duration.as_millis(),
        rows_examined = 50000,
        rows_returned = 125,
        index_used = "user_email_idx",
        query_plan_cost = 145.7
    );
    
    // Monitor network operation
    let net_timer = Instant::now();
    
    // Simulate network request
    sleep(Duration::from_millis(100)).await;
    
    let net_duration = net_timer.elapsed();
    log_network_info!(
        "Network request completed",
        operation = "peer_sync",
        duration_ms = net_duration.as_millis(),
        bytes_sent = 2048,
        bytes_received = 4096,
        peer_count = 5,
        success_rate = 1.0
    );
    
    // Monitor HTTP request processing
    let http_timer = Instant::now();
    
    // Simulate request processing
    sleep(Duration::from_millis(25)).await;
    
    let http_duration = http_timer.elapsed();
    log_http_info!(
        "HTTP request processed",
        method = "POST",
        path = "/api/schemas",
        duration_ms = http_duration.as_millis(),
        status_code = 201,
        request_size_bytes = 1024,
        response_size_bytes = 256,
        db_queries = 2,
        cache_lookups = 5
    );
}

async fn resource_usage_monitoring() {
    println!("--- Resource Usage Monitoring ---");
    
    // Simulate memory-intensive operation
    let memory_timer = Instant::now();
    let initial_memory = get_memory_usage(); // Simulated function
    
    // Simulate memory allocation
    sleep(Duration::from_millis(40)).await;
    
    let final_memory = get_memory_usage();
    let duration = memory_timer.elapsed();
    
    log::info!(
        target: "datafold_node::performance",
        "Memory-intensive operation completed",
        operation = "schema_loading",
        duration_ms = duration.as_millis(),
        memory_before_mb = initial_memory,
        memory_after_mb = final_memory,
        memory_delta_mb = final_memory - initial_memory,
        gc_collections = 0 // Rust doesn't have GC, but useful for other runtimes
    );
}

/// Demonstrate advanced timing techniques
async fn advanced_timing_techniques() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Advanced Timing Techniques ===");
    
    // Nested operation timing
    nested_operation_timing().await;
    
    // Concurrent operation monitoring
    concurrent_operation_monitoring().await;
    
    // Conditional performance logging
    conditional_performance_logging().await;
    
    Ok(())
}

async fn nested_operation_timing() {
    println!("--- Nested Operation Timing ---");
    
    let operation_id = "nested_op_123";
    let total_timer = Instant::now();
    
    log_transform_info!("Starting complex nested operation", 
                       operation_id = operation_id);
    
    // Phase 1: Data preparation
    let prep_timer = Instant::now();
    sleep(Duration::from_millis(30)).await;
    let prep_duration = prep_timer.elapsed();
    
    log_transform_info!("Phase 1 completed",
                       operation_id = operation_id,
                       phase = "data_preparation",
                       duration_ms = prep_duration.as_millis(),
                       records_prepared = 500);
    
    // Phase 2: Processing
    let proc_timer = Instant::now();
    sleep(Duration::from_millis(80)).await;
    let proc_duration = proc_timer.elapsed();
    
    log_transform_info!("Phase 2 completed",
                       operation_id = operation_id,
                       phase = "data_processing",
                       duration_ms = proc_duration.as_millis(),
                       records_processed = 500,
                       transformations_applied = 15);
    
    // Phase 3: Finalization
    let final_timer = Instant::now();
    sleep(Duration::from_millis(20)).await;
    let final_duration = final_timer.elapsed();
    
    log_transform_info!("Phase 3 completed",
                       operation_id = operation_id,
                       phase = "finalization",
                       duration_ms = final_duration.as_millis(),
                       records_finalized = 500);
    
    let total_duration = total_timer.elapsed();
    log_transform_info!("Nested operation completed",
                       operation_id = operation_id,
                       total_duration_ms = total_duration.as_millis(),
                       phase_1_ms = prep_duration.as_millis(),
                       phase_2_ms = proc_duration.as_millis(),
                       phase_3_ms = final_duration.as_millis(),
                       efficiency_score = calculate_efficiency_score(&[prep_duration, proc_duration, final_duration]));
}

async fn concurrent_operation_monitoring() {
    println!("--- Concurrent Operation Monitoring ---");
    
    let concurrent_timer = Instant::now();
    let operation_id = "concurrent_op_456";
    
    log::info!("Starting concurrent operations", operation_id = operation_id);
    
    // Spawn multiple concurrent tasks
    let tasks = vec![
        tokio::spawn(async move {
            let timer = Instant::now();
            sleep(Duration::from_millis(60)).await;
            let duration = timer.elapsed();
            log_network_info!("Concurrent task completed",
                             operation_id = "concurrent_op_456",
                             task_id = "network_sync",
                             duration_ms = duration.as_millis(),
                             peers_contacted = 3);
            duration
        }),
        tokio::spawn(async move {
            let timer = Instant::now();
            sleep(Duration::from_millis(45)).await;
            let duration = timer.elapsed();
            log::info!(
                target: "datafold_node::database",
                "Concurrent task completed",
                operation_id = "concurrent_op_456",
                task_id = "database_cleanup",
                duration_ms = duration.as_millis(),
                records_cleaned = 250
            );
            duration
        }),
        tokio::spawn(async move {
            let timer = Instant::now();
            sleep(Duration::from_millis(90)).await;
            let duration = timer.elapsed();
            log_transform_info!("Concurrent task completed",
                               operation_id = "concurrent_op_456",
                               task_id = "data_transform",
                               duration_ms = duration.as_millis(),
                               transformations = 8);
            duration
        }),
    ];
    
    // Wait for all tasks to complete
    let results = futures::future::join_all(tasks).await;
    let concurrent_duration = concurrent_timer.elapsed();
    
    let task_durations: Vec<Duration> = results
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();
    
    let max_duration = task_durations.iter().max().unwrap_or(&Duration::ZERO);
    let total_work: Duration = task_durations.iter().sum();
    let parallelization_efficiency = total_work.as_millis() as f64 / concurrent_duration.as_millis() as f64;
    
    log::info!("Concurrent operations completed",
              operation_id = operation_id,
              total_duration_ms = concurrent_duration.as_millis(),
              longest_task_ms = max_duration.as_millis(),
              total_work_ms = total_work.as_millis(),
              parallelization_efficiency = parallelization_efficiency,
              task_count = task_durations.len());
}

async fn conditional_performance_logging() {
    println!("--- Conditional Performance Logging ---");
    
    // Only log slow operations
    let operations = vec![
        ("fast_operation", 15),
        ("medium_operation", 150),
        ("slow_operation", 1500),
        ("very_slow_operation", 3000),
    ];
    
    for (operation_name, duration_ms) in operations {
        let timer = Instant::now();
        sleep(Duration::from_millis(duration_ms)).await;
        let actual_duration = timer.elapsed();
        
        // Conditional logging based on performance thresholds
        match actual_duration.as_millis() {
            0..=100 => {
                // Only debug level for fast operations
                log::debug!(
                    target: "datafold_node::performance",
                    "Fast operation completed",
                    operation = operation_name,
                    duration_ms = actual_duration.as_millis()
                );
            }
            101..=1000 => {
                // Info level for medium operations
                log::info!(
                    target: "datafold_node::performance",
                    "Medium operation completed",
                    operation = operation_name,
                    duration_ms = actual_duration.as_millis(),
                    performance_category = "medium"
                );
            }
            1001..=5000 => {
                // Warn level for slow operations
                log::warn!(
                    target: "datafold_node::performance",
                    "Slow operation detected",
                    operation = operation_name,
                    duration_ms = actual_duration.as_millis(),
                    performance_category = "slow",
                    threshold_exceeded = true,
                    suggested_optimization = "consider_caching"
                );
            }
            _ => {
                // Error level for very slow operations
                log::error!(
                    target: "datafold_node::performance",
                    "Very slow operation detected",
                    operation = operation_name,
                    duration_ms = actual_duration.as_millis(),
                    performance_category = "very_slow",
                    requires_investigation = true,
                    impact = "high"
                );
            }
        }
    }
}

/// Demonstrate batch operation monitoring
async fn batch_operation_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Batch Operation Monitoring ===");
    
    let batch_id = "batch_789";
    let total_records = 10000;
    let batch_size = 100;
    let batch_count = total_records / batch_size;
    
    let batch_timer = Instant::now();
    let mut processed_records = 0;
    let mut failed_records = 0;
    
    log::info!("Starting batch operation",
              batch_id = batch_id,
              total_records = total_records,
              batch_size = batch_size,
              estimated_batches = batch_count);
    
    let mut batch_durations = Vec::new();
    
    for batch_num in 0..batch_count {
        let batch_start = Instant::now();
        
        // Simulate batch processing with occasional failures
        let processing_time = if batch_num % 10 == 9 {
            // Simulate slow batch
            sleep(Duration::from_millis(50)).await;
            Duration::from_millis(50)
        } else {
            // Normal batch
            sleep(Duration::from_millis(10)).await;
            Duration::from_millis(10)
        };
        
        let batch_duration = batch_start.elapsed();
        batch_durations.push(batch_duration);
        
        let records_in_batch = if batch_num == batch_count - 1 {
            total_records - processed_records // Last batch might be smaller
        } else {
            batch_size
        };
        
        // Simulate some failures
        let failures = if batch_num % 15 == 14 { 2 } else { 0 };
        let successful = records_in_batch - failures;
        
        processed_records += successful;
        failed_records += failures;
        
        // Log batch completion (only every 10th batch to avoid spam)
        if batch_num % 10 == 0 || batch_num == batch_count - 1 {
            let progress_percent = (processed_records as f64 / total_records as f64) * 100.0;
            let avg_duration = batch_durations.iter().sum::<Duration>() / batch_durations.len() as u32;
            let estimated_remaining = avg_duration * (batch_count - batch_num - 1) as u32;
            
            log::info!("Batch progress update",
                      batch_id = batch_id,
                      batch_number = batch_num + 1,
                      total_batches = batch_count,
                      progress_percent = progress_percent,
                      records_processed = processed_records,
                      records_failed = failed_records,
                      batch_duration_ms = batch_duration.as_millis(),
                      avg_batch_duration_ms = avg_duration.as_millis(),
                      estimated_remaining_ms = estimated_remaining.as_millis());
        }
        
        // Log slow batches immediately
        if batch_duration.as_millis() > 30 {
            log::warn!("Slow batch detected",
                      batch_id = batch_id,
                      batch_number = batch_num + 1,
                      duration_ms = batch_duration.as_millis(),
                      threshold_ms = 30,
                      records_in_batch = records_in_batch);
        }
    }
    
    let total_duration = batch_timer.elapsed();
    let success_rate = processed_records as f64 / total_records as f64;
    let throughput = total_records as f64 / total_duration.as_secs_f64();
    
    log::info!("Batch operation completed",
              batch_id = batch_id,
              total_duration_ms = total_duration.as_millis(),
              total_records = total_records,
              successful_records = processed_records,
              failed_records = failed_records,
              success_rate = success_rate,
              throughput_records_per_sec = throughput,
              total_batches = batch_count,
              avg_batch_duration_ms = (total_duration.as_millis() / batch_count as u128));
    
    Ok(())
}

/// Demonstrate memory and resource monitoring
async fn memory_and_resource_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Memory and Resource Monitoring ===");
    
    // Simulate memory-intensive operations
    memory_intensive_operations().await;
    
    // Resource utilization monitoring
    resource_utilization_monitoring().await;
    
    Ok(())
}

async fn memory_intensive_operations() {
    println!("--- Memory-Intensive Operations ---");
    
    let operation_id = "memory_op_001";
    let initial_memory = get_memory_usage();
    
    log::info!("Starting memory-intensive operation",
              operation_id = operation_id,
              initial_memory_mb = initial_memory);
    
    // Simulate phases with different memory usage
    let phases = vec![
        ("initialization", 20, 50),  // duration_ms, memory_delta_mb
        ("data_loading", 100, 200),
        ("processing", 150, 100),
        ("cleanup", 30, -150),
    ];
    
    let mut current_memory = initial_memory;
    
    for (phase_name, duration_ms, memory_delta) in phases {
        let phase_timer = Instant::now();
        
        // Simulate work and memory change
        sleep(Duration::from_millis(duration_ms)).await;
        current_memory = (current_memory as i32 + memory_delta) as u32;
        
        let phase_duration = phase_timer.elapsed();
        
        log::info!("Memory operation phase completed",
                  operation_id = operation_id,
                  phase = phase_name,
                  duration_ms = phase_duration.as_millis(),
                  memory_before_mb = (current_memory as i32 - memory_delta) as u32,
                  memory_after_mb = current_memory,
                  memory_delta_mb = memory_delta);
        
        // Check for memory pressure
        if current_memory > initial_memory + 300 {
            log::warn!("High memory usage detected",
                      operation_id = operation_id,
                      current_memory_mb = current_memory,
                      initial_memory_mb = initial_memory,
                      memory_growth_mb = current_memory - initial_memory,
                      phase = phase_name);
        }
    }
    
    log::info!("Memory-intensive operation completed",
              operation_id = operation_id,
              final_memory_mb = current_memory,
              memory_growth_mb = current_memory - initial_memory);
}

async fn resource_utilization_monitoring() {
    println!("--- Resource Utilization Monitoring ---");
    
    // Simulate system resource monitoring
    let monitoring_duration = Duration::from_millis(500);
    let sample_interval = Duration::from_millis(100);
    let samples = monitoring_duration.as_millis() / sample_interval.as_millis();
    
    log::info!("Starting resource utilization monitoring",
              duration_ms = monitoring_duration.as_millis(),
              sample_interval_ms = sample_interval.as_millis(),
              expected_samples = samples);
    
    for sample in 0..samples {
        let cpu_usage = simulate_cpu_usage(sample);
        let memory_usage = simulate_memory_usage(sample);
        let disk_io = simulate_disk_io(sample);
        let network_io = simulate_network_io(sample);
        
        log::info!("Resource utilization sample",
                  sample_number = sample + 1,
                  cpu_percent = cpu_usage,
                  memory_percent = memory_usage,
                  disk_read_mbps = disk_io.0,
                  disk_write_mbps = disk_io.1,
                  network_in_mbps = network_io.0,
                  network_out_mbps = network_io.1);
        
        // Alert on high resource usage
        if cpu_usage > 80.0 {
            log::warn!("High CPU usage detected", 
                      cpu_percent = cpu_usage,
                      threshold = 80.0);
        }
        
        if memory_usage > 85.0 {
            log::warn!("High memory usage detected",
                      memory_percent = memory_usage,
                      threshold = 85.0);
        }
        
        sleep(sample_interval).await;
    }
}

/// Demonstrate distributed operation tracking
async fn distributed_operation_tracking() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Distributed Operation Tracking ===");
    
    let correlation_id = "dist_op_abc123";
    let operation_id = "distributed_query";
    
    // Simulate distributed operation across multiple nodes
    log::info!("Starting distributed operation",
              correlation_id = correlation_id,
              operation_id = operation_id,
              coordinator_node = "node_001",
              participant_nodes = vec!["node_002", "node_003", "node_004"]);
    
    // Simulate work on each node
    let nodes = vec![
        ("node_002", 80, true),   // node_id, duration_ms, success
        ("node_003", 120, true),
        ("node_004", 95, false),  // This node fails
    ];
    
    let mut successful_nodes = Vec::new();
    let mut failed_nodes = Vec::new();
    
    for (node_id, duration_ms, success) in nodes {
        let node_timer = Instant::now();
        
        log_network_info!("Starting work on node",
                         correlation_id = correlation_id,
                         node_id = node_id,
                         operation = "distributed_query_part");
        
        // Simulate work
        sleep(Duration::from_millis(duration_ms)).await;
        let actual_duration = node_timer.elapsed();
        
        if success {
            successful_nodes.push(node_id);
            log_network_info!("Node work completed successfully",
                             correlation_id = correlation_id,
                             node_id = node_id,
                             duration_ms = actual_duration.as_millis(),
                             records_processed = duration_ms * 10, // Simulate records
                             status = "success");
        } else {
            failed_nodes.push(node_id);
            log::error!(
                target: "datafold_node::network",
                "Node work failed",
                correlation_id = correlation_id,
                node_id = node_id,
                duration_ms = actual_duration.as_millis(),
                error = "connection_timeout",
                retry_possible = true
            );
        }
    }
    
    // Determine overall operation result
    let success_rate = successful_nodes.len() as f64 / (successful_nodes.len() + failed_nodes.len()) as f64;
    
    if success_rate >= 0.67 {
        log::info!("Distributed operation completed with acceptable success rate",
                  correlation_id = correlation_id,
                  operation_id = operation_id,
                  successful_nodes = successful_nodes.len(),
                  failed_nodes = failed_nodes.len(),
                  success_rate = success_rate,
                  status = "partial_success");
    } else {
        log::error!("Distributed operation failed",
                   correlation_id = correlation_id,
                   operation_id = operation_id,
                   successful_nodes = successful_nodes.len(),
                   failed_nodes = failed_nodes.len(),
                   success_rate = success_rate,
                   status = "failed",
                   requires_retry = true);
    }
    
    Ok(())
}

/// Demonstrate performance analysis and optimization
async fn performance_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Performance Analysis ===");
    
    // Collect performance metrics
    let metrics = Arc::new(Mutex::new(PerformanceMetrics::new()));
    
    // Simulate various operations and collect metrics
    simulate_operations_for_analysis(metrics.clone()).await;
    
    // Analyze performance data
    analyze_performance_metrics(metrics.clone()).await;
    
    Ok(())
}

async fn simulate_operations_for_analysis(metrics: Arc<Mutex<PerformanceMetrics>>) {
    println!("--- Collecting Performance Metrics ---");
    
    let operations = vec![
        ("database_query", vec![25, 30, 28, 45, 35, 32, 150, 29]), // One slow query
        ("network_request", vec![80, 85, 90, 82, 88, 300, 95, 78]), // One timeout
        ("transform_operation", vec![15, 18, 16, 20, 17, 19, 16, 22]),
        ("schema_validation", vec![5, 6, 4, 7, 5, 6, 8, 5]),
    ];
    
    for (operation_name, durations) in operations {
        let duration_count = durations.len();
        for (i, duration_ms) in durations.into_iter().enumerate() {
            let timer = Instant::now();
            
            // Simulate work
            sleep(Duration::from_millis(duration_ms)).await;
            let actual_duration = timer.elapsed();
            
            // Record metrics
            {
                let mut m = metrics.lock().await;
                m.record_operation(operation_name, actual_duration);
                
                // Simulate occasional errors
                if i == duration_count - 1 && operation_name == "network_request" {
                    m.record_error(operation_name);
                }
            }
            
            log::debug!("Operation sample recorded",
                       operation = operation_name,
                       sample_number = i + 1,
                       duration_ms = actual_duration.as_millis());
        }
    }
}

async fn analyze_performance_metrics(metrics: Arc<Mutex<PerformanceMetrics>>) {
    println!("--- Performance Analysis Results ---");
    
    let m = metrics.lock().await;
    
    for (operation, _) in &m.operation_counts {
        let count = m.operation_counts.get(operation).unwrap_or(&0);
        let avg_duration = m.get_average_duration(operation).unwrap_or(Duration::ZERO);
        let p95_duration = m.get_percentile(operation, 95.0).unwrap_or(Duration::ZERO);
        let p99_duration = m.get_percentile(operation, 99.0).unwrap_or(Duration::ZERO);
        let error_count = m.error_counts.get(operation).unwrap_or(&0);
        let error_rate = *error_count as f64 / *count as f64;
        
        log::info!("Performance analysis result",
                  operation = operation,
                  sample_count = count,
                  avg_duration_ms = avg_duration.as_millis(),
                  p95_duration_ms = p95_duration.as_millis(),
                  p99_duration_ms = p99_duration.as_millis(),
                  error_count = error_count,
                  error_rate = error_rate);
        
        // Generate recommendations
        if p95_duration.as_millis() > avg_duration.as_millis() * 3 {
            log::warn!("High variance detected in operation performance",
                      operation = operation,
                      recommendation = "investigate_outliers",
                      variance_factor = p95_duration.as_millis() as f64 / avg_duration.as_millis() as f64);
        }
        
        if error_rate > 0.1 {
            log::warn!("High error rate detected",
                      operation = operation,
                      error_rate = error_rate,
                      recommendation = "improve_error_handling");
        }
        
        if avg_duration.as_millis() > 100 {
            log::info!("Performance optimization opportunity",
                      operation = operation,
                      avg_duration_ms = avg_duration.as_millis(),
                      recommendation = "consider_optimization");
        }
    }
}

// Utility functions for simulation

fn get_memory_usage() -> u32 {
    // Simulate memory usage in MB
    150 + (rand::random::<u32>() % 50)
}

fn calculate_efficiency_score(durations: &[Duration]) -> f64 {
    // Simple efficiency calculation based on variance
    let total: Duration = durations.iter().sum();
    let avg = total / durations.len() as u32;
    let variance: f64 = durations.iter()
        .map(|d| {
            let diff = d.as_millis() as f64 - avg.as_millis() as f64;
            diff * diff
        })
        .sum::<f64>() / durations.len() as f64;
    
    // Lower variance = higher efficiency
    1.0 / (1.0 + variance / 1000.0)
}

fn simulate_cpu_usage(sample: u128) -> f64 {
    // Simulate CPU usage pattern
    let base = 30.0;
    let variation = 20.0 * (sample as f64 * 0.1).sin();
    let spike = if sample % 5 == 4 { 25.0 } else { 0.0 };
    base + variation + spike
}

fn simulate_memory_usage(sample: u128) -> f64 {
    // Simulate gradual memory increase
    let base = 45.0;
    let growth = sample as f64 * 0.5;
    let variation = 5.0 * (sample as f64 * 0.2).cos();
    (base + growth + variation).min(90.0)
}

fn simulate_disk_io(_sample: u128) -> (f64, f64) {
    // Simulate disk I/O (read, write) in MB/s
    let read = 50.0 + rand::random::<f64>() * 100.0;
    let write = 20.0 + rand::random::<f64>() * 40.0;
    (read, write)
}

fn simulate_network_io(_sample: u128) -> (f64, f64) {
    // Simulate network I/O (in, out) in MB/s
    let incoming = 10.0 + rand::random::<f64>() * 30.0;
    let outgoing = 5.0 + rand::random::<f64>() * 15.0;
    (incoming, outgoing)
}