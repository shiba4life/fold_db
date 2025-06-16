use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::path::Path;

use crate::config::error::ConfigError;
use crate::config::platform::{EnhancedPlatformInfo, PlatformConfigPaths};
use crate::config::traits::base::{
    ConfigChangeType, ConfigMetadata, ReportingConfig, ValidationRule, ValidationRuleType,
    ValidationSeverity,
};
use crate::config::traits::integration::{
    AccessPattern, AccessType, ConfigComparison, ConfigMetrics, ConfigSnapshot, ConfigTelemetry,
    DebugLevel, HealthStatus, MonitoringSession, PlatformPerformanceSettings,
    ReportingCapabilities, ReportingRegistration, UnifiedReport,
};
use crate::config::traits::network::{
    ConnectivityStatus, ConnectivityTest, ConnectivityTestResult,
    NetworkConfig as NetworkConfigTrait, NetworkHealthMetrics, NetworkPlatformSettings,
    SocketBufferSizes, TcpSettings, UdpSettings,
};
use crate::config::traits::{
    BaseConfig, ConfigLifecycle, ConfigValidation, CrossPlatformConfig, ObservableConfig,
    ReportableConfig, TraitConfigError, TraitConfigResult, ValidationContext,
};
use crate::config::value::ConfigValue;

/// Configuration for the network layer with enhanced trait support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Local listening address
    pub listen_address: String,
    /// Enable mDNS discovery
    pub enable_mdns: bool,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Connection keep-alive interval in seconds
    pub keep_alive_interval: u64,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// UDP port for discovery
    pub discovery_port: u16,
    /// Connection timeout in seconds
    pub connection_timeout: std::time::Duration,
    /// Announcement interval in milliseconds
    pub announcement_interval: std::time::Duration,

    // Enhanced trait support fields
    /// Configuration metadata
    #[serde(default)]
    pub metadata: ConfigMetadata,
    /// Reporting configuration
    #[serde(default)]
    pub reporting_config: ReportingConfig,
    /// Validation rules
    #[serde(skip)]
    pub validation_rules: Vec<ValidationRule>,
    /// Platform-specific settings
    #[serde(default)]
    pub platform_settings: NetworkPlatformSettings,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_address: "/ip4/0.0.0.0/tcp/0".to_string(),
            enable_mdns: true,
            max_connections: 50,
            keep_alive_interval: 20,
            max_message_size: 1_000_000, // 1MB
            discovery_port: 5353,        // Standard mDNS port
            connection_timeout: std::time::Duration::from_secs(30),
            announcement_interval: std::time::Duration::from_secs(30), // More frequent announcements
            metadata: ConfigMetadata::default(),
            reporting_config: ReportingConfig::default(),
            validation_rules: Self::default_validation_rules(),
            platform_settings: NetworkPlatformSettings::default(),
        }
    }
}

#[async_trait]
impl BaseConfig for NetworkConfig {
    type Error = ConfigError;
    type Event = ConfigChangeType;
    type TransformTarget = NetworkConfig;

    async fn load(path: &Path) -> Result<Self, Self::Error> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| ConfigError::Io(e))?;

        let mut config: NetworkConfig = toml::from_str(&content).map_err(|e| {
            ConfigError::validation(format!("Failed to parse network config: {}", e))
        })?;

        // Set metadata
        config.metadata.source = Some(path.to_string_lossy().to_string());
        config.metadata.accessed_at = Utc::now();

        // Validate after loading
        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<(), Self::Error> {
        // Validate network parameters
        self.validate_network_parameters()
            .map_err(|e| ConfigError::validation(e.to_string()))?;

        // Validate port configuration
        self.validate_port_configuration()
            .map_err(|e| ConfigError::validation(e.to_string()))?;

        // Validate connection settings
        self.validate_connection_settings()
            .map_err(|e| ConfigError::validation(e.to_string()))?;

        Ok(())
    }

    fn report_event(&self, event: Self::Event) {
        if self.reporting_config.report_changes {
            // Integration with unified reporting system would go here
            log::info!("Network config event: {:?}", event);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl ConfigLifecycle for NetworkConfig {
    async fn save(&self, path: &Path) -> Result<(), Self::Error> {
        let content = toml::to_string_pretty(self).map_err(|e| ConfigError::TomlSer(e))?;

        tokio::fs::write(path, content)
            .await
            .map_err(|e| ConfigError::Io(e))?;

        Ok(())
    }

    async fn reload(&mut self, path: &Path) -> Result<(), Self::Error> {
        let new_config = Self::load(path).await?;
        *self = new_config;
        Ok(())
    }

    async fn has_changed(&self, path: &Path) -> Result<bool, Self::Error> {
        let metadata = tokio::fs::metadata(path)
            .await
            .map_err(|e| ConfigError::Io(e))?;

        let modified = metadata.modified().map_err(|e| {
            ConfigError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to get modification time: {}", e),
            ))
        })?;

        let modified_utc = DateTime::<Utc>::from(modified);
        Ok(modified_utc > self.metadata.updated_at)
    }

    fn get_metadata(&self) -> ConfigMetadata {
        self.metadata.clone()
    }

    fn set_metadata(&mut self, metadata: ConfigMetadata) {
        self.metadata = metadata;
    }
}

