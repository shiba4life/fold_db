//! Error types for cryptographic operations

use thiserror::Error;

/// Result type alias for crypto operations
pub type CryptoResult<T> = Result<T, CryptoError>;

/// Errors that can occur during cryptographic operations
#[derive(Error, Debug)]
pub enum CryptoError {
    /// Error during key generation
    #[error("Failed to generate cryptographic key: {message}")]
    KeyGeneration { message: String },

    /// Error during key serialization
    #[error("Failed to serialize key: {message}")]
    Serialization { message: String },

    /// Error during key deserialization
    #[error("Failed to deserialize key: {message}")]
    Deserialization { message: String },

    /// Invalid key material provided
    #[error("Invalid key material: {message}")]
    InvalidKey { message: String },

    /// Error during signature operations
    #[error("Signature operation failed: {message}")]
    Signature { message: String },

    /// Error during key derivation
    #[error("Key derivation failed: {message}")]
    KeyDerivation { message: String },

    /// Random number generation error
    #[error("Random number generation failed: {message}")]
    RandomGeneration { message: String },

    /// Ed25519 signature verification error
    #[error("Ed25519 signature verification failed")]
    SignatureVerification,

    /// Invalid signature format
    #[error("Invalid signature format: {message}")]
    InvalidSignature { message: String },

    /// Invalid input provided
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl From<ed25519_dalek::SignatureError> for CryptoError {
    fn from(_err: ed25519_dalek::SignatureError) -> Self {
        CryptoError::SignatureVerification
    }
}

impl From<argon2::Error> for CryptoError {
    fn from(err: argon2::Error) -> Self {
        CryptoError::KeyDerivation {
            message: err.to_string(),
        }
    }
}
