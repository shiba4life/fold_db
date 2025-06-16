//! Integration traits for PBI 26/27 compatibility
//!
//! This module provides traits that enable seamless integration with existing
//! PBI 26 (unified reporting) and PBI 27 (cross-platform configuration) systems.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::base::{BaseConfig, ConfigReporting, ConfigValidation};
use super::core::{ConfigChangeEvent, ConfigEventType, ConfigEvents};
use super::error::{TraitConfigError, TraitConfigResult, ValidationContext};
use crate::config::error::ConfigError;
use crate::config::platform::{EnhancedPlatformInfo, PlatformConfigPaths};

/// Cross-platform configuration integration trait (PBI 27)
///
/// Provides seamless integration with the existing cross-platform configuration
/// infrastructure, including platform-specific path resolution, file operations,
/// and platform capabilities detection.
#[async_trait]
pub trait CrossPlatformConfig: BaseConfig {
    /// Get platform-specific configuration paths
    ///
    /// Returns the platform-specific path resolver for this configuration.
    fn platform_paths(&self) -> &dyn PlatformConfigPaths;

    /// Get enhanced platform information
    ///
    /// Returns detailed platform capabilities and feature availability.
    fn platform_info(&self) -> EnhancedPlatformInfo;

    /// Load configuration using platform-specific optimizations
    ///
    /// Leverages platform-specific file operations and caching for optimal performance.
    async fn load_platform_optimized(&self, path: &Path) -> TraitConfigResult<Self>
    where
        Self: Sized;

    /// Save configuration using platform-specific optimizations
    ///
    /// Uses atomic writes and platform-specific file operations for reliability.
    async fn save_platform_optimized(&self, path: &Path) -> TraitConfigResult<()>;

    /// Get platform-specific configuration defaults
    ///
    /// Returns default configuration values tailored to the current platform.
    fn platform_defaults(&self) -> HashMap<String, crate::config::value::ConfigValue>;

    /// Migrate configuration for current platform
    ///
    /// Adapts configuration data to be optimal for the current platform.
    async fn migrate_for_platform(&mut self) -> TraitConfigResult<()>;

    /// Validate platform compatibility
    ///
    /// Ensures the configuration is compatible with the current platform.
    fn validate_platform_compatibility(&self) -> TraitConfigResult<()>;

    /// Get platform-specific performance settings
    ///
    /// Returns optimized performance settings for the current platform.
    fn platform_performance_settings(&self) -> PlatformPerformanceSettings;
}

/// Unified reporting integration trait (PBI 26)
///
/// Integrates configuration events and metrics with the unified reporting system,
/// providing comprehensive observability and monitoring capabilities.
#[async_trait]
pub trait ReportableConfig: BaseConfig + ConfigReporting {
    /// Report configuration to unified reporting system
    ///
    /// Sends configuration data and metadata to the unified reporting infrastructure.
    async fn report_to_unified_system(&self) -> TraitConfigResult<()>;

    /// Report configuration metrics
    ///
    /// Emits configuration-related metrics for monitoring and alerting.
    async fn report_metrics(&self, metrics: ConfigMetrics) -> TraitConfigResult<()>;

    /// Report configuration health status
    ///
    /// Reports the overall health and status of the configuration.
    async fn report_health_status(&self) -> TraitConfigResult<HealthStatus>;

    /// Register with unified reporting system
    ///
    /// Registers this configuration instance with the reporting system for ongoing monitoring.
    async fn register_for_reporting(
        &self,
        config: ReportingRegistration,
    ) -> TraitConfigResult<String>;

    /// Unregister from unified reporting system
    ///
    /// Removes this configuration instance from active monitoring.
    async fn unregister_from_reporting(&self, registration_id: &str) -> TraitConfigResult<()>;

    /// Get reporting capabilities
    ///
    /// Returns the reporting features supported by this configuration.
    fn reporting_capabilities(&self) -> ReportingCapabilities;

    /// Create unified report
    ///
    /// Generates a comprehensive unified report for this configuration.
    async fn create_unified_report(&self) -> TraitConfigResult<UnifiedReport>;
}

/// Enhanced validation framework integration trait
///
/// Provides trait-based validation that integrates with existing validation
/// infrastructure while adding enhanced capabilities.
pub trait ValidatableConfig: BaseConfig + ConfigValidation {
    /// Validate using trait-based validation framework
    ///
    /// Performs comprehensive validation using the trait-based validation system.
    fn validate_with_traits(&self) -> TraitConfigResult<ValidationResult>;

    /// Get trait-specific validation rules
    ///
    /// Returns validation rules specific to the traits implemented by this configuration.
    fn trait_validation_rules(&self) -> Vec<TraitValidationRule>;

