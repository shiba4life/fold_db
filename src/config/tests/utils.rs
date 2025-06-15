//! Testing utilities and helper functions
//!
//! This module provides common utilities and helper functions used across
//! all test modules in the configuration management test suite.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use crate::config::{
    cross_platform::{Config, ConfigurationManager},
    enhanced::{EnhancedConfig, EnhancedConfigurationManager},
    value::ConfigValue,
    error::{ConfigError, ConfigResult},
    platform::{get_platform_info, PlatformInfo},
};

use super::constants::*;

/// Test execution context
#[derive(Debug, Clone)]
pub struct TestContext {
    pub test_name: String,
    pub start_time: std::time::Instant,
    pub platform_info: PlatformInfo,
    pub temp_dir: Option<PathBuf>,
}

impl TestContext {
    pub fn new(test_name: &str) -> Self {
        Self {
            test_name: test_name.to_string(),
            start_time: std::time::Instant::now(),
            platform_info: get_platform_info(),
            temp_dir: None,
        }
    }

    pub fn with_temp_dir(mut self, temp_dir: PathBuf) -> Self {
        self.temp_dir = Some(temp_dir);
        self
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn is_platform(&self, platform: &str) -> bool {
        self.platform_info.name == platform
    }

    pub fn supports_keyring(&self) -> bool {
        self.platform_info.supports_keyring
    }

    pub fn supports_xdg(&self) -> bool {
        self.platform_info.supports_xdg
    }
}

/// Configuration test builder for creating test configurations
#[derive(Debug, Default)]
pub struct ConfigTestBuilder {
    config: Config,
    sections: HashMap<String, HashMap<String, ConfigValue>>,
}

impl ConfigTestBuilder {
    pub fn new() -> Self {
        Self {
            config: Config::new(),
            sections: HashMap::new(),
        }
    }

    pub fn version(mut self, version: &str) -> Self {
        self.config.version = version.to_string();
        self
    }

    pub fn add_section(mut self, name: &str) -> SectionBuilder {
        SectionBuilder::new(self, name)
    }

    pub fn add_string(mut self, section: &str, key: &str, value: &str) -> Self {
        self.sections.entry(section.to_string())
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), ConfigValue::string(value));
        self
    }

    pub fn add_integer(mut self, section: &str, key: &str, value: i64) -> Self {
        self.sections.entry(section.to_string())
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), ConfigValue::integer(value));
        self
    }

    pub fn add_boolean(mut self, section: &str, key: &str, value: bool) -> Self {
        self.sections.entry(section.to_string())
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), ConfigValue::boolean(value));
        self
    }

    pub fn add_float(mut self, section: &str, key: &str, value: f64) -> Self {
        self.sections.entry(section.to_string())
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), ConfigValue::float(value));
        self
    }

    pub fn add_array(mut self, section: &str, key: &str, values: Vec<ConfigValue>) -> Self {
        self.sections.entry(section.to_string())
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), ConfigValue::array(values));
        self
    }

    pub fn build(mut self) -> Config {
        // Add all sections to the config
        for (section_name, section_data) in self.sections {
            self.config.set_section(section_name, ConfigValue::object(section_data));
        }
        self.config
    }
}

/// Section builder for fluent section creation
pub struct SectionBuilder {
    builder: ConfigTestBuilder,
    section_name: String,
    section_data: HashMap<String, ConfigValue>,
}

impl SectionBuilder {
    fn new(builder: ConfigTestBuilder, section_name: &str) -> Self {
        Self {
            builder,
            section_name: section_name.to_string(),
            section_data: HashMap::new(),
        }
    }

    pub fn string(mut self, key: &str, value: &str) -> Self {
        self.section_data.insert(key.to_string(), ConfigValue::string(value));
        self
    }

    pub fn integer(mut self, key: &str, value: i64) -> Self {
        self.section_data.insert(key.to_string(), ConfigValue::integer(value));
        self
    }

    pub fn boolean(mut self, key: &str, value: bool) -> Self {
        self.section_data.insert(key.to_string(), ConfigValue::boolean(value));
        self
    }

    pub fn float(mut self, key: &str, value: f64) -> Self {
        self.section_data.insert(key.to_string(), ConfigValue::float(value));
        self
    }

