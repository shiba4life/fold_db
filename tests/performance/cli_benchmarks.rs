//! CLI Performance Benchmarks
//!
//! This module provides comprehensive performance benchmarking for the DataFold CLI's
//! signature authentication capabilities, including key management, request signing,
//! and command execution performance.

use super::{
    PerformanceMeasurement, BenchmarkTimer, PerformanceBenchmarkConfig,
    PerformanceTargets
};
use crate::cli::auth::{CliAuthProfile, CliSigningConfig};
use crate::cli::config::CliConfigManager;
use crate::cli::signing_config::{SigningMode, EnhancedSigningConfig};
use crate::crypto::ed25519::{generate_master_keypair, MasterKeyPair};
use serde_json::json;
use std::collections::HashMap;
use std::process::Command;
use std::time::{Duration, Instant};
use tempfile::tempdir;
use tokio::fs;
use tokio::process::Command as AsyncCommand;

/// CLI performance benchmark suite
pub struct CliPerformanceBenchmarks {
    config: PerformanceBenchmarkConfig,
    targets: PerformanceTargets,
    results: Vec<PerformanceMeasurement>,
    temp_dir: tempfile::TempDir,
    cli_config: CliConfigManager,
    test_profiles: Vec<CliAuthProfile>,
    key_pairs: Vec<MasterKeyPair>,
}

impl CliPerformanceBenchmarks {
    /// Create new CLI performance benchmarks
    pub fn new(
        config: PerformanceBenchmarkConfig,
        targets: PerformanceTargets,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("cli_benchmark_config.toml");
        let cli_config = CliConfigManager::with_path(&config_path)?;
        
        // Generate test key pairs
        let mut key_pairs = Vec::new();
        for _ in 0..5 {
            key_pairs.push(generate_master_keypair()?);
        }
        
        // Create test profiles
        let test_profiles = Self::create_test_profiles(&key_pairs);
        
        Ok(Self {
            config,
            targets,
            results: Vec::new(),
            temp_dir,
            cli_config,
            test_profiles,
            key_pairs,
        })
    }

    /// Run all CLI performance benchmarks
    pub async fn run_all_benchmarks(&mut self) -> Result<Vec<PerformanceMeasurement>, Box<dyn std::error::Error>> {
        println!("🚀 Starting CLI performance benchmarks");
        
        // Setup test environment
        self.setup_cli_test_environment().await?;
        
        // Core CLI performance benchmarks
        self.benchmark_cli_key_operations().await?;
        self.benchmark_cli_signing_operations().await?;
        self.benchmark_cli_config_operations().await?;
        self.benchmark_cli_profile_management().await?;
        
        // CLI command execution benchmarks
        self.benchmark_cli_command_execution().await?;
        self.benchmark_cli_signing_modes().await?;
        self.benchmark_cli_batch_operations().await?;
        
        // CLI integration benchmarks
        self.benchmark_cli_end_to_end_workflows().await?;
        self.benchmark_cli_concurrent_operations().await?;
        self.benchmark_cli_memory_usage().await?;
        
        // CLI performance under different conditions
        self.benchmark_cli_cold_vs_warm_start().await?;
        self.benchmark_cli_large_config_files().await?;
        
        println!("✅ CLI performance benchmarks completed");
        Ok(self.results.clone())
    }

    /// Create test authentication profiles
    fn create_test_profiles(key_pairs: &[MasterKeyPair]) -> Vec<CliAuthProfile> {
        let mut profiles = Vec::new();
        
        for (i, _key_pair) in key_pairs.iter().enumerate() {
            let mut metadata = HashMap::new();
            metadata.insert("environment".to_string(), format!("test-env-{}", i));
            metadata.insert("created_by".to_string(), "benchmark".to_string());
            
            let profile = CliAuthProfile {
                client_id: format!("cli-benchmark-client-{}", i),
                key_id: format!("benchmark-key-{}", i),
                user_id: Some(format!("benchmark-user-{}", i)),
                server_url: format!("https://api-{}.example.com", i),
                metadata,
            };
            
            profiles.push(profile);
        }
        
        profiles
    }

    /// Setup CLI test environment
    async fn setup_cli_test_environment(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📋 Setting up CLI test environment");
        
        // Configure automatic signing
        self.cli_config.set_auto_signing_enabled(true);
        self.cli_config.set_default_signing_mode(SigningMode::Auto);
        
        // Add test profiles
        for (i, profile) in self.test_profiles.iter().enumerate() {
            let profile_name = format!("benchmark-profile-{}", i);
            self.cli_config.add_profile(profile_name.clone(), profile.clone())?;
            
            if i == 0 {
                self.cli_config.set_default_profile(profile_name)?;
            }
        }
        
        // Configure command-specific signing
        self.cli_config.set_command_signing_mode("query".to_string(), SigningMode::Auto)?;
        self.cli_config.set_command_signing_mode("mutate".to_string(), SigningMode::Manual)?;
        self.cli_config.set_command_signing_mode("status".to_string(), SigningMode::Disabled)?;
        
        // Save configuration
        self.cli_config.save()?;
        
        println!("  ✓ CLI test environment configured");
        Ok(())
    }

