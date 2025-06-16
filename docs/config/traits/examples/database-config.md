# Database Configuration Example

This example demonstrates implementing a production-ready database configuration using the DataFold configuration traits system with [`DatabaseConfig`](../../../../src/config/traits/database.rs:18) trait.

## Overview

A comprehensive database configuration that includes connection settings, backup configuration, encryption settings, and performance tuning. This example shows how to use domain-specific traits to implement specialized configuration patterns.

## Implementation

### Main Configuration Struct

```rust
use datafold::config::traits::{
    BaseConfig, ConfigLifecycle, ConfigValidation, DatabaseConfig,
    StandardConnectionConfig, StandardBackupConfig, StandardEncryptionConfig,
    DatabasePerformanceConfig, TraitConfigError, TraitConfigResult
};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionDatabaseConfig {
    /// Database connection settings
    pub connection: StandardConnectionConfig,
    
    /// Backup configuration
    pub backup: StandardBackupConfig,
    
    /// Encryption settings
    pub encryption: StandardEncryptionConfig,
    
    /// Performance tuning
    pub performance: DatabasePerformanceConfig,
    
    /// Application-specific settings
    pub application: ApplicationDatabaseSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationDatabaseSettings {
    /// Database schema version
    pub schema_version: String,
    
    /// Connection pool settings
    pub pool_settings: ConnectionPoolSettings,
    
    /// Query timeout in seconds
    pub query_timeout_seconds: u64,
    
    /// Enable query logging
    pub enable_query_logging: bool,
    
    /// Enable slow query detection
    pub enable_slow_query_detection: bool,
    
    /// Slow query threshold in milliseconds
    pub slow_query_threshold_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolSettings {
    /// Minimum number of connections in pool
    pub min_connections: u32,
    
    /// Maximum number of connections in pool
    pub max_connections: u32,
    
    /// Connection idle timeout in seconds
    pub idle_timeout_seconds: u64,
    
    /// Maximum connection lifetime in seconds
    pub max_lifetime_seconds: u64,
}

impl Default for ProductionDatabaseConfig {
    fn default() -> Self {
        Self {
            connection: StandardConnectionConfig::default(),
            backup: StandardBackupConfig::default(),
            encryption: StandardEncryptionConfig::default(),
            performance: DatabasePerformanceConfig::default(),
            application: ApplicationDatabaseSettings::default(),
        }
    }
}

impl Default for ApplicationDatabaseSettings {
    fn default() -> Self {
        Self {
            schema_version: "1.0.0".to_string(),
            pool_settings: ConnectionPoolSettings::default(),
            query_timeout_seconds: 30,
            enable_query_logging: false,
            enable_slow_query_detection: true,
            slow_query_threshold_ms: 1000,
        }
    }
}

impl Default for ConnectionPoolSettings {
    fn default() -> Self {
        Self {
            min_connections: 5,
            max_connections: 20,
            idle_timeout_seconds: 600,  // 10 minutes
            max_lifetime_seconds: 3600, // 1 hour
        }
    }
}
```

