//! Example demonstrating automatic signature injection in DataFold CLI
//! 
//! This example shows how to use the enhanced CLI with automatic signing capabilities.
//! Run with: cargo run --example cli-automatic-signing-example

use datafold::cli::config::CliConfigManager;
use datafold::cli::signing_config::{SigningMode, EnhancedSigningConfig};
use datafold::cli::auth::CliAuthProfile;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 DataFold CLI Automatic Signature Injection Example");
    println!("=====================================================");
    println!();

    // Create a sample configuration manager
    let temp_dir = std::env::temp_dir().join("datafold-cli-example");
    std::fs::create_dir_all(&temp_dir)?;
    
    let config_path = temp_dir.join("config.toml");
    let mut config_manager = CliConfigManager::with_path(&config_path)?;

    // Configure automatic signing
    println!("📋 1. Configuring automatic signature injection...");
    
    // Enable auto-signing globally
    config_manager.set_auto_signing_enabled(true);
    config_manager.set_default_signing_mode(SigningMode::Auto);
    
    // Configure command-specific signing
    config_manager.set_command_signing_mode("query".to_string(), SigningMode::Auto)?;
    config_manager.set_command_signing_mode("mutate".to_string(), SigningMode::Manual)?;
    config_manager.set_command_signing_mode("auth-status".to_string(), SigningMode::Disabled)?;
    
    // Enable debug mode
    config_manager.set_signing_debug(true);
    
    println!("✅ Automatic signing configured:");
    println!("   • Global: enabled");
    println!("   • Default mode: auto");
    println!("   • Query commands: auto-sign");
    println!("   • Mutate commands: manual sign (--sign flag required)");
    println!("   • Auth status: no signing");
    println!("   • Debug mode: enabled");
    println!();

    // Create sample authentication profiles
    println!("📋 2. Creating authentication profiles...");
    
    // Development profile
    let mut dev_metadata = HashMap::new();
    dev_metadata.insert("environment".to_string(), "development".to_string());
    dev_metadata.insert("created_by".to_string(), "example".to_string());
    
    let dev_profile = CliAuthProfile {
        client_id: "datafold-cli-dev".to_string(),
        key_id: "dev-key".to_string(),
        user_id: Some("dev-user".to_string()),
        server_url: "http://localhost:8080".to_string(),
        metadata: dev_metadata,
    };
    
    config_manager.add_profile("development".to_string(), dev_profile)?;
    
    // Production profile
    let mut prod_metadata = HashMap::new();
    prod_metadata.insert("environment".to_string(), "production".to_string());
    prod_metadata.insert("created_by".to_string(), "example".to_string());
    
    let prod_profile = CliAuthProfile {
        client_id: "datafold-cli-prod".to_string(),
        key_id: "prod-key".to_string(),
        user_id: Some("prod-user".to_string()),
        server_url: "https://api.company.com".to_string(),
        metadata: prod_metadata,
    };
    
    config_manager.add_profile("production".to_string(), prod_profile)?;
    
    // Set development as default
    config_manager.set_default_profile("development".to_string())?;
    
    println!("✅ Authentication profiles created:");
    println!("   • development (default): http://localhost:8080");
    println!("   • production: https://api.company.com");
    println!();

    // Save configuration
    config_manager.save()?;
    println!("💾 Configuration saved to: {}", config_path.display());
    println!();

    // Demonstrate signing behavior
    println!("📋 3. Demonstrating automatic signing behavior...");
    
    let signing_config = config_manager.signing_config();
    
    // Test different command contexts
    let test_commands = vec![
        ("query", "Query operations"),
        ("mutate", "Mutation operations"), 
        ("auth-status", "Authentication status"),
        ("custom-command", "Custom command (uses default)"),
    ];
    
    for (command, description) in test_commands {
        let context = signing_config.for_command(command);
        println!("🔍 {}: {}", description, command);
        println!("   • Signing mode: {}", context.mode.as_str());
        println!("   • Auto-sign: {}", context.should_auto_sign);
        println!("   • Allows explicit: {}", context.allows_explicit);
        
        // Test with different explicit flags
        println!("   • With --sign flag: {}", context.should_sign_request(Some(true)));
        println!("   • With --no-sign flag: {}", context.should_sign_request(Some(false)));
        println!("   • No explicit flag: {}", context.should_sign_request(None));
        println!();
    }

    // Show configuration summary
    println!("📋 4. Configuration Summary");
    println!("==========================");
    let config = config_manager.config();
    
    println!("🔐 Signing Configuration:");
    println!("   • Global enabled: {}", config.signing.auto_signing.enabled);
    println!("   • Default mode: {}", config.signing.auto_signing.default_mode.as_str());
    println!("   • Debug enabled: {}", config.signing.debug.enabled);
    
    if let Some(env_var) = &config.signing.auto_signing.env_override {
        println!("   • Environment override: {}", env_var);
    }
    
    println!();
    println!("👥 Authentication Profiles:");
    let profiles = config_manager.list_profiles();
    for profile_name in &profiles {
        let is_default = config.default_profile.as_ref() == Some(profile_name);
        let marker = if is_default { " (default)" } else { "" };
        
        if let Some(profile) = config_manager.get_profile(profile_name) {
            println!("   • {}{}: {}", profile_name, marker, profile.server_url);
            println!("     Client ID: {}", profile.client_id);
            println!("     Key ID: {}", profile.key_id);
        }
    }
    
    println!();
    println!("📝 Command Overrides:");
    if config.signing.auto_signing.command_overrides.is_empty() {
        println!("   (none)");
    } else {
        for (cmd, mode) in &config.signing.auto_signing.command_overrides {
            println!("   • {}: {}", cmd, mode.as_str());
        }
    }
    
    println!();
    println!("🎛️  Performance Settings:");
    println!("   • Max signing time: {}ms", config.signing.performance.max_signing_time_ms);
    println!("   • Cache keys: {}", config.signing.performance.cache_keys);
    println!("   • Max concurrent signs: {}", config.signing.performance.max_concurrent_signs);
    
    println!();
    println!("✅ Example complete! This demonstrates:");
    println!("   • Configurable automatic signature injection");
    println!("   • Per-command signing behavior");
    println!("   • Multiple authentication profiles");
    println!("   • TOML configuration management");
    println!("   • Debug and performance settings");
    println!();
    
    println!("💡 In a real CLI application, you would:");
    println!("   1. Generate actual Ed25519 key pairs");
    println!("   2. Register public keys with the server");
    println!("   3. Use the authenticated HTTP client for requests");
    println!("   4. Let the CLI automatically sign based on configuration");
    
    // Clean up
    std::fs::remove_dir_all(&temp_dir)?;
    
    Ok(())
}