//! Shared configuration trait hierarchy for DataFold
//!
//! This module provides the foundational trait hierarchy for all configuration types,
//! implementing the shared configuration trait design from PBI 28. These traits enable:
//!
//! - **Unified configuration lifecycle management** with standardized load, validate, merge operations
//! - **Cross-platform compatibility** with platform-specific optimizations
//! - **Integration with PBI 26/27** for unified reporting and cross-platform configuration
//! - **Type-safe configuration composition** through trait bounds and associated types
//! - **Async/await support** for all I/O operations
//! - **Comprehensive error handling** with trait-specific error contexts
//!
//! # Core Trait Hierarchy
//!
//! ```text
//! BaseConfig (core lifecycle, validation, reporting)
//! ├── ConfigLifecycle (load, save, reload operations)
//! ├── ConfigValidation (validation framework)
//! ├── ConfigReporting (PBI 26 unified reporting integration)
//! └── Domain-specific traits (SecurityConfig, NetworkConfig, etc.) [Task 28-4]
//! ```
//!
//! # Usage Example
//!
//! ```rust
//! use datafold::config::traits::{BaseConfig, ConfigLifecycle, ConfigValidation};
//!
//! #[derive(Debug, Clone)]
//! struct MyConfig {
//!     name: String,
//!     enabled: bool,
//! }
//!
//! impl BaseConfig for MyConfig {
//!     type Error = ConfigError;
//!     type Event = ConfigChangeEvent;
//!     type TransformTarget = ();
//!     
//!     async fn load(path: &std::path::Path) -> Result<Self, Self::Error> {
//!         // Implementation
//!     }
//!     
//!     fn validate(&self) -> Result<(), Self::Error> {
//!         // Implementation
//!     }
//!     
//!     // ... other methods
//! }
//! ```

pub mod base;
pub mod core;
pub mod database;
pub mod error;
pub mod examples;
pub mod ingestion;
pub mod integration;
pub mod logging;
pub mod network;

#[cfg(test)]
pub mod tests;

// Re-export core traits for convenience
pub use base::{
    BaseConfig, ConfigChangeType, ConfigLifecycle, ConfigMetadata, ConfigReporting,
    ConfigValidation, ReportingConfig, ValidationRule, ValidationRuleType, ValidationSeverity,
};

pub use core::{ConfigEvents, ConfigMerge, ConfigMetadataTrait, ConfigSerialization};

pub use integration::{
    ConfigMetrics, ConfigSchema, ConfigSummary, CrossPlatformConfig, HealthIndicator, HealthLevel,
    HealthStatus, ObservableConfig, PlatformPerformanceSettings, ReportableConfig,
    ReportingCapabilities, ReportingRegistration, TraitValidationRule, UnifiedReport,
    ValidatableConfig, ValidationError, ValidationResult, ValidationWarning,
};

pub use error::{ErrorContext, TraitConfigError, TraitConfigResult, ValidationContext};

pub use network::{
    ComplianceStatus, ConnectivityStatus, ConnectivityTestResult, NetworkConfig,
    NetworkHealthMetrics, NetworkPlatformSettings, SecurityComplianceReport, SecurityConfig,
    SecuritySeverity, SecurityStrengthAssessment, SecurityVulnerability,
};

pub use logging::{
    LogFormatConfig, LogLevelTrait, LogRotationConfig, LoggingConfig, OutputConfigTrait,
    PlatformLogSettingsTrait, StandardLogLevel,
};

pub use ingestion::{
    AIServiceConfig, ApiClientConfigTrait, DataProcessingConfig, EnvironmentConfigTrait,
    IngestionConfig, IngestionSecurityConfig, RetryConfigTrait, StandardRetryConfig,
};

pub use database::{
    BackupConfigTrait, BackupMode, ConnectionConfigTrait, DatabaseConfig,
    DatabasePerformanceConfig, EncryptionConfigTrait, StandardBackupConfig,
    StandardConnectionConfig, StandardEncryptionConfig, SyncMode,
};

// Type aliases for common trait object combinations
// Note: These are limited due to object safety constraints of BaseConfig
// pub type DynBaseConfig = dyn BaseConfig<Error = crate::config::error::ConfigError, Event = (), TransformTarget = ()>;
// pub type DynConfigLifecycle = dyn ConfigLifecycle<Error = crate::config::error::ConfigError, Event = (), TransformTarget = ()>;
// pub type DynObservableConfig = dyn ObservableConfig<Error = crate::config::error::ConfigError, Event = (), TransformTarget = ()>;