### Error and Event Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseConfigError {
    #[error("Database validation error in field '{field}': {message}")]
    ValidationError {
        field: String,
        message: String,
    },

    #[error("Database connection error: {message}")]
    ConnectionError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Backup operation error: {message}")]
    BackupError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Encryption error: {message}")]
    EncryptionError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("I/O error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    #[error("Serialization error: {source}")]
    SerializationError {
        #[from]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfigEvent {
    pub event_type: DatabaseEventType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub database_path: Option<String>,
    pub details: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseEventType {
    ConfigLoaded,
    ConfigSaved,
    ConnectionTested,
    BackupCreated,
    BackupRestored,
    EncryptionEnabled,
    EncryptionDisabled,
    PerformanceTuned,
    SchemaUpdated,
}

impl DatabaseConfigEvent {
    pub fn new(event_type: DatabaseEventType) -> Self {
        Self {
            event_type,
            timestamp: chrono::Utc::now(),
            database_path: None,
            details: std::collections::HashMap::new(),
        }
    }
    
    pub fn with_database_path(mut self, path: String) -> Self {
        self.database_path = Some(path);
        self
    }
    
    pub fn with_detail(mut self, key: String, value: String) -> Self {
        self.details.insert(key, value);
        self
    }
}
```

### BaseConfig Implementation

```rust
#[async_trait]
impl BaseConfig for ProductionDatabaseConfig {
    type Error = DatabaseConfigError;
    type Event = DatabaseConfigEvent;
    type TransformTarget = ();

    async fn load(path: &Path) -> Result<Self, Self::Error> {
        let content = tokio::fs::read_to_string(path).await?;
        
        let config: Self = match path.extension().and_then(|ext| ext.to_str()) {
            Some("toml") => toml::from_str(&content)
                .map_err(|e| DatabaseConfigError::SerializationError {
                    source: Box::new(e),
                })?,
            Some("json") => serde_json::from_str(&content)
                .map_err(|e| DatabaseConfigError::SerializationError {
                    source: Box::new(e),
                })?,
            Some("yaml") | Some("yml") => serde_yaml::from_str(&content)
                .map_err(|e| DatabaseConfigError::SerializationError {
                    source: Box::new(e),
                })?,
            _ => return Err(DatabaseConfigError::ValidationError {
                field: "file_format".to_string(),
                message: "Unsupported configuration file format".to_string(),
            }),
        };

        config.validate()?;
        
        // Report load event
        config.report_event(DatabaseConfigEvent::new(DatabaseEventType::ConfigLoaded)
            .with_database_path(config.connection.database_path().display().to_string()));
        
        Ok(config)
    }

    fn validate(&self) -> Result<(), Self::Error> {
        // Validate connection settings
        self.connection.validate()
            .map_err(|e| DatabaseConfigError::ValidationError {
                field: "connection".to_string(),
                message: format!("Connection validation failed: {}", e),
            })?;

        // Validate backup settings
        self.backup.validate()
            .map_err(|e| DatabaseConfigError::ValidationError {
                field: "backup".to_string(),
                message: format!("Backup validation failed: {}", e),
            })?;

        // Validate encryption settings
        self.encryption.validate()
            .map_err(|e| DatabaseConfigError::ValidationError {
                field: "encryption".to_string(),
                message: format!("Encryption validation failed: {}", e),
            })?;

        // Validate performance settings
        self.performance.validate()
            .map_err(|e| DatabaseConfigError::ValidationError {
                field: "performance".to_string(),
                message: format!("Performance validation failed: {}", e),
            })?;

        // Validate application settings
        self.validate_application_settings()?;
        
        // Validate pool settings
        self.validate_pool_settings()?;

        Ok(())
    }

    fn report_event(&self, event: Self::Event) {
        // Log the event
        log::info!("Database config event: {:?}", event);
        
        // Send to metrics system
        metrics::counter!(
            "database.config.events",
            1,
            "type" => format!("{:?}", event.event_type),
            "database" => event.database_path.as_deref().unwrap_or("unknown")
        );
        
        // Send to monitoring system (example)
        if let Some(monitoring) = get_monitoring_system() {
            monitoring.send_event(&event);
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ProductionDatabaseConfig {
    fn validate_application_settings(&self) -> Result<(), DatabaseConfigError> {
        // Validate schema version format
        if self.application.schema_version.is_empty() {
            return Err(DatabaseConfigError::ValidationError {
                field: "schema_version".to_string(),
                message: "Schema version cannot be empty".to_string(),
            });
        }

        // Validate query timeout
        if self.application.query_timeout_seconds == 0 {
            return Err(DatabaseConfigError::ValidationError {
                field: "query_timeout_seconds".to_string(),
                message: "Query timeout must be greater than 0".to_string(),
            });
        }

        if self.application.query_timeout_seconds > 3600 {
            return Err(DatabaseConfigError::ValidationError {
                field: "query_timeout_seconds".to_string(),
                message: "Query timeout should not exceed 3600 seconds".to_string(),
            });
        }

        // Validate slow query threshold
        if self.application.enable_slow_query_detection && self.application.slow_query_threshold_ms == 0 {
            return Err(DatabaseConfigError::ValidationError {
                field: "slow_query_threshold_ms".to_string(),
                message: "Slow query threshold must be greater than 0 when detection is enabled".to_string(),
            });
        }

        Ok(())
    }

    fn validate_pool_settings(&self) -> Result<(), DatabaseConfigError> {
        let pool = &self.application.pool_settings;

        if pool.min_connections == 0 {
            return Err(DatabaseConfigError::ValidationError {
                field: "min_connections".to_string(),
                message: "Minimum connections must be greater than 0".to_string(),
            });
        }

        if pool.max_connections == 0 {
            return Err(DatabaseConfigError::ValidationError {
                field: "max_connections".to_string(),
                message: "Maximum connections must be greater than 0".to_string(),
            });
        }

        if pool.min_connections > pool.max_connections {
            return Err(DatabaseConfigError::ValidationError {
                field: "connection_pool".to_string(),
                message: "Minimum connections cannot exceed maximum connections".to_string(),
            });
        }

        if pool.max_connections > 1000 {
            return Err(DatabaseConfigError::ValidationError {
                field: "max_connections".to_string(),
                message: "Maximum connections should not exceed 1000".to_string(),
            });
        }

        if pool.idle_timeout_seconds == 0 {
            return Err(DatabaseConfigError::ValidationError {
                field: "idle_timeout_seconds".to_string(),
                message: "Idle timeout must be greater than 0".to_string(),
            });
        }

        if pool.max_lifetime_seconds == 0 {
            return Err(DatabaseConfigError::ValidationError {
                field: "max_lifetime_seconds".to_string(),
                message: "Maximum lifetime must be greater than 0".to_string(),
            });
        }

        Ok(())
    }
}

// Placeholder function for monitoring system
fn get_monitoring_system() -> Option<Box<dyn MonitoringSystem>> {
    None // Implementation depends on your monitoring setup
}

trait MonitoringSystem {
    fn send_event(&self, event: &DatabaseConfigEvent);
}
```

### DatabaseConfig Implementation

```rust
#[async_trait]
impl DatabaseConfig for ProductionDatabaseConfig {
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
        let result = tokio::time::timeout(
            self.connection.connection_timeout(),
            test_database_connection(&connection_string)
        ).await;

        match result {
            Ok(Ok(())) => {
                self.report_event(DatabaseConfigEvent::new(DatabaseEventType::ConnectionTested)
                    .with_database_path(self.connection.database_path().display().to_string())
                    .with_detail("status".to_string(), "success".to_string()));
                Ok(())
            }
            Ok(Err(e)) => {
                self.report_event(DatabaseConfigEvent::new(DatabaseEventType::ConnectionTested)
                    .with_database_path(self.connection.database_path().display().to_string())
                    .with_detail("status".to_string(), "failed".to_string())
                    .with_detail("error".to_string(), e.to_string()));
                Err(TraitConfigError::ValidationError {
                    field: "connection".to_string(),
                    message: format!("Database connection failed: {}", e),
                    context: ValidationContext::default(),
                })
            }
            Err(_) => {
                self.report_event(DatabaseConfigEvent::new(DatabaseEventType::ConnectionTested)
                    .with_database_path(self.connection.database_path().display().to_string())
                    .with_detail("status".to_string(), "timeout".to_string()));
                Err(TraitConfigError::ValidationError {
                    field: "connection".to_string(),
                    message: "Database connection timeout".to_string(),
                    context: ValidationContext::default(),
                })
            }
        }
    }

    async fn apply_env_overrides(&mut self) -> TraitConfigResult<()> {
        use std::env;

        // Override database path
        if let Ok(db_path) = env::var("DATABASE_PATH") {
            self.connection.database_path = PathBuf::from(db_path);
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

        // Override backup directory
        if let Ok(backup_dir) = env::var("DATABASE_BACKUP_DIR") {
            self.backup.backup_directory = PathBuf::from(backup_dir);
        }

        // Override encryption settings
        if let Ok(encryption_str) = env::var("DATABASE_ENCRYPTION_ENABLED") {
            let enabled: bool = encryption_str.parse()
                .map_err(|_| TraitConfigError::ValidationError {
                    field: "encryption_enabled".to_string(),
                    message: format!("Invalid boolean: {}", encryption_str),
                    context: ValidationContext::default(),
                })?;
            self.encryption.enabled = enabled;
        }

        // Override pool settings
        if let Ok(max_conn_str) = env::var("DATABASE_MAX_CONNECTIONS") {
            let max_conn: u32 = max_conn_str.parse()
                .map_err(|_| TraitConfigError::ValidationError {
                    field: "max_connections".to_string(),
                    message: format!("Invalid number: {}", max_conn_str),
                    context: ValidationContext::default(),
                })?;
            self.application.pool_settings.max_connections = max_conn;
        }

        // Re-validate after applying overrides
        self.validate()
            .map_err(|e| TraitConfigError::ValidationError {
                field: "configuration".to_string(),
                message: format!("Validation failed after environment overrides: {}", e),
                context: ValidationContext::default(),
            })?;

        Ok(())
    }

    fn validate_backup_settings(&self) -> TraitConfigResult<()> {
        self.backup.validate()
            .map_err(|e| TraitConfigError::ValidationError {
                field: "backup".to_string(),
                message: e.to_string(),
                context: ValidationContext::default(),
            })
    }

    fn validate_encryption_settings(&self) -> TraitConfigResult<()> {
        self.encryption.validate()
            .map_err(|e| TraitConfigError::ValidationError {
                field: "encryption".to_string(),
                message: e.to_string(),
                context: ValidationContext::default(),
            })
    }

    fn validate_performance_settings(&self) -> TraitConfigResult<()> {
        self.performance.validate()
            .map_err(|e| TraitConfigError::ValidationError {
                field: "performance".to_string(),
                message: e.to_string(),
                context: ValidationContext::default(),
            })
    }
}

// Placeholder for database connection testing
async fn test_database_connection(connection_string: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // In a real implementation, you would:
    // 1. Parse the connection string
    // 2. Attempt to establish a connection
    // 3. Perform a simple query (like SELECT 1)
    // 4. Close the connection
    
    // For this example, we'll simulate a connection test
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Simulate connection success for localhost paths
    if connection_string.contains("localhost") || connection_string.contains("127.0.0.1") {
        Ok(())
    } else {
        Err("Connection failed".into())
    }
}
```

### Additional Utility Methods

```rust
impl ProductionDatabaseConfig {
    /// Get query timeout as Duration
    pub fn query_timeout(&self) -> Duration {
        Duration::from_secs(self.application.query_timeout_seconds)
    }

    /// Get idle timeout as Duration
    pub fn idle_timeout(&self) -> Duration {
        Duration::from_secs(self.application.pool_settings.idle_timeout_seconds)
    }

    /// Get max lifetime as Duration
    pub fn max_lifetime(&self) -> Duration {
        Duration::from_secs(self.application.pool_settings.max_lifetime_seconds)
    }

    /// Check if query logging is enabled
    pub fn should_log_queries(&self) -> bool {
        self.application.enable_query_logging
    }

    /// Check if a query duration qualifies as slow
    pub fn is_slow_query(&self, duration: Duration) -> bool {
        self.application.enable_slow_query_detection &&
        duration.as_millis() >= self.application.slow_query_threshold_ms as u128
    }

    /// Create a backup with current settings
    pub async fn create_backup(&self) -> Result<PathBuf, DatabaseConfigError> {
        // Generate backup filename with timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("backup_{}.db", timestamp);
        let backup_path = self.backup.backup_directory.join(backup_filename);

        // Ensure backup directory exists
        tokio::fs::create_dir_all(&self.backup.backup_directory).await?;

        // In a real implementation, you would:
        // 1. Create database backup using appropriate tools
        // 2. Apply compression if configured
        // 3. Verify backup integrity if configured
        // 4. Handle encryption if configured

        // Simulate backup creation
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Report backup event
        self.report_event(DatabaseConfigEvent::new(DatabaseEventType::BackupCreated)
            .with_database_path(self.connection.database_path().display().to_string())
            .with_detail("backup_path".to_string(), backup_path.display().to_string())
            .with_detail("compression".to_string(), self.backup.compression_level.to_string()));

        Ok(backup_path)
    }

    /// Enable encryption with current settings
    pub async fn enable_encryption(&mut self) -> Result<(), DatabaseConfigError> {
        if !self.encryption.enabled {
            self.encryption.enabled = true;
            
            // In a real implementation, you would:
            // 1. Generate or load encryption keys
            // 2. Encrypt existing data if any
            // 3. Update database schema if needed
            // 4. Test encryption/decryption

            self.report_event(DatabaseConfigEvent::new(DatabaseEventType::EncryptionEnabled)
                .with_database_path(self.connection.database_path().display().to_string())
                .with_detail("algorithm".to_string(), self.encryption.algorithm.clone()));
        }

        Ok(())
    }

    /// Apply performance optimizations
    pub async fn optimize_performance(&mut self) -> Result<(), DatabaseConfigError> {
        // In a real implementation, you would:
        // 1. Analyze current performance metrics
        // 2. Apply optimizations based on workload
        // 3. Update configuration parameters
        // 4. Test performance improvements

        self.report_event(DatabaseConfigEvent::new(DatabaseEventType::PerformanceTuned)
            .with_database_path(self.connection.database_path().display().to_string())
            .with_detail("cache_size_mb".to_string(), self.performance.cache_size_mb.to_string())
            .with_detail("background_threads".to_string(), self.performance.background_threads.to_string()));

        Ok(())
    }

    /// Update schema version
    pub fn update_schema_version(&mut self, new_version: String) -> Result<(), DatabaseConfigError> {
        if new_version.is_empty() {
            return Err(DatabaseConfigError::ValidationError {
                field: "schema_version".to_string(),
                message: "Schema version cannot be empty".to_string(),
            });
        }

        let old_version = self.application.schema_version.clone();
        self.application.schema_version = new_version.clone();

        self.report_event(DatabaseConfigEvent::new(DatabaseEventType::SchemaUpdated)
            .with_database_path(self.connection.database_path().display().to_string())
            .with_detail("old_version".to_string(), old_version)
            .with_detail("new_version".to_string(), new_version));

        Ok(())
    }
}
```

## Usage Examples

### Basic Usage

```rust
use tokio;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load database configuration
    let mut config = ProductionDatabaseConfig::load(Path::new("database.toml")).await?;
    
    // Apply environment variable overrides
    config.apply_env_overrides().await?;
    
    // Test connectivity
    config.validate_connectivity().await?;
    
    // Enable encryption if not already enabled
    if !config.encryption_config().encryption_enabled() {
        config.enable_encryption().await?;
    }
    
    // Create a backup
    let backup_path = config.create_backup().await?;
    println!("Backup created at: {}", backup_path.display());
    
    // Use the configuration to set up database connection
    setup_database_pool(&config).await?;
    
    Ok(())
}

async fn setup_database_pool(config: &ProductionDatabaseConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("Setting up database pool:");
    println!("  Database: {}", config.connection_config().database_path().display());
    println!("  Min connections: {}", config.application.pool_settings.min_connections);
    println!("  Max connections: {}", config.application.pool_settings.max_connections);
    println!("  Query timeout: {:?}", config.query_timeout());
    
    if config.should_log_queries() {
        println!("  Query logging: enabled");
    }
    
    if config.application.enable_slow_query_detection {
        println!("  Slow query detection: enabled (threshold: {}ms)", 
                 config.application.slow_query_threshold_ms);
    }
    
    Ok(())
}
```

### Configuration File Example

```toml
# database.toml

[connection]
database_path = "./production.db"
connection_timeout_seconds = 30
max_connections = 100
create_if_missing = true
enable_wal = true
sync_mode = "Normal"

[backup]
mode = "Full"
backup_directory = "./backups"
compression_level = 6
verify_during_creation = true
include_metadata = true
retention_count = 10

[encryption]
enabled = true
algorithm = "AES-256-GCM"
key_derivation = "Argon2"
encrypt_at_rest = true
encrypt_backups = true
key_rotation_days = 90
use_hsm = false

[performance]
cache_size_mb = 512
background_threads = 4
auto_compaction = true
compaction_interval_hours = 24
max_batch_size = 1000
enable_statistics = true

[application]
schema_version = "2.1.0"
query_timeout_seconds = 45
enable_query_logging = false
enable_slow_query_detection = true
slow_query_threshold_ms = 2000

[application.pool_settings]
min_connections = 10
max_connections = 50
idle_timeout_seconds = 900
max_lifetime_seconds = 7200
```

### Testing Example

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};

    #[tokio::test]
    async fn test_database_config_validation() {
        let config = ProductionDatabaseConfig::default();
        assert!(config.validate().is_ok());
    }

    #[tokio::test]
    async fn test_database_config_serialization() {
        let config = ProductionDatabaseConfig::default();
        
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().with_extension("toml");
        
        // Test save/load cycle
        config.save(&path).await.unwrap();
        let loaded_config = ProductionDatabaseConfig::load(&path).await.unwrap();
        
        assert_eq!(config.application.schema_version, loaded_config.application.schema_version);
        assert_eq!(config.application.pool_settings.max_connections, 
                   loaded_config.application.pool_settings.max_connections);
    }

    #[tokio::test]
    async fn test_environment_overrides() {
        use std::env;
        
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        env::set_var("DATABASE_PATH", db_path.display().to_string());
        env::set_var("DATABASE_MAX_CONNECTIONS", "100");
        env::set_var("DATABASE_ENCRYPTION_ENABLED", "true");
        
        let mut config = ProductionDatabaseConfig::default();
        config.apply_env_overrides().await.unwrap();
        
        assert_eq!(config.connection.database_path, db_path);
        assert_eq!(config.application.pool_settings.max_connections, 100);
        assert!(config.encryption.enabled);
        
        // Clean up
        env::remove_var("DATABASE_PATH");
        env::remove_var("DATABASE_MAX_CONNECTIONS");
        env::remove_var("DATABASE_ENCRYPTION_ENABLED");
    }

    #[tokio::test]
    async fn test_backup_creation() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = ProductionDatabaseConfig::default();
        config.backup.backup_directory = temp_dir.path().to_path_buf();
        
        let backup_path = config.create_backup().await.unwrap();
        assert!(backup_path.starts_with(&temp_dir.path()));
        assert!(backup_path.file_name().unwrap().to_str().unwrap().starts_with("backup_"));
    }

    #[test]
    fn test_query_performance_detection() {
        let config = ProductionDatabaseConfig::default();
        
        // Test normal query
        assert!(!config.is_slow_query(Duration::from_millis(500)));
        
        // Test slow query
        assert!(config.is_slow_query(Duration::from_millis(1500)));
    }

    #[test]
    fn test_pool_validation() {
        let mut config = ProductionDatabaseConfig::default();
        
        // Test invalid pool settings
        config.application.pool_settings.min_connections = 10;
        config.application.pool_settings.max_connections = 5; // Invalid: min > max
        
        assert!(config.validate().is_err());
        
        // Fix and test again
        config.application.pool_settings.max_connections = 20;
        assert!(config.validate().is_ok());
    }
}
```

## Key Features Demonstrated

1. **Domain-Specific Traits**: Using [`DatabaseConfig`](../../../../src/config/traits/database.rs:18) for specialized database functionality
2. **Comprehensive Validation**: Multi-level validation for all configuration aspects
3. **Environment Integration**: Complete environment variable override support
4. **Rich Error Context**: Detailed error messages with field-specific information
5. **Event Reporting**: Comprehensive event tracking for monitoring and auditing
6. **Performance Optimization**: Built-in performance tuning and monitoring
7. **Security Features**: Encryption and backup configuration management
8. **Connection Pooling**: Advanced connection pool management
9. **Async Operations**: All I/O operations are properly async
10. **Production Ready**: Comprehensive validation, error handling, and monitoring

This example provides a solid foundation for implementing production-grade database configurations using the DataFold traits system.