//! Cryptographic functionality for DataFold node

pub mod crypto_init;
pub mod crypto_routes;
pub mod crypto_validation;
pub mod key_cache_manager;
pub mod key_rotation_compliance;
pub mod key_rotation_routes;
pub mod encryption_core;
pub mod key_derivation;
pub mod encryption_at_rest_async;

pub use crypto_init::{
    CryptoInitError, CryptoInitContext, initialize_database_crypto,
    is_crypto_init_needed, validate_crypto_config_for_init,
    get_crypto_init_status, CryptoInitStatus, CryptoInitResult
};
pub use crypto_validation::{
    validate_crypto_config_comprehensive, validate_for_database_creation,
    validate_crypto_config_quick
};

// Re-exports for backward compatibility with encryption_at_rest module
pub use encryption_core::{
    EncryptionAtRest, EncryptedData, EncryptionKey,
    AES_KEY_SIZE, AES_NONCE_SIZE, AES_TAG_SIZE, MIN_ENCRYPTED_SIZE, MAX_PLAINTEXT_SIZE
};
pub use key_derivation::{KeyDerivationManager, integration};
pub use key_derivation::legacy;
pub use key_derivation::contexts;

// Maintain encryption_at_rest module alias for compatibility
pub mod encryption_at_rest {
    pub use super::encryption_core::*;
    pub use super::key_derivation;
}