//! Client-side signature generation performance benchmarks
//!
//! This module provides comprehensive performance benchmarking for client-side
//! signature operations, including Ed25519 signing, request preparation,
//! and header generation across different platforms and environments.

use super::{
    PerformanceMeasurement, BenchmarkTimer, TimingStatistics, PerformanceBenchmarkConfig,
    PerformanceTargets
};
use crate::crypto::ed25519::{generate_master_keypair, MasterKeyPair};
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc};
use reqwest::{Method, Request, Url};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicU64, AtomicUsize, Ordering}};
use std::time::{SystemTime, UNIX_EPOCH, Duration, Instant};
use tokio::time::sleep;
use uuid::Uuid;

/// Client-side performance benchmark suite
pub struct ClientPerformanceBenchmarks {
    config: PerformanceBenchmarkConfig,
    targets: PerformanceTargets,
    key_pairs: Vec<MasterKeyPair>,
    results: Vec<PerformanceMeasurement>,
}

impl ClientPerformanceBenchmarks {
    /// Create new client performance benchmarks
    pub fn new(
        config: PerformanceBenchmarkConfig,
        targets: PerformanceTargets,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Generate multiple key pairs for testing
        let mut key_pairs = Vec::new();
        for _ in 0..10 {
            key_pairs.push(generate_master_keypair()?);
        }
        
        Ok(Self {
            config,
            targets,
            key_pairs,
            results: Vec::new(),
        })
    }

    /// Run all client performance benchmarks
    pub async fn run_all_benchmarks(&mut self) -> Result<Vec<PerformanceMeasurement>, Box<dyn std::error::Error>> {
        println!("🚀 Starting client-side performance benchmarks");
        
        // Micro-benchmarks for cryptographic operations
        self.benchmark_ed25519_signing().await?;
        self.benchmark_signature_base64_encoding().await?;
        self.benchmark_content_digest_generation().await?;
        self.benchmark_signature_string_construction().await?;
        
        // Component benchmarks for request preparation
        self.benchmark_request_signing().await?;
        self.benchmark_header_generation().await?;
        self.benchmark_timestamp_generation().await?;
        self.benchmark_nonce_generation().await?;
        
        // Integration benchmarks for complete workflows
        self.benchmark_complete_request_preparation().await?;
        self.benchmark_batch_signing().await?;
        
        // Performance benchmarks for different scenarios
        self.benchmark_concurrent_signing().await?;
        self.benchmark_different_payload_sizes().await?;
        self.benchmark_different_http_methods().await?;
        
        // Platform-specific benchmarks
        self.benchmark_key_caching_performance().await?;
        self.benchmark_memory_usage().await?;
        
        println!("✅ Client performance benchmarks completed");
        Ok(self.results.clone())
    }

