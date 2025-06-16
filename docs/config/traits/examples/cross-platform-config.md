# Cross-Platform Configuration Example

This example demonstrates implementing a cross-platform configuration using the DataFold configuration traits system with [`CrossPlatformConfig`](../../../../src/config/traits/integration.rs:15) trait.

## Overview

A comprehensive configuration that adapts to different operating systems (Windows, macOS, Linux) while maintaining a unified interface. This example shows platform-specific optimizations, path handling, and performance tuning.

## Implementation

### Platform-Specific Types

```rust
use datafold::config::traits::{
    BaseConfig, ConfigLifecycle, ConfigValidation, CrossPlatformConfig,
    TraitConfigError, TraitConfigResult, PlatformPerformanceSettings
};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPlatformAppConfig {
    /// Application name
    pub app_name: String,
    
    /// Application version
    pub version: String,
    
    /// Data directory (platform-specific)
    pub data_dir: PathBuf,
    
    /// Log directory (platform-specific)
    pub log_dir: PathBuf,
    
    /// Cache directory (platform-specific)
    pub cache_dir: PathBuf,
    
    /// Platform-specific settings
    pub platform_settings: PlatformSettings,
    
    /// Performance configuration
    pub performance: PerformanceConfig,
    
    /// Feature toggles
    pub features: FeatureConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformSettings {
    /// Use platform-specific directories
    pub use_platform_dirs: bool,
    
    /// Platform-specific optimizations enabled
    pub enable_optimizations: bool,
    
    /// Windows-specific settings
    #[cfg(windows)]
    pub windows: WindowsSettings,
    
    /// macOS-specific settings
    #[cfg(target_os = "macos")]
    pub macos: MacOSSettings,
    
    /// Linux-specific settings
    #[cfg(target_os = "linux")]
    pub linux: LinuxSettings,
    
    /// Unix-like systems settings
    #[cfg(unix)]
    pub unix: UnixSettings,
}

#[cfg(windows)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsSettings {
    /// Use Windows Event Log
    pub use_event_log: bool,
    
    /// Use Windows Service mode
    pub service_mode: bool,
    
    /// Registry configuration path
    pub registry_path: Option<String>,
    
    /// Use IO completion ports
    pub use_iocp: bool,
    
    /// Windows-specific file handling
    pub handle_long_paths: bool,
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacOSSettings {
    /// Use macOS unified logging
    pub use_unified_logging: bool,
    
    /// Bundle identifier
    pub bundle_id: Option<String>,
    
    /// Use kqueue for file watching
    pub use_kqueue: bool,
    
    /// Use GCD for concurrency
    pub use_gcd: bool,
    
    /// Sandbox mode
    pub sandbox_mode: bool,
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinuxSettings {
    /// Use systemd for logging
    pub use_systemd: bool,
    
    /// Use epoll for event handling
    pub use_epoll: bool,
    
    /// Use inotify for file watching
    pub use_inotify: bool,
    
    /// Container mode
    pub container_mode: bool,
    
    /// Cgroup awareness
    pub cgroup_aware: bool,
}

#[cfg(unix)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnixSettings {
    /// Use syslog for logging
    pub use_syslog: bool,
    
    /// Signal handling mode
    pub signal_handling: SignalHandlingMode,
    
    /// File permission mode
    pub file_permissions: u32,
    
    /// Use memory mapping
    pub use_mmap: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalHandlingMode {
    Default,
    Custom,
    Ignore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Number of worker threads
    pub worker_threads: Option<usize>,
    
    /// Memory allocation strategy
    pub memory_strategy: MemoryStrategy,
    
    /// I/O strategy
    pub io_strategy: IoStrategy,
    
    /// CPU affinity settings
    pub cpu_affinity: CpuAffinityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryStrategy {
    Default,
    LowMemory,
    HighPerformance,
    Custom { page_size: usize, pool_size: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IoStrategy {
    Default,
    Async,
    Blocking,
    Platform,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuAffinityConfig {
    /// Enable CPU affinity
    pub enabled: bool,
    
    /// Specific CPU cores (None = automatic)
    pub cores: Option<Vec<usize>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    /// Enable telemetry
    pub telemetry: bool,
    
    /// Enable crash reporting
    pub crash_reporting: bool,
    
    /// Enable auto-updates
    pub auto_updates: bool,
    
    /// Platform-specific features
    pub platform_features: PlatformFeatureConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformFeatureConfig {
    #[cfg(windows)]
    pub windows_integration: bool,
    
    #[cfg(target_os = "macos")]
    pub macos_integration: bool,
    
    #[cfg(target_os = "linux")]
    pub linux_integration: bool,
}
```

