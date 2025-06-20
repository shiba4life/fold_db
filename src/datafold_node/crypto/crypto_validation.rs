//! Configuration validation for cryptographic database initialization
//!
//! This module provides validation functions to ensure that configuration
//! is valid and consistent before proceeding with crypto initialization.

use crate::unified_crypto::{
    config::CryptoConfig,
    error::UnifiedCryptoError,
};
use crate::datafold_node::crypto::crypto_init::{CryptoInitError, CryptoInitResult};
use crate::security_types::SecurityLevel;
use log::{debug, warn};

/// Comprehensive validation of crypto configuration for database initialization
///
/// This function performs all necessary validation checks to ensure the
/// crypto configuration is suitable for database initialization, including
/// security requirements and compatibility checks.
///
/// # Arguments
/// * `crypto_config` - Crypto configuration to validate
///
/// # Returns
/// * `Ok(())` - Configuration is valid for database initialization
/// * `Err(CryptoInitError)` - Configuration validation failed
pub fn validate_crypto_config_comprehensive(crypto_config: &CryptoConfig) -> CryptoInitResult<()> {
    debug!("Starting comprehensive crypto configuration validation");

    // Basic configuration validation
    validate_basic_config(crypto_config)?;

    // Key management configuration validation
    validate_key_management_config(crypto_config)?;

    // Security level validation
    validate_security_requirements(crypto_config)?;

    // Policy validation
    validate_security_policies(crypto_config)?;

    debug!("Comprehensive crypto configuration validation completed successfully");
    Ok(())
}

/// Validate basic crypto configuration structure and enablement
fn validate_basic_config(crypto_config: &CryptoConfig) -> CryptoInitResult<()> {
    debug!("Validating basic crypto configuration");

    // Use built-in validation from CryptoConfig
    crypto_config.validate_security().map_err(CryptoInitError::Crypto)?;

    debug!("Basic crypto configuration is valid");
    Ok(())
}

/// Validate key management configuration for database initialization
fn validate_key_management_config(crypto_config: &CryptoConfig) -> CryptoInitResult<()> {
    debug!("Validating key management configuration");

    // Validate that the default algorithm is supported
    if !crypto_config.primitives.supported_algorithms.contains(&crypto_config.keys.default_algorithm) {
        return Err(CryptoInitError::InvalidConfig(
            "Default key algorithm not in supported algorithms list".to_string(),
        ));
    }

    // Validate key storage configuration
    crypto_config.keys.storage_backend.validate().map_err(CryptoInitError::Crypto)?;

    // Validate key rotation configuration if enabled
    if let Some(max_age) = crypto_config.policy.max_key_age {
        if max_age.as_secs() < 3600 {
            warn!("Key rotation configured for less than 1 hour - this may be too frequent");
        }
    }

    debug!("Key management configuration is valid");
    Ok(())
}

/// Validate security requirements based on configuration
fn validate_security_requirements(crypto_config: &CryptoConfig) -> CryptoInitResult<()> {
    debug!("Validating security requirements");

    let security_level = crypto_config.general.security_level;

    // Basic security level validation
    match security_level {
        SecurityLevel::Basic => {
            debug!("Using basic security level");
        }
        SecurityLevel::Low => {
            debug!("Using low security level");
        }
        SecurityLevel::Standard => {
            debug!("Using standard security level");
        }
        SecurityLevel::High => {
            debug!("Using high security level");
            validate_high_security_requirements(crypto_config)?;
        }
    }

    debug!("Security requirements validation completed");
    Ok(())
}

/// Validate high security level specific requirements
fn validate_high_security_requirements(crypto_config: &CryptoConfig) -> CryptoInitResult<()> {
    debug!("Validating high security level requirements");

    // Ensure hardware acceleration is enabled for high security
    if !crypto_config.general.hardware_acceleration {
        warn!("Hardware acceleration disabled for high security level - performance may be impacted");
    }

    // Ensure strict validation is enabled
    if !crypto_config.general.strict_validation {
        return Err(CryptoInitError::InvalidConfig(
            "Strict validation must be enabled for high security level".to_string(),
        ));
    }

    // Ensure audit logging is appropriately configured
    if !crypto_config.audit.enabled {
        return Err(CryptoInitError::InvalidConfig(
            "Audit logging must be enabled for high security level".to_string(),
        ));
    }

    debug!("High security level requirements validated");
    Ok(())
}

/// Validate security policies configuration
fn validate_security_policies(crypto_config: &CryptoConfig) -> CryptoInitResult<()> {
    debug!("Validating security policies");

    // Use built-in policy validation
    crypto_config.policy.validate().map_err(CryptoInitError::Crypto)?;

    debug!("Security policies validation completed");
    Ok(())
}