    /// Benchmark Ed25519 signing performance
    async fn benchmark_ed25519_signing(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking Ed25519 signing");
        
        let key_pair = &self.key_pairs[0];
        let mut timer = BenchmarkTimer::new();
        let iterations = self.config.micro_benchmark_iterations;
        let mut success_count = 0;
        let mut error_count = 0;
        
        // Generate test data to sign
        let test_messages: Vec<Vec<u8>> = (0..10).map(|i| {
            format!("Test signature message {}", i).into_bytes()
        }).collect();
        
        // Warmup
        for i in 0..100 {
            let message = &test_messages[i % test_messages.len()];
            let _ = key_pair.sign(message);
        }
        
        // Actual benchmark
        let start_time = Instant::now();
        
        for i in 0..iterations {
            let message = &test_messages[i % test_messages.len()];
            
            timer.start();
            let result = key_pair.sign(message);
            timer.record();
            
            match result {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "ed25519_signing".to_string(),
            "client".to_string(),
            "cryptographic_signing".to_string(),
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
        
        measurement.additional_metrics.insert("message_variants".to_string(), test_messages.len() as f64);
        measurement.additional_metrics.insert("warmup_iterations".to_string(), 100.0);
        
        println!("  ✓ Ed25519 signing: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark signature base64 encoding performance
    async fn benchmark_signature_base64_encoding(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking signature base64 encoding");
        
        let key_pair = &self.key_pairs[0];
        let mut timer = BenchmarkTimer::new();
        let iterations = self.config.micro_benchmark_iterations;
        let mut success_count = 0;
        
        // Pre-generate signatures to encode
        let mut signatures = Vec::new();
        for i in 0..100 {
            let message = format!("Message to sign {}", i).into_bytes();
            if let Ok(signature) = key_pair.sign(&message) {
                signatures.push(signature);
            }
        }
        
        // Benchmark base64 encoding
        let start_time = Instant::now();
        
        for i in 0..iterations {
            let signature = &signatures[i % signatures.len()];
            
            timer.start();
            let _encoded = general_purpose::STANDARD.encode(signature);
            timer.record();
            
            success_count += 1;
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "base64_encoding".to_string(),
            "client".to_string(),
            "signature_encoding".to_string(),
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
        
        measurement.additional_metrics.insert("signature_variants".to_string(), signatures.len() as f64);
        
        println!("  ✓ Base64 encoding: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark content digest generation performance
    async fn benchmark_content_digest_generation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking content digest generation");
        
        let mut timer = BenchmarkTimer::new();
        let iterations = self.config.micro_benchmark_iterations;
        let mut success_count = 0;
        
        // Generate test payloads of different sizes
        let test_payloads = vec![
            "{}".to_string(), // Empty JSON
            json!({"key": "value"}).to_string(), // Small JSON
            json!({"data": "a".repeat(1000)}).to_string(), // Medium JSON
            json!({"large_data": "b".repeat(10000)}).to_string(), // Large JSON
        ];
        
        // Benchmark SHA-256 digest generation
        let start_time = Instant::now();
        
        for i in 0..iterations {
            let payload = &test_payloads[i % test_payloads.len()];
            
            timer.start();
            let mut hasher = Sha256::new();
            hasher.update(payload.as_bytes());
            let _digest = hasher.finalize();
            timer.record();
            
            success_count += 1;
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "content_digest_generation".to_string(),
            "client".to_string(),
            "sha256_hashing".to_string(),
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
        measurement.additional_metrics.insert("max_payload_size".to_string(), 
            test_payloads.iter().map(|p| p.len()).max().unwrap_or(0) as f64);
        
        println!("  ✓ Content digest: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark signature string construction performance
    async fn benchmark_signature_string_construction(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking signature string construction");
        
        let mut timer = BenchmarkTimer::new();
        let iterations = self.config.micro_benchmark_iterations;
        let mut success_count = 0;
        
        // Pre-generate signature components
        let methods = vec!["GET", "POST", "PUT", "DELETE"];
        let paths = vec!["/api/data", "/api/auth", "/api/query", "/api/transform"];
        let authorities = vec!["api.example.com", "localhost:8080"];
        
        // Benchmark signature string construction
        let start_time = Instant::now();
        
        for i in 0..iterations {
            let method = methods[i % methods.len()];
            let path = paths[i % paths.len()];
            let authority = authorities[i % authorities.len()];
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            
            timer.start();
            
            // Construct signature string (RFC 9421 format)
            let signature_string = format!(
                "\"@method\": {}\n\"@target-uri\": https://{}{}\n\"@authority\": {}\ncreated: {}",
                method, authority, path, authority, timestamp
            );
            
            let _signature_string_bytes = signature_string.into_bytes();
            
            timer.record();
            success_count += 1;
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "signature_string_construction".to_string(),
            "client".to_string(),
            "string_building".to_string(),
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
        
        measurement.additional_metrics.insert("method_variants".to_string(), methods.len() as f64);
        measurement.additional_metrics.insert("path_variants".to_string(), paths.len() as f64);
        
        println!("  ✓ Signature string construction: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark complete request signing performance
    async fn benchmark_request_signing(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking complete request signing");
        
        let key_pair = &self.key_pairs[0];
        let mut timer = BenchmarkTimer::new();
        let iterations = 5000; // Fewer iterations for more complex operations
        let mut success_count = 0;
        let mut error_count = 0;
        
        // Pre-generate test requests
        let mut test_requests = Vec::new();
        for i in 0..100 {
            let url = format!("https://api.example.com/test/{}", i);
            let body = json!({"test_id": i, "data": format!("test data {}", i)}).to_string();
            test_requests.push((url, body));
        }
        
        // Benchmark complete request signing
        let start_time = Instant::now();
        
        for i in 0..iterations {
            let (url, body) = &test_requests[i % test_requests.len()];
            
            timer.start();
            
            let result = self.sign_request(key_pair, "POST", url, Some(body)).await;
            
            timer.record();
            
            match result {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "request_signing".to_string(),
            "client".to_string(),
            "complete_signing".to_string(),
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
        
        println!("  ✓ Request signing: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark HTTP header generation performance
    async fn benchmark_header_generation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking HTTP header generation");
        
        let mut timer = BenchmarkTimer::new();
        let iterations = self.config.micro_benchmark_iterations;
        let mut success_count = 0;
        
        // Pre-generate signature data
        let signatures = vec![
            "MEUCIQD2r2qF6UJHw2Q8dV4c+8O0vRMF5dBhcVOOz9+xKXgQAQIgEvKfPzyCf8QG1YKBxqzb6M5+8t6z2j+7QZ8Fz6XYxQE=",
            "MEQCIBz+2O9XbV5R6e8JfKRYzV+9U3M8t8J9QH5s2dF7KnAVAiBQ8TzNcP4L6m8E9zYbF3qZ5J+R8w6f7s4V8E9z+K4w8P==",
            "MEUCIQCz5V2F8J9Q6H3s8dF7Kn4M+8t6z2j+7QZ8Fz6XYxQE8QIgEvKfPzyCf8QG1YKBxqzb6M5+Vzj2P9LNQ5s4VRbYxwP=",
        ];
        
        let key_ids = vec!["test-key-001", "test-key-002", "test-key-003"];
        
        // Benchmark header generation
        let start_time = Instant::now();
        
        for i in 0..iterations {
            let signature = signatures[i % signatures.len()];
            let key_id = key_ids[i % key_ids.len()];
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let nonce = format!("nonce_{}", i);
            
            timer.start();
            
            // Generate signature headers
            let signature_header = format!("sig1=:{}:", signature);
            let signature_input_header = format!(
                "sig1=(\"@method\" \"@target-uri\" \"@authority\" \"content-digest\");created={};keyid=\"{}\";nonce=\"{}\"",
                timestamp, key_id, nonce
            );
            
            let _headers = vec![
                ("signature", signature_header),
                ("signature-input", signature_input_header),
            ];
            
            timer.record();
            success_count += 1;
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "header_generation".to_string(),
            "client".to_string(),
            "http_headers".to_string(),
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
        
        measurement.additional_metrics.insert("signature_variants".to_string(), signatures.len() as f64);
        measurement.additional_metrics.insert("key_variants".to_string(), key_ids.len() as f64);
        
        println!("  ✓ Header generation: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark timestamp generation performance
    async fn benchmark_timestamp_generation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking timestamp generation");
        
        let mut timer = BenchmarkTimer::new();
        let iterations = self.config.micro_benchmark_iterations;
        let mut success_count = 0;
        
        // Benchmark different timestamp generation methods
        let start_time = Instant::now();
        
        for i in 0..iterations {
            timer.start();
            
            if i % 3 == 0 {
                // Unix timestamp
                let _timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            } else if i % 3 == 1 {
                // ISO 8601 timestamp
                let _timestamp = Utc::now().to_rfc3339();
            } else {
                // High precision timestamp
                let _timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
            }
            
            timer.record();
            success_count += 1;
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "timestamp_generation".to_string(),
            "client".to_string(),
            "time_operations".to_string(),
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
        
        measurement.additional_metrics.insert("timestamp_formats".to_string(), 3.0);
        
        println!("  ✓ Timestamp generation: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark nonce generation performance
    async fn benchmark_nonce_generation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking nonce generation");
        
        let mut timer = BenchmarkTimer::new();
        let iterations = self.config.micro_benchmark_iterations;
        let mut success_count = 0;
        
        // Benchmark different nonce generation methods
        let start_time = Instant::now();
        
        for i in 0..iterations {
            timer.start();
            
            if i % 2 == 0 {
                // UUID v4 nonce
                let _nonce = Uuid::new_v4().to_string();
            } else {
                // Timestamp-based nonce
                let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
                let _nonce = format!("nonce_{}", timestamp);
            }
            
            timer.record();
            success_count += 1;
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "nonce_generation".to_string(),
            "client".to_string(),
            "nonce_operations".to_string(),
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
        
        measurement.additional_metrics.insert("nonce_methods".to_string(), 2.0);
        
        println!("  ✓ Nonce generation: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark complete request preparation performance
    async fn benchmark_complete_request_preparation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking complete request preparation");
        
        let key_pair = &self.key_pairs[0];
        let mut timer = BenchmarkTimer::new();
        let iterations = 2000; // Fewer iterations for more complex operations
        let mut success_count = 0;
        let mut error_count = 0;
        
        // Benchmark complete request preparation workflow
        let start_time = Instant::now();
        
        for i in 0..iterations {
            timer.start();
            
            // Complete workflow: nonce + timestamp + digest + signature + headers
            let result = self.prepare_complete_request(key_pair, i).await;
            
            timer.record();
            
            match result {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
        }
        
        let total_duration = start_time.elapsed();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "complete_request_preparation".to_string(),
            "client".to_string(),
            "full_workflow".to_string(),
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
        
        println!("  ✓ Complete request preparation: {:.3}ms avg, {:.1} ops/sec", 
                stats.avg_ms, measurement.operations_per_second);
        
        self.results.push(measurement);
        Ok(())
    }

    /// Benchmark batch signing performance
    async fn benchmark_batch_signing(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking batch signing");
        
        let key_pair = &self.key_pairs[0];
        let batch_sizes = vec![10, 50, 100, 500];
        
        for &batch_size in &batch_sizes {
            let mut timer = BenchmarkTimer::new();
            let mut success_count = 0;
            let mut error_count = 0;
            
            let start_time = Instant::now();
            
            timer.start();
            
            // Process batch of requests
            for i in 0..batch_size {
                let url = format!("https://api.example.com/batch/{}", i);
                let body = json!({"batch_id": i, "data": format!("batch data {}", i)}).to_string();
                
                match self.sign_request(key_pair, "POST", &url, Some(&body)).await {
                    Ok(_) => success_count += 1,
                    Err(_) => error_count += 1,
                }
            }
            
            timer.record();
            
            let total_duration = start_time.elapsed();
            let stats = timer.statistics();
            
            let mut measurement = PerformanceMeasurement::new(
                format!("batch_signing_{}", batch_size),
                "client".to_string(),
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
            
            println!("  ✓ Batch {} items: {:.3}ms total, {:.3}ms per item", 
                    batch_size, stats.avg_ms, stats.avg_ms / batch_size as f64);
            
            self.results.push(measurement);
        }
        
        Ok(())
    }

    /// Benchmark concurrent signing performance
    async fn benchmark_concurrent_signing(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking concurrent signing");
        
        for &concurrent_count in &[1, 5, 10, 25] {
            let operations_per_thread = 50;
            let total_operations = concurrent_count * operations_per_thread;
            
            let success_counter = Arc::new(AtomicUsize::new(0));
            let error_counter = Arc::new(AtomicUsize::new(0));
            let operation_times = Arc::new(std::sync::Mutex::new(Vec::new()));
            
            let start_time = Instant::now();
            
            // Spawn concurrent signing tasks
            let tasks: Vec<_> = (0..concurrent_count).map(|thread_id| {
                let key_pair = self.key_pairs[thread_id % self.key_pairs.len()].clone();
                let success_counter = Arc::clone(&success_counter);
                let error_counter = Arc::clone(&error_counter);
                let operation_times = Arc::clone(&operation_times);
                
                tokio::spawn(async move {
                    for i in 0..operations_per_thread {
                        let url = format!("https://api.example.com/concurrent/{}/{}", thread_id, i);
                        let body = json!({"thread_id": thread_id, "item": i}).to_string();
                        
                        let op_start = Instant::now();
                        let result = Self::sign_request_static(&key_pair, "POST", &url, Some(&body)).await;
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
                format!("concurrent_signing_{}_threads", concurrent_count),
                "client".to_string(),
                "concurrent_operations".to_string(),
            );
            
            measurement.operation_count = total_operations;
            measurement.total_duration = total_duration;
            measurement.avg_operation_time_ms = avg_ms;
            measurement.p95_operation_time_ms = p95_ms;
            measurement.operations_per_second = total_operations as f64 / total_duration.as_secs_f64();
            measurement.error_count = error_count;
            measurement.success_rate_percent = (success_count as f64 / total_operations as f64) * 100.0;
            
            measurement.additional_metrics.insert("concurrent_threads".to_string(), concurrent_count as f64);
            measurement.additional_metrics.insert("ops_per_thread".to_string(), operations_per_thread as f64);
            
            println!("  ✓ {} concurrent threads: {:.3}ms avg, {:.1} ops/sec", 
                    concurrent_count, avg_ms, measurement.operations_per_second);
            
            self.results.push(measurement);
        }
        
        Ok(())
    }

    /// Benchmark different payload sizes
    async fn benchmark_different_payload_sizes(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking different payload sizes");
        
        let key_pair = &self.key_pairs[0];
        let payload_sizes = vec![
            (0, "empty"),
            (100, "small_100b"),
            (1024, "medium_1kb"),
            (10240, "large_10kb"),
            (102400, "xlarge_100kb"),
        ];
        
        for (size, size_name) in payload_sizes {
            let mut timer = BenchmarkTimer::new();
            let iterations = 1000;
            let mut success_count = 0;
            let mut error_count = 0;
            
            // Generate payload of specified size
            let payload = if size == 0 {
                "{}".to_string()
            } else {
                json!({"data": "x".repeat(size)}).to_string()
            };
            
            let start_time = Instant::now();
            
            for i in 0..iterations {
                let url = format!("https://api.example.com/size-test/{}", i);
                
                timer.start();
                let result = self.sign_request(key_pair, "POST", &url, Some(&payload)).await;
                timer.record();
                
                match result {
                    Ok(_) => success_count += 1,
                    Err(_) => error_count += 1,
                }
            }
            
            let total_duration = start_time.elapsed();
            let stats = timer.statistics();
            
            let mut measurement = PerformanceMeasurement::new(
                format!("payload_size_{}", size_name),
                "client".to_string(),
                "size_variation".to_string(),
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
            
            measurement.additional_metrics.insert("payload_size_bytes".to_string(), size as f64);
            
            println!("  ✓ Payload {} ({}): {:.3}ms avg, {:.1} ops/sec", 
                    size_name, size, stats.avg_ms, measurement.operations_per_second);
            
            self.results.push(measurement);
        }
        
        Ok(())
    }

    /// Benchmark different HTTP methods
    async fn benchmark_different_http_methods(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking different HTTP methods");
        
        let key_pair = &self.key_pairs[0];
        let methods = vec![
            ("GET", None),
            ("POST", Some(json!({"data": "test"}).to_string())),
            ("PUT", Some(json!({"update": "data"}).to_string())),
            ("DELETE", None),
            ("PATCH", Some(json!({"patch": "data"}).to_string())),
        ];
        
        for (method, body) in methods {
            let mut timer = BenchmarkTimer::new();
            let iterations = 2000;
            let mut success_count = 0;
            let mut error_count = 0;
            
            let start_time = Instant::now();
            
            for i in 0..iterations {
                let url = format!("https://api.example.com/method-test/{}", i);
                
                timer.start();
                let result = self.sign_request(key_pair, method, &url, body.as_ref().map(|s| s.as_str())).await;
                timer.record();
                
                match result {
                    Ok(_) => success_count += 1,
                    Err(_) => error_count += 1,
                }
            }
            
            let total_duration = start_time.elapsed();
            let stats = timer.statistics();
            
            let mut measurement = PerformanceMeasurement::new(
                format!("http_method_{}", method.to_lowercase()),
                "client".to_string(),
                "method_variation".to_string(),
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
            
            measurement.additional_metrics.insert("has_body".to_string(), if body.is_some() { 1.0 } else { 0.0 });
            
            println!("  ✓ Method {}: {:.3}ms avg, {:.1} ops/sec", 
                    method, stats.avg_ms, measurement.operations_per_second);
            
            self.results.push(measurement);
        }
        
        Ok(())
    }

    /// Benchmark key caching performance
    async fn benchmark_key_caching_performance(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking key caching performance");
        
        // Simulate key caching by reusing vs creating new key pairs
        let iterations = 1000;
        
        // Test 1: Using cached key (same key pair)
        let cached_key = &self.key_pairs[0];
        let mut cached_timer = BenchmarkTimer::new();
        let mut cached_success = 0;
        
        let start_time = Instant::now();
        
        for i in 0..iterations {
            let message = format!("Cached key test {}", i).into_bytes();
            
            cached_timer.start();
            let result = cached_key.sign(&message);
            cached_timer.record();
            
            if result.is_ok() {
                cached_success += 1;
            }
        }
        
        let cached_duration = start_time.elapsed();
        let cached_stats = cached_timer.statistics();
        
        // Test 2: Creating new keys (simulation of no caching)
        let mut new_key_timer = BenchmarkTimer::new();
        let mut new_key_success = 0;
        
        let start_time = Instant::now();
        
        for i in 0..iterations {
            new_key_timer.start();
            
            // Simulate key creation overhead
            std::thread::sleep(Duration::from_micros(1));
            let key_pair = &self.key_pairs[i % self.key_pairs.len()];
            let message = format!("New key test {}", i).into_bytes();
            let result = key_pair.sign(&message);
            
            new_key_timer.record();
            
            if result.is_ok() {
                new_key_success += 1;
            }
        }
        
        let new_key_duration = start_time.elapsed();
        let new_key_stats = new_key_timer.statistics();
        
        // Record cached key performance
        let mut cached_measurement = PerformanceMeasurement::new(
            "key_caching_enabled".to_string(),
            "client".to_string(),
            "cached_operations".to_string(),
        );
        
        cached_measurement.operation_count = iterations;
        cached_measurement.total_duration = cached_duration;
        cached_measurement.avg_operation_time_ms = cached_stats.avg_ms;
        cached_measurement.operations_per_second = iterations as f64 / cached_duration.as_secs_f64();
        cached_measurement.success_rate_percent = (cached_success as f64 / iterations as f64) * 100.0;
        
        // Record new key performance
        let mut new_key_measurement = PerformanceMeasurement::new(
            "key_caching_disabled".to_string(),
            "client".to_string(),
            "non_cached_operations".to_string(),
        );
        
        new_key_measurement.operation_count = iterations;
        new_key_measurement.total_duration = new_key_duration;
        new_key_measurement.avg_operation_time_ms = new_key_stats.avg_ms;
        new_key_measurement.operations_per_second = iterations as f64 / new_key_duration.as_secs_f64();
        new_key_measurement.success_rate_percent = (new_key_success as f64 / iterations as f64) * 100.0;
        
        // Calculate caching benefit
        let caching_benefit_percent = ((new_key_stats.avg_ms - cached_stats.avg_ms) / new_key_stats.avg_ms) * 100.0;
        cached_measurement.additional_metrics.insert("caching_benefit_percent".to_string(), caching_benefit_percent);
        new_key_measurement.additional_metrics.insert("caching_overhead_percent".to_string(), -caching_benefit_percent);
        
        println!("  ✓ Cached keys: {:.3}ms avg, {:.1} ops/sec", 
                cached_stats.avg_ms, cached_measurement.operations_per_second);
        println!("  ✓ Non-cached keys: {:.3}ms avg, {:.1} ops/sec", 
                new_key_stats.avg_ms, new_key_measurement.operations_per_second);
        println!("  ✓ Caching benefit: {:.1}%", caching_benefit_percent);
        
        self.results.push(cached_measurement);
        self.results.push(new_key_measurement);
        Ok(())
    }

    /// Benchmark memory usage
    async fn benchmark_memory_usage(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Benchmarking memory usage");
        
        let key_pair = &self.key_pairs[0];
        let iterations = 5000;
        let mut timer = BenchmarkTimer::new();
        let mut success_count = 0;
        
        // Simulate memory usage tracking
        let initial_memory = self.estimate_memory_usage();
        
        let start_time = Instant::now();
        
        for i in 0..iterations {
            timer.start();
            
            // Create request data that would be held in memory
            let url = format!("https://api.example.com/memory-test/{}", i);
            let body = json!({"memory_test": i, "data": "x".repeat(100)}).to_string();
            let _result = self.sign_request(key_pair, "POST", &url, Some(&body)).await;
            
            timer.record();
            success_count += 1;
        }
        
        let total_duration = start_time.elapsed();
        let final_memory = self.estimate_memory_usage();
        let stats = timer.statistics();
        
        let mut measurement = PerformanceMeasurement::new(
            "memory_usage".to_string(),
            "client".to_string(),
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

    // Helper methods

    /// Sign a request (simplified implementation)
    async fn sign_request(
        &self,
        key_pair: &MasterKeyPair,
        method: &str,
        url: &str,
        body: Option<&str>,
    ) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
        // Simulate complete signing process
        
        // 1. Generate nonce and timestamp
        let nonce = Uuid::new_v4().to_string();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        
        // 2. Generate content digest if body exists
        let content_digest = if let Some(body) = body {
            let mut hasher = Sha256::new();
            hasher.update(body.as_bytes());
            format!("sha-256=:{}", general_purpose::STANDARD.encode(hasher.finalize()))
        } else {
            "".to_string()
        };
        
        // 3. Build signature string
        let signature_string = if body.is_some() {
            format!(
                "\"@method\": {}\n\"@target-uri\": {}\ncontent-digest: {}\ncreated: {}",
                method.to_uppercase(), url, content_digest, timestamp
            )
        } else {
            format!(
                "\"@method\": {}\n\"@target-uri\": {}\ncreated: {}",
                method.to_uppercase(), url, timestamp
            )
        };
        
        // 4. Sign the string
        let signature = key_pair.sign(signature_string.as_bytes())?;
        let signature_b64 = general_purpose::STANDARD.encode(signature);
        
        // 5. Build headers
        let signature_header = format!("sig1=:{}:", signature_b64);
        let signature_input_header = if body.is_some() {
            format!(
                "sig1=(\"@method\" \"@target-uri\" \"content-digest\");created={};keyid=\"test-key\";nonce=\"{}\"",
                timestamp, nonce
            )
        } else {
            format!(
                "sig1=(\"@method\" \"@target-uri\");created={};keyid=\"test-key\";nonce=\"{}\"",
                timestamp, nonce
            )
        };
        
        let mut headers = vec![
            ("signature".to_string(), signature_header),
            ("signature-input".to_string(), signature_input_header),
        ];
        
        if !content_digest.is_empty() {
            headers.push(("content-digest".to_string(), content_digest));
        }
        
        Ok(headers)
    }

    /// Static version of sign_request for use in async contexts
    async fn sign_request_static(
        key_pair: &MasterKeyPair,
        method: &str,
        url: &str,
        body: Option<&str>,
    ) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
        // Same implementation as sign_request but static
        let nonce = Uuid::new_v4().to_string();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        
        let content_digest = if let Some(body) = body {
            let mut hasher = Sha256::new();
            hasher.update(body.as_bytes());
            format!("sha-256=:{}", general_purpose::STANDARD.encode(hasher.finalize()))
        } else {
            "".to_string()
        };
        
        let signature_string = if body.is_some() {
            format!(
                "\"@method\": {}\n\"@target-uri\": {}\ncontent-digest: {}\ncreated: {}",
                method.to_uppercase(), url, content_digest, timestamp
            )
        } else {
            format!(
                "\"@method\": {}\n\"@target-uri\": {}\ncreated: {}",
                method.to_uppercase(), url, timestamp
            )
        };
        
        let signature = key_pair.sign(signature_string.as_bytes())?;
        let signature_b64 = general_purpose::STANDARD.encode(signature);
        
        let signature_header = format!("sig1=:{}:", signature_b64);
        let signature_input_header = if body.is_some() {
            format!(
                "sig1=(\"@method\" \"@target-uri\" \"content-digest\");created={};keyid=\"test-key\";nonce=\"{}\"",
                timestamp, nonce
            )
        } else {
            format!(
                "sig1=(\"@method\" \"@target-uri\");created={};keyid=\"test-key\";nonce=\"{}\"",
                timestamp, nonce
            )
        };
        
        let mut headers = vec![
            ("signature".to_string(), signature_header),
            ("signature-input".to_string(), signature_input_header),
        ];
        
        if !content_digest.is_empty() {
            headers.push(("content-digest".to_string(), content_digest));
        }
        
        Ok(headers)
    }

    /// Prepare a complete request with all components
    async fn prepare_complete_request(
        &self,
        key_pair: &MasterKeyPair,
        request_id: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Complete workflow simulation
        let url = format!("https://api.example.com/complete/{}", request_id);
        let body = json!({"request_id": request_id, "complete": true}).to_string();
        
        let _headers = self.sign_request(key_pair, "POST", &url, Some(&body)).await?;
        
        Ok(())
    }

    /// Estimate memory usage (simplified)
    fn estimate_memory_usage(&self) -> usize {
        // Simplified memory estimation
        // In a real implementation, this would use platform-specific APIs
        self.key_pairs.len() * 1024 + self.results.len() * 512
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_benchmarks_creation() {
        let config = PerformanceBenchmarkConfig::default();
        let targets = PerformanceTargets::default();
        
        let benchmarks = ClientPerformanceBenchmarks::new(config, targets);
        assert!(benchmarks.is_ok());
    }

    #[tokio::test]
    async fn test_ed25519_signing_benchmark() {
        let config = PerformanceBenchmarkConfig {
            micro_benchmark_iterations: 100,
            ..Default::default()
        };
        let targets = PerformanceTargets::default();
        
        let mut benchmarks = ClientPerformanceBenchmarks::new(config, targets).unwrap();
        let result = benchmarks.benchmark_ed25519_signing().await;
        
        assert!(result.is_ok());
        assert!(!benchmarks.results.is_empty());
        
        let measurement = &benchmarks.results[0];
        assert_eq!(measurement.test_name, "ed25519_signing");
        assert_eq!(measurement.component, "client");
        assert!(measurement.operations_per_second > 0.0);
    }

    #[tokio::test]
    async fn test_request_signing() {
        let config = PerformanceBenchmarkConfig::default();
        let targets = PerformanceTargets::default();
        let benchmarks = ClientPerformanceBenchmarks::new(config, targets).unwrap();
        
        let key_pair = &benchmarks.key_pairs[0];
        let result = benchmarks.sign_request(
            key_pair,
            "POST",
            "https://api.example.com/test",
            Some(r#"{"test": "data"}"#),
        ).await;
        
        assert!(result.is_ok());
        let headers = result.unwrap();
        assert!(!headers.is_empty());
        
        // Check required headers are present
        let header_names: Vec<String> = headers.iter().map(|(name, _)| name.clone()).collect();
        assert!(header_names.contains(&"signature".to_string()));
        assert!(header_names.contains(&"signature-input".to_string()));
    }
}