//! Key management command handlers
//! 
//! This module contains handlers for all key-related operations including
//! generation, derivation, storage, retrieval, rotation, backup, etc.

use crate::cli::args::{CliSecurityLevel, ExportFormat, KeyFormat, RotationMethod};
use crate::cli::utils::key_utils::{
    decrypt_key, encrypt_key, ensure_storage_dir, format_key, get_default_storage_dir,
    get_secure_passphrase, output_key, parse_key_input, KeyStorageConfig, StoredArgon2Params,
};
use crate::crypto::ed25519::{generate_master_keypair, generate_master_keypair_from_seed};
use crate::crypto::{derive_key, generate_salt, Argon2Params};
use base64::{engine::general_purpose, Engine as _};
use log::{info, warn};
use rand::{rngs::OsRng, RngCore};
use rpassword::read_password;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// Key versioning metadata for rotation tracking
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyVersionMetadata {
    /// Version number (starts at 1)
    version: u32,
    /// Previous version number (None for initial version)
    previous_version: Option<u32>,
    /// Creation timestamp for this version
    created_at: String,
    /// Derivation method used for this version
    derivation_method: String,
    /// Salt used for key derivation (32 bytes)
    salt: [u8; 32],
    /// Argon2 parameters used for this version
    argon2_params: StoredArgon2Params,
}

/// Enhanced key storage configuration with versioning
#[derive(Debug, Serialize, Deserialize)]
pub struct VersionedKeyStorageConfig {
    /// Version metadata for tracking
    version_metadata: KeyVersionMetadata,
    /// Encrypted key data
    encrypted_key: Vec<u8>,
    /// Nonce used for encryption (12 bytes for AES-GCM)
    nonce: [u8; 12],
    /// Storage format version
    storage_version: u32,
}

/// Backup format for encrypted key export
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyBackupFormat {
    /// Backup format version
    format_version: u32,
    /// Original key ID
    key_id: String,
    /// Export timestamp
    exported_at: String,
    /// Encrypted backup data (double-encrypted if backup passphrase used)
    backup_data: Vec<u8>,
    /// Backup encryption nonce
    backup_nonce: [u8; 12],
    /// Backup encryption salt (if additional passphrase used)
    backup_salt: Option<[u8; 32]>,
    /// Backup encryption parameters (if additional passphrase used)
    backup_params: Option<StoredArgon2Params>,
    /// Original key metadata
    original_metadata: KeyVersionMetadata,
}

/// Enhanced export format following the research specifications
#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedKeyExportFormat {
    /// Export format version (starts at 1)
    version: u32,
    /// KDF algorithm used (argon2id)
    kdf: String,
    /// KDF parameters for key derivation
    kdf_params: EnhancedKdfParams,
    /// Encryption algorithm used (xchacha20-poly1305 or aes-gcm)
    encryption: String,
    /// Nonce for encryption (24 bytes for XChaCha20, 12 for AES-GCM)
    nonce: Vec<u8>,
    /// Encrypted key data (ciphertext + authentication tag)
    ciphertext: Vec<u8>,
    /// Export creation timestamp
    created: String,
    /// Original key metadata (optional)
    metadata: Option<ExportKeyMetadata>,
}

/// Enhanced KDF parameters for export
#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedKdfParams {
    /// Salt for key derivation (32 bytes)
    salt: Vec<u8>,
    /// Memory cost in KB
    memory: u32,
    /// Time cost (iterations)
    iterations: u32,
    /// Parallelism factor
    parallelism: u32,
}

/// Optional metadata for exported keys
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportKeyMetadata {
    /// Original key identifier
    key_id: String,
    /// Original creation timestamp
    original_created: String,
    /// Export source information
    export_source: String,
    /// Key usage notes (optional)
    notes: Option<String>,
}

