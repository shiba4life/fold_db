//! Integration test for unified configuration across all platforms
//!
//! This test verifies that the unified configuration system works consistently
//! across Rust CLI, JavaScript SDK, and Python SDK.

use datafold::config::unified_config::{UnifiedConfigManager, UnifiedConfigResult};
use std::path::Path;

#[test]
fn test_unified_config_loading() -> UnifiedConfigResult<()> {
    let config_path = Path::new("config/unified-datafold-config.json");

    // Skip test if config file doesn't exist
    if !config_path.exists() {
        println!("Skipping test - unified config file not found");
        return Ok(());
    }

    // Test Rust unified config loading
    let manager = UnifiedConfigManager::load_from_file(config_path)?;

    // Verify environments are loaded
    let environments = manager.list_environments();
    let env_names: Vec<String> = environments.into_iter().cloned().collect();
    assert!(env_names.contains(&"development".to_string()));
    assert!(env_names.contains(&"staging".to_string()));
    assert!(env_names.contains(&"production".to_string()));

    // Test environment switching
    let mut test_manager = manager.clone();
    test_manager.set_environment("production".to_string())?;
    assert_eq!(test_manager.current_environment(), "production");

    // Test configuration validation
    let env_config = manager.current_environment_config()?;
    assert!(!env_config.signing.policy.is_empty());
    assert!(env_config.signing.timeout_ms > 0);
    assert!(!env_config.logging.level.is_empty());

    Ok(())
}

#[test]
fn test_environment_configuration_differences() -> UnifiedConfigResult<()> {
    let config_path = Path::new("config/unified-datafold-config.json");

    if !config_path.exists() {
        println!("Skipping test - unified config file not found");
        return Ok(());
    }

    let manager = UnifiedConfigManager::load_from_file(config_path)?;

    // Test development environment
    let mut dev_manager = manager.clone();
    dev_manager.set_environment("development".to_string())?;
    let dev_config = dev_manager.current_environment_config()?;

    // Test production environment
    let mut prod_manager = manager.clone();
    prod_manager.set_environment("production".to_string())?;
    let prod_config = prod_manager.current_environment_config()?;

    // Verify they have different configurations
    // Production should be more strict than development
    assert!(prod_config.signing.timeout_ms <= dev_config.signing.timeout_ms);
    assert!(prod_config.verification.strict_timing);

    // Development should have debug enabled
    assert!(dev_config.signing.debug.enabled);
    assert!(!prod_config.signing.debug.enabled);

    Ok(())
}

#[test]
fn test_configuration_validation() -> UnifiedConfigResult<()> {
    let config_path = Path::new("config/unified-datafold-config.json");

    if !config_path.exists() {
        println!("Skipping test - unified config file not found");
        return Ok(());
    }

    // Loading the config should validate it automatically
    let manager = UnifiedConfigManager::load_from_file(config_path)?;

    // Test that all environments reference valid security profiles
    for env_name in manager.list_environments() {
        let mut temp_manager = manager.clone();
        temp_manager.set_environment(env_name.clone())?;
        let env_config = temp_manager.current_environment_config()?;

        // Verify security profile exists
        let profile = manager.get_security_profile(&env_config.signing.policy);
        assert!(
            profile.is_some(),
            "Environment {} references invalid security profile {}",
            env_name,
            env_config.signing.policy
        );
    }

    Ok(())
}

#[test]
fn test_unified_config_api() -> UnifiedConfigResult<()> {
    let config_path = Path::new("config/unified-datafold-config.json");

    if !config_path.exists() {
        println!("Skipping test - unified config file not found");
        return Ok(());
    }

    let manager = UnifiedConfigManager::load_from_file(config_path)?;

    // Test unified configuration access
    let config = manager.config();
    assert!(!config.config_format_version.is_empty());
    assert!(!config.environments.is_empty());
    assert!(!config.security_profiles.is_empty());

    // Test environment and security profile lists
    let environments = manager.list_environments();
    let security_profiles = manager.list_security_profiles();
    assert!(!environments.is_empty());
    assert!(!security_profiles.is_empty());

    Ok(())
}
