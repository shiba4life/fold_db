# Configuration API Reference

**PBI 27: Cross-Platform Configuration Management System**

This document provides a comprehensive API reference for the cross-platform configuration management system, including all public interfaces, methods, and usage examples.

## Table of Contents

1. [Core Traits](#core-traits)
2. [Configuration Types](#configuration-types)
3. [Configuration Managers](#configuration-managers)
4. [Platform Integration](#platform-integration)
5. [Migration Utilities](#migration-utilities)
6. [Error Handling](#error-handling)
7. [Usage Examples](#usage-examples)

## Core Traits

### ConfigurationProvider

The foundational trait for all configuration providers.

```rust
#[async_trait]
pub trait ConfigurationProvider: Send + Sync {
    async fn load(&self) -> ConfigResult<Config>;
    async fn save(&self, config: &Config) -> ConfigResult<()>;
    async fn reload(&self) -> ConfigResult<Config>;
    async fn validate(&self, config: &Config) -> ConfigResult<()>;
    async fn exists(&self) -> ConfigResult<bool>;
    fn config_path(&self) -> ConfigResult<PathBuf>;
    fn provider_type(&self) -> &'static str;
}
```

**Methods:**

- **`load()`** - Load configuration from the provider's source
  - Returns: `ConfigResult<Config>` - The loaded configuration
  - Errors: `ConfigError::NotFound`, `ConfigError::ParseError`, `ConfigError::AccessDenied`

- **`save(config)`** - Save configuration to the provider's destination
  - Parameters: `config: &Config` - Configuration to save
  - Returns: `ConfigResult<()>`
  - Errors: `ConfigError::AccessDenied`, `ConfigError::SerializationError`

- **`reload()`** - Reload configuration, bypassing any caches
  - Returns: `ConfigResult<Config>` - Freshly loaded configuration
  - Errors: Same as `load()`

- **`validate(config)`** - Validate configuration without loading/saving
  - Parameters: `config: &Config` - Configuration to validate
  - Returns: `ConfigResult<()>`
  - Errors: `ConfigError::ValidationError`

- **`exists()`** - Check if configuration file exists
  - Returns: `ConfigResult<bool>` - True if configuration exists
  - Errors: `ConfigError::Platform`

- **`config_path()`** - Get the path to the configuration file
  - Returns: `ConfigResult<PathBuf>` - Path to configuration file
  - Errors: `ConfigError::Platform`

- **`provider_type()`** - Get provider type identifier
  - Returns: `&'static str` - Provider type (e.g., "toml", "json")

**Example:**
```rust
use datafold::config::{ConfigurationProvider, TomlConfigProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = TomlConfigProvider::new();
    
    // Check if config exists
    if provider.exists().await? {
        let config = provider.load().await?;
        println!("Loaded config from: {:?}", provider.config_path()?);
    } else {
        println!("No configuration found");
    }
    
    Ok(())
}
```

### PlatformConfigPaths

Platform-specific path resolution trait.

```rust
pub trait PlatformConfigPaths: Send + Sync {
    fn config_dir(&self) -> ConfigResult<PathBuf>;
    fn data_dir(&self) -> ConfigResult<PathBuf>;
    fn cache_dir(&self) -> ConfigResult<PathBuf>;
    fn logs_dir(&self) -> ConfigResult<PathBuf>;
    fn runtime_dir(&self) -> ConfigResult<PathBuf>;
    fn config_file(&self) -> ConfigResult<PathBuf>;
    fn legacy_config_file(&self) -> ConfigResult<PathBuf>;
    fn platform_name(&self) -> &'static str;
    fn validate_paths(&self) -> ConfigResult<()>;
    fn ensure_directories(&self) -> ConfigResult<()>;
}
```

**Platform-Specific Implementations:**
- **Linux**: [`LinuxConfigPaths`](../../src/config/platform/linux.rs) - XDG Base Directory compliant
- **macOS**: [`MacOSConfigPaths`](../../src/config/platform/macos.rs) - Apple HIG compliant
- **Windows**: [`WindowsConfigPaths`](../../src/config/platform/windows.rs) - Windows conventions

## Configuration Types

### Config

The main configuration structure.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub sections: HashMap<String, ConfigValue>,
    pub metadata: HashMap<String, String>,
}
```

**Methods:**

- **`new()`** - Create a new empty configuration
  ```rust
  let config = Config::new();
  ```

- **`get_section(name)`** - Get a configuration section
  ```rust
  let database_config = config.get_section("database")?;
  ```

- **`get_section_mut(name)`** - Get a mutable reference to a section
  ```rust
  let database_config = config.get_section_mut("database")?;
  ```

- **`set_section(name, value)`** - Set a configuration section
  ```rust
  config.set_section("database".to_string(), ConfigValue::Object(db_settings));
  ```

- **`remove_section(name)`** - Remove a configuration section
  ```rust
  let removed = config.remove_section("deprecated_section");
  ```

- **`merge(other)`** - Merge another configuration into this one
  ```rust
  config.merge(other_config)?;
  ```

- **`get_value(path)`** - Get a nested value using dot notation
  ```rust
  let host = config.get_value("database.host")?;
  ```

### ConfigValue

Type-safe configuration value representation.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigValue {
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
}
```

**Type Conversion Methods:**

- **`as_bool()`** - Get value as boolean
  ```rust
  let enabled = config_value.as_bool()?;
  ```

- **`as_integer()`** - Get value as 64-bit integer
  ```rust
  let port = config_value.as_integer()?;
  ```

- **`as_float()`** - Get value as 64-bit float
  ```rust
  let ratio = config_value.as_float()?;
  ```

- **`as_string()`** - Get value as string reference
  ```rust
  let hostname = config_value.as_string()?;
  ```

- **`as_array()`** - Get value as array reference
  ```rust
  let items = config_value.as_array()?;
  ```

- **`as_object()`** - Get value as object reference
  ```rust
  let settings = config_value.as_object()?;
  ```

**Manipulation Methods:**

- **`get(key)`** - Get nested value by key
  ```rust
  let nested_value = config_value.get("nested_key")?;
  ```

- **`set(key, value)`** - Set nested value
  ```rust
  config_value.set("new_key".to_string(), ConfigValue::String("value".to_string()))?;
  ```

- **`merge(other)`** - Merge with another ConfigValue
  ```rust
  let merged = config_value.merge(other_value)?;
  ```

### EnhancedConfig

Extended configuration with platform-specific features.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedConfig {
    pub base: Config,
    pub platform_settings: PlatformSettings,
    pub performance: PerformanceSettings,
    pub security_enhanced: EnhancedSecurityConfig,
    pub encrypted_sections: HashMap<String, EncryptedSection>,
    pub monitoring: MonitoringConfig,
}
```

**Sub-structures:**

- **`PlatformSettings`** - Platform-specific configuration options
- **`PerformanceSettings`** - Performance tuning parameters
- **`EnhancedSecurityConfig`** - Advanced security settings
- **`EncryptedSection`** - Encrypted configuration data
- **`MonitoringConfig`** - Configuration monitoring settings

## Configuration Managers

### ConfigurationManager

Basic configuration management.

```rust
pub struct ConfigurationManager {
    provider: Arc<dyn ConfigurationProvider>,
    cached_config: Arc<RwLock<Option<Arc<Config>>>>,
}
```

**Methods:**

- **`new()`** - Create manager with default TOML provider
  ```rust
  let manager = ConfigurationManager::new();
  ```

- **`with_provider(provider)`** - Create manager with custom provider
  ```rust
  let provider = Arc::new(TomlConfigProvider::new());
  let manager = ConfigurationManager::with_provider(provider);
  ```

- **`with_toml_file(path)`** - Create manager with specific TOML file
  ```rust
  let manager = ConfigurationManager::with_toml_file("/path/to/config.toml");
  ```

- **`get()`** - Get current configuration (cached or loaded)
  ```rust
  let config = manager.get().await?;
  ```

- **`set(config)`** - Set new configuration and save
  ```rust
  manager.set(new_config).await?;
  ```

- **`reload()`** - Force reload from source
  ```rust
  let config = manager.reload().await?;
  ```

- **`clear_cache()`** - Clear cached configuration
  ```rust
  manager.clear_cache().await;
  ```

### EnhancedConfigurationManager

Advanced configuration management with platform optimizations.

```rust
pub struct EnhancedConfigurationManager {
    base_manager: ConfigurationManager,
    platform_info: EnhancedPlatformInfo,
    keystore: Option<Arc<dyn PlatformKeystore>>,
    file_watcher: Option<Arc<dyn PlatformFileWatcher>>,
    atomic_ops: Arc<dyn PlatformAtomicOps>,
    cache: Arc<Mutex<ConfigCache>>,
    metrics: Arc<Mutex<PerformanceMetrics>>,
}
```

**Methods:**

- **`new()`** - Create enhanced manager with auto-detected platform features
  ```rust
  let manager = EnhancedConfigurationManager::new().await?;
  ```

- **`with_custom(provider, keystore_config, enable_monitoring)`** - Create with custom settings
  ```rust
  let manager = EnhancedConfigurationManager::with_custom(
      provider,
      Some(keystore_config),
      true
  ).await?;
  ```

- **`get_enhanced()`** - Get enhanced configuration
  ```rust
  let enhanced_config = manager.get_enhanced().await?;
  ```

- **`set_enhanced(config)`** - Set enhanced configuration
  ```rust
  manager.set_enhanced(enhanced_config).await?;
  ```

- **`on_change(callback)`** - Register change notification callback
  ```rust
  manager.on_change(|event| {
      println!("Config changed: {:?}", event);
  }).await?;
  ```

- **`get_metrics()`** - Get performance metrics
  ```rust
  let metrics = manager.get_metrics().await;
  ```

- **`clear_cache()`** - Clear all caches
  ```rust
  manager.clear_cache().await?;
  ```

## Platform Integration

### Platform Information

```rust
pub struct PlatformInfo {
    pub name: String,
    pub version: String,
    pub arch: String,
    pub supports_xdg: bool,
    pub supports_keyring: bool,
    pub supports_file_watching: bool,
}
```

**Usage:**
```rust
use datafold::config::platform::get_platform_info;

let info = get_platform_info();
println!("Platform: {} on {}", info.name, info.arch);
if info.supports_keyring {
    println!("Keyring support available");
}
```

### Keystore Integration

```rust
#[async_trait]
pub trait PlatformKeystore: Send + Sync {
    async fn store_secret(&self, key: &str, secret: &[u8]) -> ConfigResult<()>;
    async fn retrieve_secret(&self, key: &str) -> ConfigResult<Vec<u8>>;
    async fn delete_secret(&self, key: &str) -> ConfigResult<()>;
    async fn list_keys(&self) -> ConfigResult<Vec<String>>;
}
```

**Example:**
```rust
use datafold::config::platform::keystore::create_platform_keystore;

let keystore = create_platform_keystore().await?;
keystore.store_secret("api_key", b"secret_value").await?;
let secret = keystore.retrieve_secret("api_key").await?;
```

## Migration Utilities

### ConfigMigrationManager

Handles migration from legacy configuration formats.

```rust
pub struct ConfigMigrationManager {
    target_provider: Arc<dyn ConfigurationProvider>,
    platform_paths: Box<dyn PlatformConfigPaths>,
}
```

**Methods:**

- **`new()`** - Create migration manager
  ```rust
  let migration_manager = ConfigMigrationManager::new();
  ```

- **`migrate_cli_config()`** - Migrate CLI configuration
  ```rust
  let result = migration_manager.migrate_cli_config().await?;
  ```

- **`migrate_logging_config()`** - Migrate logging configuration
  ```rust
  let result = migration_manager.migrate_logging_config().await?;
  ```

- **`migrate_unified_config()`** - Migrate unified configuration
  ```rust
  let result = migration_manager.migrate_unified_config().await?;
  ```

- **`migrate_config_file(source, target, strategy)`** - Migrate specific file
  ```rust
  let result = migration_manager.migrate_config_file(
      "/old/config.json",
      "/new/config.toml",
      MigrationStrategy::PreserveAndConvert
  ).await?;
  ```

- **`migrate_all()`** - Migrate all detected configurations
  ```rust
  let results = migration_manager.migrate_all().await?;
  ```

### MigrationResult

Result information from migration operations.

```rust
pub struct MigrationResult {
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub success: bool,
    pub items_migrated: usize,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub backup_path: Option<PathBuf>,
    pub migration_time: std::time::Duration,
}
```

**Methods:**

- **`was_successful()`** - Check if migration succeeded
- **`has_warnings()`** - Check if there were warnings
- **`print_summary()`** - Print migration summary

## Error Handling

### ConfigError

Comprehensive error types for configuration operations.

```rust
pub enum ConfigError {
    NotFound(String),
    ParseError(String),
    ValidationError(String),
    AccessDenied(String),
    Platform(String),
    Keystore(String),
    Migration(String),
    SerializationError(String),
    NetworkError(String),
    EncryptionError(String),
    CacheError(String),
    ConfigurationConflict(String),
}
```

**Helper Methods:**

- **`is_recoverable()`** - Check if error is recoverable
- **`user_message()`** - Get user-friendly error message

### ConfigErrorContext

Enhanced error context with operation details.

```rust
pub struct ConfigErrorContext {
    pub error: ConfigError,
    pub operation: String,
    pub file_path: Option<String>,
    pub component: Option<String>,
}
```

**Usage:**
```rust
use datafold::config::error::{ConfigError, ConfigErrorContext};

let context = ConfigErrorContext::new(
    ConfigError::NotFound("Config file not found".to_string()),
    "load_configuration".to_string()
)
.with_path("/path/to/config.toml")
.with_component("cli");
```

## Usage Examples

### Basic Configuration Loading

```rust
use datafold::config::{ConfigurationManager, ConfigValue};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = ConfigurationManager::new();
    let config = manager.get().await?;
    
    // Get database configuration
    let db_config = config.get_section("database")?;
    let host = db_config.get("host")?.as_string()?;
    let port = db_config.get("port")?.as_integer()?;
    
    println!("Database: {}:{}", host, port);
    Ok(())
}
```

### Enhanced Configuration with Keystore

```rust
use datafold::config::{EnhancedConfigurationManager, ConfigValue};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = EnhancedConfigurationManager::new().await?;
    let enhanced_config = manager.get_enhanced().await?;
    
    // Access base configuration
    let api_endpoint = enhanced_config.base.get_value("api.endpoint")?;
    
    // Use platform-specific features
    if enhanced_config.platform_settings.keystore_enabled {
        println!("Keystore integration is enabled");
    }
    
    Ok(())
}
```

### Configuration Migration

```rust
use datafold::config::migration::{ConfigMigrationManager, MigrationStrategy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let migration_manager = ConfigMigrationManager::new();
    
    // Migrate all configurations
    let results = migration_manager.migrate_all().await?;
    
    for result in results {
        if result.success {
            println!("✓ Migrated: {} -> {}", 
                result.source_path.display(), 
                result.target_path.display());
        } else {
            println!("✗ Failed: {}", result.source_path.display());
            for error in &result.errors {
                println!("  Error: {}", error);
            }
        }
    }
    
    Ok(())
}
```

### Real-time Configuration Monitoring

```rust
use datafold::config::EnhancedConfigurationManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = EnhancedConfigurationManager::new().await?;
    
    // Register for configuration changes
    manager.on_change(|event| {
        println!("Configuration changed:");
        println!("  Type: {:?}", event.change_type);
        println!("  Source: {:?}", event.source);
        println!("  Timestamp: {}", event.timestamp);
    }).await?;
    
    // Keep the application running to receive change notifications
    tokio::signal::ctrl_c().await?;
    Ok(())
}
```

### Custom Configuration Provider

```rust
use datafold::config::{ConfigurationProvider, Config, ConfigResult};
use async_trait::async_trait;
use std::path::PathBuf;

pub struct DatabaseConfigProvider {
    connection_string: String,
}

#[async_trait]
impl ConfigurationProvider for DatabaseConfigProvider {
    async fn load(&self) -> ConfigResult<Config> {
        // Load configuration from database
        todo!("Implement database loading")
    }
    
    async fn save(&self, config: &Config) -> ConfigResult<()> {
        // Save configuration to database
        todo!("Implement database saving")
    }
    
    async fn reload(&self) -> ConfigResult<Config> {
        self.load().await
    }
    
    async fn validate(&self, _config: &Config) -> ConfigResult<()> {
        Ok(())
    }
    
    async fn exists(&self) -> ConfigResult<bool> {
        Ok(true)
    }
    
    fn config_path(&self) -> ConfigResult<PathBuf> {
        Ok(PathBuf::from("database://config"))
    }
    
    fn provider_type(&self) -> &'static str {
        "database"
    }
}
```

## Thread Safety and Performance

All configuration managers and providers are designed to be thread-safe and can be shared across async tasks:

```rust
use std::sync::Arc;
use datafold::config::ConfigurationManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = Arc::new(ConfigurationManager::new());
    
    let mut handles = vec![];
    
    // Spawn multiple tasks that can safely access configuration
    for i in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let config = manager_clone.get().await.unwrap();
            println!("Task {}: Loaded config with {} sections", i, config.sections.len());
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await?;
    }
    
    Ok(())
}
```

## Related Documentation

- [Architecture](architecture.md) - System architecture and design
- [Integration Guide](integration.md) - Integration patterns and examples
- [Deployment Guide](deployment.md) - Deployment and migration procedures
- [Security Guide](security.md) - Security features and best practices