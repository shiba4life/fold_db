# Configuration Traits Usage Guide

A comprehensive developer guide for implementing and using the DataFold configuration traits system.

## Table of Contents

- [Getting Started](#getting-started)
- [Basic Implementation](#basic-implementation)
- [Advanced Patterns](#advanced-patterns)
- [Domain-Specific Traits](#domain-specific-traits)
- [Error Handling](#error-handling)
- [Testing](#testing)
- [Performance Optimization](#performance-optimization)
- [Best Practices](#best-practices)

## Getting Started

### Prerequisites

```toml
[dependencies]
datafold = { version = "0.1.0", features = ["config-traits"] }
async-trait = "0.1"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

### Basic Imports

```rust
use datafold::config::traits::{
    BaseConfig, ConfigLifecycle, ConfigValidation,
    TraitConfigError, TraitConfigResult
};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::path::Path;
```

## Basic Implementation

### Step 1: Define Your Configuration Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub port: u16,
    pub timeout_seconds: u64,
    pub max_connections: u32,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            name: "MyApplication".to_string(),
            version: "1.0.0".to_string(),
            enabled: true,
            port: 8080,
            timeout_seconds: 30,
            max_connections: 100,
        }
    }
}
```

### Step 2: Implement BaseConfig

```rust
#[async_trait]
impl BaseConfig for ApplicationConfig {
    type Error = ConfigError;
    type Event = ConfigChangeEvent;
    type TransformTarget = ();

    async fn load(path: &Path) -> Result<Self, Self::Error> {
        // Read configuration file
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| ConfigError::LoadError {
                path: path.display().to_string(),
                source: Box::new(e),
            })?;

        // Parse configuration
        let config: Self = match path.extension().and_then(|ext| ext.to_str()) {
            Some("toml") => toml::from_str(&content)?,
            Some("json") => serde_json::from_str(&content)?,
            Some("yaml") | Some("yml") => serde_yaml::from_str(&content)?,
            _ => return Err(ConfigError::UnsupportedFormat {
                format: path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
            }),
        };

        // Validate after loading
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), Self::Error> {
        if self.name.is_empty() {
            return Err(ConfigError::ValidationError {
                field: "name".to_string(),
                message: "Application name cannot be empty".to_string(),
            });
        }

        if self.port == 0 {
            return Err(ConfigError::ValidationError {
                field: "port".to_string(),
                message: "Port must be greater than 0".to_string(),
            });
        }

        if self.port < 1024 && self.port != 80 && self.port != 443 {
            return Err(ConfigError::ValidationError {
                field: "port".to_string(),
                message: "Port should be >= 1024 for non-privileged operation".to_string(),
            });
        }

        if self.timeout_seconds == 0 {
            return Err(ConfigError::ValidationError {
                field: "timeout_seconds".to_string(),
                message: "Timeout must be greater than 0".to_string(),
            });
        }

        if self.timeout_seconds > 300 {
            return Err(ConfigError::ValidationError {
                field: "timeout_seconds".to_string(),
                message: "Timeout should not exceed 300 seconds".to_string(),
            });
        }

        if self.max_connections == 0 {
            return Err(ConfigError::ValidationError {
                field: "max_connections".to_string(),
                message: "Max connections must be greater than 0".to_string(),
            });
        }

        Ok(())
    }

    fn report_event(&self, event: Self::Event) {
        // Integration with PBI 26 unified reporting
        log::info!("Application config event: {:?}", event);
        
        // Example: Send to monitoring system
        // metrics::counter!("config.events", 1, "type" => event.event_type());
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
```

### Step 3: Add Lifecycle Management

```rust
#[async_trait]
impl ConfigLifecycle for ApplicationConfig {
    async fn save(&self, path: &Path) -> Result<(), Self::Error> {
        // Validate before saving
        self.validate()?;

        // Serialize based on file extension
        let content = match path.extension().and_then(|ext| ext.to_str()) {
            Some("toml") => toml::to_string_pretty(self)?,
            Some("json") => serde_json::to_string_pretty(self)?,
            Some("yaml") | Some("yml") => serde_yaml::to_string(self)?,
            _ => return Err(ConfigError::UnsupportedFormat {
                format: path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
            }),
        };

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| ConfigError::SaveError {
                    path: path.display().to_string(),
                    source: Box::new(e),
                })?;
        }

        // Write file atomically
        let temp_path = path.with_extension("tmp");
        tokio::fs::write(&temp_path, content).await
            .map_err(|e| ConfigError::SaveError {
                path: temp_path.display().to_string(),
                source: Box::new(e),
            })?;

        tokio::fs::rename(&temp_path, path).await
            .map_err(|e| ConfigError::SaveError {
                path: path.display().to_string(),
                source: Box::new(e),
            })?;

        Ok(())
    }

    async fn reload(&mut self, path: &Path) -> Result<(), Self::Error> {
        let new_config = Self::load(path).await?;
        *self = new_config;
        Ok(())
    }

    fn get_metadata(&self) -> &ConfigMetadata {
        // Implementation depends on how you store metadata
        // This is a simplified example
        static DEFAULT_METADATA: ConfigMetadata = ConfigMetadata {
            version: "1.0.0",
            last_modified: None,
            checksum: None,
        };
        &DEFAULT_METADATA
    }

    fn set_metadata(&mut self, metadata: ConfigMetadata) {
        // Store metadata in your configuration struct or externally
        // Implementation depends on your metadata storage strategy
    }
}
```

## Advanced Patterns

### Configuration Composition

```rust
use datafold::config::traits::{ConfigMerge, MergeStrategy};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeConfig {
    pub app: ApplicationConfig,
    pub database: DatabaseConfig,
    pub network: NetworkConfig,
}

impl ConfigMerge for CompositeConfig {
    fn merge_with_strategy(&self, other: &Self, strategy: MergeStrategy) -> TraitConfigResult<Self> {
        match strategy {
            MergeStrategy::Replace => Ok(other.clone()),
            MergeStrategy::DeepMerge => {
                Ok(Self {
                    app: self.app.merge_with_strategy(&other.app, strategy)?,
                    database: self.database.merge_with_strategy(&other.database, strategy)?,
                    network: self.network.merge_with_strategy(&other.network, strategy)?,
                })
            }
            MergeStrategy::Keep => Ok(self.clone()),
            MergeStrategy::FailOnConflict => {
                let conflicts = self.check_merge_conflicts(other);
                if conflicts.is_empty() {
                    Ok(other.clone())
                } else {
                    Err(TraitConfigError::MergeConflict {
                        conflicts,
                    })
                }
            }
            _ => Err(TraitConfigError::UnsupportedMergeStrategy {
                strategy: format!("{:?}", strategy),
            }),
        }
    }

    fn deep_merge(&self, other: &Self) -> TraitConfigResult<Self> {
        self.merge_with_strategy(other, MergeStrategy::DeepMerge)
    }

    // ... other required methods
}
```

### Environment Variable Integration

```rust
impl ApplicationConfig {
    /// Apply environment variable overrides
    pub fn apply_env_overrides(&mut self) -> Result<(), ConfigError> {
        use std::env;

        if let Ok(name) = env::var("APP_NAME") {
            self.name = name;
        }

        if let Ok(port_str) = env::var("APP_PORT") {
            self.port = port_str.parse()
                .map_err(|_| ConfigError::ValidationError {
                    field: "port".to_string(),
                    message: format!("Invalid port in APP_PORT: {}", port_str),
                })?;
        }

        if let Ok(enabled_str) = env::var("APP_ENABLED") {
            self.enabled = enabled_str.parse()
                .map_err(|_| ConfigError::ValidationError {
                    field: "enabled".to_string(),
                    message: format!("Invalid boolean in APP_ENABLED: {}", enabled_str),
                })?;
        }

        if let Ok(timeout_str) = env::var("APP_TIMEOUT_SECONDS") {
            self.timeout_seconds = timeout_str.parse()
                .map_err(|_| ConfigError::ValidationError {
                    field: "timeout_seconds".to_string(),
                    message: format!("Invalid timeout in APP_TIMEOUT_SECONDS: {}", timeout_str),
                })?;
        }

        // Re-validate after applying overrides
        self.validate()?;
        Ok(())
    }
}
```

## Domain-Specific Traits

### Database Configuration Example

```rust
use datafold::config::traits::{
    DatabaseConfig, ConnectionConfigTrait, BackupConfigTrait, EncryptionConfigTrait
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyDatabaseConfig {
    pub connection: StandardConnectionConfig,
    pub backup: StandardBackupConfig,
    pub encryption: StandardEncryptionConfig,
    pub performance: DatabasePerformanceConfig,
}

#[async_trait]
impl DatabaseConfig for MyDatabaseConfig {
    type ConnectionConfig = StandardConnectionConfig;
    type BackupConfig = StandardBackupConfig;
    type EncryptionConfig = StandardEncryptionConfig;
    type PerformanceConfig = DatabasePerformanceConfig;

    fn connection_config(&self) -> &Self::ConnectionConfig {
        &self.connection
    }

    fn backup_config(&self) -> &Self::BackupConfig {
        &self.backup
    }

    fn encryption_config(&self) -> &Self::EncryptionConfig {
        &self.encryption
    }

    fn performance_config(&self) -> &Self::PerformanceConfig {
        &self.performance
    }

    async fn validate_connectivity(&self) -> TraitConfigResult<()> {
        // Test database connection
        let connection_string = format!("sqlite://{}", 
            self.connection.database_path().display());
        
        // Attempt connection with timeout
        tokio::time::timeout(
            self.connection.connection_timeout(),
            test_database_connection(&connection_string)
        ).await
        .map_err(|_| TraitConfigError::ValidationError {
            field: "connection".to_string(),
            message: "Database connection timeout".to_string(),
            context: ValidationContext::default(),
        })??;

        Ok(())
    }

    async fn apply_env_overrides(&mut self) -> TraitConfigResult<()> {
        use std::env;

        // Override database path
        if let Ok(db_path) = env::var("DATABASE_PATH") {
            self.connection.database_path = db_path.into();
        }

        // Override connection timeout
        if let Ok(timeout_str) = env::var("DATABASE_TIMEOUT") {
            let timeout: u64 = timeout_str.parse()
                .map_err(|_| TraitConfigError::ValidationError {
                    field: "connection_timeout".to_string(),
                    message: format!("Invalid timeout: {}", timeout_str),
                    context: ValidationContext::default(),
                })?;
            self.connection.connection_timeout_seconds = timeout;
        }

        // Re-validate after overrides
        self.validate()?;
        Ok(())
    }

    fn validate_backup_settings(&self) -> TraitConfigResult<()> {
        self.backup.validate()
    }

    fn validate_encryption_settings(&self) -> TraitConfigResult<()> {
        self.encryption.validate()
    }

    fn validate_performance_settings(&self) -> TraitConfigResult<()> {
        self.performance.validate()
    }
}

async fn test_database_connection(connection_string: &str) -> TraitConfigResult<()> {
    // Implementation depends on your database driver
    // This is a placeholder
    Ok(())
}
```

### Network Configuration Example

```rust
use datafold::config::traits::{NetworkConfig, SecurityConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyNetworkConfig {
    pub bind_address: String,
    pub port: u16,
    pub security: SecuritySettings,
    pub timeouts: TimeoutSettings,
}

#[async_trait]
impl NetworkConfig for MyNetworkConfig {
    type SecuritySettings = SecuritySettings;
    type HealthMetrics = NetworkHealthMetrics;

    fn security_settings(&self) -> &Self::SecuritySettings {
        &self.security
    }

    async fn test_connectivity(&self) -> TraitConfigResult<ConnectivityTestResult> {
        use std::net::SocketAddr;
        use tokio::net::TcpListener;

        let addr: SocketAddr = format!("{}:{}", self.bind_address, self.port)
            .parse()
            .map_err(|e| TraitConfigError::ValidationError {
                field: "bind_address".to_string(),
                message: format!("Invalid address format: {}", e),
                context: ValidationContext::default(),
            })?;

        // Test if we can bind to the address
        match TcpListener::bind(&addr).await {
            Ok(_) => Ok(ConnectivityTestResult {
                status: ConnectivityStatus::Available,
                latency: None,
                error_message: None,
            }),
            Err(e) => Ok(ConnectivityTestResult {
                status: ConnectivityStatus::Failed,
                latency: None,
                error_message: Some(format!("Cannot bind to {}: {}", addr, e)),
            }),
        }
    }

    async fn get_health_metrics(&self) -> TraitConfigResult<Self::HealthMetrics> {
        // Collect network health metrics
        Ok(NetworkHealthMetrics {
            connections_active: 0, // Get from actual monitoring
            bandwidth_usage: 0.0,
            error_rate: 0.0,
            response_time_avg: 0.0,
        })
    }

    async fn apply_env_overrides(&mut self) -> TraitConfigResult<()> {
        use std::env;

        if let Ok(bind_addr) = env::var("NETWORK_BIND_ADDRESS") {
            self.bind_address = bind_addr;
        }

        if let Ok(port_str) = env::var("NETWORK_PORT") {
            self.port = port_str.parse()
                .map_err(|_| TraitConfigError::ValidationError {
                    field: "port".to_string(),
                    message: format!("Invalid port: {}", port_str),
                    context: ValidationContext::default(),
                })?;
        }

        self.validate()?;
        Ok(())
    }
}
```

## Error Handling

### Custom Error Types

```rust
use datafold::config::traits::{TraitConfigError, ValidationContext};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Validation error in field '{field}': {message}")]
    ValidationError {
        field: String,
        message: String,
    },

    #[error("Failed to load configuration from '{path}': {source}")]
    LoadError {
        path: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Failed to save configuration to '{path}': {source}")]
    SaveError {
        path: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Unsupported format: {format}")]
    UnsupportedFormat {
        format: String,
    },

    #[error("Serialization error: {source}")]
    SerializationError {
        #[from]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        ConfigError::SerializationError {
            source: Box::new(err),
        }
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::SerializationError {
            source: Box::new(err),
        }
    }
}
```

### Error Handling Patterns

```rust
use datafold::config::traits::ValidationContext;

impl ApplicationConfig {
    /// Validate with detailed context
    fn validate_with_context(&self, context: &ValidationContext) -> Result<(), ConfigError> {
        let mut errors = Vec::new();

        // Collect all validation errors
        if self.name.is_empty() {
            errors.push(format!("name: cannot be empty"));
        }

        if self.port == 0 {
            errors.push(format!("port: must be greater than 0"));
        }

        if self.timeout_seconds == 0 {
            errors.push(format!("timeout_seconds: must be greater than 0"));
        }

        if !errors.is_empty() {
            return Err(ConfigError::ValidationError {
                field: "multiple".to_string(),
                message: format!("Multiple validation errors: {}", errors.join(", ")),
            });
        }

        Ok(())
    }

    /// Validate individual field
    fn validate_field(&self, field: &str, value: &dyn Any) -> Result<(), ConfigError> {
        match field {
            "name" => {
                if let Some(name) = value.downcast_ref::<String>() {
                    if name.is_empty() {
                        return Err(ConfigError::ValidationError {
                            field: field.to_string(),
                            message: "Name cannot be empty".to_string(),
                        });
                    }
                }
            },
            "port" => {
                if let Some(&port) = value.downcast_ref::<u16>() {
                    if port == 0 {
                        return Err(ConfigError::ValidationError {
                            field: field.to_string(),
                            message: "Port must be greater than 0".to_string(),
                        });
                    }
                }
            },
            _ => {
                return Err(ConfigError::ValidationError {
                    field: field.to_string(),
                    message: "Unknown field".to_string(),
                });
            }
        }
        Ok(())
    }
}
```

## Testing

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use datafold::config::traits::testing::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_application_config_basic() {
        let config = ApplicationConfig::default();
        
        // Test basic validation
        assert!(config.validate().is_ok());
        
        // Test trait compliance
        ConfigTestHelper::assert_base_config_compliance(&config).await;
    }

    #[tokio::test]
    async fn test_configuration_loading() {
        let config = ApplicationConfig {
            name: "TestApp".to_string(),
            port: 3000,
            ..Default::default()
        };

        // Test TOML serialization roundtrip
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().with_extension("toml");
        
        config.save(&path).await.unwrap();
        let loaded_config = ApplicationConfig::load(&path).await.unwrap();
        
        assert_eq!(config.name, loaded_config.name);
        assert_eq!(config.port, loaded_config.port);
    }

    #[tokio::test]
    async fn test_validation_errors() {
        let mut config = ApplicationConfig::default();
        config.name = String::new(); // Invalid: empty name
        config.port = 0; // Invalid: zero port
        
        let result = config.validate();
        assert!(result.is_err());
        
        // Test specific error fields
        if let Err(ConfigError::ValidationError { field, message }) = result {
            assert!(field == "name" || message.contains("name"));
        }
    }

    #[tokio::test]
    async fn test_environment_overrides() {
        use std::env;
        
        // Set environment variables
        env::set_var("APP_NAME", "EnvTestApp");
        env::set_var("APP_PORT", "9000");
        
        let mut config = ApplicationConfig::default();
        config.apply_env_overrides().unwrap();
        
        assert_eq!(config.name, "EnvTestApp");
        assert_eq!(config.port, 9000);
        
        // Clean up
        env::remove_var("APP_NAME");
        env::remove_var("APP_PORT");
    }

    #[test]
    fn test_trait_object_compatibility() {
        let config = ApplicationConfig::default();
        let boxed: Box<dyn BaseConfig<Error = ConfigError, Event = ConfigChangeEvent, TransformTarget = ()>> 
            = Box::new(config);
        
        // Should be able to use trait methods
        assert!(boxed.validate().is_ok());
    }
}
```

### Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use datafold::config::traits::testing::*;

    #[tokio::test]
    async fn test_full_configuration_lifecycle() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("app_config.toml");
        
        // Create initial configuration
        let mut config = ApplicationConfig {
            name: "IntegrationTest".to_string(),
            port: 8080,
            enabled: true,
            ..Default::default()
        };
        
        // Test save
        config.save(&config_path).await.unwrap();
        assert!(config_path.exists());
        
        // Test load
        let loaded_config = ApplicationConfig::load(&config_path).await.unwrap();
        assert_eq!(config.name, loaded_config.name);
        
        // Test reload
        config.name = "Modified".to_string();
        config.save(&config_path).await.unwrap();
        
        let mut original_config = ApplicationConfig::default();
        original_config.reload(&config_path).await.unwrap();
        assert_eq!(original_config.name, "Modified");
    }

    #[tokio::test]
    async fn test_cross_platform_compatibility() {
        let config = ApplicationConfig::default();
        
        // Test on current platform
        ConfigTestHelper::test_platform_compatibility(&config).await.unwrap();
    }
}
```

## Performance Optimization

### Lazy Loading

```rust
use std::sync::Arc;
use tokio::sync::OnceCell;

#[derive(Debug, Clone)]
pub struct LazyLoadedConfig {
    // Core config always loaded
    pub core: ApplicationConfig,
    
    // Heavy configurations loaded on demand
    database_config: Arc<OnceCell<DatabaseConfig>>,
    analytics_config: Arc<OnceCell<AnalyticsConfig>>,
}

impl LazyLoadedConfig {
    pub async fn database_config(&self) -> Result<&DatabaseConfig, ConfigError> {
        self.database_config.get_or_try_init(|| async {
            DatabaseConfig::load(Path::new("database.toml")).await
        }).await
    }

    pub async fn analytics_config(&self) -> Result<&AnalyticsConfig, ConfigError> {
        self.analytics_config.get_or_try_init(|| async {
            AnalyticsConfig::load(Path::new("analytics.toml")).await
        }).await
    }
}
```

### Caching and Memoization

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Debug)]
pub struct CachedConfigManager {
    cache: Arc<RwLock<HashMap<String, Arc<dyn BaseConfig<Error = ConfigError, Event = ConfigChangeEvent, TransformTarget = ()>>>>>,
}

impl CachedConfigManager {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_config<T>(&self, key: &str, path: &Path) -> Result<Arc<T>, ConfigError>
    where
        T: BaseConfig<Error = ConfigError, Event = ConfigChangeEvent, TransformTarget = ()> + 'static,
    {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(key) {
                if let Some(typed) = cached.as_any().downcast_ref::<T>() {
                    return Ok(Arc::new(typed.clone()));
                }
            }
        }

        // Load and cache
        let config = T::load(path).await?;
        let arc_config = Arc::new(config);
        
        {
            let mut cache = self.cache.write().await;
            cache.insert(key.to_string(), arc_config.clone());
        }

        Ok(arc_config)
    }
}
```

## Best Practices

### 1. Configuration Validation

- **Validate Early**: Always validate configurations immediately after loading
- **Comprehensive Checks**: Include range checks, format validation, and business logic validation
- **Clear Error Messages**: Provide actionable error messages with context

```rust
fn validate(&self) -> Result<(), Self::Error> {
    // Example of comprehensive validation
    if self.timeout_seconds == 0 {
        return Err(ConfigError::ValidationError {
            field: "timeout_seconds".to_string(),
            message: "Timeout must be greater than 0. Consider using 30 seconds for most applications.".to_string(),
        });
    }
    
    if self.timeout_seconds > 300 {
        return Err(ConfigError::ValidationError {
            field: "timeout_seconds".to_string(),
            message: "Timeout should not exceed 300 seconds to avoid hanging operations.".to_string(),
        });
    }
    
    Ok(())
}
```

### 2. Error Handling

- **Use Structured Errors**: Implement thiserror for better error messages
- **Provide Context**: Include relevant information in error messages
- **Chain Errors**: Use error chaining to preserve original error information

### 3. Documentation

- **Document Traits**: Provide comprehensive documentation for trait implementations
- **Example Configurations**: Include example configuration files
- **Migration Guides**: Document migration paths from legacy configurations

### 4. Testing

- **Test All Paths**: Test both success and failure scenarios
- **Use Trait Helpers**: Leverage the built-in testing infrastructure
- **Integration Tests**: Test the complete configuration lifecycle

### 5. Performance

- **Minimize Trait Objects**: Use static dispatch when possible
- **Cache Configurations**: Cache frequently accessed configurations
- **Lazy Loading**: Load expensive configurations only when needed

### 6. Security

- **Sensitive Data**: Never log sensitive configuration values
- **Validation**: Validate all input from configuration files
- **Environment Variables**: Use environment variables for sensitive overrides

```rust
impl ApplicationConfig {
    pub fn masked_for_logging(&self) -> ApplicationConfigMasked {
        ApplicationConfigMasked {
            name: self.name.clone(),
            port: self.port,
            enabled: self.enabled,
            // Don't include sensitive fields
        }
    }
}
```

---

This usage guide provides comprehensive examples and patterns for implementing configuration traits in the DataFold system. For more specific domain examples, see the [examples directory](examples/) or the [API reference](../../../docs/api-reference.md).