impl ConfigValidation for NetworkConfig {
    fn validate_with_context(&self) -> Result<(), ValidationContext> {
        let context = ValidationContext::new("NetworkConfig", "network_validation".to_string());

        self.validate().map_err(|_e| context)?;
        Ok(())
    }

    fn validate_field(&self, field_path: &str) -> Result<(), Self::Error> {
        match field_path {
            "listen_address" => {
                if self.listen_address.is_empty() {
                    return Err(ConfigError::validation("Listen address cannot be empty"));
                }
            }
            "max_connections" => {
                if self.max_connections == 0 {
                    return Err(ConfigError::validation(
                        "Max connections must be greater than 0",
                    ));
                }
            }
            "discovery_port" => {
                if self.discovery_port == 0 {
                    return Err(ConfigError::validation(
                        "Discovery port must be greater than 0",
                    ));
                }
            }
            _ => {
                return Err(ConfigError::validation(format!(
                    "Unknown field: {}",
                    field_path
                )));
            }
        }
        Ok(())
    }

    fn validation_rules(&self) -> Vec<ValidationRule> {
        self.validation_rules.clone()
    }

    fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }
}

impl NetworkConfig {
    /// Create a new network configuration with the specified listen address
    pub fn new(listen_address: &str) -> Self {
        Self {
            listen_address: listen_address.to_string(),
            ..Default::default()
        }
    }

    /// Enable or disable mDNS discovery
    pub fn with_mdns(mut self, enable: bool) -> Self {
        self.enable_mdns = enable;
        self
    }

    /// Set the maximum number of concurrent connections
    pub fn with_max_connections(mut self, max: usize) -> Self {
        self.max_connections = max;
        self
    }

    /// Set the connection keep-alive interval in seconds
    pub fn with_keep_alive_interval(mut self, interval: u64) -> Self {
        self.keep_alive_interval = interval;
        self
    }

    /// Set the maximum message size in bytes
    pub fn with_max_message_size(mut self, size: usize) -> Self {
        self.max_message_size = size;
        self
    }

    /// Set the discovery port
    pub fn with_discovery_port(mut self, port: u16) -> Self {
        self.discovery_port = port;
        self
    }

    /// Set the connection timeout
    pub fn with_connection_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// Set the announcement interval
    pub fn with_announcement_interval(mut self, interval: std::time::Duration) -> Self {
        self.announcement_interval = interval;
        self
    }
    /// Default validation rules for network configuration
    fn default_validation_rules() -> Vec<ValidationRule> {
        vec![
            ValidationRule {
                name: "listen_address_required".to_string(),
                description: "Listen address must not be empty".to_string(),
                field_path: "listen_address".to_string(),
                rule_type: ValidationRuleType::Required,
                severity: ValidationSeverity::Error,
            },
            ValidationRule {
                name: "max_connections_positive".to_string(),
                description: "Maximum connections must be greater than 0".to_string(),
                field_path: "max_connections".to_string(),
                rule_type: ValidationRuleType::NumericRange {
                    min: Some(1.0),
                    max: Some(100000.0),
                },
                severity: ValidationSeverity::Error,
            },
            ValidationRule {
                name: "discovery_port_range".to_string(),
                description: "Discovery port must be in valid range".to_string(),
                field_path: "discovery_port".to_string(),
                rule_type: ValidationRuleType::NumericRange {
                    min: Some(1.0),
                    max: Some(65535.0),
                },
                severity: ValidationSeverity::Error,
            },
            ValidationRule {
                name: "message_size_reasonable".to_string(),
                description: "Message size should be reasonable".to_string(),
                field_path: "max_message_size".to_string(),
                rule_type: ValidationRuleType::NumericRange {
                    min: Some(1.0),
                    max: Some(100000000.0),
                },
                severity: ValidationSeverity::Warning,
            },
        ]
    }
}