    /// Validate trait composition
    ///
    /// Ensures that the combination of traits is valid and consistent.
    fn validate_trait_composition(&self) -> TraitConfigResult<()>;

    /// Validate against schema
    ///
    /// Validates configuration against a predefined schema.
    fn validate_against_schema(&self, schema: &ConfigSchema)
        -> TraitConfigResult<ValidationResult>;

    /// Get validation context for debugging
    ///
    /// Returns detailed context information for validation errors.
    fn validation_context(&self) -> ValidationContext;

    /// Perform incremental validation
    ///
    /// Validates only the parts of the configuration that have changed.
    fn validate_incremental(&self, changes: &[String]) -> TraitConfigResult<ValidationResult>;
}

/// Configuration observability and monitoring trait
///
/// Provides comprehensive observability features including change tracking,
/// performance monitoring, and debugging capabilities.
#[async_trait]
pub trait ObservableConfig: BaseConfig + ConfigEvents {
    /// Start configuration monitoring
    ///
    /// Begins active monitoring of configuration changes and performance.
    async fn start_monitoring(&mut self) -> TraitConfigResult<MonitoringSession>;

    /// Stop configuration monitoring
    ///
    /// Ends active monitoring and cleanup resources.
    async fn stop_monitoring(&mut self, session: MonitoringSession) -> TraitConfigResult<()>;

    /// Get configuration telemetry
    ///
    /// Returns telemetry data about configuration usage and performance.
    async fn get_telemetry(&self) -> TraitConfigResult<ConfigTelemetry>;

    /// Record configuration access
    ///
    /// Logs access to configuration values for usage tracking.
    fn record_access(&self, field_path: &str, access_type: AccessType);

    /// Get access patterns
    ///
    /// Returns information about how the configuration is being accessed.
    fn access_patterns(&self) -> Vec<AccessPattern>;

    /// Enable debug mode
    ///
    /// Enables detailed logging and debugging for configuration operations.
    fn enable_debug_mode(&mut self, level: DebugLevel);

    /// Get configuration snapshot
    ///
    /// Creates a point-in-time snapshot of the configuration state.
    async fn create_snapshot(&self) -> TraitConfigResult<ConfigSnapshot>;

    /// Compare with snapshot
    ///
    /// Compares current configuration state with a previous snapshot.
    fn compare_with_snapshot(&self, snapshot: &ConfigSnapshot) -> ConfigComparison;
}

/// Platform-specific performance settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformPerformanceSettings {
    /// Enable memory mapping for large files
    pub enable_memory_mapping: bool,

    /// Use platform-specific atomic operations
    pub use_atomic_operations: bool,

    /// Enable file system caching
    pub enable_fs_caching: bool,

    /// Optimal buffer size for I/O operations
    pub optimal_buffer_size: usize,

    /// Maximum concurrent operations
    pub max_concurrent_operations: usize,

    /// Platform-specific optimization flags
    pub optimization_flags: HashMap<String, bool>,
}

/// Configuration metrics for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetrics {
    /// Load time in milliseconds
    pub load_time_ms: f64,

    /// Save time in milliseconds
    pub save_time_ms: f64,

    /// Validation time in milliseconds
    pub validation_time_ms: f64,

    /// Configuration size in bytes
    pub size_bytes: u64,

    /// Number of configuration fields
    pub field_count: u32,

    /// Number of sections
    pub section_count: u32,

    /// Cache hit rate
    pub cache_hit_rate: f64,

    /// Error rate
    pub error_rate: f64,

    /// Custom metrics
    pub custom_metrics: HashMap<String, f64>,
}

/// Health status of configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall health status
    pub status: HealthLevel,

    /// Health score (0-100)
    pub score: u8,

    /// Specific health indicators
    pub indicators: Vec<HealthIndicator>,

    /// Recommendations for improvement
    pub recommendations: Vec<String>,

    /// Last health check timestamp
    pub last_checked: chrono::DateTime<chrono::Utc>,
}

/// Health levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthLevel {
    /// Configuration is healthy
    Healthy,

    /// Configuration has minor issues
    Warning,

    /// Configuration has significant issues
    Critical,

    /// Configuration is unusable
    Failed,
}

/// Health indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIndicator {
    /// Indicator name
    pub name: String,

    /// Indicator status
    pub status: HealthLevel,

    /// Description
    pub description: String,

    /// Metric value (if applicable)
    pub value: Option<f64>,

    /// Threshold that triggered this indicator
    pub threshold: Option<f64>,
}

