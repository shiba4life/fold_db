//! Platform-specific configuration management demonstration
//!
//! This example demonstrates the enhanced cross-platform configuration system
//! with platform-specific optimizations, keystore integration, and migration utilities.

use datafold::config::{
    create_platform_keystore, get_platform_info, ConfigChangeEvent, ConfigChangeSource,
    ConfigChangeType, ConfigMigrationManager, ConfigValue, EnhancedConfig,
    EnhancedConfigurationManager, EnhancedPlatformInfo, MigrationStrategy,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 DataFold Platform-Specific Configuration Demo");
    println!("================================================\n");

    // 1. Platform Detection and Capabilities
    demo_platform_detection().await?;

    // 2. Enhanced Configuration Management
    demo_enhanced_config_management().await?;

    // 3. Keystore Integration
    demo_keystore_integration().await?;

    // 4. Configuration Migration
    demo_configuration_migration().await?;

    // 5. Performance Monitoring
    demo_performance_monitoring().await?;

    // 6. Change Notifications
    demo_change_notifications().await?;

    println!("\n✅ All platform-specific features demonstrated successfully!");
    Ok(())
}

/// Demonstrate platform detection and capabilities
async fn demo_platform_detection() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Platform Detection and Capabilities");
    println!("---------------------------------------");

    let platform_info = get_platform_info();
    println!("Platform: {} ({})", platform_info.name, platform_info.arch);
    println!("Version: {}", platform_info.version);
    println!("XDG Support: {}", platform_info.supports_xdg);
    println!("Keyring Support: {}", platform_info.supports_keyring);
    println!("File Watching: {}", platform_info.supports_file_watching);

    let enhanced_info = EnhancedPlatformInfo::detect();
    println!("Enhanced Capabilities:");
    println!(
        "  - Keystore Available: {}",
        enhanced_info.keystore_available
    );
    println!(
        "  - File Watching Available: {}",
        enhanced_info.file_watching_available
    );
    println!(
        "  - Atomic Operations Available: {}",
        enhanced_info.atomic_operations_available
    );
    println!(
        "  - Memory Mapping Available: {}",
        enhanced_info.memory_mapping_available
    );

    println!("✅ Platform detection completed\n");
    Ok(())
}

/// Demonstrate enhanced configuration management
async fn demo_enhanced_config_management() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚙️  Enhanced Configuration Management");
    println!("------------------------------------");

    // Create enhanced configuration manager
    let manager = EnhancedConfigurationManager::new().await?;
    println!("✅ Created enhanced configuration manager");

    // Get current configuration
    let config = manager.get_enhanced().await?;
    println!("📋 Loaded configuration (version: {})", config.base.version);
    println!(
        "   Platform optimizations: {}",
        config.platform_settings.enable_optimizations
    );
    println!("   Lazy loading: {}", config.performance.lazy_loading);
    println!(
        "   Auto-encrypt sensitive: {}",
        config.security_enhanced.auto_encrypt_sensitive
    );

    // Create a sample enhanced configuration
    let mut enhanced_config = EnhancedConfig::default();

    // Add application configuration
    let mut app_config = HashMap::new();
    app_config.insert(
        "name".to_string(),
        ConfigValue::string("DataFold Platform Demo"),
    );
    app_config.insert("version".to_string(), ConfigValue::string("2.0.0"));
    app_config.insert(
        "environment".to_string(),
        ConfigValue::string("development"),
    );
    enhanced_config
        .base
        .set_section("app".to_string(), ConfigValue::object(app_config));

    // Add database configuration
    let mut db_config = HashMap::new();
    db_config.insert("host".to_string(), ConfigValue::string("localhost"));
    db_config.insert("port".to_string(), ConfigValue::integer(5432));
    db_config.insert("database".to_string(), ConfigValue::string("datafold_demo"));
    db_config.insert("ssl_enabled".to_string(), ConfigValue::boolean(true));
    enhanced_config
        .base
        .set_section("database".to_string(), ConfigValue::object(db_config));

    // Configure platform-specific settings
    enhanced_config.platform_settings.enable_optimizations = true;
    enhanced_config.platform_settings.use_native_file_ops = true;
    enhanced_config.platform_settings.enable_memory_mapping = true;

    // Configure performance settings
    enhanced_config.performance.lazy_loading = true;
    enhanced_config.performance.cache_ttl_secs = 600;
    enhanced_config.performance.max_cache_size_mb = 50;

    // Store enhanced configuration
    manager.set_enhanced(enhanced_config).await?;
    println!("💾 Stored enhanced configuration with platform optimizations");

    // Test configuration retrieval and caching
    let start_time = std::time::Instant::now();
    let cached_config = manager.get_enhanced().await?;
    let load_time = start_time.elapsed();
    println!("⚡ Configuration loaded in {:?} (cached)", load_time);

    println!("✅ Enhanced configuration management completed\n");
    Ok(())
}

