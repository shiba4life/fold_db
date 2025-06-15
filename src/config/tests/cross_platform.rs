//! Cross-platform testing framework for configuration management
//!
//! This module tests all platform-specific implementations across Linux, macOS,
//! and Windows, ensuring consistent behavior and compatibility.

use std::collections::HashMap;
use std::path::PathBuf;
use tokio::time::{timeout, Duration};

use crate::config::{
    cross_platform::{Config, ConfigurationManager, TomlConfigProvider},
    enhanced::{EnhancedConfig, EnhancedConfigurationManager},
    value::ConfigValue,
    platform::{
        create_platform_resolver, get_platform_info, PlatformConfigPaths,
        linux::LinuxConfigPaths, macos::MacOSConfigPaths, windows::WindowsConfigPaths,
    },
};

use super::{
    mocks::{MockPlatformPaths, MockKeystore, MockFileWatcher, MockAtomicOps},
    utils::*,
    constants::*,
    create_test_dir, init_test_env,
};

/// Test suite for cross-platform path resolution
#[cfg(test)]
mod path_resolution_tests {
    use super::*;

    #[test]
    fn test_linux_path_resolution() {
        init_test_env();
        let paths = LinuxConfigPaths::new();
        
        // Test basic path resolution
        let config_dir = paths.config_dir().unwrap();
        let data_dir = paths.data_dir().unwrap();
        let cache_dir = paths.cache_dir().unwrap();
        let logs_dir = paths.logs_dir().unwrap();
        let runtime_dir = paths.runtime_dir().unwrap();
        
        // Verify paths contain datafold
        assert!(config_dir.to_string_lossy().to_lowercase().contains("datafold"));
        assert!(data_dir.to_string_lossy().to_lowercase().contains("datafold"));
        assert!(cache_dir.to_string_lossy().to_lowercase().contains("datafold"));
        assert!(logs_dir.to_string_lossy().to_lowercase().contains("datafold"));
        assert!(runtime_dir.to_string_lossy().to_lowercase().contains("datafold"));
        
        // Test platform name
        assert_eq!(paths.platform_name(), "linux");
        
        // Test file paths
        let config_file = paths.config_file().unwrap();
        let legacy_file = paths.legacy_config_file().unwrap();
        assert!(config_file.to_string_lossy().ends_with("config.toml"));
        assert!(legacy_file.to_string_lossy().ends_with("config.json"));
    }

    #[test]
    fn test_macos_path_resolution() {
        init_test_env();
        let paths = MacOSConfigPaths::new();
        
        let config_dir = paths.config_dir().unwrap();
        let data_dir = paths.data_dir().unwrap();
        
        // macOS should use Library/Application Support
        assert!(config_dir.to_string_lossy().contains("Library/Application Support") ||
                config_dir.to_string_lossy().contains("DataFold"));
        assert!(data_dir.to_string_lossy().contains("Library/Application Support") ||
                data_dir.to_string_lossy().contains("DataFold"));
        
        assert_eq!(paths.platform_name(), "macos");
    }

    #[test]
    fn test_windows_path_resolution() {
        init_test_env();
        let paths = WindowsConfigPaths::new();
        
        let config_dir = paths.config_dir().unwrap();
        let data_dir = paths.data_dir().unwrap();
        
        // Windows should use AppData
        let config_str = config_dir.to_string_lossy().to_lowercase();
        let data_str = data_dir.to_string_lossy().to_lowercase();
        
        assert!(config_str.contains("appdata") || config_str.contains("datafold"));
        assert!(data_str.contains("appdata") || data_str.contains("datafold"));
        
        assert_eq!(paths.platform_name(), "windows");
    }

    #[test]
    fn test_cross_platform_consistency() {
        init_test_env();
        
        let platforms: Vec<Box<dyn PlatformConfigPaths>> = vec![
            Box::new(LinuxConfigPaths::new()),
            Box::new(MacOSConfigPaths::new()),
            Box::new(WindowsConfigPaths::new()),
        ];
        
        for platform in platforms {
            // All platforms should provide these paths
            assert!(platform.config_dir().is_ok());
            assert!(platform.data_dir().is_ok());
            assert!(platform.cache_dir().is_ok());
            assert!(platform.logs_dir().is_ok());
            assert!(platform.runtime_dir().is_ok());
            
            // All platforms should provide these files
            assert!(platform.config_file().is_ok());
            assert!(platform.legacy_config_file().is_ok());
            
            // Platform name should be valid
            assert!(!platform.platform_name().is_empty());
        }
    }

