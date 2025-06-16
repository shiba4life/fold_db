//! Example implementation of the shared configuration traits
//!
//! This demonstrates how to implement the core shared configuration traits
//! for a concrete configuration type.

use std::any::Any;
use std::path::Path;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use super::base::{BaseConfig, ConfigLifecycle, ConfigValidation, ConfigMetadata};
use super::core::{ConfigMerge, ConfigSerialization, ConfigMetadataTrait, ConfigEvents, ConfigChangeEvent, MergeStrategy, SerializationFormat, ConfigChangeEvent as CoreConfigChangeEvent, ConfigEventType, MergeConflict, ConfigHistoryEntry, ConfigDiff};
use super::integration::{CrossPlatformConfig, ReportableConfig, ValidatableConfig, ObservableConfig};
use super::error::{TraitConfigError, TraitConfigResult};

/// Example configuration implementing all shared traits
#[derive(Debug, Clone)]
pub struct ExampleConfig {
    pub name: String,
    pub value: i32,
    pub enabled: bool,
    pub metadata: ConfigMetadata,
}

/// Example error type
#[derive(Debug, thiserror::Error)]
pub enum ExampleError {
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Example event type
#[derive(Debug, Clone)]
pub struct ExampleEvent {
    pub timestamp: DateTime<Utc>,
    pub message: String,
}

#[async_trait]
impl BaseConfig for ExampleConfig {
    type Error = ExampleError;
    type Event = ExampleEvent;
    type TransformTarget = ();

    async fn load(_path: &Path) -> Result<Self, Self::Error> {
        Ok(ExampleConfig {
            name: "example".to_string(),
            value: 42,
            enabled: true,
            metadata: ConfigMetadata::default(),
        })
    }

    fn validate(&self) -> Result<(), Self::Error> {
        if self.name.is_empty() {
            return Err(ExampleError::Validation("Name cannot be empty".to_string()));
        }
        Ok(())
    }

