//! # Logging Examples
//!
//! This file demonstrates the enhanced logging capabilities of the datafold
//! system, showcasing various logging patterns, configurations, and features.

use fold_node::logging::{LoggingSystem, LogConfig};
use fold_node::{
    log_transform_debug, log_transform_info, log_transform_warn, log_transform_error,
    log_network_debug, log_network_info, log_network_warn, log_network_error,
    log_schema_debug, log_schema_info, log_schema_warn, log_schema_error,
    log_http_debug, log_http_info, log_http_warn, log_http_error
};
use log::{debug, info, warn, error};
use tokio::time::{sleep, Duration};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Datafold Enhanced Logging Examples");
    
    // Initialize with custom configuration
    initialize_logging().await?;
    
    // Run different types of logging examples
    basic_logging_examples().await;
    feature_specific_examples().await;
    structured_logging_examples().await;
    performance_logging_examples().await;
    error_handling_examples().await;
    recovery_examples().await;
    distributed_operation_examples().await;
    business_event_examples().await;
    configuration_examples().await;
    integration_examples().await;
    
    Ok(())
}

/// Initialize logging system with custom configuration
async fn initialize_logging() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = LogConfig::default();
    
    // Configure console output
    config.outputs.console.enabled = true;
    config.outputs.console.level = "DEBUG".to_string();
    config.outputs.console.colors = true;
    
    config.outputs.file.enabled = true;
    config.outputs.file.path = "examples/logs/example.log".to_string();
    config.outputs.file.level = "TRACE".to_string();
    
    config.outputs.web.enabled = true;
    config.outputs.web.level = "INFO".to_string();
    
    LoggingSystem::init_with_config(config).await.map_err(|e| format!("Logging init failed: {:?}", e))?;
    
    info!("Logging system initialized successfully");
    Ok(())
}

/// Demonstrate basic logging patterns
async fn basic_logging_examples() {
    println!("\n=== Basic Logging Examples ===");
    
    // Simple message logging
    debug!("This is a debug message for development");
    info!("Application started successfully");
    warn!("This is a warning about potential issues");
    error!("This demonstrates error logging");
    
    sleep(Duration::from_millis(100)).await;
    
    // With structured data
    log::info!(
        "User action recorded: user_id={}, action={}, timestamp={}, success={}",
        "user_123",
        "login",
        chrono::Utc::now().to_rfc3339(),
        true
    );
    
    sleep(Duration::from_millis(100)).await;
}

/// Demonstrate feature-specific logging
async fn feature_specific_examples() {
    println!("\n=== Feature-Specific Logging Examples ===");
    
    // Transform operations
    simulate_transform_operations().await;
    
    // Network operations
    simulate_network_operations().await;
    
    // Schema operations
    simulate_schema_operations().await;
    
    // HTTP server operations
    simulate_http_operations().await;
}

async fn simulate_transform_operations() {
    println!("--- Transform Operations ---");
    
    let transform_id = "user_score_calc";
    let input_data = vec!["data1", "data2", "data3"];
    
    log_transform_debug!("Starting transform: transform_id={}", transform_id);
    log_transform_info!("Transform initialized: transform_id={}, input_records={}", 
                       transform_id,
                       input_data.len());
    
    // Simulate processing
    for (i, data) in input_data.iter().enumerate() {
        log_transform_debug!("Processing record: transform_id={}, record_index={}, record_data={}", 
                           transform_id,
                           i,
                           data);
        sleep(Duration::from_millis(10)).await;
    }
    
    log_transform_info!("Transform completed successfully: transform_id={}, output_records={}", 
                       transform_id,
                       input_data.len());
    
    // Simulate a warning
    log_transform_warn!("Transform used deprecated function: transform_id={}, function_name={}, replacement={}",
                       transform_id,
                       "legacy_calc",
                       "modern_calc");
    
    // Simulate an error scenario
    log_transform_error!("Transform validation failed: transform_id={}, error={}",
                        "invalid_transform",
                        "Missing required field: 'user_id'");
}