#[async_trait]
impl NetworkConfigTrait for NetworkConfig {
    fn validate_network_parameters(&self) -> TraitConfigResult<()> {
        // Validate listen address format
        if self.listen_address.is_empty() {
            return Err(TraitConfigError::trait_validation(
                "Listen address cannot be empty",
                Some(
                    ValidationContext::new("NetworkConfig", "listen_address_required".to_string())
                        .with_path("listen_address"),
                ),
            ));
        }

        // Validate message size limits
        if self.max_message_size == 0 {
            return Err(TraitConfigError::trait_validation(
                "Maximum message size must be greater than 0",
                Some(
                    ValidationContext::new(
                        "NetworkConfig",
                        "max_message_size_positive".to_string(),
                    )
                    .with_path("max_message_size"),
                ),
            ));
        }

        // Validate reasonable message size (not too large)
        if self.max_message_size > 100_000_000 {
            // 100MB
            return Err(TraitConfigError::trait_validation(
                "Maximum message size is too large (max 100MB)",
                Some(
                    ValidationContext::new("NetworkConfig", "max_message_size_limit".to_string())
                        .with_path("max_message_size"),
                ),
            ));
        }

        Ok(())
    }

    fn validate_port_configuration(&self) -> TraitConfigResult<()> {
        // Validate discovery port range
        if self.discovery_port < 1024 && self.discovery_port != 0 {
            return Err(TraitConfigError::trait_validation(
                "Discovery port should not use privileged ports (< 1024) unless necessary",
                Some(
                    ValidationContext::new("NetworkConfig", "port_privilege_warning".to_string())
                        .with_path("discovery_port"),
                ),
            ));
        }

        // Standard mDNS port is 5353
        if self.enable_mdns && self.discovery_port != 5353 {
            log::warn!(
                "Using non-standard mDNS port {}, standard is 5353",
                self.discovery_port
            );
        }

        Ok(())
    }

    fn validate_connection_settings(&self) -> TraitConfigResult<()> {
        // Validate connection limits
        if self.max_connections == 0 {
            return Err(TraitConfigError::trait_validation(
                "Maximum connections must be greater than 0",
                Some(
                    ValidationContext::new("NetworkConfig", "max_connections_positive".to_string())
                        .with_path("max_connections"),
                ),
            ));
        }

        // Warn about very high connection limits
        if self.max_connections > 10000 {
            log::warn!(
                "Very high connection limit: {}. Consider system resource limits.",
                self.max_connections
            );
        }

        // Validate timeout values
        if self.connection_timeout.as_secs() == 0 {
            return Err(TraitConfigError::trait_validation(
                "Connection timeout must be greater than 0",
                Some(
                    ValidationContext::new("NetworkConfig", "timeout_positive".to_string())
                        .with_path("connection_timeout"),
                ),
            ));
        }

        // Validate keep-alive interval
        if self.keep_alive_interval == 0 {
            return Err(TraitConfigError::trait_validation(
                "Keep-alive interval must be greater than 0",
                Some(
                    ValidationContext::new("NetworkConfig", "keepalive_positive".to_string())
                        .with_path("keep_alive_interval"),
                ),
            ));
        }

        Ok(())
    }

    async fn get_network_health(&self) -> TraitConfigResult<NetworkHealthMetrics> {
        // In a real implementation, this would collect actual network metrics
        let mut metrics = NetworkHealthMetrics::new();

        // Simulate some basic health metrics
        metrics.connection_success_rate = 0.98; // 98% success rate
        metrics.avg_latency_ms = 25.0;
        metrics.active_connections = 15;
        metrics.max_connections_reached = self.max_connections as u32;

        Ok(metrics)
    }

