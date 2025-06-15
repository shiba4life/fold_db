//! Configuration validation for cryptographic database initialization
//!
//! This module provides validation functions to ensure that configuration
//! is valid and consistent before proceeding with crypto initialization.

use crate::config::crypto::{CryptoConfig, MasterKeyConfig};
use crate::datafold_node::crypto_init::{CryptoInitError, CryptoInitResult};
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

    // Master key configuration validation
    validate_master_key_config(&crypto_config.master_key)?;

    // Key derivation configuration validation
    validate_key_derivation_config(crypto_config)?;

    // Security level validation
    validate_security_requirements(crypto_config)?;

    debug!("Comprehensive crypto configuration validation completed successfully");
    Ok(())
}

/// Validate basic crypto configuration structure and enablement
fn validate_basic_config(crypto_config: &CryptoConfig) -> CryptoInitResult<()> {
    debug!("Validating basic crypto configuration");

    // Use built-in validation from CryptoConfig
    crypto_config.validate().map_err(CryptoInitError::Crypto)?;

    if !crypto_config.enabled {
        debug!("Crypto is disabled - skipping validation");
        return Ok(());
    }

    debug!("Basic crypto configuration is valid");
    Ok(())
}

/// Validate master key configuration for database initialization
fn validate_master_key_config(master_key_config: &MasterKeyConfig) -> CryptoInitResult<()> {
    debug!("Validating master key configuration");

    match master_key_config {
        MasterKeyConfig::Random => {
            debug!("Random key generation - no additional validation required");
            Ok(())
        }

        MasterKeyConfig::Passphrase { passphrase } => validate_passphrase_config(passphrase),

        MasterKeyConfig::External { .. } => Err(CryptoInitError::InvalidConfig(
            "External key sources are not supported for database initialization".to_string(),
        )),
    }
}

/// Validate passphrase configuration for security requirements
fn validate_passphrase_config(passphrase: &str) -> CryptoInitResult<()> {
    debug!("Validating passphrase configuration");

    if passphrase.is_empty() {
        return Err(CryptoInitError::InvalidConfig(
            "Passphrase cannot be empty".to_string(),
        ));
    }

    if passphrase.len() < 8 {
        warn!("Passphrase is shorter than 8 characters - this may be insecure");
        // Allow but warn - some users may want shorter passphrases for testing
        // Only fail for extremely short passphrases
        if passphrase.len() < 6 {
            return Err(CryptoInitError::InvalidConfig(
                "Passphrase must be at least 6 characters".to_string(),
            ));
        }
    }

    if passphrase.len() > 1024 {
        return Err(CryptoInitError::InvalidConfig(
            "Passphrase exceeds maximum length of 1024 characters".to_string(),
        ));
    }

    // Check for obviously weak passphrases (but don't fail, just warn)
    let _ = validate_passphrase_strength(passphrase);

    debug!("Passphrase configuration validated successfully");
    Ok(())
}

/// Validate passphrase strength (basic checks)
fn validate_passphrase_strength(passphrase: &str) -> CryptoInitResult<()> {
    // Check for common weak patterns
    let weak_patterns = [
        "password", "123456", "qwerty", "admin", "test", "guest", "default", "root", "user", "demo",
    ];

    let passphrase_lower = passphrase.to_lowercase();
    for pattern in &weak_patterns {
        if passphrase_lower.contains(pattern) {
            warn!("Passphrase contains common weak pattern: {}", pattern);
            // Don't fail, just warn - users should be able to choose their passphrases
            break;
        }
    }

    // Check for very simple patterns
    if is_simple_pattern(passphrase) {
        warn!(
            "Passphrase appears to be a simple pattern - consider using a more complex passphrase"
        );
    }

    Ok(())
}

/// Check if passphrase is a simple pattern
fn is_simple_pattern(passphrase: &str) -> bool {
    // Check for all same character
    if passphrase
        .chars()
        .all(|c| c == passphrase.chars().next().unwrap())
    {
        return true;
    }

    // Check for simple sequences (123456, abcdef, etc.)
    let chars: Vec<char> = passphrase.chars().collect();
    if chars.len() >= 3 {
        let mut is_sequence = true;
        for i in 1..chars.len() {
            if chars[i] as u8 != chars[i - 1] as u8 + 1 {
                is_sequence = false;
                break;
            }
        }
        if is_sequence {
            return true;
        }
    }

    false
}

