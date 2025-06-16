//! Core utility traits for configuration management
//!
//! This module provides essential utility traits that extend the base configuration
//! functionality with merging, serialization, metadata management, and event handling.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::base::{BaseConfig, ConfigMetadata as BaseConfigMetadata};
use super::error::{TraitConfigError, TraitConfigResult};
use crate::config::error::ConfigError;
use crate::config::value::ConfigValue;

/// Configuration merging and override patterns
///
/// Provides standardized configuration merging capabilities with conflict resolution
/// and merge strategy customization.
pub trait ConfigMerge: BaseConfig {
    /// Merge configuration with a specific merge strategy
    ///
    /// Allows customization of merge behavior for different use cases.
    fn merge_with_strategy(&self, other: &Self, strategy: MergeStrategy) -> TraitConfigResult<Self>
    where
        Self: Sized;

    /// Merge configuration with default strategy
    ///
    /// The default merge strategy is that the `other` configuration takes
    /// precedence over `self` for conflicting values.
    fn merge_default(&self, other: &Self) -> TraitConfigResult<Self>
    where
        Self: Sized,
    {
        self.merge_with_strategy(other, MergeStrategy::Replace)
    }

    /// Deep merge configuration preserving nested structures
    ///
    /// Recursively merges nested configuration objects rather than replacing them.
    fn deep_merge(&self, other: &Self) -> TraitConfigResult<Self>
    where
        Self: Sized;

    /// Merge configuration section by section
    ///
    /// Provides fine-grained control over which sections are merged.
    fn merge_sections(&self, other: &Self, sections: &[&str]) -> TraitConfigResult<Self>
    where
        Self: Sized;

    /// Check for merge conflicts without performing the merge
    ///
    /// Returns a list of potential conflicts that would occur during merge.
    fn check_merge_conflicts(&self, other: &Self) -> Vec<MergeConflict>;

    /// Resolve merge conflicts interactively
    ///
    /// Provides a callback-based mechanism for resolving merge conflicts.
    fn resolve_conflicts<F>(&self, other: &Self, resolver: F) -> TraitConfigResult<Self>
    where
        Self: Sized,
        F: Fn(&MergeConflict) -> ConflictResolution;
}

/// Format-agnostic configuration serialization
///
/// Provides standardized serialization capabilities across different configuration formats.
#[async_trait]
pub trait ConfigSerialization: BaseConfig {
    /// Serialize configuration to the specified format
    ///
    /// Supports multiple output formats with automatic format detection
    /// based on file extension or explicit format specification.
    async fn serialize_to_format(&self, format: SerializationFormat) -> TraitConfigResult<String>;

    /// Deserialize configuration from the specified format
    ///
    /// Parses configuration data from various formats into the configuration type.
    async fn deserialize_from_format(
        data: &str,
        format: SerializationFormat,
    ) -> TraitConfigResult<Self>
    where
        Self: Sized;

    /// Serialize to file with automatic format detection
    ///
    /// Determines output format based on file extension.
    async fn serialize_to_file(&self, path: &Path) -> TraitConfigResult<()>;

    /// Deserialize from file with automatic format detection
    ///
    /// Determines input format based on file extension.
    async fn deserialize_from_file(path: &Path) -> TraitConfigResult<Self>
    where
        Self: Sized;

    /// Get supported serialization formats
    ///
    /// Returns the list of formats supported by this configuration type.
    fn supported_formats(&self) -> Vec<SerializationFormat>;

    /// Validate serialized data without full deserialization
    ///
    /// Performs lightweight validation of serialized configuration data.
    async fn validate_serialized_data(
        data: &str,
        format: SerializationFormat,
    ) -> TraitConfigResult<()>;
}

/// Configuration metadata and versioning management
///
/// Provides comprehensive metadata management including versioning,
/// change tracking, and configuration history.
pub trait ConfigMetadataTrait: BaseConfig {
    /// Get configuration metadata
    ///
    /// Returns comprehensive metadata about the configuration instance.
    fn get_metadata(&self) -> &BaseConfigMetadata;

    /// Set configuration metadata
    ///
    /// Updates metadata fields with new values.
    fn set_metadata(&mut self, metadata: BaseConfigMetadata);

    /// Update metadata timestamp
    ///
    /// Updates the last modified timestamp to the current time.
    fn touch(&mut self);

    /// Get configuration version
    ///
    /// Returns the semantic version of the configuration.
    fn version(&self) -> &str;

    /// Set configuration version
    ///
    /// Updates the configuration version, typically done during upgrades.
    fn set_version(&mut self, version: String);

    /// Get configuration checksum
    ///
    /// Computes and returns a checksum for integrity verification.
    fn checksum(&self) -> TraitConfigResult<String>;

    /// Verify configuration integrity
    ///
    /// Validates configuration integrity using stored checksum.
    fn verify_integrity(&self) -> TraitConfigResult<bool>;

    /// Get configuration history
    ///
    /// Returns the change history for this configuration.
    fn history(&self) -> Vec<ConfigHistoryEntry>;

