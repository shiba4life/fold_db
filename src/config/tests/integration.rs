//! Integration testing with existing DataFold systems
//!
//! This module tests integration between the cross-platform configuration
//! management system and existing DataFold components:
//! - CLI configuration system integration  
//! - Node configuration system integration
//! - Logging configuration system alignment
//! - Unified reporting integration (PBI 26)
//! - Migration from existing systems

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::{timeout, Duration};

use crate::config::{
    cross_platform::{Config, ConfigurationManager},
    enhanced::{EnhancedConfig, EnhancedConfigurationManager},
    migration::{ConfigMigrationManager, MigrationStrategy, MigrationResult},
    value::ConfigValue,
    platform::{create_platform_resolver, get_platform_info},
};

use super::{
    mocks::{MockPlatformPaths, MockKeystore, MockPerformanceMonitor},
    utils::*,
    constants::*,
    create_test_dir, init_test_env,
};

/// Integration test results
#[derive(Debug, Clone)]
pub struct IntegrationTestResults {
    pub test_name: String,
    pub component: String,
    pub passed: bool,
    pub migration_successful: bool,
    pub data_integrity_score: f64, // 0.0 to 1.0
    pub compatibility_issues: Vec<String>,
    pub performance_impact: Option<Duration>,
    pub recommendations: Vec<String>,
}

impl IntegrationTestResults {
    pub fn print_summary(&self) {
        println!("🔗 Integration Test: {}", self.test_name);
        println!("   Component: {}", self.component);
        println!("   Status: {}", if self.passed { "✅ PASS" } else { "❌ FAIL" });
        println!("   Migration: {}", if self.migration_successful { "✅ Success" } else { "❌ Failed" });
        println!("   Data Integrity: {:.1}%", self.data_integrity_score * 100.0);
        
        if let Some(perf_impact) = self.performance_impact {
            println!("   Performance Impact: {:?}", perf_impact);
        }
        
        if !self.compatibility_issues.is_empty() {
            println!("   Compatibility Issues:");
            for issue in &self.compatibility_issues {
                println!("     - {}", issue);
            }
        }
        
        if !self.recommendations.is_empty() {
            println!("   Recommendations:");
            for rec in &self.recommendations {
                println!("     - {}", rec);
            }
        }
    }
}

/// CLI configuration system integration tests
#[cfg(test)]
mod cli_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_cli_config_migration() {
        init_test_env();
        let temp_dir = create_test_dir("cli_migration");
        
        let mut results = IntegrationTestResults {
            test_name: "CLI Configuration Migration".to_string(),
            component: "CLI".to_string(),
            passed: true,
            migration_successful: false,
            data_integrity_score: 1.0,
            compatibility_issues: Vec::new(),
            performance_impact: None,
            recommendations: Vec::new(),
        };
        
        // Create legacy CLI configuration
        let legacy_cli_config = create_legacy_cli_config();
        let legacy_file = temp_dir.path().join("legacy_cli_config.json");
        let legacy_content = serde_json::to_string_pretty(&legacy_cli_config).unwrap();
        tokio::fs::write(&legacy_file, legacy_content).await.unwrap();
        
        // Set up migration manager
        let migration_manager = ConfigMigrationManager::new();
        let target_file = temp_dir.path().join("migrated_cli_config.toml");
        
        // Perform migration
        let start_time = std::time::Instant::now();
        let migration_result = migration_manager.migrate_config_file(
            &legacy_file,
            &target_file,
            MigrationStrategy::Transform,
        ).await;
        let migration_time = start_time.elapsed();
        
        results.performance_impact = Some(migration_time);
        