    #[test]
    fn test_mock_platform_paths() {
        init_test_env();
        let temp_dir = create_test_dir("mock_platform");
        let mock_paths = MockPlatformPaths::new(temp_dir.path().to_path_buf(), "mock");
        
        let config_dir = mock_paths.config_dir().unwrap();
        let data_dir = mock_paths.data_dir().unwrap();
        
        assert_eq!(config_dir, temp_dir.path().join("config"));
        assert_eq!(data_dir, temp_dir.path().join("data"));
        assert_eq!(mock_paths.platform_name(), "mock");
    }

    #[tokio::test]
    async fn test_path_validation_and_creation() {
        init_test_env();
        let temp_dir = create_test_dir("path_validation");
        let mock_paths = MockPlatformPaths::new(temp_dir.path().to_path_buf(), "test");
        
        // Initially directories shouldn't exist
        assert!(!mock_paths.config_dir().unwrap().exists());
        
        // Validation should create directories
        assert!(mock_paths.validate_paths().is_ok());
        
        // Now directories should exist
        assert!(mock_paths.config_dir().unwrap().exists());
        assert!(mock_paths.data_dir().unwrap().exists());
        assert!(mock_paths.cache_dir().unwrap().exists());
        assert!(mock_paths.logs_dir().unwrap().exists());
        assert!(mock_paths.runtime_dir().unwrap().exists());
    }
}

/// Test suite for platform-specific configuration providers
#[cfg(test)]
mod configuration_provider_tests {
    use super::*;

    #[tokio::test]
    async fn test_toml_provider_basic_operations() {
        init_test_env();
        let temp_dir = create_test_dir("toml_provider");
        let config_file = temp_dir.path().join("test_config.toml");
        
        let provider = TomlConfigProvider::with_path(&config_file);
        
        // Initially config shouldn't exist
        assert!(!provider.exists().await.unwrap());
        
        // Create test configuration
        let mut config = Config::new();
        config.version = "2.0.0".to_string();
        
        let mut app_section = HashMap::new();
        app_section.insert("name".to_string(), ConfigValue::string("test_app"));
        app_section.insert("debug".to_string(), ConfigValue::boolean(true));
        config.set_section("app".to_string(), ConfigValue::object(app_section));
        
        // Save configuration
        provider.save(&config).await.unwrap();
        
        // Now config should exist
        assert!(provider.exists().await.unwrap());
        
        // Load and verify
        let loaded_config = provider.load().await.unwrap();
        assert_eq!(loaded_config.version, "2.0.0");
        assert_eq!(loaded_config.get_value("app.name").unwrap().as_string().unwrap(), "test_app");
        assert_eq!(loaded_config.get_value("app.debug").unwrap().as_bool().unwrap(), true);
    }

    #[tokio::test]
    async fn test_configuration_manager_with_mock_platform() {
        init_test_env();
        let temp_dir = create_test_dir("config_manager_mock");
        let config_file = temp_dir.path().join("config.toml");
        
        let manager = ConfigurationManager::with_toml_file(&config_file);
        
        // Test get creates default config
        let config = manager.get().await.unwrap();
        assert_eq!(config.version, "1.0.0");
        
        // Test set and get round-trip
        let mut new_config = Config::new();
        new_config.version = "2.1.0".to_string();
        
        let mut db_config = HashMap::new();
        db_config.insert("host".to_string(), ConfigValue::string("localhost"));
        db_config.insert("port".to_string(), ConfigValue::integer(5432));
        new_config.set_section("database".to_string(), ConfigValue::object(db_config));
        
        manager.set(new_config).await.unwrap();
        
        // Clear cache and reload
        manager.clear_cache().await;
        let reloaded = manager.get().await.unwrap();
        
        assert_eq!(reloaded.version, "2.1.0");
        assert_eq!(reloaded.get_value("database.host").unwrap().as_string().unwrap(), "localhost");
        assert_eq!(reloaded.get_value("database.port").unwrap().as_integer().unwrap(), 5432);
    }

