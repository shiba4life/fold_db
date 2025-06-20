//! Key storage, retrieval, deletion, and listing operations
//! 
//! This module handles all key storage operations including storing keys securely,
//! retrieving them, deleting them, and listing stored keys.

use crate::cli::args::{CliSecurityLevel, KeyFormat};
use crate::cli::commands::keys::error::{KeyError, KeyResult};
use crate::cli::commands::keys::utils::{
    confirm_operation, format_and_output_key, get_operation_passphrase, get_passphrase_with_retry,
    key_exists_in_storage, security_level_to_argon2, set_secure_file_permissions, validate_key_id,
    validate_private_key_input,
};
use crate::cli::utils::key_utils::{
    decrypt_key, encrypt_key, ensure_storage_dir, get_default_storage_dir, KeyStorageConfig,
};
use crate::crypto::ed25519::generate_master_keypair_from_seed;
use log::info;
use std::fs;
use std::path::PathBuf;

/// Key versioning metadata for rotation tracking
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyVersionMetadata {
    /// Version number (starts at 1)
    pub version: u32,
    /// Previous version number (None for initial version)
    pub previous_version: Option<u32>,
    /// Creation timestamp for this version
    pub created_at: String,
    /// Derivation method used for this version
    pub derivation_method: String,
    /// Salt used for key derivation (32 bytes)
    pub salt: [u8; 32],
    /// Argon2 parameters used for this version
    pub argon2_params: crate::cli::utils::key_utils::StoredArgon2Params,
}

/// Enhanced key storage configuration with versioning
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct VersionedKeyStorageConfig {
    /// Version metadata for tracking
    pub version_metadata: KeyVersionMetadata,
    /// Encrypted key data
    pub encrypted_key: Vec<u8>,
    /// Nonce used for encryption (12 bytes for AES-GCM)
    pub nonce: [u8; 12],
    /// Storage format version
    pub storage_version: u32,
}

/// Handle storing a key securely
pub fn handle_store_key(
    key_id: String,
    private_key: Option<String>,
    private_key_file: Option<PathBuf>,
    storage_dir: Option<PathBuf>,
    force: bool,
    security_level: CliSecurityLevel,
    passphrase: Option<String>,
) -> KeyResult<()> {
    // Validate key ID
    validate_key_id(&key_id)?;

    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()
        .map_err(|e| KeyError::StorageError(format!("Failed to get storage directory: {}", e)))?);
    ensure_storage_dir(&storage_path)
        .map_err(|e| KeyError::StorageError(format!("Failed to ensure storage directory: {}", e)))?;

    // Get private key bytes using utility function
    let private_key_bytes = validate_private_key_input(private_key, private_key_file)?;

    // Check if key already exists
    if key_exists_in_storage(&key_id, &storage_path) && !force {
        return Err(KeyError::KeyExists(format!("Key '{}' already exists. Use --force to overwrite", key_id)));
    }

    // Get passphrase for encryption
    let passphrase = get_operation_passphrase("key storage", passphrase)?;

    // Convert security level to Argon2 parameters
    let argon2_params = security_level_to_argon2(&security_level);

    // Encrypt the key
    let storage_config = encrypt_key(&private_key_bytes, &passphrase, &argon2_params)
        .map_err(|e| KeyError::CryptographicError(format!("Failed to encrypt key: {}", e)))?;

    // Write encrypted key to file
    let key_file_path = storage_path.join(format!("{}.json", key_id));
    let config_json = serde_json::to_string_pretty(&storage_config)?;
    fs::write(&key_file_path, config_json)?;

    // Set file permissions to 600 (owner read/write only)
    set_secure_file_permissions(&key_file_path)?;

    info!(
        "✅ Key '{}' stored securely at: {}",
        key_id,
        key_file_path.display()
    );
    info!("Security level: {:?}", security_level);

    Ok(())
}

