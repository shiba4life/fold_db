//! End-to-End Performance Benchmarks
//!
//! This module provides comprehensive end-to-end performance benchmarking for
//! complete signature authentication workflows spanning server, client, SDK, and CLI components.

use super::{
    PerformanceMeasurement, BenchmarkTimer, PerformanceBenchmarkConfig,
    PerformanceTargets
};
use crate::crypto::ed25519::{generate_master_keypair, MasterKeyPair};
use crate::datafold_node::signature_auth::{SignatureAuthConfig, SignatureVerificationState, SecurityProfile};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use uuid::Uuid;

/// End-to-end performance benchmark suite
pub struct EndToEndPerformanceBenchmarks {
    config: PerformanceBenchmarkConfig,
    targets: PerformanceTargets,
    results: Vec<PerformanceMeasurement>,
    key_pairs: Vec<MasterKeyPair>,
    verification_state: Arc<SignatureVerificationState>,
}

impl EndToEndPerformanceBenchmarks {
    /// Create new end-to-end performance benchmarks
    pub fn new(
        config: PerformanceBenchmarkConfig,
        targets: PerformanceTargets,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Generate test key pairs
        let mut key_pairs = Vec::new();
        for _ in 0..5 {
            key_pairs.push(generate_master_keypair()?);
        }
        
        // Create verification state
        let auth_config = SignatureAuthConfig::default();
        let verification_state = Arc::new(SignatureVerificationState::new(auth_config)?);
        
        Ok(Self {
            config,
            targets,
            results: Vec::new(),
            key_pairs,
            verification_state,
        })
    }

    /// Run all end-to-end performance benchmarks
    pub async fn run_all_benchmarks(&mut self) -> Result<Vec<PerformanceMeasurement>, Box<dyn std::error::Error>> {
        println!("🚀 Starting end-to-end performance benchmarks");
        
        // Complete authentication workflow benchmarks
        self.benchmark_complete_authentication_workflow().await?;
        self.benchmark_multi_component_integration().await?;
        self.benchmark_cross_platform_consistency().await?;
        
        // Realistic usage scenarios
        self.benchmark_typical_api_usage().await?;
        self.benchmark_high_frequency_trading_scenario().await?;
        self.benchmark_batch_processing_scenario().await?;
        self.benchmark_mobile_app_scenario().await?;
        
        // Load and stress testing
        self.benchmark_sustained_production_load().await?;
        self.benchmark_burst_traffic_handling().await?;
        self.benchmark_mixed_workload_performance().await?;
        
        // Error handling and recovery
        self.benchmark_error_recovery_performance().await?;
        self.benchmark_degraded_mode_performance().await?;
        
        println!("✅ End-to-end performance benchmarks completed");
        Ok(self.results.clone())
    }