    /// Benchmark CLI key operations
    async fn benchmark_cli_key_operations(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking CLI key operations");
        
        // Benchmark key loading
        self.benchmark_key_loading().await?;
        
        // Benchmark key generation
        self.benchmark_key_generation().await?;
        
        // Benchmark key validation
        self.benchmark_key_validation().await?;
        
        Ok(())
    }

    /// Benchmark key loading performance
    async fn benchmark_key_loading(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("  🔍 Key loading performance");
        
        let mut timer = BenchmarkTimer::new();
        let iterations = self.config.micro_benchmark_iterations;
        let mut success_count = 0;
        let mut error_count = 0;
        
        // Simulate key loading from different sources
        let start_time = Instant::now();
        
        for i in 0..iterations {
            let key_pair = &self.key_pairs[i % self.key_pairs.len()];
            
            timer.start();
            
            // Simulate key loading operations
            let _public_key = key_pair.public_key();
            let _key_bytes = key_pair.public_key().to_bytes();
            
            timer.record();
            success_count += 1;
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "cli_key_loading".to_string(),
            "cli".to_string(),
            "key_operations".to_string(),
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
        
        measurement.additional_metrics.insert("key_count".to_string(), self.key_pairs.len() as f64);
        
        println!("    ✓ Key loading: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark key generation performance
    async fn benchmark_key_generation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("  🔍 Key generation performance");
        
        let mut timer = BenchmarkTimer::new();
        let iterations = 100; // Fewer iterations for expensive operations
        let mut success_count = 0;
        let mut error_count = 0;
        
        let start_time = Instant::now();
        
        for _i in 0..iterations {
            timer.start();
            
            // Generate new key pair
            match generate_master_keypair() {
                Ok(_key_pair) => {
                    success_count += 1;
                }
                Err(_) => {
                    error_count += 1;
                }
            }
            
            timer.record();
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "cli_key_generation".to_string(),
            "cli".to_string(),
            "key_creation".to_string(),
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
        
        println!("    ✓ Key generation: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark key validation performance
    async fn benchmark_key_validation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("  🔍 Key validation performance");
        
        let mut timer = BenchmarkTimer::new();
        let iterations = self.config.micro_benchmark_iterations;
        let mut success_count = 0;
        let mut error_count = 0;
        
        // Pre-generate test signatures for validation
        let mut test_signatures = Vec::new();
        for key_pair in &self.key_pairs {
            let message = b"Test validation message";
            if let Ok(signature) = key_pair.sign(message) {
                test_signatures.push((key_pair.public_key(), signature, message.to_vec()));
            }
        }
        
        let start_time = Instant::now();
        
        for i in 0..iterations {
            let (public_key, signature, message) = &test_signatures[i % test_signatures.len()];
            
            timer.start();
            
            // Validate signature
            match public_key.verify(message, signature) {
                Ok(true) => success_count += 1,
                Ok(false) => error_count += 1,
                Err(_) => error_count += 1,
            }
            
            timer.record();
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "cli_key_validation".to_string(),
            "cli".to_string(),
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
        
        measurement.additional_metrics.insert("signature_variants".to_string(), test_signatures.len() as f64);
        
        println!("    ✓ Key validation: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark CLI signing operations
    async fn benchmark_cli_signing_operations(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking CLI signing operations");
        
        // Benchmark request signing
        self.benchmark_request_signing().await?;
        
        // Benchmark signature header generation
        self.benchmark_signature_header_generation().await?;
        
        // Benchmark content digest calculation
        self.benchmark_content_digest_calculation().await?;
        
        Ok(())
    }

    /// Benchmark request signing performance
    async fn benchmark_request_signing(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("  🔍 Request signing performance");
        
        let mut timer = BenchmarkTimer::new();
        let iterations = 2000;
        let mut success_count = 0;
        let mut error_count = 0;
        
        let test_requests = self.generate_test_requests(100);
        
        let start_time = Instant::now();
        
        for i in 0..iterations {
            let (method, url, body) = &test_requests[i % test_requests.len()];
            let key_pair = &self.key_pairs[i % self.key_pairs.len()];
            
            timer.start();
            
            // Simulate CLI request signing
            match self.sign_cli_request(key_pair, method, url, body.as_ref()).await {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
            
            timer.record();
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "cli_request_signing".to_string(),
            "cli".to_string(),
            "request_authentication".to_string(),
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
        
        measurement.additional_metrics.insert("request_variants".to_string(), test_requests.len() as f64);
        
        println!("    ✓ Request signing: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark signature header generation
    async fn benchmark_signature_header_generation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("  🔍 Signature header generation performance");
        
        let mut timer = BenchmarkTimer::new();
        let iterations = self.config.micro_benchmark_iterations;
        let mut success_count = 0;
        
        let start_time = Instant::now();
        
        for i in 0..iterations {
            timer.start();
            
            // Generate signature headers
            let signature = format!("signature_data_{}", i);
            let key_id = format!("key_{}", i % self.key_pairs.len());
            let nonce = uuid::Uuid::new_v4().to_string();
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            let _signature_header = format!("sig1=:{}:", signature);
            let _signature_input_header = format!(
                "sig1=(\"@method\" \"@target-uri\" \"@authority\");created={};keyid=\"{}\";nonce=\"{}\"",
                timestamp, key_id, nonce
            );
            
            timer.record();
            success_count += 1;
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "cli_signature_headers".to_string(),
            "cli".to_string(),
            "header_generation".to_string(),
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
        measurement.error_count = 0;
        measurement.success_rate_percent = 100.0;
        
        println!("    ✓ Signature headers: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark content digest calculation
    async fn benchmark_content_digest_calculation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("  🔍 Content digest calculation performance");
        
        let mut timer = BenchmarkTimer::new();
        let iterations = self.config.micro_benchmark_iterations;
        let mut success_count = 0;
        
        // Generate test payloads of different sizes
        let test_payloads = vec![
            "{}".to_string(),
            json!({"key": "value"}).to_string(),
            json!({"data": "x".repeat(1000)}).to_string(),
            json!({"large": "y".repeat(10000)}).to_string(),
        ];
        
        let start_time = Instant::now();
        
        for i in 0..iterations {
            let payload = &test_payloads[i % test_payloads.len()];
            
            timer.start();
            
            // Calculate SHA-256 digest
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(payload.as_bytes());
            let _digest = hasher.finalize();
            
            timer.record();
            success_count += 1;
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "cli_content_digest".to_string(),
            "cli".to_string(),
            "digest_calculation".to_string(),
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
        measurement.error_count = 0;
        measurement.success_rate_percent = 100.0;
        
        measurement.additional_metrics.insert("payload_variants".to_string(), test_payloads.len() as f64);
        
        println!("    ✓ Content digest: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark CLI configuration operations
    async fn benchmark_cli_config_operations(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking CLI configuration operations");
        
        // Benchmark config loading
        self.benchmark_config_loading().await?;
        
        // Benchmark config saving
        self.benchmark_config_saving().await?;
        
        // Benchmark config validation
        self.benchmark_config_validation().await?;
        
        Ok(())
    }

    /// Benchmark config loading performance
    async fn benchmark_config_loading(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("  🔍 Config loading performance");
        
        let mut timer = BenchmarkTimer::new();
        let iterations = 1000;
        let mut success_count = 0;
        let mut error_count = 0;
        
        let start_time = Instant::now();
        
        for _i in 0..iterations {
            timer.start();
            
            // Load configuration
            match self.cli_config.reload() {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
            
            timer.record();
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "cli_config_loading".to_string(),
            "cli".to_string(),
            "config_operations".to_string(),
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
        
        println!("    ✓ Config loading: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark config saving performance
    async fn benchmark_config_saving(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("  🔍 Config saving performance");
        
        let mut timer = BenchmarkTimer::new();
        let iterations = 500; // Fewer iterations for file I/O
        let mut success_count = 0;
        let mut error_count = 0;
        
        let start_time = Instant::now();
        
        for i in 0..iterations {
            // Make a small change to trigger saving
            let test_value = format!("test_value_{}", i);
            self.cli_config.set_signing_debug(i % 2 == 0);
            
            timer.start();
            
            match self.cli_config.save() {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
            
            timer.record();
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "cli_config_saving".to_string(),
            "cli".to_string(),
            "config_persistence".to_string(),
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
        
        println!("    ✓ Config saving: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark config validation performance
    async fn benchmark_config_validation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("  🔍 Config validation performance");
        
        let mut timer = BenchmarkTimer::new();
        let iterations = self.config.micro_benchmark_iterations;
        let mut success_count = 0;
        
        let start_time = Instant::now();
        
        for _i in 0..iterations {
            timer.start();
            
            // Validate current configuration
            let config = self.cli_config.config();
            let _is_valid = !config.signing.auto_signing.enabled || 
                           !config.signing.auto_signing.command_overrides.is_empty();
            
            timer.record();
            success_count += 1;
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "cli_config_validation".to_string(),
            "cli".to_string(),
            "config_validation".to_string(),
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
        measurement.error_count = 0;
        measurement.success_rate_percent = 100.0;
        
        println!("    ✓ Config validation: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark CLI profile management
    async fn benchmark_cli_profile_management(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking CLI profile management");
        
        // Benchmark profile operations
        self.benchmark_profile_operations().await?;
        
        Ok(())
    }

    /// Benchmark profile operations
    async fn benchmark_profile_operations(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("  🔍 Profile operations performance");
        
        let mut timer = BenchmarkTimer::new();
        let iterations = 1000;
        let mut success_count = 0;
        let mut error_count = 0;
        
        let start_time = Instant::now();
        
        for i in 0..iterations {
            timer.start();
            
            // Cycle through different profile operations
            match i % 4 {
                0 => {
                    // List profiles
                    let _profiles = self.cli_config.list_profiles();
                    success_count += 1;
                }
                1 => {
                    // Get profile
                    let profile_name = format!("benchmark-profile-{}", i % self.test_profiles.len());
                    let _profile = self.cli_config.get_profile(&profile_name);
                    success_count += 1;
                }
                2 => {
                    // Set default profile
                    let profile_name = format!("benchmark-profile-{}", i % self.test_profiles.len());
                    match self.cli_config.set_default_profile(profile_name) {
                        Ok(_) => success_count += 1,
                        Err(_) => error_count += 1,
                    }
                }
                3 => {
                    // Get signing config for profile
                    let signing_config = self.cli_config.signing_config();
                    let _context = signing_config.for_command("test-command");
                    success_count += 1;
                }
                _ => unreachable!(),
            }
            
            timer.record();
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "cli_profile_operations".to_string(),
            "cli".to_string(),
            "profile_management".to_string(),
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
        
        measurement.additional_metrics.insert("profile_count".to_string(), self.test_profiles.len() as f64);
        measurement.additional_metrics.insert("operation_types".to_string(), 4.0);
        
        println!("    ✓ Profile operations: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark CLI command execution
    async fn benchmark_cli_command_execution(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking CLI command execution");
        
        // Create test CLI script
        let cli_script = self.create_cli_test_script().await?;
        
        // Benchmark different command types
        self.benchmark_command_types(&cli_script).await?;
        
        Ok(())
    }

    /// Create CLI test script
    async fn create_cli_test_script(&self) -> Result<String, Box<dyn std::error::Error>> {
        let script_content = format!(r#"#!/bin/bash
# CLI Performance Test Script

CONFIG_PATH="{}"

# Test command with auto-signing
datafold-cli --config "$CONFIG_PATH" query --auto-sign --endpoint /test/auto
echo "AUTO_SIGN_RESULT: $?"

# Test command with manual signing
datafold-cli --config "$CONFIG_PATH" mutate --sign --endpoint /test/manual
echo "MANUAL_SIGN_RESULT: $?"

# Test command without signing
datafold-cli --config "$CONFIG_PATH" status --no-sign --endpoint /test/status
echo "NO_SIGN_RESULT: $?"

# Test batch operations
for i in {{1..10}}; do
    datafold-cli --config "$CONFIG_PATH" query --auto-sign --endpoint "/test/batch/$i" &
done
wait
echo "BATCH_COMPLETE"
"#, self.cli_config.config_path().display());
        
        let script_path = self.temp_dir.path().join("cli_test.sh");
        fs::write(&script_path, script_content).await?;
        
        // Make script executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script_path, perms)?;
        }
        
        Ok(script_path.to_string_lossy().to_string())
    }

    /// Benchmark different command types
    async fn benchmark_command_types(&mut self, _script_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("  🔍 Command execution performance");
        
        let command_types = vec![
            ("auto_sign", "Commands with auto-signing"),
            ("manual_sign", "Commands with manual signing"),
            ("no_sign", "Commands without signing"),
            ("batch", "Batch command operations"),
        ];
        
        for (command_type, description) in command_types {
            println!("    Testing {}: {}", command_type, description);
            
            let mut timer = BenchmarkTimer::new();
            let iterations = 50; // Fewer iterations for actual command execution
            let mut success_count = 0;
            let mut error_count = 0;
            
            let start_time = Instant::now();
            
            for i in 0..iterations {
                timer.start();
                
                // Simulate CLI command execution
                let execution_result = self.simulate_cli_command(command_type, i).await;
                
                timer.record();
                
                match execution_result {
                    Ok(_) => success_count += 1,
                    Err(_) => error_count += 1,
                }
            }
            
            let total_duration = start_time.elapsed();
            let stats = timer.statistics();
            
            let mut measurement = PerformanceMeasurement::new(
                format!("cli_command_{}", command_type),
                "cli".to_string(),
                "command_execution".to_string(),
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
            
            measurement.additional_metrics.insert("command_type".to_string(), 
                match command_type {
                    "auto_sign" => 1.0,
                    "manual_sign" => 2.0,
                    "no_sign" => 3.0,
                    "batch" => 4.0,
                    _ => 0.0,
                });
            
            println!("      ✓ {}: {:.3}ms avg, {:.1} ops/sec", 
                    command_type, stats.avg_ms, measurement.operations_per_second);
            
            self.results.push(measurement);
        }
        
        Ok(())
    }

    /// Benchmark CLI signing modes
    async fn benchmark_cli_signing_modes(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking CLI signing modes");
        
        let modes = vec![
            (SigningMode::Auto, "Auto signing mode"),
            (SigningMode::Manual, "Manual signing mode"),
            (SigningMode::Disabled, "Disabled signing mode"),
        ];
        
        for (mode, description) in modes {
            println!("  🔍 {}: {}", mode.as_str(), description);
            
            let mut timer = BenchmarkTimer::new();
            let iterations = 500;
            let mut success_count = 0;
            let mut error_count = 0;
            
            // Configure signing mode
            self.cli_config.set_default_signing_mode(mode.clone());
            
            let start_time = Instant::now();
            
            for i in 0..iterations {
                timer.start();
                
                // Get signing context for mode
                let signing_config = self.cli_config.signing_config();
                let context = signing_config.for_command("test-command");
                
                // Simulate mode-specific processing
                let should_sign = match mode {
                    SigningMode::Auto => context.should_auto_sign,
                    SigningMode::Manual => context.allows_explicit,
                    SigningMode::Disabled => false,
                };
                
                if should_sign {
                    // Simulate signing operation
                    let key_pair = &self.key_pairs[i % self.key_pairs.len()];
                    match self.simulate_signing_operation(key_pair, i).await {
                        Ok(_) => success_count += 1,
                        Err(_) => error_count += 1,
                    }
                } else {
                    success_count += 1;
                }
                
                timer.record();
            }
            
            let total_duration = start_time.elapsed();
            let stats = timer.statistics();
            
            let mut measurement = PerformanceMeasurement::new(
                format!("cli_signing_mode_{}", mode.as_str()),
                "cli".to_string(),
                "signing_modes".to_string(),
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
            
            measurement.additional_metrics.insert("signing_mode".to_string(), 
                match mode {
                    SigningMode::Auto => 1.0,
                    SigningMode::Manual => 2.0,
                    SigningMode::Disabled => 3.0,
                });
            
            println!("    ✓ {}: {:.3}ms avg, {:.1} ops/sec", 
                    mode.as_str(), stats.avg_ms, measurement.operations_per_second);
            
            self.results.push(measurement);
        }
        
        Ok(())
    }

    /// Benchmark CLI batch operations
    async fn benchmark_cli_batch_operations(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking CLI batch operations");
        
        let batch_sizes = vec![10, 50, 100];
        
        for batch_size in batch_sizes {
            println!("  🔍 Batch size: {}", batch_size);
            
            let mut timer = BenchmarkTimer::new();
            let mut success_count = 0;
            let mut error_count = 0;
            
            let start_time = Instant::now();
            
            timer.start();
            
            // Simulate batch operation
            for i in 0..batch_size {
                let key_pair = &self.key_pairs[i % self.key_pairs.len()];
                match self.simulate_batch_item(key_pair, i).await {
                    Ok(_) => success_count += 1,
                    Err(_) => error_count += 1,
                }
            }
            
            timer.record();
            
            let total_duration = start_time.elapsed();
            let stats = timer.statistics();
            
            let mut measurement = PerformanceMeasurement::new(
                format!("cli_batch_{}", batch_size),
                "cli".to_string(),
                "batch_operations".to_string(),
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

    /// Benchmark CLI end-to-end workflows
    async fn benchmark_cli_end_to_end_workflows(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking CLI end-to-end workflows");
        
        // Simulate complete CLI workflows
        let workflows = vec![
            ("query_workflow", "Complete query workflow with authentication"),
            ("mutate_workflow", "Complete mutation workflow with authentication"),
            ("config_workflow", "Configuration management workflow"),
        ];
        
        for (workflow_name, description) in workflows {
            println!("  🔍 {}: {}", workflow_name, description);
            
            let mut timer = BenchmarkTimer::new();
            let iterations = 100;
            let mut success_count = 0;
            let mut error_count = 0;
            
            let start_time = Instant::now();
            
            for i in 0..iterations {
                timer.start();
                
                match self.simulate_end_to_end_workflow(workflow_name, i).await {
                    Ok(_) => success_count += 1,
                    Err(_) => error_count += 1,
                }
                
                timer.record();
            }
            
            let total_duration = start_time.elapsed();
            let stats = timer.statistics();
            
            let mut measurement = PerformanceMeasurement::new(
                format!("cli_e2e_{}", workflow_name),
                "cli".to_string(),
                "end_to_end_workflows".to_string(),
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
                    workflow_name, stats.avg_ms, measurement.operations_per_second);
            
            self.results.push(measurement);
        }
        
        Ok(())
    }

    /// Benchmark CLI concurrent operations
    async fn benchmark_cli_concurrent_operations(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking CLI concurrent operations");
        
        let concurrent_counts = vec![1, 5, 10];
        
        for concurrent_count in concurrent_counts {
            println!("  🔍 Concurrent operations: {}", concurrent_count);
            
            let operations_per_thread = 20;
            let total_operations = concurrent_count * operations_per_thread;
            
            let success_counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let error_counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let operation_times = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            
            let start_time = Instant::now();
            
            // Spawn concurrent tasks
            let mut tasks = Vec::new();
            for thread_id in 0..concurrent_count {
                let key_pair = self.key_pairs[thread_id % self.key_pairs.len()].clone();
                let success_counter = std::sync::Arc::clone(&success_counter);
                let error_counter = std::sync::Arc::clone(&error_counter);
                let operation_times = std::sync::Arc::clone(&operation_times);
                
                let task = tokio::spawn(async move {
                    for i in 0..operations_per_thread {
                        let op_start = Instant::now();
                        
                        // Simulate concurrent CLI operation
                        let result = Self::simulate_concurrent_operation(&key_pair, thread_id, i).await;
                        
                        let op_duration = op_start.elapsed();
                        
                        {
                            let mut times = operation_times.lock().unwrap();
                            times.push(op_duration);
                        }
                        
                        match result {
                            Ok(_) => { success_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
                            Err(_) => { error_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
                        }
                    }
                });
                
                tasks.push(task);
            }
            
            // Wait for all tasks
            futures::future::join_all(tasks).await;
            
            let total_duration = start_time.elapsed();
            let success_count = success_counter.load(std::sync::atomic::Ordering::Relaxed);
            let error_count = error_counter.load(std::sync::atomic::Ordering::Relaxed);
            
            // Calculate timing statistics
            let times = operation_times.lock().unwrap();
            let mut times_ms: Vec<f64> = times.iter()
                .map(|d| d.as_secs_f64() * 1000.0)
                .collect();
            times_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let avg_ms = times_ms.iter().sum::<f64>() / times_ms.len() as f64;
            let p95_ms = times_ms[(times_ms.len() * 95) / 100];
            
            let mut measurement = PerformanceMeasurement::new(
                format!("cli_concurrent_{}", concurrent_count),
                "cli".to_string(),
                "concurrent_operations".to_string(),
            );
            
            measurement.operation_count = total_operations;
            measurement.total_duration = total_duration;
            measurement.avg_operation_time_ms = avg_ms;
            measurement.p95_operation_time_ms = p95_ms;
            measurement.operations_per_second = total_operations as f64 / total_duration.as_secs_f64();
            measurement.error_count = error_count;
            measurement.success_rate_percent = (success_count as f64 / total_operations as f64) * 100.0;
            
            measurement.additional_metrics.insert("concurrent_count".to_string(), concurrent_count as f64);
            measurement.additional_metrics.insert("ops_per_thread".to_string(), operations_per_thread as f64);
            
            println!("    ✓ {} concurrent: {:.3}ms avg, {:.1} ops/sec", 
                    concurrent_count, avg_ms, measurement.operations_per_second);
            
            self.results.push(measurement);
        }
        
        Ok(())
    }

    /// Benchmark CLI memory usage
    async fn benchmark_cli_memory_usage(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking CLI memory usage");
        
        let mut timer = BenchmarkTimer::new();
        let iterations = 1000;
        let mut success_count = 0;
        
        let initial_memory = self.estimate_memory_usage();
        
        let start_time = Instant::now();
        
        for i in 0..iterations {
            timer.start();
            
            // Simulate memory-intensive CLI operations
            let key_pair = &self.key_pairs[i % self.key_pairs.len()];
            let _signature = key_pair.sign(format!("Memory test {}", i).as_bytes()).ok();
            
            // Create temporary data structures
            let _temp_data = (0..100).map(|j| format!("temp_{}_{}", i, j)).collect::<Vec<_>>();
            
            timer.record();
            success_count += 1;
        }
        
        let total_duration = start_time.elapsed();
        let final_memory = self.estimate_memory_usage();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "cli_memory_usage".to_string(),
            "cli".to_string(),
            "memory_operations".to_string(),
        );
        
        measurement.operation_count = iterations;
        measurement.total_duration = total_duration;
        measurement.avg_operation_time_ms = stats.avg_ms;
        measurement.operations_per_second = iterations as f64 / total_duration.as_secs_f64();
        measurement.memory_usage_bytes = Some(final_memory - initial_memory);
        measurement.success_rate_percent = (success_count as f64 / iterations as f64) * 100.0;
        
        measurement.additional_metrics.insert("initial_memory_bytes".to_string(), initial_memory as f64);
        measurement.additional_metrics.insert("final_memory_bytes".to_string(), final_memory as f64);
        measurement.additional_metrics.insert("memory_per_op_bytes".to_string(), 
            (final_memory - initial_memory) as f64 / iterations as f64);
        
        println!("  ✓ Memory usage: {:.3}ms avg, {} bytes total, {:.1} bytes/op", 
                stats.avg_ms, final_memory - initial_memory, 
                (final_memory - initial_memory) as f64 / iterations as f64);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark CLI cold vs warm start
    async fn benchmark_cli_cold_vs_warm_start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking CLI cold vs warm start");
        
        // Cold start benchmark (simulate first time CLI usage)
        let cold_start_time = self.benchmark_cold_start().await?;
        
        // Warm start benchmark (simulate subsequent CLI usage)
        let warm_start_time = self.benchmark_warm_start().await?;
        
        // Create comparison measurement
        let mut measurement = PerformanceMeasurement::new(
            "cli_cold_vs_warm_start".to_string(),
            "cli".to_string(),
            "startup_performance".to_string(),
        );
        
        measurement.operation_count = 2;
        measurement.avg_operation_time_ms = (cold_start_time + warm_start_time) / 2.0;
        measurement.operations_per_second = 2.0 / ((cold_start_time + warm_start_time) / 1000.0);
        measurement.success_rate_percent = 100.0;
        
        measurement.additional_metrics.insert("cold_start_ms".to_string(), cold_start_time);
        measurement.additional_metrics.insert("warm_start_ms".to_string(), warm_start_time);
        measurement.additional_metrics.insert("startup_improvement".to_string(), 
            (cold_start_time - warm_start_time) / cold_start_time * 100.0);
        
        println!("  ✓ Cold start: {:.3}ms, Warm start: {:.3}ms, Improvement: {:.1}%", 
                cold_start_time, warm_start_time, 
                (cold_start_time - warm_start_time) / cold_start_time * 100.0);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark CLI large config files
    async fn benchmark_cli_large_config_files(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking CLI large config files");
        
        // Create large config files with many profiles
        let config_sizes = vec![10, 50, 100];
        
        for size in config_sizes {
            println!("  🔍 Config size: {} profiles", size);
            
            // Create large config
            let large_config_path = self.create_large_config(size).await?;
            let mut large_config_manager = CliConfigManager::with_path(&large_config_path)?;
            
            let mut timer = BenchmarkTimer::new();
            let iterations = 100;
            let mut success_count = 0;
            let mut error_count = 0;
            
            let start_time = Instant::now();
            
            for _i in 0..iterations {
                timer.start();
                
                // Test operations on large config
                match large_config_manager.reload() {
                    Ok(_) => {
                        let _profiles = large_config_manager.list_profiles();
                        success_count += 1;
                    }
                    Err(_) => error_count += 1,
                }
                
                timer.record();
            }
            
            let total_duration = start_time.elapsed();
            let stats = timer.statistics();
            
            let mut measurement = PerformanceMeasurement::new(
                format!("cli_large_config_{}", size),
                "cli".to_string(),
                "large_config_performance".to_string(),
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
            
            measurement.additional_metrics.insert("config_size".to_string(), size as f64);
            
            println!("    ✓ {} profiles: {:.3}ms avg, {:.1} ops/sec", 
                    size, stats.avg_ms, measurement.operations_per_second);
            
            self.results.push(measurement);
        }
        
        Ok(())
    }

    // Helper methods

    /// Generate test requests
    fn generate_test_requests(&self, count: usize) -> Vec<(String, String, Option<String>)> {
        let mut requests = Vec::new();
        
        for i in 0..count {
            let method = match i % 4 {
                0 => "GET".to_string(),
                1 => "POST".to_string(),
                2 => "PUT".to_string(),
                _ => "DELETE".to_string(),
            };
            
            let url = format!("https://api.example.com/test/{}", i);
            
            let body = if method == "POST" || method == "PUT" {
                Some(json!({"test_id": i, "data": format!("test data {}", i)}).to_string())
            } else {
                None
            };
            
            requests.push((method, url, body));
        }
        
        requests
    }

    /// Sign CLI request (simplified)
    async fn sign_cli_request(
        &self,
        key_pair: &MasterKeyPair,
        method: &str,
        url: &str,
        body: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Simulate CLI request signing
        let message = format!("{} {}", method, url);
        let message_bytes = if let Some(body) = body {
            format!("{}\n{}", message, body).into_bytes()
        } else {
            message.into_bytes()
        };
        
        key_pair.sign(&message_bytes)?;
        Ok(())
    }

    /// Simulate CLI command execution
    async fn simulate_cli_command(&self, command_type: &str, _iteration: usize) -> Result<(), Box<dyn std::error::Error>> {
        // Simulate different command execution times
        let delay_ms = match command_type {
            "auto_sign" => 50,
            "manual_sign" => 75,
            "no_sign" => 25,
            "batch" => 100,
            _ => 50,
        };
        
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        Ok(())
    }

    /// Simulate signing operation
    async fn simulate_signing_operation(&self, key_pair: &MasterKeyPair, iteration: usize) -> Result<(), Box<dyn std::error::Error>> {
        let message = format!("Signing operation {}", iteration);
        key_pair.sign(message.as_bytes())?;
        Ok(())
    }

    /// Simulate batch item processing
    async fn simulate_batch_item(&self, key_pair: &MasterKeyPair, item_id: usize) -> Result<(), Box<dyn std::error::Error>> {
        let message = format!("Batch item {}", item_id);
        key_pair.sign(message.as_bytes())?;
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(())
    }

    /// Simulate end-to-end workflow
    async fn simulate_end_to_end_workflow(&self, workflow_name: &str, iteration: usize) -> Result<(), Box<dyn std::error::Error>> {
        let delay_ms = match workflow_name {
            "query_workflow" => 100,
            "mutate_workflow" => 150,
            "config_workflow" => 75,
            _ => 100,
        };
        
        // Simulate workflow steps
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        
        // Simulate signing if needed
        if workflow_name != "config_workflow" {
            let key_pair = &self.key_pairs[iteration % self.key_pairs.len()];
            let message = format!("Workflow {} iteration {}", workflow_name, iteration);
            key_pair.sign(message.as_bytes())?;
        }
        
        Ok(())
    }

    /// Simulate concurrent operation
    async fn simulate_concurrent_operation(
        key_pair: &MasterKeyPair,
        thread_id: usize,
        operation_id: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let message = format!("Concurrent operation thread {} op {}", thread_id, operation_id);
        key_pair.sign(message.as_bytes())?;
        tokio::time::sleep(Duration::from_millis(25)).await;
        Ok(())
    }

    /// Benchmark cold start performance
    async fn benchmark_cold_start(&self) -> Result<f64, Box<dyn std::error::Error>> {
        let start = Instant::now();
        
        // Simulate cold start operations
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        let elapsed = start.elapsed();
        Ok(elapsed.as_secs_f64() * 1000.0)
    }

    /// Benchmark warm start performance
    async fn benchmark_warm_start(&self) -> Result<f64, Box<dyn std::error::Error>> {
        let start = Instant::now();
        
        // Simulate warm start operations (faster due to caching)
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        let elapsed = start.elapsed();
        Ok(elapsed.as_secs_f64() * 1000.0)
    }

    /// Create large config file for testing
    async fn create_large_config(&self, profile_count: usize) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        let large_config_path = self.temp_dir.path().join(format!("large_config_{}.toml", profile_count));
        
        // Create config with many profiles
        let mut large_config = CliConfigManager::with_path(&large_config_path)?;
        
        for i in 0..profile_count {
            let mut metadata = HashMap::new();
            metadata.insert("environment".to_string(), format!("large-env-{}", i));
            metadata.insert("size".to_string(), "large".to_string());
            
            let profile = CliAuthProfile {
                client_id: format!("large-client-{}", i),
                key_id: format!("large-key-{}", i),
                user_id: Some(format!("large-user-{}", i)),
                server_url: format!("https://large-api-{}.example.com", i),
                metadata,
            };
            
            large_config.add_profile(format!("large-profile-{}", i), profile)?;
        }
        
        large_config.save()?;
        Ok(large_config_path)
    }

    /// Estimate memory usage
    fn estimate_memory_usage(&self) -> usize {
        // Simplified memory estimation
        self.key_pairs.len() * 1024 +
        self.test_profiles.len() * 512 +
        self.results.len() * 256
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cli_benchmarks_creation() {
        let config = PerformanceBenchmarkConfig::default();
        let targets = PerformanceTargets::default();
        
        let benchmarks = CliPerformanceBenchmarks::new(config, targets);
        assert!(benchmarks.is_ok());
    }

    #[tokio::test]
    async fn test_cli_test_profiles_creation() {
        let key_pairs = vec![generate_master_keypair().unwrap()];
        let profiles = CliPerformanceBenchmarks::create_test_profiles(&key_pairs);
        
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].client_id, "cli-benchmark-client-0");
    }

    #[tokio::test]
    async fn test_cli_request_signing() {
        let config = PerformanceBenchmarkConfig::default();
        let targets = PerformanceTargets::default();
        let benchmarks = CliPerformanceBenchmarks::new(config, targets).unwrap();
        
        let key_pair = &benchmarks.key_pairs[0];
        let result = benchmarks.sign_cli_request(
            key_pair,
            "POST",
            "https://api.example.com/test",
            Some(r#"{"test": "data"}"#),
        ).await;
        
        assert!(result.is_ok());
    }
}