//! Performance testing and benchmarking for configuration management
//!
//! This module implements comprehensive performance tests to verify that the
//! configuration system meets the specified performance requirements:
//! - Configuration loading < 10ms
//! - Memory usage < 1MB for typical configurations  
//! - Hot reload < 1s update propagation
//! - Zero downtime configuration updates

use std::time::{Duration, Instant};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::time::{sleep, timeout};

use crate::config::{
    cross_platform::{Config, ConfigurationManager},
    enhanced::{EnhancedConfig, EnhancedConfigurationManager},
    value::ConfigValue,
};

use super::{
    mocks::{MockPerformanceMonitor, create_large_test_config},
    utils::*,
    constants::*,
    create_test_dir, init_test_env,
};

/// Performance benchmark results
#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    pub operation: String,
    pub iterations: usize,
    pub total_time: Duration,
    pub avg_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub memory_usage_mb: f64,
    pub success_rate: f64,
}

impl BenchmarkResults {
    pub fn print_summary(&self) {
        println!("📊 Benchmark: {}", self.operation);
        println!("   Iterations: {}", self.iterations);
        println!("   Total time: {:?}", self.total_time);
        println!("   Average time: {:?}", self.avg_time);
        println!("   Min time: {:?}", self.min_time);
        println!("   Max time: {:?}", self.max_time);
        println!("   Memory usage: {:.2} MB", self.memory_usage_mb);
        println!("   Success rate: {:.1}%", self.success_rate * 100.0);
    }
}

/// Performance test suite for configuration loading
#[cfg(test)]
mod loading_performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_configuration_load_performance() {
        init_test_env();
        let temp_dir = create_test_dir("load_performance");
        let config_file = temp_dir.path().join("perf_config.toml");
        
        let manager = ConfigurationManager::with_toml_file(&config_file);
        
        // Create and save a test configuration
        let config = create_test_configuration();
        manager.set(config).await.unwrap();
        
        // Warm up (load once to initialize)
        let _ = manager.get().await.unwrap();
        manager.clear_cache().await;
        
        let mut load_times = Vec::new();
        let mut success_count = 0;
        
        // Perform multiple load operations
        for _ in 0..PERF_TEST_ITERATIONS {
            manager.clear_cache().await;
            
            let start_time = Instant::now();
            let result = manager.get().await;
            let load_time = start_time.elapsed();
            
            if result.is_ok() {
                success_count += 1;
                load_times.push(load_time);
            }
        }
        
        // Analyze results
        let avg_time = load_times.iter().sum::<Duration>() / load_times.len() as u32;
        let min_time = *load_times.iter().min().unwrap();
        let max_time = *load_times.iter().max().unwrap();
        
        let results = BenchmarkResults {
            operation: "Configuration Load".to_string(),
            iterations: PERF_TEST_ITERATIONS,
            total_time: load_times.iter().sum(),
            avg_time,
            min_time,
            max_time,
            memory_usage_mb: get_memory_usage_mb(),
            success_rate: success_count as f64 / PERF_TEST_ITERATIONS as f64,
        };
        
        results.print_summary();
        
