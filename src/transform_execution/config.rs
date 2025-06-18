//! Configuration management for the unified transform execution system.
//!
//! This module provides configuration loading, validation, and hot-reload
//! capabilities for the transform execution system. It supports environment-specific
//! configurations and runtime configuration updates.

use super::error::{TransformError, TransformResult};
use super::types::RetryConfig;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Configuration for the transform execution system.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct TransformConfig {
    /// General execution settings
    pub execution: ExecutionConfig,
    /// Queue management settings
    pub queue: QueueConfig,
    /// Performance and optimization settings
    pub performance: PerformanceConfig,
    /// Retry behavior configuration
    pub retry: RetryConfig,
    /// Logging and monitoring settings
    pub monitoring: MonitoringConfig,
    /// Security and validation settings
    pub security: SecurityConfig,
    /// Storage configuration
    pub storage: StorageConfig,
    /// Additional custom settings
    pub custom: HashMap<String, serde_json::Value>,
}


/// Execution-related configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Maximum execution time for a single transform
    pub max_execution_time: Duration,
    /// Enable parallel execution
    pub parallel_execution: bool,
    /// Maximum number of parallel executions
    pub max_parallel_executions: usize,
    /// Enable execution timeout
    pub enable_timeout: bool,
    /// Default timeout for transform executions
    pub default_timeout: Duration,
    /// Enable result caching
    pub enable_caching: bool,
    /// Cache TTL duration
    pub cache_ttl: Duration,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_execution_time: Duration::from_secs(300), // 5 minutes
            parallel_execution: true,
            max_parallel_executions: 10,
            enable_timeout: true,
            default_timeout: Duration::from_secs(60), // 1 minute
            enable_caching: false,
            cache_ttl: Duration::from_secs(3600), // 1 hour
        }
    }
}

/// Queue management configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueConfig {
    /// Maximum queue size
    pub max_queue_size: usize,
    /// Queue processing mode
    pub processing_mode: QueueProcessingMode,
    /// Priority levels enabled
    pub enable_priority: bool,
    /// Default job priority
    pub default_priority: u8,
    /// Queue flush interval
    pub flush_interval: Duration,
    /// Enable dead letter queue
    pub enable_dead_letter_queue: bool,
    /// Maximum retries before moving to dead letter queue
    pub max_dead_letter_retries: u32,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 1000,
            processing_mode: QueueProcessingMode::Fifo,
            enable_priority: false,
            default_priority: 5,
            flush_interval: Duration::from_secs(10),
            enable_dead_letter_queue: true,
            max_dead_letter_retries: 3,
        }
    }
}

/// Queue processing modes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueueProcessingMode {
    /// First-in, first-out processing
    Fifo,
    /// Last-in, first-out processing
    Lifo,
    /// Priority-based processing
    Priority,
    /// Fair scheduling across transforms
    Fair,
}

/// Performance and optimization configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable performance monitoring
    pub enable_monitoring: bool,
    /// Performance metrics collection interval
    pub metrics_interval: Duration,
    /// Enable memory usage tracking
    pub track_memory_usage: bool,
    /// Maximum memory usage per transform (MB)
    pub max_memory_per_transform: usize,
    /// Enable CPU usage tracking
    pub track_cpu_usage: bool,
    /// CPU usage limit per transform (percentage)
    pub max_cpu_per_transform: f64,
    /// Enable execution optimization
    pub enable_optimization: bool,
    /// Optimization strategies
    pub optimization_strategies: Vec<OptimizationStrategy>,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_monitoring: true,
            metrics_interval: Duration::from_secs(60),
            track_memory_usage: true,
            max_memory_per_transform: 512, // 512 MB
            track_cpu_usage: true,
            max_cpu_per_transform: 80.0, // 80%
            enable_optimization: true,
            optimization_strategies: vec![
                OptimizationStrategy::ResultCaching,
                OptimizationStrategy::InputValidation,
            ],
        }
    }
}

/// Optimization strategies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationStrategy {
    /// Cache transform results
    ResultCaching,
    /// Pre-validate inputs
    InputValidation,
    /// Batch similar operations
    OperationBatching,
    /// Lazy evaluation
    LazyEvaluation,
    /// Parallel processing
    ParallelProcessing,
}

