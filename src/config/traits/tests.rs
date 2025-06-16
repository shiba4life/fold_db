//! Testing utilities and mock implementations for configuration traits
//!
//! This module provides comprehensive testing infrastructure for configuration traits,
//! including mock implementations, test helpers, and integration test utilities.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::base::{
    BaseConfig, ConfigChangeType, ConfigLifecycle, ConfigMetadata, ConfigReporting,
    ConfigValidation, ReportingConfig, ValidationRule,
};
use super::core::{
    ConfigChangeEvent, ConfigEvents, ConfigMerge, ConfigMetadataTrait, ConfigSerialization,
    ListenerId, MergeStrategy, SerializationFormat,
};
use super::error::{TraitConfigError, TraitConfigResult, ValidationContext};
use super::integration::{
    AccessType, ConfigComparison, ConfigMetrics, ConfigSnapshot, ConfigTelemetry,
    CrossPlatformConfig, DebugLevel, HealthStatus, MonitoringSession, ObservableConfig,
    PlatformPerformanceSettings, ReportableConfig, ReportingCapabilities, UnifiedReport,
    ValidatableConfig, ValidationResult,
};
use crate::config::platform::{EnhancedPlatformInfo, PlatformConfigPaths};
use crate::config::{ConfigError, ConfigValue};

/// Mock configuration implementation for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockConfig {
    /// Configuration data
    pub data: HashMap<String, ConfigValue>,

    /// Configuration metadata
    pub metadata: ConfigMetadata,

    /// Mock state for testing
    pub mock_state: MockState,
}

/// Mock state for controlling test behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockState {
    /// Whether validation should succeed
    pub validation_should_succeed: bool,

    /// Whether save operations should succeed
    pub save_should_succeed: bool,

    /// Whether load operations should succeed
    pub load_should_succeed: bool,

    /// Simulated load time in milliseconds
    pub simulated_load_time_ms: u64,

    /// Simulated save time in milliseconds
    pub simulated_save_time_ms: u64,

    /// Event listeners for testing
    pub event_listeners: Vec<String>,

    /// Access log for testing
    pub access_log: Vec<AccessLogEntry>,
}

/// Access log entry for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessLogEntry {
    /// Timestamp of access
    pub timestamp: DateTime<Utc>,

    /// Field that was accessed
    pub field_path: String,

    /// Type of access
    pub access_type: AccessType,

    /// Source of access
    pub source: String,
}

/// Test helper for creating mock configurations
pub struct ConfigTestHelper;

/// Trait composition validator for testing
pub struct TraitCompositionValidator;

/// Mock platform paths for testing
#[derive(Debug)]
pub struct MockPlatformPaths {
    /// Base test directory
    pub base_dir: std::path::PathBuf,
}

impl MockConfig {
    /// Create a new mock configuration
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            metadata: ConfigMetadata::default(),
            mock_state: MockState {
                validation_should_succeed: true,
                save_should_succeed: true,
                load_should_succeed: true,
                simulated_load_time_ms: 10,
                simulated_save_time_ms: 5,
                event_listeners: Vec::new(),
                access_log: Vec::new(),
            },
        }
    }

    /// Create a mock configuration with test data
    pub fn with_test_data() -> Self {
        let mut config = Self::new();
        config
            .data
            .insert("test_string".to_string(), ConfigValue::string("test_value"));
        config
            .data
            .insert("test_number".to_string(), ConfigValue::integer(42));
        config
            .data
            .insert("test_bool".to_string(), ConfigValue::boolean(true));
        config
    }

    /// Set mock validation to fail
    pub fn set_validation_fail(&mut self) {
        self.mock_state.validation_should_succeed = false;
    }

    /// Set mock save to fail
    pub fn set_save_fail(&mut self) {
        self.mock_state.save_should_succeed = false;
    }

    /// Set mock load to fail
    pub fn set_load_fail(&mut self) {
        self.mock_state.load_should_succeed = false;
    }

    /// Get access log for testing
    pub fn access_log(&self) -> &[AccessLogEntry] {
        &self.mock_state.access_log
    }

    /// Record access for testing
    pub fn record_access(&mut self, field_path: &str, access_type: AccessType, source: &str) {
        self.mock_state.access_log.push(AccessLogEntry {
            timestamp: Utc::now(),
            field_path: field_path.to_string(),
            access_type,
            source: source.to_string(),
        });
    }
}