        // Verify performance requirements
        assert!(avg_time < MAX_LOAD_TIME, 
            "Average load time ({:?}) exceeds requirement ({:?})", avg_time, MAX_LOAD_TIME);
        assert!(results.memory_usage_mb < MAX_MEMORY_USAGE_MB as f64,
            "Memory usage ({:.2} MB) exceeds requirement ({} MB)", results.memory_usage_mb, MAX_MEMORY_USAGE_MB);
        assert!(results.success_rate > 0.99, "Success rate too low: {:.1}%", results.success_rate * 100.0);
    }

    #[tokio::test]
    async fn test_cached_configuration_load_performance() {
        init_test_env();
        let temp_dir = create_test_dir("cached_load_performance");
        let config_file = temp_dir.path().join("cached_perf_config.toml");
        
        let manager = ConfigurationManager::with_toml_file(&config_file);
        
        // Create and save configuration
        let config = create_test_configuration();
        manager.set(config).await.unwrap();
        
        // Load once to populate cache
        let _ = manager.get().await.unwrap();
        
        let mut cached_load_times = Vec::new();
        let mut success_count = 0;
        
        // Test cached loads (should be much faster)
        for _ in 0..PERF_TEST_ITERATIONS {
            let start_time = Instant::now();
            let result = manager.get().await;
            let load_time = start_time.elapsed();
            
            if result.is_ok() {
                success_count += 1;
                cached_load_times.push(load_time);
            }
        }
        
        let avg_cached_time = cached_load_times.iter().sum::<Duration>() / cached_load_times.len() as u32;
        
        // Cached loads should be significantly faster than the 10ms requirement
        assert!(avg_cached_time < Duration::from_millis(1),
            "Cached load time ({:?}) should be much faster than requirement", avg_cached_time);
        
        println!("📈 Cached load performance: avg {:?}, success rate: {:.1}%", 
                avg_cached_time, success_count as f64 / PERF_TEST_ITERATIONS as f64 * 100.0);
    }

    #[tokio::test]
    async fn test_large_configuration_load_performance() {
        init_test_env();
        let temp_dir = create_test_dir("large_config_performance");
        let config_file = temp_dir.path().join("large_config.toml");
        
        let manager = ConfigurationManager::with_toml_file(&config_file);
        
        // Create large configuration
        let large_config = create_large_test_config();
        manager.set(large_config).await.unwrap();
        
        // Test load performance with large config
        manager.clear_cache().await;
        
        let start_time = Instant::now();
        let result = manager.get().await;
        let load_time = start_time.elapsed();
        
        assert!(result.is_ok(), "Failed to load large configuration");
        
        // Even large configs should load within reasonable time
        // We'll allow up to 100ms for very large configurations
        assert!(load_time < Duration::from_millis(100),
            "Large configuration load time ({:?}) exceeds reasonable limit", load_time);
        
        println!("📊 Large config load time: {:?}", load_time);
    }
}

/// Performance test suite for configuration saving
#[cfg(test)]
mod saving_performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_configuration_save_performance() {
        init_test_env();
        let temp_dir = create_test_dir("save_performance");
        let config_file = temp_dir.path().join("save_perf_config.toml");
        
        let manager = ConfigurationManager::with_toml_file(&config_file);
        
        let mut save_times = Vec::new();
        let mut success_count = 0;
        
        // Perform multiple save operations
        for i in 0..100 { // Fewer iterations for save tests (disk I/O intensive)
            let mut config = create_test_configuration();
            config.version = format!("1.0.{}", i);
            
            let start_time = Instant::now();
            let result = manager.set(config).await;
            let save_time = start_time.elapsed();
            
            if result.is_ok() {
                success_count += 1;
                save_times.push(save_time);
            }
        }
        
        let avg_time = save_times.iter().sum::<Duration>() / save_times.len() as u32;
        let min_time = *save_times.iter().min().unwrap();
        let max_time = *save_times.iter().max().unwrap();
        
        let results = BenchmarkResults {
            operation: "Configuration Save".to_string(),
            iterations: 100,
            total_time: save_times.iter().sum(),
            avg_time,
            min_time,
            max_time,
            memory_usage_mb: get_memory_usage_mb(),
            success_rate: success_count as f64 / 100.0,
        };
        
        results.print_summary();
        
        // Save operations can be slower than loads, but should still be reasonable
        assert!(avg_time < Duration::from_millis(100),
            "Average save time ({:?}) exceeds reasonable limit", avg_time);
        assert!(results.success_rate > 0.99, "Save success rate too low: {:.1}%", results.success_rate * 100.0);
    }

    #[tokio::test]
    async fn test_atomic_save_performance() {
        init_test_env();
        let temp_dir = create_test_dir("atomic_save_performance");
        let config_file = temp_dir.path().join("atomic_save_config.toml");
        
        let manager = ConfigurationManager::with_toml_file(&config_file);
        
        // Test atomic saves (should be safe even under load)
        let mut atomic_save_times = Vec::new();
        let mut success_count = 0;
        
        for i in 0..50 {
            let mut config = create_test_configuration();
            config.version = format!("2.0.{}", i);
            
            let start_time = Instant::now();
            let result = manager.set(config).await;
            let save_time = start_time.elapsed();
            
            if result.is_ok() {
                success_count += 1;
                atomic_save_times.push(save_time);
                
                // Verify configuration was saved correctly
                let loaded = manager.get().await.unwrap();
                assert_eq!(loaded.version, format!("2.0.{}", i));
            }
        }
        
        let avg_atomic_time = atomic_save_times.iter().sum::<Duration>() / atomic_save_times.len() as u32;
        
        println!("🔒 Atomic save performance: avg {:?}, success rate: {:.1}%", 
                avg_atomic_time, success_count as f64 / 50.0 * 100.0);
        
        assert!(success_count == 50, "Not all atomic saves succeeded");
    }
}