    fn report_event(&self, _event: Self::Event) {
        // Implementation would report to monitoring system
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ConfigMerge for ExampleConfig {
    fn merge_with_strategy(&self, other: &Self, _strategy: MergeStrategy) -> TraitConfigResult<Self> {
        Ok(ExampleConfig {
            name: if other.name.is_empty() { self.name.clone() } else { other.name.clone() },
            value: other.value,
            enabled: other.enabled,
            metadata: other.metadata.clone(),
        })
    }

    fn deep_merge(&self, other: &Self) -> TraitConfigResult<Self> {
        self.merge_with_strategy(other, MergeStrategy::DeepMerge)
    }

    fn merge_sections(&self, other: &Self, _sections: &[&str]) -> TraitConfigResult<Self> {
        self.merge_with_strategy(other, MergeStrategy::Replace)
    }

    fn check_merge_conflicts(&self, _other: &Self) -> Vec<MergeConflict> {
        Vec::new()
    }

    fn resolve_conflicts<F>(&self, other: &Self, _resolver: F) -> TraitConfigResult<Self>
    where
        Self: Sized,
        F: Fn(&MergeConflict) -> super::core::ConflictResolution,
    {
        self.merge_with_strategy(other, MergeStrategy::Replace)
    }
}

#[async_trait]
impl ConfigSerialization for ExampleConfig {
    async fn serialize_to_format(&self, _format: SerializationFormat) -> TraitConfigResult<Vec<u8>> {
        // Mock serialization
        Ok(format!("name={},value={},enabled={}", self.name, self.value, self.enabled).into_bytes())
    }

    async fn deserialize_from_format(_data: &[u8], _format: SerializationFormat) -> TraitConfigResult<Self> {
        // Mock deserialization
        Ok(ExampleConfig {
            name: "deserialized".to_string(),
            value: 0,
            enabled: false,
            metadata: ConfigMetadata::default(),
        })
    }

    async fn auto_detect_format(_data: &[u8]) -> TraitConfigResult<SerializationFormat> {
        Ok(SerializationFormat::Json)
    }

    fn supported_formats(&self) -> Vec<SerializationFormat> {
        vec![SerializationFormat::Json, SerializationFormat::Toml]
    }

    fn calculate_checksum(&self) -> String {
        "mock_checksum".to_string()
    }

    fn verify_integrity(&self) -> TraitConfigResult<bool> {
        Ok(true)
    }

    fn history(&self) -> Vec<ConfigHistoryEntry> {
        Vec::new()
    }

    fn add_history_entry(&mut self, _entry: ConfigHistoryEntry) {
        // Mock implementation
    }

    fn diff(&self, _other: &Self) -> ConfigDiff {
        ConfigDiff {
            added_fields: Vec::new(),
            removed_fields: Vec::new(),
            changed_fields: Vec::new(),
            unchanged_fields: Vec::new(),
        }
    }
}

impl ConfigMetadataTrait for ExampleConfig {
    fn get_metadata(&self) -> &ConfigMetadata {
        &self.metadata
    }

    fn set_metadata(&mut self, metadata: ConfigMetadata) {
        self.metadata = metadata;
    }

    fn update_metadata_field(&mut self, key: &str, value: String) {
        self.metadata.additional.insert(key.to_string(), value);
    }

    fn get_metadata_field(&self, key: &str) -> Option<&String> {
        self.metadata.additional.get(key)
    }

    fn get_version(&self) -> &str {
        &self.metadata.version
    }

    fn set_version(&mut self, version: String) {
        self.metadata.version = version;
    }

    fn get_created_at(&self) -> Option<DateTime<Utc>> {
        self.metadata.created_at
    }

    fn get_updated_at(&self) -> Option<DateTime<Utc>> {
        self.metadata.updated_at
    }

    fn touch(&mut self) {
        self.metadata.updated_at = Some(Utc::now());
    }
}

#[async_trait]
impl ConfigEvents for ExampleConfig {
    async fn on_change<F>(&mut self, _listener: F) -> TraitConfigResult<super::core::ListenerId>
    where
        F: Fn(&CoreConfigChangeEvent) + Send + Sync + 'static,
    {
        Ok(super::core::ListenerId(1))
    }

    async fn remove_listener(&mut self, _listener_id: super::core::ListenerId) -> TraitConfigResult<()> {
        Ok(())
    }

    async fn emit_change(&self, _event: CoreConfigChangeEvent) -> TraitConfigResult<()> {
        Ok(())
    }

    fn event_history(&self) -> Vec<CoreConfigChangeEvent> {
        Vec::new()
    }

    fn clear_event_history(&mut self) {
        // Mock implementation
    }

    fn set_event_logging(&mut self, _enabled: bool) {
        // Mock implementation
    }

    async fn batch_events<F, R>(&mut self, operation: F) -> TraitConfigResult<R>
    where
        F: FnOnce(&mut Self) -> TraitConfigResult<R> + Send,
        R: Send,
    {
        operation(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_example_config_basic_functionality() {
        let config = ExampleConfig::load(Path::new("test")).await.unwrap();
        assert_eq!(config.name, "example");
        assert_eq!(config.value, 42);
        assert!(config.enabled);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_merge() {
        let config1 = ExampleConfig {
            name: "config1".to_string(),
            value: 1,
            enabled: true,
            metadata: ConfigMetadata::default(),
        };

        let config2 = ExampleConfig {
            name: "config2".to_string(),
            value: 2,
            enabled: false,
            metadata: ConfigMetadata::default(),
        };

        let merged = config1.merge_default(&config2).unwrap();
        assert_eq!(merged.name, "config2");
        assert_eq!(merged.value, 2);
        assert!(!merged.enabled);
    }

    #[tokio::test]
    async fn test_config_serialization() {
        let config = ExampleConfig {
            name: "test".to_string(),
            value: 123,
            enabled: true,
            metadata: ConfigMetadata::default(),
        };

        let data = config.serialize_to_format(SerializationFormat::Json).await.unwrap();
        assert!(!data.is_empty());

        let formats = config.supported_formats();
        assert!(formats.contains(&SerializationFormat::Json));
        assert!(formats.contains(&SerializationFormat::Toml));
    }

    #[test]
    fn test_config_metadata() {
        let mut config = ExampleConfig {
            name: "test".to_string(),
            value: 123,
            enabled: true,
            metadata: ConfigMetadata::default(),
        };

        assert_eq!(config.get_version(), "1.0.0");
        
        config.set_version("2.0.0".to_string());
        assert_eq!(config.get_version(), "2.0.0");

        config.update_metadata_field("environment", "test".to_string());
        assert_eq!(config.get_metadata_field("environment"), Some(&"test".to_string()));
    }

    #[tokio::test]
    async fn test_config_events() {
        let mut config = ExampleConfig {
            name: "test".to_string(),
            value: 123,
            enabled: true,
            metadata: ConfigMetadata::default(),
        };

        let listener_id = config.on_change(|_event| {
            // Mock event handler
        }).await.unwrap();

        assert!(config.remove_listener(listener_id).await.is_ok());

        let test_event = CoreConfigChangeEvent {
            timestamp: Utc::now(),
            event_type: ConfigEventType::Loaded,
            description: "Test event".to_string(),
            field_path: None,
            old_value: None,
            new_value: None,
            metadata: std::collections::HashMap::new(),
        };

        assert!(config.emit_change(test_event).await.is_ok());
    }
}