#[async_trait]
impl BaseConfig for MockConfig {
    type Error = ConfigError;
    type Event = ConfigChangeType;
    type TransformTarget = MockConfig;

    async fn load(_path: &Path) -> Result<Self, Self::Error> {
        // Simulate load time
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let mut config = MockConfig::with_test_data();
        if !config.mock_state.load_should_succeed {
            return Err(ConfigError::runtime("Mock load failure"));
        }

        config.metadata.accessed_at = Utc::now();
        Ok(config)
    }

    fn validate(&self) -> Result<(), Self::Error> {
        if !self.mock_state.validation_should_succeed {
            return Err(ConfigError::validation("Mock validation failure"));
        }

        // Basic validation
        for (key, value) in &self.data {
            if key.is_empty() {
                return Err(ConfigError::validation("Empty key not allowed"));
            }
            if value.is_null() {
                return Err(ConfigError::validation(format!(
                    "Null value not allowed for key '{}'",
                    key
                )));
            }
        }

        Ok(())
    }

    fn report_event(&self, _event: Self::Event) {
        // Mock implementation - just record that an event was reported
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl ConfigLifecycle for MockConfig {
    async fn save(&self, _path: &Path) -> Result<(), Self::Error> {
        // Simulate save time
        let save_time = self.mock_state.simulated_save_time_ms;
        tokio::time::sleep(tokio::time::Duration::from_millis(save_time)).await;

        if !self.mock_state.save_should_succeed {
            return Err(ConfigError::runtime("Mock save failure"));
        }

        Ok(())
    }

    async fn reload(&mut self, _path: &Path) -> Result<(), Self::Error> {
        if !self.mock_state.load_should_succeed {
            return Err(ConfigError::runtime("Mock reload failure"));
        }

        self.metadata.accessed_at = Utc::now();
        Ok(())
    }

    async fn has_changed(&self, _path: &Path) -> Result<bool, Self::Error> {
        // Mock implementation - return false for simplicity
        Ok(false)
    }

    fn get_metadata(&self) -> ConfigMetadata {
        self.metadata.clone()
    }

    fn set_metadata(&mut self, metadata: ConfigMetadata) {
        self.metadata = metadata;
    }
}

impl ConfigValidation for MockConfig {
    fn validate_with_context(&self) -> Result<(), ValidationContext> {
        if !self.mock_state.validation_should_succeed {
            return Err(
                ValidationContext::new("MockConfig", "mock_validation".to_string())
                    .with_path("mock.config")
                    .with_expected("success")
                    .with_actual("failure"),
            );
        }
        Ok(())
    }

    fn validate_field(&self, field_path: &str) -> Result<(), Self::Error> {
        if !self.data.contains_key(field_path) {
            return Err(ConfigError::not_found(format!("Field '{}'", field_path)));
        }
        Ok(())
    }

    fn validation_rules(&self) -> Vec<ValidationRule> {
        vec![
            ValidationRule::required("test_string"),
            ValidationRule::required("test_number"),
        ]
    }

    fn add_validation_rule(&mut self, _rule: ValidationRule) {
        // Mock implementation - rules are not actually stored
    }
}

impl ConfigReporting for MockConfig {
    fn report_change(&self, _change_type: ConfigChangeType, _context: Option<String>) {
        // Mock implementation
    }

    fn report_error(&self, _error: &Self::Error, _context: Option<String>) {
        // Mock implementation
    }

    fn report_metric(
        &self,
        _metric_name: &str,
        _value: f64,
        _tags: Option<HashMap<String, String>>,
    ) {
        // Mock implementation
    }

    fn reporting_config(&self) -> ReportingConfig {
        ReportingConfig::default()
    }

    fn set_reporting_config(&mut self, _config: ReportingConfig) {
        // Mock implementation
    }
}

impl ConfigMerge for MockConfig {
    fn merge_with_strategy(
        &self,
        other: &Self,
        strategy: MergeStrategy,
    ) -> TraitConfigResult<Self> {
        match strategy {
            MergeStrategy::Replace => {
                let mut merged = self.clone();
                for (key, value) in &other.data {
                    merged.data.insert(key.clone(), value.clone());
                }
                merged.metadata.updated_at = Utc::now();
                Ok(merged)
            }
            MergeStrategy::Keep => Ok(self.clone()),
            MergeStrategy::DeepMerge => {
                let mut merged = self.clone();
                for (key, value) in &other.data {
                    merged.data.insert(key.clone(), value.clone());
                }
                merged.metadata.updated_at = Utc::now();
                Ok(merged)
            } // Simple implementation
            MergeStrategy::FailOnConflict => {
                // Check for conflicts
                for key in other.data.keys() {
                    if self.data.contains_key(key) && self.data[key] != other.data[key] {
                        return Err(TraitConfigError::merge_conflict(format!(
                            "Conflict on key '{}'",
                            key
                        )));
                    }
                }
                let mut merged = self.clone();
                for (key, value) in &other.data {
                    merged.data.insert(key.clone(), value.clone());
                }
                merged.metadata.updated_at = Utc::now();
                Ok(merged)
            }
            MergeStrategy::Custom(_) => Err(TraitConfigError::not_implemented(
                "MockConfig",
                "custom_merge",
            )),
        }
    }

    fn deep_merge(&self, other: &Self) -> TraitConfigResult<Self> {
        let mut merged = self.clone();
        for (key, value) in &other.data {
            merged.data.insert(key.clone(), value.clone());
        }
        merged.metadata.updated_at = Utc::now();
        Ok(merged)
    }

    fn merge_sections(&self, other: &Self, sections: &[&str]) -> TraitConfigResult<Self> {
        let mut merged = self.clone();
        for section in sections {
            if let Some(value) = other.data.get(*section) {
                merged.data.insert(section.to_string(), value.clone());
            }
        }
        merged.metadata.updated_at = Utc::now();
        Ok(merged)
    }

    fn check_merge_conflicts(&self, other: &Self) -> Vec<super::core::MergeConflict> {
        let mut conflicts = Vec::new();
        for (key, other_value) in &other.data {
            if let Some(current_value) = self.data.get(key) {
                if current_value != other_value {
                    conflicts.push(super::core::MergeConflict {
                        field_path: key.clone(),
                        current_value: current_value.clone(),
                        other_value: other_value.clone(),
                        conflict_type: super::core::ConflictType::ValueMismatch,
                    });
                }
            }
        }
        conflicts
    }

    fn resolve_conflicts<F>(&self, other: &Self, resolver: F) -> TraitConfigResult<Self>
    where
        F: Fn(&super::core::MergeConflict) -> super::core::ConflictResolution,
    {
        let conflicts = self.check_merge_conflicts(other);
        let mut merged = self.clone();

        for conflict in conflicts {
            match resolver(&conflict) {
                super::core::ConflictResolution::UseCurrent => {
                    // Keep current value - no action needed
                }
                super::core::ConflictResolution::UseOther => {
                    merged
                        .data
                        .insert(conflict.field_path, conflict.other_value);
                }
                super::core::ConflictResolution::Custom(value) => {
                    merged.data.insert(conflict.field_path, value);
                }
                super::core::ConflictResolution::Skip => {
                    merged.data.remove(&conflict.field_path);
                }
                super::core::ConflictResolution::Merge => {
                    // Simple merge implementation
                    merged
                        .data
                        .insert(conflict.field_path, conflict.other_value);
                }
            }
        }

        merged.metadata.updated_at = Utc::now();
        Ok(merged)
    }
}

#[async_trait]
impl ConfigSerialization for MockConfig {
    async fn serialize_to_format(&self, format: SerializationFormat) -> TraitConfigResult<String> {
        match format {
            SerializationFormat::Json => serde_json::to_string_pretty(self)
                .map_err(|e| TraitConfigError::serialization(e.to_string())),
            SerializationFormat::Toml => toml::to_string_pretty(self)
                .map_err(|e| TraitConfigError::serialization(e.to_string())),
            _ => Err(TraitConfigError::not_implemented(
                "MockConfig",
                "serialize_to_format",
            )),
        }
    }

    async fn deserialize_from_format(
        data: &str,
        format: SerializationFormat,
    ) -> TraitConfigResult<Self> {
        match format {
            SerializationFormat::Json => serde_json::from_str(data)
                .map_err(|e| TraitConfigError::serialization(e.to_string())),
            SerializationFormat::Toml => {
                toml::from_str(data).map_err(|e| TraitConfigError::serialization(e.to_string()))
            }
            _ => Err(TraitConfigError::not_implemented(
                "MockConfig",
                "deserialize_from_format",
            )),
        }
    }

    async fn serialize_to_file(&self, path: &Path) -> TraitConfigResult<()> {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("json");

        let format = SerializationFormat::from_extension(ext).unwrap_or(SerializationFormat::Json);

        let content = self.serialize_to_format(format).await?;
        tokio::fs::write(path, content)
            .await
            .map_err(|e| TraitConfigError::serialization(e.to_string()))?;

        Ok(())
    }

    async fn deserialize_from_file(path: &Path) -> TraitConfigResult<Self> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| TraitConfigError::serialization(e.to_string()))?;

        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("json");

        let format = SerializationFormat::from_extension(ext).unwrap_or(SerializationFormat::Json);

        Self::deserialize_from_format(&content, format).await
    }

