//! Configuration management for protocol validation

use crate::{ValidationConfig, rfc9421, security, cross_platform, performance};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Load validation configuration from file
pub async fn load_config<P: AsRef<Path>>(path: P) -> Result<ValidationConfig> {
    let content = tokio::fs::read_to_string(path.as_ref()).await
        .context("Failed to read configuration file")?;
    
    let config: ValidationConfig = serde_yaml::from_str(&content)
        .context("Failed to parse configuration file")?;
    
    validate_config(&config)
        .context("Configuration validation failed")?;
    
    Ok(config)
}

/// Validate configuration for consistency and correctness
pub fn validate_config(config: &ValidationConfig) -> Result<()> {
    // Validate that at least one category is enabled
    if config.enabled_categories.is_empty() {
        return Err(anyhow::anyhow!("No validation categories enabled"));
    }

    // Validate output directory is writable
    if !Path::new(&config.output_config.output_directory).exists() {
        if let Err(e) = std::fs::create_dir_all(&config.output_config.output_directory) {
            return Err(anyhow::anyhow!("Cannot create output directory: {}", e));
        }
    }

    Ok(())
}

/// Save configuration to file
pub async fn save_config<P: AsRef<Path>>(config: &ValidationConfig, path: P) -> Result<()> {
    let content = serde_yaml::to_string(config)
        .context("Failed to serialize configuration")?;
    
    tokio::fs::write(path.as_ref(), content).await
        .context("Failed to write configuration file")?;
    
    Ok(())
}

/// Create default configuration files for different scenarios
pub mod defaults {
    use super::*;

    /// Create configuration for CI/CD environments
    pub fn ci_config() -> ValidationConfig {
        let mut config = ValidationConfig::default();
        
        // Skip performance tests in CI for speed
        config.enabled_categories.retain(|c| *c != crate::ValidationCategory::Performance);
        
        // Disable DoS tests in CI
        config.security_config.enable_dos_simulation = false;
        
        // Use faster settings
        config.security_config.attack_duration_secs = 10;
        config.security_config.concurrent_attack_count = 10;
        
        // Enable all report formats for CI
        config.output_config.generate_html_report = true;
        config.output_config.generate_json_report = true;
        config.output_config.generate_junit_xml = true;
        
        config
    }

    /// Create configuration for development testing
    pub fn development_config() -> ValidationConfig {
        let mut config = ValidationConfig::default();
        
        // Enable all categories for comprehensive testing
        config.enabled_categories = vec![
            crate::ValidationCategory::RFC9421Compliance,
            crate::ValidationCategory::CrossPlatform,
            crate::ValidationCategory::Security,
            crate::ValidationCategory::Performance,
            crate::ValidationCategory::TestVectors,
        ];
        
        // Use lenient security settings for development
        config.security_config.enable_dos_simulation = false;
        config.security_config.attack_duration_secs = 30;
        
        // Include debug information
        config.output_config.include_debug_info = true;
        config.output_config.include_test_vectors = true;
        
        config
    }

    /// Create configuration for production validation
    pub fn production_config() -> ValidationConfig {
        let mut config = ValidationConfig::default();
        
        // Enable strict validation
        config.rfc9421_config.strict_header_validation = true;
        config.rfc9421_config.validate_component_ordering = true;
        
        // Use strict security settings
        config.security_config.enable_dos_simulation = false; // Still disabled for safety
        config.security_config.attack_duration_secs = 60;
        config.security_config.concurrent_attack_count = 100;
        
        // Disable debug information in production
        config.output_config.include_debug_info = false;
        
        config
    }

    /// Create minimal configuration for quick validation
    pub fn quick_config() -> ValidationConfig {
        let mut config = ValidationConfig::default();
        
        // Only run essential tests
        config.enabled_categories = vec![
            crate::ValidationCategory::RFC9421Compliance,
            crate::ValidationCategory::Security,
        ];
        
        // Reduce test counts for speed
        config.security_config.attack_duration_secs = 5;
        config.security_config.concurrent_attack_count = 5;
        config.cross_platform_config.random_test_count = 10;
        
        // Disable expensive tests
        config.security_config.enable_dos_simulation = false;
        config.cross_platform_config.enable_performance_comparison = false;
        
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_config_save_load_roundtrip() {
        let original_config = ValidationConfig::default();
        
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();
        
        // Save config
        save_config(&original_config, temp_path).await.unwrap();
        
        // Load config
        let loaded_config = load_config(temp_path).await.unwrap();
        
        // Verify they match (basic check)
        assert_eq!(original_config.enabled_categories.len(), 
                   loaded_config.enabled_categories.len());
    }

    #[test]
    fn test_validate_config() {
        let valid_config = ValidationConfig::default();
        assert!(validate_config(&valid_config).is_ok());
        
        let mut invalid_config = ValidationConfig::default();
        invalid_config.enabled_categories.clear();
        assert!(validate_config(&invalid_config).is_err());
    }

    #[test]
    fn test_default_configs() {
        // Test that all default configs are valid
        assert!(validate_config(&defaults::ci_config()).is_ok());
        assert!(validate_config(&defaults::development_config()).is_ok());
        assert!(validate_config(&defaults::production_config()).is_ok());
        assert!(validate_config(&defaults::quick_config()).is_ok());
    }
}