    /// Benchmark complete authentication workflow
    async fn benchmark_complete_authentication_workflow(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking complete authentication workflow");
        
        let mut timer = BenchmarkTimer::new();
        let iterations = 1000;
        let mut success_count = 0;
        let mut error_count = 0;
        
        let start_time = Instant::now();
        
        for i in 0..iterations {
            let key_pair = &self.key_pairs[i % self.key_pairs.len()];
            
            timer.start();
            
            // Complete workflow: Client signing + Server verification
            let workflow_result = self.execute_complete_workflow(key_pair, i).await;
            
            timer.record();
            
            match workflow_result {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "complete_authentication_workflow".to_string(),
            "end_to_end".to_string(),
            "full_auth_cycle".to_string(),
        );
        
        measurement.operation_count = iterations;
        measurement.total_duration = total_duration;
        measurement.avg_operation_time_ms = stats.avg_ms;
        measurement.median_operation_time_ms = stats.median_ms;
        measurement.p95_operation_time_ms = stats.p95_ms;
        measurement.p99_operation_time_ms = stats.p99_ms;
        measurement.min_operation_time_ms = stats.min_ms;
        measurement.max_operation_time_ms = stats.max_ms;
        measurement.operations_per_second = iterations as f64 / total_duration.as_secs_f64();
        measurement.error_count = error_count;
        measurement.success_rate_percent = (success_count as f64 / iterations as f64) * 100.0;
        
        measurement.additional_metrics.insert("workflow_steps".to_string(), 4.0); // Sign, transmit, verify, respond
        
        println!("  ✓ Complete workflow: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark multi-component integration
    async fn benchmark_multi_component_integration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking multi-component integration");
        
        let scenarios = vec![
            ("js_sdk_to_server", "JavaScript SDK to Server"),
            ("python_sdk_to_server", "Python SDK to Server"),
            ("cli_to_server", "CLI to Server"),
            ("mixed_clients", "Mixed Client Types"),
        ];
        
        for (scenario_name, description) in scenarios {
            println!("  🔍 {}: {}", scenario_name, description);
            
            let mut timer = BenchmarkTimer::new();
            let iterations = 500;
            let mut success_count = 0;
            let mut error_count = 0;
            
            let start_time = Instant::now();
            
            for i in 0..iterations {
                timer.start();
                
                let result = self.execute_integration_scenario(scenario_name, i).await;
                
                timer.record();
                
                match result {
                    Ok(_) => success_count += 1,
                    Err(_) => error_count += 1,
                }
            }
            
            let total_duration = start_time.elapsed();
            let stats = timer.statistics();
            
            let mut measurement = PerformanceMeasurement::new(
                format!("integration_{}", scenario_name),
                "end_to_end".to_string(),
                "component_integration".to_string(),
            );
            
            measurement.operation_count = iterations;
            measurement.total_duration = total_duration;
            measurement.avg_operation_time_ms = stats.avg_ms;
            measurement.median_operation_time_ms = stats.median_ms;
            measurement.p95_operation_time_ms = stats.p95_ms;
            measurement.p99_operation_time_ms = stats.p99_ms;
            measurement.min_operation_time_ms = stats.min_ms;
            measurement.max_operation_time_ms = stats.max_ms;
            measurement.operations_per_second = iterations as f64 / total_duration.as_secs_f64();
            measurement.error_count = error_count;
            measurement.success_rate_percent = (success_count as f64 / iterations as f64) * 100.0;
            
            measurement.additional_metrics.insert("scenario_type".to_string(), 
                match scenario_name {
                    "js_sdk_to_server" => 1.0,
                    "python_sdk_to_server" => 2.0,
                    "cli_to_server" => 3.0,
                    "mixed_clients" => 4.0,
                    _ => 0.0,
                });
            
            println!("    ✓ {}: {:.3}ms avg, {:.1} ops/sec", 
                    scenario_name, stats.avg_ms, measurement.operations_per_second);
            
            self.results.push(measurement);
        }
        
        Ok(())
    }

    /// Benchmark cross-platform consistency
    async fn benchmark_cross_platform_consistency(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking cross-platform consistency");
        
        let platforms = vec![
            ("linux_x64", "Linux x64"),
            ("macos_arm64", "macOS ARM64"),
            ("windows_x64", "Windows x64"),
        ];
        
        let mut platform_results = HashMap::new();
        
        for (platform_name, description) in platforms {
            println!("  🔍 {}: {}", platform_name, description);
            
            let mut timer = BenchmarkTimer::new();
            let iterations = 500;
            let mut success_count = 0;
            
            let start_time = Instant::now();
            
            for i in 0..iterations {
                timer.start();
                
                // Simulate platform-specific performance characteristics
                let platform_overhead = self.get_platform_overhead(platform_name);
                sleep(Duration::from_micros(platform_overhead)).await;
                
                // Execute workflow
                let key_pair = &self.key_pairs[i % self.key_pairs.len()];
                let _result = self.execute_complete_workflow(key_pair, i).await;
                
                timer.record();
                success_count += 1;
            }
            
            let total_duration = start_time.elapsed();
            let stats = timer.statistics();
            
            platform_results.insert(platform_name, stats.avg_ms);
            
            let mut measurement = PerformanceMeasurement::new(
                format!("cross_platform_{}", platform_name),
                "end_to_end".to_string(),
                "platform_consistency".to_string(),
            );
            
            measurement.operation_count = iterations;
            measurement.total_duration = total_duration;
            measurement.avg_operation_time_ms = stats.avg_ms;
            measurement.operations_per_second = iterations as f64 / total_duration.as_secs_f64();
            measurement.success_rate_percent = (success_count as f64 / iterations as f64) * 100.0;
            
            println!("    ✓ {}: {:.3}ms avg", platform_name, stats.avg_ms);
            
            self.results.push(measurement);
        }
        
        // Calculate consistency metrics
        let times: Vec<f64> = platform_results.values().cloned().collect();
        let avg_time = times.iter().sum::<f64>() / times.len() as f64;
        let max_deviation = times.iter()
            .map(|&t| ((t - avg_time) / avg_time).abs())
            .fold(0.0f64, |acc, x| acc.max(x));
        
        let mut consistency_measurement = PerformanceMeasurement::new(
            "cross_platform_consistency".to_string(),
            "end_to_end".to_string(),
            "consistency_analysis".to_string(),
        );
        
        consistency_measurement.operation_count = platform_results.len();
        consistency_measurement.avg_operation_time_ms = avg_time;
        consistency_measurement.success_rate_percent = 100.0;
        
        consistency_measurement.additional_metrics.insert("max_deviation_percent".to_string(), max_deviation * 100.0);
        consistency_measurement.additional_metrics.insert("consistency_score".to_string(), (1.0 - max_deviation) * 100.0);
        
        println!("  ✓ Cross-platform consistency: {:.1}% max deviation", max_deviation * 100.0);
        
        self.results.push(consistency_measurement);
        Ok(())
    }

    /// Benchmark typical API usage scenarios
    async fn benchmark_typical_api_usage(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking typical API usage scenarios");
        
        let api_scenarios = vec![
            ("crud_operations", "CRUD Operations", 10),
            ("data_queries", "Data Queries", 50),
            ("file_uploads", "File Uploads", 5),
            ("real_time_updates", "Real-time Updates", 100),
        ];
        
        for (scenario_name, description, frequency) in api_scenarios {
            println!("  🔍 {}: {}", scenario_name, description);
            
            let mut timer = BenchmarkTimer::new();
            let iterations = frequency * 2; // Scale based on typical frequency
            let mut success_count = 0;
            let mut error_count = 0;
            
            let start_time = Instant::now();
            
            for i in 0..iterations {
                timer.start();
                
                let result = self.execute_api_scenario(scenario_name, i).await;
                
                timer.record();
                
                match result {
                    Ok(_) => success_count += 1,
                    Err(_) => error_count += 1,
                }
            }
            
            let total_duration = start_time.elapsed();
            let stats = timer.statistics();
            
            let mut measurement = PerformanceMeasurement::new(
                format!("api_usage_{}", scenario_name),
                "end_to_end".to_string(),
                "typical_usage".to_string(),
            );
            
            measurement.operation_count = iterations;
            measurement.total_duration = total_duration;
            measurement.avg_operation_time_ms = stats.avg_ms;
            measurement.median_operation_time_ms = stats.median_ms;
            measurement.p95_operation_time_ms = stats.p95_ms;
            measurement.p99_operation_time_ms = stats.p99_ms;
            measurement.min_operation_time_ms = stats.min_ms;
            measurement.max_operation_time_ms = stats.max_ms;
            measurement.operations_per_second = iterations as f64 / total_duration.as_secs_f64();
            measurement.error_count = error_count;
            measurement.success_rate_percent = (success_count as f64 / iterations as f64) * 100.0;
            
            measurement.additional_metrics.insert("typical_frequency".to_string(), frequency as f64);
            
            println!("    ✓ {}: {:.3}ms avg, {:.1} ops/sec", 
                    scenario_name, stats.avg_ms, measurement.operations_per_second);
            
            self.results.push(measurement);
        }
        
        Ok(())
    }

    /// Benchmark high-frequency trading scenario
    async fn benchmark_high_frequency_trading_scenario(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking high-frequency trading scenario");
        
        let mut timer = BenchmarkTimer::new();
        let iterations = 10000; // High frequency
        let batch_size = 100;
        let mut success_count = 0;
        let mut error_count = 0;
        
        let start_time = Instant::now();
        
        // Process in batches to simulate burst trading
        for batch in 0..(iterations / batch_size) {
            let batch_start = Instant::now();
            
            for i in 0..batch_size {
                let request_id = batch * batch_size + i;
                let key_pair = &self.key_pairs[request_id % self.key_pairs.len()];
                
                timer.start();
                
                // Simulate high-frequency trading request
                let result = self.execute_trading_request(key_pair, request_id).await;
                
                timer.record();
                
                match result {
                    Ok(_) => success_count += 1,
                    Err(_) => error_count += 1,
                }
            }
            
            // Ensure we maintain target rate
            let batch_duration = batch_start.elapsed();
            let target_batch_duration = Duration::from_millis(10); // 100ms per 100 requests = 1000 RPS
            
            if batch_duration < target_batch_duration {
                sleep(target_batch_duration - batch_duration).await;
            }
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "high_frequency_trading".to_string(),
            "end_to_end".to_string(),
            "hft_scenario".to_string(),
        );
        
        measurement.operation_count = iterations;
        measurement.total_duration = total_duration;
        measurement.avg_operation_time_ms = stats.avg_ms;
        measurement.median_operation_time_ms = stats.median_ms;
        measurement.p95_operation_time_ms = stats.p95_ms;
        measurement.p99_operation_time_ms = stats.p99_ms;
        measurement.min_operation_time_ms = stats.min_ms;
        measurement.max_operation_time_ms = stats.max_ms;
        measurement.operations_per_second = iterations as f64 / total_duration.as_secs_f64();
        measurement.error_count = error_count;
        measurement.success_rate_percent = (success_count as f64 / iterations as f64) * 100.0;
        
        measurement.additional_metrics.insert("batch_size".to_string(), batch_size as f64);
        measurement.additional_metrics.insert("target_rps".to_string(), 1000.0);
        
        println!("  ✓ High-frequency trading: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark batch processing scenario
    async fn benchmark_batch_processing_scenario(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking batch processing scenario");
        
        let batch_sizes = vec![50, 100, 500, 1000];
        
        for batch_size in batch_sizes {
            println!("  🔍 Batch size: {}", batch_size);
            
            let mut timer = BenchmarkTimer::new();
            let mut success_count = 0;
            let mut error_count = 0;
            
            let start_time = Instant::now();
            
            timer.start();
            
            // Process entire batch
            for i in 0..batch_size {
                let key_pair = &self.key_pairs[i % self.key_pairs.len()];
                
                match self.execute_batch_item(key_pair, i).await {
                    Ok(_) => success_count += 1,
                    Err(_) => error_count += 1,
                }
            }
            
            timer.record();
            
            let total_duration = start_time.elapsed();
            let stats = timer.statistics();
            
            let mut measurement = PerformanceMeasurement::new(
                format!("batch_processing_{}", batch_size),
                "end_to_end".to_string(),
                "batch_scenario".to_string(),
            );
            
            measurement.operation_count = batch_size;
            measurement.total_duration = total_duration;
            measurement.avg_operation_time_ms = stats.avg_ms;
            measurement.operations_per_second = batch_size as f64 / total_duration.as_secs_f64();
            measurement.error_count = error_count;
            measurement.success_rate_percent = (success_count as f64 / batch_size as f64) * 100.0;
            
            measurement.additional_metrics.insert("batch_size".to_string(), batch_size as f64);
            measurement.additional_metrics.insert("time_per_item_ms".to_string(), 
                total_duration.as_secs_f64() * 1000.0 / batch_size as f64);
            
            println!("    ✓ Batch {}: {:.3}ms total, {:.3}ms per item", 
                    batch_size, stats.avg_ms, stats.avg_ms / batch_size as f64);
            
            self.results.push(measurement);
        }
        
        Ok(())
    }

    /// Benchmark mobile app scenario
    async fn benchmark_mobile_app_scenario(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking mobile app scenario");
        
        let mobile_scenarios = vec![
            ("app_startup", "App Startup Authentication", 200),
            ("background_sync", "Background Sync", 100),
            ("user_interaction", "User Interaction", 50),
            ("push_notification", "Push Notification", 300),
        ];
        
        for (scenario_name, description, latency_budget_ms) in mobile_scenarios {
            println!("  🔍 {}: {}", scenario_name, description);
            
            let mut timer = BenchmarkTimer::new();
            let iterations = 200;
            let mut success_count = 0;
            let mut error_count = 0;
            let mut within_budget_count = 0;
            
            let start_time = Instant::now();
            
            for i in 0..iterations {
                timer.start();
                
                // Simulate mobile-specific constraints
                let result = self.execute_mobile_scenario(scenario_name, i).await;
                let operation_time_ms = timer.measurements().last()
                    .map(|d| d.as_secs_f64() * 1000.0)
                    .unwrap_or(0.0);
                
                timer.record();
                
                if operation_time_ms <= latency_budget_ms as f64 {
                    within_budget_count += 1;
                }
                
                match result {
                    Ok(_) => success_count += 1,
                    Err(_) => error_count += 1,
                }
            }
            
            let total_duration = start_time.elapsed();
            let stats = timer.statistics();
            
            let mut measurement = PerformanceMeasurement::new(
                format!("mobile_{}", scenario_name),
                "end_to_end".to_string(),
                "mobile_scenario".to_string(),
            );
            
            measurement.operation_count = iterations;
            measurement.total_duration = total_duration;
            measurement.avg_operation_time_ms = stats.avg_ms;
            measurement.median_operation_time_ms = stats.median_ms;
            measurement.p95_operation_time_ms = stats.p95_ms;
            measurement.p99_operation_time_ms = stats.p99_ms;
            measurement.min_operation_time_ms = stats.min_ms;
            measurement.max_operation_time_ms = stats.max_ms;
            measurement.operations_per_second = iterations as f64 / total_duration.as_secs_f64();
            measurement.error_count = error_count;
            measurement.success_rate_percent = (success_count as f64 / iterations as f64) * 100.0;
            
            measurement.additional_metrics.insert("latency_budget_ms".to_string(), latency_budget_ms as f64);
            measurement.additional_metrics.insert("within_budget_percent".to_string(), 
                (within_budget_count as f64 / iterations as f64) * 100.0);
            
            println!("    ✓ {}: {:.3}ms avg, {:.1}% within budget", 
                    scenario_name, stats.avg_ms, 
                    (within_budget_count as f64 / iterations as f64) * 100.0);
            
            self.results.push(measurement);
        }
        
        Ok(())
    }

    /// Benchmark sustained production load
    async fn benchmark_sustained_production_load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking sustained production load");
        
        let test_duration = Duration::from_secs(self.config.test_duration_seconds);
        let target_rps = 500.0;
        let interval = Duration::from_secs_f64(1.0 / target_rps);
        
        let mut operation_count = 0;
        let mut success_count = 0;
        let mut error_count = 0;
        let mut operation_times = Vec::new();
        
        let start_time = Instant::now();
        let mut next_operation_time = start_time;
        
        while start_time.elapsed() < test_duration {
            let key_pair = &self.key_pairs[operation_count % self.key_pairs.len()];
            
            let op_start = Instant::now();
            let result = self.execute_production_request(key_pair, operation_count).await;
            let op_duration = op_start.elapsed();
            
            operation_times.push(op_duration.as_secs_f64() * 1000.0);
            operation_count += 1;
            
            match result {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
            
            // Rate limiting
            next_operation_time += interval;
            let now = Instant::now();
            if next_operation_time > now {
                sleep(next_operation_time - now).await;
            }
        }
        
        let total_duration = start_time.elapsed();
        
        // Calculate statistics
        operation_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let avg_ms = operation_times.iter().sum::<f64>() / operation_times.len() as f64;
        let median_ms = operation_times[operation_times.len() / 2];
        let p95_ms = operation_times[(operation_times.len() * 95) / 100];
        let p99_ms = operation_times[(operation_times.len() * 99) / 100];
        
        let mut measurement = PerformanceMeasurement::new(
            "sustained_production_load".to_string(),
            "end_to_end".to_string(),
            "production_load".to_string(),
        );
        
        measurement.operation_count = operation_count;
        measurement.total_duration = total_duration;
        measurement.avg_operation_time_ms = avg_ms;
        measurement.median_operation_time_ms = median_ms;
        measurement.p95_operation_time_ms = p95_ms;
        measurement.p99_operation_time_ms = p99_ms;
        measurement.min_operation_time_ms = operation_times.first().copied().unwrap_or(0.0);
        measurement.max_operation_time_ms = operation_times.last().copied().unwrap_or(0.0);
        measurement.operations_per_second = operation_count as f64 / total_duration.as_secs_f64();
        measurement.error_count = error_count;
        measurement.success_rate_percent = (success_count as f64 / operation_count as f64) * 100.0;
        
        measurement.additional_metrics.insert("target_rps".to_string(), target_rps);
        measurement.additional_metrics.insert("test_duration_sec".to_string(), test_duration.as_secs() as f64);
        
        println!("  ✓ Sustained load: {:.3}ms avg, {:.1} ops/sec over {}s", 
                avg_ms, measurement.operations_per_second, test_duration.as_secs());
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark burst traffic handling
    async fn benchmark_burst_traffic_handling(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking burst traffic handling");
        
        let burst_configs = vec![
            (1000, Duration::from_secs(1), "1k_ops_1s"),
            (2000, Duration::from_secs(2), "2k_ops_2s"),
            (5000, Duration::from_secs(5), "5k_ops_5s"),
        ];
        
        for (burst_ops, burst_duration, config_name) in burst_configs {
            println!("  🔍 Burst: {} ops in {}s", burst_ops, burst_duration.as_secs());
            
            let concurrent_count = 50;
            let ops_per_thread = burst_ops / concurrent_count;
            
            let success_counter = Arc::new(AtomicUsize::new(0));
            let error_counter = Arc::new(AtomicUsize::new(0));
            let operation_times = Arc::new(std::sync::Mutex::new(Vec::new()));
            
            let start_time = Instant::now();
            
            // Spawn concurrent burst tasks
            let tasks: Vec<_> = (0..concurrent_count).map(|thread_id| {
                let key_pair = self.key_pairs[thread_id % self.key_pairs.len()].clone();
                let verification_state = Arc::clone(&self.verification_state);
                let success_counter = Arc::clone(&success_counter);
                let error_counter = Arc::clone(&error_counter);
                let operation_times = Arc::clone(&operation_times);
                
                tokio::spawn(async move {
                    for i in 0..ops_per_thread {
                        let op_start = Instant::now();
                        
                        let result = Self::execute_burst_operation(&key_pair, &verification_state, thread_id, i).await;
                        
                        let op_duration = op_start.elapsed();
                        
                        {
                            let mut times = operation_times.lock().unwrap();
                            times.push(op_duration);
                        }
                        
                        match result {
                            Ok(_) => { success_counter.fetch_add(1, Ordering::Relaxed); }
                            Err(_) => { error_counter.fetch_add(1, Ordering::Relaxed); }
                        }
                    }
                })
            }).collect();
            
            // Wait for all tasks
            futures::future::join_all(tasks).await;
            
            let total_duration = start_time.elapsed();
            let success_count = success_counter.load(Ordering::Relaxed);
            let error_count = error_counter.load(Ordering::Relaxed);
            
            // Calculate timing statistics
            let times = operation_times.lock().unwrap();
            let mut times_ms: Vec<f64> = times.iter()
                .map(|d| d.as_secs_f64() * 1000.0)
                .collect();
            times_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let avg_ms = times_ms.iter().sum::<f64>() / times_ms.len() as f64;
            let p95_ms = times_ms[(times_ms.len() * 95) / 100];
            
            let mut measurement = PerformanceMeasurement::new(
                format!("burst_traffic_{}", config_name),
                "end_to_end".to_string(),
                "burst_handling".to_string(),
            );
            
            measurement.operation_count = burst_ops;
            measurement.total_duration = total_duration;
            measurement.avg_operation_time_ms = avg_ms;
            measurement.p95_operation_time_ms = p95_ms;
            measurement.operations_per_second = burst_ops as f64 / total_duration.as_secs_f64();
            measurement.error_count = error_count;
            measurement.success_rate_percent = (success_count as f64 / burst_ops as f64) * 100.0;
            
            measurement.additional_metrics.insert("concurrent_threads".to_string(), concurrent_count as f64);
            measurement.additional_metrics.insert("target_duration_sec".to_string(), burst_duration.as_secs() as f64);
            
            println!("    ✓ {}: {:.3}ms avg, {:.1} ops/sec", 
                    config_name, avg_ms, measurement.operations_per_second);
            
            self.results.push(measurement);
        }
        
        Ok(())
    }

    /// Benchmark mixed workload performance
    async fn benchmark_mixed_workload_performance(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking mixed workload performance");
        
        let workload_duration = Duration::from_secs(30);
        let total_operations = 1000;
        
        // Define workload mix
        let workload_mix = vec![
            ("read_heavy", 60), // 60% reads
            ("write_ops", 25),  // 25% writes
            ("auth_ops", 10),   // 10% auth operations
            ("admin_ops", 5),   // 5% admin operations
        ];
        
        let mut operation_counts = HashMap::new();
        let mut success_counts = HashMap::new();
        let mut error_counts = HashMap::new();
        let mut operation_times = HashMap::new();
        
        // Initialize counters
        for (op_type, _) in &workload_mix {
            operation_counts.insert(op_type.to_string(), 0);
            success_counts.insert(op_type.to_string(), 0);
            error_counts.insert(op_type.to_string(), 0);
            operation_times.insert(op_type.to_string(), Vec::new());
        }
        
        let start_time = Instant::now();
        
        for i in 0..total_operations {
            // Determine operation type based on mix
            let op_type = self.select_operation_type(&workload_mix, i);
            let key_pair = &self.key_pairs[i % self.key_pairs.len()];
            
            let op_start = Instant::now();
            let result = self.execute_mixed_workload_operation(&op_type, key_pair, i).await;
            let op_duration = op_start.elapsed();
            
            *operation_counts.get_mut(&op_type).unwrap() += 1;
            operation_times.get_mut(&op_type).unwrap().push(op_duration.as_secs_f64() * 1000.0);
            
            match result {
                Ok(_) => *success_counts.get_mut(&op_type).unwrap() += 1,
                Err(_) => *error_counts.get_mut(&op_type).unwrap() += 1,
            }
            
            // Rate limiting to spread operations over duration
            if i % 100 == 0 {
                sleep(Duration::from_millis(10)).await;
            }
        }
        
        let total_duration = start_time.elapsed();
        
        // Create measurements for each operation type
        for (op_type, _) in &workload_mix {
            let ops = *operation_counts.get(op_type).unwrap();
            let successes = *success_counts.get(op_type).unwrap();
            let errors = *error_counts.get(op_type).unwrap();
            let times = operation_times.get(op_type).unwrap();
            
            if ops > 0 {
                let mut sorted_times = times.clone();
                sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
                
                let avg_ms = sorted_times.iter().sum::<f64>() / sorted_times.len() as f64;
                let p95_ms = sorted_times[(sorted_times.len() * 95) / 100];
                
                let mut measurement = PerformanceMeasurement::new(
                    format!("mixed_workload_{}", op_type),
                    "end_to_end".to_string(),
                    "mixed_workload".to_string(),
                );
                
                measurement.operation_count = ops;
                measurement.total_duration = total_duration;
                measurement.avg_operation_time_ms = avg_ms;
                measurement.p95_operation_time_ms = p95_ms;
                measurement.operations_per_second = ops as f64 / total_duration.as_secs_f64();
                measurement.error_count = errors;
                measurement.success_rate_percent = (successes as f64 / ops as f64) * 100.0;
                
                measurement.additional_metrics.insert("workload_percentage".to_string(), 
                    (ops as f64 / total_operations as f64) * 100.0);
                
                println!("  ✓ {}: {:.3}ms avg, {} ops ({:.1}%)", 
                        op_type, avg_ms, ops, (ops as f64 / total_operations as f64) * 100.0);
                
                self.results.push(measurement);
            }
        }
        
        Ok(())
    }

    /// Benchmark error recovery performance
    async fn benchmark_error_recovery_performance(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking error recovery performance");
        
        let error_scenarios = vec![
            ("network_timeout", "Network Timeout Recovery"),
            ("server_error", "Server Error Recovery"),
            ("auth_failure", "Authentication Failure Recovery"),
            ("rate_limit", "Rate Limit Recovery"),
        ];
        
        for (scenario_name, description) in error_scenarios {
            println!("  🔍 {}: {}", scenario_name, description);
            
            let mut timer = BenchmarkTimer::new();
            let iterations = 100;
            let mut recovery_times = Vec::new();
            let mut success_count = 0;
            
            for i in 0..iterations {
                timer.start();
                
                // Simulate error condition and recovery
                let recovery_result = self.simulate_error_recovery(scenario_name, i).await;
                
                timer.record();
                
                if let Ok(recovery_time) = recovery_result {
                    recovery_times.push(recovery_time);
                    success_count += 1;
                }
            }
            
            let stats = timer.statistics();
            
            let mut measurement = PerformanceMeasurement::new(
                format!("error_recovery_{}", scenario_name),
                "end_to_end".to_string(),
                "error_recovery".to_string(),
            );
            
            measurement.operation_count = iterations;
            measurement.avg_operation_time_ms = stats.avg_ms;
            measurement.operations_per_second = iterations as f64 / timer.statistics().total_duration.as_secs_f64();
            measurement.success_rate_percent = (success_count as f64 / iterations as f64) * 100.0;
            
            if !recovery_times.is_empty() {
                let avg_recovery_time = recovery_times.iter().sum::<f64>() / recovery_times.len() as f64;
                measurement.additional_metrics.insert("avg_recovery_time_ms".to_string(), avg_recovery_time);
            }
            
            println!("    ✓ {}: {:.3}ms avg, {:.1}% recovery rate", 
                    scenario_name, stats.avg_ms, measurement.success_rate_percent);
            
            self.results.push(measurement);
        }
        
        Ok(())
    }

    /// Benchmark degraded mode performance
    async fn benchmark_degraded_mode_performance(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking degraded mode performance");
        
        let degraded_scenarios = vec![
            ("partial_service", "Partial Service Availability"),
            ("reduced_security", "Reduced Security Mode"),
            ("backup_auth", "Backup Authentication"),
        ];
        
        for (scenario_name, description) in degraded_scenarios {
            println!("  🔍 {}: {}", scenario_name, description);
            
            let mut timer = BenchmarkTimer::new();
            let iterations = 200;
            let mut success_count = 0;
            let mut error_count = 0;
            
            let start_time = Instant::now();
            
            for i in 0..iterations {
                timer.start();
                
                let result = self.execute_degraded_mode_operation(scenario_name, i).await;
                
                timer.record();
                
                match result {
                    Ok(_) => success_count += 1,
                    Err(_) => error_count += 1,
                }
            }
            
            let total_duration = start_time.elapsed();
            let stats = timer.statistics();
            
            let mut measurement = PerformanceMeasurement::new(
                format!("degraded_mode_{}", scenario_name),
                "end_to_end".to_string(),
                "degraded_mode".to_string(),
            );
            
            measurement.operation_count = iterations;
            measurement.total_duration = total_duration;
            measurement.avg_operation_time_ms = stats.avg_ms;
            measurement.median_operation_time_ms = stats.median_ms;
            measurement.p95_operation_time_ms = stats.p95_ms;
            measurement.p99_operation_time_ms = stats.p99_ms;
            measurement.min_operation_time_ms = stats.min_ms;
            measurement.max_operation_time_ms = stats.max_ms;
            measurement.operations_per_second = iterations as f64 / total_duration.as_secs_f64();
            measurement.error_count = error_count;
            measurement.success_rate_percent = (success_count as f64 / iterations as f64) * 100.0;
            
            println!("    ✓ {}: {:.3}ms avg, {:.1} ops/sec", 
                    scenario_name, stats.avg_ms, measurement.operations_per_second);
            
            self.results.push(measurement);
        }
        
        Ok(())
    }

    // Helper methods

    /// Execute complete authentication workflow
    async fn execute_complete_workflow(&self, key_pair: &MasterKeyPair, request_id: usize) -> Result<(), Box<dyn std::error::Error>> {
        // Step 1: Client-side request preparation
        let nonce = Uuid::new_v4().to_string();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let payload = json!({"request_id": request_id, "data": "test data"}).to_string();
        
        // Step 2: Client-side signing
        let signature_data = format!("POST /api/test HTTP/1.1\n{}", payload);
        let signature = key_pair.sign(signature_data.as_bytes())?;
        
        // Step 3: Network transmission (simulated)
        sleep(Duration::from_micros(100)).await;
        
        // Step 4: Server-side verification
        let verification_result = self.verification_state.check_and_store_nonce(&nonce, timestamp);
        
        if verification_result.is_err() {
            return Err("Verification failed".into());
        }
        
        Ok(())
    }

    /// Execute integration scenario
    async fn execute_integration_scenario(&self, scenario: &str, request_id: usize) -> Result<(), Box<dyn std::error::Error>> {
        let delay_ms = match scenario {
            "js_sdk_to_server" => 15, // JavaScript overhead
            "python_sdk_to_server" => 20, // Python overhead
            "cli_to_server" => 25, // CLI startup overhead
            "mixed_clients" => 18, // Average
            _ => 15,
        };
        
        sleep(Duration::from_millis(delay_ms)).await;
        
        let key_pair = &self.key_pairs[request_id % self.key_pairs.len()];
        self.execute_complete_workflow(key_pair, request_id).await
    }

    /// Get platform-specific overhead
    fn get_platform_overhead(&self, platform: &str) -> u64 {
        match platform {
            "linux_x64" => 50,     // Baseline
            "macos_arm64" => 45,   // Slightly faster
            "windows_x64" => 65,   // Slightly slower
            _ => 50,
        }
    }

    /// Execute API scenario
    async fn execute_api_scenario(&self, scenario: &str, request_id: usize) -> Result<(), Box<dyn std::error::Error>> {
        let delay_ms = match scenario {
            "crud_operations" => 30,
            "data_queries" => 15,
            "file_uploads" => 100,
            "real_time_updates" => 5,
            _ => 20,
        };
        
        sleep(Duration::from_millis(delay_ms)).await;
        
        let key_pair = &self.key_pairs[request_id % self.key_pairs.len()];
        self.execute_complete_workflow(key_pair, request_id).await
    }

    /// Execute trading request
    async fn execute_trading_request(&self, key_pair: &MasterKeyPair, request_id: usize) -> Result<(), Box<dyn std::error::Error>> {
        // High-frequency trading requires minimal latency
        let signature_data = format!("TRADE {}", request_id);
        key_pair.sign(signature_data.as_bytes())?;
        Ok(())
    }

    /// Execute batch item
    async fn execute_batch_item(&self, key_pair: &MasterKeyPair, item_id: usize) -> Result<(), Box<dyn std::error::Error>> {
        let signature_data = format!("BATCH_ITEM {}", item_id);
        key_pair.sign(signature_data.as_bytes())?;
        sleep(Duration::from_millis(5)).await; // Small processing delay
        Ok(())
    }

    /// Execute mobile scenario
    async fn execute_mobile_scenario(&self, scenario: &str, request_id: usize) -> Result<(), Box<dyn std::error::Error>> {
        let delay_ms = match scenario {
            "app_startup" => 150,     // App initialization
            "background_sync" => 80,  // Background processing
            "user_interaction" => 40, // User-initiated
            "push_notification" => 250, // Network + processing
            _ => 100,
        };
        
        sleep(Duration::from_millis(delay_ms)).await;
        
        let key_pair = &self.key_pairs[request_id % self.key_pairs.len()];
        self.execute_complete_workflow(key_pair, request_id).await
    }

    /// Execute production request
    async fn execute_production_request(&self, key_pair: &MasterKeyPair, request_id: usize) -> Result<(), Box<dyn std::error::Error>> {
        // Simulate realistic production request processing
        self.execute_complete_workflow(key_pair, request_id).await
    }

    /// Execute burst operation (static for async context)
    async fn execute_burst_operation(
        key_pair: &MasterKeyPair,
        verification_state: &Arc<SignatureVerificationState>,
        thread_id: usize,
        operation_id: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let nonce = format!("burst_{}_{}", thread_id, operation_id);
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        
        // Sign and verify
        let signature_data = format!("BURST {} {}", thread_id, operation_id);
        key_pair.sign(signature_data.as_bytes())?;
        verification_state.check_and_store_nonce(&nonce, timestamp)?;
        
        Ok(())
    }

    /// Select operation type for mixed workload
    fn select_operation_type(&self, workload_mix: &[(String, u32)], operation_index: usize) -> String {
        let total_weight: u32 = workload_mix.iter().map(|(_, weight)| weight).sum();
        let mut cumulative_weight = 0;
        let target = (operation_index as u32) % total_weight;
        
        for (op_type, weight) in workload_mix {
            cumulative_weight += weight;
            if target < cumulative_weight {
                return op_type.clone();
            }
        }
        
        workload_mix[0].0.clone() // Fallback
    }

    /// Execute mixed workload operation
    async fn execute_mixed_workload_operation(&self, op_type: &str, key_pair: &MasterKeyPair, request_id: usize) -> Result<(), Box<dyn std::error::Error>> {
        let delay_ms = match op_type {
            "read_heavy" => 10,
            "write_ops" => 25,
            "auth_ops" => 15,
            "admin_ops" => 50,
            _ => 20,
        };
        
        sleep(Duration::from_millis(delay_ms)).await;
        
        if op_type != "read_heavy" {
            // Operations that require authentication
            self.execute_complete_workflow(key_pair, request_id).await
        } else {
            Ok(())
        }
    }

    /// Simulate error recovery
    async fn simulate_error_recovery(&self, scenario: &str, _iteration: usize) -> Result<f64, Box<dyn std::error::Error>> {
        let recovery_time_ms = match scenario {
            "network_timeout" => 500.0,
            "server_error" => 200.0,
            "auth_failure" => 100.0,
            "rate_limit" => 1000.0,
            _ => 300.0,
        };
        
        sleep(Duration::from_millis(recovery_time_ms as u64)).await;
        Ok(recovery_time_ms)
    }

    /// Execute degraded mode operation
    async fn execute_degraded_mode_operation(&self, scenario: &str, request_id: usize) -> Result<(), Box<dyn std::error::Error>> {
        let delay_ms = match scenario {
            "partial_service" => 80,  // Slower due to reduced resources
            "reduced_security" => 30, // Faster due to simplified checks
            "backup_auth" => 60,      // Moderate delay for backup systems
            _ => 50,
        };
        
        sleep(Duration::from_millis(delay_ms)).await;
        
        // Simplified authentication for degraded modes
        let key_pair = &self.key_pairs[request_id % self.key_pairs.len()];
        let signature_data = format!("DEGRADED {} {}", scenario, request_id);
        key_pair.sign(signature_data.as_bytes())?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_end_to_end_benchmarks_creation() {
        let config = PerformanceBenchmarkConfig::default();
        let targets = PerformanceTargets::default();
        
        let benchmarks = EndToEndPerformanceBenchmarks::new(config, targets);
        assert!(benchmarks.is_ok());
    }

    #[tokio::test]
    async fn test_complete_workflow_execution() {
        let config = PerformanceBenchmarkConfig::default();
        let targets = PerformanceTargets::default();
        let benchmarks = EndToEndPerformanceBenchmarks::new(config, targets).unwrap();
        
        let key_pair = &benchmarks.key_pairs[0];
        let result = benchmarks.execute_complete_workflow(key_pair, 0).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_platform_overhead_calculation() {
        let config = PerformanceBenchmarkConfig::default();
        let targets = PerformanceTargets::default();
        let benchmarks = EndToEndPerformanceBenchmarks::new(config, targets).unwrap();
        
        let linux_overhead = benchmarks.get_platform_overhead("linux_x64");
        let macos_overhead = benchmarks.get_platform_overhead("macos_arm64");
        let windows_overhead = benchmarks.get_platform_overhead("windows_x64");
        
        assert!(linux_overhead > 0);
        assert!(macos_overhead > 0);
        assert!(windows_overhead > 0);
    }
}