    fn supported_formats(&self) -> Vec<SerializationFormat> {
        vec![SerializationFormat::Json, SerializationFormat::Toml]
    }

    async fn validate_serialized_data(
        data: &str,
        format: SerializationFormat,
    ) -> TraitConfigResult<()> {
        match format {
            SerializationFormat::Json => {
                serde_json::from_str::<serde_json::Value>(data)
                    .map_err(|e| TraitConfigError::serialization(e.to_string()))?;
            }
            SerializationFormat::Toml => {
                toml::from_str::<toml::Value>(data)
                    .map_err(|e| TraitConfigError::serialization(e.to_string()))?;
            }
            _ => {
                return Err(TraitConfigError::not_implemented(
                    "MockConfig",
                    "validate_serialized_data",
                ))
            }
        }
        Ok(())
    }
}

impl ConfigTestHelper {
    /// Create a mock configuration for testing
    pub fn create_mock_config() -> MockConfig {
        MockConfig::with_test_data()
    }

    /// Create a mock configuration with validation failure
    pub fn create_failing_config() -> MockConfig {
        let mut config = MockConfig::new();
        config.set_validation_fail();
        config
    }

    /// Create a configuration with specific test data
    pub fn create_config_with_data(data: HashMap<String, ConfigValue>) -> MockConfig {
        let mut config = MockConfig::new();
        config.data = data;
        config
    }