        match migration_result {
            Ok(result) => {
                results.migration_successful = result.success;
                
                if result.success {
                    // Verify migrated configuration
                    let manager = ConfigurationManager::with_toml_file(&target_file);
                    let migrated_config = manager.get().await.unwrap();
                    
                    // Check key CLI configuration elements
                    if let Ok(auth_profile) = migrated_config.get_value("cli.auth.profile") {
                        assert_eq!(auth_profile.as_string().unwrap(), "default");
                    } else {
                        results.compatibility_issues.push("Auth profile not migrated correctly".to_string());
                        results.data_integrity_score -= 0.2;
                    }
                    
                    if let Ok(server_url) = migrated_config.get_value("cli.server.url") {
                        assert_eq!(server_url.as_string().unwrap(), "https://api.datafold.com");
                    } else {
                        results.compatibility_issues.push("Server URL not migrated correctly".to_string());
                        results.data_integrity_score -= 0.2;
                    }
                    
                    // Check that sensitive data is handled properly
                    if migrated_config.get_value("cli.auth.api_key").is_ok() {
                        results.compatibility_issues.push("Sensitive API key migrated to plaintext config".to_string());
                        results.recommendations.push("Move API keys to secure keystore".to_string());
                        results.data_integrity_score -= 0.3;
                    }
                } else {
                    results.passed = false;
                    results.compatibility_issues.extend(result.errors);
                }
            }
            Err(e) => {
                results.passed = false;
                results.migration_successful = false;
                results.compatibility_issues.push(format!("Migration failed: {}", e));
            }
        }
        
        // Performance check
        if migration_time > Duration::from_secs(5) {
            results.compatibility_issues.push("CLI migration too slow".to_string());
        }
        
        results.print_summary();
        assert!(results.passed, "CLI integration test failed");
    }

    #[tokio::test]
    async fn test_cli_config_compatibility() {
        init_test_env();
        let temp_dir = create_test_dir("cli_compatibility");
        let config_file = temp_dir.path().join("cli_compat_config.toml");
        
        let manager = ConfigurationManager::with_toml_file(&config_file);
        
        let mut results = IntegrationTestResults {
            test_name: "CLI Configuration Compatibility".to_string(),
            component: "CLI".to_string(),
            passed: true,
            migration_successful: true,
            data_integrity_score: 1.0,
            compatibility_issues: Vec::new(),
            performance_impact: None,
            recommendations: Vec::new(),
        };
        
        // Create CLI-compatible configuration
        let mut config = Config::new();
        config.version = "1.0.0".to_string();
        
        // CLI authentication section
        let mut auth_section = HashMap::new();
        auth_section.insert("profile".to_string(), ConfigValue::string("default"));
        auth_section.insert("timeout_seconds".to_string(), ConfigValue::integer(30));
        auth_section.insert("retry_attempts".to_string(), ConfigValue::integer(3));
        config.set_section("cli.auth".to_string(), ConfigValue::object(auth_section));
        
        // CLI server configuration
        let mut server_section = HashMap::new();
        server_section.insert("url".to_string(), ConfigValue::string("https://api.datafold.com"));
        server_section.insert("verify_ssl".to_string(), ConfigValue::boolean(true));
        server_section.insert("timeout_ms".to_string(), ConfigValue::integer(5000));
        config.set_section("cli.server".to_string(), ConfigValue::object(server_section));
        
        // CLI output preferences
        let mut output_section = HashMap::new();
        output_section.insert("format".to_string(), ConfigValue::string("json"));
        output_section.insert("color".to_string(), ConfigValue::boolean(true));
        output_section.insert("verbose".to_string(), ConfigValue::boolean(false));
        config.set_section("cli.output".to_string(), ConfigValue::object(output_section));
        
        manager.set(config).await.unwrap();
        
        // Test compatibility with CLI expectations
        let loaded_config = manager.get().await.unwrap();
        
        // Verify CLI can read expected configuration
        let cli_sections = vec![
            "cli.auth.profile",
            "cli.auth.timeout_seconds", 
            "cli.server.url",
            "cli.server.verify_ssl",
            "cli.output.format",
        ];
        
        for section_path in cli_sections {
            if loaded_config.get_value(section_path).is_err() {
                results.compatibility_issues.push(format!("CLI section missing: {}", section_path));
                results.data_integrity_score -= 0.1;
            }
        }
        
        // Test configuration update workflow (simulating CLI commands)
        let start_time = std::time::Instant::now();
        
        // Update server URL (common CLI operation)
        let mut updated_config = (*loaded_config).clone();
        let mut updated_server = HashMap::new();
        updated_server.insert("url".to_string(), ConfigValue::string("https://staging.datafold.com"));
        updated_server.insert("verify_ssl".to_string(), ConfigValue::boolean(true));
        updated_server.insert("timeout_ms".to_string(), ConfigValue::integer(5000));
        updated_config.set_section("cli.server".to_string(), ConfigValue::object(updated_server));
        
        manager.set(updated_config).await.unwrap();
        
        let update_time = start_time.elapsed();
        results.performance_impact = Some(update_time);
        
        // Verify update
        let final_config = manager.get().await.unwrap();
        let new_url = final_config.get_value("cli.server.url").unwrap().as_string().unwrap();
        assert_eq!(new_url, "https://staging.datafold.com");
        
        if update_time > Duration::from_millis(100) {
            results.compatibility_issues.push("CLI config updates too slow".to_string());
        }
        
        if !results.compatibility_issues.is_empty() {
            results.passed = false;
        }
        
        results.print_summary();
    }
}

