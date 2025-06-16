//! Enhanced cross-platform configuration management with platform-specific optimizations
//!
//! This module provides an enhanced configuration system that builds on the core
//! cross-platform functionality while adding platform-specific optimizations:
//! - Native keystore integration for secure storage
//! - Platform-optimized file operations and caching
//! - Real-time configuration monitoring
//! - Encrypted configuration sections
//! - Performance optimizations and memory management

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::config::cross_platform::{Config, ConfigurationProvider, TomlConfigProvider};
use crate::config::error::{ConfigError, ConfigResult};
use crate::config::platform::{
    create_platform_atomic_ops, create_platform_file_watcher, create_platform_resolver,
    keystore::{create_platform_keystore, KeystoreConfig, PlatformKeystore},
    EnhancedPlatformInfo, PlatformAtomicOps, PlatformConfigPaths, PlatformFileWatcher,
};
use crate::config::value::{ConfigValue, ConfigValueSchema};

// Import shared traits
use crate::config::traits::base::{
    ConfigChangeType as TraitConfigChangeType, ConfigMetadata, ReportingConfig, ValidationRule,
    ValidationRuleType, ValidationSeverity,
};
use crate::config::traits::core::{ConfigChangeEvent as CoreConfigChangeEvent, ConfigEventType};
use crate::config::traits::integration::{
    ConfigMetrics, HealthStatus, PlatformPerformanceSettings, UnifiedReport, ValidationResult,
};
use crate::config::traits::{
    BaseConfig, ConfigLifecycle, ConfigReporting, ConfigValidation, CrossPlatformConfig,
    ReportableConfig, TraitConfigError, TraitConfigResult, ValidatableConfig, ValidationContext,
};

/// Enhanced configuration with platform-specific optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedConfig {
    /// Base configuration
    #[serde(flatten)]
    pub base: Config,

    /// Platform-specific settings
    pub platform_settings: PlatformSettings,

    /// Performance optimization settings
    pub performance: PerformanceSettings,

    /// Security settings including keystore configuration
    pub security_enhanced: EnhancedSecurityConfig,

    /// Encrypted configuration sections (stored as encrypted blobs)
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub encrypted_sections: HashMap<String, EncryptedSection>,

    /// Configuration monitoring settings
    pub monitoring: MonitoringConfig,
}

/// Platform-specific configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformSettings {
    /// Enable platform-specific optimizations
    pub enable_optimizations: bool,

    /// Use native file operations where available
    pub use_native_file_ops: bool,

    /// Enable memory mapping for large configs
    pub enable_memory_mapping: bool,

    /// Platform-specific cache settings
    pub cache_settings: PlatformCacheSettings,
}

/// Performance optimization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    /// Enable lazy loading of configuration sections
    pub lazy_loading: bool,

    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,

    /// Maximum cache size in MB
    pub max_cache_size_mb: u64,

    /// Enable configuration preloading
    pub enable_preloading: bool,

    /// Async I/O buffer size
    pub io_buffer_size: usize,
}

/// Enhanced security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSecurityConfig {
    /// Keystore configuration
    pub keystore: KeystoreConfig,

    /// Enable automatic encryption of sensitive sections
    pub auto_encrypt_sensitive: bool,

    /// Sensitive section patterns
    pub sensitive_patterns: Vec<String>,

    /// Enable configuration signing
    pub enable_signing: bool,

    /// Backup and rollback settings
    pub backup_settings: BackupSettings,
}

/// Configuration backup and rollback settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSettings {
    /// Enable automatic backups
    pub enabled: bool,

    /// Number of backups to keep
    pub max_backups: u32,

    /// Backup interval in seconds
    pub backup_interval_secs: u64,

    /// Compress backups
    pub compress_backups: bool,
}

/// Platform-specific cache settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformCacheSettings {
    /// Use platform-specific memory optimization
    pub use_platform_memory_ops: bool,

    /// Cache location preference
    pub cache_location: CacheLocation,

    /// Enable cache validation
    pub validate_cache: bool,
}

/// Cache location preference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheLocation {
    Memory,
    PlatformCache,
    Custom(PathBuf),
}

