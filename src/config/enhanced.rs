//! Enhanced cross-platform configuration management with platform-specific optimizations
//!
//! This module provides an enhanced configuration system that builds on the core
//! cross-platform functionality while adding platform-specific optimizations:
//! - Native keystore integration for secure storage
//! - Platform-optimized file operations and caching
//! - Real-time configuration monitoring
//! - Encrypted configuration sections
//! - Performance optimizations and memory management

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
use tokio::sync::{RwLock, Mutex};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::config::error::{ConfigError, ConfigResult};
use crate::config::value::{ConfigValue, ConfigValueSchema};
use crate::config::cross_platform::{Config, ConfigurationProvider, TomlConfigProvider};
use crate::config::platform::{
    keystore::{PlatformKeystore, create_platform_keystore, KeystoreConfig},
    create_platform_resolver, create_platform_file_watcher, create_platform_atomic_ops,
    PlatformConfigPaths, EnhancedPlatformInfo, PlatformFileWatcher, PlatformAtomicOps,
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
        let keystore = keystore.unwrap_or_else(|| Arc::new(create_platform_keystore()));
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
        metrics.avg_load_time_ms = 
            (metrics.avg_load_time_ms * (metrics.total_loads - 1) as f64 + 
             start_time.elapsed().as_millis() as f64) / metrics.total_loads as f64;

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
        }).await;

        Ok(())
    }

    /// Store encrypted sections in keystore
    async fn store_encrypted_sections(&self, config: &EnhancedConfig) -> ConfigResult<()> {
        for (key, section) in &config.encrypted_sections {
            let serialized = serde_json::to_vec(section)
                .map_err(|e| ConfigError::encryption(format!("Failed to serialize encrypted section: {}", e)))?;
            
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

#[cfg(test)]
mod tests {
    use super::*;

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
}