    #[tokio::test]
    async fn test_enhanced_configuration_manager() {
        init_test_env();
        
        // Note: This test would need the full enhanced config implementation
        // For now, test what we can with the basic structure
        
        let temp_dir = create_test_dir("enhanced_config");
        let config_file = temp_dir.path().join("enhanced_config.toml");
        
        // Test with basic configuration manager for now
        let manager = ConfigurationManager::with_toml_file(&config_file);
        
        let config = manager.get().await.unwrap();
        assert!(!config.version.is_empty());
        
        // Test performance requirements
        let start_time = std::time::Instant::now();
        let _config = manager.get().await.unwrap(); // Should be cached
        let load_time = start_time.elapsed();
        
        // Should be much faster than 10ms for cached config
        assert!(load_time < MAX_LOAD_TIME);
    }
}

/// Test suite for platform feature compatibility
#[cfg(test)]
mod platform_compatibility_tests {
    use super::*;

    #[test]
    fn test_platform_info_detection() {
        init_test_env();
        let info = get_platform_info();
        
        // Basic platform info should be available
        assert!(!info.name.is_empty());
        assert!(!info.arch.is_empty());
        
        // Platform-specific checks
        match info.name.as_str() {
            "linux" => {
                assert!(info.supports_xdg);
                assert!(info.supports_keyring);
                assert!(info.supports_file_watching);
            }
            "macos" => {
                assert!(!info.supports_xdg); // macOS doesn't use XDG
                assert!(info.supports_keyring);
                assert!(info.supports_file_watching);
            }
            "windows" => {
                assert!(!info.supports_xdg); // Windows doesn't use XDG
                assert!(info.supports_keyring);
                assert!(info.supports_file_watching);
            }
            _ => {
                // Unknown platform - should have some basic capabilities
                assert!(info.supports_file_watching);
            }
        }
    }

    #[test]
    fn test_platform_resolver_creation() {
        init_test_env();
        let resolver = create_platform_resolver();
        
        // Should be able to resolve all required paths
        assert!(resolver.config_dir().is_ok());
        assert!(resolver.data_dir().is_ok());
        assert!(resolver.cache_dir().is_ok());
        assert!(resolver.logs_dir().is_ok());
        assert!(resolver.runtime_dir().is_ok());
        
        // Platform name should be valid
        let platform_name = resolver.platform_name();
        assert!(["linux", "macos", "windows"].contains(&platform_name));
    }

    #[tokio::test] 
    async fn test_platform_specific_optimizations() {
        init_test_env();
        
        // Test that different platforms can handle the same configuration
        let temp_dir = create_test_dir("platform_optimizations");
        
        let test_configs = vec![
            ("linux", MockPlatformPaths::new(temp_dir.path().join("linux"), "linux")),
            ("macos", MockPlatformPaths::new(temp_dir.path().join("macos"), "macos")),
            ("windows", MockPlatformPaths::new(temp_dir.path().join("windows"), "windows")),
        ];
        
        for (platform_name, paths) in test_configs {
            // Ensure directories exist
            paths.ensure_directories().unwrap();
            
            // Create configuration file
            let config_file = paths.config_file().unwrap();
            let provider = TomlConfigProvider::with_path(&config_file);
            
            // Create test configuration
            let mut config = Config::new();
            config.version = "1.0.0".to_string();
            
            let mut platform_section = HashMap::new();
            platform_section.insert("platform".to_string(), ConfigValue::string(platform_name));
            platform_section.insert("optimized".to_string(), ConfigValue::boolean(true));
            config.set_section("platform".to_string(), ConfigValue::object(platform_section));
            
            // Save and load
            provider.save(&config).await.unwrap();
            let loaded = provider.load().await.unwrap();
            
            // Verify platform-specific data
            assert_eq!(loaded.get_value("platform.platform").unwrap().as_string().unwrap(), platform_name);
            assert_eq!(loaded.get_value("platform.optimized").unwrap().as_bool().unwrap(), true);
        }
    }
}

/// Test suite for configuration format compatibility
#[cfg(test)]
mod format_compatibility_tests {
    use super::*;

