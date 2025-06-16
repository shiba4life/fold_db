# Configuration Traits Architecture

Technical architecture documentation for the DataFold configuration traits system.

## Table of Contents

- [System Overview](#system-overview)
- [Trait Hierarchy](#trait-hierarchy)
- [Design Principles](#design-principles)
- [Architecture Patterns](#architecture-patterns)
- [Integration Points](#integration-points)
- [Error Handling Architecture](#error-handling-architecture)
- [Performance Architecture](#performance-architecture)
- [Extension Patterns](#extension-patterns)

## System Overview

The DataFold configuration traits system is a production-ready, type-safe foundation for configuration management that achieved **80.7% duplication reduction** while maintaining high performance and extensibility.

### Key Architecture Goals

1. **Code Reuse**: Eliminate duplication through shared trait implementations
2. **Type Safety**: Compile-time guarantees for configuration correctness
3. **Performance**: Minimal overhead with optimization opportunities
4. **Extensibility**: Easy addition of new configuration types and patterns
5. **Integration**: Seamless integration with existing DataFold systems

### System Components

```text
┌─────────────────────────────────────────────────────────────┐
│                   Configuration Traits System               │
├─────────────────────────────────────────────────────────────┤
│ Core Traits Layer                                          │
│ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐            │
│ │ BaseConfig  │ │ Lifecycle   │ │ Validation  │            │
│ │ (68 LOC)    │ │ (95 LOC)    │ │ (110 LOC)   │            │
│ └─────────────┘ └─────────────┘ └─────────────┘            │
├─────────────────────────────────────────────────────────────┤
│ Integration Layer                                           │
│ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐            │
│ │ CrossPlat   │ │ Reportable  │ │ Observable  │            │
│ │ Config      │ │ Config      │ │ Config      │            │
│ └─────────────┘ └─────────────┘ └─────────────┘            │
├─────────────────────────────────────────────────────────────┤
│ Domain-Specific Layer                                       │
│ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐            │
│ │ Database    │ │ Network     │ │ Logging     │            │
│ │ Config      │ │ Config      │ │ Config      │            │
│ └─────────────┘ └─────────────┘ └─────────────┘            │
├─────────────────────────────────────────────────────────────┤
│ Utility Layer                                               │
│ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐            │
│ │ Config      │ │ Serializa-  │ │ Error       │            │
│ │ Merge       │ │ tion        │ │ Handling    │            │
│ └─────────────┘ └─────────────┘ └─────────────┘            │
└─────────────────────────────────────────────────────────────┘
```

## Trait Hierarchy

### Core Trait Relationships

```rust
// Primary trait hierarchy
pub trait BaseConfig: Debug + Clone + Send + Sync + 'static {
    type Error: std::error::Error + Send + Sync + 'static;
    type Event: Clone + Send + Sync + 'static;
    type TransformTarget: Send + Sync + 'static;
    
    // Core operations
    async fn load(path: &Path) -> Result<Self, Self::Error>;
    fn validate(&self) -> Result<(), Self::Error>;
    fn report_event(&self, event: Self::Event);
    fn as_any(&self) -> &dyn Any;
}

// Lifecycle extension
pub trait ConfigLifecycle: BaseConfig {
    async fn save(&self, path: &Path) -> Result<(), Self::Error>;
    async fn reload(&mut self, path: &Path) -> Result<(), Self::Error>;
    fn get_metadata(&self) -> &ConfigMetadata;
    fn set_metadata(&mut self, metadata: ConfigMetadata);
}

// Enhanced validation
pub trait ConfigValidation: BaseConfig {
    fn validate_with_context(&self, context: &ValidationContext) -> Result<ValidationResult, Self::Error>;
    fn validate_field(&self, field: &str, value: &dyn Any) -> Result<(), Self::Error>;
}

// Event reporting integration
pub trait ConfigReporting: BaseConfig {
    async fn generate_report(&self) -> Result<ConfigReport, Self::Error>;
    fn get_reporting_metadata(&self) -> ReportingMetadata;
}
```

### Integration Trait Relationships

```rust
// Cross-platform support
pub trait CrossPlatformConfig: BaseConfig {
    fn platform_performance_settings(&self) -> PlatformPerformanceSettings;
    fn apply_platform_optimizations(&mut self) -> TraitConfigResult<()>;
}

// PBI 26 unified reporting integration
pub trait ReportableConfig: BaseConfig {
    async fn generate_status_report(&self) -> TraitConfigResult<StatusReport>;
    async fn get_health_metrics(&self) -> TraitConfigResult<HealthMetrics>;
}

// Monitoring and observability
pub trait ObservableConfig: BaseConfig {
    async fn get_telemetry(&self) -> TraitConfigResult<ConfigTelemetry>;
    async fn start_monitoring(&self) -> TraitConfigResult<()>;
    async fn stop_monitoring(&self) -> TraitConfigResult<()>;
}

// Schema validation
pub trait ValidatableConfig: BaseConfig {
    async fn validate_with_traits(&self) -> TraitConfigResult<ValidationResult>;
    fn get_validation_schema(&self) -> ValidationSchema;
}
```

### Domain-Specific Trait Architecture

```rust
// Database configurations
pub trait DatabaseConfig: BaseConfig + ConfigValidation + ConfigLifecycle {
    type ConnectionConfig: Clone + std::fmt::Debug;
    type BackupConfig: Clone + std::fmt::Debug;
    type EncryptionConfig: Clone + std::fmt::Debug;
    type PerformanceConfig: Clone + std::fmt::Debug + Default;
    
    // Database-specific operations
    async fn validate_connectivity(&self) -> TraitConfigResult<()>;
    fn validate_backup_settings(&self) -> TraitConfigResult<()>;
    fn validate_encryption_settings(&self) -> TraitConfigResult<()>;
}

// Network configurations
pub trait NetworkConfig: BaseConfig + ConfigValidation + ConfigLifecycle {
    type SecuritySettings: Clone + std::fmt::Debug;
    type HealthMetrics: Clone + std::fmt::Debug;
    
    // Network-specific operations
    async fn test_connectivity(&self) -> TraitConfigResult<ConnectivityTestResult>;
    async fn get_health_metrics(&self) -> TraitConfigResult<Self::HealthMetrics>;
}

// Logging configurations
pub trait LoggingConfig: BaseConfig + ConfigValidation + ConfigLifecycle {
    type LogLevel: Clone + std::fmt::Debug + PartialEq;
    type OutputConfig: Clone + std::fmt::Debug;
    type PlatformSettings: Clone + std::fmt::Debug + Default;
    
    // Logging-specific operations
    fn default_log_level(&self) -> Self::LogLevel;
    fn output_configs(&self) -> Vec<Self::OutputConfig>;
    fn parse_log_level(&self, level: &str) -> TraitConfigResult<Self::LogLevel>;
}
```

## Design Principles

### 1. Composition Over Inheritance

The trait system uses composition to build complex configurations:

```rust
// Instead of deep inheritance hierarchies
// Use trait composition
pub struct ComplexConfig {
    // Compose multiple concerns
    pub database: impl DatabaseConfig,
    pub network: impl NetworkConfig,
    pub logging: impl LoggingConfig,
}

impl BaseConfig for ComplexConfig {
    // Coordinate between composed elements
}
```

### 2. Associated Types for Type Safety

Associated types ensure compile-time correctness:

```rust
pub trait BaseConfig {
    // Associated types prevent mixing incompatible types
    type Error: std::error::Error + Send + Sync + 'static;
    type Event: Clone + Send + Sync + 'static;
    type TransformTarget: Send + Sync + 'static;
    
    // Methods use associated types for type safety
    fn validate(&self) -> Result<(), Self::Error>;
    fn report_event(&self, event: Self::Event);
}
```

### 3. Async-First Design

All I/O operations are async by default:

```rust
#[async_trait]
pub trait BaseConfig {
    // Async loading for better performance
    async fn load(path: &Path) -> Result<Self, Self::Error>;
}

#[async_trait]
pub trait ConfigLifecycle: BaseConfig {
    // Async operations throughout
    async fn save(&self, path: &Path) -> Result<(), Self::Error>;
    async fn reload(&mut self, path: &Path) -> Result<(), Self::Error>;
}
```

### 4. Error Context Preservation

Rich error context for debugging:

```rust
pub enum TraitConfigError {
    ValidationError {
        field: String,
        message: String,
        context: ValidationContext,  // Rich context
    },
    LoadError {
        path: String,
        source: Box<dyn std::error::Error + Send + Sync>,  // Chained errors
    },
    // ... other variants with context
}
```

### 5. Platform Abstraction

Platform-specific concerns are abstracted:

```rust
pub trait CrossPlatformConfig: BaseConfig {
    fn platform_performance_settings(&self) -> PlatformPerformanceSettings;
    fn apply_platform_optimizations(&mut self) -> TraitConfigResult<()>;
}

// Platform-specific implementations
impl CrossPlatformConfig for DatabaseConfig {
    fn platform_performance_settings(&self) -> PlatformPerformanceSettings {
        #[cfg(windows)]
        return PlatformPerformanceSettings::windows_optimized();
        
        #[cfg(target_os = "macos")]
        return PlatformPerformanceSettings::macos_optimized();
        
        #[cfg(unix)]
        return PlatformPerformanceSettings::unix_optimized();
    }
}
```

## Architecture Patterns

### 1. Trait Object Pattern

For dynamic configuration handling:

```rust
// Type-erased configuration storage
pub struct ConfigManager {
    configs: HashMap<String, Box<dyn BaseConfig<Error = ConfigError, Event = ConfigEvent, TransformTarget = ()>>>,
}

impl ConfigManager {
    pub fn register_config<T>(&mut self, name: String, config: T) 
    where
        T: BaseConfig<Error = ConfigError, Event = ConfigEvent, TransformTarget = ()> + 'static
    {
        self.configs.insert(name, Box::new(config));
    }
    
    pub fn get_config(&self, name: &str) -> Option<&dyn BaseConfig<Error = ConfigError, Event = ConfigEvent, TransformTarget = ()>> {
        self.configs.get(name).map(|b| b.as_ref())
    }
}
```

### 2. Builder Pattern Integration

For complex configuration construction:

```rust
pub struct ConfigBuilder<T> {
    config: T,
    validation_context: ValidationContext,
}

impl<T: BaseConfig> ConfigBuilder<T> {
    pub fn new(config: T) -> Self {
        Self {
            config,
            validation_context: ValidationContext::default(),
        }
    }
    
    pub fn with_validation_context(mut self, context: ValidationContext) -> Self {
        self.validation_context = context;
        self
    }
    
    pub async fn build(self) -> Result<T, T::Error> {
        self.config.validate()?;
        Ok(self.config)
    }
}
```

### 3. Factory Pattern

For configuration creation:

```rust
pub struct ConfigFactory;

impl ConfigFactory {
    pub async fn create_database_config(path: &Path) -> TraitConfigResult<impl DatabaseConfig> {
        let config = StandardDatabaseConfig::load(path).await?;
        Ok(config)
    }
    
    pub async fn create_network_config(path: &Path) -> TraitConfigResult<impl NetworkConfig> {
        let config = StandardNetworkConfig::load(path).await?;
        Ok(config)
    }
    
    pub async fn create_from_type<T>(path: &Path, config_type: &str) -> TraitConfigResult<T>
    where
        T: BaseConfig,
    {
        match config_type {
            "database" => T::load(path).await,
            "network" => T::load(path).await,
            _ => Err(TraitConfigError::UnsupportedConfigType {
                config_type: config_type.to_string(),
            }),
        }
    }
}
```

### 4. Observer Pattern

For configuration change notifications:

```rust
#[async_trait]
pub trait ConfigEvents: BaseConfig {
    async fn on_change<F>(&mut self, listener: F) -> TraitConfigResult<ListenerId>
    where
        F: Fn(&ConfigChangeEvent) + Send + Sync + 'static;
    
    async fn emit_change(&self, event: ConfigChangeEvent) -> TraitConfigResult<()>;
}

// Usage
impl MyConfig {
    pub async fn setup_change_monitoring(&mut self) -> TraitConfigResult<()> {
        self.on_change(|event| {
            log::info!("Configuration changed: {:?}", event);
            // Notify dependent systems
        }).await?;
        
        Ok(())
    }
}
```

## Integration Points

### PBI 26: Unified Reporting Integration

```rust
// Integration with unified reporting system
#[async_trait]
pub trait ReportableConfig: BaseConfig {
    async fn generate_status_report(&self) -> TraitConfigResult<StatusReport> {
        Ok(StatusReport {
            config_name: std::any::type_name::<Self>().to_string(),
            status: self.get_health_status().await?,
            metrics: self.get_metrics().await?,
            last_updated: chrono::Utc::now(),
        })
    }
    
    async fn get_health_status(&self) -> TraitConfigResult<HealthStatus>;
    async fn get_metrics(&self) -> TraitConfigResult<HashMap<String, f64>>;
}

// Integration point
pub struct UnifiedReportingIntegration;

impl UnifiedReportingIntegration {
    pub async fn collect_config_reports<T>(&self, configs: Vec<T>) -> Vec<StatusReport>
    where
        T: ReportableConfig,
    {
        let mut reports = Vec::new();
        for config in configs {
            if let Ok(report) = config.generate_status_report().await {
                reports.push(report);
            }
        }
        reports
    }
}
```

### PBI 27: Cross-Platform Configuration

```rust
// Cross-platform optimization integration
pub trait CrossPlatformConfig: BaseConfig {
    fn platform_performance_settings(&self) -> PlatformPerformanceSettings {
        PlatformPerformanceSettings {
            #[cfg(windows)]
            io_completion_ports: true,
            
            #[cfg(target_os = "macos")]
            use_kqueue: true,
            
            #[cfg(target_os = "linux")]
            use_epoll: true,
            
            memory_optimization: self.get_memory_optimization_level(),
            cpu_affinity: self.get_cpu_affinity_settings(),
        }
    }
    
    fn get_memory_optimization_level(&self) -> MemoryOptimizationLevel;
    fn get_cpu_affinity_settings(&self) -> CpuAffinitySettings;
}
```

### Event System Integration

```rust
// Integration with DataFold event system
pub struct EventSystemIntegration {
    event_bus: Arc<EventBus>,
}

impl EventSystemIntegration {
    pub async fn register_config_events<T>(&self, config: &mut T) -> TraitConfigResult<()>
    where
        T: ConfigEvents,
    {
        let event_bus = self.event_bus.clone();
        
        config.on_change(move |event| {
            let bus = event_bus.clone();
            tokio::spawn(async move {
                bus.publish(ConfigChangeEvent::from(event)).await;
            });
        }).await?;
        
        Ok(())
    }
}
```

## Error Handling Architecture

### Error Type Hierarchy

```rust
// Central error type for all trait operations
pub enum TraitConfigError {
    // Validation errors with rich context
    ValidationError {
        field: String,
        message: String,
        context: ValidationContext,
    },
    
    // I/O operation errors
    LoadError {
        path: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    SaveError {
        path: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    // Serialization errors
    SerializationError {
        format: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    // Network-related errors
    NetworkError {
        operation: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    // Database-related errors
    DatabaseError {
        operation: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    // Integration errors
    IntegrationError {
        system: String,
        message: String,
    },
    
    // Custom errors for extensibility
    Custom {
        error_type: String,
        message: String,
        context: HashMap<String, String>,
    },
}
```

### Error Context System

```rust
// Rich error context for debugging
#[derive(Debug, Clone, Default)]
pub struct ValidationContext {
    pub config_path: Option<PathBuf>,
    pub config_type: Option<String>,
    pub environment: Option<String>,
    pub validation_level: ValidationLevel,
    pub custom_fields: HashMap<String, String>,
}

#[derive(Debug, Clone, Default)]
pub struct ErrorContext {
    pub operation: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub stack_trace: Option<String>,
    pub system_info: SystemInfo,
}

// Error reporting integration
pub trait ErrorReporting {
    fn report_error(&self, error: &TraitConfigError, context: &ErrorContext);
    fn get_error_metrics(&self) -> ErrorMetrics;
}
```

### Error Recovery Patterns

```rust
// Automatic error recovery for common issues
pub struct ErrorRecoverySystem;

impl ErrorRecoverySystem {
    pub async fn attempt_recovery<T>(
        &self,
        error: &TraitConfigError,
        recovery_context: &RecoveryContext,
    ) -> Option<T>
    where
        T: BaseConfig,
    {
        match error {
            TraitConfigError::LoadError { path, .. } => {
                // Try backup locations
                self.try_backup_locations(path).await
            }
            
            TraitConfigError::ValidationError { field, .. } => {
                // Try default values
                self.try_default_values(field).await
            }
            
            _ => None,
        }
    }
    
    async fn try_backup_locations<T>(&self, original_path: &str) -> Option<T>
    where
        T: BaseConfig,
    {
        let backup_paths = [
            format!("{}.backup", original_path),
            format!("{}.bak", original_path),
            format!("{}~", original_path),
        ];
        
        for backup_path in &backup_paths {
            if let Ok(config) = T::load(Path::new(backup_path)).await {
                log::warn!("Recovered configuration from backup: {}", backup_path);
                return Some(config);
            }
        }
        
        None
    }
}
```

## Performance Architecture

### Memory Management

```rust
// Memory-efficient configuration management
pub struct MemoryEfficientConfig<T> {
    // Use Cow for copy-on-write semantics
    data: std::borrow::Cow<'static, T>,
    
    // Lazy-loaded heavy data
    heavy_data: std::sync::Arc<tokio::sync::OnceCell<HeavyConfigData>>,
    
    // Weak references to avoid cycles
    parent_refs: std::sync::Weak<dyn BaseConfig<Error = ConfigError, Event = ConfigEvent, TransformTarget = ()>>,
}

impl<T> MemoryEfficientConfig<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new(data: T) -> Self {
        Self {
            data: std::borrow::Cow::Owned(data),
            heavy_data: std::sync::Arc::new(tokio::sync::OnceCell::new()),
            parent_refs: std::sync::Weak::new(),
        }
    }
    
    pub async fn get_heavy_data(&self) -> &HeavyConfigData {
        self.heavy_data.get_or_init(|| async {
            // Load heavy data only when needed
            HeavyConfigData::load().await
        }).await
    }
}
```

### Caching Architecture

```rust
// Multi-level caching system
pub struct ConfigCache {
    // L1: In-memory cache
    memory_cache: Arc<RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>>,
    
    // L2: Persistent cache
    persistent_cache: Arc<dyn PersistentCache>,
    
    // Cache metrics
    metrics: Arc<CacheMetrics>,
}

impl ConfigCache {
    pub async fn get_or_load<T>(&self, key: &str, loader: impl Future<Output = Result<T, TraitConfigError>>) -> Result<Arc<T>, TraitConfigError>
    where
        T: BaseConfig + 'static,
    {
        // Try L1 cache first
        if let Some(cached) = self.try_memory_cache::<T>(key).await {
            self.metrics.record_hit("memory").await;
            return Ok(cached);
        }
        
        // Try L2 cache
        if let Some(cached) = self.try_persistent_cache::<T>(key).await? {
            self.metrics.record_hit("persistent").await;
            // Populate L1 cache
            self.store_memory_cache(key, cached.clone()).await;
            return Ok(cached);
        }
        
        // Load from source
        self.metrics.record_miss().await;
        let config = loader.await?;
        let arc_config = Arc::new(config);
        
        // Populate both caches
        self.store_memory_cache(key, arc_config.clone()).await;
        self.store_persistent_cache(key, arc_config.clone()).await?;
        
        Ok(arc_config)
    }
}
```

### Performance Monitoring

```rust
// Built-in performance monitoring
#[async_trait]
pub trait ObservableConfig: BaseConfig {
    async fn get_telemetry(&self) -> TraitConfigResult<ConfigTelemetry> {
        Ok(ConfigTelemetry {
            load_time: self.get_load_time(),
            validation_time: self.get_validation_time(),
            memory_usage: self.get_memory_usage(),
            access_patterns: self.get_access_patterns(),
        })
    }
    
    fn get_load_time(&self) -> Duration;
    fn get_validation_time(&self) -> Duration;
    fn get_memory_usage(&self) -> usize;
    fn get_access_patterns(&self) -> AccessPatterns;
}

// Performance benchmarking
pub struct ConfigPerformanceBenchmark;

impl ConfigPerformanceBenchmark {
    pub async fn benchmark_config<T>(&self, config: &T) -> PerformanceReport
    where
        T: BaseConfig + ObservableConfig,
    {
        let start = std::time::Instant::now();
        
        // Benchmark operations
        let load_perf = self.benchmark_load::<T>().await;
        let validation_perf = self.benchmark_validation(config).await;
        let save_perf = self.benchmark_save(config).await;
        
        let total_time = start.elapsed();
        
        PerformanceReport {
            total_time,
            load_performance: load_perf,
            validation_performance: validation_perf,
            save_performance: save_perf,
            memory_efficiency: self.calculate_memory_efficiency(config).await,
        }
    }
}
```

## Extension Patterns

### Custom Trait Definition

Guidelines for adding new traits to the system:

```rust
// Template for new domain-specific traits
#[async_trait]
pub trait CustomDomainConfig: BaseConfig + ConfigValidation + ConfigLifecycle {
    // Associated types for domain-specific data
    type DomainSettings: Clone + std::fmt::Debug;
    type DomainMetrics: Clone + std::fmt::Debug;
    
    // Domain-specific operations
    fn domain_settings(&self) -> &Self::DomainSettings;
    async fn validate_domain_specific(&self) -> TraitConfigResult<()>;
    async fn get_domain_metrics(&self) -> TraitConfigResult<Self::DomainMetrics>;
    
    // Environment variable integration
    async fn apply_env_overrides(&mut self) -> TraitConfigResult<()>;
    
    // Default implementations where appropriate
    fn domain_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
```

### Plugin Architecture

Support for configuration plugins:

```rust
// Plugin trait for extending configurations
pub trait ConfigPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    
    async fn initialize(&self, config: &mut dyn BaseConfig<Error = ConfigError, Event = ConfigEvent, TransformTarget = ()>) -> TraitConfigResult<()>;
    async fn validate(&self, config: &dyn BaseConfig<Error = ConfigError, Event = ConfigEvent, TransformTarget = ()>) -> TraitConfigResult<()>;
    async fn finalize(&self, config: &dyn BaseConfig<Error = ConfigError, Event = ConfigEvent, TransformTarget = ()>) -> TraitConfigResult<()>;
}

// Plugin manager
pub struct ConfigPluginManager {
    plugins: Vec<Box<dyn ConfigPlugin>>,
}

impl ConfigPluginManager {
    pub fn register_plugin(&mut self, plugin: Box<dyn ConfigPlugin>) {
        self.plugins.push(plugin);
    }
    
    pub async fn apply_plugins(&self, config: &mut dyn BaseConfig<Error = ConfigError, Event = ConfigEvent, TransformTarget = ()>) -> TraitConfigResult<()> {
        for plugin in &self.plugins {
            plugin.initialize(config).await?;
            plugin.validate(config).await?;
            plugin.finalize(config).await?;
        }
        Ok(())
    }
}
```

### Macro-Based Code Generation

Reducing boilerplate with procedural macros:

```rust
// Derive macro for common trait implementations
#[derive(BaseConfig)]
#[config(error = "MyError", event = "MyEvent")]
pub struct AutoConfig {
    #[config(validate = "non_empty")]
    pub name: String,
    
    #[config(validate = "range(1..=65535)")]
    pub port: u16,
    
    #[config(env = "AUTO_ENABLED")]
    pub enabled: bool,
}

// Expands to:
// impl BaseConfig for AutoConfig { ... }
// impl ConfigValidation for AutoConfig { ... }
// impl ConfigLifecycle for AutoConfig { ... }
```

---

This architecture documentation provides a comprehensive technical overview of the DataFold configuration traits system, including design principles, patterns, and extension points for future development.

For implementation details, see:
- [Usage Guide](usage-guide.md) for practical implementation
- [Migration Guide](migration-guide.md) for migration from legacy systems
- [Examples](examples/) for working code examples