    /// Assert that two configurations are equivalent
    pub fn assert_configs_equal(config1: &MockConfig, config2: &MockConfig) {
        assert_eq!(config1.data, config2.data);
    }

    /// Assert that a configuration is valid
    pub async fn assert_config_valid(config: &MockConfig) {
        assert!(config.validate().is_ok());
    }

    /// Test configuration lifecycle operations
    pub async fn test_lifecycle(mut config: MockConfig) -> Result<(), ConfigError> {
        let temp_path = std::env::temp_dir().join("test_config.json");

        // Test save
        config.save(&temp_path).await?;

        // Test reload
        config.reload(&temp_path).await?;

        // Test change detection
        let changed = config.has_changed(&temp_path).await?;
        assert!(!changed);

        // Cleanup
        let _ = std::fs::remove_file(&temp_path);

        Ok(())
    }
}

impl TraitCompositionValidator {
    /// Validate that a configuration implements required traits correctly
    pub fn validate_base_config<T>(_config: &T) -> Result<(), String>
    where
        T: BaseConfig,
    {
        // Mock validation - in a real implementation, this would check
        // that the configuration implements all required methods correctly
        Ok(())
    }

    /// Validate trait composition for multiple traits
    pub fn validate_composition<T>(_config: &T) -> Result<(), String>
    where
        T: BaseConfig + ConfigLifecycle,
    {
        // Mock validation for trait composition
        Ok(())
    }
}

