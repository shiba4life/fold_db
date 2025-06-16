//! Configuration system demonstration
//!
//! This example shows how to use the new unified configuration management system.

use datafold::config::{Config, ConfigValue, ConfigurationManager};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("DataFold Configuration System Demo");
    println!("==================================");

    // Create configuration manager
    let manager = ConfigurationManager::new();

    // Check if config exists
    let exists = manager.exists().await?;
    println!("Configuration exists: {}", exists);

    // Get current configuration (creates default if none exists)
    let config = manager.get().await?;
    println!("Configuration version: {}", config.version);
    println!("Platform: {:?}", config.metadata.get("platform"));

    // Create a new configuration with some sample data
    let mut new_config = Config::new();

    // Add application configuration
    let mut app_config = HashMap::new();
    app_config.insert("name".to_string(), ConfigValue::string("datafold-demo"));
    app_config.insert("version".to_string(), ConfigValue::string("1.0.0"));
    app_config.insert("debug".to_string(), ConfigValue::boolean(true));
    new_config.set_section("app".to_string(), ConfigValue::object(app_config));

    // Add logging configuration
    let mut logging_config = HashMap::new();
    logging_config.insert("level".to_string(), ConfigValue::string("info"));
    logging_config.insert("max_size_mb".to_string(), ConfigValue::integer(100));
    new_config.set_section("logging".to_string(), ConfigValue::object(logging_config));

    // Save the configuration
    manager.set(new_config).await?;
    println!("Configuration saved successfully");

    // Reload and verify
    let reloaded_config = manager.reload().await?;
    println!("Reloaded configuration:");

    if let Ok(app_name) = reloaded_config.get_value("app.name") {
        println!("  App name: {}", app_name.as_string()?);
    }

    if let Ok(log_level) = reloaded_config.get_value("logging.level") {
        println!("  Log level: {}", log_level.as_string()?);
    }

    // Show config file location
    let config_path = manager.config_path()?;
    println!("Configuration file: {}", config_path.display());

    // Show platform information
    let platform_info = datafold::config::get_platform_info();
    println!("Platform: {} ({})", platform_info.name, platform_info.arch);
    println!("XDG Support: {}", platform_info.supports_xdg);
    println!("Keyring Support: {}", platform_info.supports_keyring);

    Ok(())
}