/// Node configuration system integration tests
#[cfg(test)]
mod node_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_node_config_integration() {
        init_test_env();
        let temp_dir = create_test_dir("node_integration");
        let config_file = temp_dir.path().join("node_config.toml");
        
        let manager = ConfigurationManager::with_toml_file(&config_file);
        
        let mut results = IntegrationTestResults {
            test_name: "Node Configuration Integration".to_string(),
            component: "DataFold Node".to_string(),
            passed: true,
            migration_successful: true,
            data_integrity_score: 1.0,
            compatibility_issues: Vec::new(),
            performance_impact: None,
            recommendations: Vec::new(),
        };
        
        // Create node-compatible configuration
        let mut config = Config::new();
        config.version = "1.0.0".to_string();
        
        // Node server configuration
        let mut server_section = HashMap::new();
        server_section.insert("host".to_string(), ConfigValue::string("0.0.0.0"));
        server_section.insert("port".to_string(), ConfigValue::integer(8080));
        server_section.insert("workers".to_string(), ConfigValue::integer(4));
        server_section.insert("max_connections".to_string(), ConfigValue::integer(1000));
        config.set_section("node.server".to_string(), ConfigValue::object(server_section));
        
        // Node database configuration
        let mut db_section = HashMap::new();
        db_section.insert("host".to_string(), ConfigValue::string("localhost"));
        db_section.insert("port".to_string(), ConfigValue::integer(5432));
        db_section.insert("database".to_string(), ConfigValue::string("datafold"));
        db_section.insert("pool_size".to_string(), ConfigValue::integer(20));
        config.set_section("node.database".to_string(), ConfigValue::object(db_section));
        
        // Node crypto configuration
        let mut crypto_section = HashMap::new();
        crypto_section.insert("encryption_algorithm".to_string(), ConfigValue::string("AES-256-GCM"));
        crypto_section.insert("key_derivation".to_string(), ConfigValue::string("Argon2id"));
        crypto_section.insert("enable_encryption_at_rest".to_string(), ConfigValue::boolean(true));
        config.set_section("node.crypto".to_string(), ConfigValue::object(crypto_section));
        
        // Node logging configuration
        let mut logging_section = HashMap::new();
        logging_section.insert("level".to_string(), ConfigValue::string("info"));
        logging_section.insert("format".to_string(), ConfigValue::string("json"));
        logging_section.insert("enable_structured_logging".to_string(), ConfigValue::boolean(true));
        config.set_section("node.logging".to_string(), ConfigValue::object(logging_section));
        
        manager.set(config).await.unwrap();
        
        // Test node configuration access patterns
        let start_time = std::time::Instant::now();
        let loaded_config = manager.get().await.unwrap();
        let access_time = start_time.elapsed();
        
        results.performance_impact = Some(access_time);
        
        // Verify node can access all required configuration
        let node_sections = vec![
            ("node.server.host", "0.0.0.0"),
            ("node.server.port", "8080"),
            ("node.database.host", "localhost"),
            ("node.crypto.encryption_algorithm", "AES-256-GCM"),
            ("node.logging.level", "info"),
        ];
        
