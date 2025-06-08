//! # Cryptographic Utilities for DataFold
//!
//! This module provides cryptographic functionality for DataFold's database master key encryption
//! system. It includes secure key generation, management, and serialization for Ed25519 keys.
//!
//! ## Security Features
//!
//! * Secure Ed25519 key pair generation using cryptographically secure random number generation
//! * Automatic memory zeroization for sensitive key material
//! * Safe serialization and deserialization with proper error handling
//! * Integration with Argon2 for passphrase-based key derivation
//!
//! ## Example Usage
//!
//! ```rust
//! use datafold::crypto::ed25519::{generate_master_keypair, MasterKeyPair};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Generate a new master key pair for database encryption
//! let master_keys = generate_master_keypair()?;
//!
//! // Extract public key for storage
//! let public_key_bytes = master_keys.public_key_bytes();
//!
//! // Sign database operations
//! let signature = master_keys.sign_data(b"database-operation-data")?;
//! # Ok(())
//! # }
//! ```

pub mod argon2;
pub mod ed25519;
pub mod error;

// Re-export commonly used types
pub use argon2::{
    generate_salt, derive_key, derive_master_keypair, derive_master_keypair_default,
    generate_salt_and_derive_keypair, Salt, Argon2Params, DerivedKey
};
pub use ed25519::{generate_master_keypair, MasterKeyPair, PublicKey};
pub use error::{CryptoError, CryptoResult}; 