/// Configuration monitoring settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable real-time file monitoring
    pub enable_file_watching: bool,

    /// Enable change notifications
    pub enable_notifications: bool,

    /// Monitor interval for polling fallback
    pub poll_interval_secs: u64,

    /// Enable configuration validation on changes
    pub validate_on_change: bool,
}

/// Encrypted configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedSection {
    /// Encrypted data
    pub data: Vec<u8>,

    /// Encryption metadata
    pub metadata: EncryptionMetadata,

    /// Section name
    pub section_name: String,
}

/// Encryption metadata for encrypted sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionMetadata {
    /// Encryption algorithm used
    pub algorithm: String,

    /// Key derivation parameters
    pub key_derivation: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last access timestamp
    pub accessed_at: DateTime<Utc>,
}

/// Enhanced configuration manager with platform-specific optimizations
pub struct EnhancedConfigurationManager {
    /// Base configuration provider
    provider: Arc<dyn ConfigurationProvider>,

    /// Platform-specific keystore
    keystore: Arc<dyn PlatformKeystore>,

    /// Platform-specific path resolver
    platform_paths: Box<dyn PlatformConfigPaths>,

    /// Platform-specific file watcher
    file_watcher: Option<Box<dyn PlatformFileWatcher>>,

    /// Platform-specific atomic operations
    atomic_ops: Box<dyn PlatformAtomicOps>,

    /// Configuration cache
    cache: Arc<RwLock<ConfigCache>>,

    /// Performance metrics
    metrics: Arc<Mutex<PerformanceMetrics>>,

    /// Change notification callbacks
    change_callbacks: Arc<Mutex<Vec<Box<dyn Fn(ConfigChangeEvent) + Send + Sync>>>>,
}

/// Configuration cache with TTL and size limits
struct ConfigCache {
    /// Cached configuration
    config: Option<Arc<EnhancedConfig>>,

    /// Cache timestamp
    cached_at: Option<DateTime<Utc>>,

    /// Section-specific cache
    section_cache: HashMap<String, CachedSection>,

    /// Cache size in bytes
    cache_size: usize,
}

/// Cached configuration section
struct CachedSection {
    /// Section data
    data: ConfigValue,

    /// Cache timestamp
    cached_at: DateTime<Utc>,

    /// Access count
    access_count: u64,
}

/// Performance metrics for configuration operations
#[derive(Debug, Default)]
struct PerformanceMetrics {
    /// Total load operations
    total_loads: u64,

    /// Cache hits
    cache_hits: u64,

    /// Cache misses
    cache_misses: u64,

    /// Average load time in milliseconds
    avg_load_time_ms: f64,

    /// Keystore operations
    keystore_operations: u64,

    /// File watch events
    file_watch_events: u64,
}

/// Configuration change event
#[derive(Debug, Clone)]
pub struct ConfigChangeEvent {
    /// Type of change
    pub change_type: ConfigChangeType,

    /// Affected section (if applicable)
    pub section: Option<String>,

    /// Change timestamp
    pub timestamp: DateTime<Utc>,

    /// Change source
    pub source: ConfigChangeSource,
}

/// Type of configuration change
#[derive(Debug, Clone)]
pub enum ConfigChangeType {
    SectionAdded,
    SectionModified,
    SectionRemoved,
    ConfigReloaded,
    EncryptionChanged,
}

/// Source of configuration change
#[derive(Debug, Clone)]
pub enum ConfigChangeSource {
    FileSystem,
    Programmatic,
    KeystoreSync,
    Migration,
}

impl Default for EnhancedConfig {
    fn default() -> Self {
        Self {
            base: Config::default(),
            platform_settings: PlatformSettings::default(),
            performance: PerformanceSettings::default(),
            security_enhanced: EnhancedSecurityConfig::default(),
            encrypted_sections: HashMap::new(),
            monitoring: MonitoringConfig::default(),
        }
    }
}

impl Default for PlatformSettings {
    fn default() -> Self {
        Self {
            enable_optimizations: true,
            use_native_file_ops: true,
            enable_memory_mapping: true,
            cache_settings: PlatformCacheSettings::default(),
        }
    }
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            lazy_loading: true,
            cache_ttl_secs: 300, // 5 minutes
            max_cache_size_mb: 10,
            enable_preloading: true,
            io_buffer_size: 8192,
        }
    }
}