/// Demonstrate keystore integration for secure storage
async fn demo_keystore_integration() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Keystore Integration");
    println!("-----------------------");

    let keystore = create_platform_keystore();
    println!("🔑 Created platform keystore: {}", keystore.keystore_type());
    println!("   Available: {}", keystore.is_available());

    if keystore.is_available() {
        // Store some sensitive configuration data
        let sensitive_data = r#"{
            "api_key": "sk-1234567890abcdef",
            "database_password": "super_secret_password",
            "encryption_key": "0123456789abcdef0123456789abcdef"
        }"#;

        keystore
            .store_secret("demo_sensitive_config", sensitive_data.as_bytes())
            .await?;
        println!("🔒 Stored sensitive configuration in keystore");

        // Retrieve sensitive data
        if let Some(retrieved_data) = keystore.get_secret("demo_sensitive_config").await? {
            let config_str = String::from_utf8(retrieved_data)?;
            let config: serde_json::Value = serde_json::from_str(&config_str)?;
            println!("🔓 Retrieved sensitive configuration:");
            println!(
                "   API Key: {}",
                config["api_key"].as_str().unwrap_or("N/A")
            );
            println!("   Database Password: [REDACTED]");
            println!("   Encryption Key: [REDACTED]");
        }

        // List all stored keys
        let keys = keystore.list_keys().await?;
        println!("📋 Stored keys in keystore: {:?}", keys);

        // Clean up demo data
        keystore.delete_secret("demo_sensitive_config").await?;
        println!("🗑️  Cleaned up demo keystore data");
    } else {
        println!("⚠️  Keystore not available on this platform, using fallback storage");
    }

    println!("✅ Keystore integration completed\n");
    Ok(())
}

/// Demonstrate configuration migration utilities
async fn demo_configuration_migration() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Configuration Migration");
    println!("--------------------------");

    let migration_manager = ConfigMigrationManager::new();

    // Simulate legacy JSON configuration
    let legacy_json = r#"{
        "version": "1.0",
        "app": {
            "name": "Legacy DataFold",
            "debug": true
        },
        "database": {
            "url": "sqlite://legacy.db"
        }
    }"#;

    // Create temporary legacy file
    let temp_dir = tempfile::tempdir()?;
    let legacy_path = temp_dir.path().join("legacy_config.json");
    let target_path = temp_dir.path().join("migrated_config.toml");

    tokio::fs::write(&legacy_path, legacy_json).await?;
    println!("📄 Created legacy JSON configuration");

    // Perform migration
    let migration_result = migration_manager
        .migrate_config_file(&legacy_path, &target_path, MigrationStrategy::Transform)
        .await?;

    println!("🔄 Migration completed:");
    println!("   Success: {}", migration_result.success);
    println!("   Source: {}", migration_result.source_path.display());
    println!("   Target: {}", migration_result.target_path.display());
    println!(
        "   Sections migrated: {}",
        migration_result.sections_migrated
    );

    if !migration_result.warnings.is_empty() {
        println!("   Warnings: {:?}", migration_result.warnings);
    }

    // Show migrated content
    if target_path.exists() {
        let migrated_content = tokio::fs::read_to_string(&target_path).await?;
        println!("📋 Migrated TOML configuration:");
        println!("{}", migrated_content);
    }

    // Test comprehensive migration
    println!("\n🔄 Testing comprehensive migration...");
    let all_results = migration_manager.migrate_all().await?;
    for (i, result) in all_results.iter().enumerate() {
        println!(
            "   Migration {}: {} ({})",
            i + 1,
            if result.success {
                "✅ Success"
            } else {
                "❌ Failed"
            },
            result
                .source_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        );
    }

    println!("✅ Configuration migration completed\n");
    Ok(())
}

/// Demonstrate performance monitoring
async fn demo_performance_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Performance Monitoring");
    println!("-------------------------");

    let manager = EnhancedConfigurationManager::new().await?;

    // Perform multiple configuration operations to generate metrics
    for i in 0..5 {
        let _config = manager.get_enhanced().await?;
        if i % 2 == 0 {
            // Simulate cache clear to test cache misses
            manager.clear_cache().await?;
        }
        sleep(Duration::from_millis(10)).await;
    }

    // Get performance metrics
    let metrics = manager.get_metrics().await;
    println!("📈 Performance Metrics:");
    println!("   Total loads: {}", metrics.total_loads);
    println!("   Cache hits: {}", metrics.cache_hits);
    println!("   Cache misses: {}", metrics.cache_misses);
    println!("   Average load time: {:.2}ms", metrics.avg_load_time_ms);
    println!(
        "   Cache hit ratio: {:.1}%",
        if metrics.total_loads > 0 {
            (metrics.cache_hits as f64 / metrics.total_loads as f64) * 100.0
        } else {
            0.0
        }
    );

    println!("✅ Performance monitoring completed\n");
    Ok(())
}

