# Basic Configuration Example

This example demonstrates implementing a simple application configuration using the DataFold configuration traits system.

## Overview

A basic configuration for a web application that includes common settings like server address, port, timeouts, and feature flags.

## Implementation

### Configuration Struct

```rust
use datafold::config::traits::{BaseConfig, ConfigLifecycle, ConfigValidation};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAppConfig {
    /// Application name
    pub app_name: String,
    
    /// Server bind address
    pub bind_address: String,
    
    /// Server port
    pub port: u16,
    
    /// Request timeout in seconds
    pub request_timeout_seconds: u64,
    
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    
    /// Enable debug logging
    pub debug_mode: bool,
    
    /// Feature flags
    pub features: FeatureFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub enable_metrics: bool,
    pub enable_tracing: bool,
    pub enable_health_check: bool,
    pub enable_admin_api: bool,
}

impl Default for WebAppConfig {
    fn default() -> Self {
        Self {
            app_name: "WebApp".to_string(),
            bind_address: "127.0.0.1".to_string(),
            port: 8080,
            request_timeout_seconds: 30,
            max_connections: 1000,
            debug_mode: false,
            features: FeatureFlags::default(),
        }
    }
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            enable_tracing: false,
            enable_health_check: true,
            enable_admin_api: false,
        }
    }
}
```

