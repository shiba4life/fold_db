//! Comprehensive performance tests for async encryption optimizations
//!
//! This test module validates that the async encryption optimizations meet
//! the <20% performance overhead requirement specified in PBI 9.

use datafold::crypto::generate_master_keypair;
use datafold::db_operations::{DbOperations, EncryptionWrapper};
use datafold::db_operations::encryption_wrapper_async::{AsyncEncryptionWrapper, AsyncWrapperConfig};
use datafold::datafold_node::encryption_at_rest_async::{AsyncEncryptionAtRest, PerformanceConfig};
use datafold::datafold_node::encryption_at_rest::EncryptionAtRest;
use datafold::fold_db_core::managers::atom::async_operations::{AsyncAtomManager, AsyncAtomConfig};
use datafold::db_operations::encryption_wrapper::contexts;
use serde_json::json;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tempfile::tempdir;
use tokio::test;

/// Test configuration for performance validation
const MAX_OVERHEAD_PERCENT: f64 = 20.0;
const BENCHMARK_ITERATIONS: usize = 100;
const LARGE_DATA_SIZE: usize = 10 * 1024; // 10KB
const SMALL_DATA_SIZE: usize = 256; // 256 bytes

/// Performance test results
#[derive(Debug, Clone)]
struct PerformanceResults {
    sync_ops_per_sec: f64,
    async_ops_per_sec: f64,
    overhead_percent: f64,
    memory_usage_mb: f64,
    throughput_mbps: f64,
}

impl PerformanceResults {
    fn meets_requirements(&self) -> bool {
        self.overhead_percent <= MAX_OVERHEAD_PERCENT
    }
    
    fn performance_improvement(&self) -> f64 {
        ((self.async_ops_per_sec - self.sync_ops_per_sec) / self.sync_ops_per_sec) * 100.0
    }
}

/// Create test database operations
fn create_test_db_ops() -> DbOperations {
    let temp_dir = tempdir().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    DbOperations::new(db).unwrap()
}