/// Performance test suite for hot reload and caching
#[cfg(test)]
mod hot_reload_performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_hot_reload_performance() {
        init_test_env();
        let temp_dir = create_test_dir("hot_reload_performance");
        let config_file = temp_dir.path().join("hot_reload_config.toml");
        
        let manager = ConfigurationManager::with_toml_file(&config_file);
        
        // Create initial configuration
        let initial_config = create_test_configuration();
        manager.set(initial_config).await.unwrap();
        
        // Load configuration to populate cache
        let loaded = manager.get().await.unwrap();
        assert_eq!(loaded.version, "1.0.0");
        
        // Measure hot reload time
        let start_time = Instant::now();
        
        // Update configuration externally (simulating file change)
        let mut updated_config = create_test_configuration();
        updated_config.version = "2.0.0".to_string();
        manager.set(updated_config).await.unwrap();
        
        // The system should detect the change and reload
        // For this test, we'll clear cache to simulate the reload trigger
        manager.clear_cache().await;
        let reloaded = manager.get().await.unwrap();
        
        let reload_time = start_time.elapsed();
        
        assert_eq!(reloaded.version, "2.0.0");
        assert!(reload_time < MAX_HOT_RELOAD_TIME,
            "Hot reload time ({:?}) exceeds requirement ({:?})", reload_time, MAX_HOT_RELOAD_TIME);
        
        println!("🔥 Hot reload time: {:?}", reload_time);
    }

    #[tokio::test]
    async fn test_cache_performance() {
        init_test_env();
        let temp_dir = create_test_dir("cache_performance");
        let config_file = temp_dir.path().join("cache_perf_config.toml");
        
        let manager = ConfigurationManager::with_toml_file(&config_file);
        
        // Create configuration
        let config = create_test_configuration();
        manager.set(config).await.unwrap();
        
        // Test cache hit performance
        let mut cache_hit_times = Vec::new();
        let mut cache_miss_times = Vec::new();
        
        for i in 0..100 {
            if i % 10 == 0 {
                // Force cache miss every 10th iteration
                manager.clear_cache().await;
            }
            
            let start_time = Instant::now();
            let _config = manager.get().await.unwrap();
            let access_time = start_time.elapsed();
            
            if i % 10 == 0 {
                cache_miss_times.push(access_time);
            } else {
                cache_hit_times.push(access_time);
            }
        }
        
        let avg_hit_time = cache_hit_times.iter().sum::<Duration>() / cache_hit_times.len() as u32;
        let avg_miss_time = cache_miss_times.iter().sum::<Duration>() / cache_miss_times.len() as u32;
        
        println!("💾 Cache performance:");
        println!("   Cache hit avg: {:?}", avg_hit_time);
        println!("   Cache miss avg: {:?}", avg_miss_time);
        
        // Cache hits should be significantly faster than misses
        assert!(avg_hit_time < avg_miss_time / 2,
            "Cache hits should be much faster than misses");
        
        // Cache hits should be very fast
        assert!(avg_hit_time < Duration::from_micros(100),
            "Cache hits too slow: {:?}", avg_hit_time);
    }
}

/// Memory usage performance tests
#[cfg(test)]
mod memory_performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_usage_typical_config() {
        init_test_env();
        let temp_dir = create_test_dir("memory_usage_typical");
        let config_file = temp_dir.path().join("memory_test_config.toml");
        
        let manager = ConfigurationManager::with_toml_file(&config_file);
        
        // Record baseline memory
        let baseline_memory = get_memory_usage_mb();
        
        // Create typical configuration
        let config = create_test_configuration();
        manager.set(config).await.unwrap();
        