### Error Type

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WebAppConfigError {
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

    #[error("Serialization error: {source}")]
    SerializationError {
        #[from]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl From<toml::de::Error> for WebAppConfigError {
    fn from(err: toml::de::Error) -> Self {
        WebAppConfigError::SerializationError {
            source: Box::new(err),
        }
    }
}

impl From<toml::ser::Error> for WebAppConfigError {
    fn from(err: toml::ser::Error) -> Self {
        WebAppConfigError::SerializationError {
            source: Box::new(err),
        }
    }
}
```

### Event Type

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAppConfigEvent {
    pub event_type: WebAppEventType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub field: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebAppEventType {
    ConfigLoaded,
    ConfigSaved,
    ConfigReloaded,
    ConfigValidated,
    FieldChanged,
    FeatureToggled,
}

impl WebAppConfigEvent {
    pub fn new(event_type: WebAppEventType) -> Self {
        Self {
            event_type,
            timestamp: chrono::Utc::now(),
            field: None,
            old_value: None,
            new_value: None,
        }
    }
    
    pub fn field_changed(field: String, old_value: String, new_value: String) -> Self {
        Self {
            event_type: WebAppEventType::FieldChanged,
            timestamp: chrono::Utc::now(),
            field: Some(field),
            old_value: Some(old_value),
            new_value: Some(new_value),
        }
    }
}
```

### BaseConfig Implementation

```rust
#[async_trait]
impl BaseConfig for WebAppConfig {
    type Error = WebAppConfigError;
    type Event = WebAppConfigEvent;
    type TransformTarget = ();

    async fn load(path: &Path) -> Result<Self, Self::Error> {
        // Read configuration file
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| WebAppConfigError::LoadError {
                path: path.display().to_string(),
                source: Box::new(e),
            })?;

        // Parse based on file extension
        let config: Self = match path.extension().and_then(|ext| ext.to_str()) {
            Some("toml") => toml::from_str(&content)?,
            Some("json") => serde_json::from_str(&content)
                .map_err(|e| WebAppConfigError::SerializationError {
                    source: Box::new(e),
                })?,
            Some("yaml") | Some("yml") => serde_yaml::from_str(&content)
                .map_err(|e| WebAppConfigError::SerializationError {
                    source: Box::new(e),
                })?,
            _ => return Err(WebAppConfigError::ValidationError {
                field: "file_format".to_string(),
                message: format!("Unsupported file format for: {}", path.display()),
            }),
        };

        // Validate the loaded configuration
        config.validate()?;
        
        Ok(config)
    }

    fn validate(&self) -> Result<(), Self::Error> {
        // Validate application name
        if self.app_name.is_empty() {
            return Err(WebAppConfigError::ValidationError {
                field: "app_name".to_string(),
                message: "Application name cannot be empty".to_string(),
            });
        }

        if self.app_name.len() > 50 {
            return Err(WebAppConfigError::ValidationError {
                field: "app_name".to_string(),
                message: "Application name should not exceed 50 characters".to_string(),
            });
        }

        // Validate bind address
        if self.bind_address.is_empty() {
            return Err(WebAppConfigError::ValidationError {
                field: "bind_address".to_string(),
                message: "Bind address cannot be empty".to_string(),
            });
        }

        // Basic IP address validation
        if !self.bind_address.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return Err(WebAppConfigError::ValidationError {
                field: "bind_address".to_string(),
                message: "Invalid IP address format".to_string(),
            });
        }

        // Validate port
        if self.port == 0 {
            return Err(WebAppConfigError::ValidationError {
                field: "port".to_string(),
                message: "Port must be greater than 0".to_string(),
            });
        }

        if self.port < 1024 && self.bind_address != "127.0.0.1" {
            return Err(WebAppConfigError::ValidationError {
                field: "port".to_string(),
                message: "Ports below 1024 require privileges on non-localhost addresses".to_string(),
            });
        }

        // Validate timeout
        if self.request_timeout_seconds == 0 {
            return Err(WebAppConfigError::ValidationError {
                field: "request_timeout_seconds".to_string(),
                message: "Request timeout must be greater than 0".to_string(),
            });
        }

        if self.request_timeout_seconds > 300 {
            return Err(WebAppConfigError::ValidationError {
                field: "request_timeout_seconds".to_string(),
                message: "Request timeout should not exceed 300 seconds".to_string(),
            });
        }

        // Validate max connections
        if self.max_connections == 0 {
            return Err(WebAppConfigError::ValidationError {
                field: "max_connections".to_string(),
                message: "Max connections must be greater than 0".to_string(),
            });
        }

        if self.max_connections > 100_000 {
            return Err(WebAppConfigError::ValidationError {
                field: "max_connections".to_string(),
                message: "Max connections should not exceed 100,000".to_string(),
            });
        }

        Ok(())
    }

    fn report_event(&self, event: Self::Event) {
        // Log the event
        log::info!("WebApp config event: {:?}", event);
        
        // Send to metrics system (example)
        match event.event_type {
            WebAppEventType::ConfigLoaded => {
                metrics::counter!("config.events", 1, "type" => "loaded", "app" => &self.app_name);
            }
            WebAppEventType::ConfigSaved => {
                metrics::counter!("config.events", 1, "type" => "saved", "app" => &self.app_name);
            }
            WebAppEventType::FieldChanged => {
                if let Some(field) = &event.field {
                    metrics::counter!("config.field_changes", 1, "field" => field, "app" => &self.app_name);
                }
            }
            WebAppEventType::FeatureToggled => {
                metrics::counter!("config.feature_toggles", 1, "app" => &self.app_name);
            }
            _ => {}
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
```

### ConfigLifecycle Implementation

```rust
#[async_trait]
impl ConfigLifecycle for WebAppConfig {
    async fn save(&self, path: &Path) -> Result<(), Self::Error> {
        // Validate before saving
        self.validate()?;

        // Serialize based on file extension
        let content = match path.extension().and_then(|ext| ext.to_str()) {
            Some("toml") => toml::to_string_pretty(self)?,
            Some("json") => serde_json::to_string_pretty(self)
                .map_err(|e| WebAppConfigError::SerializationError {
                    source: Box::new(e),
                })?,
            Some("yaml") | Some("yml") => serde_yaml::to_string(self)
                .map_err(|e| WebAppConfigError::SerializationError {
                    source: Box::new(e),
                })?,
            _ => return Err(WebAppConfigError::ValidationError {
                field: "file_format".to_string(),
                message: format!("Unsupported file format for: {}", path.display()),
            }),
        };

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| WebAppConfigError::SaveError {
                    path: path.display().to_string(),
                    source: Box::new(e),
                })?;
        }

        // Write atomically using temporary file
        let temp_path = path.with_extension("tmp");
        tokio::fs::write(&temp_path, content).await
            .map_err(|e| WebAppConfigError::SaveError {
                path: temp_path.display().to_string(),
                source: Box::new(e),
            })?;

        tokio::fs::rename(&temp_path, path).await
            .map_err(|e| WebAppConfigError::SaveError {
                path: path.display().to_string(),
                source: Box::new(e),
            })?;

        // Report save event
        self.report_event(WebAppConfigEvent::new(WebAppEventType::ConfigSaved));

        Ok(())
    }

    async fn reload(&mut self, path: &Path) -> Result<(), Self::Error> {
        let new_config = Self::load(path).await?;
        *self = new_config;
        
        // Report reload event
        self.report_event(WebAppConfigEvent::new(WebAppEventType::ConfigReloaded));
        
        Ok(())
    }

    fn get_metadata(&self) -> &datafold::config::traits::ConfigMetadata {
        // In a real implementation, you'd store metadata
        // This is a simplified example
        static DEFAULT_METADATA: datafold::config::traits::ConfigMetadata = 
            datafold::config::traits::ConfigMetadata {
                version: "1.0.0",
                last_modified: None,
                checksum: None,
            };
        &DEFAULT_METADATA
    }

    fn set_metadata(&mut self, _metadata: datafold::config::traits::ConfigMetadata) {
        // Store metadata appropriately
        // Implementation depends on your metadata storage strategy
    }
}
```

### Environment Variable Support

```rust
impl WebAppConfig {
    /// Apply environment variable overrides
    pub fn apply_env_overrides(&mut self) -> Result<(), WebAppConfigError> {
        use std::env;

        // Override application name
        if let Ok(app_name) = env::var("WEBAPP_NAME") {
            let old_value = self.app_name.clone();
            self.app_name = app_name.clone();
            self.report_event(WebAppConfigEvent::field_changed(
                "app_name".to_string(),
                old_value,
                app_name,
            ));
        }

        // Override bind address
        if let Ok(bind_address) = env::var("WEBAPP_BIND_ADDRESS") {
            let old_value = self.bind_address.clone();
            self.bind_address = bind_address.clone();
            self.report_event(WebAppConfigEvent::field_changed(
                "bind_address".to_string(),
                old_value,
                bind_address,
            ));
        }

        // Override port
        if let Ok(port_str) = env::var("WEBAPP_PORT") {
            let port: u16 = port_str.parse()
                .map_err(|_| WebAppConfigError::ValidationError {
                    field: "port".to_string(),
                    message: format!("Invalid port in WEBAPP_PORT: {}", port_str),
                })?;
            
            let old_value = self.port.to_string();
            self.port = port;
            self.report_event(WebAppConfigEvent::field_changed(
                "port".to_string(),
                old_value,
                port.to_string(),
            ));
        }

        // Override debug mode
        if let Ok(debug_str) = env::var("WEBAPP_DEBUG") {
            let debug_mode: bool = debug_str.parse()
                .map_err(|_| WebAppConfigError::ValidationError {
                    field: "debug_mode".to_string(),
                    message: format!("Invalid boolean in WEBAPP_DEBUG: {}", debug_str),
                })?;
            
            let old_value = self.debug_mode.to_string();
            self.debug_mode = debug_mode;
            self.report_event(WebAppConfigEvent::field_changed(
                "debug_mode".to_string(),
                old_value,
                debug_mode.to_string(),
            ));
        }

        // Re-validate after applying overrides
        self.validate()?;
        
        Ok(())
    }

    /// Get timeout as Duration for easier use in async code
    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.request_timeout_seconds)
    }

    /// Get server socket address
    pub fn socket_address(&self) -> String {
        format!("{}:{}", self.bind_address, self.port)
    }

    /// Check if a feature is enabled
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        match feature {
            "metrics" => self.features.enable_metrics,
            "tracing" => self.features.enable_tracing,
            "health_check" => self.features.enable_health_check,
            "admin_api" => self.features.enable_admin_api,
            _ => false,
        }
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
    // Load configuration from file
    let mut config = WebAppConfig::load(Path::new("webapp.toml")).await?;
    
    // Apply environment variable overrides
    config.apply_env_overrides()?;
    
    // Use configuration
    println!("Starting {} on {}", config.app_name, config.socket_address());
    
    // Example: Start server with configuration
    start_server(&config).await?;
    
    Ok(())
}

async fn start_server(config: &WebAppConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("Server would start on {} with {} max connections", 
             config.socket_address(), 
             config.max_connections);
    
    if config.is_feature_enabled("metrics") {
        println!("Metrics collection enabled");
    }
    
    if config.debug_mode {
        println!("Debug mode enabled");
    }
    
    Ok(())
}
```

### Configuration File Example

```toml
# webapp.toml
app_name = "MyWebApplication"
bind_address = "0.0.0.0"
port = 8080
request_timeout_seconds = 30
max_connections = 1000
debug_mode = false

[features]
enable_metrics = true
enable_tracing = false
enable_health_check = true
enable_admin_api = false
```

### Testing Example

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_basic_config_operations() {
        // Create a test configuration
        let config = WebAppConfig {
            app_name: "TestApp".to_string(),
            port: 3000,
            ..Default::default()
        };

        // Test validation
        assert!(config.validate().is_ok());

        // Test serialization roundtrip
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().with_extension("toml");
        
        config.save(&path).await.unwrap();
        let loaded_config = WebAppConfig::load(&path).await.unwrap();
        
        assert_eq!(config.app_name, loaded_config.app_name);
        assert_eq!(config.port, loaded_config.port);
    }

    #[test]
    fn test_validation_errors() {
        let mut config = WebAppConfig::default();
        
        // Test empty app name
        config.app_name = String::new();
        assert!(config.validate().is_err());
        
        // Test invalid port
        config.app_name = "TestApp".to_string();
        config.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_environment_overrides() {
        use std::env;
        
        // Set environment variables
        env::set_var("WEBAPP_NAME", "EnvTestApp");
        env::set_var("WEBAPP_PORT", "9000");
        
        let mut config = WebAppConfig::default();
        config.apply_env_overrides().unwrap();
        
        assert_eq!(config.app_name, "EnvTestApp");
        assert_eq!(config.port, 9000);
        
        // Clean up
        env::remove_var("WEBAPP_NAME");
        env::remove_var("WEBAPP_PORT");
    }

    #[test]
    fn test_utility_methods() {
        let config = WebAppConfig {
            request_timeout_seconds: 45,
            bind_address: "192.168.1.1".to_string(),
            port: 8080,
            ..Default::default()
        };

        assert_eq!(config.request_timeout(), Duration::from_secs(45));
        assert_eq!(config.socket_address(), "192.168.1.1:8080");
        assert!(config.is_feature_enabled("metrics"));
        assert!(!config.is_feature_enabled("unknown"));
    }
}
```

## Key Features Demonstrated

1. **Type Safety**: Compile-time guarantees for configuration correctness
2. **Async Operations**: All I/O operations are async for better performance
3. **Multi-Format Support**: TOML, JSON, and YAML configuration files
4. **Validation**: Comprehensive validation with detailed error messages
5. **Environment Variables**: Override configuration from environment
6. **Event Reporting**: Configuration change tracking and metrics
7. **Atomic Operations**: Safe file operations with atomic writes
8. **Testing**: Comprehensive test coverage for all functionality

This example provides a solid foundation for implementing configuration management in web applications using the DataFold traits system.