/// Demonstrate configuration change notifications
async fn demo_change_notifications() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔔 Configuration Change Notifications");
    println!("-------------------------------------");

    let manager = Arc::new(EnhancedConfigurationManager::new().await?);

    // Register change callback
    let callback_manager = manager.clone();
    manager
        .on_change(move |event: ConfigChangeEvent| {
            println!("🔔 Configuration change detected:");
            println!("   Type: {:?}", event.change_type);
            println!("   Source: {:?}", event.source);
            println!("   Timestamp: {}", event.timestamp);
            if let Some(section) = &event.section {
                println!("   Section: {}", section);
            }
        })
        .await?;

    println!("👂 Registered change notification callback");

    // Trigger some configuration changes
    let mut config = manager.get_enhanced().await?;
    let mut new_config = (*config).clone();

    // Simulate adding a new section
    let mut new_section = HashMap::new();
    new_section.insert("feature_flag".to_string(), ConfigValue::boolean(true));
    new_section.insert("timeout_ms".to_string(), ConfigValue::integer(5000));
    new_config
        .base
        .set_section("features".to_string(), ConfigValue::object(new_section));

    manager.set_enhanced(new_config).await?;
    println!("🔧 Triggered configuration change");

    // Give callbacks time to execute
    sleep(Duration::from_millis(100)).await;

    println!("✅ Change notifications completed\n");
    Ok(())
}

/// Integration example showing CLI and node configuration working together
#[allow(dead_code)]
async fn demo_unified_integration() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Unified Configuration Integration");
    println!("-----------------------------------");

    // This would demonstrate how CLI, node, and logging configurations
    // all work together with the enhanced platform-specific system

    // For the demo, we'll show the concept with mock integrations
    println!("🖥️  CLI Configuration Integration:");
    println!("   - Authentication profiles stored in platform keystore");
    println!("   - Configuration paths use platform-specific directories");
    println!("   - Migration from legacy JSON to new TOML format");

    println!("\n🔧 Node Configuration Integration:");
    println!("   - Storage paths use platform data directories");
    println!("   - Crypto configuration secured in keystore");
    println!("   - Performance settings optimized per platform");

    println!("\n📝 Logging Configuration Integration:");
    println!("   - Log files stored in platform logs directory");
    println!("   - Platform-specific rotation and compression");
    println!("   - Unified configuration format across all systems");

    println!("\n🔄 Migration and Compatibility:");
    println!("   - Automatic detection and migration of legacy configs");
    println!("   - Backward compatibility maintained");
    println!("   - Graceful fallbacks for unsupported features");

    println!("✅ Unified integration demonstration completed\n");
    Ok(())
}

/// Helper function to display configuration tree
#[allow(dead_code)]
fn display_config_tree(config: &EnhancedConfig, indent: usize) {
    let prefix = "  ".repeat(indent);

    println!("{}📁 Configuration Structure:", prefix);
    println!("{}├── Version: {}", prefix, config.base.version);
    println!("{}├── Platform Settings:", prefix);
    println!(
        "{}│   ├── Optimizations: {}",
        prefix, config.platform_settings.enable_optimizations
    );
    println!(
        "{}│   ├── Native File Ops: {}",
        prefix, config.platform_settings.use_native_file_ops
    );
    println!(
        "{}│   └── Memory Mapping: {}",
        prefix, config.platform_settings.enable_memory_mapping
    );
    println!("{}├── Performance Settings:", prefix);
    println!(
        "{}│   ├── Lazy Loading: {}",
        prefix, config.performance.lazy_loading
    );
    println!(
        "{}│   ├── Cache TTL: {}s",
        prefix, config.performance.cache_ttl_secs
    );
    println!(
        "{}│   └── Max Cache Size: {}MB",
        prefix, config.performance.max_cache_size_mb
    );
    println!(
        "{}└── Sections: {} configured",
        prefix,
        config.base.sections.len()
    );
}

/// Helper function to create sample configuration
#[allow(dead_code)]
fn create_sample_config() -> EnhancedConfig {
    let mut config = EnhancedConfig::default();

    // Sample application configuration
    let mut app_config = HashMap::new();
    app_config.insert("name".to_string(), ConfigValue::string("DataFold Demo"));
    app_config.insert("version".to_string(), ConfigValue::string("2.0.0"));
    app_config.insert("debug".to_string(), ConfigValue::boolean(false));
    config
        .base
        .set_section("app".to_string(), ConfigValue::object(app_config));

    // Sample service configuration
    let mut service_config = HashMap::new();
    service_config.insert("host".to_string(), ConfigValue::string("0.0.0.0"));
    service_config.insert("port".to_string(), ConfigValue::integer(8080));
    service_config.insert("workers".to_string(), ConfigValue::integer(4));
    config
        .base
        .set_section("service".to_string(), ConfigValue::object(service_config));

    config
}