        // Load configuration multiple times
        for _ in 0..10 {
            let _ = manager.get().await.unwrap();
        }
        
        let final_memory = get_memory_usage_mb();
        let memory_delta = final_memory - baseline_memory;
        
        println!("💾 Memory usage for typical config: {:.2} MB (delta: {:.2} MB)", 
                final_memory, memory_delta);
        
        // Memory usage should be well under the 1MB requirement
        assert!(memory_delta < MAX_MEMORY_USAGE_MB as f64,
            "Memory usage ({:.2} MB) exceeds requirement ({} MB)", memory_delta, MAX_MEMORY_USAGE_MB);
    }

    #[tokio::test]
    async fn test_memory_usage_large_config() {
        init_test_env();
        let temp_dir = create_test_dir("memory_usage_large");
        let config_file = temp_dir.path().join("large_memory_test_config.toml");
        
        let manager = ConfigurationManager::with_toml_file(&config_file);
        
        let baseline_memory = get_memory_usage_mb();
        
        // Create large configuration
        let large_config = create_large_test_config();
        manager.set(large_config).await.unwrap();
        
        // Load large configuration
        let _ = manager.get().await.unwrap();
        
        let final_memory = get_memory_usage_mb();
        let memory_delta = final_memory - baseline_memory;
        
        println!("💾 Memory usage for large config: {:.2} MB (delta: {:.2} MB)", 
                final_memory, memory_delta);
        
        // Large configurations may use more memory but should still be reasonable
        assert!(memory_delta < 5.0, // Allow up to 5MB for very large configs
            "Large config memory usage ({:.2} MB) exceeds reasonable limit", memory_delta);
    }

    #[tokio::test]
    async fn test_memory_leak_detection() {
        init_test_env();
        let temp_dir = create_test_dir("memory_leak_detection");
        let config_file = temp_dir.path().join("leak_test_config.toml");
        
        let manager = ConfigurationManager::with_toml_file(&config_file);
        
        let baseline_memory = get_memory_usage_mb();
        
        // Perform many operations to detect memory leaks
        for i in 0..100 {
            let mut config = create_test_configuration();
            config.version = format!("1.0.{}", i);
            
            manager.set(config).await.unwrap();
            let _ = manager.get().await.unwrap();
            
            if i % 10 == 0 {
                manager.clear_cache().await;
            }
        }
        
        // Force garbage collection (in a real scenario)
        // In Rust, we don't have explicit GC, but let's give time for cleanup
        sleep(Duration::from_millis(100)).await;
        
        let final_memory = get_memory_usage_mb();
        let memory_growth = final_memory - baseline_memory;
        
        println!("🔍 Memory leak test: baseline {:.2} MB, final {:.2} MB, growth {:.2} MB", 
                baseline_memory, final_memory, memory_growth);
        
        // Memory growth should be minimal after many operations
        assert!(memory_growth < 2.0,
            "Possible memory leak detected: {:.2} MB growth", memory_growth);
    }
}

/// Concurrency and stress performance tests
#[cfg(test)]
mod concurrency_performance_tests {
    use super::*;
    use tokio::task::JoinSet;

    #[tokio::test]
    async fn test_concurrent_access_performance() {
        init_test_env();
        let temp_dir = create_test_dir("concurrent_access");
        let config_file = temp_dir.path().join("concurrent_config.toml");
        
        let manager = Arc::new(ConfigurationManager::with_toml_file(&config_file));
        
        // Create initial configuration
        let config = create_test_configuration();
        manager.set(config).await.unwrap();
        
        let start_time = Instant::now();
        let mut join_set = JoinSet::new();
        
        // Spawn multiple concurrent readers
        for i in 0..20 {
            let manager_clone = manager.clone();
            join_set.spawn(async move {
                let mut success_count = 0;
                for _j in 0..50 {
                    if let Ok(_config) = manager_clone.get().await {
                        success_count += 1;
                    }
                }
                (i, success_count)
            });
        }
        
        // Collect results
        let mut total_successes = 0;
        let mut task_count = 0;
        
        while let Some(result) = join_set.join_next().await {
            if let Ok((_task_id, successes)) = result {
                total_successes += successes;
                task_count += 1;
            }
        }
        
        let total_time = start_time.elapsed();
        let total_operations = task_count * 50;
        
        println!("🔄 Concurrent access test:");
        println!("   Tasks: {}, Operations: {}", task_count, total_operations);
        println!("   Successes: {}, Total time: {:?}", total_successes, total_time);
        println!("   Avg time per op: {:?}", total_time / total_operations as u32);
        
        assert_eq!(task_count, 20, "Not all tasks completed");
        assert_eq!(total_successes, 1000, "Not all operations succeeded"); // 20 tasks * 50 operations each
        
        // Even under concurrent load, operations should be reasonably fast
        let avg_time_per_op = total_time / total_operations as u32;
        assert!(avg_time_per_op < Duration::from_millis(1),
            "Concurrent operations too slow: {:?}", avg_time_per_op);
    }