/// Reporting registration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingRegistration {
    /// Instance identifier
    pub instance_id: String,

    /// Configuration type
    pub config_type: String,

    /// Reporting frequency in seconds
    pub frequency_secs: u64,

    /// Metrics to report
    pub metrics: Vec<String>,

    /// Event types to report
    pub events: Vec<String>,

    /// Tags for categorization
    pub tags: HashMap<String, String>,
}

/// Reporting capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingCapabilities {
    /// Supported report types
    pub report_types: Vec<String>,

    /// Supported metrics
    pub metrics: Vec<String>,

    /// Supported event types
    pub event_types: Vec<String>,

    /// Real-time reporting support
    pub real_time_support: bool,

    /// Batch reporting support
    pub batch_support: bool,

    /// Custom reporting support
    pub custom_support: bool,
}

/// Unified report structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedReport {
    /// Report timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Report type
    pub report_type: String,

    /// Configuration summary
    pub config_summary: ConfigSummary,

    /// Metrics data
    pub metrics: ConfigMetrics,

    /// Health status
    pub health: HealthStatus,

    /// Recent events
    pub events: Vec<ConfigChangeEvent>,

    /// Validation results
    pub validation: Option<ValidationResult>,

    /// Custom report sections
    pub custom_sections: HashMap<String, serde_json::Value>,
}

/// Configuration summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSummary {
    /// Configuration type
    pub config_type: String,

    /// Version
    pub version: String,

    /// Size in bytes
    pub size_bytes: u64,

    /// Number of sections
    pub section_count: u32,

    /// Number of fields
    pub field_count: u32,

    /// Last modified timestamp
    pub last_modified: chrono::DateTime<chrono::Utc>,

    /// Platform information
    pub platform: String,

    /// Tags
    pub tags: HashMap<String, String>,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Overall validation success
    pub success: bool,

    /// Validation score (0-100)
    pub score: u8,

    /// Validation errors
    pub errors: Vec<ValidationError>,

    /// Validation warnings
    pub warnings: Vec<ValidationWarning>,

    /// Validation time in milliseconds
    pub validation_time_ms: f64,

    /// Rules that were applied
    pub applied_rules: Vec<String>,
}

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error code
    pub code: String,

    /// Error message
    pub message: String,

    /// Field path that caused the error
    pub field_path: String,

    /// Error severity
    pub severity: String,

    /// Suggestions for fixing
    pub suggestions: Vec<String>,
}

/// Validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning code
    pub code: String,

    /// Warning message
    pub message: String,

    /// Field path that caused the warning
    pub field_path: String,

    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Trait-specific validation rule
pub struct TraitValidationRule {
    /// Rule name
    pub name: String,

    /// Trait that defines this rule
    pub trait_name: &'static str,

    /// Rule description
    pub description: String,

    /// Rule implementation
    pub validator: Box<dyn Fn(&dyn std::any::Any) -> TraitConfigResult<()> + Send + Sync>,
}

impl std::fmt::Debug for TraitValidationRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TraitValidationRule")
            .field("name", &self.name)
            .field("trait_name", &self.trait_name)
            .field("description", &self.description)
            .field("validator", &"<function>")
            .finish()
    }
}

/// Configuration schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSchema {
    /// Schema version
    pub version: String,

    /// Schema name
    pub name: String,

    /// Required fields
    pub required_fields: Vec<String>,

    /// Field definitions
    pub fields: HashMap<String, FieldSchema>,

    /// Validation constraints
    pub constraints: Vec<SchemaConstraint>,
}

/// Field schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    /// Field type
    pub field_type: String,

    /// Field description
    pub description: String,

    /// Default value
    pub default: Option<serde_json::Value>,

    /// Validation rules
    pub validation: Vec<String>,

    /// Whether field is required
    pub required: bool,
}

/// Schema constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaConstraint {
    /// Constraint name
    pub name: String,

    /// Constraint type
    pub constraint_type: String,

    /// Constraint parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Monitoring session
#[derive(Debug, Clone)]
pub struct MonitoringSession {
    /// Session ID
    pub session_id: String,

    /// Start time
    pub start_time: chrono::DateTime<chrono::Utc>,

    /// Monitoring configuration
    pub config: MonitoringConfig,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable change tracking
    pub track_changes: bool,

    /// Enable performance monitoring
    pub monitor_performance: bool,

    /// Enable access logging
    pub log_access: bool,

    /// Sampling rate (0.0 to 1.0)
    pub sampling_rate: f64,

    /// Metrics collection interval in seconds
    pub metrics_interval_secs: u64,
}

