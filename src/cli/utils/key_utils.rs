//! Key utility functions for the DataFold CLI
//! 
//! This module contains utility functions for key management including
//! storage, retrieval, encryption, decryption, and directory management.

use crate::cli::args::KeyFormat;
use crate::crypto::{derive_key, generate_salt, Argon2Params, MasterKeyPair};
use base64::{engine::general_purpose, Engine as _};
use log::{error, info};
use rand::{rngs::OsRng, RngCore};
use rpassword::read_password;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// Secure key storage configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyStorageConfig {
    /// Encrypted key data
    pub encrypted_key: Vec<u8>,
    /// Nonce used for encryption (12 bytes for AES-GCM)
    pub nonce: [u8; 12],
    /// Salt used for key derivation (32 bytes)
    pub salt: [u8; 32],
    /// Argon2 parameters used for key derivation
    pub argon2_params: StoredArgon2Params,
    /// Timestamp when key was stored
    pub created_at: String,
    /// Version of storage format
    pub version: u32,
}

/// Simplified Argon2 parameters for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredArgon2Params {
    pub memory_cost: u32,
    pub time_cost: u32,
    pub parallelism: u32,
}

impl From<&Argon2Params> for StoredArgon2Params {
    fn from(params: &Argon2Params) -> Self {
        Self {
            memory_cost: params.memory_cost,
            time_cost: params.time_cost,
            parallelism: params.parallelism,
        }
    }
}

impl From<StoredArgon2Params> for Argon2Params {
    fn from(val: StoredArgon2Params) -> Self {
        Argon2Params::new(val.memory_cost, val.time_cost, val.parallelism).unwrap_or_default()
    }
}

/// Get the default storage directory for keys
pub fn get_default_storage_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home_dir = dirs::home_dir().ok_or("Unable to determine home directory")?;
    Ok(home_dir.join(".datafold").join("keys"))
}

/// Ensure storage directory exists with proper permissions
pub fn ensure_storage_dir(dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if !dir.exists() {
        fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create storage directory: {}", e))?;
    }

    // Set directory permissions to 700 (owner read/write/execute only)
    let mut perms = fs::metadata(dir)?.permissions();
    perms.set_mode(0o700);
    fs::set_permissions(dir, perms)
        .map_err(|e| format!("Failed to set directory permissions: {}", e))?;

    Ok(())
}

/// Encrypt a private key with a passphrase using BLAKE3-based stream cipher
pub fn encrypt_key(
    private_key: &[u8; 32],
    passphrase: &str,
    argon2_params: &Argon2Params,
) -> Result<KeyStorageConfig, Box<dyn std::error::Error>> {
    // Generate salt and nonce
    let salt = generate_salt();
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);

    // Derive encryption key from passphrase
    let derived_key = derive_key(passphrase, &salt, argon2_params)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    // Use BLAKE3 to generate a keystream for encryption
    let mut hasher = blake3::Hasher::new();
    hasher.update(derived_key.as_bytes());
    hasher.update(&nonce);
    let keystream = hasher.finalize();

    // XOR encrypt the private key
    let mut encrypted_key = [0u8; 32];
    for i in 0..32 {
        encrypted_key[i] = private_key[i] ^ keystream.as_bytes()[i];
    }

    Ok(KeyStorageConfig {
        encrypted_key: encrypted_key.to_vec(),
        nonce,
        salt: *salt.as_bytes(),
        argon2_params: argon2_params.into(),
        created_at: chrono::Utc::now().to_rfc3339(),
        version: 1,
    })
}

/// Decrypt a private key from storage
pub fn decrypt_key(
    config: &KeyStorageConfig,
    passphrase: &str,
) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    // Reconstruct Argon2 params
    let argon2_params: Argon2Params = config.argon2_params.clone().into();

    // Create Salt from stored bytes
    let salt = crate::crypto::argon2::Salt::from_bytes(config.salt);

    // Derive decryption key from passphrase
    let derived_key = derive_key(passphrase, &salt, &argon2_params)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    // Use BLAKE3 to generate the same keystream
    let mut hasher = blake3::Hasher::new();
    hasher.update(derived_key.as_bytes());
    hasher.update(&config.nonce);
    let keystream = hasher.finalize();

    // XOR decrypt the private key
    if config.encrypted_key.len() != 32 {
        return Err("Invalid encrypted key length".into());
    }

    let mut decrypted_key = [0u8; 32];
    for (i, item) in decrypted_key.iter_mut().enumerate() {
        *item = config.encrypted_key[i] ^ keystream.as_bytes()[i];
    }

    Ok(decrypted_key)
}

/// Get secure passphrase with confirmation
pub fn get_secure_passphrase() -> Result<String, Box<dyn std::error::Error>> {
    loop {
        print!("Enter passphrase for master key derivation: ");
        io::stdout().flush()?;

        let passphrase = read_password()?;

        if passphrase.len() < 6 {
            error!("Passphrase must be at least 6 characters long");
            continue;
        }

        if passphrase.len() > 1024 {
            error!("Passphrase is too long (maximum 1024 characters)");
            continue;
        }

        // Confirm passphrase
        print!("Confirm passphrase: ");
        io::stdout().flush()?;

        let confirmation = read_password()?;

        if passphrase != confirmation {
            error!("Passphrases do not match. Please try again.");
            continue;
        }

        // Clear confirmation from memory
        drop(confirmation);

        info!("✅ Passphrase accepted");
        return Ok(passphrase);
    }
}

