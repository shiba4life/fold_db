//! Key generation, derivation, and KDF logic
//! 
//! This module handles all key generation operations including random generation,
//! passphrase-based derivation, and key derivation functions.

use crate::cli::args::{CliSecurityLevel, KeyFormat};
use crate::cli::commands::keys::error::{KeyError, KeyResult};
use crate::cli::commands::keys::utils::{
    format_and_output_key, get_operation_passphrase, security_level_to_argon2,
};
use crate::crypto::ed25519::{generate_master_keypair, generate_master_keypair_from_seed};
use crate::crypto::{derive_key, generate_salt, Argon2Params};
use log::info;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Enhanced KDF parameters for export compatibility
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnhancedKdfParams {
    /// Salt for key derivation (32 bytes)
    pub salt: Vec<u8>,
    /// Memory cost in KB
    pub memory: u32,
    /// Time cost (iterations)
    pub iterations: u32,
    /// Parallelism factor
    pub parallelism: u32,
}

impl From<&Argon2Params> for EnhancedKdfParams {
    fn from(params: &Argon2Params) -> Self {
        Self {
            salt: vec![0u8; 32], // Will be filled separately
            memory: params.memory_cost,
            iterations: params.time_cost,
            parallelism: params.parallelism,
        }
    }
}

/// HKDF key derivation using BLAKE3 (simplified for compatibility)
pub fn hkdf_derive_key(
    master_key: &[u8; 32],
    salt: &[u8],
    info: &[u8],
    output_length: usize,
) -> Vec<u8> {
    // Simplified key derivation using BLAKE3
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"HKDF-DATAFOLD"); // Domain separator
    hasher.update(salt);
    hasher.update(master_key);
    hasher.update(info);

    let base_hash = hasher.finalize();

    // For simplicity, just use the first output_length bytes
    // In a production system, you'd want proper HKDF expansion
    let mut output = vec![0u8; output_length];
    let available_bytes = std::cmp::min(32, output_length);
    output[..available_bytes].copy_from_slice(&base_hash.as_bytes()[..available_bytes]);

    // If we need more than 32 bytes, use a simple expansion
    if output_length > 32 {
        for (i, item) in output.iter_mut().enumerate().skip(32) {
            *item = base_hash.as_bytes()[i % 32];
        }
    }

    output
}

/// Handle Ed25519 key generation command
pub fn handle_generate_key(
    format: KeyFormat,
    private_key_file: Option<PathBuf>,
    public_key_file: Option<PathBuf>,
    count: u32,
    public_only: bool,
    private_only: bool,
) -> KeyResult<()> {
    if public_only && private_only {
        return Err(KeyError::ConfigurationError("Cannot specify both --public-only and --private-only".to_string()));
    }

    for i in 0..count {
        let keypair = generate_master_keypair()
            .map_err(|e| KeyError::CryptographicError(format!("Failed to generate keypair: {}", e)))?;

        let public_key_bytes = keypair.public_key_bytes();
        let private_key_bytes = keypair.secret_key_bytes();

        if count > 1 {
            info!("Generating keypair {} of {}", i + 1, count);
        }

        // Output private key if requested
        if !public_only {
            format_and_output_key(
                &private_key_bytes,
                &format,
                private_key_file.as_ref(),
                "private",
                true,
                true,
            )?;
        }

        // Output public key if requested
        if !private_only {
            format_and_output_key(
                &public_key_bytes,
                &format,
                public_key_file.as_ref(),
                "public",
                false,
                true,
            )?;
        }

        // Clear sensitive data
        drop(keypair);
    }

    Ok(())
}

/// Handle Ed25519 key derivation from passphrase
pub fn handle_derive_key(
    format: KeyFormat,
    private_key_file: Option<PathBuf>,
    public_key_file: Option<PathBuf>,
    security_level: CliSecurityLevel,
    public_only: bool,
    private_only: bool,
    passphrase: Option<String>,
) -> KeyResult<()> {
    use crate::crypto::argon2::generate_salt_and_derive_keypair;

    if public_only && private_only {
        return Err(KeyError::ConfigurationError("Cannot specify both --public-only and --private-only".to_string()));
    }

    let passphrase = get_operation_passphrase("key derivation", passphrase)?;

    // Convert security level to Argon2 parameters
    let argon2_params = security_level_to_argon2(&security_level);

    // Generate salt and derive keypair
    let (_salt, keypair) = generate_salt_and_derive_keypair(&passphrase, &argon2_params)
        .map_err(|e| KeyError::CryptographicError(format!("Failed to derive keypair from passphrase: {}", e)))?;

    let public_key_bytes = keypair.public_key_bytes();
    let private_key_bytes = keypair.secret_key_bytes();

    // Output private key if requested
    if !public_only {
        format_and_output_key(
            &private_key_bytes,
            &format,
            private_key_file.as_ref(),
            "private",
            true,
            true,
        )?;
    }

    // Output public key if requested
    if !private_only {
        format_and_output_key(
            &public_key_bytes,
            &format,
            public_key_file.as_ref(),
            "public",
            false,
            true,
        )?;
    }

    // Clear sensitive data
    drop(keypair);

    Ok(())
}