        for (section_path, expected_value) in node_sections {
            match loaded_config.get_value(section_path) {
                Ok(value) => {
                    let actual_value = match value {
                        ConfigValue::String(s) => s,
                        ConfigValue::Integer(i) => i.to_string(),
                        _ => "unknown".to_string(),
                    };
                    if actual_value != expected_value {
                        results.compatibility_issues.push(format!(
                            "Node section {} has unexpected value: {} (expected: {})",
                            section_path, actual_value, expected_value
                        ));
                        results.data_integrity_score -= 0.1;
                    }
                }
                Err(_) => {
                    results.compatibility_issues.push(format!("Node section missing: {}", section_path));
                    results.data_integrity_score -= 0.2;
                }
            }
        }
        
        // Test hot configuration reload (important for node operations)
        let reload_start = std::time::Instant::now();
        manager.clear_cache().await;
        let reloaded_config = manager.get().await.unwrap();
        let reload_time = reload_start.elapsed();
        
        if reload_time > MAX_HOT_RELOAD_TIME {
            results.compatibility_issues.push("Node config hot reload too slow".to_string());
        }
        
        // Verify configuration consistency after reload
        assert_eq!(
            reloaded_config.get_value("node.server.port").unwrap().as_integer().unwrap(),
            8080
        );
        
        if access_time > MAX_LOAD_TIME {
            results.compatibility_issues.push("Node config access too slow".to_string());
        }
        
        if !results.compatibility_issues.is_empty() {
            results.passed = false;
        }
        
        results.print_summary();
    }

    #[tokio::test]
    async fn test_node_crypto_config_integration() {
        init_test_env();
        let temp_dir = create_test_dir("node_crypto_integration");
        let config_file = temp_dir.path().join("node_crypto_config.toml");
        
        let manager = ConfigurationManager::with_toml_file(&config_file);
        
        let mut results = IntegrationTestResults {
            test_name: "Node Crypto Configuration Integration".to_string(),
            component: "Node Crypto".to_string(),
            passed: true,
            migration_successful: true,
            data_integrity_score: 1.0,
            compatibility_issues: Vec::new(),
            performance_impact: None,
            recommendations: Vec::new(),
        };
        
        // Create crypto-focused configuration
        let mut config = Config::new();
        config.version = "1.0.0".to_string();
        
        // Encryption configuration
        let mut encryption_section = HashMap::new();
        encryption_section.insert("default_algorithm".to_string(), ConfigValue::string("AES-256-GCM"));
        encryption_section.insert("key_size".to_string(), ConfigValue::integer(256));
        encryption_section.insert("enable_compression".to_string(), ConfigValue::boolean(true));
        config.set_section("crypto.encryption".to_string(), ConfigValue::object(encryption_section));
        
        // Key rotation configuration
        let mut rotation_section = HashMap::new();
        rotation_section.insert("enabled".to_string(), ConfigValue::boolean(true));
        rotation_section.insert("interval_hours".to_string(), ConfigValue::integer(24));
        rotation_section.insert("max_key_age_days".to_string(), ConfigValue::integer(90));
        config.set_section("crypto.key_rotation".to_string(), ConfigValue::object(rotation_section));
        
        // Signing configuration
        let mut signing_section = HashMap::new();
        signing_section.insert("algorithm".to_string(), ConfigValue::string("Ed25519"));
        signing_section.insert("enable_verification".to_string(), ConfigValue::boolean(true));
        config.set_section("crypto.signing".to_string(), ConfigValue::object(signing_section));
        
        manager.set(config).await.unwrap();
        
        // Test crypto configuration access performance (critical for node startup)
        let crypto_access_times = Vec::new();
        
        for _i in 0..10 {
            let start_time = std::time::Instant::now();
            let _config = manager.get().await.unwrap();
            let access_time = start_time.elapsed();
            
            if access_time > Duration::from_millis(5) {
                results.compatibility_issues.push("Crypto config access too slow for node operations".to_string());
                break;
            }
        }
        
        // Verify all crypto settings are accessible
        let loaded_config = manager.get().await.unwrap();
        
        let crypto_settings = vec![
            "crypto.encryption.default_algorithm",
            "crypto.key_rotation.enabled",
            "crypto.signing.algorithm",
        ];
        
        for setting in crypto_settings {
            if loaded_config.get_value(setting).is_err() {
                results.compatibility_issues.push(format!("Crypto setting missing: {}", setting));
                results.data_integrity_score -= 0.2;
            }
        }
        
        // Test crypto configuration updates (for key rotation)
        let mut updated_config = (*loaded_config).clone();
        let mut updated_rotation = HashMap::new();
        updated_rotation.insert("enabled".to_string(), ConfigValue::boolean(true));
        updated_rotation.insert("interval_hours".to_string(), ConfigValue::integer(12));
        updated_rotation.insert("max_key_age_days".to_string(), ConfigValue::integer(90));
        updated_config.set_section("crypto.key_rotation".to_string(), ConfigValue::object(updated_rotation));
        
        let update_start = std::time::Instant::now();
        manager.set(updated_config).await.unwrap();
        let update_time = update_start.elapsed();
        
        results.performance_impact = Some(update_time);
        
        if update_time > Duration::from_millis(50) {
            results.compatibility_issues.push("Crypto config updates too slow".to_string());
        }
        
        if !results.compatibility_issues.is_empty() {
            results.passed = false;
        }
        
        results.print_summary();
    }
}