/// Format key bytes according to the specified format
pub fn format_key(
    key_bytes: &[u8],
    format: &KeyFormat,
    is_private: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    match format {
        KeyFormat::Hex => Ok(hex::encode(key_bytes)),
        KeyFormat::Base64 => Ok(general_purpose::STANDARD.encode(key_bytes)),
        KeyFormat::Raw => Ok(String::from_utf8(key_bytes.to_vec())
            .unwrap_or_else(|_| format!("Binary data: {} bytes", key_bytes.len()))),
        KeyFormat::Pem => {
            if is_private {
                // For private keys, we'll use a simple PEM-like format
                // In a production system, you'd want proper PKCS#8 encoding
                let base64_content = general_purpose::STANDARD.encode(key_bytes);
                Ok(format!(
                    "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----",
                    base64_content
                ))
            } else {
                // For public keys, use SubjectPublicKeyInfo format
                let base64_content = general_purpose::STANDARD.encode(key_bytes);
                Ok(format!(
                    "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
                    base64_content
                ))
            }
        }
    }
}

/// Output formatted key to file or stdout
pub fn output_key(
    formatted_key: &str,
    output_file: Option<&PathBuf>,
    key_type: &str,
    index: u32,
    total: u32,
    include_comments: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match output_file {
        Some(file_path) => {
            let actual_path = if total > 1 {
                // Add index to filename for batch generation
                let stem = file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("key");
                let extension = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
                let parent = file_path.parent().unwrap_or(std::path::Path::new("."));

                if extension.is_empty() {
                    parent.join(format!("{}_{}", stem, index + 1))
                } else {
                    parent.join(format!("{}_{}.{}", stem, index + 1, extension))
                }
            } else {
                file_path.clone()
            };

            fs::write(&actual_path, formatted_key)
                .map_err(|e| format!("Failed to write {} key to file: {}", key_type, e))?;
            info!("✅ {} key written to: {}", key_type, actual_path.display());
        }
        None => {
            if include_comments {
                if total > 1 {
                    println!("# {} key {} of {}", key_type, index + 1, total);
                } else {
                    println!("# {} key", key_type);
                }
            }
            println!("{}", formatted_key);
        }
    }
    Ok(())
}

/// Parse key input from string (supports hex, base64, and PEM formats)
pub fn parse_key_input(input: &str, _is_private: bool) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let trimmed = input.trim();

    // Try to parse as PEM first
    if trimmed.starts_with("-----BEGIN") && trimmed.ends_with("-----") {
        let lines: Vec<&str> = trimmed.lines().collect();
        if lines.len() >= 3 {
            let base64_content = lines[1..lines.len() - 1].join("");
            let decoded = general_purpose::STANDARD
                .decode(&base64_content)
                .map_err(|e| format!("Invalid base64 in PEM: {}", e))?;

            if decoded.len() == 32 {
                let mut key_bytes = [0u8; 32];
                key_bytes.copy_from_slice(&decoded);
                return Ok(key_bytes);
            }
        }
        return Err("Invalid PEM format or wrong key size".into());
    }

    // Try hex format
    if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        let decoded = hex::decode(trimmed).map_err(|e| format!("Invalid hex: {}", e))?;
        if decoded.len() == 32 {
            let mut key_bytes = [0u8; 32];
            key_bytes.copy_from_slice(&decoded);
            return Ok(key_bytes);
        }
    }

    // Try base64 format
    if let Ok(decoded) = general_purpose::STANDARD.decode(trimmed) {
        if decoded.len() == 32 {
            let mut key_bytes = [0u8; 32];
            key_bytes.copy_from_slice(&decoded);
            return Ok(key_bytes);
        }
    }

    let expected_size = 32; // Ed25519 keys are always 32 bytes
    Err(format!(
        "Unable to parse key: expected {} bytes in hex, base64, or PEM format",
        expected_size
    )
    .into())
}

/// Internal helper for retrieving keys without authentication
#[allow(dead_code)]
pub fn handle_retrieve_key_internal(
    key_id: &str,
    storage_dir: &Path,
    public_only: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let key_file = storage_dir.join(format!("{}.json", key_id));

    if !key_file.exists() {
        return Err(format!("Key '{}' not found in storage", key_id).into());
    }

    let content = fs::read_to_string(&key_file)?;
    let key_config: KeyStorageConfig = serde_json::from_str(&content)?;

    // Prompt for passphrase
    print!("Enter passphrase for key '{}': ", key_id);
    io::stdout().flush()?;
    let passphrase = read_password()?;

    // Decrypt the key
    let decrypted_bytes = decrypt_key(&key_config, &passphrase)?;

    if public_only {
        // Extract public key from private key
        let keypair = MasterKeyPair::from_secret_bytes(&decrypted_bytes)
            .map_err(|e| format!("Failed to parse private key: {}", e))?;
        let public_key_bytes = keypair.public_key_bytes();
        Ok(general_purpose::STANDARD.encode(public_key_bytes))
    } else {
        Ok(general_purpose::STANDARD.encode(decrypted_bytes))
    }
}