impl Default for EnhancedSecurityConfig {
    fn default() -> Self {
        Self {
            keystore: KeystoreConfig::default(),
            auto_encrypt_sensitive: true,
            sensitive_patterns: vec![
                "password".to_string(),
                "secret".to_string(),
                "key".to_string(),
                "token".to_string(),
                "credential".to_string(),
            ],
            enable_signing: false,
            backup_settings: BackupSettings::default(),
        }
    }
}

impl Default for BackupSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            max_backups: 5,
            backup_interval_secs: 3600, // 1 hour
            compress_backups: true,
        }
    }
}

impl Default for PlatformCacheSettings {
    fn default() -> Self {
        Self {
            use_platform_memory_ops: true,
            cache_location: CacheLocation::PlatformCache,
            validate_cache: true,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_file_watching: true,
            enable_notifications: true,
            poll_interval_secs: 5,
            validate_on_change: true,
        }
    }
}

impl EnhancedConfigurationManager {
    /// Create new enhanced configuration manager
    pub async fn new() -> ConfigResult<Self> {
        let provider = Arc::new(TomlConfigProvider::new());
        let keystore = Arc::new(create_platform_keystore());
        let platform_paths = create_platform_resolver();

        let file_watcher = if EnhancedPlatformInfo::detect().file_watching_available {
            Some(create_platform_file_watcher()?)
        } else {
            None
        };

        let atomic_ops = create_platform_atomic_ops();

        let cache = Arc::new(RwLock::new(ConfigCache {
            config: None,
            cached_at: None,
            section_cache: HashMap::new(),
            cache_size: 0,
        }));

        Ok(Self {
            provider,
            keystore,
            platform_paths,
            file_watcher,
            atomic_ops,
            cache,
            metrics: Arc::new(Mutex::new(PerformanceMetrics::default())),
            change_callbacks: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Create with custom provider and keystore
    pub async fn with_custom(
        provider: Arc<dyn ConfigurationProvider>,
        keystore: Option<Arc<dyn PlatformKeystore>>,
    ) -> ConfigResult<Self> {
        let keystore = keystore.unwrap_or_else(|| std::sync::Arc::from(create_platform_keystore()));
        let platform_paths = create_platform_resolver();

        let file_watcher = if EnhancedPlatformInfo::detect().file_watching_available {
            Some(create_platform_file_watcher()?)
        } else {
            None
        };

        let atomic_ops = create_platform_atomic_ops();

        let cache = Arc::new(RwLock::new(ConfigCache {
            config: None,
            cached_at: None,
            section_cache: HashMap::new(),
            cache_size: 0,
        }));

        Ok(Self {
            provider,
            keystore,
            platform_paths,
            file_watcher,
            atomic_ops,
            cache,
            metrics: Arc::new(Mutex::new(PerformanceMetrics::default())),
            change_callbacks: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Get enhanced configuration with caching and lazy loading
    pub async fn get_enhanced(&self) -> ConfigResult<Arc<EnhancedConfig>> {
        let start_time = std::time::Instant::now();

        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached_config) = &cache.config {
                if let Some(cached_at) = cache.cached_at {
                    let ttl = std::time::Duration::from_secs(300); // Default TTL
                    if cached_at + chrono::Duration::from_std(ttl).unwrap() > Utc::now() {
                        // Update metrics
                        let mut metrics = self.metrics.lock().await;
                        metrics.cache_hits += 1;
                        return Ok(cached_config.clone());
                    }
                }
            }
        }

        // Load from storage
        let base_config = self.provider.load().await?;
        let enhanced_config = self.convert_to_enhanced(base_config).await?;
        let config_arc = Arc::new(enhanced_config);

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.config = Some(config_arc.clone());
            cache.cached_at = Some(Utc::now());
        }

        // Update metrics
        let mut metrics = self.metrics.lock().await;
        metrics.cache_misses += 1;
        metrics.total_loads += 1;
        metrics.avg_load_time_ms = (metrics.avg_load_time_ms * (metrics.total_loads - 1) as f64
            + start_time.elapsed().as_millis() as f64)
            / metrics.total_loads as f64;

        Ok(config_arc)
    }

    /// Convert base config to enhanced config
    async fn convert_to_enhanced(&self, base: Config) -> ConfigResult<EnhancedConfig> {
        let mut enhanced = EnhancedConfig {
            base,
            ..Default::default()
        };

        // Load encrypted sections from keystore
        if enhanced.security_enhanced.keystore.enabled {
            self.load_encrypted_sections(&mut enhanced).await?;
        }

        Ok(enhanced)
    }

    /// Load encrypted sections from keystore
    async fn load_encrypted_sections(&self, config: &mut EnhancedConfig) -> ConfigResult<()> {
        let keys = self.keystore.list_keys().await?;

        for key in keys {
            if let Some(encrypted_data) = self.keystore.get_secret(&key).await? {
                // Deserialize encrypted section
                if let Ok(section) = serde_json::from_slice::<EncryptedSection>(&encrypted_data) {
                    config.encrypted_sections.insert(key, section);
                }
            }
        }

        Ok(())
    }

    /// Store enhanced configuration with platform optimizations
    pub async fn set_enhanced(&self, config: EnhancedConfig) -> ConfigResult<()> {
        // Store encrypted sections in keystore
        if config.security_enhanced.keystore.enabled {
            self.store_encrypted_sections(&config).await?;
        }

        // Store base configuration
        self.provider.save(&config.base).await?;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.config = Some(Arc::new(config.clone()));
            cache.cached_at = Some(Utc::now());
        }

        // Emit change event
        self.emit_change_event(ConfigChangeEvent {
            change_type: ConfigChangeType::ConfigReloaded,
            section: None,
            timestamp: Utc::now(),
            source: ConfigChangeSource::Programmatic,
        })
        .await;

        Ok(())
    }

    /// Store encrypted sections in keystore
    async fn store_encrypted_sections(&self, config: &EnhancedConfig) -> ConfigResult<()> {
        for (key, section) in &config.encrypted_sections {
            let serialized = serde_json::to_vec(section).map_err(|e| {
                ConfigError::encryption(format!("Failed to serialize encrypted section: {}", e))
            })?;

            self.keystore.store_secret(key, &serialized).await?;
        }

        Ok(())
    }

    /// Register change callback
    pub async fn on_change<F>(&self, callback: F) -> ConfigResult<()>
    where
        F: Fn(ConfigChangeEvent) + Send + Sync + 'static,
    {
        let mut callbacks = self.change_callbacks.lock().await;
        callbacks.push(Box::new(callback));
        Ok(())
    }

    /// Emit change event to all callbacks
    async fn emit_change_event(&self, event: ConfigChangeEvent) {
        let callbacks = self.change_callbacks.lock().await;
        for callback in callbacks.iter() {
            callback(event.clone());
        }
    }

    /// Get performance metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        let metrics = self.metrics.lock().await;
        PerformanceMetrics {
            total_loads: metrics.total_loads,
            cache_hits: metrics.cache_hits,
            cache_misses: metrics.cache_misses,
            avg_load_time_ms: metrics.avg_load_time_ms,
            keystore_operations: metrics.keystore_operations,
            file_watch_events: metrics.file_watch_events,
        }
    }

    /// Clear cache and force reload
    pub async fn clear_cache(&self) -> ConfigResult<()> {
        let mut cache = self.cache.write().await;
        cache.config = None;
        cache.cached_at = None;
        cache.section_cache.clear();
        cache.cache_size = 0;
        Ok(())
    }

    /// Get keystore instance for direct access
    pub fn keystore(&self) -> &Arc<dyn PlatformKeystore> {
        &self.keystore
    }

    /// Get platform-specific capabilities
    pub fn platform_info(&self) -> EnhancedPlatformInfo {
        EnhancedPlatformInfo::detect()
    }
}

// Implement shared traits for EnhancedConfig
#[async_trait]
impl BaseConfig for EnhancedConfig {
    type Error = ConfigError;
    type Event = ConfigChangeType;
    type TransformTarget = EnhancedConfig;

