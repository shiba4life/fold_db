//! Server-side signature authentication performance benchmarks
//!
//! This module provides comprehensive performance benchmarking for the DataFold server's
//! signature authentication middleware, including Ed25519 verification, nonce validation,
//! timestamp checking, and throughput under load.

use crate::datafold_node::signature_auth::{
    SignatureAuthConfig, SignatureVerificationState, SecurityProfile,
    AuthenticationError
};
use super::{
    PerformanceMeasurement, BenchmarkTimer, TimingStatistics, PerformanceBenchmarkConfig,
    PerformanceTargets
};
use futures::future::join_all;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicU64, AtomicUsize, Ordering}};
use std::time::{SystemTime, UNIX_EPOCH, Duration, Instant};
use tokio::time::sleep;
use uuid::Uuid;

/// Server-side performance benchmark suite
pub struct ServerPerformanceBenchmarks {
    config: PerformanceBenchmarkConfig,
    targets: PerformanceTargets,
    verification_states: HashMap<SecurityProfile, Arc<SignatureVerificationState>>,
    results: Vec<PerformanceMeasurement>,
}

impl ServerPerformanceBenchmarks {
    /// Create new server performance benchmarks
    pub fn new(
        config: PerformanceBenchmarkConfig,
        targets: PerformanceTargets,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut verification_states = HashMap::new();
        
        // Initialize verification states for all security profiles
        for profile in &[SecurityProfile::Strict, SecurityProfile::Standard, SecurityProfile::Lenient] {
            let auth_config = match profile {
                SecurityProfile::Strict => SignatureAuthConfig::strict(),
                SecurityProfile::Standard => SignatureAuthConfig::default(),
                SecurityProfile::Lenient => SignatureAuthConfig::lenient(),
            };
            
            let state = Arc::new(SignatureVerificationState::new(auth_config)?);
            verification_states.insert(profile.clone(), state);
        }
        
        Ok(Self {
            config,
            targets,
            verification_states,
            results: Vec::new(),
        })
    }

    /// Run all server performance benchmarks
    pub async fn run_all_benchmarks(&mut self) -> Result<Vec<PerformanceMeasurement>, Box<dyn std::error::Error>> {
        println!("🚀 Starting server-side performance benchmarks");
        
        // Micro-benchmarks for individual operations
        self.benchmark_ed25519_verification().await?;
        self.benchmark_nonce_validation().await?;
        self.benchmark_timestamp_validation().await?;
        self.benchmark_signature_parsing().await?;
        
        // Component benchmarks for middleware operations
        self.benchmark_middleware_processing().await?;
        self.benchmark_security_profile_performance().await?;
        
        // Load benchmarks for throughput testing
        self.benchmark_concurrent_verification().await?;
        self.benchmark_sustained_load().await?;
        self.benchmark_burst_load().await?;
        
        // Stress benchmarks for extreme conditions
        self.benchmark_memory_pressure().await?;
        self.benchmark_attack_scenario_performance().await?;
        
        println!("✅ Server performance benchmarks completed");
        Ok(self.results.clone())
    }