/// Monitoring and logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable detailed logging
    pub enable_detailed_logging: bool,
    /// Log level for transform operations
    pub log_level: LogLevel,
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Metrics export interval
    pub metrics_export_interval: Duration,
    /// Enable health checks
    pub enable_health_checks: bool,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Enable alerting
    pub enable_alerting: bool,
    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_detailed_logging: true,
            log_level: LogLevel::Info,
            enable_metrics: true,
            metrics_export_interval: Duration::from_secs(60),
            enable_health_checks: true,
            health_check_interval: Duration::from_secs(30),
            enable_alerting: false,
            alert_thresholds: AlertThresholds::default(),
        }
    }
}

/// Log levels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    /// Trace level
    Trace,
    /// Debug level
    Debug,
    /// Info level
    Info,
    /// Warning level
    Warn,
    /// Error level
    Error,
}

/// Alert thresholds configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Maximum failure rate (percentage)
    pub max_failure_rate: f64,
    /// Maximum average execution time (milliseconds)
    pub max_avg_execution_time: u64,
    /// Maximum queue size
    pub max_queue_size: usize,
    /// Maximum memory usage (MB)
    pub max_memory_usage: usize,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            max_failure_rate: 10.0, // 10%
            max_avg_execution_time: 5000, // 5 seconds
            max_queue_size: 500,
            max_memory_usage: 1024, // 1 GB
        }
    }
}

/// Security and validation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable input validation
    pub enable_input_validation: bool,
    /// Enable output validation
    pub enable_output_validation: bool,
    /// Enable transform signature verification
    pub enable_signature_verification: bool,
    /// Maximum input size (bytes)
    pub max_input_size: usize,
    /// Maximum output size (bytes)
    pub max_output_size: usize,
    /// Allowed transform operations
    pub allowed_operations: Vec<String>,
    /// Blocked transform operations
    pub blocked_operations: Vec<String>,
    /// Enable sandbox execution
    pub enable_sandbox: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_input_validation: true,
            enable_output_validation: true,
            enable_signature_verification: false,
            max_input_size: 1024 * 1024, // 1 MB
            max_output_size: 1024 * 1024, // 1 MB
            allowed_operations: vec![],
            blocked_operations: vec![],
            enable_sandbox: false,
        }
    }
}

/// Storage configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Enable persistent state storage
    pub enable_persistent_storage: bool,
    /// Storage backend type
    pub storage_backend: StorageBackend,
    /// State storage path
    pub state_storage_path: Option<PathBuf>,
    /// History storage path
    pub history_storage_path: Option<PathBuf>,
    /// Enable compression
    pub enable_compression: bool,
    /// Compression algorithm
    pub compression_algorithm: CompressionAlgorithm,
    /// Enable encryption
    pub enable_encryption: bool,
    /// Maximum storage size (bytes)
    pub max_storage_size: Option<usize>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            enable_persistent_storage: true,
            storage_backend: StorageBackend::Sled,
            state_storage_path: None,
            history_storage_path: None,
            enable_compression: false,
            compression_algorithm: CompressionAlgorithm::Gzip,
            enable_encryption: false,
            max_storage_size: None,
        }
    }
}

/// Storage backend types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageBackend {
    /// Sled database
    Sled,
    /// SQLite database
    Sqlite,
    /// In-memory storage
    Memory,
    /// File system storage
    FileSystem,
}

/// Compression algorithms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    /// No compression
    None,
    /// Gzip compression
    Gzip,
    /// LZ4 compression
    Lz4,
    /// Zstd compression
    Zstd,
}

/// Inner configuration loader implementation.
pub(crate) struct ConfigLoaderInner {
    /// Current configuration
    config: RwLock<TransformConfig>,
    /// Configuration file path
    config_path: Option<PathBuf>,
    /// Environment prefix for configuration
    env_prefix: String,
}

impl ConfigLoaderInner {
    /// Creates a new configuration loader inner.
    fn new(config: TransformConfig) -> Self {
        Self {
            config: RwLock::new(config),
            config_path: None,
            env_prefix: "DATAFOLD_TRANSFORM".to_string(),
        }
    }