/// Configuration telemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigTelemetry {
    /// Telemetry collection timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Performance metrics
    pub performance: ConfigMetrics,

    /// Usage statistics
    pub usage: UsageStatistics,

    /// Error statistics
    pub errors: ErrorStatistics,

    /// Resource usage
    pub resources: ResourceUsage,
}

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStatistics {
    /// Total number of accesses
    pub total_accesses: u64,

    /// Most accessed fields
    pub top_accessed_fields: Vec<(String, u64)>,

    /// Access patterns by time
    pub access_patterns: HashMap<String, u64>,

    /// Cache utilization
    pub cache_utilization: f64,
}

/// Error statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStatistics {
    /// Total error count
    pub total_errors: u64,

    /// Errors by type
    pub errors_by_type: HashMap<String, u64>,

    /// Error rate over time
    pub error_rate_history: Vec<(chrono::DateTime<chrono::Utc>, f64)>,

    /// Most common error messages
    pub common_errors: Vec<(String, u64)>,
}

/// Resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Memory usage in bytes
    pub memory_bytes: u64,

    /// CPU usage percentage
    pub cpu_percent: f64,

    /// Disk I/O operations
    pub disk_io_ops: u64,

    /// Network I/O bytes
    pub network_io_bytes: u64,

    /// File handles used
    pub file_handles: u32,
}

/// Access type for tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessType {
    /// Configuration field was read
    Read,

    /// Configuration field was written
    Write,

    /// Configuration was validated
    Validate,

    /// Configuration was serialized
    Serialize,
}

/// Access pattern information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPattern {
    /// Field path that was accessed
    pub field_path: String,

    /// Type of access
    pub access_type: AccessType,

    /// Access timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Frequency of access
    pub frequency: u64,

    /// Access source (if available)
    pub source: Option<String>,
}

/// Debug level for configuration operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebugLevel {
    /// No debug logging
    None,

    /// Basic debug information
    Basic,

    /// Detailed debug information
    Detailed,

    /// Verbose debug information
    Verbose,

    /// All possible debug information
    Trace,
}

/// Configuration snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSnapshot {
    /// Snapshot timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Snapshot ID
    pub snapshot_id: String,

    /// Configuration data at time of snapshot
    pub config_data: serde_json::Value,

    /// Metadata at time of snapshot
    pub metadata: HashMap<String, String>,

    /// Checksum for integrity
    pub checksum: String,
}

/// Configuration comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigComparison {
    /// Comparison timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Fields that were added since snapshot
    pub added_fields: Vec<String>,

    /// Fields that were removed since snapshot
    pub removed_fields: Vec<String>,

    /// Fields that were modified since snapshot
    pub modified_fields: Vec<String>,

    /// Overall change percentage
    pub change_percentage: f64,

    /// Detailed field changes
    pub field_changes: HashMap<String, FieldChange>,
}

/// Individual field change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldChange {
    /// Change type
    pub change_type: String,

    /// Old value (if applicable)
    pub old_value: Option<serde_json::Value>,

    /// New value (if applicable)
    pub new_value: Option<serde_json::Value>,

    /// Change timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for PlatformPerformanceSettings {
    fn default() -> Self {
        Self {
            enable_memory_mapping: true,
            use_atomic_operations: true,
            enable_fs_caching: true,
            optimal_buffer_size: 8192,
            max_concurrent_operations: 4,
            optimization_flags: HashMap::new(),
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            track_changes: true,
            monitor_performance: true,
            log_access: false,
            sampling_rate: 1.0,
            metrics_interval_secs: 60,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_level() {
        assert_eq!(HealthLevel::Healthy, HealthLevel::Healthy);
        assert_ne!(HealthLevel::Healthy, HealthLevel::Warning);
    }

    #[test]
    fn test_access_type() {
        assert_eq!(AccessType::Read, AccessType::Read);
        assert_ne!(AccessType::Read, AccessType::Write);
    }

    #[test]
    fn test_debug_level() {
        assert_eq!(DebugLevel::Basic, DebugLevel::Basic);
        assert_ne!(DebugLevel::Basic, DebugLevel::Verbose);
    }

    #[test]
    fn test_platform_performance_settings_default() {
        let settings = PlatformPerformanceSettings::default();
        assert!(settings.enable_memory_mapping);
        assert!(settings.use_atomic_operations);
        assert_eq!(settings.optimal_buffer_size, 8192);
    }

    #[test]
    fn test_monitoring_config_default() {
        let config = MonitoringConfig::default();
        assert!(config.track_changes);
        assert!(config.monitor_performance);
        assert!(!config.log_access);
        assert_eq!(config.sampling_rate, 1.0);
    }
}