/// Validate key derivation configuration
fn validate_key_derivation_config(crypto_config: &CryptoConfig) -> CryptoInitResult<()> {
    debug!("Validating key derivation configuration");

    // Convert to Argon2 parameters to validate (validation happens in to_argon2_params)
    let _params = crypto_config
        .key_derivation
        .to_argon2_params()
        .map_err(CryptoInitError::Crypto)?;

    // Additional validation for initialization context
    if let Some(preset) = &crypto_config.key_derivation.preset {
        debug!("Key derivation security level: {}", preset.as_str());

        // Warn about performance implications of high security levels
        if preset == &crate::security_types::SecurityLevel::High {
            warn!("Using 'High' security level - key derivation will be very slow");
            warn!("Consider using 'Low' level unless high security is required");
        }
    } else {
        debug!("Using custom key derivation parameters");
    }

    debug!("Key derivation configuration validated successfully");
    Ok(())
}

/// Validate overall security requirements and consistency
fn validate_security_requirements(crypto_config: &CryptoConfig) -> CryptoInitResult<()> {
    debug!("Validating security requirements");

    // Check consistency between master key type and security level
    match &crypto_config.master_key {
        MasterKeyConfig::Random => {
            // Random keys are always secure regardless of derivation settings
            debug!("Random key generation provides maximum security");
        }

        MasterKeyConfig::Passphrase { passphrase } => {
            // Check if passphrase length matches security level expectations
            if let Some(preset) = &crypto_config.key_derivation.preset {
                validate_passphrase_security_level_consistency(passphrase, preset)?;
            }
        }

        MasterKeyConfig::External { .. } => {
            // Already validated as unsupported above
        }
    }

    debug!("Security requirements validation completed");
    Ok(())
}

/// Validate consistency between passphrase and security level
fn validate_passphrase_security_level_consistency(
    passphrase: &str,
    security_level: &crate::security_types::SecurityLevel,
) -> CryptoInitResult<()> {
    use crate::security_types::SecurityLevel;

    match security_level {
        SecurityLevel::Standard => {
            if passphrase.len() < 8 {
                warn!("Short passphrase with Standard security level may be vulnerable");
            }
        }

        SecurityLevel::Low => {
            if passphrase.len() < 12 {
                warn!("Consider using a longer passphrase with Low security level");
            }
        }

        SecurityLevel::High => {
            if passphrase.len() < 16 {
                warn!("High security level recommended with passphrases of 16+ characters");
            }
        }
    }

    Ok(())
}

/// Validate crypto configuration specifically for new database creation
///
/// This validation is specific to database creation scenarios and includes
/// additional checks that may not apply to other crypto operations.
///
/// # Arguments
/// * `crypto_config` - Crypto configuration for new database
///
/// # Returns
/// * `Ok(())` - Configuration is suitable for database creation
/// * `Err(CryptoInitError)` - Configuration not suitable for database creation
pub fn validate_for_database_creation(crypto_config: &CryptoConfig) -> CryptoInitResult<()> {
    debug!("Validating crypto configuration for database creation");

    // Perform comprehensive validation first
    validate_crypto_config_comprehensive(crypto_config)?;

    // Additional checks specific to database creation
    validate_database_creation_requirements(crypto_config)?;

    debug!("Database creation validation completed successfully");
    Ok(())
}

/// Validate requirements specific to database creation
fn validate_database_creation_requirements(crypto_config: &CryptoConfig) -> CryptoInitResult<()> {
    debug!("Validating database creation specific requirements");

    // For database creation validation, crypto must be enabled
    if !crypto_config.enabled {
        return Err(CryptoInitError::InvalidConfig(
            "Crypto must be enabled for database creation validation".to_string(),
        ));
    }

    // Validate that the configuration will work for initialization
    match &crypto_config.master_key {
        MasterKeyConfig::Random => {
            debug!("Random key generation is suitable for database creation");
        }

        MasterKeyConfig::Passphrase { .. } => {
            debug!("Passphrase-based key derivation is suitable for database creation");
            // Additional passphrase-specific validation already done above
        }

        MasterKeyConfig::External { .. } => {
            return Err(CryptoInitError::InvalidConfig(
                "External key sources cannot be used for initial database creation".to_string(),
            ));
        }
    }

    debug!("Database creation requirements validated successfully");
    Ok(())
}