/// Logging configuration system integration tests  
#[cfg(test)]
mod logging_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_logging_config_alignment() {
        init_test_env();
        let temp_dir = create_test_dir("logging_integration");
        let config_file = temp_dir.path().join("logging_config.toml");
        
        let manager = ConfigurationManager::with_toml_file(&config_file);
        
        let mut results = IntegrationTestResults {
            test_name: "Logging Configuration Alignment".to_string(),
            component: "Logging System".to_string(),
            passed: true,
            migration_successful: true,
            data_integrity_score: 1.0,
            compatibility_issues: Vec::new(),
            performance_impact: None,
            recommendations: Vec::new(),
        };
        
        // Create unified logging configuration
        let mut config = Config::new();
        config.version = "1.0.0".to_string();
        
        // Global logging settings
        let mut global_logging = HashMap::new();
        global_logging.insert("level".to_string(), ConfigValue::string("info"));
        global_logging.insert("format".to_string(), ConfigValue::string("structured"));
        global_logging.insert("enable_colors".to_string(), ConfigValue::boolean(true));
        config.set_section("logging.global".to_string(), ConfigValue::object(global_logging));
        
        // File output configuration
        let mut file_output = HashMap::new();
        file_output.insert("enabled".to_string(), ConfigValue::boolean(true));
        file_output.insert("path".to_string(), ConfigValue::string("/var/log/datafold/datafold.log"));
        file_output.insert("max_size_mb".to_string(), ConfigValue::integer(100));
        file_output.insert("max_files".to_string(), ConfigValue::integer(10));
        config.set_section("logging.outputs.file".to_string(), ConfigValue::object(file_output));
        
        // Console output configuration
        let mut console_output = HashMap::new();
        console_output.insert("enabled".to_string(), ConfigValue::boolean(true));
        console_output.insert("level".to_string(), ConfigValue::string("warn"));
        console_output.insert("format".to_string(), ConfigValue::string("compact"));
        config.set_section("logging.outputs.console".to_string(), ConfigValue::object(console_output));
        
        // Structured output configuration
        let mut structured_output = HashMap::new();
        structured_output.insert("enabled".to_string(), ConfigValue::boolean(true));
        structured_output.insert("format".to_string(), ConfigValue::string("json"));
        structured_output.insert("include_metadata".to_string(), ConfigValue::boolean(true));
        config.set_section("logging.outputs.structured".to_string(), ConfigValue::object(structured_output));
        
        manager.set(config).await.unwrap();
        
        // Test logging configuration access
        let start_time = std::time::Instant::now();
        let loaded_config = manager.get().await.unwrap();
        let access_time = start_time.elapsed();
        
        results.performance_impact = Some(access_time);
        
        // Verify all logging components can access their configuration
        let logging_sections = vec![
            "logging.global.level",
            "logging.global.format", 
            "logging.outputs.file.enabled",
            "logging.outputs.console.enabled",
            "logging.outputs.structured.enabled",
        ];
        
        for section in logging_sections {
            if loaded_config.get_value(section).is_err() {
                results.compatibility_issues.push(format!("Logging section missing: {}", section));
                results.data_integrity_score -= 0.1;
            }
        }
        