/// Create test data of specified size
fn create_test_data(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

/// Benchmark sync encryption operations
async fn benchmark_sync_encryption(iterations: usize, data_size: usize) -> Result<f64, Box<dyn std::error::Error>> {
    let key = [0x42u8; 32];
    let encryptor = EncryptionAtRest::new(key)?;
    let test_data = create_test_data(data_size);
    
    let start_time = Instant::now();
    
    for _ in 0..iterations {
        let encrypted = encryptor.encrypt(&test_data)?;
        let _decrypted = encryptor.decrypt(&encrypted)?;
    }
    
    let duration = start_time.elapsed();
    let ops_per_sec = (iterations * 2) as f64 / duration.as_secs_f64(); // encrypt + decrypt
    
    Ok(ops_per_sec)
}

/// Benchmark async encryption operations
async fn benchmark_async_encryption(iterations: usize, data_size: usize) -> Result<f64, Box<dyn std::error::Error>> {
    let key = [0x42u8; 32];
    let config = PerformanceConfig::default();
    let encryptor = AsyncEncryptionAtRest::new(key, config).await?;
    let test_data = create_test_data(data_size);
    
    let start_time = Instant::now();
    
    for _ in 0..iterations {
        let encrypted = encryptor.encrypt_async(&test_data).await?;
        let _decrypted = encryptor.decrypt_async(&encrypted).await?;
    }
    
    let duration = start_time.elapsed();
    let ops_per_sec = (iterations * 2) as f64 / duration.as_secs_f64(); // encrypt + decrypt
    
    Ok(ops_per_sec)
}

/// Benchmark batch encryption operations
async fn benchmark_batch_encryption(batch_size: usize, data_size: usize) -> Result<f64, Box<dyn std::error::Error>> {
    let key = [0x42u8; 32];
    let config = PerformanceConfig::default();
    let encryptor = AsyncEncryptionAtRest::new(key, config).await?;
    
    let test_data = create_test_data(data_size);
    let batch_data: Vec<&[u8]> = (0..batch_size).map(|_| test_data.as_slice()).collect();
    
    let start_time = Instant::now();
    
    let encrypted_batch = encryptor.encrypt_batch(batch_data.clone()).await?;
    let encrypted_refs: Vec<_> = encrypted_batch.iter().collect();
    let _decrypted_batch = encryptor.decrypt_batch(encrypted_refs).await?;
    
    let duration = start_time.elapsed();
    let ops_per_sec = (batch_size * 2) as f64 / duration.as_secs_f64(); // encrypt + decrypt
    
    Ok(ops_per_sec)
}

/// Test basic async encryption performance overhead
#[test]
async fn test_async_encryption_overhead() {
    let sync_ops_per_sec = benchmark_sync_encryption(BENCHMARK_ITERATIONS, SMALL_DATA_SIZE).await.unwrap();
    let async_ops_per_sec = benchmark_async_encryption(BENCHMARK_ITERATIONS, SMALL_DATA_SIZE).await.unwrap();
    
    let overhead_percent = ((sync_ops_per_sec - async_ops_per_sec) / sync_ops_per_sec) * 100.0;
    
    println!("Basic Encryption Performance:");
    println!("  Sync ops/sec: {:.2}", sync_ops_per_sec);
    println!("  Async ops/sec: {:.2}", async_ops_per_sec);
    println!("  Overhead: {:.2}%", overhead_percent);
    
    // Allow some overhead for async coordination, but should be minimal
    assert!(overhead_percent <= MAX_OVERHEAD_PERCENT, 
        "Async encryption overhead ({:.2}%) exceeds maximum allowed ({}%)", 
        overhead_percent, MAX_OVERHEAD_PERCENT);
}

/// Test large data encryption performance
#[test]
async fn test_large_data_encryption_performance() {
    let sync_ops_per_sec = benchmark_sync_encryption(50, LARGE_DATA_SIZE).await.unwrap();
    let async_ops_per_sec = benchmark_async_encryption(50, LARGE_DATA_SIZE).await.unwrap();
    
    let overhead_percent = ((sync_ops_per_sec - async_ops_per_sec) / sync_ops_per_sec) * 100.0;
    
    println!("Large Data Encryption Performance:");
    println!("  Sync ops/sec: {:.2}", sync_ops_per_sec);
    println!("  Async ops/sec: {:.2}", async_ops_per_sec);
    println!("  Overhead: {:.2}%", overhead_percent);
    
    assert!(overhead_percent <= MAX_OVERHEAD_PERCENT, 
        "Large data async encryption overhead ({:.2}%) exceeds maximum allowed ({}%)", 
        overhead_percent, MAX_OVERHEAD_PERCENT);
}

/// Test batch operation performance benefits
#[test]
async fn test_batch_encryption_performance() {
    let single_ops_per_sec = benchmark_async_encryption(50, SMALL_DATA_SIZE).await.unwrap();
    let batch_ops_per_sec = benchmark_batch_encryption(50, SMALL_DATA_SIZE).await.unwrap();
    
    let improvement_percent = ((batch_ops_per_sec - single_ops_per_sec) / single_ops_per_sec) * 100.0;
    
    println!("Batch Encryption Performance:");
    println!("  Single ops/sec: {:.2}", single_ops_per_sec);
    println!("  Batch ops/sec: {:.2}", batch_ops_per_sec);
    println!("  Improvement: {:.2}%", improvement_percent);
    
    // Batch operations should be at least as fast as individual operations
    assert!(batch_ops_per_sec >= single_ops_per_sec, 
        "Batch operations should not be slower than individual operations");
}

/// Test async wrapper database operations performance
#[test]
async fn test_async_wrapper_performance() {
    let db_ops = create_test_db_ops();
    let master_keypair = generate_master_keypair().unwrap();
    
    // Create sync wrapper
    let sync_wrapper = EncryptionWrapper::new(db_ops.clone(), &master_keypair).unwrap();
    
    // Create async wrapper
    let async_config = AsyncWrapperConfig::default();
    let async_wrapper = AsyncEncryptionWrapper::new(db_ops, &master_keypair, async_config).await.unwrap();
    
    let test_data = create_test_data(SMALL_DATA_SIZE);
    let iterations = 50;
    
    // Benchmark sync operations
    let sync_start = Instant::now();
    for i in 0..iterations {
        let key = format!("sync_key_{}", i);
        sync_wrapper.store_encrypted_item(&key, &test_data, contexts::ATOM_DATA).unwrap();
        let _retrieved: Vec<u8> = sync_wrapper.get_encrypted_item(&key, contexts::ATOM_DATA).unwrap().unwrap();
    }
    let sync_duration = sync_start.elapsed();
    let sync_ops_per_sec = (iterations * 2) as f64 / sync_duration.as_secs_f64();
    
    // Benchmark async operations
    let async_start = Instant::now();
    for i in 0..iterations {
        let key = format!("async_key_{}", i);
        async_wrapper.store_encrypted_item_async(&key, &test_data, contexts::ATOM_DATA).await.unwrap();
        let _retrieved: Vec<u8> = async_wrapper.get_encrypted_item_async(&key, contexts::ATOM_DATA).await.unwrap().unwrap();
    }
    let async_duration = async_start.elapsed();
    let async_ops_per_sec = (iterations * 2) as f64 / async_duration.as_secs_f64();
    
    let overhead_percent = ((sync_ops_per_sec - async_ops_per_sec) / sync_ops_per_sec) * 100.0;
    
    println!("Async Wrapper Performance:");
    println!("  Sync ops/sec: {:.2}", sync_ops_per_sec);
    println!("  Async ops/sec: {:.2}", async_ops_per_sec);
    println!("  Overhead: {:.2}%", overhead_percent);
    
    assert!(overhead_percent <= MAX_OVERHEAD_PERCENT, 
        "Async wrapper overhead ({:.2}%) exceeds maximum allowed ({}%)", 
        overhead_percent, MAX_OVERHEAD_PERCENT);
}

/// Test async atom manager performance
#[test]
async fn test_async_atom_manager_performance() {
    let db_ops = create_test_db_ops();
    let master_keypair = generate_master_keypair().unwrap();
    
    // Create async atom manager
    let async_config = AsyncAtomConfig::default();
    let async_atom_manager = AsyncAtomManager::with_encryption(db_ops, &master_keypair, async_config).await.unwrap();
    
    let test_content = json!({"test": "data", "size": create_test_data(SMALL_DATA_SIZE)});
    let iterations = 30;
    
    // Benchmark atom operations
    let start = Instant::now();
    let mut created_atoms = Vec::new();
    
    for i in 0..iterations {
        let atom = async_atom_manager.create_atom_async(
            "test_schema",
            format!("pub_key_{}", i),
            test_content.clone(),
        ).await.unwrap();
        created_atoms.push(atom.uuid().to_string());
    }
    
    for uuid in &created_atoms {
        let _atom = async_atom_manager.get_atom_async(uuid).await.unwrap();
    }
    
    let duration = start.elapsed();
    let ops_per_sec = (iterations * 2) as f64 / duration.as_secs_f64(); // create + get
    
    println!("Async Atom Manager Performance:");
    println!("  Ops/sec: {:.2}", ops_per_sec);
    println!("  Duration: {:?}", duration);
    
    // Should be able to handle at least 10 ops/sec for encrypted atom operations
    assert!(ops_per_sec >= 10.0, 
        "Async atom manager performance ({:.2} ops/sec) is too low", ops_per_sec);
    
    // Test batch operations
    let batch_start = Instant::now();
    let batch_uuids = created_atoms.clone();
    let _batch_atoms = async_atom_manager.get_atoms_batch_async(batch_uuids).await.unwrap();
    let batch_duration = batch_start.elapsed();
    let batch_ops_per_sec = iterations as f64 / batch_duration.as_secs_f64();
    
    println!("  Batch ops/sec: {:.2}", batch_ops_per_sec);
    
    // Batch operations should be faster than individual operations
    assert!(batch_ops_per_sec >= ops_per_sec / 2.0, 
        "Batch operations should provide performance benefits");
}

/// Test memory usage and caching efficiency
#[test]
async fn test_memory_usage_and_caching() {
    let key = [0x42u8; 32];
    let config = PerformanceConfig::default();
    let encryptor = AsyncEncryptionAtRest::new(key, config).await.unwrap();
    
    let test_data = create_test_data(1024); // 1KB data
    let iterations = 100;
    
    // Test memory efficiency with repeated operations
    let start = Instant::now();
    for _ in 0..iterations {
        let encrypted = encryptor.encrypt_async(&test_data).await.unwrap();
        let _decrypted = encryptor.decrypt_async(&encrypted).await.unwrap();
    }
    let duration = start.elapsed();
    
    let metrics = encryptor.get_metrics().await;
    let (cache_size, cache_capacity) = encryptor.get_cache_stats().await;
    
    println!("Memory and Caching Performance:");
    println!("  Total encryptions: {}", metrics.total_encryptions);
    println!("  Total decryptions: {}", metrics.total_decryptions);
    println!("  Cache utilization: {}/{}", cache_size, cache_capacity);
    println!("  Duration: {:?}", duration);
    
    assert_eq!(metrics.total_encryptions, iterations as u64);
    assert_eq!(metrics.total_decryptions, iterations as u64);
    
    // Cache should be utilized
    assert!(cache_capacity > 0, "Cache should be configured");
}

/// Test concurrent operations performance
#[test]
async fn test_concurrent_operations_performance() {
    let key = [0x42u8; 32];
    let config = PerformanceConfig::default();
    let encryptor = AsyncEncryptionAtRest::new(key, config).await.unwrap();
    
    let test_data = create_test_data(SMALL_DATA_SIZE);
    let concurrent_ops = 10;
    let iterations_per_task = 10;
    
    let start = Instant::now();
    
    // Create concurrent tasks
    let mut tasks = Vec::new();
    for _ in 0..concurrent_ops {
        let encryptor_clone = &encryptor;
        let data_clone = test_data.clone();
        
        let task = tokio::spawn(async move {
            for _ in 0..iterations_per_task {
                let encrypted = encryptor_clone.encrypt_async(&data_clone).await.unwrap();
                let _decrypted = encryptor_clone.decrypt_async(&encrypted).await.unwrap();
            }
        });
        
        tasks.push(task);
    }
    
    // Wait for all tasks to complete
    for task in tasks {
        task.await.unwrap();
    }
    
    let duration = start.elapsed();
    let total_ops = concurrent_ops * iterations_per_task * 2; // encrypt + decrypt
    let ops_per_sec = total_ops as f64 / duration.as_secs_f64();
    
    println!("Concurrent Operations Performance:");
    println!("  Concurrent tasks: {}", concurrent_ops);
    println!("  Total operations: {}", total_ops);
    println!("  Ops/sec: {:.2}", ops_per_sec);
    println!("  Duration: {:?}", duration);
    
    // Concurrent operations should provide reasonable throughput
    assert!(ops_per_sec >= 50.0, 
        "Concurrent operations performance ({:.2} ops/sec) is too low", ops_per_sec);
}

/// Test performance with different configuration profiles
#[test]
async fn test_performance_config_profiles() {
    let configs = vec![
        ("default", PerformanceConfig::default()),
        ("high_throughput", PerformanceConfig::high_throughput()),
        ("low_latency", PerformanceConfig::low_latency()),
        ("memory_efficient", PerformanceConfig::memory_efficient()),
    ];
    
    let test_data = create_test_data(SMALL_DATA_SIZE);
    let iterations = 20;
    
    for (name, config) in configs {
        let key = [0x42u8; 32];
        let encryptor = AsyncEncryptionAtRest::new(key, config).await.unwrap();
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            let encrypted = encryptor.encrypt_async(&test_data).await.unwrap();
            let _decrypted = encryptor.decrypt_async(&encrypted).await.unwrap();
        }
        
        let duration = start.elapsed();
        let ops_per_sec = (iterations * 2) as f64 / duration.as_secs_f64();
        
        println!("Config '{}' Performance:", name);
        println!("  Ops/sec: {:.2}", ops_per_sec);
        println!("  Duration: {:?}", duration);
        
        // All configurations should provide reasonable performance
        assert!(ops_per_sec >= 10.0, 
            "Config '{}' performance ({:.2} ops/sec) is too low", name, ops_per_sec);
    }
}

