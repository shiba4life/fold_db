//! Mock implementations for testing cross-platform functionality
//!
//! This module provides mock implementations of platform-specific components
//! to enable comprehensive testing in CI/CD environments and across platforms.

use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::config::error::{ConfigError, ConfigResult};
use crate::config::platform::{
    PlatformConfigPaths, PlatformFileWatcher, PlatformAtomicOps,
    keystore::PlatformKeystore,
};

/// Mock platform configuration paths for testing
#[derive(Debug, Clone)]
pub struct MockPlatformPaths {
    base_dir: PathBuf,
    platform_name: String,
}

impl MockPlatformPaths {
    pub fn new(base_dir: PathBuf, platform_name: &str) -> Self {
        Self {
            base_dir,
            platform_name: platform_name.to_string(),
        }
    }
}

impl PlatformConfigPaths for MockPlatformPaths {
    fn config_dir(&self) -> ConfigResult<PathBuf> {
        Ok(self.base_dir.join("config"))
    }

    fn data_dir(&self) -> ConfigResult<PathBuf> {
        Ok(self.base_dir.join("data"))
    }

    fn cache_dir(&self) -> ConfigResult<PathBuf> {
        Ok(self.base_dir.join("cache"))
    }

    fn logs_dir(&self) -> ConfigResult<PathBuf> {
        Ok(self.base_dir.join("logs"))
    }

    fn runtime_dir(&self) -> ConfigResult<PathBuf> {
        Ok(self.base_dir.join("runtime"))
    }

    fn platform_name(&self) -> &'static str {
        // Need to return static str, so we'll use a match
        match self.platform_name.as_str() {
            "linux" => "linux",
            "macos" => "macos", 
            "windows" => "windows",
            _ => "mock",
        }
    }
}

/// Mock keystore for testing secure storage functionality
#[derive(Debug, Clone)]
pub struct MockKeystore {
    storage: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    available: bool,
    should_fail: Arc<Mutex<bool>>,
}

impl MockKeystore {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
            available: true,
            should_fail: Arc::new(Mutex::new(false)),
        }
    }

    pub fn new_unavailable() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
            available: false,
            should_fail: Arc::new(Mutex::new(false)),
        }
    }

    pub fn set_should_fail(&self, should_fail: bool) {
        *self.should_fail.lock().unwrap() = should_fail;
    }

    pub fn get_stored_keys(&self) -> Vec<String> {
        self.storage.lock().unwrap().keys().cloned().collect()
    }
}

#[async_trait]
impl PlatformKeystore for MockKeystore {
    async fn store_secret(&self, key: &str, value: &[u8]) -> ConfigResult<()> {
        if *self.should_fail.lock().unwrap() {
            return Err(ConfigError::access_denied("Mock keystore failure"));
        }
        
        self.storage.lock().unwrap().insert(key.to_string(), value.to_vec());
        Ok(())
    }

    async fn get_secret(&self, key: &str) -> ConfigResult<Option<Vec<u8>>> {
        if *self.should_fail.lock().unwrap() {
            return Err(ConfigError::access_denied("Mock keystore failure"));
        }
        
        Ok(self.storage.lock().unwrap().get(key).cloned())
    }

    async fn delete_secret(&self, key: &str) -> ConfigResult<()> {
        if *self.should_fail.lock().unwrap() {
            return Err(ConfigError::access_denied("Mock keystore failure"));
        }
        
        self.storage.lock().unwrap().remove(key);
        Ok(())
    }

    async fn list_keys(&self) -> ConfigResult<Vec<String>> {
        if *self.should_fail.lock().unwrap() {
            return Err(ConfigError::access_denied("Mock keystore failure"));
        }
        
        Ok(self.storage.lock().unwrap().keys().cloned().collect())
    }

    fn is_available(&self) -> bool {
        self.available
    }

    fn keystore_type(&self) -> &'static str {
        "mock"
    }
}

/// Mock file watcher for testing file change notifications
#[derive(Debug)]
pub struct MockFileWatcher {
    callbacks: Arc<Mutex<Vec<Box<dyn Fn() + Send + 'static>>>>,
}