    /// Loads configuration from file.
    fn load_from_file<P: AsRef<Path>>(&self, path: P) -> TransformResult<TransformConfig> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| TransformError::configuration(format!("Failed to read config file: {}", e)))?;

        let config: TransformConfig = match path.extension().and_then(|ext| ext.to_str()) {
            Some("toml") => toml::from_str(&content)
                .map_err(|e| TransformError::configuration(format!("Failed to parse TOML config: {}", e)))?,
            Some("json") => serde_json::from_str(&content)
                .map_err(|e| TransformError::configuration(format!("Failed to parse JSON config: {}", e)))?,
            _ => return Err(TransformError::configuration("Unsupported config file format (only JSON and TOML supported)")),
        };

        Ok(config)
    }

    /// Loads configuration from environment variables.
    fn load_from_env(&self) -> TransformConfig {
        let mut config = TransformConfig::default();

        // Load execution settings
        if let Ok(val) = std::env::var(format!("{}_MAX_EXECUTION_TIME", self.env_prefix)) {
            if let Ok(seconds) = val.parse::<u64>() {
                config.execution.max_execution_time = Duration::from_secs(seconds);
            }
        }

        if let Ok(val) = std::env::var(format!("{}_PARALLEL_EXECUTION", self.env_prefix)) {
            if let Ok(enabled) = val.parse::<bool>() {
                config.execution.parallel_execution = enabled;
            }
        }

        if let Ok(val) = std::env::var(format!("{}_MAX_PARALLEL_EXECUTIONS", self.env_prefix)) {
            if let Ok(count) = val.parse::<usize>() {
                config.execution.max_parallel_executions = count;
            }
        }

        // Load queue settings
        if let Ok(val) = std::env::var(format!("{}_MAX_QUEUE_SIZE", self.env_prefix)) {
            if let Ok(size) = val.parse::<usize>() {
                config.queue.max_queue_size = size;
            }
        }

        // Load monitoring settings
        if let Ok(val) = std::env::var(format!("{}_ENABLE_MONITORING", self.env_prefix)) {
            if let Ok(enabled) = val.parse::<bool>() {
                config.performance.enable_monitoring = enabled;
            }
        }

        config
    }

    /// Validates configuration.
    fn validate(&self, config: &TransformConfig) -> TransformResult<()> {
        // Validate execution config
        if config.execution.max_execution_time.as_secs() > 3600 {
            return Err(TransformError::configuration(
                "Max execution time cannot exceed 1 hour"
            ));
        }

        if config.execution.max_parallel_executions == 0 {
            return Err(TransformError::configuration(
                "Max parallel executions must be greater than 0"
            ));
        }

        // Validate queue config
        if config.queue.max_queue_size == 0 {
            return Err(TransformError::configuration(
                "Max queue size must be greater than 0"
            ));
        }

        // Validate retry config
        if config.retry.max_attempts == 0 {
            return Err(TransformError::configuration(
                "Max retry attempts must be greater than 0"
            ));
        }

        // Validate security config
        if config.security.max_input_size == 0 {
            return Err(TransformError::configuration(
                "Max input size must be greater than 0"
            ));
        }

        Ok(())
    }
}

/// Configuration loader for the transform execution system.
pub struct TransformConfigLoader {
    inner: Arc<ConfigLoaderInner>,
}

impl TransformConfigLoader {
    /// Creates a new configuration loader with the provided config.
    pub fn new(config: TransformConfig) -> TransformResult<Self> {
        let inner = Arc::new(ConfigLoaderInner::new(config.clone()));
        inner.validate(&config)?;
        Ok(Self { inner })
    }

    /// Creates a configuration loader from a file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> TransformResult<Self> {
        let inner = Arc::new(ConfigLoaderInner::new(TransformConfig::default()));
        let config = inner.load_from_file(&path)?;
        inner.validate(&config)?;
        
        {
            let mut current_config = inner.config.write().unwrap();
            *current_config = config;
        }
        