/// Handle retrieving a key from storage
pub fn handle_retrieve_key(
    key_id: String,
    storage_dir: Option<PathBuf>,
    format: KeyFormat,
    output_file: Option<PathBuf>,
    public_only: bool,
) -> KeyResult<()> {
    // Validate key ID
    validate_key_id(&key_id)?;

    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()
        .map_err(|e| KeyError::StorageError(format!("Failed to get storage directory: {}", e)))?);
    let key_file_path = storage_path.join(format!("{}.json", key_id));

    if !key_file_path.exists() {
        return Err(KeyError::KeyNotFound(format!("Key '{}' not found", key_id)));
    }

    // Read storage config
    let config_content = fs::read_to_string(&key_file_path)?;
    let storage_config: KeyStorageConfig = serde_json::from_str(&config_content)?;

    // Get passphrase for decryption
    let passphrase = get_passphrase_with_retry("Enter passphrase to decrypt key: ")?;

    // Decrypt the private key
    let private_key_bytes = decrypt_key(&storage_config, &passphrase)
        .map_err(|e| KeyError::CryptographicError(format!("Failed to decrypt key: {}", e)))?;

    if public_only {
        // Extract and output public key only
        let keypair = generate_master_keypair_from_seed(&private_key_bytes)
            .map_err(|e| KeyError::InvalidKey(format!("Failed to generate keypair from stored key: {}", e)))?;
        let public_key_bytes = keypair.public_key_bytes();
        
        // Convert to fixed-size array for compatibility
        if public_key_bytes.len() != 32 {
            return Err(KeyError::InvalidKey("Public key must be 32 bytes".to_string()));
        }
        let mut public_key_array = [0u8; 32];
        public_key_array.copy_from_slice(public_key_bytes);
        
        format_and_output_key(
            &public_key_array,
            &format,
            output_file.as_ref(),
            "public",
            false,
            true,
        )?;
    } else {
        // Output private key
        format_and_output_key(
            &private_key_bytes,
            &format,
            output_file.as_ref(),
            "private",
            true,
            true,
        )?;
    }

    info!("✅ Key '{}' retrieved successfully", key_id);

    Ok(())
}

/// Handle deleting a key from storage
pub fn handle_delete_key(
    key_id: String,
    storage_dir: Option<PathBuf>,
    force: bool,
) -> KeyResult<()> {
    // Validate key ID
    validate_key_id(&key_id)?;

    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()
        .map_err(|e| KeyError::StorageError(format!("Failed to get storage directory: {}", e)))?);
    let key_file_path = storage_path.join(format!("{}.json", key_id));

    if !key_file_path.exists() {
        return Err(KeyError::KeyNotFound(format!("Key '{}' not found", key_id)));
    }

    // Confirm deletion unless force is specified
    if !confirm_operation(&format!("Are you sure you want to delete key '{}'?", key_id), force)? {
        info!("Key deletion cancelled");
        return Ok(());
    }

    // Delete the key file
    fs::remove_file(&key_file_path)?;

    info!("✅ Key '{}' deleted successfully", key_id);

    Ok(())
}

/// Handle listing keys in storage
pub fn handle_list_keys(
    storage_dir: Option<PathBuf>,
    verbose: bool,
) -> KeyResult<()> {
    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()
        .map_err(|e| KeyError::StorageError(format!("Failed to get storage directory: {}", e)))?);

    if !storage_path.exists() {
        info!("No keys found (storage directory doesn't exist)");
        return Ok(());
    }

    // Read directory entries
    let entries = fs::read_dir(&storage_path)?;

    let mut keys = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                keys.push((stem.to_string(), path));
            }
        }
    }

    if keys.is_empty() {
        info!("No keys found in storage directory");
        return Ok(());
    }

    keys.sort_by(|a, b| a.0.cmp(&b.0));

    info!("Stored keys in {}:", storage_path.display());

    for (key_id, path) in keys {
        if verbose {
            // Read and parse key config for detailed info
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<KeyStorageConfig>(&content) {
                    Ok(config) => {
                        info!(
                            "  {} (created: {}, security: {}KB/{}/{})",
                            key_id,
                            config.created_at,
                            config.argon2_params.memory_cost,
                            config.argon2_params.time_cost,
                            config.argon2_params.parallelism
                        );
                    }
                    Err(_) => {
                        info!("  {} (invalid format)", key_id);
                    }
                },
                Err(_) => {
                    info!("  {} (read error)", key_id);
                }
            }
        } else {
            info!("  {}", key_id);
        }
    }

    Ok(())
}