    #[tokio::test]
    async fn test_toml_json_round_trip() {
        init_test_env();
        let temp_dir = create_test_dir("format_compatibility");
        
        // Create complex configuration
        let mut config = Config::new();
        config.version = "1.5.0".to_string();
        
        // Add various data types
        let mut complex_section = HashMap::new();
        complex_section.insert("string_val".to_string(), ConfigValue::string("test_string"));
        complex_section.insert("int_val".to_string(), ConfigValue::integer(42));
        complex_section.insert("float_val".to_string(), ConfigValue::float(3.14159));
        complex_section.insert("bool_val".to_string(), ConfigValue::boolean(true));
        
        // Add array
        let array_data = vec![
            ConfigValue::string("item1"),
            ConfigValue::string("item2"),
            ConfigValue::string("item3"),
        ];
        complex_section.insert("array_val".to_string(), ConfigValue::array(array_data));
        
        // Add nested object
        let mut nested_obj = HashMap::new();
        nested_obj.insert("nested_key".to_string(), ConfigValue::string("nested_value"));
        complex_section.insert("nested_obj".to_string(), ConfigValue::object(nested_obj));
        
        config.set_section("complex".to_string(), ConfigValue::object(complex_section));
        
        // Test TOML serialization/deserialization
        let toml_file = temp_dir.path().join("config.toml");
        let toml_provider = TomlConfigProvider::with_path(&toml_file);
        
        toml_provider.save(&config).await.unwrap();
        let toml_loaded = toml_provider.load().await.unwrap();
        
        // Verify all data preserved
        assert_eq!(toml_loaded.version, "1.5.0");
        assert_eq!(toml_loaded.get_value("complex.string_val").unwrap().as_string().unwrap(), "test_string");
        assert_eq!(toml_loaded.get_value("complex.int_val").unwrap().as_integer().unwrap(), 42);
        assert_eq!(toml_loaded.get_value("complex.bool_val").unwrap().as_bool().unwrap(), true);
        
        let loaded_array = toml_loaded.get_value("complex.array_val").unwrap().as_array().unwrap();
        assert_eq!(loaded_array.len(), 3);
        assert_eq!(loaded_array[0].as_string().unwrap(), "item1");
        
        assert_eq!(toml_loaded.get_value("complex.nested_obj.nested_key").unwrap().as_string().unwrap(), "nested_value");
    }

    #[tokio::test]
    async fn test_configuration_validation() {
        init_test_env();
        let temp_dir = create_test_dir("config_validation");
        let config_file = temp_dir.path().join("validation_test.toml");
        
        let provider = TomlConfigProvider::with_path(&config_file);
        
        // Test invalid configuration (empty version)
        let mut invalid_config = Config::new();
        invalid_config.version = "".to_string(); // Invalid
        
        let result = provider.validate(&invalid_config).await;
        assert!(result.is_err());
        
        // Test valid configuration
        let mut valid_config = Config::new();
        valid_config.version = "1.0.0".to_string();
        
        let result = provider.validate(&valid_config).await;
        assert!(result.is_ok());
    }
}

/// Test utilities for cross-platform testing
pub mod test_utils {
    use super::*;

    /// Create test configuration for a specific platform
    pub fn create_platform_test_config(platform: &str) -> Config {
        let mut config = Config::new();
        config.version = "1.0.0".to_string();
        
        let mut platform_section = HashMap::new();
        platform_section.insert("name".to_string(), ConfigValue::string(platform));
        platform_section.insert("timestamp".to_string(), 
            ConfigValue::string(chrono::Utc::now().to_rfc3339()));
        
        config.set_section("platform".to_string(), ConfigValue::object(platform_section));
        config
    }

    /// Verify configuration contains expected platform-specific data
    pub fn verify_platform_config(config: &Config, expected_platform: &str) {
        assert_eq!(config.get_value("platform.name").unwrap().as_string().unwrap(), expected_platform);
        assert!(!config.get_value("platform.timestamp").unwrap().as_string().unwrap().is_empty());
    }

    /// Test configuration operations with timeout
    pub async fn test_with_timeout<F, R>(test_fn: F) -> R 
    where
        F: std::future::Future<Output = R>,
    {
        timeout(TEST_TIMEOUT, test_fn)
            .await
            .expect("Test timed out")
    }
}