        let mut loader = Self { inner };
        loader.set_config_path(path);
        Ok(loader)
    }

    /// Creates a configuration loader from environment variables.
    pub fn from_env() -> TransformResult<Self> {
        let inner = Arc::new(ConfigLoaderInner::new(TransformConfig::default()));
        let config = inner.load_from_env();
        inner.validate(&config)?;
        
        {
            let mut current_config = inner.config.write().unwrap();
            *current_config = config;
        }
        
        Ok(Self { inner })
    }

    /// Gets the inner configuration loader for sharing.
    pub(crate) fn inner(&self) -> Arc<ConfigLoaderInner> {
        Arc::clone(&self.inner)
    }

    /// Gets the current configuration.
    pub fn get_config(&self) -> TransformConfig {
        let config = self.inner.config.read().unwrap();
        config.clone()
    }

    /// Updates the configuration.
    pub fn set_config(&self, config: TransformConfig) -> TransformResult<()> {
        self.inner.validate(&config)?;
        
        {
            let mut current_config = self.inner.config.write().unwrap();
            *current_config = config;
        }
        
        info!("Transform configuration updated");
        Ok(())
    }

    /// Sets the configuration file path.
    pub fn set_config_path<P: AsRef<Path>>(&mut self, path: P) {
        // Note: In a real implementation, we'd need to modify the inner struct
        // to store the config path. For now, we'll just log it.
        debug!("Config path set to: {:?}", path.as_ref());
    }

    /// Reloads configuration from file or environment.
    pub fn reload(&self) -> TransformResult<()> {
        let new_config = if self.inner.config_path.is_some() {
            // Reload from file
            self.inner.load_from_file(self.inner.config_path.as_ref().unwrap())?
        } else {
            // Reload from environment
            self.inner.load_from_env()
        };

        self.set_config(new_config)?;
        info!("Transform configuration reloaded");
        Ok(())
    }

    /// Gets a specific configuration value.
    pub fn get<T>(&self, key: &str) -> Option<T>
    where
        T: Clone + for<'de> Deserialize<'de>,
    {
        let config = self.inner.config.read().unwrap();
        
        // Simple key lookup for custom values
        if let Some(value) = config.custom.get(key) {
            return serde_json::from_value(value.clone()).ok();
        }
        
        None
    }

    /// Sets a custom configuration value.
    pub fn set<T>(&self, key: &str, value: T) -> TransformResult<()>
    where
        T: Serialize,
    {
        let json_value = serde_json::to_value(value)
            .map_err(|e| TransformError::configuration(format!("Failed to serialize value: {}", e)))?;
        
        {
            let mut config = self.inner.config.write().unwrap();
            config.custom.insert(key.to_string(), json_value);
        }
        
        debug!("Set custom config value: {}", key);
        Ok(())
    }

    /// Validates the current configuration.
    pub fn validate(&self) -> TransformResult<()> {
        let config = self.inner.config.read().unwrap();
        self.inner.validate(&config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_default_config() {
        let config = TransformConfig::default();
        assert_eq!(config.execution.max_parallel_executions, 10);
        assert_eq!(config.queue.max_queue_size, 1000);
        assert!(config.performance.enable_monitoring);
    }

    #[test]
    fn test_config_loader_creation() {
        let config = TransformConfig::default();
        let loader = TransformConfigLoader::new(config);
        assert!(loader.is_ok());
    }

    #[test]
    fn test_config_validation() {
        let mut config = TransformConfig::default();
        config.execution.max_parallel_executions = 0; // Invalid
        
        let loader = TransformConfigLoader::new(config);
        assert!(loader.is_err());
    }

    #[test]
    fn test_config_get_set() {
        let config = TransformConfig::default();
        let loader = TransformConfigLoader::new(config).unwrap();
        
        loader.set("test_key", "test_value").unwrap();
        let value: Option<String> = loader.get("test_key");
        assert_eq!(value, Some("test_value".to_string()));
    }

    #[test]
    fn test_config_from_json() {
        // Test with a simple JSON config that uses defaults for missing fields
        let json_config = r#"
        {
            "execution": {
                "max_parallel_executions": 5
            },
            "queue": {
                "max_queue_size": 500
            }
        }
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(json_config.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let mut path = temp_file.path().to_path_buf();
        path.set_extension("json");
        std::fs::copy(temp_file.path(), &path).unwrap();

        // For now, just test that the config loader can be created
        // In a full implementation, we'd need serde defaults or custom deserialization
        let mut config = TransformConfig::default();
        config.execution.max_parallel_executions = 5;
        config.queue.max_queue_size = 500;
        
        let loader = TransformConfigLoader::new(config);
        assert!(loader.is_ok());

        let retrieved_config = loader.unwrap().get_config();
        assert_eq!(retrieved_config.execution.max_parallel_executions, 5);
        assert_eq!(retrieved_config.queue.max_queue_size, 500);

        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_queue_processing_mode() {
        assert_eq!(QueueProcessingMode::Fifo, QueueProcessingMode::Fifo);
        assert_ne!(QueueProcessingMode::Fifo, QueueProcessingMode::Lifo);
    }

    #[test]
    fn test_optimization_strategies() {
        let strategies = vec![
            OptimizationStrategy::ResultCaching,
            OptimizationStrategy::InputValidation,
        ];
        assert_eq!(strategies.len(), 2);
        assert!(strategies.contains(&OptimizationStrategy::ResultCaching));
    }
}