/// Quick validation for crypto configuration (lightweight check)
///
/// This function performs essential validation checks without comprehensive
/// security analysis. Suitable for quick checks before more expensive operations.
///
/// # Arguments
/// * `crypto_config` - Crypto configuration to validate
///
/// # Returns
/// * `Ok(())` - Configuration passes basic validation
/// * `Err(CryptoInitError)` - Configuration has critical issues
pub fn validate_crypto_config_quick(crypto_config: &CryptoConfig) -> CryptoInitResult<()> {
    debug!("Performing quick crypto configuration validation");

    // Basic structure validation
    crypto_config.validate().map_err(CryptoInitError::Crypto)?;

    // Check basic requirements
    if crypto_config.enabled {
        match &crypto_config.master_key {
            MasterKeyConfig::Passphrase { passphrase } => {
                if passphrase.is_empty() {
                    return Err(CryptoInitError::InvalidConfig(
                        "Passphrase cannot be empty".to_string(),
                    ));
                }
                if passphrase.len() > 1024 {
                    return Err(CryptoInitError::InvalidConfig(
                        "Passphrase too long".to_string(),
                    ));
                }
            }
            MasterKeyConfig::External { .. } => {
                return Err(CryptoInitError::InvalidConfig(
                    "External keys not supported".to_string(),
                ));
            }
            MasterKeyConfig::Random => {
                // Always valid
            }
        }
    }

    debug!("Quick crypto configuration validation completed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::crypto::KeyDerivationConfig;
    use crate::security_types::SecurityLevel;

    fn create_valid_random_config() -> CryptoConfig {
        CryptoConfig {
            enabled: true,
            master_key: MasterKeyConfig::Random,
            key_derivation: KeyDerivationConfig::default(),
        }
    }

    fn create_valid_passphrase_config() -> CryptoConfig {
        CryptoConfig {
            enabled: true,
            master_key: MasterKeyConfig::Passphrase {
                passphrase: "secure_test_passphrase_123".to_string(),
            },
            key_derivation: KeyDerivationConfig::for_security_level(SecurityLevel::Low),
        }
    }

    #[test]
    fn test_validate_crypto_config_comprehensive_valid() {
        let config = create_valid_random_config();
        assert!(validate_crypto_config_comprehensive(&config).is_ok());

        let config = create_valid_passphrase_config();
        assert!(validate_crypto_config_comprehensive(&config).is_ok());
    }

    #[test]
    fn test_validate_crypto_config_comprehensive_disabled() {
        let mut config = create_valid_random_config();
        config.enabled = false;

        let result = validate_crypto_config_comprehensive(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_passphrase_config() {
        // Valid passphrase
        assert!(validate_passphrase_config("secure_passphrase_123").is_ok());

        // Empty passphrase
        assert!(validate_passphrase_config("").is_err());

        // Too long passphrase
        let long_passphrase = "a".repeat(1025);
        assert!(validate_passphrase_config(&long_passphrase).is_err());
    }

    #[test]
    fn test_validate_passphrase_strength() {
        // Should not fail but may warn
        assert!(validate_passphrase_strength("password123").is_ok());
        assert!(validate_passphrase_strength("123456789").is_ok());
        assert!(validate_passphrase_strength("secure_test_phrase").is_ok());
    }

    #[test]
    fn test_is_simple_pattern() {
        assert!(is_simple_pattern("aaaaaaa"));
        assert!(is_simple_pattern("1234567"));
        assert!(is_simple_pattern("abcdefg"));
        assert!(!is_simple_pattern("secure123"));
        assert!(!is_simple_pattern("random_phrase"));
    }

    #[test]
    fn test_validate_for_database_creation() {
        let config = create_valid_random_config();
        assert!(validate_for_database_creation(&config).is_ok());

        let config = create_valid_passphrase_config();
        assert!(validate_for_database_creation(&config).is_ok());

        // Disabled crypto should fail for database creation
        let mut config = create_valid_random_config();
        config.enabled = false;
        assert!(validate_for_database_creation(&config).is_err());
    }

    #[test]
    fn test_validate_crypto_config_quick() {
        let config = create_valid_random_config();
        assert!(validate_crypto_config_quick(&config).is_ok());

        let config = create_valid_passphrase_config();
        assert!(validate_crypto_config_quick(&config).is_ok());

        // Empty passphrase should fail quick validation
        let config = CryptoConfig {
            enabled: true,
            master_key: MasterKeyConfig::Passphrase {
                passphrase: "".to_string(),
            },
            key_derivation: KeyDerivationConfig::default(),
        };
        assert!(validate_crypto_config_quick(&config).is_err());
    }

    #[test]
    fn test_external_key_validation() {
        let config = CryptoConfig {
            enabled: true,
            master_key: MasterKeyConfig::External {
                key_source: "test".to_string(),
            },
            key_derivation: KeyDerivationConfig::default(),
        };

        assert!(validate_crypto_config_comprehensive(&config).is_err());
        assert!(validate_for_database_creation(&config).is_err());
        assert!(validate_crypto_config_quick(&config).is_err());
    }
}