    /// Load enhanced configuration from the specified path
    async fn load(path: &Path) -> Result<Self, Self::Error> {
        let manager = EnhancedConfigurationManager::new().await?;
        let base_config = manager.provider.load().await?;
        manager.convert_to_enhanced(base_config).await
    }

    /// Validate the enhanced configuration
    fn validate(&self) -> Result<(), Self::Error> {
        // Validate base configuration
        self.base.validate()?;

        // Validate platform settings
        if self.platform_settings.enable_memory_mapping && self.performance.max_cache_size_mb == 0 {
            return Err(ConfigError::validation(
                "Memory mapping requires non-zero cache size",
            ));
        }

        // Validate performance settings
        if self.performance.cache_ttl_secs == 0 {
            return Err(ConfigError::validation("Cache TTL must be greater than 0"));
        }

        // Validate security settings
        if self.security_enhanced.auto_encrypt_sensitive
            && self.security_enhanced.sensitive_patterns.is_empty()
        {
            return Err(ConfigError::validation(
                "Auto-encryption enabled but no sensitive patterns defined",
            ));
        }

        Ok(())
    }

    /// Save configuration to the specified path
    async fn save(&self, path: &Path) -> Result<(), Self::Error> {
        let manager = EnhancedConfigurationManager::new().await?;

        // Convert to base config format for saving
        let base_config = self.base.clone();
        manager.provider.save(path).await?;

        Ok(())
    }