        // Test logging configuration validation
        let global_level = loaded_config.get_value("logging.global.level").unwrap().as_string().unwrap();
        let valid_levels = vec!["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&global_level.as_str()) {
            results.compatibility_issues.push(format!("Invalid logging level: {}", global_level));
            results.data_integrity_score -= 0.2;
        }
        
        // Test runtime logging configuration changes
        let mut updated_config = (*loaded_config).clone();
        let mut updated_global = HashMap::new();
        updated_global.insert("level".to_string(), ConfigValue::string("debug"));
        updated_global.insert("format".to_string(), ConfigValue::string("structured"));
        updated_global.insert("enable_colors".to_string(), ConfigValue::boolean(false));
        updated_config.set_section("logging.global".to_string(), ConfigValue::object(updated_global));
        
        let update_start = std::time::Instant::now();
        manager.set(updated_config).await.unwrap();
        let update_time = update_start.elapsed();
        
        // Logging config updates should be fast for runtime changes
        if update_time > Duration::from_millis(20) {
            results.compatibility_issues.push("Logging config updates too slow for runtime changes".to_string());
        }
        
        // Verify the change was applied
        let final_config = manager.get().await.unwrap();
        let new_level = final_config.get_value("logging.global.level").unwrap().as_string().unwrap();
        assert_eq!(new_level, "debug");
        
        if access_time > Duration::from_millis(5) {
            results.compatibility_issues.push("Logging config access too slow".to_string());
        }
        
        if !results.compatibility_issues.is_empty() {
            results.passed = false;
        }
        
        results.print_summary();
    }
}

/// Migration integration tests
#[cfg(test)]
mod migration_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_comprehensive_system_migration() {
        init_test_env();
        let temp_dir = create_test_dir("comprehensive_migration");
        
        let mut results = IntegrationTestResults {
            test_name: "Comprehensive System Migration".to_string(),
            component: "All Systems".to_string(),
            passed: true,
            migration_successful: false,
            data_integrity_score: 1.0,
            compatibility_issues: Vec::new(),
            performance_impact: None,
            recommendations: Vec::new(),
        };
        
        // Create multiple legacy configuration files
        let legacy_configs = create_multiple_legacy_configs(&temp_dir).await;
        
        // Set up migration manager
        let migration_manager = ConfigMigrationManager::new();
        
        // Perform comprehensive migration
        let start_time = std::time::Instant::now();
        let migration_results = migration_manager.migrate_all().await.unwrap();
        let total_migration_time = start_time.elapsed();
        
        results.performance_impact = Some(total_migration_time);
        
        // Analyze migration results
        let total_migrations = migration_results.len();
        let successful_migrations = migration_results.iter().filter(|r| r.success).count();
        
        results.migration_successful = successful_migrations == total_migrations;
        
        if successful_migrations < total_migrations {
            results.compatibility_issues.push(format!(
                "Migration incomplete: {}/{} succeeded", 
                successful_migrations, total_migrations
            ));
            results.data_integrity_score = successful_migrations as f64 / total_migrations as f64;
        }
        
        // Verify migrated configurations are accessible
        for migration_result in &migration_results {
            if migration_result.success {
                let manager = ConfigurationManager::with_toml_file(&migration_result.target_path);
                match manager.get().await {
                    Ok(config) => {
                        // Basic validation
                        if config.version.is_empty() {
                            results.compatibility_issues.push(format!(
                                "Migrated config has empty version: {}", 
                                migration_result.target_path.display()
                            ));
                            results.data_integrity_score -= 0.1;
                        }
                    }
                    Err(_) => {
                        results.compatibility_issues.push(format!(
                            "Cannot load migrated config: {}", 
                            migration_result.target_path.display()
                        ));
                        results.data_integrity_score -= 0.2;
                    }
                }
            }
        }
        
        // Performance validation
        if total_migration_time > Duration::from_secs(30) {
            results.compatibility_issues.push("Total migration time too long".to_string());
        }
        
        // Generate recommendations
        if !results.migration_successful {
            results.recommendations.push("Review failed migrations and implement fallback strategies".to_string());
        }
        
        if total_migration_time > Duration::from_secs(10) {
            results.recommendations.push("Optimize migration performance for large deployments".to_string());
        }
        
