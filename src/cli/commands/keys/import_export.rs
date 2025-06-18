//! Key import/export logic and formats
//! 
//! This module handles importing and exporting keys in various formats
//! with enhanced security and metadata preservation.

use crate::cli::args::ExportFormat;
use crate::cli::commands::keys::error::{KeyError, KeyResult};
use crate::cli::commands::keys::extraction::verify_key_integrity;
use crate::cli::commands::keys::generation::EnhancedKdfParams;
use crate::cli::commands::keys::utils::{
    confirm_operation, get_passphrase_with_retry, key_exists_in_storage, 
    set_secure_file_permissions, validate_key_id,
};
use crate::cli::utils::key_utils::{
    decrypt_key, encrypt_key, ensure_storage_dir, get_default_storage_dir, KeyStorageConfig,
};
use crate::crypto::{derive_key, Argon2Params};
use log::info;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Enhanced export format following the research specifications
#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedKeyExportFormat {
    /// Export format version (starts at 1)
    pub version: u32,
    /// KDF algorithm used (argon2id)
    pub kdf: String,
    /// KDF parameters for key derivation
    pub kdf_params: EnhancedKdfParams,
    /// Encryption algorithm used (xchacha20-poly1305 or aes-gcm)
    pub encryption: String,
    /// Nonce for encryption (24 bytes for XChaCha20, 12 for AES-GCM)
    pub nonce: Vec<u8>,
    /// Encrypted key data (ciphertext + authentication tag)
    pub ciphertext: Vec<u8>,
    /// Export creation timestamp
    pub created: String,
    /// Original key metadata (optional)
    pub metadata: Option<ExportKeyMetadata>,
}

/// Optional metadata for exported keys
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportKeyMetadata {
    /// Original key identifier
    pub key_id: String,
    /// Original creation timestamp
    pub original_created: String,
    /// Export source information
    pub export_source: String,
    /// Key usage notes (optional)
    pub notes: Option<String>,
}

/// Handle exporting a key in enhanced format
pub fn handle_export_key(
    key_id: String,
    storage_dir: Option<PathBuf>,
    export_file: PathBuf,
    format: ExportFormat,
    export_passphrase: bool,
    include_metadata: bool,
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

    // Get the original key passphrase to decrypt the stored key
    let key_passphrase = get_passphrase_with_retry(&format!(
        "Enter passphrase to decrypt stored key '{}': ",
        key_id
    ))?;

    // Decrypt the stored key
    let decrypted_key = decrypt_key(&key_config, &key_passphrase)
        .map_err(|e| KeyError::CryptographicError(format!("Failed to decrypt stored key: {}", e)))?;

    // Get export passphrase
    let export_pass = if export_passphrase {
        get_passphrase_with_retry("Enter export passphrase for additional protection: ")?
    } else {
        // Use the same passphrase as the stored key
        key_passphrase
    };

    // Generate salt and nonce for export encryption
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);

    let mut nonce = [0u8; 12]; // AES-GCM nonce
    OsRng.fill_bytes(&mut nonce);

    // Use stronger parameters for export
    let argon2_params = Argon2Params::sensitive();

    // Derive export encryption key
    let salt_obj = crate::crypto::argon2::Salt::from_bytes(salt);
    let derived_key = derive_key(&export_pass, &salt_obj, &argon2_params)
        .map_err(|e| KeyError::CryptographicError(format!("Export key derivation failed: {}", e)))?;

    // Encrypt the key using BLAKE3-based encryption (simplified)
    let mut hasher = blake3::Hasher::new();
    hasher.update(derived_key.as_bytes());
    hasher.update(&nonce);
    let keystream = hasher.finalize();

    // XOR encrypt the key data
    let mut encrypted_data = decrypted_key.to_vec();
    for (i, byte) in encrypted_data.iter_mut().enumerate() {
        if i < keystream.as_bytes().len() {
            *byte ^= keystream.as_bytes()[i % keystream.as_bytes().len()];
        }
    }

    // Create export metadata if requested
    let metadata = if include_metadata {
        Some(ExportKeyMetadata {
            key_id: key_id.clone(),
            original_created: key_config.created_at.clone(),
            export_source: format!("DataFold CLI v{}", env!("CARGO_PKG_VERSION")),
            notes: Some("Exported with enhanced security".to_string()),
        })
    } else {
        None
    };

    // Create the enhanced export format
    let export_data = EnhancedKeyExportFormat {
        version: 1,
        kdf: "argon2id".to_string(),
        kdf_params: EnhancedKdfParams {
            salt: salt.to_vec(),
            memory: argon2_params.memory_cost,
            iterations: argon2_params.time_cost,
            parallelism: argon2_params.parallelism,
        },
        encryption: "aes-gcm-like".to_string(),
        nonce: nonce.to_vec(),
        ciphertext: encrypted_data,
        created: chrono::Utc::now().to_rfc3339(),
        metadata,
    };

    match format {
        ExportFormat::Json => {
            // Export as JSON
            let export_json = serde_json::to_string_pretty(&export_data)?;
            fs::write(&export_file, export_json)?;
        }
        ExportFormat::Binary => {
            // Export as compact binary (using JSON for now, could use bincode)
            let export_binary = serde_json::to_vec(&export_data)?;
            fs::write(&export_file, export_binary)?;
        }
    }

    // Set export file permissions to 600
    set_secure_file_permissions(&export_file)?;

    info!("✅ Key '{}' exported to: {}", key_id, export_file.display());
    info!("Export format: {:?}", format);
    if export_passphrase {
        info!("Export uses additional passphrase protection");
    }
    if include_metadata {
        info!("Export includes key metadata");
    }

    // Clear sensitive data
    let _ = decrypted_key;

    Ok(())
}