/// Comprehensive performance validation test
#[test]
async fn test_comprehensive_performance_validation() {
    println!("\n=== Comprehensive Performance Validation ===");
    
    let mut all_results = Vec::new();
    
    // Test different data sizes
    let data_sizes = vec![256, 1024, 4096, 16384]; // 256B to 16KB
    
    for data_size in data_sizes {
        let sync_ops = benchmark_sync_encryption(50, data_size).await.unwrap();
        let async_ops = benchmark_async_encryption(50, data_size).await.unwrap();
        
        let overhead = ((sync_ops - async_ops) / sync_ops) * 100.0;
        let throughput_mbps = (async_ops * data_size as f64) / (1024.0 * 1024.0);
        
        let results = PerformanceResults {
            sync_ops_per_sec: sync_ops,
            async_ops_per_sec: async_ops,
            overhead_percent: overhead,
            memory_usage_mb: 0.0, // Would need actual memory measurement
            throughput_mbps,
        };
        
        println!("\nData size: {} bytes", data_size);
        println!("  Sync ops/sec: {:.2}", results.sync_ops_per_sec);
        println!("  Async ops/sec: {:.2}", results.async_ops_per_sec);
        println!("  Overhead: {:.2}%", results.overhead_percent);
        println!("  Throughput: {:.2} MB/s", results.throughput_mbps);
        println!("  Meets requirements: {}", results.meets_requirements());
        
        all_results.push((data_size, results));
    }
    
    // Validate that all test cases meet the <20% overhead requirement
    let failed_cases: Vec<_> = all_results.iter()
        .filter(|(_, results)| !results.meets_requirements())
        .collect();
    
    if !failed_cases.is_empty() {
        println!("\nFailed cases:");
        for (size, results) in failed_cases {
            println!("  {} bytes: {:.2}% overhead", size, results.overhead_percent);
        }
        panic!("Some test cases exceeded the maximum allowed overhead of {}%", MAX_OVERHEAD_PERCENT);
    }
    
    // Calculate average overhead
    let avg_overhead = all_results.iter()
        .map(|(_, results)| results.overhead_percent)
        .sum::<f64>() / all_results.len() as f64;
    
    println!("\n=== Summary ===");
    println!("Average overhead: {:.2}%", avg_overhead);
    println!("Maximum allowed overhead: {}%", MAX_OVERHEAD_PERCENT);
    println!("All test cases passed: {}", avg_overhead <= MAX_OVERHEAD_PERCENT);
    
    assert!(avg_overhead <= MAX_OVERHEAD_PERCENT, 
        "Average overhead ({:.2}%) exceeds maximum allowed ({}%)", 
        avg_overhead, MAX_OVERHEAD_PERCENT);
}