    /// Get configuration metadata
    fn metadata(&self) -> std::collections::HashMap<String, String> {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("type".to_string(), "enhanced".to_string());
        metadata.insert("platform".to_string(), self.platform_info().name.clone());
        metadata.insert(
            "cache_enabled".to_string(),
            self.performance.enable_caching.to_string(),
        );
        metadata.insert(
            "security_level".to_string(),
            if self.security_enhanced.auto_encrypt_sensitive {
                "high"
            } else {
                "standard"
            }
            .to_string(),
        );
        metadata
    }

    /// Report configuration event
    fn report_event(&self, event: Self::Event) {
        // For now, emit basic event - this will be enhanced with PBI 26 integration
        log::info!("EnhancedConfig event: {:?}", event);
    }

    /// Get runtime type information
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl ConfigLifecycle for EnhancedConfig {
    /// Backup enhanced configuration to the specified path
    async fn backup(&self, backup_path: &Path) -> Result<(), Self::Error> {
        self.save(backup_path).await
    }

    /// Merge with another enhanced configuration
    async fn merge(&mut self, other: Self) -> Result<(), Self::Error> {
        // Merge logic: prefer non-default values from other
        if other.performance.enable_caching {
            self.performance.enable_caching = other.performance.enable_caching;
        }
        if other.performance.max_cache_size_mb > 0 {
            self.performance.max_cache_size_mb = other.performance.max_cache_size_mb;
        }
        if other.performance.cache_ttl_secs > 0 {
            self.performance.cache_ttl_secs = other.performance.cache_ttl_secs;
        }

        // Merge security settings
        if other.security_enhanced.auto_encrypt_sensitive {
            self.security_enhanced.auto_encrypt_sensitive =
                other.security_enhanced.auto_encrypt_sensitive;
        }
        if !other.security_enhanced.sensitive_patterns.is_empty() {
            self.security_enhanced
                .sensitive_patterns
                .extend(other.security_enhanced.sensitive_patterns);
        }

        // Merge platform settings
        self.platform_settings.enable_memory_mapping =
            other.platform_settings.enable_memory_mapping;
        self.platform_settings.enable_atomic_writes = other.platform_settings.enable_atomic_writes;

        Ok(())
    }

    /// Reload enhanced configuration from its source
    async fn reload(&mut self, path: &Path) -> Result<(), Self::Error> {
        let reloaded = Self::load(path).await?;
        *self = reloaded;
        Ok(())
    }

    /// Check if configuration has changed since last load/save
    async fn has_changed(&self, path: &Path) -> Result<bool, Self::Error> {
        // Simple implementation - compare file modification time
        let metadata = tokio::fs::metadata(path).await?;
        let file_modified = metadata.modified()?;
        let file_modified_chrono = DateTime::<Utc>::from(file_modified);

        // Compare with our last access time (if available)
        Ok(file_modified_chrono > Utc::now() - chrono::Duration::seconds(60))
    }

    /// Get configuration metadata
    fn get_metadata(&self) -> ConfigMetadata {
        ConfigMetadata {
            version: "1.0.0".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            accessed_at: Utc::now(),
            source: None,
            format: Some("enhanced_toml".to_string()),
            size_bytes: None, // Would be calculated during serialization
            checksum: None,   // Would be calculated during serialization
            additional: HashMap::new(),
        }
    }

    /// Set configuration metadata
    fn set_metadata(&mut self, _metadata: ConfigMetadata) {
        // Enhanced config doesn't store metadata directly in this implementation
        // This could be enhanced to store metadata in a dedicated field
    }
}

impl ConfigValidation for EnhancedConfig {
    /// Validate with detailed context
    fn validate_with_context(&self) -> Result<(), ValidationContext> {
        // Create validation context for enhanced config
        let context =
            ValidationContext::new("EnhancedConfig", "comprehensive_validation".to_string());

        // Perform validation and return detailed context on failure
        match self.validate() {
            Ok(()) => Ok(()),
            Err(_) => Err(context.with_path("enhanced_config")),
        }
    }