### Default Implementations

```rust
impl Default for CrossPlatformAppConfig {
    fn default() -> Self {
        Self {
            app_name: "CrossPlatformApp".to_string(),
            version: "1.0.0".to_string(),
            data_dir: Self::default_data_dir(),
            log_dir: Self::default_log_dir(),
            cache_dir: Self::default_cache_dir(),
            platform_settings: PlatformSettings::default(),
            performance: PerformanceConfig::default(),
            features: FeatureConfig::default(),
        }
    }
}

impl CrossPlatformAppConfig {
    /// Get platform-appropriate default data directory
    fn default_data_dir() -> PathBuf {
        #[cfg(windows)]
        {
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("C:\\ProgramData"))
                .join("CrossPlatformApp")
        }
        
        #[cfg(target_os = "macos")]
        {
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("/Library/Application Support"))
                .join("CrossPlatformApp")
        }
        
        #[cfg(target_os = "linux")]
        {
            dirs::data_dir()
                .unwrap_or_else(|| {
                    std::env::var("XDG_DATA_HOME")
                        .map(PathBuf::from)
                        .unwrap_or_else(|_| {
                            dirs::home_dir()
                                .unwrap_or_else(|| PathBuf::from("/tmp"))
                                .join(".local/share")
                        })
                })
                .join("crossplatformapp")
        }
        
        #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
        {
            PathBuf::from("./data")
        }
    }
    
    fn default_log_dir() -> PathBuf {
        #[cfg(windows)]
        {
            Self::default_data_dir().join("logs")
        }
        
        #[cfg(target_os = "macos")]
        {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("Library/Logs/CrossPlatformApp")
        }
        
        #[cfg(target_os = "linux")]
        {
            if std::env::var("USER").unwrap_or_default() == "root" {
                PathBuf::from("/var/log/crossplatformapp")
            } else {
                Self::default_data_dir().join("logs")
            }
        }
        
        #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
        {
            PathBuf::from("./logs")
        }
    }
    
    fn default_cache_dir() -> PathBuf {
        #[cfg(windows)]
        {
            dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("C:\\Temp"))
                .join("CrossPlatformApp")
        }
        
        #[cfg(target_os = "macos")]
        {
            dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("CrossPlatformApp")
        }
        
        #[cfg(target_os = "linux")]
        {
            dirs::cache_dir()
                .unwrap_or_else(|| {
                    std::env::var("XDG_CACHE_HOME")
                        .map(PathBuf::from)
                        .unwrap_or_else(|_| {
                            dirs::home_dir()
                                .unwrap_or_else(|| PathBuf::from("/tmp"))
                                .join(".cache")
                        })
                })
                .join("crossplatformapp")
        }
        
        #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
        {
            PathBuf::from("./cache")
        }
    }
}

impl Default for PlatformSettings {
    fn default() -> Self {
        Self {
            use_platform_dirs: true,
            enable_optimizations: true,
            
            #[cfg(windows)]
            windows: WindowsSettings::default(),
            
            #[cfg(target_os = "macos")]
            macos: MacOSSettings::default(),
            
            #[cfg(target_os = "linux")]
            linux: LinuxSettings::default(),
            
            #[cfg(unix)]
            unix: UnixSettings::default(),
        }
    }
}

#[cfg(windows)]
impl Default for WindowsSettings {
    fn default() -> Self {
        Self {
            use_event_log: false,
            service_mode: false,
            registry_path: None,
            use_iocp: true,
            handle_long_paths: true,
        }
    }
}

#[cfg(target_os = "macos")]
impl Default for MacOSSettings {
    fn default() -> Self {
        Self {
            use_unified_logging: true,
            bundle_id: None,
            use_kqueue: true,
            use_gcd: true,
            sandbox_mode: false,
        }
    }
}

#[cfg(target_os = "linux")]
impl Default for LinuxSettings {
    fn default() -> Self {
        Self {
            use_systemd: true,
            use_epoll: true,
            use_inotify: true,
            container_mode: false,
            cgroup_aware: true,
        }
    }
}

#[cfg(unix)]
impl Default for UnixSettings {
    fn default() -> Self {
        Self {
            use_syslog: true,
            signal_handling: SignalHandlingMode::Default,
            file_permissions: 0o644,
            use_mmap: true,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            worker_threads: None, // Auto-detect
            memory_strategy: MemoryStrategy::Default,
            io_strategy: IoStrategy::Platform,
            cpu_affinity: CpuAffinityConfig::default(),
        }
    }
}

impl Default for CpuAffinityConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cores: None,
        }
    }
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            telemetry: true,
            crash_reporting: true,
            auto_updates: false,
            platform_features: PlatformFeatureConfig::default(),
        }
    }
}

impl Default for PlatformFeatureConfig {
    fn default() -> Self {
        Self {
            #[cfg(windows)]
            windows_integration: true,
            
            #[cfg(target_os = "macos")]
            macos_integration: true,
            
            #[cfg(target_os = "linux")]
            linux_integration: true,
        }
    }
}
```

