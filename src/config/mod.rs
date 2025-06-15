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
pub mod error;
pub mod value;
pub mod platform;
pub mod enhanced;
pub mod migration;

// Legacy modules (maintained for backward compatibility)
pub mod crypto;
pub mod unified_config;

// Test modules
#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use cross_platform::{
    Config,
    ConfigurationManager,
    ConfigurationProvider,
    TomlConfigProvider,
    PlatformRequirements,
    SecurityConfig,
};
pub use error::{ConfigError, ConfigResult};
pub use value::{ConfigValue, ConfigValueSchema};
pub use platform::{
    PlatformConfigPaths,
    PlatformInfo,
    EnhancedPlatformInfo,
    create_platform_resolver,
    get_platform_info,
    keystore::{PlatformKeystore, create_platform_keystore, KeystoreConfig},
};
pub use enhanced::{
    EnhancedConfig,
    EnhancedConfigurationManager,
    PlatformSettings,
    PerformanceSettings,
    EnhancedSecurityConfig,
    ConfigChangeEvent,
    ConfigChangeType,
    ConfigChangeSource,
};
pub use migration::{
    ConfigMigrationManager,
    MigrationResult,
    MigrationStrategy,
};

// Re-export legacy types for backward compatibility
pub use unified_config::{
    UnifiedConfig,
    UnifiedConfigManager,
    UnifiedConfigError,
    UnifiedConfigResult,
    EnvironmentConfig,
    SigningConfig,
    VerificationConfig,
    LoggingConfig,
    AuthenticationConfig,
    PerformanceConfig,
    SecurityProfile,
    DefaultConfig,
};