    pub fn array(mut self, key: &str, values: Vec<ConfigValue>) -> Self {
        self.section_data.insert(key.to_string(), ConfigValue::array(values));
        self
    }

    pub fn nested_object(mut self, key: &str, object: HashMap<String, ConfigValue>) -> Self {
        self.section_data.insert(key.to_string(), ConfigValue::object(object));
        self
    }

    pub fn finish_section(mut self) -> ConfigTestBuilder {
        self.builder.sections.insert(self.section_name, self.section_data);
        self.builder
    }
}

/// Performance measurement utilities
#[derive(Debug, Clone)]
pub struct PerformanceMeasurement {
    pub operation: String,
    pub duration: Duration,
    pub memory_usage_bytes: Option<usize>,
    pub success: bool,
}

impl PerformanceMeasurement {
    pub fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            duration: Duration::from_nanos(0),
            memory_usage_bytes: None,
            success: false,
        }
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn with_memory_usage(mut self, bytes: usize) -> Self {
        self.memory_usage_bytes = Some(bytes);
        self
    }

    pub fn with_success(mut self, success: bool) -> Self {
        self.success = success;
        self
    }

    pub fn meets_performance_requirements(&self) -> bool {
        match self.operation.as_str() {
            "load" => self.duration < MAX_LOAD_TIME,
            "save" => self.duration < Duration::from_millis(100), // Reasonable save time
            "hot_reload" => self.duration < MAX_HOT_RELOAD_TIME,
            _ => true, // Unknown operation, assume OK
        }
    }

    pub fn meets_memory_requirements(&self) -> bool {
        if let Some(memory_bytes) = self.memory_usage_bytes {
            memory_bytes < (MAX_MEMORY_USAGE_MB * 1024 * 1024)
        } else {
            true // No memory measurement, assume OK
        }
    }
}

/// Test result aggregation utilities
#[derive(Debug, Clone)]
pub struct TestResults {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub performance_measurements: Vec<PerformanceMeasurement>,
    pub test_duration: Duration,
}

impl TestResults {
    pub fn new() -> Self {
        Self {
            total_tests: 0,
            passed_tests: 0,
            failed_tests: 0,
            performance_measurements: Vec::new(),
            test_duration: Duration::from_nanos(0),
        }
    }

    pub fn add_test_result(&mut self, passed: bool) {
        self.total_tests += 1;
        if passed {
            self.passed_tests += 1;
        } else {
            self.failed_tests += 1;
        }
    }

    pub fn add_performance_measurement(&mut self, measurement: PerformanceMeasurement) {
        self.performance_measurements.push(measurement);
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_tests == 0 {
            1.0
        } else {
            self.passed_tests as f64 / self.total_tests as f64
        }
    }

    pub fn all_performance_requirements_met(&self) -> bool {
        self.performance_measurements.iter()
            .all(|m| m.meets_performance_requirements() && m.meets_memory_requirements())
    }

    pub fn print_summary(&self) {
        println!("📊 Test Results Summary:");
        println!("   Total Tests: {}", self.total_tests);
        println!("   Passed: {} ({:.1}%)", self.passed_tests, self.success_rate() * 100.0);
        println!("   Failed: {}", self.failed_tests);
        println!("   Test Duration: {:?}", self.test_duration);
        
        if !self.performance_measurements.is_empty() {
            println!("   Performance Tests: {}", self.performance_measurements.len());
            let perf_passed = self.performance_measurements.iter()
                .filter(|m| m.meets_performance_requirements() && m.meets_memory_requirements())
                .count();
            println!("   Performance Passed: {} ({:.1}%)", 
                    perf_passed, 
                    perf_passed as f64 / self.performance_measurements.len() as f64 * 100.0);
        }
    }
}

/// Assertion utilities with better error messages
pub fn assert_config_value_eq(config: &Config, path: &str, expected: &ConfigValue) -> ConfigResult<()> {
    let actual = config.get_value(path)?;
    
    if actual != *expected {
        return Err(ConfigError::validation(format!(
            "Configuration value mismatch at '{}': expected {:?}, got {:?}",
            path, expected, actual
        )));
    }
    
    Ok(())
}