async fn simulate_network_operations() {
    println!("--- Network Operations ---");
    
    let peer_id = "node_abc123";
    let local_port = 9000;
    
    log_network_debug!("Initializing network layer on port {}", local_port);
    log_network_info!("Network service started on port {}", local_port);
    
    // Peer discovery
    log_network_debug!("Attempting peer discovery on broadcast port {}", 9000);
    log_network_info!("Peer discovered: peer_id={}, peer_address={}, response_time_ms={}",
                     peer_id,
                     "192.168.1.100:9000",
                     45);
    
    // Heartbeat
    log_network_debug!("Sending heartbeat to peer {}", peer_id);
    log_network_debug!("Received heartbeat response: peer_id={}, latency_ms={}",
                      peer_id,
                      12);
    
    // Connection issues
    log_network_warn!("Missed heartbeats detected: peer_id={}, missed_heartbeats={}, threshold={}",
                     peer_id,
                     3,
                     5);
    
    log_network_error!("Peer connection lost: peer_id={}, last_seen={}, reason={}",
                      peer_id,
                      "2025-06-02T21:13:00Z",
                      "timeout");
}

async fn simulate_schema_operations() {
    println!("--- Schema Operations ---");
    
    let schema_name = "UserProfile";
    let schema_version = "1.2.3";
    
    log_schema_debug!("Loading schema definition: schema_name={}", schema_name);
    log_schema_info!("Schema loaded successfully: schema_name={}, version={}, fields_count={}",
                    schema_name,
                    schema_version,
                    15);
    
    // Field processing
    log_schema_debug!("Processing field: schema_name={}, field_name={}, field_type={}",
                     schema_name,
                     "email",
                     "string");
    
    log_schema_warn!("Deprecated field usage: schema_name={}, field_name={}, deprecated_since={}, replacement={}",
                    schema_name,
                    "legacy_id",
                    "1.1.0",
                    "new_id");
    
    // Validation errors
    log_schema_error!("Schema validation failed: schema_name={}, error={}, field_path={}",
                     "InvalidSchema",
                     "Circular reference detected",
                     "user.profile.user");
}

async fn simulate_http_operations() {
    println!("--- HTTP Server Operations ---");
    
    let request_id = "req_abc123";
    let method = "POST";
    let path = "/api/schemas";
    let user_agent = "DataFoldClient/1.0";
    
    // Request processing
    log_http_info!("Request received: request_id={}, method={}, path={}, user_agent={}",
                   request_id,
                   method,
                   path,
                   user_agent);
    
    log_http_debug!("User authenticated: request_id={}, user_id={}, permissions={}",
                   request_id,
                   "user_456",
                   "schema:write");
    
    // Success response
    sleep(Duration::from_millis(50)).await;
    log_http_info!("Request completed: request_id={}, method={}, path={}, status_code={}, response_time_ms={}, response_size_bytes={}",
                   request_id,
                   method,
                   path,
                   201,
                   50,
                   1024);
    
    // Slow request warning
    log_http_warn!("Slow request detected: request_id={}, method={}, path={}, response_time_ms={}, threshold_ms={}",
                   "req_slow123",
                   "GET",
                   "/api/query/complex",
                   5000,
                   1000);
    
    // Error response
    log_http_error!("Request failed: request_id={}, method={}, path={}, status_code={}, error={}, client_ip={}",
                    "req_error456",
                    "PUT",
                    "/api/schemas/invalid",
                    400,
                    "Invalid schema format",
                    "192.168.1.50");
}

/// Demonstrate structured logging for complex data
async fn structured_logging_examples() {
    println!("\n=== Structured Logging Examples ===");
    
    // Performance metrics with custom timer
    let timer = PerformanceTimer::new(
        "network_discovery",
        vec!["duration", "peers_found", "success_rate"]
    );
    
    let start = std::time::Instant::now();
    
    // Simulate work
    sleep(Duration::from_millis(200)).await;
    
    let duration = start.elapsed();
    info!("Operation completed: operation={}, duration_ms={}, peers_found={}",
                     "peer_discovery",
                     duration.as_millis(),
                     5);
}