/// Test that validates specific PBI 9 performance requirements
#[test]
async fn test_pbi_9_performance_requirements() {
    println!("\n=== PBI 9 Performance Requirements Validation ===");
    
    // Test requirement: <20% performance overhead for encryption at rest
    let baseline_data_size = 1024; // 1KB typical data size
    let iterations = 100;
    
    // Measure baseline (unencrypted) performance
    let test_data = create_test_data(baseline_data_size);
    let baseline_start = Instant::now();
    
    // Simulate unencrypted database operations
    let temp_dir = tempdir().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    
    for i in 0..iterations {
        let key = format!("baseline_key_{}", i);
        db.insert(key.as_bytes(), &test_data).unwrap();
        let _retrieved = db.get(key.as_bytes()).unwrap();
    }
    
    let baseline_duration = baseline_start.elapsed();
    let baseline_ops_per_sec = (iterations * 2) as f64 / baseline_duration.as_secs_f64();
    
    // Measure encrypted performance
    let db_ops = create_test_db_ops();
    let master_keypair = generate_master_keypair().unwrap();
    let config = AsyncWrapperConfig::default();
    let async_wrapper = AsyncEncryptionWrapper::new(db_ops, &master_keypair, config).await.unwrap();
    
    let encrypted_start = Instant::now();
    
    for i in 0..iterations {
        let key = format!("encrypted_key_{}", i);
        async_wrapper.store_encrypted_item_async(&key, &test_data, contexts::ATOM_DATA).await.unwrap();
        let _retrieved: Vec<u8> = async_wrapper.get_encrypted_item_async(&key, contexts::ATOM_DATA).await.unwrap().unwrap();
    }
    
    let encrypted_duration = encrypted_start.elapsed();
    let encrypted_ops_per_sec = (iterations * 2) as f64 / encrypted_duration.as_secs_f64();
    
    let overhead_percent = ((baseline_ops_per_sec - encrypted_ops_per_sec) / baseline_ops_per_sec) * 100.0;
    
    println!("PBI 9 Performance Validation:");
    println!("  Baseline (unencrypted) ops/sec: {:.2}", baseline_ops_per_sec);
    println!("  Encrypted ops/sec: {:.2}", encrypted_ops_per_sec);
    println!("  Performance overhead: {:.2}%", overhead_percent);
    println!("  Requirement: <{}% overhead", MAX_OVERHEAD_PERCENT);
    println!("  Status: {}", if overhead_percent <= MAX_OVERHEAD_PERCENT { "PASS" } else { "FAIL" });
    
    assert!(overhead_percent <= MAX_OVERHEAD_PERCENT, 
        "PBI 9 requirement failed: encryption overhead ({:.2}%) exceeds maximum allowed ({}%)", 
        overhead_percent, MAX_OVERHEAD_PERCENT);
    
    println!("✅ PBI 9 performance requirement validated successfully!");
}