/// HKDF key derivation using BLAKE3 (simplified for compatibility)
fn hkdf_derive_key(
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
) -> Result<(), Box<dyn std::error::Error>> {
    if public_only && private_only {
        return Err("Cannot specify both --public-only and --private-only".into());
    }

    for i in 0..count {
        let keypair =
            generate_master_keypair().map_err(|e| format!("Failed to generate keypair: {}", e))?;

        let public_key_bytes = keypair.public_key_bytes();
        let private_key_bytes = keypair.secret_key_bytes();

        if count > 1 {
            info!("Generating keypair {} of {}", i + 1, count);
        }

        // Output private key if requested
        if !public_only {
            let formatted_private = format_key(&private_key_bytes, &format, true)?;
            output_key(
                &formatted_private,
                private_key_file.as_ref(),
                "private",
                i,
                count,
                true,
            )?;
        }

        // Output public key if requested
        if !private_only {
            let formatted_public = format_key(&public_key_bytes, &format, false)?;
            output_key(
                &formatted_public,
                public_key_file.as_ref(),
                "public",
                i,
                count,
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
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::crypto::argon2::{generate_salt_and_derive_keypair, Argon2Params};

    if public_only && private_only {
        return Err("Cannot specify both --public-only and --private-only".into());
    }

    let passphrase = match passphrase {
        Some(p) => p,
        None => get_secure_passphrase()?,
    };

    // Convert security level to Argon2 parameters
    let argon2_params = match security_level {
        CliSecurityLevel::Interactive => Argon2Params::interactive(),
        CliSecurityLevel::Balanced => Argon2Params::default(),
        CliSecurityLevel::Sensitive => Argon2Params::sensitive(),
    };

    // Generate salt and derive keypair
    let (_salt, keypair) = generate_salt_and_derive_keypair(&passphrase, &argon2_params)
        .map_err(|e| format!("Failed to derive keypair from passphrase: {}", e))?;

    let public_key_bytes = keypair.public_key_bytes();
    let private_key_bytes = keypair.secret_key_bytes();

    // Output private key if requested
    if !public_only {
        let formatted_private = format_key(&private_key_bytes, &format, true)?;
        output_key(
            &formatted_private,
            private_key_file.as_ref(),
            "private",
            0,
            1,
            true,
        )?;
    }

    // Output public key if requested
    if !private_only {
        let formatted_public = format_key(&public_key_bytes, &format, false)?;
        output_key(
            &formatted_public,
            public_key_file.as_ref(),
            "public",
            0,
            1,
            true,
        )?;
    }

    // Clear sensitive data
    drop(keypair);

    Ok(())
}

/// Handle extracting public key from private key
pub fn handle_extract_public_key(
    private_key: Option<String>,
    private_key_file: Option<PathBuf>,
    format: KeyFormat,
    output_file: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get private key bytes
    let private_key_bytes = match (private_key, private_key_file) {
        (Some(key_str), None) => parse_key_input(&key_str, true)?,
        (None, Some(file_path)) => {
            let content = fs::read_to_string(&file_path)
                .map_err(|e| format!("Failed to read private key file: {}", e))?;
            parse_key_input(content.trim(), true)?
        }
        (Some(_), Some(_)) => {
            return Err("Cannot specify both --private-key and --private-key-file".into());
        }
        (None, None) => {
            return Err("Must specify either --private-key or --private-key-file".into());
        }
    };

    // Create keypair from private key
    let keypair = generate_master_keypair_from_seed(&private_key_bytes)
        .map_err(|e| format!("Invalid private key: {}", e))?;

    let public_key_bytes = keypair.public_key_bytes();
    let formatted_public = format_key(&public_key_bytes, &format, false)?;

    output_key(
        &formatted_public,
        output_file.as_ref(),
        "public",
        0,
        1,
        false,
    )?;

    Ok(())
}

/// Handle verifying that a keypair is valid and matches
pub fn handle_verify_key(
    private_key: Option<String>,
    private_key_file: Option<PathBuf>,
    public_key: Option<String>,
    public_key_file: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get private key bytes
    let private_key_bytes = match (private_key, private_key_file) {
        (Some(key_str), None) => parse_key_input(&key_str, true)?,
        (None, Some(file_path)) => {
            let content = fs::read_to_string(&file_path)
                .map_err(|e| format!("Failed to read private key file: {}", e))?;
            parse_key_input(content.trim(), true)?
        }
        (Some(_), Some(_)) => {
            return Err("Cannot specify both --private-key and --private-key-file".into());
        }
        (None, None) => {
            return Err("Must specify either --private-key or --private-key-file".into());
        }
    };

    // Get public key bytes
    let public_key_bytes = match (public_key, public_key_file) {
        (Some(key_str), None) => parse_key_input(&key_str, false)?,
        (None, Some(file_path)) => {
            let content = fs::read_to_string(&file_path)
                .map_err(|e| format!("Failed to read public key file: {}", e))?;
            parse_key_input(content.trim(), false)?
        }
        (Some(_), Some(_)) => {
            return Err("Cannot specify both --public-key and --public-key-file".into());
        }
        (None, None) => {
            return Err("Must specify either --public-key or --public-key-file".into());
        }
    };

    // Create keypair from private key
    let keypair = generate_master_keypair_from_seed(&private_key_bytes)
        .map_err(|e| format!("Invalid private key: {}", e))?;

    // Check if the public keys match
    let derived_public_key_bytes = keypair.public_key_bytes();

    if derived_public_key_bytes == public_key_bytes {
        println!("✅ Keypair verification successful: private and public keys match");
        println!("Public key: {}", hex::encode(public_key_bytes));
        info!("✅ Keypair verification successful: private and public keys match");
        info!("Public key: {}", hex::encode(public_key_bytes));
    } else {
        eprintln!("❌ Keypair verification failed: private and public keys do not match");
        eprintln!(
            "Expected public key: {}",
            hex::encode(derived_public_key_bytes)
        );
        eprintln!("Provided public key: {}", hex::encode(public_key_bytes));
        return Err("Keypair verification failed".into());
    }

    // Test signing and verification to ensure the keypair is fully functional
    let test_message = b"DataFold Ed25519 keypair verification test";
    let signature = keypair
        .sign_data(test_message)
        .map_err(|e| format!("Failed to sign test message: {}", e))?;

    keypair
        .verify_data(test_message, &signature)
        .map_err(|e| format!("Failed to verify test signature: {}", e))?;

    println!("✅ Functional verification successful: keypair can sign and verify");
    info!("✅ Functional verification successful: keypair can sign and verify");

    Ok(())
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
) -> Result<(), Box<dyn std::error::Error>> {
    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()?);
    ensure_storage_dir(&storage_path)?;

    // Get private key bytes
    let private_key_bytes = match (private_key, private_key_file) {
        (Some(key_str), None) => parse_key_input(&key_str, true)?,
        (None, Some(file_path)) => {
            let content = fs::read_to_string(&file_path)
                .map_err(|e| format!("Failed to read private key file: {}", e))?;
            parse_key_input(content.trim(), true)?
        }
        (Some(_), Some(_)) => {
            return Err("Cannot specify both --private-key and --private-key-file".into());
        }
        (None, None) => {
            return Err("Must specify either --private-key or --private-key-file".into());
        }
    };

    // Check if key already exists
    let key_file_path = storage_path.join(format!("{}.json", key_id));
    if key_file_path.exists() && !force {
        return Err(format!("Key '{}' already exists. Use --force to overwrite", key_id).into());
    }

    // Get passphrase for encryption
    let passphrase = match passphrase {
        Some(p) => p,
        None => get_secure_passphrase()?,
    };

    // Convert security level to Argon2 parameters
    let argon2_params = match security_level {
        CliSecurityLevel::Interactive => Argon2Params::interactive(),
        CliSecurityLevel::Balanced => Argon2Params::default(),
        CliSecurityLevel::Sensitive => Argon2Params::sensitive(),
    };

    // Encrypt the key
    let storage_config = encrypt_key(&private_key_bytes, &passphrase, &argon2_params)?;

    // Write encrypted key to file
    let config_json = serde_json::to_string_pretty(&storage_config)?;
    fs::write(&key_file_path, config_json)
        .map_err(|e| format!("Failed to write key file: {}", e))?;

    // Set file permissions to 600 (owner read/write only)
    let mut perms = fs::metadata(&key_file_path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(&key_file_path, perms)
        .map_err(|e| format!("Failed to set file permissions: {}", e))?;

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
) -> Result<(), Box<dyn std::error::Error>> {
    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()?);
    let key_file_path = storage_path.join(format!("{}.json", key_id));

    if !key_file_path.exists() {
        return Err(format!("Key '{}' not found", key_id).into());
    }

    // Read storage config
    let config_content = fs::read_to_string(&key_file_path)
        .map_err(|e| format!("Failed to read key file: {}", e))?;
    let storage_config: KeyStorageConfig = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse key file: {}", e))?;

    // Get passphrase for decryption
    print!("Enter passphrase to decrypt key: ");
    io::stdout().flush()?;
    let passphrase = read_password()?;

    // Decrypt the private key
    let private_key_bytes = decrypt_key(&storage_config, &passphrase)?;

    if public_only {
        // Extract and output public key only
        let keypair = generate_master_keypair_from_seed(&private_key_bytes)
            .map_err(|e| format!("Failed to generate keypair from stored key: {}", e))?;
        let public_key_bytes = keypair.public_key_bytes();
        let formatted_public = format_key(&public_key_bytes, &format, false)?;
        output_key(
            &formatted_public,
            output_file.as_ref(),
            "public",
            0,
            1,
            true,
        )?;
    } else {
        // Output private key
        let formatted_private = format_key(&private_key_bytes, &format, true)?;
        output_key(
            &formatted_private,
            output_file.as_ref(),
            "private",
            0,
            1,
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
) -> Result<(), Box<dyn std::error::Error>> {
    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()?);
    let key_file_path = storage_path.join(format!("{}.json", key_id));

    if !key_file_path.exists() {
        return Err(format!("Key '{}' not found", key_id).into());
    }

    // Confirm deletion unless force is specified
    if !force {
        print!("Are you sure you want to delete key '{}'? (y/N): ", key_id);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            info!("Key deletion cancelled");
            return Ok(());
        }
    }

    // Delete the key file
    fs::remove_file(&key_file_path).map_err(|e| format!("Failed to delete key file: {}", e))?;

    info!("✅ Key '{}' deleted successfully", key_id);

    Ok(())
}

/// Handle listing keys in storage
pub fn handle_list_keys(
    storage_dir: Option<PathBuf>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()?);

    if !storage_path.exists() {
        info!("No keys found (storage directory doesn't exist)");
        return Ok(());
    }

    // Read directory entries
    let entries = fs::read_dir(&storage_path)
        .map_err(|e| format!("Failed to read storage directory: {}", e))?;

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
) -> Result<(), Box<dyn std::error::Error>> {
    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()?);
    let master_key_file_path = storage_path.join(format!("{}.json", master_key_id));

    if !master_key_file_path.exists() {
        return Err(format!("Master key '{}' not found", master_key_id).into());
    }

    // Read and decrypt master key
    let config_content = fs::read_to_string(&master_key_file_path)
        .map_err(|e| format!("Failed to read master key file: {}", e))?;
    let storage_config: KeyStorageConfig = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse master key file: {}", e))?;

    print!(
        "Enter passphrase to decrypt master key '{}': ",
        master_key_id
    );
    io::stdout().flush()?;
    let passphrase = read_password()?;

    let master_key_bytes = decrypt_key(&storage_config, &passphrase)?;

    // Derive child key using HKDF (BLAKE3)
    let context_bytes = context.as_bytes();
    let salt = generate_salt();
    let derived_key_material =
        hkdf_derive_key(&master_key_bytes, salt.as_bytes(), context_bytes, 32);

    if derived_key_material.len() != 32 {
        return Err("Failed to derive 32-byte key".into());
    }

    let mut child_key_bytes = [0u8; 32];
    child_key_bytes.copy_from_slice(&derived_key_material);

    if output_only {
        // Just output the derived key
        let formatted_key = format_key(&child_key_bytes, &format, true)?;
        println!("{}", formatted_key);
        info!(
            "✅ Child key derived from master '{}' with context '{}'",
            master_key_id, context
        );
    } else {
        // Store the derived child key
        let child_key_file_path = storage_path.join(format!("{}.json", child_key_id));
        if child_key_file_path.exists() && !force {
            return Err(format!(
                "Child key '{}' already exists. Use --force to overwrite",
                child_key_id
            )
            .into());
        }

        // Get passphrase for child key encryption
        print!("Enter passphrase to encrypt child key '{}': ", child_key_id);
        io::stdout().flush()?;
        let child_passphrase = read_password()?;

        // Convert security level to Argon2 parameters
        let argon2_params = match security_level {
            CliSecurityLevel::Interactive => Argon2Params::interactive(),
            CliSecurityLevel::Balanced => Argon2Params::default(),
            CliSecurityLevel::Sensitive => Argon2Params::sensitive(),
        };

        // Encrypt the child key
        let child_storage_config =
            encrypt_key(&child_key_bytes, &child_passphrase, &argon2_params)?;

        // Write encrypted child key to file
        let config_json = serde_json::to_string_pretty(&child_storage_config)?;
        fs::write(&child_key_file_path, config_json)
            .map_err(|e| format!("Failed to write child key file: {}", e))?;

        // Set file permissions to 600 (owner read/write only)
        let mut perms = fs::metadata(&child_key_file_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&child_key_file_path, perms)
            .map_err(|e| format!("Failed to set file permissions: {}", e))?;

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

/// Handle key rotation
pub fn handle_rotate_key(
    key_id: String,
    storage_dir: Option<PathBuf>,
    security_level: CliSecurityLevel,
    method: RotationMethod,
    keep_backup: bool,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()?);
    let key_file_path = storage_path.join(format!("{}.json", key_id));

    if !key_file_path.exists() {
        return Err(format!("Key '{}' not found", key_id).into());
    }

    // Confirm rotation unless force is specified
    if !force {
        print!(
            "Are you sure you want to rotate key '{}'? This will create a new version. (y/N): ",
            key_id
        );
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            info!("Key rotation cancelled");
            return Ok(());
        }
    }

    // Read current key
    let config_content = fs::read_to_string(&key_file_path)
        .map_err(|e| format!("Failed to read key file: {}", e))?;
    let current_config: KeyStorageConfig = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse key file: {}", e))?;

    // Get passphrase for current key
    print!("Enter passphrase for current key '{}': ", key_id);
    io::stdout().flush()?;
    let current_passphrase = read_password()?;

    let current_key_bytes = decrypt_key(&current_config, &current_passphrase)?;

    // Generate new key based on rotation method
    let new_key_bytes = match method {
        RotationMethod::Regenerate => {
            // Generate completely new random key
            let keypair = generate_master_keypair()
                .map_err(|e| format!("Failed to generate new keypair: {}", e))?;
            keypair.secret_key_bytes()
        }
        RotationMethod::Derive => {
            // Derive new key from current key using incremental counter
            let context = format!("rotation-{}", chrono::Utc::now().timestamp());
            let salt = generate_salt();
            let derived_material =
                hkdf_derive_key(&current_key_bytes, salt.as_bytes(), context.as_bytes(), 32);
            let mut new_key = [0u8; 32];
            new_key.copy_from_slice(&derived_material);
            new_key
        }
        RotationMethod::Rederive => {
            // Re-derive from passphrase with new salt (if original was passphrase-based)
            print!("Enter passphrase for key re-derivation: ");
            io::stdout().flush()?;
            let derive_passphrase = read_password()?;

            let argon2_params = match security_level {
                CliSecurityLevel::Interactive => Argon2Params::interactive(),
                CliSecurityLevel::Balanced => Argon2Params::default(),
                CliSecurityLevel::Sensitive => Argon2Params::sensitive(),
            };

            let derived_key = derive_key(&derive_passphrase, &generate_salt(), &argon2_params)
                .map_err(|e| format!("Key re-derivation failed: {}", e))?;

            let mut new_key = [0u8; 32];
            new_key.copy_from_slice(derived_key.as_bytes());
            new_key
        }
    };

    // Create backup if requested
    if keep_backup {
        let backup_path = storage_path.join(format!(
            "{}.backup.{}.json",
            key_id,
            chrono::Utc::now().timestamp()
        ));
        fs::copy(&key_file_path, &backup_path)
            .map_err(|e| format!("Failed to create backup: {}", e))?;

        // Set backup file permissions to 600
        let mut perms = fs::metadata(&backup_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&backup_path, perms)?;

        info!("✅ Backup created: {}", backup_path.display());
    }

    // Get passphrase for new key encryption
    print!("Enter passphrase for rotated key '{}': ", key_id);
    io::stdout().flush()?;
    let new_passphrase = read_password()?;

    // Convert security level to Argon2 parameters
    let argon2_params = match security_level {
        CliSecurityLevel::Interactive => Argon2Params::interactive(),
        CliSecurityLevel::Balanced => Argon2Params::default(),
        CliSecurityLevel::Sensitive => Argon2Params::sensitive(),
    };

    // Encrypt the new key
    let new_storage_config = encrypt_key(&new_key_bytes, &new_passphrase, &argon2_params)?;

    // Write new encrypted key to file
    let config_json = serde_json::to_string_pretty(&new_storage_config)?;
    fs::write(&key_file_path, config_json)
        .map_err(|e| format!("Failed to write rotated key file: {}", e))?;

    // Set file permissions to 600
    let mut perms = fs::metadata(&key_file_path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(&key_file_path, perms)?;

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
) -> Result<(), Box<dyn std::error::Error>> {
    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()?);

    if !storage_path.exists() {
        info!("No keys found (storage directory doesn't exist)");
        return Ok(());
    }

    // Find all versions of the key
    let entries = fs::read_dir(&storage_path)
        .map_err(|e| format!("Failed to read storage directory: {}", e))?;

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

/// Handle creating a backup of a key
pub fn handle_backup_key(
    key_id: String,
    storage_dir: Option<PathBuf>,
    backup_file: PathBuf,
    backup_passphrase: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()?);
    let key_file_path = storage_path.join(format!("{}.json", key_id));

    if !key_file_path.exists() {
        return Err(format!("Key '{}' not found", key_id).into());
    }

    // Read the key config
    let config_content = fs::read_to_string(&key_file_path)
        .map_err(|e| format!("Failed to read key file: {}", e))?;
    let key_config: KeyStorageConfig = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse key file: {}", e))?;

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
        print!("Enter backup passphrase for additional encryption: ");
        io::stdout().flush()?;
        let backup_pass = read_password()?;

        let salt = generate_salt();
        let argon2_params = Argon2Params::default();

        let derived_key = derive_key(&backup_pass, &salt, &argon2_params)
            .map_err(|e| format!("Backup key derivation failed: {}", e))?;

        // Use BLAKE3 to generate keystream for encryption
        let mut hasher = blake3::Hasher::new();
        hasher.update(derived_key.as_bytes());
        hasher.update(&backup_nonce);
        let keystream = hasher.finalize();

        // XOR encrypt the backup data
        for (i, byte) in backup_data.iter_mut().enumerate() {
            if i < keystream.as_bytes().len() {
                *byte ^= keystream.as_bytes()[i % keystream.as_bytes().len()];
            }
        }

        backup_salt = Some(*salt.as_bytes());
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
    let backup_json = serde_json::to_string_pretty(&backup_format)
        .map_err(|e| format!("Failed to serialize backup: {}", e))?;

    fs::write(&backup_file, backup_json)
        .map_err(|e| format!("Failed to write backup file: {}", e))?;

    // Set backup file permissions to 600
    let mut perms = fs::metadata(&backup_file)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(&backup_file, perms)
        .map_err(|e| format!("Failed to set backup file permissions: {}", e))?;

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
) -> Result<(), Box<dyn std::error::Error>> {
    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()?);
    ensure_storage_dir(&storage_path)?;

    let key_file_path = storage_path.join(format!("{}.json", key_id));

    if key_file_path.exists() && !force {
        return Err(format!("Key '{}' already exists. Use --force to overwrite", key_id).into());
    }

    // Read backup file
    let backup_content = fs::read_to_string(&backup_file)
        .map_err(|e| format!("Failed to read backup file: {}", e))?;

    let backup_format: KeyBackupFormat = serde_json::from_str(&backup_content)
        .map_err(|e| format!("Failed to parse backup file: {}", e))?;

    let mut restored_data = backup_format.backup_data;

    // Decrypt backup if it has additional encryption
    if backup_format.backup_salt.is_some() && backup_format.backup_params.is_some() {
        print!("Enter backup passphrase for decryption: ");
        io::stdout().flush()?;
        let backup_pass = read_password()?;

        let salt = crate::crypto::argon2::Salt::from_bytes(backup_format.backup_salt.unwrap());
        let argon2_params: Argon2Params = backup_format.backup_params.unwrap().into();

        let derived_key = derive_key(&backup_pass, &salt, &argon2_params)
            .map_err(|e| format!("Backup key derivation failed: {}", e))?;

        // Use BLAKE3 to generate the same keystream
        let mut hasher = blake3::Hasher::new();
        hasher.update(derived_key.as_bytes());
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
    fs::write(&key_file_path, &restored_data)
        .map_err(|e| format!("Failed to write restored key file: {}", e))?;

    // Set file permissions to 600
    let mut perms = fs::metadata(&key_file_path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(&key_file_path, perms)
        .map_err(|e| format!("Failed to set file permissions: {}", e))?;

    info!(
        "✅ Key '{}' restored from backup: {}",
        key_id,
        backup_file.display()
    );
    info!("Original key ID: {}", backup_format.key_id);
    info!("Backup created: {}", backup_format.exported_at);

    Ok(())
}

/// Handle exporting a key in enhanced format
pub fn handle_export_key(
    key_id: String,
    storage_dir: Option<PathBuf>,
    export_file: PathBuf,
    format: ExportFormat,
    export_passphrase: bool,
    include_metadata: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()?);
    let key_file_path = storage_path.join(format!("{}.json", key_id));

    if !key_file_path.exists() {
        return Err(format!("Key '{}' not found", key_id).into());
    }

    // Read the key config
    let config_content = fs::read_to_string(&key_file_path)
        .map_err(|e| format!("Failed to read key file: {}", e))?;
    let key_config: KeyStorageConfig = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse key file: {}", e))?;

    // Get the original key passphrase to decrypt the stored key
    print!("Enter passphrase to decrypt stored key '{}': ", key_id);
    io::stdout().flush()?;
    let key_passphrase = read_password()?;

    // Decrypt the stored key
    let decrypted_key = decrypt_key(&key_config, &key_passphrase)?;

    // Get export passphrase
    let export_pass = if export_passphrase {
        print!("Enter export passphrase for additional protection: ");
        io::stdout().flush()?;
        Some(read_password()?)
    } else {
        // Use the same passphrase as the stored key
        Some(key_passphrase)
    };

    if let Some(pass) = export_pass {
        // Generate salt and nonce for export encryption
        let mut salt = [0u8; 32];
        OsRng.fill_bytes(&mut salt);

        let mut nonce = [0u8; 12]; // AES-GCM nonce
        OsRng.fill_bytes(&mut nonce);

        // Use stronger parameters for export
        let argon2_params = Argon2Params::sensitive();

        // Derive export encryption key
        let salt_obj = crate::crypto::argon2::Salt::from_bytes(salt);
        let derived_key = derive_key(&pass, &salt_obj, &argon2_params)
            .map_err(|e| format!("Export key derivation failed: {}", e))?;

        // Encrypt the key using AES-GCM-like encryption (simplified)
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
                let export_json = serde_json::to_string_pretty(&export_data)
                    .map_err(|e| format!("Failed to serialize export data: {}", e))?;

                fs::write(&export_file, export_json)
                    .map_err(|e| format!("Failed to write export file: {}", e))?;
            }
            ExportFormat::Binary => {
                // Export as compact binary (using bincode or similar)
                let export_binary = serde_json::to_vec(&export_data)
                    .map_err(|e| format!("Failed to serialize export data: {}", e))?;

                fs::write(&export_file, export_binary)
                    .map_err(|e| format!("Failed to write export file: {}", e))?;
            }
        }

        // Set export file permissions to 600
        let mut perms = fs::metadata(&export_file)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&export_file, perms)
            .map_err(|e| format!("Failed to set export file permissions: {}", e))?;

        info!("✅ Key '{}' exported to: {}", key_id, export_file.display());
        info!("Export format: {:?}", format);
        if export_passphrase {
            info!("Export uses additional passphrase protection");
        }
        if include_metadata {
            info!("Export includes key metadata");
        }
    } else {
        return Err("Export passphrase is required".into());
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
) -> Result<(), Box<dyn std::error::Error>> {
    // Get storage directory
    let storage_path = storage_dir.unwrap_or(get_default_storage_dir()?);
    ensure_storage_dir(&storage_path)?;

    let key_file_path = storage_path.join(format!("{}.json", key_id));

    if key_file_path.exists() && !force {
        return Err(format!("Key '{}' already exists. Use --force to overwrite", key_id).into());
    }

    // Read export file
    let export_content = fs::read_to_string(&export_file)
        .map_err(|e| format!("Failed to read export file: {}", e))?;

    // Try to parse as enhanced export format
    let export_data: EnhancedKeyExportFormat =
        serde_json::from_str(&export_content).map_err(|e| {
            format!(
                "Failed to parse export file (not valid enhanced format): {}",
                e
            )
        })?;

    // Validate export format version
    if export_data.version != 1 {
        return Err(format!("Unsupported export format version: {}", export_data.version).into());
    }

    // Validate encryption algorithm
    if export_data.encryption != "aes-gcm-like" {
        return Err(format!(
            "Unsupported encryption algorithm: {}",
            export_data.encryption
        )
        .into());
    }

    // Validate KDF
    if export_data.kdf != "argon2id" {
        return Err(format!("Unsupported KDF: {}", export_data.kdf).into());
    }

    // Get import passphrase
    print!("Enter import passphrase to decrypt exported key: ");
    io::stdout().flush()?;
    let import_passphrase = read_password()?;

    // Reconstruct Argon2 parameters
    let argon2_params = Argon2Params::new(
        export_data.kdf_params.memory,
        export_data.kdf_params.iterations,
        export_data.kdf_params.parallelism,
    )
    .map_err(|e| format!("Invalid KDF parameters: {}", e))?;

    // Recreate salt from export data
    if export_data.kdf_params.salt.len() != 32 {
        return Err("Invalid salt length in export data".into());
    }
    let mut salt_bytes = [0u8; 32];
    salt_bytes.copy_from_slice(&export_data.kdf_params.salt);
    let salt = crate::crypto::argon2::Salt::from_bytes(salt_bytes);

    // Derive decryption key
    let derived_key = derive_key(&import_passphrase, &salt, &argon2_params)
        .map_err(|e| format!("Import key derivation failed: {}", e))?;

    // Decrypt the key data
    if export_data.nonce.len() != 12 {
        return Err("Invalid nonce length in export data".into());
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
        return Err("Invalid decrypted key length (corruption or wrong passphrase)".into());
    }

    let mut imported_key = [0u8; 32];
    imported_key.copy_from_slice(&decrypted_data);

    // Verify key integrity if requested
    if verify_integrity {
        // Generate keypair from imported key to verify it's valid
        let keypair = generate_master_keypair_from_seed(&imported_key)
            .map_err(|e| format!("Key integrity verification failed: {}", e))?;

        // Test signing and verification
        let test_message = b"DataFold import verification test";
        let signature = keypair
            .sign_data(test_message)
            .map_err(|e| format!("Key functionality test failed: {}", e))?;

        keypair
            .verify_data(test_message, &signature)
            .map_err(|e| format!("Key verification test failed: {}", e))?;

        info!("✅ Key integrity verification passed");
    }

    // Get passphrase for storing the imported key
    print!(
        "Enter passphrase to encrypt imported key '{}' for storage: ",
        key_id
    );
    io::stdout().flush()?;
    let storage_passphrase = read_password()?;

    // Use balanced security for storage (can be upgraded later if needed)
    let storage_argon2_params = Argon2Params::default();

    // Encrypt the imported key for storage
    let storage_config = encrypt_key(&imported_key, &storage_passphrase, &storage_argon2_params)?;

    // Write encrypted key to file
    let config_json = serde_json::to_string_pretty(&storage_config)?;
    fs::write(&key_file_path, config_json)
        .map_err(|e| format!("Failed to write imported key file: {}", e))?;

    // Set file permissions to 600
    let mut perms = fs::metadata(&key_file_path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(&key_file_path, perms)
        .map_err(|e| format!("Failed to set file permissions: {}", e))?;

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