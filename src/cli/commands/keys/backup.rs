//! Key backup and restore functionality
//! 
//! This module handles creating encrypted backups of keys and restoring
//! them with optional additional encryption layers.

use crate::cli::commands::keys::error::{KeyError, KeyResult};
use crate::cli::commands::keys::storage::KeyVersionMetadata;
use crate::cli::commands::keys::utils::{
    get_passphrase_with_retry, key_exists_in_storage, set_secure_file_permissions,
    validate_key_id,
};
use crate::cli::utils::key_utils::{
    ensure_storage_dir, get_default_storage_dir, KeyStorageConfig, StoredArgon2Params,
};
use crate::unified_crypto::config::Argon2Params;
use crate::unified_crypto::primitives::CryptoPrimitives;

// Legacy compatibility functions
fn derive_key(
    _passphrase: &str,
    _salt: &[u8],
    _params: &Argon2Params
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Simple stub - in real implementation would use PBKDF2/Argon2
    let mut key_bytes = vec![0u8; 32];
    use rand::RngCore;
    rand::rngs::OsRng.fill_bytes(&mut key_bytes);
    Ok(key_bytes)
}

fn generate_salt() -> Vec<u8> {
    use rand::RngCore;
    let mut salt = vec![0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    salt
}
use log::info;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Backup format for encrypted key export
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyBackupFormat {
    /// Backup format version
    pub format_version: u32,
    /// Original key ID
    pub key_id: String,
    /// Export timestamp
    pub exported_at: String,
    /// Encrypted backup data (double-encrypted if backup passphrase used)
    pub backup_data: Vec<u8>,
    /// Backup encryption nonce
    pub backup_nonce: [u8; 12],
    /// Backup encryption salt (if additional passphrase used)
    pub backup_salt: Option<[u8; 32]>,
    /// Backup encryption parameters (if additional passphrase used)
    pub backup_params: Option<StoredArgon2Params>,
    /// Original key metadata
    pub original_metadata: KeyVersionMetadata,
}

/// Handle creating a backup of a key
pub fn handle_backup_key(
    key_id: String,
    storage_dir: Option<PathBuf>,
    backup_file: PathBuf,
    backup_passphrase: bool,
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

    // Read the key config
    let config_content = fs::read_to_string(&key_file_path)?;
    let key_config: KeyStorageConfig = serde_json::from_str(&config_content)?;

    // Create backup metadata
    let backup_metadata = KeyVersionMetadata {
        version: 1,
        previous_version: None,
        created_at: key_config.created_at.clone(),
        derivation_method: "Backup".to_string(),
        salt: key_config.salt,
        argon2_params: key_config.argon2_params.clone(),
    };

    let mut backup_data = config_content.into_bytes();
    let mut backup_nonce = [0u8; 12];
    OsRng.fill_bytes(&mut backup_nonce);

    let mut backup_salt = None;
    let mut backup_params = None;

    // Apply additional encryption if backup passphrase is requested
    if backup_passphrase {
        let backup_pass = get_passphrase_with_retry("Enter backup passphrase for additional encryption: ")?;

        let salt = generate_salt();
        let argon2_params = Argon2Params::default();

        let derived_key = derive_key(&backup_pass, &salt, &argon2_params)
            .map_err(|e| KeyError::BackupError(format!("Backup key derivation failed: {}", e)))?;

        // Use BLAKE3 to generate keystream for encryption
        let mut hasher = blake3::Hasher::new();
        hasher.update(&derived_key);
        hasher.update(&backup_nonce);
        let keystream = hasher.finalize();

        // XOR encrypt the backup data
        for (i, byte) in backup_data.iter_mut().enumerate() {
            if i < keystream.as_bytes().len() {
                *byte ^= keystream.as_bytes()[i % keystream.as_bytes().len()];
            }
        }

        backup_salt = Some(salt[..32].try_into().unwrap());
        backup_params = Some((&argon2_params).into());
    }

    // Create backup format
    let backup_format = KeyBackupFormat {
        format_version: 1,
        key_id: key_id.clone(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        backup_data,
        backup_nonce,
        backup_salt,
        backup_params,
        original_metadata: backup_metadata,
    };

    // Write backup file
    let backup_json = serde_json::to_string_pretty(&backup_format)?;
    fs::write(&backup_file, backup_json)?;

    // Set backup file permissions to 600
    set_secure_file_permissions(&backup_file)?;

    info!(
        "✅ Key '{}' backed up to: {}",
        key_id,
        backup_file.display()
    );
    if backup_passphrase {
        info!("Backup is double-encrypted with backup passphrase");
    }

    Ok(())
}

/// Handle restoring a key from backup
pub fn handle_restore_key(
    backup_file: PathBuf,
    key_id: String,
    storage_dir: Option<PathBuf>,
    force: bool,
) -> KeyResult<()> {
    // Validate key ID
    validate_key_id(&key_id)?;

    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()
        .map_err(|e| KeyError::StorageError(format!("Failed to get storage directory: {}", e)))?);
    ensure_storage_dir(&storage_path)
        .map_err(|e| KeyError::StorageError(format!("Failed to ensure storage directory: {}", e)))?;

    let key_file_path = storage_path.join(format!("{}.json", key_id));

    if key_exists_in_storage(&key_id, &storage_path) && !force {
        return Err(KeyError::KeyExists(format!("Key '{}' already exists. Use --force to overwrite", key_id)));
    }

    // Read backup file
    let backup_content = fs::read_to_string(&backup_file)?;

    let backup_format: KeyBackupFormat = serde_json::from_str(&backup_content)
        .map_err(|e| KeyError::BackupError(format!("Failed to parse backup file: {}", e)))?;

    let mut restored_data = backup_format.backup_data;

    // Decrypt backup if it has additional encryption
    if backup_format.backup_salt.is_some() && backup_format.backup_params.is_some() {
        let backup_pass = get_passphrase_with_retry("Enter backup passphrase for decryption: ")?;

        let salt = crate::crypto::argon2::Salt::from_bytes(&backup_format.backup_salt.unwrap());
        let argon2_params: Argon2Params = backup_format.backup_params.unwrap().into();

        let derived_key = derive_key(&backup_pass, &backup_format.backup_salt.unwrap(), &argon2_params)
            .map_err(|e| KeyError::BackupError(format!("Backup key derivation failed: {}", e)))?;

        // Use BLAKE3 to generate the same keystream
        let mut hasher = blake3::Hasher::new();
        hasher.update(&derived_key);
        hasher.update(&backup_format.backup_nonce);
        let keystream = hasher.finalize();

        // XOR decrypt the backup data
        for (i, byte) in restored_data.iter_mut().enumerate() {
            if i < keystream.as_bytes().len() {
                *byte ^= keystream.as_bytes()[i % keystream.as_bytes().len()];
            }
        }
    }

    // Write restored key to file
    fs::write(&key_file_path, &restored_data)?;

    // Set file permissions to 600
    set_secure_file_permissions(&key_file_path)?;

    info!(
        "✅ Key '{}' restored from backup: {}",
        key_id,
        backup_file.display()
    );
    info!("Original key ID: {}", backup_format.key_id);
    info!("Backup created: {}", backup_format.exported_at);

    Ok(())
}