pub fn assert_config_string_eq(config: &Config, path: &str, expected: &str) -> ConfigResult<()> {
    let value = config.get_value(path)?;
    let actual = value.as_string()?;
    
    if actual != expected {
        return Err(ConfigError::validation(format!(
            "String value mismatch at '{}': expected '{}', got '{}'",
            path, expected, actual
        )));
    }
    
    Ok(())
}

pub fn assert_config_integer_eq(config: &Config, path: &str, expected: i64) -> ConfigResult<()> {
    let value = config.get_value(path)?;
    let actual = value.as_integer()?;
    
    if actual != expected {
        return Err(ConfigError::validation(format!(
            "Integer value mismatch at '{}': expected {}, got {}",
            path, expected, actual
        )));
    }
    
    Ok(())
}

pub fn assert_config_boolean_eq(config: &Config, path: &str, expected: bool) -> ConfigResult<()> {
    let value = config.get_value(path)?;
    let actual = value.as_bool()?;
    
    if actual != expected {
        return Err(ConfigError::validation(format!(
            "Boolean value mismatch at '{}': expected {}, got {}",
            path, expected, actual
        )));
    }
    
    Ok(())
}

/// Async test utilities
pub async fn run_with_timeout<F, R>(test_fn: F, timeout_duration: Duration) -> Result<R, String> 
where
    F: std::future::Future<Output = R>,
{
    match timeout(timeout_duration, test_fn).await {
        Ok(result) => Ok(result),
        Err(_) => Err(format!("Test timed out after {:?}", timeout_duration)),
    }
}

pub async fn measure_async_operation<F, R>(operation_name: &str, operation: F) -> (R, PerformanceMeasurement)
where
    F: std::future::Future<Output = ConfigResult<R>>,
{
    let start_time = std::time::Instant::now();
    let memory_before = get_current_memory_usage();
    
    let result = operation.await;
    
    let duration = start_time.elapsed();
    let memory_after = get_current_memory_usage();
    let memory_delta = memory_after.saturating_sub(memory_before);
    
    let measurement = PerformanceMeasurement::new(operation_name)
        .with_duration(duration)
        .with_memory_usage(memory_delta)
        .with_success(result.is_ok());
    
    match result {
        Ok(value) => (value, measurement),
        Err(e) => panic!("Operation '{}' failed: {}", operation_name, e),
    }
}

/// Memory usage utilities
pub fn get_current_memory_usage() -> usize {
    // Platform-specific memory usage detection
    #[cfg(target_os = "linux")]
    {
        get_linux_memory_usage().unwrap_or(0)
    }
    #[cfg(target_os = "macos")]
    {
        get_macos_memory_usage().unwrap_or(0)
    }
    #[cfg(target_os = "windows")]
    {
        get_windows_memory_usage().unwrap_or(0)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        0 // Fallback for unknown platforms
    }
}

#[cfg(target_os = "linux")]
fn get_linux_memory_usage() -> Option<usize> {
    use std::fs;
    
    let status = fs::read_to_string("/proc/self/status").ok()?;
    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(kb) = parts[1].parse::<usize>() {
                    return Some(kb * 1024); // Convert KB to bytes
                }
            }
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn get_macos_memory_usage() -> Option<usize> {
    // On macOS, we could use mach_task_self() and task_info()
    // For simplicity in testing, return a reasonable estimate
    Some(10 * 1024 * 1024) // 10MB estimate
}

#[cfg(target_os = "windows")]
fn get_windows_memory_usage() -> Option<usize> {
    // On Windows, we could use GetProcessMemoryInfo()
    // For simplicity in testing, return a reasonable estimate
    Some(10 * 1024 * 1024) // 10MB estimate
}

/// Configuration comparison utilities
pub fn configs_equal(config1: &Config, config2: &Config) -> bool {
    if config1.version != config2.version {
        return false;
    }
    
    if config1.sections.len() != config2.sections.len() {
        return false;
    }
    
    for (key, value1) in &config1.sections {
        match config2.sections.get(key) {
            Some(value2) => {
                if value1 != value2 {
                    return false;
                }
            }
            None => return false,
        }
    }
    
    true
}