/// Quick validation of crypto configuration (lightweight checks)
///
/// This function performs essential validation checks with minimal overhead,
/// suitable for frequent validation scenarios.
///
/// # Arguments
/// * `crypto_config` - Crypto configuration to validate
///
/// # Returns
/// * `Ok(())` - Configuration passes quick validation
/// * `Err(CryptoInitError)` - Configuration validation failed
pub fn validate_crypto_config_quick(crypto_config: &CryptoConfig) -> CryptoInitResult<()> {
    debug!("Starting quick crypto configuration validation");

    // Check if crypto is enabled
    if !crypto_config.enabled {
        return Err(CryptoInitError::Validation(
            "Crypto configuration is disabled".to_string(),
        ));
    }

    // Basic validation only
    validate_basic_config(crypto_config)?;

    debug!("Quick crypto configuration validation completed successfully");
    Ok(())
}

/// Validate crypto configuration for database creation
///
/// This function validates the crypto configuration specifically for
/// database creation scenarios, ensuring all required components are present.
///
/// # Arguments
/// * `crypto_config` - Crypto configuration to validate
///
/// # Returns
/// * `Ok(())` - Configuration is suitable for database creation
/// * `Err(CryptoInitError)` - Configuration validation failed
pub fn validate_for_database_creation(crypto_config: &CryptoConfig) -> CryptoInitResult<()> {
    debug!("Starting database creation crypto configuration validation");

    // Perform comprehensive validation
    validate_crypto_config_comprehensive(crypto_config)?;

    // Additional database-specific checks
    if crypto_config.general.security_level == SecurityLevel::Basic {
        warn!("Using Basic security level for database creation - consider upgrading");
    }

    // Ensure key management is properly configured
    // Note: Algorithm is an enum, so we can't check if it's "empty"
    // This validation is satisfied by having a valid enum value
    debug!("Using default algorithm: {:?}", crypto_config.keys.default_algorithm);

    debug!("Database creation crypto configuration validation completed successfully");
    Ok(())
}

/// Validate configuration for specific algorithm
pub fn validate_algorithm_config(
    crypto_config: &CryptoConfig,
    algorithm: &crate::unified_crypto::types::Algorithm,
) -> CryptoInitResult<()> {
    debug!("Validating configuration for algorithm: {:?}", algorithm);

    // Check if algorithm is supported
    if !crypto_config.primitives.supported_algorithms.contains(algorithm) {
        return Err(CryptoInitError::InvalidConfig(
            format!("Algorithm {:?} is not supported by current configuration", algorithm),
        ));
    }

    // Check if algorithm meets security level requirements
    if !algorithm.is_approved_for_level(crypto_config.general.security_level) {
        return Err(CryptoInitError::InvalidConfig(
            format!(
                "Algorithm {:?} does not meet security level {:?} requirements",
                algorithm, crypto_config.general.security_level
            ),
        ));
    }

    debug!("Algorithm configuration validation completed");
    Ok(())
}

/// Quick validation for basic crypto setup
pub fn validate_minimal_crypto_config(crypto_config: &CryptoConfig) -> CryptoInitResult<()> {
    debug!("Performing minimal crypto configuration validation");

    // Just do basic validation - sufficient for testing/development
    crypto_config.validate_security().map_err(CryptoInitError::Crypto)?;

    debug!("Minimal crypto configuration validation completed");
    Ok(())
}

/// Validate crypto configuration compatibility with database format
pub fn validate_database_compatibility(
    crypto_config: &CryptoConfig,
    _database_version: &str,
) -> CryptoInitResult<()> {
    debug!("Validating crypto configuration compatibility with database");

    // For now, just validate the crypto config itself
    // Future versions could add specific database format compatibility checks
    crypto_config.validate_security().map_err(CryptoInitError::Crypto)?;

    debug!("Database compatibility validation completed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unified_crypto::config::CryptoConfig;
    use crate::security_types::SecurityLevel;

    #[test]
    fn test_basic_config_validation() {
        let config = CryptoConfig::default();
        assert!(validate_basic_config(&config).is_ok());
    }

    #[test]
    fn test_comprehensive_validation() {
        let config = CryptoConfig::default();
        assert!(validate_crypto_config_comprehensive(&config).is_ok());
    }

    #[test]
    fn test_high_security_validation() {
        let config = CryptoConfig::for_security_level(SecurityLevel::High);
        assert!(validate_crypto_config_comprehensive(&config).is_ok());
    }

    #[test]
    fn test_minimal_validation() {
        let config = CryptoConfig::default();
        assert!(validate_minimal_crypto_config(&config).is_ok());
    }

    #[test]
    fn test_algorithm_validation() {
        let config = CryptoConfig::default();
        let algorithm = crate::unified_crypto::types::Algorithm::Ed25519;
        assert!(validate_algorithm_config(&config, &algorithm).is_ok());
    }
}
