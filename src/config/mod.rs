//! Configuration module for DataFold
//!
//! This module provides unified configuration management across all DataFold components,
//! including cryptographic initialization, database encryption settings, and
//! cross-platform configuration management.
//!
//! # Core Features
//!
//! - **Cross-platform configuration management** with platform-specific path resolution
//! - **Format-agnostic configuration handling** with TOML as the primary format
//! - **Configuration validation and type safety** through Rust's type system
//! - **Async/await support** for all IO operations
//! - **Configuration merging and override capabilities**
//! - **Legacy configuration migration** from JSON to TOML
//! - **Event emission for configuration changes** (foundation for unified reporting)
//! - **Security hooks** for encryption and access control
//!
//! # Example Usage
//!
//! ```rust
//! use datafold::config::{ConfigurationManager, Config, ConfigValue};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create configuration manager
//!     let manager = ConfigurationManager::new();
//!
//!     // Get current configuration (creates default if none exists)
//!     let config = manager.get().await?;
//!
//!     // Access configuration values
//!     if let Ok(logging_section) = config.get_section("logging") {
//!         println!("Logging configuration: {}", logging_section);
//!     }
//!
//!     // Create and save new configuration
//!     let mut new_config = Config::new();
//!     new_config.set_section("app".to_string(), ConfigValue::object({
//!         let mut app_config = std::collections::HashMap::new();
//!         app_config.insert("name".to_string(), ConfigValue::string("datafold"));
//!         app_config.insert("version".to_string(), ConfigValue::string("1.0.0"));
//!         app_config
//!     }));
//!
//!     manager.set(new_config).await?;
//!
//!     Ok(())
//! }
//! ```

// Core configuration management
pub mod cross_platform;
pub mod enhanced;
pub mod error;
pub mod migration;
pub mod platform;
pub mod value;

// Shared configuration traits (PBI 28)
pub mod traits;

// Legacy modules (maintained for backward compatibility)
pub mod crypto;
pub mod unified_config;

// Test modules
#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use cross_platform::{
    Config, ConfigurationManager, ConfigurationProvider, PlatformRequirements, SecurityConfig,
    TomlConfigProvider,
};
pub use enhanced::{
    ConfigChangeEvent, ConfigChangeSource, ConfigChangeType, EnhancedConfig,
    EnhancedConfigurationManager, EnhancedSecurityConfig, PerformanceSettings, PlatformSettings,
};
pub use error::{ConfigError, ConfigResult};
pub use migration::{ConfigMigrationManager, MigrationResult, MigrationStrategy};
pub use platform::{
    create_platform_resolver, get_platform_info,
    keystore::{create_platform_keystore, KeystoreConfig, PlatformKeystore},
    EnhancedPlatformInfo, PlatformConfigPaths, PlatformInfo,
};
pub use value::{ConfigValue, ConfigValueSchema};

// Re-export shared configuration traits (PBI 28)
pub use traits::{
    // Base traits
    BaseConfig,
    ConfigEvents,
    ConfigLifecycle,
    // Core utility traits
    ConfigMerge,
    ConfigMetadataTrait,
    ConfigReporting,
    ConfigSerialization,
    ConfigValidation,
    // Integration traits
    CrossPlatformConfig,
    ErrorContext,
    // Note: Dynamic trait objects for BaseConfig are not object-safe due to generic methods
    // Users should use concrete types or define their own trait objects as needed
    ObservableConfig,
    ReportableConfig,
    // Error types
    TraitConfigError,
    TraitConfigResult,
    ValidatableConfig,
    ValidationContext,
};

// Re-export legacy types for backward compatibility
pub use unified_config::{
    AuthenticationConfig, DefaultConfig, EnvironmentConfig, LoggingConfig, PerformanceConfig,
    SecurityProfile, SigningConfig, UnifiedConfig, UnifiedConfigError, UnifiedConfigManager,
    UnifiedConfigResult, VerificationConfig,
};