impl MockPlatformPaths {
    pub fn new() -> Self {
        Self {
            base_dir: std::env::temp_dir().join("datafold_test"),
        }
    }
}

impl PlatformConfigPaths for MockPlatformPaths {
    fn config_dir(&self) -> Result<std::path::PathBuf, ConfigError> {
        Ok(self.base_dir.join("config"))
    }

    fn data_dir(&self) -> Result<std::path::PathBuf, ConfigError> {
        Ok(self.base_dir.join("data"))
    }

    fn cache_dir(&self) -> Result<std::path::PathBuf, ConfigError> {
        Ok(self.base_dir.join("cache"))
    }

    fn logs_dir(&self) -> Result<std::path::PathBuf, ConfigError> {
        Ok(self.base_dir.join("logs"))
    }

    fn runtime_dir(&self) -> Result<std::path::PathBuf, ConfigError> {
        Ok(self.base_dir.join("runtime"))
    }

    fn platform_name(&self) -> &'static str {
        "mock"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_config_creation() {
        let config = MockConfig::new();
        assert!(config.data.is_empty());
        assert!(config.mock_state.validation_should_succeed);
    }

    #[tokio::test]
    async fn test_mock_config_with_data() {
        let config = MockConfig::with_test_data();
        assert!(!config.data.is_empty());
        assert!(config.data.contains_key("test_string"));
        assert!(config.data.contains_key("test_number"));
        assert!(config.data.contains_key("test_bool"));
    }

    #[tokio::test]
    async fn test_config_validation() {
        let config = MockConfig::with_test_data();
        assert!(config.validate().is_ok());

        let mut failing_config = MockConfig::new();
        failing_config.set_validation_fail();
        assert!(failing_config.validate().is_err());
    }

    #[tokio::test]
    async fn test_config_lifecycle() {
        let config = MockConfig::with_test_data();
        let result = ConfigTestHelper::test_lifecycle(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_config_merge() {
        let config1 = MockConfig::with_test_data();
        let mut config2 = MockConfig::new();
        config2
            .data
            .insert("new_field".to_string(), ConfigValue::string("new_value"));

        let merged = config1.merge_default(&config2).unwrap();
        assert!(merged.data.contains_key("test_string"));
        assert!(merged.data.contains_key("new_field"));
    }

    #[tokio::test]
    async fn test_serialization() {
        let config = MockConfig::with_test_data();

        // Test JSON serialization
        let json_result = config.serialize_to_format(SerializationFormat::Json).await;
        assert!(json_result.is_ok());

        // Test TOML serialization
        let toml_result = config.serialize_to_format(SerializationFormat::Toml).await;
        assert!(toml_result.is_ok());
    }

    #[test]
    fn test_mock_platform_paths() {
        let paths = MockPlatformPaths::new();
        assert!(paths.config_dir().is_ok());
        assert!(paths.data_dir().is_ok());
        assert!(paths.cache_dir().is_ok());
        assert_eq!(paths.platform_name(), "mock");
    }

    #[test]
    fn test_trait_composition_validator() {
        let config = MockConfig::with_test_data();
        assert!(TraitCompositionValidator::validate_base_config(&config).is_ok());
        assert!(TraitCompositionValidator::validate_composition(&config).is_ok());
    }

    #[test]
    fn test_config_test_helper() {
        let config = ConfigTestHelper::create_mock_config();
        assert!(!config.data.is_empty());

        let failing_config = ConfigTestHelper::create_failing_config();
        assert!(!failing_config.mock_state.validation_should_succeed);

        let mut data = HashMap::new();
        data.insert("custom".to_string(), ConfigValue::string("value"));
        let custom_config = ConfigTestHelper::create_config_with_data(data);
        assert!(custom_config.data.contains_key("custom"));
    }
}
