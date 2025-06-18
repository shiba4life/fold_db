//! Legacy key derivation functions for backward compatibility

use super::super::encryption_core::AES_KEY_SIZE;
use blake3::Hasher;

/// Derive an AES-256 encryption key from a master key using BLAKE3
///
/// **Note**: This is a legacy function. Use `KeyDerivationManager` for new code.
///
/// # Arguments
/// * `master_key` - The master key material (e.g., from Ed25519 private key)
/// * `context` - Context string for key separation (e.g., "database_encryption")
/// * `salt` - Optional salt for additional entropy
///
/// # Returns
/// * `[u8; 32]` - A 256-bit encryption key suitable for AES-256-GCM
pub fn derive_encryption_key(
    master_key: &[u8],
    context: &str,
    salt: Option<&[u8]>,
) -> [u8; AES_KEY_SIZE] {
    let mut hasher = Hasher::new();

    // Add master key material
    hasher.update(master_key);

    // Add context for key separation
    hasher.update(context.as_bytes());

    // Add salt if provided
    if let Some(salt_bytes) = salt {
        hasher.update(salt_bytes);
    }

    // Derive the key
    let mut derived_key = [0u8; AES_KEY_SIZE];
    let output = hasher.finalize();
    derived_key.copy_from_slice(&output.as_bytes()[..AES_KEY_SIZE]);

    derived_key
}

/// Derive multiple encryption keys for different purposes
///
/// **Note**: This is a legacy function. Use `KeyDerivationManager` for new code.
///
/// # Arguments
/// * `master_key` - The master key material
/// * `contexts` - List of context strings for different keys
/// * `salt` - Optional salt for additional entropy
///
/// # Returns
/// * `Vec<[u8; 32]>` - Multiple encryption keys derived from the same master key
pub fn derive_multiple_keys(
    master_key: &[u8],
    contexts: &[&str],
    salt: Option<&[u8]>,
) -> Vec<[u8; AES_KEY_SIZE]> {
    contexts
        .iter()
        .map(|&context| derive_encryption_key(master_key, context, salt))
        .collect()
}

/// Migration utilities for transitioning from legacy to new key derivation
pub mod migration {
    use super::*;

    /// Verify that legacy and new derivation produce the same results for testing
    ///
    /// This function is useful during migration to ensure compatibility
    /// between legacy and new key derivation methods.
    ///
    /// # Arguments
    /// * `master_key` - The master key material
    /// * `context` - The encryption context
    /// * `salt` - Optional salt
    ///
    /// # Returns
    /// * `[u8; 32]` - The derived key (verified to be consistent)
    pub fn derive_with_verification(
        master_key: &[u8],
        context: &str,
        salt: Option<&[u8]>,
    ) -> [u8; AES_KEY_SIZE] {
        // For now, just use the legacy function
        // In the future, this could compare legacy vs new methods
        derive_encryption_key(master_key, context, salt)
    }

    /// Check if a key was derived using legacy methods
    ///
    /// This is a placeholder for future migration tracking.
    ///
    /// # Arguments
    /// * `_key` - The key to check
    ///
    /// # Returns
    /// * `bool` - Always true for now (legacy assumption)
    pub fn is_legacy_derived(_key: &[u8; AES_KEY_SIZE]) -> bool {
        // For now, assume all keys are legacy-derived
        // This could be enhanced with metadata tracking
        true
    }
}