pub fn find_config_differences(config1: &Config, config2: &Config) -> Vec<String> {
    let mut differences = Vec::new();
    
    if config1.version != config2.version {
        differences.push(format!("Version: '{}' vs '{}'", config1.version, config2.version));
    }
    
    // Check for sections in config1 but not in config2
    for key in config1.sections.keys() {
        if !config2.sections.contains_key(key) {
            differences.push(format!("Section '{}' only in first config", key));
        }
    }
    
    // Check for sections in config2 but not in config1
    for key in config2.sections.keys() {
        if !config1.sections.contains_key(key) {
            differences.push(format!("Section '{}' only in second config", key));
        }
    }
    
    // Check for value differences in common sections
    for (key, value1) in &config1.sections {
        if let Some(value2) = config2.sections.get(key) {
            if value1 != value2 {
                differences.push(format!("Section '{}' has different values", key));
            }
        }
    }
    
    differences
}

/// Test data generators
pub fn generate_test_configurations(count: usize) -> Vec<Config> {
    let mut configs = Vec::new();
    
    for i in 0..count {
        let config = ConfigTestBuilder::new()
            .version(&format!("1.0.{}", i))
            .add_section("app")
                .string("name", &format!("test_app_{}", i))
                .integer("instance_id", i as i64)
                .boolean("debug", i % 2 == 0)
                .finish_section()
            .add_section("database")
                .string("host", "localhost")
                .integer("port", 5432 + (i as i64 % 10))
                .boolean("ssl", true)
                .finish_section()
            .build();
        
        configs.push(config);
    }
    
    configs
}

pub fn generate_large_configuration() -> Config {
    let mut builder = ConfigTestBuilder::new()
        .version("1.0.0");
    
    // Generate many sections with various data types
    for i in 0..LARGE_CONFIG_SECTIONS {
        let section_builder = builder
            .add_section(&format!("section_{}", i))
            .string("string_field", &format!("value_{}", i))
            .integer("integer_field", i as i64)
            .boolean("boolean_field", i % 2 == 0)
            .float("float_field", i as f64 * 3.14159);
        
        // Add arrays
        let array_values = (0..10).map(|j| 
            ConfigValue::string(&format!("array_item_{}_{}", i, j))
        ).collect();
        
        builder = section_builder
            .array("array_field", array_values)
            .finish_section();
    }
    
    builder.build()
}

/// Validation utilities
pub fn validate_configuration_structure(config: &Config) -> Result<(), String> {
    if config.version.is_empty() {
        return Err("Configuration version cannot be empty".to_string());
    }
    
    if !config.version.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-') {
        return Err("Configuration version contains invalid characters".to_string());
    }
    
    // Validate section names
    for section_name in config.sections.keys() {
        if section_name.is_empty() {
            return Err("Section name cannot be empty".to_string());
        }
        
        if section_name.contains(' ') {
            return Err(format!("Section name '{}' cannot contain spaces", section_name));
        }
    }
    
    Ok(())
}

/// Test reporting utilities
pub fn print_test_header(test_name: &str) {
    println!("\n🧪 {}", test_name);
    println!("{}", "=".repeat(test_name.len() + 3));
}

pub fn print_test_result(test_name: &str, passed: bool, duration: Duration) {
    let status = if passed { "✅ PASS" } else { "❌ FAIL" };
    println!("   {} {} ({:?})", status, test_name, duration);
}

pub fn print_performance_summary(measurements: &[PerformanceMeasurement]) {
    if measurements.is_empty() {
        return;
    }
    
    println!("\n📊 Performance Summary:");
    for measurement in measurements {
        let status = if measurement.meets_performance_requirements() { "✅" } else { "❌" };
        println!("   {} {}: {:?}", status, measurement.operation, measurement.duration);
        
        if let Some(memory) = measurement.memory_usage_bytes {
            let memory_mb = memory as f64 / (1024.0 * 1024.0);
            let memory_status = if measurement.meets_memory_requirements() { "✅" } else { "❌" };
            println!("      {} Memory: {:.2} MB", memory_status, memory_mb);
        }
    }
}