/// Demonstrate performance-focused logging
async fn performance_logging_examples() {
    println!("\n=== Performance Logging Examples ===");
    
    let records_processed = 1000;
    let start = std::time::Instant::now();
    
    // Simulate batch processing
    for batch in 0..10 {
        sleep(Duration::from_millis(50)).await;
        
        if batch % 3 == 0 {
            info!("Batch processed: batch_number={}, records_in_batch={}, cumulative_records={}",
                batch,
                100,
                (batch + 1) * 100
            );
        }
    }
    
    let total_duration = start.elapsed();
    let throughput = records_processed as f64 / total_duration.as_secs_f64();
    
    info!("Processing summary: total_records={}, total_duration_ms={}, throughput_records_per_sec={}, average_batch_time_ms={}",
        records_processed,
        total_duration.as_millis(),
        throughput as u64,
        total_duration.as_millis() / 10
    );
}

/// Demonstrate error handling and logging patterns
async fn error_handling_examples() {
    println!("\n=== Error Handling Examples ===");
    
    let errors = vec![
        ("ERR001", "Connection timeout", "network"),
        ("ERR002", "Invalid schema format", "schema"),
        ("ERR003", "Division by zero", "transform"),
        ("ERR004", "Unauthorized access", "auth"),
    ];
    
    for (error_code, error_message, component) in errors {
        match component {
            "network" => {
                log_network_error!("Network error: error_code={}, error_message={}, retry_count={}, max_retries={}, next_retry_in_ms={}",
                                  error_code,
                                  error_message,
                                  3,
                                  5,
                                  5000);
            }
            "schema" => {
                log_schema_error!("Schema validation error: error_code={}, error_message={}, schema_name={}, field_path={}, line_number={}",
                                 error_code,
                                 error_message,
                                 "UserProfile",
                                 "user.profile.id",
                                 15);
            }
            "transform" => {
                log_transform_error!("Transform execution error: error_code={}, error_message={}, transform_id={}, expression={}, input_values={}",
                                    error_code,
                                    error_message,
                                    "user_calc",
                                    "score / count",
                                    "score=100, count=0");
            }
            _ => {
                error!("Generic error: error_code={}, error_message={}, component={}",
                            error_code,
                            error_message,
                            component);
            }
        }
        
        sleep(Duration::from_millis(100)).await;
    }
}

/// Demonstrate system recovery logging
async fn recovery_examples() {
    println!("\n=== Recovery and Resilience Examples ===");
    
    let operation_id = "op_recovery_123";
    
    // Initial failure
    warn!("Operation failed, initiating recovery: operation_id={}, error={}, attempt={}",
                        operation_id,
                        "Connection lost",
                        1);
    
    info!("Recovery strategy selected: operation_id={}, recovery_strategy={}",
                       operation_id,
                       "reconnect_and_retry");
    
    // Recovery progress
    sleep(Duration::from_millis(100)).await;
    info!("Partial recovery completed: operation_id={}, recovered_items={}, lost_items={}, total_items={}",
                       operation_id,
                       8,
                       2,
                       10);
    
    info!("Recovery completed: operation_id={}, final_status={}, success_rate={}",
                       operation_id,
                       "partial_success",
                       0.8);
}