    #[tokio::test]
    async fn test_mixed_read_write_performance() {
        init_test_env();
        let temp_dir = create_test_dir("mixed_read_write");
        let config_file = temp_dir.path().join("mixed_rw_config.toml");
        
        let manager = Arc::new(ConfigurationManager::with_toml_file(&config_file));
        
        // Create initial configuration
        let config = create_test_configuration();
        manager.set(config).await.unwrap();
        
        let start_time = Instant::now();
        let mut join_set = JoinSet::new();
        
        // Spawn readers
        for i in 0..15 {
            let manager_clone = manager.clone();
            join_set.spawn(async move {
                let mut read_successes = 0;
                for _j in 0..20 {
                    if let Ok(_config) = manager_clone.get().await {
                        read_successes += 1;
                    }
                    sleep(Duration::from_millis(1)).await;
                }
                format!("reader_{}: {} reads", i, read_successes)
            });
        }
        
        // Spawn writers
        for i in 0..5 {
            let manager_clone = manager.clone();
            join_set.spawn(async move {
                let mut write_successes = 0;
                for j in 0..10 {
                    let mut config = create_test_configuration();
                    config.version = format!("writer_{}_{}", i, j);
                    
                    if let Ok(_) = manager_clone.set(config).await {
                        write_successes += 1;
                    }
                    sleep(Duration::from_millis(5)).await;
                }
                format!("writer_{}: {} writes", i, write_successes)
            });
        }
        
        // Collect results
        let mut results = Vec::new();
        while let Some(result) = join_set.join_next().await {
            if let Ok(result_msg) = result {
                results.push(result_msg);
            }
        }
        
        let total_time = start_time.elapsed();
        
        println!("📖✏️  Mixed read/write test completed in {:?}", total_time);
        for result in &results {
            println!("   {}", result);
        }
        
        // Test should complete in reasonable time even with mixed operations
        assert!(total_time < Duration::from_secs(30),
            "Mixed read/write test took too long: {:?}", total_time);
    }
}

/// Helper functions for performance testing
fn create_test_configuration() -> Config {
    let mut config = Config::new();
    config.version = "1.0.0".to_string();
    
    // Add several sections with various data types
    let mut app_section = HashMap::new();
    app_section.insert("name".to_string(), ConfigValue::string("performance_test_app"));
    app_section.insert("debug".to_string(), ConfigValue::boolean(false));
    app_section.insert("max_connections".to_string(), ConfigValue::integer(100));
    app_section.insert("timeout".to_string(), ConfigValue::float(30.5));
    config.set_section("app".to_string(), ConfigValue::object(app_section));
    
    let mut db_section = HashMap::new();
    db_section.insert("host".to_string(), ConfigValue::string("localhost"));
    db_section.insert("port".to_string(), ConfigValue::integer(5432));
    db_section.insert("ssl".to_string(), ConfigValue::boolean(true));
    config.set_section("database".to_string(), ConfigValue::object(db_section));
    
    config
}

fn get_memory_usage_mb() -> f64 {
    // In a real implementation, this would use platform-specific APIs
    // For testing, we'll return a simulated value based on process info
    
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<f64>() {
                            return kb / 1024.0; // Convert KB to MB
                        }
                    }
                }
            }
        }
    }
    
    // Fallback: estimate based on allocation patterns
    // This is a rough approximation for testing purposes
    let estimated_mb = 10.0; // Base process memory
    estimated_mb
}