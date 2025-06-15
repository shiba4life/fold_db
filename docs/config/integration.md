# Configuration Integration Guide

**PBI 27: Cross-Platform Configuration Management System**

This document provides comprehensive integration guides for incorporating the cross-platform configuration system into existing DataFold components and external systems.

## Table of Contents

1. [CLI Configuration Integration](#cli-configuration-integration)
2. [Node Configuration Integration](#node-configuration-integration)
3. [Logging Configuration Integration](#logging-configuration-integration)
4. [Unified Reporting Integration](#unified-reporting-integration)
5. [Migration Procedures](#migration-procedures)
6. [Integration Patterns](#integration-patterns)
7. [Best Practices](#best-practices)

## CLI Configuration Integration

The CLI system has been fully integrated with the cross-platform configuration system, providing seamless authentication profile management and settings persistence.

### Enhanced CLI Configuration Structure

```rust
// CLI configuration now uses EnhancedConfigurationManager
use datafold::config::{
    EnhancedConfigurationManager, EnhancedConfig, ConfigValue,
    create_platform_keystore, PlatformKeystore
};

pub struct EnhancedCliConfig {
    pub profiles: HashMap<String, CliAuthProfile>,
    pub default_profile: Option<String>,
    pub global_settings: CliGlobalSettings,
    pub signing_config: EnhancedSigningConfig,
    pub platform_settings: CliPlatformSettings,
}
```

### Integration Example

```rust
use datafold::cli::config::{CliConfigManager, EnhancedCliConfig};
use datafold::config::EnhancedConfigurationManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize enhanced configuration manager
    let config_manager = EnhancedConfigurationManager::new().await?;
    let cli_manager = CliConfigManager::new(config_manager).await?;
    
    // Load CLI configuration with cross-platform paths
    let cli_config = cli_manager.load_config().await?;
    
    // Access authentication profiles
    if let Some(profile) = cli_config.get_profile("production") {
        println!("Using profile: {}", profile.name);
        
        // Keystore integration for secure credential storage
        if cli_config.platform_settings.use_keystore {
            let keystore = create_platform_keystore().await?;
            let api_key = keystore.retrieve_secret("production_api_key").await?;
            // Use secure credentials...
        }
    }
    
    Ok(())
}
```

### CLI Migration Pattern

```rust
use datafold::config::migration::{ConfigMigrationManager, MigrationStrategy};

async fn migrate_cli_config() -> Result<(), Box<dyn std::error::Error>> {
    let migration_manager = ConfigMigrationManager::new();
    
    // Migrate legacy CLI configuration
    let result = migration_manager.migrate_cli_config().await?;
    
    if result.success {
        println!("✓ CLI configuration migrated successfully");
        println!("  {} profiles migrated", result.items_migrated);
        
        if let Some(backup_path) = result.backup_path {
            println!("  Backup saved to: {}", backup_path.display());
        }
    } else {
        eprintln!("✗ CLI migration failed:");
        for error in &result.errors {
            eprintln!("  {}", error);
        }
    }
    
    Ok(())
}
```

### CLI Configuration Format

**Legacy Format** (`~/.config/datafold/cli_config.json`):
```json
{
  "default_profile": "production",
  "profiles": {
    "production": {
      "name": "production",
      "api_endpoint": "https://api.datafold.com",
      "auth_token": "secret_token"
    }
  }
}
```

**New Format** (`~/.config/datafold/config.toml`):
```toml
[cli]
default_profile = "production"

[cli.profiles.production]
name = "production"
api_endpoint = "https://api.datafold.com"
# auth_token stored securely in keystore

[cli.platform_settings]
use_keystore = true
enable_file_watching = true
cache_credentials = true
```

## Node Configuration Integration

DataFold nodes leverage the enhanced configuration system for optimal performance and platform-specific optimizations.

### Enhanced Node Configuration

```rust
use datafold::datafold_node::config::{EnhancedNodeConfig, NodePlatformSettings};
use datafold::config::{EnhancedConfigurationManager, create_platform_resolver};

pub struct NodeConfigurationIntegration {
    config_manager: EnhancedConfigurationManager,
    node_config: EnhancedNodeConfig,
    platform_paths: Box<dyn PlatformConfigPaths>,
}

impl NodeConfigurationIntegration {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_manager = EnhancedConfigurationManager::new().await?;
        let platform_paths = create_platform_resolver();
        
        // Load enhanced configuration
        let enhanced_config = config_manager.get_enhanced().await?;
        let node_config = Self::extract_node_config(&enhanced_config)?;
        
        Ok(Self {
            config_manager,
            node_config,
            platform_paths,
        })
    }
    
    fn extract_node_config(config: &EnhancedConfig) -> Result<EnhancedNodeConfig, ConfigError> {
        let node_section = config.base.get_section("node")?;
        
        // Convert ConfigValue to EnhancedNodeConfig
        let node_config: EnhancedNodeConfig = serde_json::from_value(
            serde_json::to_value(node_section)?
        )?;
        
        Ok(node_config)
    }
}
```

### Node Configuration Structure

```toml
[node]
storage_path = "data"
network_listen_address = "127.0.0.1:8080"

[node.platform]
use_platform_paths = true
enable_memory_mapping = true
use_atomic_operations = true

[node.paths]
# Automatically resolved using platform-specific paths
data_dir = "auto"
cache_dir = "auto"
logs_dir = "auto"
runtime_dir = "auto"

[node.performance]
cache_size_mb = 256
worker_threads = 4
io_buffer_size = 8192
enable_compression = true

[node.crypto]
encryption_enabled = true
key_rotation_enabled = true
# Master key stored in platform keystore

[node.signature_auth]
enabled = true
require_signatures = true
allowed_signers = ["admin@company.com"]
```

### Node Integration Example

```rust
use datafold::datafold_node::{DataFoldNode, NodeConfig};
use datafold::config::EnhancedConfigurationManager;

async fn start_node_with_enhanced_config() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize configuration system
    let config_manager = EnhancedConfigurationManager::new().await?;
    let enhanced_config = config_manager.get_enhanced().await?;
    
    // Extract node-specific configuration
    let node_config = enhanced_config.base.get_section("node")?;
    let storage_path = node_config.get("storage_path")?.as_string()?;
    let listen_addr = node_config.get("network_listen_address")?.as_string()?;
    
    // Platform-optimized paths
    let platform_paths = create_platform_resolver();
    let data_dir = platform_paths.data_dir()?;
    let logs_dir = platform_paths.logs_dir()?;
    
    // Create node with enhanced configuration
    let mut node = DataFoldNode::new(
        storage_path.into(),
        enhanced_config.security_enhanced.crypto_config.clone(),
    ).await?;
    
    // Start node with platform-optimized settings
    node.start(listen_addr).await?;
    
    Ok(())
}
```

## Logging Configuration Integration

The logging system integrates seamlessly with the cross-platform configuration for consistent path handling and unified configuration management.

### Logging Configuration Structure

```rust
use datafold::logging::config::{LogConfig, GeneralConfig, OutputsConfig};
use datafold::config::{create_platform_resolver, PlatformConfigPaths};

pub struct LoggingConfigurationIntegration {
    platform_paths: Box<dyn PlatformConfigPaths>,
    log_config: LogConfig,
}

impl LoggingConfigurationIntegration {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let platform_paths = create_platform_resolver();
        
        // Load logging configuration from cross-platform config
        let config_manager = EnhancedConfigurationManager::new().await?;
        let enhanced_config = config_manager.get_enhanced().await?;
        let log_config = Self::extract_logging_config(&enhanced_config, &platform_paths)?;
        
        Ok(Self {
            platform_paths,
            log_config,
        })
    }
    
    fn extract_logging_config(
        config: &EnhancedConfig,
        platform_paths: &dyn PlatformConfigPaths,
    ) -> Result<LogConfig, ConfigError> {
        let logging_section = config.base.get_section("logging")?;
        
        // Apply platform-specific path resolution
        let mut log_config: LogConfig = serde_json::from_value(
            serde_json::to_value(logging_section)?
        )?;
        
        // Resolve platform-specific log directory
        if log_config.outputs.file.path.to_string_lossy() == "auto" {
            let logs_dir = platform_paths.logs_dir()?;
            log_config.outputs.file.path = logs_dir.join("datafold.log");
        }
        
        Ok(log_config)
    }
}
```

### Unified Logging Configuration

```toml
[logging]
[logging.general]
default_level = "info"
enable_colors = true
enable_correlation_ids = true
max_correlation_id_length = 32

[logging.outputs]
[logging.outputs.console]
enabled = true
level = "info"
format = "compact"

[logging.outputs.file]
enabled = true
level = "debug"
path = "auto"  # Resolves to platform-specific logs directory
max_size_mb = 100
max_files = 5
rotation_policy = "size"

[logging.outputs.web]
enabled = false
port = 8081
level = "info"

[logging.outputs.structured]
enabled = true
level = "warn"
format = "json"
output_file = "auto"  # Resolves to platform-specific path

[logging.features]
"datafold_node" = "debug"
"network" = "info"
"crypto" = "warn"
"config" = "debug"
```

### Logging Integration Example

```rust
use datafold::logging::{LoggingSystem, init_logging};
use datafold::config::EnhancedConfigurationManager;

async fn initialize_logging_with_config() -> Result<(), Box<dyn std::error::Error>> {
    let config_manager = EnhancedConfigurationManager::new().await?;
    let enhanced_config = config_manager.get_enhanced().await?;
    
    // Extract logging configuration
    let logging_config = enhanced_config.base.get_section("logging")?;
    
    // Initialize logging system with cross-platform paths
    let logging_system = LoggingSystem::from_config(logging_config).await?;
    logging_system.initialize().await?;
    
    // Enable real-time configuration updates
    config_manager.on_change(|event| {
        if event.section == "logging" {
            // Reload logging configuration
            tokio::spawn(async move {
                if let Err(e) = logging_system.reload_config().await {
                    eprintln!("Failed to reload logging config: {}", e);
                }
            });
        }
    }).await?;
    
    Ok(())
}
```

## Unified Reporting Integration

Integration with the unified reporting system (PBI 26) for comprehensive configuration and reporting alignment.

### Reporting Configuration Integration

```rust
use datafold::reporting::{ReportingSystem, ReportingConfig};
use datafold::config::EnhancedConfigurationManager;

pub struct ReportingConfigurationIntegration {
    config_manager: EnhancedConfigurationManager,
    reporting_config: ReportingConfig,
}

impl ReportingConfigurationIntegration {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_manager = EnhancedConfigurationManager::new().await?;
        let enhanced_config = config_manager.get_enhanced().await?;
        
        // Extract reporting configuration
        let reporting_section = config.base.get_section("reporting")?;
        let reporting_config: ReportingConfig = serde_json::from_value(
            serde_json::to_value(reporting_section)?
        )?;
        
        Ok(Self {
            config_manager,
            reporting_config,
        })
    }
    
    pub async fn generate_config_report(&self) -> Result<String, Box<dyn std::error::Error>> {
        let metrics = self.config_manager.get_metrics().await;
        let enhanced_config = self.config_manager.get_enhanced().await?;
        
        // Generate configuration status report
        let report = format!(
            "Configuration System Status Report\n\
             =====================================\n\
             Load Time: {:?}\n\
             Cache Hit Ratio: {:.2}%\n\
             Memory Usage: {:.2} MB\n\
             Platform: {}\n\
             Keystore Available: {}\n\
             File Watching: {}\n\
             Active Sections: {}\n",
            metrics.last_load_time,
            metrics.cache_hit_ratio * 100.0,
            metrics.memory_usage_mb,
            enhanced_config.platform_settings.platform_name,
            enhanced_config.platform_settings.keystore_available,
            enhanced_config.platform_settings.file_watching_enabled,
            enhanced_config.base.sections.len()
        );
        
        Ok(report)
    }
}
```

### Unified Configuration for Reporting

```toml
[reporting]
enabled = true
output_format = "json"
include_config_metrics = true

[reporting.outputs]
console = false
file = true
web_dashboard = true

[reporting.config_monitoring]
track_changes = true
include_performance_metrics = true
audit_security_changes = true
alert_on_validation_errors = true

[reporting.dashboards]
config_status = true
platform_capabilities = true
migration_history = true
security_audit = true
```

## Migration Procedures

Comprehensive migration from legacy configuration systems to the new cross-platform system.

### Automated Migration Script

```rust
use datafold::config::migration::{ConfigMigrationManager, MigrationResult, MigrationStrategy};

pub struct ComprehensiveMigration {
    migration_manager: ConfigMigrationManager,
}

impl ComprehensiveMigration {
    pub fn new() -> Self {
        Self {
            migration_manager: ConfigMigrationManager::new(),
        }
    }
    
    pub async fn migrate_all_systems(&self) -> Result<Vec<MigrationResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        
        // 1. Migrate CLI configurations
        println!("Migrating CLI configurations...");
        match self.migration_manager.migrate_cli_config().await {
            Ok(result) => {
                self.print_migration_result("CLI", &result);
                results.push(result);
            }
            Err(e) => eprintln!("CLI migration failed: {}", e),
        }
        
        // 2. Migrate logging configurations
        println!("Migrating logging configurations...");
        match self.migration_manager.migrate_logging_config().await {
            Ok(result) => {
                self.print_migration_result("Logging", &result);
                results.push(result);
            }
            Err(e) => eprintln!("Logging migration failed: {}", e),
        }
        
        // 3. Migrate unified configurations
        println!("Migrating unified configurations...");
        match self.migration_manager.migrate_unified_config().await {
            Ok(result) => {
                self.print_migration_result("Unified", &result);
                results.push(result);
            }
            Err(e) => eprintln!("Unified migration failed: {}", e),
        }
        
        // 4. Generate migration report
        self.generate_migration_report(&results).await?;
        
        Ok(results)
    }
    
    fn print_migration_result(&self, system: &str, result: &MigrationResult) {
        if result.success {
            println!("✓ {} migration completed", system);
            println!("  Source: {}", result.source_path.display());
            println!("  Target: {}", result.target_path.display());
            println!("  Items migrated: {}", result.items_migrated);
            println!("  Duration: {:?}", result.migration_time);
            
            if !result.warnings.is_empty() {
                println!("  Warnings:");
                for warning in &result.warnings {
                    println!("    ⚠ {}", warning);
                }
            }
        } else {
            println!("✗ {} migration failed", system);
            for error in &result.errors {
                println!("    {}", error);
            }
        }
    }
    
    async fn generate_migration_report(&self, results: &[MigrationResult]) -> Result<(), Box<dyn std::error::Error>> {
        let total_items = results.iter().map(|r| r.items_migrated).sum::<usize>();
        let successful_migrations = results.iter().filter(|r| r.success).count();
        let total_migrations = results.len();
        
        let report = format!(
            "Migration Summary Report\n\
             =======================\n\
             Total systems migrated: {}/{}\n\
             Total configuration items: {}\n\
             Success rate: {:.1}%\n\
             \n\
             Individual Results:\n",
            successful_migrations,
            total_migrations,
            total_items,
            (successful_migrations as f64 / total_migrations as f64) * 100.0
        );
        
        // Save migration report
        let platform_paths = create_platform_resolver();
        let logs_dir = platform_paths.logs_dir()?;
        let report_path = logs_dir.join("migration_report.txt");
        
        std::fs::write(&report_path, report)?;
        println!("Migration report saved to: {}", report_path.display());
        
        Ok(())
    }
}
```

### Migration Command Line Tool

```bash
# Run comprehensive migration
cargo run --bin datafold-migrate -- --all

# Migrate specific system
cargo run --bin datafold-migrate -- --system cli

# Migration with backup
cargo run --bin datafold-migrate -- --all --backup

# Dry run (preview changes)
cargo run --bin datafold-migrate -- --all --dry-run

# Force migration (overwrite existing)
cargo run --bin datafold-migrate -- --all --force
```

## Integration Patterns

### Pattern 1: Lazy Configuration Loading

```rust
use datafold::config::{EnhancedConfigurationManager, EnhancedConfig};
use std::sync::Arc;
use tokio::sync::OnceCell;

pub struct LazyConfigIntegration {
    config_manager: OnceCell<Arc<EnhancedConfigurationManager>>,
}

impl LazyConfigIntegration {
    pub fn new() -> Self {
        Self {
            config_manager: OnceCell::new(),
        }
    }
    
    pub async fn get_config(&self) -> Result<Arc<EnhancedConfig>, Box<dyn std::error::Error>> {
        let manager = self.config_manager.get_or_try_init(|| async {
            let manager = EnhancedConfigurationManager::new().await?;
            Ok(Arc::new(manager))
        }).await?;
        
        let config = manager.get_enhanced().await?;
        Ok(config)
    }
}
```

### Pattern 2: Configuration Dependency Injection

```rust
use datafold::config::{EnhancedConfig, ConfigValue};

pub trait ConfigurableService {
    fn configure(&mut self, config: &EnhancedConfig) -> Result<(), Box<dyn std::error::Error>>;
    fn get_config_section(&self) -> &'static str;
}

pub struct ServiceManager {
    services: Vec<Box<dyn ConfigurableService>>,
    config: Arc<EnhancedConfig>,
}

impl ServiceManager {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_manager = EnhancedConfigurationManager::new().await?;
        let config = config_manager.get_enhanced().await?;
        
        Ok(Self {
            services: Vec::new(),
            config,
        })
    }
    
    pub fn register_service(&mut self, mut service: Box<dyn ConfigurableService>) -> Result<(), Box<dyn std::error::Error>> {
        service.configure(&self.config)?;
        self.services.push(service);
        Ok(())
    }
    
    pub async fn reload_all_configurations(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Reload configuration
        let config_manager = EnhancedConfigurationManager::new().await?;
        self.config = config_manager.get_enhanced().await?;
        
        // Reconfigure all services
        for service in &mut self.services {
            service.configure(&self.config)?;
        }
        
        Ok(())
    }
}
```

### Pattern 3: Real-time Configuration Updates

```rust
use datafold::config::{EnhancedConfigurationManager, ConfigChangeEvent};
use tokio::sync::broadcast;

pub struct ConfigurationEventBus {
    config_manager: EnhancedConfigurationManager,
    event_sender: broadcast::Sender<ConfigChangeEvent>,
}

impl ConfigurationEventBus {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_manager = EnhancedConfigurationManager::new().await?;
        let (event_sender, _) = broadcast::channel(100);
        
        let event_sender_clone = event_sender.clone();
        config_manager.on_change(move |event| {
            let _ = event_sender_clone.send(event);
        }).await?;
        
        Ok(Self {
            config_manager,
            event_sender,
        })
    }
    
    pub fn subscribe(&self) -> broadcast::Receiver<ConfigChangeEvent> {
        self.event_sender.subscribe()
    }
    
    pub async fn get_config(&self) -> Result<Arc<EnhancedConfig>, Box<dyn std::error::Error>> {
        self.config_manager.get_enhanced().await
    }
}

// Usage example
async fn service_with_live_config_updates() -> Result<(), Box<dyn std::error::Error>> {
    let config_bus = ConfigurationEventBus::new().await?;
    let mut event_receiver = config_bus.subscribe();
    
    // Initial configuration load
    let mut current_config = config_bus.get_config().await?;
    
    // Listen for configuration changes
    tokio::spawn(async move {
        while let Ok(event) = event_receiver.recv().await {
            println!("Configuration changed: {:?}", event);
            
            // Reload configuration
            if let Ok(new_config) = config_bus.get_config().await {
                current_config = new_config;
                // Apply new configuration...
            }
        }
    });
    
    Ok(())
}
```

## Best Practices

### 1. Configuration Validation

```rust
use datafold::config::{ConfigValue, ConfigValueSchema};

fn validate_service_config(config: &ConfigValue) -> Result<(), Box<dyn std::error::Error>> {
    // Define configuration schema
    let schema = ConfigValueSchema::object(true)
        .with_field("host", ConfigValueSchema::string(true))
        .with_field("port", ConfigValueSchema::integer(true))
        .with_field("ssl_enabled", ConfigValueSchema::boolean(false).with_default(ConfigValue::Bool(true)));
    
    // Validate configuration
    config.validate(&schema)?;
    
    // Additional business logic validation
    let port = config.get("port")?.as_integer()?;
    if port < 1024 || port > 65535 {
        return Err("Port must be between 1024 and 65535".into());
    }
    
    Ok(())
}
```

### 2. Secure Configuration Handling

```rust
use datafold::config::{create_platform_keystore, PlatformKeystore};

async fn handle_sensitive_config() -> Result<(), Box<dyn std::error::Error>> {
    let config_manager = EnhancedConfigurationManager::new().await?;
    let enhanced_config = config_manager.get_enhanced().await?;
    
    // Store sensitive values in keystore
    if enhanced_config.platform_settings.keystore_available {
        let keystore = create_platform_keystore().await?;
        
        // Store API keys securely
        keystore.store_secret("api_key", b"sensitive_api_key").await?;
        keystore.store_secret("db_password", b"database_password").await?;
        
        // Retrieve when needed
        let api_key = keystore.retrieve_secret("api_key").await?;
        // Use API key...
    }
    
    Ok(())
}
```

### 3. Performance Optimization

```rust
use datafold::config::EnhancedConfigurationManager;
use std::sync::Arc;

// Singleton pattern for configuration manager
pub struct ConfigSingleton {
    manager: Arc<EnhancedConfigurationManager>,
}

impl ConfigSingleton {
    pub async fn instance() -> &'static ConfigSingleton {
        static INSTANCE: OnceCell<ConfigSingleton> = OnceCell::new();
        
        INSTANCE.get_or_init(|| async {
            let manager = EnhancedConfigurationManager::new().await
                .expect("Failed to initialize configuration manager");
            ConfigSingleton {
                manager: Arc::new(manager),
            }
        }).await
    }
    
    pub async fn get_config(&self) -> Result<Arc<EnhancedConfig>, Box<dyn std::error::Error>> {
        self.manager.get_enhanced().await
    }
}
```

### 4. Error Handling and Recovery

```rust
use datafold::config::{ConfigError, ConfigErrorContext};

async fn robust_config_loading() -> Result<EnhancedConfig, Box<dyn std::error::Error>> {
    let config_manager = EnhancedConfigurationManager::new().await?;
    
    match config_manager.get_enhanced().await {
        Ok(config) => Ok(config),
        Err(ConfigError::NotFound(_)) => {
            // Create default configuration
            println!("Configuration not found, creating default...");
            let default_config = create_default_config();
            config_manager.set_enhanced(default_config.clone()).await?;
            Ok(default_config)
        }
        Err(ConfigError::ParseError(msg)) => {
            // Attempt to migrate from backup
            println!("Configuration parse error, attempting recovery...");
            attempt_config_recovery(&config_manager).await
        }
        Err(e) => {
            // Create error context for better debugging
            let context = ConfigErrorContext::new(e, "load_configuration".to_string())
                .with_component("integration");
            Err(context.into())
        }
    }
}

async fn attempt_config_recovery(manager: &EnhancedConfigurationManager) -> Result<EnhancedConfig, Box<dyn std::error::Error>> {
    // Try to find backup configuration
    let platform_paths = create_platform_resolver();
    let config_dir = platform_paths.config_dir()?;
    let backup_pattern = config_dir.join("config.toml.backup.*");
    
    // Find most recent backup
    if let Some(backup_path) = find_most_recent_backup(&backup_pattern)? {
        println!("Found backup configuration: {}", backup_path.display());
        
        // Load from backup
        let backup_provider = TomlConfigProvider::with_path(&backup_path);
        let backup_config = backup_provider.load().await?;
        
        // Save as current configuration
        manager.set_enhanced(backup_config.clone()).await?;
        Ok(backup_config)
    } else {
        // No backup found, create minimal configuration
        println!("No backup found, creating minimal configuration...");
        let minimal_config = create_minimal_config();
        manager.set_enhanced(minimal_config.clone()).await?;
        Ok(minimal_config)
    }
}
```

## Related Documentation

- [Architecture](architecture.md) - System architecture and design
- [API Reference](api.md) - Complete API documentation
- [Deployment Guide](deployment.md) - Deployment and migration procedures
- [Security Guide](security.md) - Security features and best practices