### Error and Event Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CrossPlatformConfigError {
    #[error("Validation error in field '{field}': {message}")]
    ValidationError {
        field: String,
        message: String,
    },

    #[error("Platform-specific error on {platform}: {message}")]
    PlatformError {
        platform: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Directory creation error: {path}")]
    DirectoryError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Permission error: {message}")]
    PermissionError {
        message: String,
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
pub struct CrossPlatformConfigEvent {
    pub event_type: CrossPlatformEventType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub platform: String,
    pub details: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossPlatformEventType {
    ConfigLoaded,
    ConfigSaved,
    PlatformDetected,
    OptimizationApplied,
    DirectoryCreated,
    PermissionChanged,
    FeatureToggled,
}

impl CrossPlatformConfigEvent {
    pub fn new(event_type: CrossPlatformEventType) -> Self {
        Self {
            event_type,
            timestamp: chrono::Utc::now(),
            platform: std::env::consts::OS.to_string(),
            details: std::collections::HashMap::new(),
        }
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
impl BaseConfig for CrossPlatformAppConfig {
    type Error = CrossPlatformConfigError;
    type Event = CrossPlatformConfigEvent;
    type TransformTarget = ();

    async fn load(path: &Path) -> Result<Self, Self::Error> {
        let content = tokio::fs::read_to_string(path).await?;
        
        let mut config: Self = match path.extension().and_then(|ext| ext.to_str()) {
            Some("toml") => toml::from_str(&content)
                .map_err(|e| CrossPlatformConfigError::SerializationError {
                    source: Box::new(e),
                })?,
            Some("json") => serde_json::from_str(&content)
                .map_err(|e| CrossPlatformConfigError::SerializationError {
                    source: Box::new(e),
                })?,
            Some("yaml") | Some("yml") => serde_yaml::from_str(&content)
                .map_err(|e| CrossPlatformConfigError::SerializationError {
                    source: Box::new(e),
                })?,
            _ => return Err(CrossPlatformConfigError::ValidationError {
                field: "file_format".to_string(),
                message: "Unsupported configuration file format".to_string(),
            }),
        };

        // Apply platform-specific adjustments
        config.apply_platform_defaults().await?;
        
        // Validate the configuration
        config.validate()?;
        
        // Report load event
        config.report_event(CrossPlatformConfigEvent::new(CrossPlatformEventType::ConfigLoaded)
            .with_detail("platform".to_string(), std::env::consts::OS.to_string())
            .with_detail("arch".to_string(), std::env::consts::ARCH.to_string()));
        
        Ok(config)
    }

    fn validate(&self) -> Result<(), Self::Error> {
        // Validate application name
        if self.app_name.is_empty() {
            return Err(CrossPlatformConfigError::ValidationError {
                field: "app_name".to_string(),
                message: "Application name cannot be empty".to_string(),
            });
        }

        // Validate version format
        if self.version.is_empty() {
            return Err(CrossPlatformConfigError::ValidationError {
                field: "version".to_string(),
                message: "Version cannot be empty".to_string(),
            });
        }

        // Validate directories exist or can be created
        self.validate_directories()?;
        
        // Validate performance settings
        self.validate_performance_settings()?;
        
        // Platform-specific validation
        self.validate_platform_specific()?;

        Ok(())
    }

    fn report_event(&self, event: Self::Event) {
        // Log the event
        log::info!("Cross-platform config event: {:?}", event);
        
        // Send to metrics system
        metrics::counter!(
            "crossplatform.config.events",
            1,
            "type" => format!("{:?}", event.event_type),
            "platform" => &event.platform,
            "app" => &self.app_name
        );
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl CrossPlatformAppConfig {
    fn validate_directories(&self) -> Result<(), CrossPlatformConfigError> {
        let dirs = [&self.data_dir, &self.log_dir, &self.cache_dir];
        
        for dir in &dirs {
            // Check if directory exists or can be created
            if !dir.exists() {
                // Check if parent directory exists and is writable
                if let Some(parent) = dir.parent() {
                    if !parent.exists() {
                        return Err(CrossPlatformConfigError::DirectoryError {
                            path: parent.display().to_string(),
                            source: std::io::Error::new(
                                std::io::ErrorKind::NotFound,
                                "Parent directory does not exist"
                            ),
                        });
                    }
                }
            }
            
            // Platform-specific permission checks
            #[cfg(unix)]
            {
                if dir.exists() {
                    use std::os::unix::fs::PermissionsExt;
                    let metadata = std::fs::metadata(dir)
                        .map_err(|e| CrossPlatformConfigError::DirectoryError {
                            path: dir.display().to_string(),
                            source: e,
                        })?;
                    
                    let permissions = metadata.permissions();
                    if permissions.mode() & 0o200 == 0 {
                        return Err(CrossPlatformConfigError::PermissionError {
                            message: format!("Directory {} is not writable", dir.display()),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_performance_settings(&self) -> Result<(), CrossPlatformConfigError> {
        // Validate worker thread count
        if let Some(threads) = self.performance.worker_threads {
            if threads == 0 {
                return Err(CrossPlatformConfigError::ValidationError {
                    field: "worker_threads".to_string(),
                    message: "Worker thread count must be greater than 0".to_string(),
                });
            }
            
            let max_threads = num_cpus::get() * 2;
            if threads > max_threads {
                return Err(CrossPlatformConfigError::ValidationError {
                    field: "worker_threads".to_string(),
                    message: format!("Worker thread count ({}) exceeds recommended maximum ({})", 
                                   threads, max_threads),
                });
            }
        }

        // Validate CPU affinity
        if self.performance.cpu_affinity.enabled {
            if let Some(ref cores) = self.performance.cpu_affinity.cores {
                let max_cores = num_cpus::get();
                for &core in cores {
                    if core >= max_cores {
                        return Err(CrossPlatformConfigError::ValidationError {
                            field: "cpu_affinity.cores".to_string(),
                            message: format!("CPU core {} does not exist (max: {})", 
                                           core, max_cores - 1),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_platform_specific(&self) -> Result<(), CrossPlatformConfigError> {
        #[cfg(windows)]
        {
            // Windows-specific validation
            if let Some(ref registry_path) = self.platform_settings.windows.registry_path {
                if !registry_path.starts_with("HKEY_") {
                    return Err(CrossPlatformConfigError::PlatformError {
                        platform: "Windows".to_string(),
                        message: "Registry path must start with HKEY_".to_string(),
                        source: None,
                    });
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS-specific validation
            if let Some(ref bundle_id) = self.platform_settings.macos.bundle_id {
                if !bundle_id.contains('.') {
                    return Err(CrossPlatformConfigError::PlatformError {
                        platform: "macOS".to_string(),
                        message: "Bundle ID must contain at least one dot".to_string(),
                        source: None,
                    });
                }
            }
        }

        #[cfg(unix)]
        {
            // Unix-specific validation
            let perms = self.platform_settings.unix.file_permissions;
            if perms & 0o777 != perms {
                return Err(CrossPlatformConfigError::PlatformError {
                    platform: "Unix".to_string(),
                    message: "Invalid file permissions".to_string(),
                    source: None,
                });
            }
        }

        Ok(())
    }
}
```

### CrossPlatformConfig Implementation

```rust
impl CrossPlatformConfig for CrossPlatformAppConfig {
    fn platform_performance_settings(&self) -> PlatformPerformanceSettings {
        PlatformPerformanceSettings {
            #[cfg(windows)]
            use_iocp: self.platform_settings.windows.use_iocp,
            
            #[cfg(target_os = "macos")]
            use_kqueue: self.platform_settings.macos.use_kqueue,
            
            #[cfg(target_os = "linux")]
            use_epoll: self.platform_settings.linux.use_epoll,
            
            worker_threads: self.performance.worker_threads.unwrap_or_else(|| {
                match self.performance.io_strategy {
                    IoStrategy::Async => num_cpus::get(),
                    IoStrategy::Blocking => num_cpus::get() * 2,
                    IoStrategy::Platform => {
                        #[cfg(windows)]
                        return num_cpus::get();
                        
                        #[cfg(target_os = "macos")]
                        return num_cpus::get();
                        
                        #[cfg(target_os = "linux")]
                        return num_cpus::get() * 2;
                        
                        #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
                        return num_cpus::get();
                    }
                    IoStrategy::Default => num_cpus::get(),
                }
            }),
            
            memory_optimization: match self.performance.memory_strategy {
                MemoryStrategy::LowMemory => MemoryOptimizationLevel::Low,
                MemoryStrategy::HighPerformance => MemoryOptimizationLevel::High,
                MemoryStrategy::Default => MemoryOptimizationLevel::Medium,
                MemoryStrategy::Custom { .. } => MemoryOptimizationLevel::Custom,
            },
            
            cpu_affinity: if self.performance.cpu_affinity.enabled {
                CpuAffinitySettings::Enabled {
                    cores: self.performance.cpu_affinity.cores.clone(),
                }
            } else {
                CpuAffinitySettings::Disabled
            },
        }
    }

    fn apply_platform_optimizations(&mut self) -> TraitConfigResult<()> {
        if !self.platform_settings.enable_optimizations {
            return Ok(());
        }

        #[cfg(windows)]
        {
            // Windows optimizations
            if self.platform_settings.windows.use_iocp {
                self.performance.io_strategy = IoStrategy::Platform;
            }
            
            if self.platform_settings.windows.handle_long_paths {
                // Enable long path support if needed
                self.report_event(CrossPlatformConfigEvent::new(CrossPlatformEventType::OptimizationApplied)
                    .with_detail("optimization".to_string(), "long_paths".to_string()));
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS optimizations
            if self.platform_settings.macos.use_gcd {
                // Optimize for Grand Central Dispatch
                self.performance.worker_threads = Some(num_cpus::get());
            }
            
            if self.platform_settings.macos.use_kqueue {
                self.performance.io_strategy = IoStrategy::Platform;
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Linux optimizations
            if self.platform_settings.linux.use_epoll {
                self.performance.io_strategy = IoStrategy::Platform;
            }
            
            if self.platform_settings.linux.cgroup_aware {
                // Adjust based on cgroup limits
                if let Ok(limit) = self.get_cgroup_memory_limit() {
                    if limit < 1024 * 1024 * 1024 { // Less than 1GB
                        self.performance.memory_strategy = MemoryStrategy::LowMemory;
                    }
                }
            }
        }

        self.report_event(CrossPlatformConfigEvent::new(CrossPlatformEventType::OptimizationApplied)
            .with_detail("platform".to_string(), std::env::consts::OS.to_string()));

        Ok(())
    }
}

impl CrossPlatformAppConfig {
    pub async fn apply_platform_defaults(&mut self) -> Result<(), CrossPlatformConfigError> {
        if self.platform_settings.use_platform_dirs {
            // Update directories to platform-appropriate locations
            self.data_dir = Self::default_data_dir();
            self.log_dir = Self::default_log_dir();
            self.cache_dir = Self::default_cache_dir();
        }

        // Create directories if they don't exist
        self.ensure_directories_exist().await?;

        // Apply platform-specific defaults
        self.apply_platform_specific_defaults().await?;

        Ok(())
    }

    async fn ensure_directories_exist(&self) -> Result<(), CrossPlatformConfigError> {
        let dirs = [&self.data_dir, &self.log_dir, &self.cache_dir];
        
        for dir in &dirs {
            if !dir.exists() {
                tokio::fs::create_dir_all(dir).await
                    .map_err(|e| CrossPlatformConfigError::DirectoryError {
                        path: dir.display().to_string(),
                        source: e,
                    })?;
                
                self.report_event(CrossPlatformConfigEvent::new(CrossPlatformEventType::DirectoryCreated)
                    .with_detail("path".to_string(), dir.display().to_string()));
                
                // Set appropriate permissions on Unix systems
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let permissions = std::fs::Permissions::from_mode(self.platform_settings.unix.file_permissions);
                    tokio::fs::set_permissions(dir, permissions).await
                        .map_err(|e| CrossPlatformConfigError::DirectoryError {
                            path: dir.display().to_string(),
                            source: e,
                        })?;
                }
            }
        }

        Ok(())
    }

    async fn apply_platform_specific_defaults(&mut self) -> Result<(), CrossPlatformConfigError> {
        #[cfg(windows)]
        {
            // Windows-specific defaults
            if self.platform_settings.windows.service_mode {
                // Adjust settings for Windows service mode
                self.platform_settings.windows.use_event_log = true;
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS-specific defaults
            if self.platform_settings.macos.sandbox_mode {
                // Adjust paths for sandbox mode
                self.data_dir = dirs::data_dir()
                    .unwrap_or_else(|| PathBuf::from("/tmp"))
                    .join("Containers")
                    .join(&self.app_name);
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Linux-specific defaults
            if self.platform_settings.linux.container_mode {
                // Detect if running in container
                if self.is_running_in_container().await {
                    self.platform_settings.linux.use_systemd = false;
                    self.data_dir = PathBuf::from("/app/data");
                    self.log_dir = PathBuf::from("/app/logs");
                    self.cache_dir = PathBuf::from("/tmp/cache");
                }
            }
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn is_running_in_container(&self) -> bool {
        // Check for container indicators
        tokio::fs::metadata("/.dockerenv").await.is_ok() ||
        tokio::fs::read_to_string("/proc/1/cgroup").await
            .map(|content| content.contains("docker") || content.contains("kubepods"))
            .unwrap_or(false)
    }

    #[cfg(target_os = "linux")]
    fn get_cgroup_memory_limit(&self) -> Result<u64, Box<dyn std::error::Error>> {
        // Try to read cgroup memory limit
        let content = std::fs::read_to_string("/sys/fs/cgroup/memory/memory.limit_in_bytes")?;
        let limit: u64 = content.trim().parse()?;
        Ok(limit)
    }

    /// Get platform-specific log path
    pub fn get_platform_log_path(&self, log_name: &str) -> PathBuf {
        #[cfg(windows)]
        {
            if self.platform_settings.windows.use_event_log {
                // Return a special marker for Windows Event Log
                PathBuf::from("WINDOWS_EVENT_LOG")
            } else {
                self.log_dir.join(format!("{}.log", log_name))
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            if self.platform_settings.macos.use_unified_logging {
                // Return a special marker for macOS Unified Logging
                PathBuf::from("MACOS_UNIFIED_LOG")
            } else {
                self.log_dir.join(format!("{}.log", log_name))
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            if self.platform_settings.linux.use_systemd {
                // systemd journal handles logging
                PathBuf::from("SYSTEMD_JOURNAL")
            } else {
                self.log_dir.join(format!("{}.log", log_name))
            }
        }
        
        #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
        {
            self.log_dir.join(format!("{}.log", log_name))
        }
    }

    /// Check if a platform feature is available
    pub fn is_platform_feature_available(&self, feature: &str) -> bool {
        match feature {
            #[cfg(windows)]
            "event_log" => self.platform_settings.windows.use_event_log,
            #[cfg(windows)]
            "iocp" => self.platform_settings.windows.use_iocp,
            
            #[cfg(target_os = "macos")]
            "unified_logging" => self.platform_settings.macos.use_unified_logging,
            #[cfg(target_os = "macos")]
            "kqueue" => self.platform_settings.macos.use_kqueue,
            
            #[cfg(target_os = "linux")]
            "systemd" => self.platform_settings.linux.use_systemd,
            #[cfg(target_os = "linux")]
            "epoll" => self.platform_settings.linux.use_epoll,
            
            #[cfg(unix)]
            "syslog" => self.platform_settings.unix.use_syslog,
            
            _ => false,
        }
    }
}
```

## Usage Examples

### Basic Cross-Platform Usage

```rust
use tokio;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load cross-platform configuration
    let mut config = CrossPlatformAppConfig::load(Path::new("app.toml")).await?;
    
    // Apply platform-specific optimizations
    config.apply_platform_optimizations()?;
    
    // Print platform-specific information
    println!("Running on: {}", std::env::consts::OS);
    println!("Architecture: {}", std::env::consts::ARCH);
    println!("Data directory: {}", config.data_dir.display());
    println!("Log directory: {}", config.log_dir.display());
    
    // Check platform features
    if config.is_platform_feature_available("systemd") {
        println!("systemd logging available");
    }
    
    if config.is_platform_feature_available("event_log") {
        println!("Windows Event Log available");
    }
    
    // Set up platform-appropriate logging
    setup_logging(&config).await?;
    
    Ok(())
}

async fn setup_logging(config: &CrossPlatformAppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let log_path = config.get_platform_log_path("application");
    
    match log_path.to_str() {
        Some("WINDOWS_EVENT_LOG") => {
            println!("Setting up Windows Event Log");
            // Initialize Windows Event Log
        }
        Some("MACOS_UNIFIED_LOG") => {
            println!("Setting up macOS Unified Logging");
            // Initialize macOS Unified Logging
        }
        Some("SYSTEMD_JOURNAL") => {
            println!("Setting up systemd journal logging");
            // Initialize systemd journal
        }
        _ => {
            println!("Setting up file logging: {}", log_path.display());
            // Initialize file-based logging
        }
    }
    
    Ok(())
}
```

### Configuration File Example

```toml
# app.toml - Cross-platform configuration

app_name = "CrossPlatformApp"
version = "1.2.0"

# Platform directories (will be adjusted automatically)
data_dir = "./data"
log_dir = "./logs"
cache_dir = "./cache"

[platform_settings]
use_platform_dirs = true
enable_optimizations = true

# Windows-specific settings
[platform_settings.windows]
use_event_log = false
service_mode = false
use_iocp = true
handle_long_paths = true

# macOS-specific settings
[platform_settings.macos]
use_unified_logging = true
use_kqueue = true
use_gcd = true
sandbox_mode = false

# Linux-specific settings
[platform_settings.linux]
use_systemd = true
use_epoll = true
use_inotify = true
container_mode = false
cgroup_aware = true

# Unix-specific settings
[platform_settings.unix]
use_syslog = true
signal_handling = "Default"
file_permissions = 0o644
use_mmap = true

[performance]
memory_strategy = "Default"
io_strategy = "Platform"

[performance.cpu_affinity]
enabled = false

[features]
telemetry = true
crash_reporting = true
auto_updates = false

[features.platform_features]
windows_integration = true
macos_integration = true
linux_integration = true
```

## Key Features Demonstrated

1. **Platform Detection**: Automatic platform detection and adaptation
2. **Directory Management**: Platform-appropriate default directories
3. **Performance Optimization**: Platform-specific performance tuning
4. **Feature Availability**: Runtime feature detection and validation
5. **Configuration Adaptation**: Automatic adjustment for platform capabilities
6. **Error Handling**: Platform-specific error handling and reporting
7. **Resource Management**: Platform-appropriate resource allocation
8. **Integration Support**: Integration with platform-specific systems
9. **Container Awareness**: Detection and adaptation for containerized environments
10. **Security Considerations**: Platform-appropriate permission handling

This example demonstrates how the DataFold traits system enables writing truly cross-platform applications while maintaining platform-specific optimizations and capabilities.