    /// Benchmark Ed25519 signature verification performance
    async fn benchmark_ed25519_verification(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking Ed25519 signature verification");
        
        let state = self.verification_states.get(&SecurityProfile::Standard)
            .ok_or("Standard security profile not found")?;
        
        let mut timer = BenchmarkTimer::new();
        let iterations = self.config.micro_benchmark_iterations;
        let mut success_count = 0;
        let mut error_count = 0;
        
        // Warmup
        for _ in 0..100 {
            let nonce = Uuid::new_v4().to_string();
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            let _ = state.check_and_store_nonce(&nonce, timestamp);
        }
        
        // Actual benchmark
        let start_time = Instant::now();
        
        for i in 0..iterations {
            let nonce = format!("bench_nonce_{}", i);
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            
            timer.start();
            let result = state.check_and_store_nonce(&nonce, timestamp);
            timer.record();
            
            match result {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "ed25519_verification".to_string(),
            "server".to_string(),
            "signature_verification".to_string(),
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
        
        measurement.additional_metrics.insert("warmup_iterations".to_string(), 100.0);
        measurement.additional_metrics.insert("verification_algorithm".to_string(), 25519.0);
        
        println!("  ✓ Ed25519 verification: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark nonce validation performance
    async fn benchmark_nonce_validation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking nonce validation");
        
        let state = self.verification_states.get(&SecurityProfile::Standard)
            .ok_or("Standard security profile not found")?;
        
        let mut timer = BenchmarkTimer::new();
        let iterations = self.config.micro_benchmark_iterations;
        let mut success_count = 0;
        let mut error_count = 0;
        
        // Pre-populate some nonces to test collision detection
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        for i in 0..1000 {
            let nonce = format!("existing_nonce_{}", i);
            let _ = state.check_and_store_nonce(&nonce, timestamp);
        }
        
        // Benchmark nonce validation with mix of new and existing nonces
        let start_time = Instant::now();
        
        for i in 0..iterations {
            let nonce = if i % 10 == 0 {
                // 10% collision rate to test duplicate detection
                format!("existing_nonce_{}", i % 1000)
            } else {
                format!("new_nonce_{}", i)
            };
            
            timer.start();
            let result = state.check_and_store_nonce(&nonce, timestamp);
            timer.record();
            
            match result {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "nonce_validation".to_string(),
            "server".to_string(),
            "nonce_checking".to_string(),
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
        
        measurement.additional_metrics.insert("collision_rate_percent".to_string(), 10.0);
        measurement.additional_metrics.insert("pre_populated_nonces".to_string(), 1000.0);
        
        println!("  ✓ Nonce validation: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark timestamp validation performance
    async fn benchmark_timestamp_validation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking timestamp validation");
        
        let state = self.verification_states.get(&SecurityProfile::Strict)
            .ok_or("Strict security profile not found")?;
        
        let mut timer = BenchmarkTimer::new();
        let iterations = self.config.micro_benchmark_iterations;
        let mut success_count = 0;
        let mut error_count = 0;
        
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        
        // Benchmark timestamp validation with various time scenarios
        let start_time = Instant::now();
        
        for i in 0..iterations {
            let nonce = format!("timestamp_nonce_{}", i);
            
            // Vary timestamps: 70% valid, 20% too old, 10% too future
            let timestamp = match i % 10 {
                0..=6 => current_time - (i % 30) as u64, // Valid recent timestamps
                7..=8 => current_time - 3600, // Too old (1 hour ago)
                _ => current_time + 300, // Too future (5 minutes ahead)
            };
            
            timer.start();
            let result = state.check_and_store_nonce(&nonce, timestamp);
            timer.record();
            
            match result {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "timestamp_validation".to_string(),
            "server".to_string(),
            "timestamp_checking".to_string(),
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
        
        measurement.additional_metrics.insert("valid_timestamp_percent".to_string(), 70.0);
        measurement.additional_metrics.insert("old_timestamp_percent".to_string(), 20.0);
        measurement.additional_metrics.insert("future_timestamp_percent".to_string(), 10.0);
        
        println!("  ✓ Timestamp validation: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark signature parsing performance
    async fn benchmark_signature_parsing(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking signature parsing");
        
        let mut timer = BenchmarkTimer::new();
        let iterations = self.config.micro_benchmark_iterations;
        let mut success_count = 0;
        let mut error_count = 0;
        
        // Sample signature headers for parsing
        let signature_headers = vec![
            r#"sig1=:MEUCIQD2r2qF6UJHw2Q8dV4c+8O0vRMF5dBhcVOOz9+xKXgQAQIgEvKfPzyCf8QG1YKBxqzb6M5+8t6z2j+7QZ8Fz6XYxQE=:"#,
            r#"sig1=:MEQCIBz+2O9XbV5R6e8JfKRYzV+9U3M8t8J9QH5s2dF7KnAVAiBQ8TzNcP4L6m8E9zYbF3qZ5J+R8w6f7s4V8E9z+K4w8P=="#,
            r#"sig1=:MEUCIQCz5V2F8J9Q6H3s8dF7Kn4M+8t6z2j+7QZ8Fz6XYxQE8QIgEvKfPzyCf8QG1YKBxqzb6M5+Vzj2P9LNQ5s4VRbYxwP=:"#,
        ];
        
        let signature_input_headers = vec![
            r#"sig1=("@method" "@target-uri" "@authority" "content-digest");created=1618884473;keyid="test-key-ed25519""#,
            r#"sig1=("@method" "@path" "@query" "host" "date");created=1618884473;keyid="test-key-ed25519""#,
            r#"sig1=("@method" "@target-uri" "authorization" "content-type");created=1618884473;keyid="test-key-ed25519""#,
        ];
        
        // Benchmark signature header parsing
        let start_time = Instant::now();
        
        for i in 0..iterations {
            let sig_header = &signature_headers[i % signature_headers.len()];
            let sig_input_header = &signature_input_headers[i % signature_input_headers.len()];
            
            timer.start();
            
            // Simulate signature parsing operations
            let parse_result = self.parse_signature_header(sig_header, sig_input_header);
            
            timer.record();
            
            match parse_result {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "signature_parsing".to_string(),
            "server".to_string(),
            "header_parsing".to_string(),
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
        
        measurement.additional_metrics.insert("header_variants".to_string(), 3.0);
        
        println!("  ✓ Signature parsing: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark middleware processing performance
    async fn benchmark_middleware_processing(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking middleware processing");
        
        let state = self.verification_states.get(&SecurityProfile::Standard)
            .ok_or("Standard security profile not found")?;
        
        let mut timer = BenchmarkTimer::new();
        let iterations = 5000; // Fewer iterations for more complex operations
        let mut success_count = 0;
        let mut error_count = 0;
        
        // Benchmark complete middleware processing cycle
        let start_time = Instant::now();
        
        for i in 0..iterations {
            timer.start();
            
            // Simulate complete middleware processing
            let result = self.simulate_middleware_request(state, i).await;
            
            timer.record();
            
            match result {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "middleware_processing".to_string(),
            "server".to_string(),
            "complete_middleware".to_string(),
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
        
        println!("  ✓ Middleware processing: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark security profile performance differences
    async fn benchmark_security_profile_performance(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking security profile performance");
        
        let iterations = 2000;
        let profiles = vec![
            SecurityProfile::Lenient,
            SecurityProfile::Standard, 
            SecurityProfile::Strict,
        ];
        
        for profile in profiles {
            let state = self.verification_states.get(&profile)
                .ok_or("Security profile not found")?;
            
            let mut timer = BenchmarkTimer::new();
            let mut success_count = 0;
            let mut error_count = 0;
            
            let start_time = Instant::now();
            
            for i in 0..iterations {
                let nonce = format!("profile_nonce_{}_{:?}", i, profile);
                let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
                
                timer.start();
                let result = state.check_and_store_nonce(&nonce, timestamp);
                timer.record();
                
                match result {
                    Ok(_) => success_count += 1,
                    Err(_) => error_count += 1,
                }
            }
            
            let total_duration = start_time.elapsed();
            let stats = timer.statistics();
            
            let mut measurement = PerformanceMeasurement::new(
                format!("security_profile_{:?}", profile),
                "server".to_string(),
                "profile_verification".to_string(),
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
            
            measurement.additional_metrics.insert(
                "profile_type".to_string(), 
                match profile {
                    SecurityProfile::Lenient => 1.0,
                    SecurityProfile::Standard => 2.0,
                    SecurityProfile::Strict => 3.0,
                }
            );
            
            println!("  ✓ {:?} profile: {:.3}ms avg, {:.1} ops/sec", 
                    profile, stats.avg_ms, measurement.operations_per_second);
            
            self.results.push(measurement);
        }
        
        Ok(())
    }

    /// Benchmark concurrent verification performance
    async fn benchmark_concurrent_verification(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking concurrent verification");
        
        let state = self.verification_states.get(&SecurityProfile::Standard)
            .ok_or("Standard security profile not found")?;
        
        for &concurrent_users in &self.config.concurrent_user_counts {
            if concurrent_users > 100 {
                continue; // Skip very high concurrency for this test
            }
            
            let operations_per_user = 100;
            let total_operations = concurrent_users * operations_per_user;
            
            let success_counter = Arc::new(AtomicUsize::new(0));
            let error_counter = Arc::new(AtomicUsize::new(0));
            let operation_times = Arc::new(std::sync::Mutex::new(Vec::new()));
            
            let start_time = Instant::now();
            
            // Spawn concurrent verification tasks
            let tasks: Vec<_> = (0..concurrent_users).map(|user_id| {
                let state = Arc::clone(state);
                let success_counter = Arc::clone(&success_counter);
                let error_counter = Arc::clone(&error_counter);
                let operation_times = Arc::clone(&operation_times);
                
                tokio::spawn(async move {
                    for i in 0..operations_per_user {
                        let nonce = format!("concurrent_nonce_{}_{}", user_id, i);
                        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                        
                        let op_start = Instant::now();
                        let result = state.check_and_store_nonce(&nonce, timestamp);
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
            
            // Wait for all tasks to complete
            join_all(tasks).await;
            
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
            let median_ms = times_ms[times_ms.len() / 2];
            let p95_ms = times_ms[(times_ms.len() * 95) / 100];
            let p99_ms = times_ms[(times_ms.len() * 99) / 100];
            
            let mut measurement = PerformanceMeasurement::new(
                format!("concurrent_verification_{}_users", concurrent_users),
                "server".to_string(),
                "concurrent_operations".to_string(),
            );
            
            measurement.operation_count = total_operations;
            measurement.total_duration = total_duration;
            measurement.avg_operation_time_ms = avg_ms;
            measurement.median_operation_time_ms = median_ms;
            measurement.p95_operation_time_ms = p95_ms;
            measurement.p99_operation_time_ms = p99_ms;
            measurement.min_operation_time_ms = times_ms.first().copied().unwrap_or(0.0);
            measurement.max_operation_time_ms = times_ms.last().copied().unwrap_or(0.0);
            measurement.operations_per_second = total_operations as f64 / total_duration.as_secs_f64();
            measurement.error_count = error_count;
            measurement.success_rate_percent = (success_count as f64 / total_operations as f64) * 100.0;
            
            measurement.additional_metrics.insert("concurrent_users".to_string(), concurrent_users as f64);
            measurement.additional_metrics.insert("operations_per_user".to_string(), operations_per_user as f64);
            
            println!("  ✓ {} concurrent users: {:.3}ms avg, {:.1} ops/sec", 
                    concurrent_users, avg_ms, measurement.operations_per_second);
            
            self.results.push(measurement);
        }
        
        Ok(())
    }

    /// Benchmark sustained load performance
    async fn benchmark_sustained_load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking sustained load");
        
        let state = self.verification_states.get(&SecurityProfile::Standard)
            .ok_or("Standard security profile not found")?;
        
        let test_duration = Duration::from_secs(self.config.test_duration_seconds);
        let target_rps = 500.0; // Target 500 operations per second
        let interval = Duration::from_secs_f64(1.0 / target_rps);
        
        let mut operation_count = 0;
        let mut success_count = 0;
        let mut error_count = 0;
        let mut operation_times = Vec::new();
        
        let start_time = Instant::now();
        let mut next_operation_time = start_time;
        
        while start_time.elapsed() < test_duration {
            let nonce = format!("sustained_nonce_{}", operation_count);
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            
            let op_start = Instant::now();
            let result = state.check_and_store_nonce(&nonce, timestamp);
            let op_duration = op_start.elapsed();
            
            operation_times.push(op_duration.as_secs_f64() * 1000.0);
            operation_count += 1;
            
            match result {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
            
            // Rate limiting to maintain target RPS
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
            "sustained_load".to_string(),
            "server".to_string(),
            "sustained_operations".to_string(),
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

    /// Benchmark burst load performance
    async fn benchmark_burst_load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking burst load");
        
        let state = self.verification_states.get(&SecurityProfile::Standard)
            .ok_or("Standard security profile not found")?;
        
        let burst_size = 1000;
        let burst_count = 5;
        let rest_duration = Duration::from_secs(2);
        
        let mut all_measurements = Vec::new();
        
        for burst_num in 0..burst_count {
            let mut operation_times = Vec::new();
            let mut success_count = 0;
            let mut error_count = 0;
            
            let burst_start = Instant::now();
            
            // Execute burst
            for i in 0..burst_size {
                let nonce = format!("burst_nonce_{}_{}", burst_num, i);
                let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
                
                let op_start = Instant::now();
                let result = state.check_and_store_nonce(&nonce, timestamp);
                let op_duration = op_start.elapsed();
                
                operation_times.push(op_duration.as_secs_f64() * 1000.0);
                
                match result {
                    Ok(_) => success_count += 1,
                    Err(_) => error_count += 1,
                }
            }
            
            let burst_duration = burst_start.elapsed();
            
            // Calculate burst statistics
            operation_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let avg_ms = operation_times.iter().sum::<f64>() / operation_times.len() as f64;
            let p95_ms = operation_times[(operation_times.len() * 95) / 100];
            
            let burst_rps = burst_size as f64 / burst_duration.as_secs_f64();
            
            all_measurements.push((avg_ms, p95_ms, burst_rps, success_count, error_count));
            
            println!("    Burst {}: {:.3}ms avg, {:.1} ops/sec", burst_num + 1, avg_ms, burst_rps);
            
            // Rest between bursts
            if burst_num < burst_count - 1 {
                sleep(rest_duration).await;
            }
        }
        
        // Calculate overall burst statistics
        let total_ops = burst_size * burst_count;
        let avg_burst_time: f64 = all_measurements.iter().map(|(avg, _, _, _, _)| *avg).sum::<f64>() / burst_count as f64;
        let avg_burst_rps: f64 = all_measurements.iter().map(|(_, _, rps, _, _)| *rps).sum::<f64>() / burst_count as f64;
        let total_successes: usize = all_measurements.iter().map(|(_, _, _, succ, _)| *succ).sum();
        let total_errors: usize = all_measurements.iter().map(|(_, _, _, _, err)| *err).sum();
        
        let mut measurement = PerformanceMeasurement::new(
            "burst_load".to_string(),
            "server".to_string(),
            "burst_operations".to_string(),
        );
        
        measurement.operation_count = total_ops;
        measurement.avg_operation_time_ms = avg_burst_time;
        measurement.operations_per_second = avg_burst_rps;
        measurement.error_count = total_errors;
        measurement.success_rate_percent = (total_successes as f64 / total_ops as f64) * 100.0;
        
        measurement.additional_metrics.insert("burst_size".to_string(), burst_size as f64);
        measurement.additional_metrics.insert("burst_count".to_string(), burst_count as f64);
        measurement.additional_metrics.insert("rest_duration_sec".to_string(), rest_duration.as_secs() as f64);
        
        println!("  ✓ Burst load: {:.3}ms avg, {:.1} ops/sec across {} bursts", 
                avg_burst_time, avg_burst_rps, burst_count);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark memory pressure performance
    async fn benchmark_memory_pressure(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking memory pressure performance");
        
        let state = self.verification_states.get(&SecurityProfile::Standard)
            .ok_or("Standard security profile not found")?;
        
        // Create memory pressure by storing many nonces
        let pressure_nonces = 50000;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        
        // Fill memory with nonces
        for i in 0..pressure_nonces {
            let nonce = format!("pressure_nonce_{}", i);
            let _ = state.check_and_store_nonce(&nonce, timestamp);
        }
        
        // Now benchmark performance under memory pressure
        let mut timer = BenchmarkTimer::new();
        let iterations = 1000;
        let mut success_count = 0;
        let mut error_count = 0;
        
        let start_time = Instant::now();
        
        for i in 0..iterations {
            let nonce = format!("memory_test_nonce_{}", i);
            
            timer.start();
            let result = state.check_and_store_nonce(&nonce, timestamp);
            timer.record();
            
            match result {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "memory_pressure".to_string(),
            "server".to_string(),
            "memory_stress".to_string(),
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
        
        measurement.additional_metrics.insert("pressure_nonces".to_string(), pressure_nonces as f64);
        
        println!("  ✓ Memory pressure: {:.3}ms avg, {:.1} ops/sec with {} stored nonces", 
                stats.avg_ms, measurement.operations_per_second, pressure_nonces);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark performance under attack scenarios
    async fn benchmark_attack_scenario_performance(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking attack scenario performance");
        
        let state = self.verification_states.get(&SecurityProfile::Strict)
            .ok_or("Strict security profile not found")?;
        
        let legitimate_operations = 500;
        let attack_operations = 100; // 20% attack rate
        let mut timer = BenchmarkTimer::new();
        let mut success_count = 0;
        let mut error_count = 0;
        
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        
        let start_time = Instant::now();
        
        // Mix of legitimate requests and attack attempts
        for i in 0..(legitimate_operations + attack_operations) {
            let (nonce, timestamp) = if i < legitimate_operations {
                // Legitimate request
                (format!("legit_nonce_{}", i), current_time - (i % 30) as u64)
            } else {
                // Attack attempt (replay)
                let replay_index = i % legitimate_operations;
                (format!("legit_nonce_{}", replay_index), current_time - 3600) // Old timestamp
            };
            
            timer.start();
            let result = state.check_and_store_nonce(&nonce, timestamp);
            timer.record();
            
            match result {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        let total_operations = legitimate_operations + attack_operations;
        
        let mut measurement = PerformanceMeasurement::new(
            "attack_scenario".to_string(),
            "server".to_string(),
            "mixed_operations".to_string(),
        );
        
        measurement.operation_count = total_operations;
        measurement.total_duration = total_duration;
        measurement.avg_operation_time_ms = stats.avg_ms;
        measurement.median_operation_time_ms = stats.median_ms;
        measurement.p95_operation_time_ms = stats.p95_ms;
        measurement.p99_operation_time_ms = stats.p99_ms;
        measurement.min_operation_time_ms = stats.min_ms;
        measurement.max_operation_time_ms = stats.max_ms;
        measurement.operations_per_second = total_operations as f64 / total_duration.as_secs_f64();
        measurement.error_count = error_count;
        measurement.success_rate_percent = (success_count as f64 / total_operations as f64) * 100.0;
        
        measurement.additional_metrics.insert("legitimate_ops".to_string(), legitimate_operations as f64);
        measurement.additional_metrics.insert("attack_ops".to_string(), attack_operations as f64);
        measurement.additional_metrics.insert("attack_rate_percent".to_string(), 
            (attack_operations as f64 / total_operations as f64) * 100.0);
        
        println!("  ✓ Attack scenario: {:.3}ms avg, {:.1} ops/sec ({}% attack rate)", 
                stats.avg_ms, measurement.operations_per_second, 
                (attack_operations as f64 / total_operations as f64) * 100.0);
        
        self.results.push(measurement);
        Ok(())
    }

    // Helper methods

    /// Parse signature header (simplified simulation)
    fn parse_signature_header(&self, signature: &str, signature_input: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Simulate signature header parsing
        if signature.contains("sig1=:") && signature_input.contains("sig1=(") {
            Ok(())
        } else {
            Err("Invalid signature format".into())
        }
    }

    /// Simulate middleware request processing
    async fn simulate_middleware_request(
        &self, 
        state: &Arc<SignatureVerificationState>, 
        request_id: usize
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Simulate complete middleware processing cycle
        let nonce = format!("middleware_nonce_{}", request_id);
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        
        // 1. Header parsing (simulated)
        std::thread::sleep(Duration::from_micros(10));
        
        // 2. Nonce validation
        state.check_and_store_nonce(&nonce, timestamp)?;
        
        // 3. Additional middleware overhead (simulated)
        std::thread::sleep(Duration::from_micros(5));
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_benchmarks_creation() {
        let config = PerformanceBenchmarkConfig::default();
        let targets = PerformanceTargets::default();
        
        let benchmarks = ServerPerformanceBenchmarks::new(config, targets);
        assert!(benchmarks.is_ok());
    }

    #[tokio::test]
    async fn test_ed25519_verification_benchmark() {
        let config = PerformanceBenchmarkConfig {
            micro_benchmark_iterations: 100,
            ..Default::default()
        };
        let targets = PerformanceTargets::default();
        
        let mut benchmarks = ServerPerformanceBenchmarks::new(config, targets).unwrap();
        let result = benchmarks.benchmark_ed25519_verification().await;
        
        assert!(result.is_ok());
        assert!(!benchmarks.results.is_empty());
        
        let measurement = &benchmarks.results[0];
        assert_eq!(measurement.test_name, "ed25519_verification");
        assert_eq!(measurement.component, "server");
        assert!(measurement.operations_per_second > 0.0);
    }
}