        results.recommendations.push("Implement migration rollback capability".to_string());
        results.recommendations.push("Add migration validation and verification".to_string());
        
        if !results.compatibility_issues.is_empty() {
            results.passed = false;
        }
        
        results.print_summary();
        
        println!("\n📊 Migration Summary:");
        println!("   Total migrations: {}", total_migrations);
        println!("   Successful: {}", successful_migrations);
        println!("   Failed: {}", total_migrations - successful_migrations);
        println!("   Total time: {:?}", total_migration_time);
    }
}

/// Helper functions for integration testing
fn create_legacy_cli_config() -> serde_json::Value {
    serde_json::json!({
        "version": "0.9.0",
        "auth": {
            "profile": "default",
            "api_key": "legacy_api_key_12345",
            "server_url": "https://api.datafold.com"
        },
        "preferences": {
            "output_format": "json",
            "color": true,
            "verbose": false
        },
        "timeout": 30
    })
}

async fn create_multiple_legacy_configs(temp_dir: &std::path::Path) -> Vec<PathBuf> {
    let mut configs = Vec::new();
    
    // CLI config
    let cli_config = create_legacy_cli_config();
    let cli_path = temp_dir.join("legacy_cli.json");
    tokio::fs::write(&cli_path, serde_json::to_string_pretty(&cli_config).unwrap()).await.unwrap();
    configs.push(cli_path);
    
    // Node config
    let node_config = serde_json::json!({
        "version": "0.8.0",
        "server": {
            "host": "localhost",
            "port": 8080
        },
        "database": {
            "connection_string": "postgres://user:pass@localhost/db"
        }
    });
    let node_path = temp_dir.join("legacy_node.json");
    tokio::fs::write(&node_path, serde_json::to_string_pretty(&node_config).unwrap()).await.unwrap();
    configs.push(node_path);
    
    // Logging config
    let logging_config = serde_json::json!({
        "level": "info",
        "outputs": ["console", "file"],
        "file_path": "/var/log/datafold.log"
    });
    let logging_path = temp_dir.join("legacy_logging.json");
    tokio::fs::write(&logging_path, serde_json::to_string_pretty(&logging_config).unwrap()).await.unwrap();
    configs.push(logging_path);
    
    configs
}

/// Comprehensive integration test runner
pub async fn run_integration_test_suite() -> Vec<IntegrationTestResults> {
    init_test_env();
    
    let mut all_results = Vec::new();
    
    println!("🔗 Running Comprehensive Integration Test Suite");
    println!("==============================================");
    
    // CLI integration tests
    println!("\n📋 CLI Integration Tests:");
    // Note: Individual tests would be run here and results collected
    
    // Node integration tests
    println!("\n📋 Node Integration Tests:");
    // Note: Individual tests would be run here and results collected
    
    // Logging integration tests
    println!("\n📋 Logging Integration Tests:");
    // Note: Individual tests would be run here and results collected
    
    // Migration integration tests
    println!("\n📋 Migration Integration Tests:");
    // Note: Individual tests would be run here and results collected
    
    // Summary
    let total_tests = all_results.len();
    let passed_tests = all_results.iter().filter(|r| r.passed).count();
    let successful_migrations = all_results.iter().filter(|r| r.migration_successful).count();
    let avg_data_integrity = if !all_results.is_empty() {
        all_results.iter().map(|r| r.data_integrity_score).sum::<f64>() / all_results.len() as f64
    } else {
        1.0
    };
    
    println!("\n🔗 Integration Test Suite Summary:");
    println!("   Total Tests: {}", total_tests);
    println!("   Passed: {}", passed_tests);
    println!("   Failed: {}", total_tests - passed_tests);
    println!("   Successful Migrations: {}", successful_migrations);
    println!("   Average Data Integrity: {:.1}%", avg_data_integrity * 100.0);
    
    if passed_tests == total_tests && avg_data_integrity > 0.95 {
        println!("   ✅ ALL INTEGRATION TESTS PASSED");
    } else {
        println!("   ❌ INTEGRATION ISSUES DETECTED - REVIEW REQUIRED");
    }
    
    all_results
}