impl MockFileWatcher {
    pub fn new() -> Self {
        Self {
            callbacks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn trigger_change(&self) {
        let callbacks = self.callbacks.lock().unwrap();
        for callback in callbacks.iter() {
            callback();
        }
    }

    pub fn callback_count(&self) -> usize {
        self.callbacks.lock().unwrap().len()
    }
}

impl PlatformFileWatcher for MockFileWatcher {
    fn watch_file<F>(&self, _path: &std::path::Path, callback: F) -> ConfigResult<()>
    where
        F: Fn() + Send + 'static,
    {
        self.callbacks.lock().unwrap().push(Box::new(callback));
        Ok(())
    }
}

/// Mock atomic operations for testing file operations
#[derive(Debug)]
pub struct MockAtomicOps {
    should_fail: Arc<Mutex<bool>>,
    operations_log: Arc<Mutex<Vec<String>>>,
}

impl MockAtomicOps {
    pub fn new() -> Self {
        Self {
            should_fail: Arc::new(Mutex::new(false)),
            operations_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn set_should_fail(&self, should_fail: bool) {
        *self.should_fail.lock().unwrap() = should_fail;
    }

    pub fn get_operations_log(&self) -> Vec<String> {
        self.operations_log.lock().unwrap().clone()
    }

    pub fn clear_operations_log(&self) {
        self.operations_log.lock().unwrap().clear();
    }
}

impl PlatformAtomicOps for MockAtomicOps {
    fn atomic_write(&self, path: &std::path::Path, content: &[u8]) -> ConfigResult<()> {
        if *self.should_fail.lock().unwrap() {
            return Err(ConfigError::io_error("Mock atomic write failure"));
        }

        self.operations_log.lock().unwrap().push(format!(
            "atomic_write: {} ({} bytes)", 
            path.display(), 
            content.len()
        ));

        std::fs::write(path, content)
            .map_err(|e| ConfigError::io_error(format!("Write failed: {}", e)))
    }

    fn create_with_lock(&self, path: &std::path::Path, content: &[u8]) -> ConfigResult<()> {
        if *self.should_fail.lock().unwrap() {
            return Err(ConfigError::io_error("Mock create with lock failure"));
        }

        self.operations_log.lock().unwrap().push(format!(
            "create_with_lock: {} ({} bytes)", 
            path.display(), 
            content.len()
        ));

        self.atomic_write(path, content)
    }
}

/// Mock performance monitor for testing performance tracking
#[derive(Debug, Clone)]
pub struct MockPerformanceMonitor {
    metrics: Arc<Mutex<PerformanceMetrics>>,
}

#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    pub load_times: Vec<u64>,      // in microseconds
    pub save_times: Vec<u64>,      // in microseconds
    pub memory_usage: Vec<usize>,  // in bytes
    pub cache_hits: usize,
    pub cache_misses: usize,
}

impl MockPerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(PerformanceMetrics::default())),
        }
    }

    pub fn record_load_time(&self, time_us: u64) {
        self.metrics.lock().unwrap().load_times.push(time_us);
    }

    pub fn record_save_time(&self, time_us: u64) {
        self.metrics.lock().unwrap().save_times.push(time_us);
    }

    pub fn record_memory_usage(&self, bytes: usize) {
        self.metrics.lock().unwrap().memory_usage.push(bytes);
    }

    pub fn record_cache_hit(&self) {
        self.metrics.lock().unwrap().cache_hits += 1;
    }

    pub fn record_cache_miss(&self) {
        self.metrics.lock().unwrap().cache_misses += 1;
    }

    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.lock().unwrap().clone()
    }

    pub fn reset_metrics(&self) {
        *self.metrics.lock().unwrap() = PerformanceMetrics::default();
    }

    pub fn average_load_time_us(&self) -> Option<u64> {
        let metrics = self.metrics.lock().unwrap();
        if metrics.load_times.is_empty() {
            None
        } else {
            Some(metrics.load_times.iter().sum::<u64>() / metrics.load_times.len() as u64)
        }
    }

    pub fn max_memory_usage(&self) -> Option<usize> {
        let metrics = self.metrics.lock().unwrap();
        metrics.memory_usage.iter().max().copied()
    }
}

/// Test condition simulator for error handling tests
#[derive(Debug, Clone)]
pub struct TestConditionSimulator {
    conditions: Arc<Mutex<HashMap<String, bool>>>,
}

impl TestConditionSimulator {
    pub fn new() -> Self {
        Self {
            conditions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set_condition(&self, name: &str, active: bool) {
        self.conditions.lock().unwrap().insert(name.to_string(), active);
    }

    pub fn is_condition_active(&self, name: &str) -> bool {
        self.conditions.lock().unwrap().get(name).copied().unwrap_or(false)
    }

    pub fn simulate_disk_full(&self) -> bool {
        self.is_condition_active("disk_full")
    }

    pub fn simulate_permission_denied(&self) -> bool {
        self.is_condition_active("permission_denied")
    }

    pub fn simulate_network_error(&self) -> bool {
        self.is_condition_active("network_error")
    }

    pub fn simulate_corruption(&self) -> bool {
        self.is_condition_active("corruption")
    }

    pub fn simulate_keystore_unavailable(&self) -> bool {
        self.is_condition_active("keystore_unavailable")
    }
}

/// Helper to create consistent test platform info
pub fn create_test_platform_info(platform: &str) -> crate::config::platform::PlatformInfo {
    crate::config::platform::PlatformInfo {
        name: platform.to_string(),
        version: "test-1.0".to_string(),
        arch: "x86_64".to_string(),
        supports_xdg: platform == "linux",
        supports_keyring: true,
        supports_file_watching: true,
    }
}

/// Helper to create test configurations with known characteristics
pub fn create_large_test_config() -> crate::config::cross_platform::Config {
    use crate::config::value::ConfigValue;
    use std::collections::HashMap;

    let mut config = crate::config::cross_platform::Config::new();
    
    // Add multiple large sections to test memory usage
    for i in 0..super::constants::LARGE_CONFIG_SECTIONS {
        let mut section = HashMap::new();
        
        // Add various types of data
        section.insert("string_value".to_string(), 
            ConfigValue::string(&format!("test_string_value_{}", i)));
        section.insert("integer_value".to_string(), 
            ConfigValue::integer(i as i64));
        section.insert("boolean_value".to_string(), 
            ConfigValue::boolean(i % 2 == 0));
        section.insert("float_value".to_string(), 
            ConfigValue::float(i as f64 * 3.14159));
        
        // Add nested arrays
        let array_data = (0..10).map(|j| 
            ConfigValue::string(&format!("array_item_{}_{}", i, j))
        ).collect();
        section.insert("array_value".to_string(), ConfigValue::array(array_data));
        
        config.set_section(format!("section_{}", i), ConfigValue::object(section));
    }
    
    config
}