    /// Add configuration history entry
    ///
    /// Records a change in the configuration history.
    fn add_history_entry(&mut self, entry: ConfigHistoryEntry);

    /// Compare configurations and generate diff
    ///
    /// Creates a structured diff between this and another configuration.
    fn diff(&self, other: &Self) -> ConfigDiff;
}

/// Configuration change notification and event emission
///
/// Provides event-driven configuration management with change notifications
/// and lifecycle event handling.
#[async_trait]
pub trait ConfigEvents: BaseConfig {
    /// Register change event listener
    ///
    /// Registers a callback to be invoked when configuration changes occur.
    async fn on_change<F>(&mut self, listener: F) -> TraitConfigResult<ListenerId>
    where
        F: Fn(&ConfigChangeEvent) + Send + Sync + 'static;

    /// Unregister change event listener
    ///
    /// Removes a previously registered change listener.
    async fn remove_listener(&mut self, listener_id: ListenerId) -> TraitConfigResult<()>;

    /// Emit configuration change event
    ///
    /// Manually triggers change event notifications.
    async fn emit_change(&self, event: ConfigChangeEvent) -> TraitConfigResult<()>;

    /// Get event history
    ///
    /// Returns the history of events for this configuration.
    fn event_history(&self) -> Vec<ConfigChangeEvent>;

    /// Clear event history
    ///
    /// Removes all stored event history.
    fn clear_event_history(&mut self);

    /// Enable/disable event logging
    ///
    /// Controls whether events are logged to the event history.
    fn set_event_logging(&mut self, enabled: bool);

    /// Batch event operations
    ///
    /// Groups multiple configuration changes into a single event batch.
    async fn batch_events<F, R>(&mut self, operation: F) -> TraitConfigResult<R>
    where
        F: FnOnce(&mut Self) -> TraitConfigResult<R> + Send,
        R: Send;
}

/// Merge strategies for configuration composition
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeStrategy {
    /// Replace conflicting values with values from other config
    Replace,

    /// Keep existing values, ignore conflicts
    Keep,

    /// Deep merge nested objects
    DeepMerge,

    /// Fail on any conflicts
    FailOnConflict,

    /// Custom merge logic
    Custom(String),
}

/// Represents a merge conflict between configurations
#[derive(Debug, Clone)]
pub struct MergeConflict {
    /// Path to the conflicting field
    pub field_path: String,

    /// Value in the current configuration
    pub current_value: ConfigValue,

    /// Value in the other configuration
    pub other_value: ConfigValue,

    /// Conflict type
    pub conflict_type: ConflictType,
}

/// Types of merge conflicts
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictType {
    /// Values are different
    ValueMismatch,

    /// Types are incompatible
    TypeMismatch,

    /// Field exists in one config but not the other
    FieldMissing,

    /// Nested structure conflicts
    StructuralMismatch,
}

/// Resolution for merge conflicts
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictResolution {
    /// Use the current value
    UseCurrent,

    /// Use the other value
    UseOther,

    /// Merge the values (if possible)
    Merge,

    /// Skip this field
    Skip,

    /// Use custom value
    Custom(ConfigValue),
}

/// Supported serialization formats
#[derive(Debug, Clone, PartialEq)]
pub enum SerializationFormat {
    /// TOML format
    Toml,

    /// JSON format
    Json,

    /// YAML format
    Yaml,

    /// XML format
    Xml,

    /// Binary format
    Binary,

    /// Custom format
    Custom(String),
}

/// Configuration history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigHistoryEntry {
    /// Entry timestamp
    pub timestamp: DateTime<Utc>,

    /// Change description
    pub description: String,

    /// User or system that made the change
    pub changed_by: Option<String>,

    /// Change type
    pub change_type: HistoryChangeType,

    /// Affected fields
    pub affected_fields: Vec<String>,

    /// Previous values (for rollback)
    pub previous_values: Option<HashMap<String, ConfigValue>>,
}

/// Types of configuration changes for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HistoryChangeType {
    /// Configuration was created
    Created,

    /// Configuration was loaded
    Loaded,

    /// Configuration was saved
    Saved,

    /// Field value was modified
    Modified,

    /// Field was added
    Added,

    /// Field was removed
    Removed,

    /// Configuration was merged
    Merged,

    /// Configuration was validated
    Validated,

    /// Configuration was migrated
    Migrated,
}

/// Configuration change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChangeEvent {
    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Event type
    pub event_type: ConfigEventType,

    /// Affected configuration path
    pub config_path: Option<String>,

    /// Changed fields
    pub changed_fields: Vec<String>,

    /// Event source
    pub source: String,

    /// Additional event data
    pub data: HashMap<String, ConfigValue>,
}

/// Types of configuration events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigEventType {
    /// Configuration was loaded
    Loaded,

    /// Configuration was saved
    Saved,

    /// Configuration was reloaded
    Reloaded,

    /// Configuration was validated
    Validated,

    /// Configuration was merged
    Merged,

    /// Field was changed
    FieldChanged,

    /// Error occurred
    Error,

    /// Custom event
    Custom(String),
}