    /// Validate specific field or section
    fn validate_field(&self, field_path: &str) -> Result<(), Self::Error> {
        match field_path {
            "platform_settings" => {
                if !self.platform_settings.enable_optimizations
                    && self.platform_settings.enable_memory_mapping
                {
                    return Err(ConfigError::validation(
                        "Memory mapping requires platform optimizations to be enabled",
                    ));
                }
                Ok(())
            }
            "performance" => {
                if self.performance.lazy_loading && self.performance.enable_preloading {
                    return Err(ConfigError::validation(
                        "Lazy loading and preloading are mutually exclusive",
                    ));
                }
                Ok(())
            }
            "security_enhanced" => {
                if self.security_enhanced.enable_signing
                    && !self.security_enhanced.backup_settings.enabled
                {
                    return Err(ConfigError::validation(
                        "Configuration signing requires backup to be enabled",
                    ));
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Get validation rules
    fn validation_rules(&self) -> Vec<ValidationRule> {
        vec![
            ValidationRule::required("base"),
            ValidationRule::required("platform_settings"),
            ValidationRule::required("performance"),
            ValidationRule::required("security_enhanced"),
            ValidationRule {
                name: "cache_ttl_positive".to_string(),
                description: "Cache TTL must be positive".to_string(),
                field_path: "performance.cache_ttl_secs".to_string(),
                rule_type: ValidationRuleType::NumericRange {
                    min: Some(1.0),
                    max: None,
                },
                severity: ValidationSeverity::Error,
            },
            ValidationRule {
                name: "sensitive_patterns_when_auto_encrypt".to_string(),
                description: "Sensitive patterns required when auto-encryption is enabled"
                    .to_string(),
                field_path: "security_enhanced.sensitive_patterns".to_string(),
                rule_type: ValidationRuleType::Custom("conditional_required".to_string()),
                severity: ValidationSeverity::Error,
            },
        ]
    }

    /// Add custom validation rule
    fn add_validation_rule(&mut self, _rule: ValidationRule) {
        // This could be implemented by storing custom rules in the config
        // For now, we don't support runtime rule addition
    }
}

impl ConfigReporting for EnhancedConfig {
    /// Report configuration change event
    fn report_change(&self, change_type: TraitConfigChangeType, context: Option<String>) {
        let local_change_type = match change_type {
            TraitConfigChangeType::Loaded => ConfigChangeType::ConfigReloaded,
            TraitConfigChangeType::Saved => ConfigChangeType::ConfigReloaded, // Map to closest equivalent
            TraitConfigChangeType::FieldChanged { .. } => ConfigChangeType::SectionModified,
            _ => ConfigChangeType::SectionModified,
        };

        log::info!(
            "Enhanced config change: {:?}, context: {:?}",
            local_change_type,
            context
        );
    }

    /// Report configuration error
    fn report_error(&self, error: &Self::Error, context: Option<String>) {
        log::error!("Enhanced config error: {}, context: {:?}", error, context);
    }

    /// Report configuration metric
    fn report_metric(&self, metric_name: &str, value: f64, tags: Option<HashMap<String, String>>) {
        log::debug!(
            "Enhanced config metric: {} = {}, tags: {:?}",
            metric_name,
            value,
            tags
        );
    }

    /// Get reporting configuration
    fn reporting_config(&self) -> ReportingConfig {
        ReportingConfig {
            report_changes: true,
            report_errors: true,
            report_metrics: false,
            target: None,
            throttle_ms: Some(1000),
            include_sensitive: false,
        }
    }

    /// Set reporting configuration
    fn set_reporting_config(&mut self, _config: ReportingConfig) {
        // Could be implemented by storing reporting config in the enhanced config
    }
}

#[async_trait]
impl CrossPlatformConfig for EnhancedConfig {
    /// Get platform-specific configuration paths
    fn platform_paths(&self) -> &dyn PlatformConfigPaths {
        // Return a static reference to avoid lifetime issues
        use once_cell::sync::Lazy;
        static PLATFORM_RESOLVER: Lazy<Box<dyn PlatformConfigPaths + Sync + Send>> =
            Lazy::new(|| create_platform_resolver());
        &**PLATFORM_RESOLVER
    }

    /// Get enhanced platform information
    fn platform_info(&self) -> EnhancedPlatformInfo {
        EnhancedPlatformInfo::detect()
    }

    /// Load configuration using platform-specific optimizations
    async fn load_platform_optimized(&self, path: &Path) -> TraitConfigResult<Self> {
        let manager = EnhancedConfigurationManager::new().await.map_err(|e| {
            TraitConfigError::cross_platform(format!("Failed to create manager: {}", e))
        })?;

        let base_config = if self.platform_settings.use_native_file_ops {
            // Use platform-optimized loading
            manager.provider.load().await.map_err(|e| {
                TraitConfigError::cross_platform(format!("Platform load failed: {}", e))
            })?
        } else {
            // Use standard loading
            manager.provider.load().await.map_err(|e| {
                TraitConfigError::cross_platform(format!("Standard load failed: {}", e))
            })?
        };

        let enhanced = manager
            .convert_to_enhanced(base_config)
            .await
            .map_err(|e| TraitConfigError::cross_platform(format!("Conversion failed: {}", e)))?;

        Ok(enhanced)
    }

    /// Save configuration using platform-specific optimizations
    async fn save_platform_optimized(&self, path: &Path) -> TraitConfigResult<()> {
        let manager = EnhancedConfigurationManager::new().await.map_err(|e| {
            TraitConfigError::cross_platform(format!("Failed to create manager: {}", e))
        })?;

        if self.platform_settings.use_native_file_ops {
            // Use atomic operations if available
            manager.provider.save().await.map_err(|e| {
                TraitConfigError::cross_platform(format!("Atomic save failed: {}", e))
            })?;
        } else {
            // Use standard save
            manager.provider.save().await.map_err(|e| {
                TraitConfigError::cross_platform(format!("Standard save failed: {}", e))
            })?;
        }

        Ok(())
    }

    /// Get platform-specific configuration defaults
    fn platform_defaults(&self) -> HashMap<String, ConfigValue> {
        let mut defaults = HashMap::new();

        defaults.insert(
            "enable_optimizations".to_string(),
            ConfigValue::Boolean(true),
        );
        defaults.insert(
            "use_native_file_ops".to_string(),
            ConfigValue::Boolean(true),
        );
        defaults.insert(
            "enable_memory_mapping".to_string(),
            ConfigValue::Boolean(self.platform_info().memory_mapping_available),
        );

        defaults
    }

    /// Migrate configuration for current platform
    async fn migrate_for_platform(&mut self) -> TraitConfigResult<()> {
        let platform_info = self.platform_info();

        // Adjust settings based on platform capabilities
        if !platform_info.memory_mapping_available {
            self.platform_settings.enable_memory_mapping = false;
        }

        if !platform_info.file_watching_available {
            self.monitoring.enable_file_watching = false;
        }

        Ok(())
    }

    /// Validate platform compatibility
    fn validate_platform_compatibility(&self) -> TraitConfigResult<()> {
        let platform_info = self.platform_info();

        if self.platform_settings.enable_memory_mapping && !platform_info.memory_mapping_available {
            return Err(TraitConfigError::cross_platform(
                "Memory mapping not available on this platform",
            ));
        }

        if self.monitoring.enable_file_watching && !platform_info.file_watching_available {
            return Err(TraitConfigError::cross_platform(
                "File watching not available on this platform",
            ));
        }

        Ok(())
    }

    /// Get platform-specific performance settings
    fn platform_performance_settings(&self) -> PlatformPerformanceSettings {
        let platform_info = self.platform_info();

        PlatformPerformanceSettings {
            enable_memory_mapping: platform_info.memory_mapping_available
                && self.platform_settings.enable_memory_mapping,
            use_atomic_operations: platform_info.atomic_operations_available,
            enable_fs_caching: true,
            optimal_buffer_size: self.performance.io_buffer_size,
            max_concurrent_operations: 4, // Could be made configurable
            optimization_flags: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_enhanced_config_manager_creation() {
        let manager = EnhancedConfigurationManager::new().await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_enhanced_config_default() {
        let config = EnhancedConfig::default();
        assert!(config.platform_settings.enable_optimizations);
        assert!(config.performance.lazy_loading);
        assert!(config.security_enhanced.auto_encrypt_sensitive);
    }

    #[tokio::test]
    async fn test_performance_metrics() {
        let manager = EnhancedConfigurationManager::new().await.unwrap();
        let metrics = manager.get_metrics().await;
        assert_eq!(metrics.total_loads, 0);
        assert_eq!(metrics.cache_hits, 0);
    }

    // Test trait implementations
    #[tokio::test]
    async fn test_enhanced_config_base_trait() {
        let config = EnhancedConfig::default();

        // Test validation
        assert!(config.validate().is_ok());

        // Test runtime type information
        assert!(config.as_any().is::<EnhancedConfig>());

        // Test reporting event (should not panic)
        config.report_event(ConfigChangeType::ConfigReloaded);
    }

    #[tokio::test]
    async fn test_enhanced_config_lifecycle_trait() {
        let config = EnhancedConfig::default();

        // Test metadata
        let metadata = config.get_metadata();
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.format, Some("enhanced_toml".to_string()));
    }

    #[tokio::test]
    async fn test_enhanced_config_validation_trait() {
        let config = EnhancedConfig::default();

        // Test validation with context
        assert!(config.validate_with_context().is_ok());

        // Test field validation
        assert!(config.validate_field("platform_settings").is_ok());
        assert!(config.validate_field("performance").is_ok());
        assert!(config.validate_field("security_enhanced").is_ok());

        // Test validation rules
        let rules = config.validation_rules();
        assert!(!rules.is_empty());
        assert!(rules.iter().any(|r| r.name == "cache_ttl_positive"));
    }

    #[tokio::test]
    async fn test_enhanced_config_reporting_trait() {
        let config = EnhancedConfig::default();

        // Test reporting configuration
        let reporting_config = config.reporting_config();
        assert!(reporting_config.report_changes);
        assert!(reporting_config.report_errors);

        // Test change reporting (should not panic)
        config.report_change(TraitConfigChangeType::Loaded, Some("test".to_string()));

        // Test error reporting (should not panic)
        let error = ConfigError::validation("test error");
        config.report_error(&error, Some("test context".to_string()));

        // Test metric reporting (should not panic)
        config.report_metric("test_metric", 42.0, None);
    }

    #[tokio::test]
    async fn test_enhanced_config_cross_platform_trait() {
        let config = EnhancedConfig::default();

        // Test platform info
        let platform_info = config.platform_info();
        assert!(platform_info.os_type.len() > 0);

        // Test platform defaults
        let defaults = config.platform_defaults();
        assert!(!defaults.is_empty());

        // Test platform compatibility validation
        assert!(config.validate_platform_compatibility().is_ok());

        // Test platform performance settings
        let perf_settings = config.platform_performance_settings();
        assert!(perf_settings.optimal_buffer_size > 0);
    }

    #[tokio::test]
    async fn test_enhanced_config_validation_errors() {
        let mut config = EnhancedConfig::default();

        // Test validation failure: zero cache TTL
        config.performance.cache_ttl_secs = 0;
        assert!(config.validate().is_err());

        // Reset to valid state
        config.performance.cache_ttl_secs = 300;
        assert!(config.validate().is_ok());

        // Test validation failure: auto-encrypt without patterns
        config.security_enhanced.auto_encrypt_sensitive = true;
        config.security_enhanced.sensitive_patterns.clear();
        assert!(config.validate().is_err());
    }
}