/// Demonstrate distributed operation logging
async fn distributed_operation_examples() {
    println!("\n=== Distributed Operation Examples ===");
    
    let correlation_id = "corr_abc123";
    let user_id = "user_789";
    
    // Request tracing across services
    info!("Distributed request started: correlation_id={}, user_id={}, endpoint={}, method={}",
                  correlation_id,
                  user_id,
                  "/api/users/profile",
                  "GET");
    
    // Schema service
    log_schema_info!("Schema validation started: correlation_id={}, schema_name={}, validation_rules={}",
                     correlation_id,
                     "UserProfileRequest",
                     5);
    
    // Query execution
    sleep(Duration::from_millis(50)).await;
    info!("Query execution started: correlation_id={}, query_type={}, estimated_duration_ms={}",
        correlation_id,
        "user_profile_lookup",
        50
    );
    
    // Transform service
    log_transform_info!("Transform pipeline started: correlation_id={}, transform_count={}, input_records={}",
                        correlation_id,
                        3,
                        1);
    
    // Final response
    log_http_info!("Distributed request completed: correlation_id={}, status_code={}, response_size_bytes={}, total_duration_ms={}",
                   correlation_id,
                   200,
                   512,
                   75);
}

/// Demonstrate business event logging
async fn business_event_examples() {
    println!("\n=== Business Event Examples ===");
    
    // User registration
    info!("User registration completed: event_type={}, user_id={}, registration_method={}, account_type={}, referral_source={}",
        "user_registered",
        "user_new_456",
        "email",
        "standard",
        "organic"
    );
    
    // Data pipeline completion
    info!("Data pipeline completed: event_type={}, pipeline_id={}, records_processed={}, processing_time_minutes={}, success_rate={}, error_count={}",
        "pipeline_completed",
        "daily_user_scores",
        50000,
        15,
        0.998,
        100
    );
    
    // System health check
    info!("System health check: event_type={}, status={}, cpu_usage_percent={}, memory_usage_percent={}, disk_usage_percent={}, active_connections={}",
        "health_check",
        "healthy",
        35.5,
        42.1,
        78.9,
        127
    );
}

/// Demonstrate configuration and system introspection
async fn configuration_examples() {
    println!("\n=== Configuration and System Examples ===");
    
    if let Some(config) = LoggingSystem::get_config().await {
        info!("Current logging configuration retrieved: default_level={}, colors_enabled={}, console_enabled={}, file_enabled={}, web_enabled={}",
                  config.general.default_level,
                  config.general.enable_colors,
                  config.outputs.console.enabled,
                  config.outputs.file.enabled,
                  config.outputs.web.enabled);
    }
    
    if let Some(features) = LoggingSystem::get_features().await {
        for (feature, level) in features {
            debug!("Feature log level: feature={}, level={}",
                       feature,
                       level);
        }
    }
}

/// Demonstrate runtime configuration changes
async fn integration_examples() {
    println!("\n=== Integration and Dynamic Configuration Examples ===");
    
    if let Err(e) = LoggingSystem::update_feature_level("transform", "TRACE").await {
        error!("Failed to update transform log level: {}", e);
    } else {
        info!("Transform logging level updated to TRACE");
        
        // Test the new level
        if let Err(e) = LoggingSystem::update_feature_level("transform", "DEBUG").await {
            error!("Failed to reset transform log level: {}", e);
        } else {
            info!("Transform logging level reset to DEBUG");
        }
    }
}

/// Example helper for performance timing and logging
struct PerformanceTimer {
    operation: String,
    metrics: Vec<String>,
    start_time: Instant,
}

impl PerformanceTimer {
    fn new(operation: &str, metrics: Vec<&str>) -> Self {
        Self {
            operation: operation.to_string(),
            metrics: metrics.iter().map(|s| s.to_string()).collect(),
            start_time: Instant::now(),
        }
    }
}

/// Simple integration example showing structured data logging
async fn simple_integration_examples() {
    let user_id = "user_123";
    let action = "login";
    
    // Structured user action
    info!("User action performed: user_id={}, action={}, timestamp={}",
              user_id,
              action,
              chrono::Utc::now().to_rfc3339());
    
    let error_msg = "Connection timeout";
    error!("Database operation failed: error={}, retry_count={}, operation={}, table={}",
               error_msg,
               3,
               "user_lookup",
               "users");
    
    let duration = std::time::Duration::from_millis(150);
    info!("Operation completed: operation={}, duration_ms={}, records_returned={}, cache_hit={}",
              "complex_query",
              duration.as_millis(),
              1500,
              false);
}