    async fn test_connectivity(&self) -> TraitConfigResult<ConnectivityTestResult> {
        let mut result = ConnectivityTestResult::new();

        // DNS resolution test
        let dns_test = ConnectivityTest {
            name: "DNS Resolution".to_string(),
            success: true, // Simplified for example
            duration_ms: 10.0,
            error: None,
            metadata: HashMap::new(),
        };
        result.add_test(dns_test);

        // Port binding test
        let port_test = ConnectivityTest {
            name: "Port Binding".to_string(),
            success: true, // Simplified for example
            duration_ms: 5.0,
            error: None,
            metadata: HashMap::new(),
        };
        result.add_test(port_test);

        // Update overall status based on tests
        if result.tests.iter().all(|t| t.success) {
            result.status = ConnectivityStatus::Healthy;
        }

        Ok(result)
    }

    fn get_platform_network_settings(&self) -> NetworkPlatformSettings {
        self.platform_settings.clone()
    }

    fn validate_network_security(&self) -> TraitConfigResult<()> {
        // Check if using secure defaults
        if self.listen_address.starts_with("/ip4/0.0.0.0") {
            log::warn!(
                "Listening on all interfaces (0.0.0.0) - consider restricting for production"
            );
        }

        // Validate secure protocols (this would be more comprehensive in real implementation)
        if self.enable_mdns {
            log::info!("mDNS enabled - ensure network is trusted");
        }

        Ok(())
    }
}

#[async_trait]
impl CrossPlatformConfig for NetworkConfig {
    fn platform_paths(&self) -> &dyn PlatformConfigPaths {
        // This would return platform-specific path resolver
        unimplemented!("Platform paths integration needed")
    }

    fn platform_info(&self) -> EnhancedPlatformInfo {
        // This would return platform information
        unimplemented!("Platform info integration needed")
    }

    async fn load_platform_optimized(&self, path: &Path) -> TraitConfigResult<Self> {
        // Use platform-optimized loading
        Self::load(path).await.map_err(TraitConfigError::from)
    }

    async fn save_platform_optimized(&self, path: &Path) -> TraitConfigResult<()> {
        // Use platform-optimized saving with atomic writes
        self.save(path).await.map_err(TraitConfigError::from)
    }

    fn platform_defaults(&self) -> HashMap<String, ConfigValue> {
        let mut defaults = HashMap::new();

        // Platform-specific network defaults
        #[cfg(target_os = "linux")]
        {
            defaults.insert("tcp_fastopen".to_string(), ConfigValue::boolean(true));
            defaults.insert("so_reuseport".to_string(), ConfigValue::boolean(true));
        }

        #[cfg(target_os = "windows")]
        {
            defaults.insert("tcp_fastopen".to_string(), ConfigValue::boolean(false));
            defaults.insert("so_reuseport".to_string(), ConfigValue::boolean(false));
        }

        #[cfg(target_os = "macos")]
        {
            defaults.insert("tcp_fastopen".to_string(), ConfigValue::boolean(true));
            defaults.insert("so_reuseport".to_string(), ConfigValue::boolean(true));
        }

        defaults
    }

    async fn migrate_for_platform(&mut self) -> TraitConfigResult<()> {
        // Adjust settings for current platform
        let platform_defaults = self.platform_defaults();

        // Apply platform-specific optimizations
        #[cfg(target_os = "linux")]
        {
            self.platform_settings.tcp_settings.keepalive = true;
        }

        Ok(())
    }

    fn validate_platform_compatibility(&self) -> TraitConfigResult<()> {
        // Check platform-specific limitations
        #[cfg(target_os = "windows")]
        {
            if self.max_connections > 2048 {
                log::warn!("High connection count on Windows may require registry adjustments");
            }
        }

        Ok(())
    }

    fn platform_performance_settings(&self) -> PlatformPerformanceSettings {
        let mut settings = PlatformPerformanceSettings::default();

        // Platform-specific optimizations
        #[cfg(target_os = "linux")]
        {
            settings.enable_memory_mapping = true;
            settings.use_atomic_operations = true;
            settings.max_concurrent_operations = 1000;
        }

        #[cfg(target_os = "windows")]
        {
            settings.enable_memory_mapping = false;
            settings.use_atomic_operations = true;
            settings.max_concurrent_operations = 500;
        }

        settings.optimal_buffer_size = 65536;
        settings
    }
}