/// Configuration diff representation
#[derive(Debug, Clone)]
pub struct ConfigDiff {
    /// Fields that were added
    pub added: HashMap<String, ConfigValue>,

    /// Fields that were removed
    pub removed: HashMap<String, ConfigValue>,

    /// Fields that were modified
    pub modified: HashMap<String, (ConfigValue, ConfigValue)>, // (old, new)

    /// Unchanged fields
    pub unchanged: HashMap<String, ConfigValue>,
}

/// Unique identifier for event listeners
pub type ListenerId = uuid::Uuid;

impl SerializationFormat {
    /// Get file extension for this format
    pub fn file_extension(&self) -> &str {
        match self {
            SerializationFormat::Toml => "toml",
            SerializationFormat::Json => "json",
            SerializationFormat::Yaml => "yaml",
            SerializationFormat::Xml => "xml",
            SerializationFormat::Binary => "bin",
            SerializationFormat::Custom(ext) => ext,
        }
    }

    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "toml" => Some(SerializationFormat::Toml),
            "json" => Some(SerializationFormat::Json),
            "yaml" | "yml" => Some(SerializationFormat::Yaml),
            "xml" => Some(SerializationFormat::Xml),
            "bin" => Some(SerializationFormat::Binary),
            _ => None,
        }
    }
}

impl ConfigHistoryEntry {
    /// Create a new history entry
    pub fn new(description: String, change_type: HistoryChangeType) -> Self {
        Self {
            timestamp: Utc::now(),
            description,
            changed_by: None,
            change_type,
            affected_fields: Vec::new(),
            previous_values: None,
        }
    }

    /// Add affected field
    pub fn with_field(mut self, field: String) -> Self {
        self.affected_fields.push(field);
        self
    }

    /// Set who made the change
    pub fn with_changed_by(mut self, changed_by: String) -> Self {
        self.changed_by = Some(changed_by);
        self
    }

    /// Set previous values for rollback
    pub fn with_previous_values(mut self, values: HashMap<String, ConfigValue>) -> Self {
        self.previous_values = Some(values);
        self
    }
}

impl ConfigChangeEvent {
    /// Create a new change event
    pub fn new(event_type: ConfigEventType, source: String) -> Self {
        Self {
            timestamp: Utc::now(),
            event_type,
            config_path: None,
            changed_fields: Vec::new(),
            source,
            data: HashMap::new(),
        }
    }

    /// Add changed field
    pub fn with_field(mut self, field: String) -> Self {
        self.changed_fields.push(field);
        self
    }

    /// Set configuration path
    pub fn with_path(mut self, path: String) -> Self {
        self.config_path = Some(path);
        self
    }

    /// Add event data
    pub fn with_data(mut self, key: String, value: ConfigValue) -> Self {
        self.data.insert(key, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization_format_extension() {
        assert_eq!(SerializationFormat::Toml.file_extension(), "toml");
        assert_eq!(SerializationFormat::Json.file_extension(), "json");
        assert_eq!(SerializationFormat::Yaml.file_extension(), "yaml");
    }

    #[test]
    fn test_format_from_extension() {
        assert_eq!(
            SerializationFormat::from_extension("toml"),
            Some(SerializationFormat::Toml)
        );
        assert_eq!(
            SerializationFormat::from_extension("json"),
            Some(SerializationFormat::Json)
        );
        assert_eq!(SerializationFormat::from_extension("unknown"), None);
    }

    #[test]
    fn test_merge_strategy() {
        let strategy = MergeStrategy::DeepMerge;
        assert_eq!(strategy, MergeStrategy::DeepMerge);
        assert_ne!(strategy, MergeStrategy::Replace);
    }

    #[test]
    fn test_conflict_type() {
        let conflict = ConflictType::ValueMismatch;
        assert_eq!(conflict, ConflictType::ValueMismatch);
        assert_ne!(conflict, ConflictType::TypeMismatch);
    }

    #[test]
    fn test_history_entry_creation() {
        let entry = ConfigHistoryEntry::new("Test change".to_string(), HistoryChangeType::Modified)
            .with_field("test.field".to_string())
            .with_changed_by("test_user".to_string());

        assert_eq!(entry.description, "Test change");
        assert_eq!(entry.affected_fields, vec!["test.field"]);
        assert_eq!(entry.changed_by, Some("test_user".to_string()));
    }

    #[test]
    fn test_change_event_creation() {
        let event =
            ConfigChangeEvent::new(ConfigEventType::FieldChanged, "test_source".to_string())
                .with_field("test.field".to_string())
                .with_path("/test/config".to_string());

        assert_eq!(event.source, "test_source");
        assert_eq!(event.changed_fields, vec!["test.field"]);
        assert_eq!(event.config_path, Some("/test/config".to_string()));
    }
}