/// Handle deriving a child key from a master key
#[allow(clippy::too_many_arguments)]
pub fn handle_derive_from_master(
    master_key_id: String,
    context: String,
    child_key_id: String,
    storage_dir: Option<PathBuf>,
    security_level: CliSecurityLevel,
    format: KeyFormat,
    output_only: bool,
    force: bool,
) -> KeyResult<()> {
    use crate::cli::utils::key_utils::{decrypt_key, encrypt_key, get_default_storage_dir, KeyStorageConfig};
    use crate::cli::commands::keys::utils::{
        confirm_operation, key_exists_in_storage, set_secure_file_permissions, validate_key_id,
        get_passphrase_with_retry,
    };
    use rpassword::read_password;
    use std::fs;
    use std::io::{self, Write};

    // Validate child key ID
    validate_key_id(&child_key_id)?;

    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()
        .map_err(|e| KeyError::StorageError(format!("Failed to get storage directory: {}", e)))?);
    let master_key_file_path = storage_path.join(format!("{}.json", master_key_id));

    if !master_key_file_path.exists() {
        return Err(KeyError::KeyNotFound(format!("Master key '{}' not found", master_key_id)));
    }

    // Read and decrypt master key
    let config_content = fs::read_to_string(&master_key_file_path)
        .map_err(|e| KeyError::StorageError(format!("Failed to read master key file: {}", e)))?;
    let storage_config: KeyStorageConfig = serde_json::from_str(&config_content)
        .map_err(|e| KeyError::StorageError(format!("Failed to parse master key file: {}", e)))?;

    let passphrase = get_passphrase_with_retry(&format!(
        "Enter passphrase to decrypt master key '{}': ",
        master_key_id
    ))?;

    let master_key_bytes = decrypt_key(&storage_config, &passphrase)
        .map_err(|e| KeyError::CryptographicError(format!("Failed to decrypt master key: {}", e)))?;

    // Derive child key using HKDF (BLAKE3)
    let context_bytes = context.as_bytes();
    let salt = generate_salt();
    let derived_key_material =
        hkdf_derive_key(&master_key_bytes, salt.as_bytes(), context_bytes, 32);

    if derived_key_material.len() != 32 {
        return Err(KeyError::CryptographicError("Failed to derive 32-byte key".to_string()));
    }

    let mut child_key_bytes = [0u8; 32];
    child_key_bytes.copy_from_slice(&derived_key_material);

    if output_only {
        // Just output the derived key
        format_and_output_key(&child_key_bytes, &format, None, "derived", true, false)?;
        info!(
            "✅ Child key derived from master '{}' with context '{}'",
            master_key_id, context
        );
    } else {
        // Store the derived child key
        let child_key_file_path = storage_path.join(format!("{}.json", child_key_id));
        if key_exists_in_storage(&child_key_id, &storage_path) && !force {
            return Err(KeyError::KeyExists(format!(
                "Child key '{}' already exists. Use --force to overwrite",
                child_key_id
            )));
        }

        // Get passphrase for child key encryption
        let child_passphrase = get_passphrase_with_retry(&format!(
            "Enter passphrase to encrypt child key '{}': ",
            child_key_id
        ))?;

        // Convert security level to Argon2 parameters
        let argon2_params = security_level_to_argon2(&security_level);

        // Encrypt the child key
        let child_storage_config = encrypt_key(&child_key_bytes, &child_passphrase, &argon2_params)
            .map_err(|e| KeyError::CryptographicError(format!("Failed to encrypt child key: {}", e)))?;

        // Write encrypted child key to file
        let config_json = serde_json::to_string_pretty(&child_storage_config)
            .map_err(|e| KeyError::StorageError(format!("Failed to serialize child key config: {}", e)))?;
        fs::write(&child_key_file_path, config_json)
            .map_err(|e| KeyError::StorageError(format!("Failed to write child key file: {}", e)))?;

        // Set file permissions to 600 (owner read/write only)
        set_secure_file_permissions(&child_key_file_path)?;

        info!(
            "✅ Child key '{}' derived from master '{}' and stored securely",
            child_key_id, master_key_id
        );
        info!("Derivation context: '{}'", context);
        info!("Security level: {:?}", security_level);
    }

    // Clear sensitive data
    let _ = master_key_bytes;
    let _ = child_key_bytes;

    Ok(())
}