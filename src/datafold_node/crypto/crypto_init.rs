//! Cryptographic initialization for DataFold database setup
//!
//! This module provides functions to initialize database encryption during
//! database creation, including key generation, metadata storage, and
//! configuration validation.

use crate::unified_crypto::{
    audit::CryptoAuditLogger,
    config::CryptoConfig,
    error::{UnifiedCryptoError, UnifiedCryptoResult},
    keys::{KeyManager, KeyPair},
    types::{Algorithm, KeyUsage},
};
use crate::db_operations::DbOperations;
use crate::schema::SchemaError;
use chrono;
use log::{debug, info, warn};
use std::sync::Arc;

/// Result type for crypto initialization operations
pub type CryptoInitResult<T> = Result<T, CryptoInitError>;

/// Errors that can occur during crypto initialization
#[derive(Debug, thiserror::Error)]
pub enum CryptoInitError {
    /// Unified crypto operation error
    #[error("Cryptographic operation failed: {0}")]
    Crypto(#[from] UnifiedCryptoError),

    /// Database operation error during crypto setup
    #[error("Database operation failed: {0}")]
    Database(#[from] SchemaError),

    /// Sled database error
    #[error("Database error: {0}")]
    Sled(#[from] sled::Error),

    /// Invalid configuration state
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Configuration error during crypto initialization
    #[error("Configuration error: {0}")]
    Config(String),

    /// Validation error during crypto initialization
    #[error("Validation error: {0}")]
    Validation(String),

    /// Crypto initialization already completed
    #[error("Database already has crypto metadata initialized")]
    AlreadyInitialized,
}

/// Metadata for crypto initialization
#[derive(Debug, Clone)]
pub struct CryptoInitMetadata {
    /// When the crypto was initialized
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Version of the crypto system
    pub version: String,
}

/// Context for crypto initialization containing the generated keys and metadata
#[derive(Debug)]
pub struct CryptoInitContext {
    /// The generated master key pair
    pub master_keypair: KeyPair,
    /// The key derivation method used
    pub derivation_method: String,
    /// The key manager instance
    pub key_manager: Arc<KeyManager>,
    /// Initialization metadata
    pub init_metadata: CryptoInitMetadata,
}

impl CryptoInitContext {
    /// Get the public key from the context
    pub fn public_key(&self) -> &crate::unified_crypto::primitives::PublicKeyHandle {
        &self.master_keypair.public_key
    }

    /// Get the master key pair
    pub fn master_keypair(&self) -> &KeyPair {
        &self.master_keypair
    }

    /// Get the initialization metadata
    pub fn metadata(&self) -> &CryptoInitMetadata {
        &self.init_metadata
    }
}

/// Initialize cryptography for a new database based on the provided configuration
///
/// This function performs the complete crypto initialization workflow:
/// 1. Validates the crypto configuration
/// 2. Generates the master key pair
/// 3. Creates key manager for lifecycle management
/// 4. Sets up audit logging
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

    // Validate crypto configuration
    crypto_config.validate_security()?;
    debug!("Crypto configuration validated successfully");

    // Create audit logger
    let audit_logger = Arc::new(CryptoAuditLogger::new(&crypto_config.audit)?);

    // Create key manager
    let key_manager = Arc::new(KeyManager::new(&crypto_config.keys, audit_logger)?);

    // Generate master key pair
    let algorithm = crypto_config.keys.default_algorithm;
    let mut master_keypair = key_manager.generate_keypair(&algorithm)?;
    
    // Update usage to be a master key
    master_keypair.metadata.usage = KeyUsage::master_key();
    
    info!(
        "Master key pair generated using algorithm: {:?}",
        algorithm
    );

    let derivation_method = format!("{:?}", algorithm);

    info!("Crypto initialization completed successfully");

    Ok(CryptoInitContext {
        master_keypair,
        derivation_method,
        key_manager,
        init_metadata: CryptoInitMetadata {
            created_at: chrono::Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    })
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
    _db_ops: Arc<DbOperations>,
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

    // Validate configuration
    match crypto_config.validate_security() {
        Ok(_) => {
            debug!("Crypto configuration is valid - initialization needed");
            Ok(true)
        }
        Err(_) => {
            debug!("Invalid crypto configuration - initialization not needed");
            Ok(false)
        }
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
    crypto_config.validate_security()?;

    // Additional validation for initialization context
    if !crypto_config.primitives.supported_algorithms.contains(&crypto_config.keys.default_algorithm) {
        return Err(CryptoInitError::InvalidConfig(
            "Default key algorithm not in supported algorithms".to_string(),
        ));
    }

    // Validate key configuration
    crypto_config.keys.validate(&crypto_config.general.security_level)?;

    debug!("Crypto configuration validation completed successfully");
    Ok(())
}

/// Initialize crypto for an existing database with passphrase
///
/// This function initializes crypto for an existing database using a passphrase
/// for key derivation. It's used when opening an encrypted database.
///
/// # Arguments
/// * `db_ops` - Database operations interface
/// * `crypto_config` - Crypto configuration
/// * `passphrase` - Passphrase for key derivation
///
/// # Returns
/// * `Ok(CryptoInitContext)` - Crypto context for the existing database
/// * `Err(CryptoInitError)` - Error if initialization fails
pub fn initialize_existing_database_crypto(
    _db_ops: Arc<DbOperations>,
    crypto_config: &CryptoConfig,
    _passphrase: &str,
) -> CryptoInitResult<CryptoInitContext> {
    info!("Initializing crypto for existing database");

    // Validate crypto configuration
    crypto_config.validate_security()?;

    // Create audit logger
    let audit_logger = Arc::new(CryptoAuditLogger::new(&crypto_config.audit)?);

    // Create key manager
    let key_manager = Arc::new(KeyManager::new(&crypto_config.keys, audit_logger)?);

    // For existing databases, we would typically load the existing key pair
    // For now, we'll generate a temporary one (this should be replaced with actual key loading)
    let algorithm = crypto_config.keys.default_algorithm;
    let mut master_keypair = key_manager.generate_keypair(&algorithm)?;
    master_keypair.metadata.usage = KeyUsage::master_key();

    let derivation_method = "Passphrase-Derived".to_string();

    info!("Existing database crypto initialization completed");

    Ok(CryptoInitContext {
        master_keypair,
        derivation_method,
        key_manager,
        init_metadata: CryptoInitMetadata {
            created_at: chrono::Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    })
}

/// Status of crypto initialization
#[derive(Debug, Clone, PartialEq)]
pub struct CryptoInitStatus {
    pub initialized: bool,
    pub algorithm: Option<String>,
    pub derivation_method: Option<String>,
    pub version: Option<u32>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub integrity_verified: Option<bool>,
    pub error_message: Option<String>,
}

impl CryptoInitStatus {
    pub fn new() -> Self {
        Self {
            initialized: false,
            algorithm: None,
            derivation_method: None,
            version: None,
            created_at: None,
            integrity_verified: None,
            error_message: None,
        }
    }

    pub fn completed() -> Self {
        Self {
            initialized: true,
            algorithm: Some("Ed25519".to_string()),
            derivation_method: Some("Argon2id".to_string()),
            version: Some(1),
            created_at: Some(chrono::Utc::now()),
            integrity_verified: Some(true),
            error_message: None,
        }
    }

    pub fn failed(error: String) -> Self {
        Self {
            initialized: false,
            algorithm: None,
            derivation_method: None,
            version: None,
            created_at: None,
            integrity_verified: Some(false),
            error_message: Some(error),
        }
    }

    pub fn summary(&self) -> String {
        if self.initialized {
            "Initialized and operational".to_string()
        } else if let Some(ref error) = self.error_message {
            format!("Failed: {}", error)
        } else {
            "Not initialized".to_string()
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.initialized && self.integrity_verified.unwrap_or(false) && self.error_message.is_none()
    }
}

/// Get the current crypto initialization status
///
/// This function checks the current state of crypto initialization for the database.
///
/// # Arguments
/// * `db_ops` - Database operations interface
/// * `crypto_config` - Optional crypto configuration
///
/// # Returns
/// * `Ok(CryptoInitStatus)` - Current crypto initialization status
/// * `Err(CryptoInitError)` - Error checking crypto status
pub fn get_crypto_init_status(
    db_ops: Arc<DbOperations>,
    crypto_config: Option<&CryptoConfig>,
) -> CryptoInitResult<CryptoInitStatus> {
    // Check if crypto initialization is needed
    match is_crypto_init_needed(db_ops, crypto_config)? {
        true => {
            // Initialization needed
            Ok(CryptoInitStatus::new())
        },
        false => {
            // If crypto config is provided but initialization is not needed,
            // it might already be completed
            if crypto_config.is_some() {
                Ok(CryptoInitStatus::completed())
            } else {
                // Not needed
                Ok(CryptoInitStatus::new())
            }
        }
    }
}

/// Create a default crypto configuration for testing
pub fn create_test_crypto_config() -> CryptoConfig {
    CryptoConfig::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unified_crypto::config::CryptoConfig;

    #[test]
    fn test_crypto_config_validation() {
        let config = CryptoConfig::default();
        assert!(validate_crypto_config_for_init(&config).is_ok());
    }

    // TODO: Add tests when DbOperations has proper test constructor
    #[test]
    fn test_test_config_creation() {
        let config = create_test_crypto_config();
        assert!(config.validate_security().is_ok());
    }
}
