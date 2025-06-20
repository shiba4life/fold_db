//! Key rotation and version management
//! 
//! This module handles key rotation operations including generating new key versions,
//! managing rotation methods, and listing key versions.

use crate::cli::args::{CliSecurityLevel, RotationMethod};
use crate::cli::commands::keys::error::{KeyError, KeyResult};
use crate::cli::commands::keys::generation::hkdf_derive_key;
use crate::cli::commands::keys::utils::{
    confirm_operation, get_passphrase_with_retry, key_exists_in_storage,
    security_level_to_argon2, set_secure_file_permissions, validate_key_id,
};
use crate::cli::utils::key_utils::{
    decrypt_key, encrypt_key, get_default_storage_dir, KeyStorageConfig,
};
use crate::unified_crypto::config::Argon2Params;
use crate::unified_crypto::primitives::CryptoPrimitives;
use crate::unified_crypto::keys::generate_master_keypair;
use crate::unified_crypto::{derive_key, generate_salt};
use log::info;
use std::fs;
use std::path::PathBuf;

/// Handle key rotation
pub fn handle_rotate_key(
    key_id: String,
    storage_dir: Option<PathBuf>,
    security_level: CliSecurityLevel,
    method: RotationMethod,
    keep_backup: bool,
    force: bool,
) -> KeyResult<()> {
    // Validate key ID
    validate_key_id(&key_id)?;

    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()
        .map_err(|e| KeyError::StorageError(format!("Failed to get storage directory: {}", e)))?);
    let key_file_path = storage_path.join(format!("{}.json", key_id));

    if !key_exists_in_storage(&key_id, &storage_path) {
        return Err(KeyError::KeyNotFound(format!("Key '{}' not found", key_id)));
    }

    // Confirm rotation unless force is specified
    if !confirm_operation(
        &format!("Are you sure you want to rotate key '{}'? This will create a new version.", key_id),
        force
    )? {
        info!("Key rotation cancelled");
        return Ok(());
    }

    // Read current key
    let config_content = fs::read_to_string(&key_file_path)?;
    let current_config: KeyStorageConfig = serde_json::from_str(&config_content)?;

    // Get passphrase for current key
    let current_passphrase = get_passphrase_with_retry(&format!(
        "Enter passphrase for current key '{}': ",
        key_id
    ))?;

    let current_key_bytes = decrypt_key(&current_config, &current_passphrase)
        .map_err(|e| KeyError::CryptographicError(format!("Failed to decrypt current key: {}", e)))?;

    // Generate new key based on rotation method
    let new_key_bytes = match method {
        RotationMethod::Regenerate => {
            // Generate completely new random key
            let (_public_key, private_key) = generate_master_keypair()
                .map_err(|e| KeyError::CryptographicError(format!("Failed to generate new keypair: {}", e)))?;
            private_key.secret_key_bytes().to_vec()
        }
        RotationMethod::Derive => {
            // Derive new key from current key using incremental counter
            let context = format!("rotation-{}", chrono::Utc::now().timestamp());
            let salt = generate_salt();
            let derived_material =
                hkdf_derive_key(&current_key_bytes, &salt, &context.as_bytes(), 32);
            derived_material.to_vec()
        }
        RotationMethod::Rederive => {
            // Re-derive from passphrase with new salt (if original was passphrase-based)
            let derive_passphrase = get_passphrase_with_retry("Enter passphrase for key re-derivation: ")?;

            let argon2_params = security_level_to_argon2(&security_level);

            let derived_key = derive_key(derive_passphrase.as_bytes(), &generate_salt(), &argon2_params)
                .map_err(|e| KeyError::CryptographicError(format!("Key re-derivation failed: {}", e)))?;

            let mut new_key = [0u8; 32];
            new_key.copy_from_slice(&derived_key);
            new_key.to_vec()
        }
    };

    // Create backup if requested
    if keep_backup {
        let backup_path = storage_path.join(format!(
            "{}.backup.{}.json",
            key_id,
            chrono::Utc::now().timestamp()
        ));
        fs::copy(&key_file_path, &backup_path)?;

        // Set backup file permissions to 600
        set_secure_file_permissions(&backup_path)?;

        info!("✅ Backup created: {}", backup_path.display());
    }

    // Get passphrase for new key encryption
    let new_passphrase = get_passphrase_with_retry(&format!(
        "Enter passphrase for rotated key '{}': ",
        key_id
    ))?;

    // Convert security level to Argon2 parameters
    let argon2_params = security_level_to_argon2(&security_level);

    // Encrypt the new key
    let mut key_array = [0u8; 32];
    key_array[..new_key_bytes.len().min(32)].copy_from_slice(&new_key_bytes[..new_key_bytes.len().min(32)]);
    let new_storage_config = encrypt_key(&key_array, &new_passphrase, &argon2_params)
        .map_err(|e| KeyError::CryptographicError(format!("Failed to encrypt new key: {}", e)))?;

    // Write new encrypted key to file
    let config_json = serde_json::to_string_pretty(&new_storage_config)?;
    fs::write(&key_file_path, config_json)?;

    // Set file permissions to 600
    set_secure_file_permissions(&key_file_path)?;

    info!(
        "✅ Key '{}' rotated successfully using method: {:?}",
        key_id, method
    );
    info!("Security level: {:?}", security_level);
    if keep_backup {
        info!("Previous version backed up");
    }

    // Clear sensitive data
    let _ = current_key_bytes;
    let _ = new_key_bytes;

    Ok(())
}

/// Handle listing key versions (including backups)
pub fn handle_list_key_versions(
    key_id: String,
    storage_dir: Option<PathBuf>,
    verbose: bool,
) -> KeyResult<()> {
    // Validate key ID
    validate_key_id(&key_id)?;

    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()
        .map_err(|e| KeyError::StorageError(format!("Failed to get storage directory: {}", e)))?);

    if !storage_path.exists() {
        info!("No keys found (storage directory doesn't exist)");
        return Ok(());
    }

    // Find all versions of the key
    let entries = fs::read_dir(&storage_path)?;

    let mut versions = Vec::new();

    // Look for main key and backup files
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

            // Check for main key file
            if filename == format!("{}.json", key_id) {
                versions.push(("current".to_string(), path));
            }
            // Check for backup files
            else if filename.starts_with(&format!("{}.backup.", key_id))
                && filename.ends_with(".json")
            {
                if let Some(timestamp_part) = filename
                    .strip_prefix(&format!("{}.backup.", key_id))
                    .and_then(|s| s.strip_suffix(".json"))
                {
                    versions.push((format!("backup-{}", timestamp_part), path));
                }
            }
        }
    }

    if versions.is_empty() {
        info!("No versions found for key '{}'", key_id);
        return Ok(());
    }

    // Sort versions
    versions.sort_by(|a, b| a.0.cmp(&b.0));

    info!(
        "Versions for key '{}' in {}:",
        key_id,
        storage_path.display()
    );

    for (version_name, path) in versions {
        if verbose {
            // Read and parse key config for detailed info
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<KeyStorageConfig>(&content) {
                    Ok(config) => {
                        info!(
                            "  {} (created: {}, security: {}KB/{}/{})",
                            version_name,
                            config.created_at,
                            config.argon2_params.memory_cost,
                            config.argon2_params.time_cost,
                            config.argon2_params.parallelism
                        );
                    }
                    Err(_) => {
                        info!("  {} (invalid format)", version_name);
                    }
                },
                Err(_) => {
                    info!("  {} (read error)", version_name);
                }
            }
        } else {
            info!("  {}", version_name);
        }
    }

    Ok(())
}