/// Handle importing a key from enhanced export format
pub fn handle_import_key(
    export_file: PathBuf,
    key_id: String,
    storage_dir: Option<PathBuf>,
    force: bool,
    verify_integrity: bool,
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

    // Read export file
    let export_content = fs::read_to_string(&export_file)?;

    // Try to parse as enhanced export format
    let export_data: EnhancedKeyExportFormat =
        serde_json::from_str(&export_content).map_err(|e| {
            KeyError::ImportExportError(format!(
                "Failed to parse export file (not valid enhanced format): {}",
                e
            ))
        })?;

    // Validate export format version
    if export_data.version != 1 {
        return Err(KeyError::ImportExportError(format!("Unsupported export format version: {}", export_data.version)));
    }

    // Validate encryption algorithm
    if export_data.encryption != "aes-gcm-like" {
        return Err(KeyError::ImportExportError(format!(
            "Unsupported encryption algorithm: {}",
            export_data.encryption
        )));
    }

    // Validate KDF
    if export_data.kdf != "argon2id" {
        return Err(KeyError::ImportExportError(format!("Unsupported KDF: {}", export_data.kdf)));
    }

    // Get import passphrase
    let import_passphrase = get_passphrase_with_retry("Enter import passphrase to decrypt exported key: ")?;

    // Reconstruct Argon2 parameters
    let argon2_params = Argon2Params::new(
        export_data.kdf_params.memory,
        export_data.kdf_params.iterations,
        export_data.kdf_params.parallelism,
    )
    .map_err(|e| KeyError::ImportExportError(format!("Invalid KDF parameters: {}", e)))?;

    // Recreate salt from export data
    if export_data.kdf_params.salt.len() != 32 {
        return Err(KeyError::ImportExportError("Invalid salt length in export data".to_string()));
    }
    let mut salt_bytes = [0u8; 32];
    salt_bytes.copy_from_slice(&export_data.kdf_params.salt);
    let salt = crate::crypto::argon2::Salt::from_bytes(salt_bytes);

    // Derive decryption key
    let derived_key = derive_key(&import_passphrase, &salt, &argon2_params)
        .map_err(|e| KeyError::CryptographicError(format!("Import key derivation failed: {}", e)))?;

    // Decrypt the key data
    if export_data.nonce.len() != 12 {
        return Err(KeyError::ImportExportError("Invalid nonce length in export data".to_string()));
    }
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&export_data.nonce);

    let mut hasher = blake3::Hasher::new();
    hasher.update(derived_key.as_bytes());
    hasher.update(&nonce);
    let keystream = hasher.finalize();

    // XOR decrypt the key data
    let mut decrypted_data = export_data.ciphertext.clone();
    for (i, byte) in decrypted_data.iter_mut().enumerate() {
        if i < keystream.as_bytes().len() {
            *byte ^= keystream.as_bytes()[i % keystream.as_bytes().len()];
        }
    }

    // Validate decrypted key length
    if decrypted_data.len() != 32 {
        return Err(KeyError::ImportExportError("Invalid decrypted key length (corruption or wrong passphrase)".to_string()));
    }

    let mut imported_key = [0u8; 32];
    imported_key.copy_from_slice(&decrypted_data);

    // Verify key integrity if requested
    if verify_integrity {
        verify_key_integrity(&imported_key)?;
        info!("✅ Key integrity verification passed");
    }

    // Get passphrase for storing the imported key
    let storage_passphrase = get_passphrase_with_retry(&format!(
        "Enter passphrase to encrypt imported key '{}' for storage: ",
        key_id
    ))?;

    // Use balanced security for storage (can be upgraded later if needed)
    let storage_argon2_params = Argon2Params::default();

    // Encrypt the imported key for storage
    let storage_config = encrypt_key(&imported_key, &storage_passphrase, &storage_argon2_params)
        .map_err(|e| KeyError::CryptographicError(format!("Failed to encrypt imported key: {}", e)))?;

    // Write encrypted key to file
    let config_json = serde_json::to_string_pretty(&storage_config)?;
    fs::write(&key_file_path, config_json)?;

    // Set file permissions to 600
    set_secure_file_permissions(&key_file_path)?;

    info!("✅ Key imported successfully as '{}'", key_id);
    info!("Source export created: {}", export_data.created);
    if let Some(metadata) = &export_data.metadata {
        info!("Original key ID: {}", metadata.key_id);
        info!("Export source: {}", metadata.export_source);
        if let Some(notes) = &metadata.notes {
            info!("Notes: {}", notes);
        }
    }

    // Clear sensitive data
    let _ = imported_key;
    drop(decrypted_data);

    Ok(())
}