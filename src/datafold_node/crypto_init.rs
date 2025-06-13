//! Cryptographic initialization for DataFold database setup
//!
//! This module provides functions to initialize database encryption during
//! database creation, including key generation, metadata storage, and
//! configuration validation.

use crate::config::crypto::{ConfigError, CryptoConfig, KeyDerivationConfig, MasterKeyConfig};
use crate::crypto::{
    derive_master_keypair, generate_master_keypair, generate_salt, CryptoError, MasterKeyPair,
};
use crate::db_operations::{CryptoMetadata, DbOperations};
use crate::schema::SchemaError;
use log::{debug, info, warn};
use std::sync::Arc;

/// Result type for crypto initialization operations
pub type CryptoInitResult<T> = Result<T, CryptoInitError>;

/// Errors that can occur during crypto initialization
#[derive(Debug, thiserror::Error)]
pub enum CryptoInitError {
    /// Crypto configuration validation error
    #[error("Crypto configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Cryptographic operation error
    #[error("Cryptographic operation failed: {0}")]
    Crypto(#[from] CryptoError),

    /// Database operation error during crypto setup
    #[error("Database operation failed: {0}")]
    Database(#[from] SchemaError),

    /// Sled database error
    #[error("Database error: {0}")]
    Sled(#[from] sled::Error),

    /// Invalid configuration state
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Crypto initialization already completed
    #[error("Database already has crypto metadata initialized")]
    AlreadyInitialized,
}

/// Context for crypto initialization containing the generated keys and metadata
#[derive(Debug)]
pub struct CryptoInitContext {
    /// The generated master key pair
    pub master_keypair: MasterKeyPair,
    /// The crypto metadata to be stored
    pub metadata: CryptoMetadata,
    /// The key derivation method used
    pub derivation_method: String,
}

impl CryptoInitContext {
    /// Get the public key from the context
    pub fn public_key(&self) -> &crate::crypto::PublicKey {
        self.master_keypair.public_key()
    }

    /// Get the master key pair
    pub fn master_keypair(&self) -> &MasterKeyPair {
        &self.master_keypair
    }

    /// Get the crypto metadata
    pub fn metadata(&self) -> &CryptoMetadata {
        &self.metadata
    }
}

/// Initialize cryptography for a new database based on the provided configuration
///
/// This function performs the complete crypto initialization workflow:
/// 1. Validates the crypto configuration
/// 2. Generates or derives the master key pair based on configuration
/// 3. Creates crypto metadata
/// 4. Stores the metadata in the database
///
/// # Arguments
/// * `db_ops` - Database operations interface
/// * `crypto_config` - Crypto configuration specifying how to initialize encryption
///
/// # Returns
/// * `Ok(CryptoInitContext)` - Crypto initialization context with keys and metadata
/// * `Err(CryptoInitError)` - Error if initialization fails
pub fn initialize_database_crypto(
    db_ops: Arc<DbOperations>,
    crypto_config: &CryptoConfig,
) -> CryptoInitResult<CryptoInitContext> {
    info!("Starting database crypto initialization");

    // Check if crypto is already initialized
    if db_ops.has_crypto_metadata()? {
        warn!("Database already has crypto metadata - crypto initialization skipped");
        return Err(CryptoInitError::AlreadyInitialized);
    }

    // Validate crypto configuration
    crypto_config.validate()?;
    debug!("Crypto configuration validated successfully");

    if !crypto_config.enabled {
        info!("Crypto is disabled in configuration - skipping initialization");
        return Err(CryptoInitError::InvalidConfig(
            "Crypto is disabled in configuration".to_string(),
        ));
    }

    // Generate or derive master key pair based on configuration
    let (master_keypair, derivation_method) = generate_master_keypair_from_config(crypto_config)?;
    info!(
        "Master key pair generated using method: {}",
        derivation_method
    );

    // Create crypto metadata
    let metadata = CryptoMetadata::new(
        master_keypair.public_key().clone(),
        derivation_method.clone(),
    )?;
    debug!(
        "Crypto metadata created with checksum: {}",
        metadata.checksum
    );

    // Store crypto metadata in database
    db_ops.store_crypto_metadata(&metadata)?;
    info!("Crypto metadata stored successfully in database");

    // Verify storage by reading back
    let stored_metadata = db_ops.get_crypto_metadata()?.ok_or_else(|| {
        CryptoInitError::Database(SchemaError::InvalidData(
            "Failed to retrieve stored crypto metadata".to_string(),
        ))
    })?;

    // Verify integrity
    if !stored_metadata.verify_integrity()? {
        return Err(CryptoInitError::Database(SchemaError::InvalidData(
            "Stored crypto metadata failed integrity check".to_string(),
        )));
    }

    info!("Crypto initialization completed successfully");

    Ok(CryptoInitContext {
        master_keypair,
        metadata,
        derivation_method,
    })
}

/// Generate master key pair based on crypto configuration
fn generate_master_keypair_from_config(
    crypto_config: &CryptoConfig,
) -> CryptoInitResult<(MasterKeyPair, String)> {
    match &crypto_config.master_key {
        MasterKeyConfig::Random => {
            debug!("Generating random master key pair");
            let keypair = generate_master_keypair()?;
            Ok((keypair, "Random".to_string()))
        }

        MasterKeyConfig::Passphrase { passphrase } => {
            debug!("Deriving master key pair from passphrase");
            let salt = generate_salt();
            let params = crypto_config.key_derivation.to_argon2_params()?;
            let keypair = derive_master_keypair(passphrase, &salt, &params)?;

            let method = if let Some(preset) = &crypto_config.key_derivation.preset {
                // Check if parameters were customized from preset defaults
                let preset_config = KeyDerivationConfig::for_security_level(*preset);
                if crypto_config.key_derivation.memory_cost != preset_config.memory_cost
                    || crypto_config.key_derivation.time_cost != preset_config.time_cost
                    || crypto_config.key_derivation.parallelism != preset_config.parallelism
                {
                    "Argon2id-Custom".to_string()
                } else {
                    format!("Argon2id-{}", preset.as_str())
                }
            } else {
                "Argon2id-Custom".to_string()
            };
            Ok((keypair, method))
        }

        MasterKeyConfig::External { .. } => {
            // For now, external key sources are not implemented
            Err(CryptoInitError::InvalidConfig(
                "External key sources are not yet implemented".to_string(),
            ))
        }
    }
}

/// Check if database crypto initialization is needed
///
/// This function checks whether a database needs crypto initialization
/// based on the configuration and current database state.
///
/// # Arguments
/// * `db_ops` - Database operations interface
/// * `crypto_config` - Optional crypto configuration
///
/// # Returns
/// * `Ok(true)` - Crypto initialization is needed
/// * `Ok(false)` - Crypto initialization is not needed or already done
/// * `Err(CryptoInitError)` - Error checking crypto status
pub fn is_crypto_init_needed(
    db_ops: Arc<DbOperations>,
    crypto_config: Option<&CryptoConfig>,
) -> CryptoInitResult<bool> {
    // If no crypto config provided, no initialization needed
    let crypto_config = match crypto_config {
        Some(config) => config,
        None => {
            debug!("No crypto configuration provided - initialization not needed");
            return Ok(false);
        }
    };

    // If crypto is disabled in config, no initialization needed
    if !crypto_config.enabled {
        debug!("Crypto disabled in configuration - initialization not needed");
        return Ok(false);
    }

    // Check if crypto metadata already exists
    let has_crypto = db_ops.has_crypto_metadata()?;

    if has_crypto {
        debug!("Database already has crypto metadata - initialization not needed");
        Ok(false)
    } else {
        debug!("Database needs crypto initialization");
        Ok(true)
    }
}

/// Validate crypto configuration before database initialization
///
/// This function performs comprehensive validation of crypto configuration
/// to ensure it's suitable for database initialization.
///
/// # Arguments
/// * `crypto_config` - Crypto configuration to validate
///
/// # Returns
/// * `Ok(())` - Configuration is valid
/// * `Err(CryptoInitError)` - Configuration validation failed
pub fn validate_crypto_config_for_init(crypto_config: &CryptoConfig) -> CryptoInitResult<()> {
    debug!("Validating crypto configuration for database initialization");

    // Basic configuration validation
    crypto_config.validate()?;

    // Additional validation for initialization context
    match &crypto_config.master_key {
        MasterKeyConfig::Random => {
            debug!("Random key generation - no additional validation needed");
            Ok(())
        }

        MasterKeyConfig::Passphrase { passphrase } => {
            debug!("Validating passphrase configuration");

            if passphrase.is_empty() {
                return Err(CryptoInitError::InvalidConfig(
                    "Passphrase cannot be empty".to_string(),
                ));
            }

            if passphrase.len() < 8 {
                return Err(CryptoInitError::InvalidConfig(
                    "Passphrase must be at least 8 characters long".to_string(),
                ));
            }

            if passphrase.len() > 1024 {
                return Err(CryptoInitError::InvalidConfig(
                    "Passphrase is too long (max 1024 characters)".to_string(),
                ));
            }

            // Check for common weak passphrases
            let weak_passphrases = [
                "password",
                "123456",
                "12345678",
                "qwerty",
                "abc123",
                "password123",
                "admin",
                "letmein",
                "welcome",
                "monkey",
            ];

            if weak_passphrases
                .iter()
                .any(|&weak| passphrase.eq_ignore_ascii_case(weak))
            {
                return Err(CryptoInitError::InvalidConfig(
                    "Passphrase is too weak - please choose a stronger passphrase".to_string(),
                ));
            }

            // Validate key derivation parameters
            let _params = crypto_config.key_derivation.to_argon2_params()?;

            debug!("Passphrase configuration validated successfully");
            Ok(())
        }

        MasterKeyConfig::External { .. } => Err(CryptoInitError::InvalidConfig(
            "External key sources are not yet supported for database initialization".to_string(),
        )),
    }
}

/// Get crypto initialization statistics and status
///
/// This function provides detailed information about the crypto initialization
/// status of a database for debugging and monitoring purposes.
///
/// # Arguments
/// * `db_ops` - Database operations interface
///
/// # Returns
/// * `Ok(CryptoInitStatus)` - Crypto initialization status information
/// * `Err(CryptoInitError)` - Error getting crypto status
pub fn get_crypto_init_status(db_ops: Arc<DbOperations>) -> CryptoInitResult<CryptoInitStatus> {
    let has_crypto = db_ops.has_crypto_metadata()?;

    if !has_crypto {
        return Ok(CryptoInitStatus {
            initialized: false,
            version: None,
            algorithm: None,
            derivation_method: None,
            created_at: None,
            integrity_verified: None,
        });
    }

    let metadata = db_ops.get_crypto_metadata()?.ok_or_else(|| {
        CryptoInitError::Database(SchemaError::InvalidData(
            "Crypto metadata exists but cannot be retrieved".to_string(),
        ))
    })?;

    let integrity_verified = metadata.verify_integrity().unwrap_or(false);

    Ok(CryptoInitStatus {
        initialized: true,
        version: Some(metadata.version),
        algorithm: Some(metadata.signature_algorithm),
        derivation_method: Some(metadata.key_derivation_method),
        created_at: Some(metadata.created_at),
        integrity_verified: Some(integrity_verified),
    })
}

/// Status information about crypto initialization
#[derive(Debug, Clone)]
pub struct CryptoInitStatus {
    /// Whether crypto has been initialized
    pub initialized: bool,
    /// Crypto metadata version (if initialized)
    pub version: Option<u32>,
    /// Signature algorithm used (if initialized)
    pub algorithm: Option<String>,
    /// Key derivation method used (if initialized)
    pub derivation_method: Option<String>,
    /// When crypto was initialized (if initialized)
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Whether crypto metadata integrity is verified (if initialized)
    pub integrity_verified: Option<bool>,
}

impl CryptoInitStatus {
    /// Check if crypto is properly initialized and verified
    pub fn is_healthy(&self) -> bool {
        self.initialized && self.integrity_verified.unwrap_or(false)
    }

    /// Get a human-readable status summary
    pub fn summary(&self) -> String {
        if !self.initialized {
            "Not initialized".to_string()
        } else if self.is_healthy() {
            format!(
                "Initialized ({})",
                self.derivation_method.as_deref().unwrap_or("Unknown")
            )
        } else {
            "Initialized but integrity check failed".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::crypto::{KeyDerivationConfig, SecurityLevel};
    use tempfile::tempdir;

    fn create_test_db_ops() -> Arc<DbOperations> {
        let temp_dir = tempdir().unwrap();
        let db = sled::open(temp_dir.path()).unwrap();
        Arc::new(DbOperations::new(db).unwrap())
    }

    fn create_random_crypto_config() -> CryptoConfig {
        CryptoConfig {
            enabled: true,
            master_key: MasterKeyConfig::Random,
            key_derivation: KeyDerivationConfig::default(),
        }
    }

    fn create_passphrase_crypto_config() -> CryptoConfig {
        CryptoConfig {
            enabled: true,
            master_key: MasterKeyConfig::Passphrase {
                passphrase: "test_passphrase_for_crypto_init".to_string(),
            },
            key_derivation: KeyDerivationConfig::for_security_level(SecurityLevel::Interactive),
        }
    }

    #[test]
    fn test_crypto_init_with_random_key() {
        let db_ops = create_test_db_ops();
        let config = create_random_crypto_config();

        let context = initialize_database_crypto(db_ops.clone(), &config).unwrap();

        assert_eq!(context.derivation_method, "Random");
        assert!(context.metadata.verify_integrity().unwrap());

        // Verify stored in database
        assert!(db_ops.has_crypto_metadata().unwrap());
        let stored_key = db_ops.get_master_public_key().unwrap().unwrap();
        assert_eq!(stored_key.to_bytes(), context.public_key().to_bytes());
    }

    #[test]
    fn test_crypto_init_with_passphrase() {
        let db_ops = create_test_db_ops();
        let config = create_passphrase_crypto_config();

        let context = initialize_database_crypto(db_ops.clone(), &config).unwrap();

        assert!(context.derivation_method.starts_with("Argon2id-"));
        assert!(context.metadata.verify_integrity().unwrap());

        // Verify stored in database
        assert!(db_ops.has_crypto_metadata().unwrap());
    }

    #[test]
    fn test_crypto_init_disabled_config() {
        let db_ops = create_test_db_ops();
        let mut config = create_random_crypto_config();
        config.enabled = false;

        let result = initialize_database_crypto(db_ops, &config);
        assert!(matches!(result, Err(CryptoInitError::InvalidConfig(_))));
    }

    #[test]
    fn test_crypto_init_already_initialized() {
        let db_ops = create_test_db_ops();
        let config = create_random_crypto_config();

        // Initialize once
        initialize_database_crypto(db_ops.clone(), &config).unwrap();

        // Try to initialize again
        let result = initialize_database_crypto(db_ops, &config);
        assert!(matches!(result, Err(CryptoInitError::AlreadyInitialized)));
    }

    #[test]
    fn test_is_crypto_init_needed() {
        let db_ops = create_test_db_ops();
        let config = create_random_crypto_config();

        // Initially needed
        assert!(is_crypto_init_needed(db_ops.clone(), Some(&config)).unwrap());

        // After initialization, not needed
        initialize_database_crypto(db_ops.clone(), &config).unwrap();
        assert!(!is_crypto_init_needed(db_ops, Some(&config)).unwrap());
    }

    #[test]
    fn test_validate_crypto_config_for_init() {
        // Valid random config
        let config = create_random_crypto_config();
        assert!(validate_crypto_config_for_init(&config).is_ok());

        // Valid passphrase config
        let config = create_passphrase_crypto_config();
        assert!(validate_crypto_config_for_init(&config).is_ok());

        // Invalid empty passphrase
        let config = CryptoConfig {
            enabled: true,
            master_key: MasterKeyConfig::Passphrase {
                passphrase: "".to_string(),
            },
            key_derivation: KeyDerivationConfig::default(),
        };
        assert!(validate_crypto_config_for_init(&config).is_err());
    }

    #[test]
    fn test_crypto_init_status() {
        let db_ops = create_test_db_ops();

        // No crypto initially
        let status = get_crypto_init_status(db_ops.clone()).unwrap();
        assert!(!status.initialized);
        assert!(!status.is_healthy());
        assert_eq!(status.summary(), "Not initialized");

        // After initialization
        let config = create_random_crypto_config();
        initialize_database_crypto(db_ops.clone(), &config).unwrap();

        let status = get_crypto_init_status(db_ops).unwrap();
        assert!(status.initialized);
        assert!(status.is_healthy());
        assert!(status.summary().contains("Random"));
    }
}
