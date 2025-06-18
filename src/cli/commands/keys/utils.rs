//! Common utilities for key management operations
//! 
//! This module contains shared helper functions for passphrase handling,
//! Argon2 parameters, file I/O, serialization, and validation.

use crate::cli::args::{CliSecurityLevel, KeyFormat};
use crate::cli::utils::key_utils::{
    format_key, get_secure_passphrase, output_key, parse_key_input
};
use crate::crypto::Argon2Params;
use crate::cli::commands::keys::error::{KeyError, KeyResult};
use rand::{rngs::OsRng, RngCore};
use rpassword::read_password;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// Convert CLI security level to Argon2 parameters
pub fn security_level_to_argon2(security_level: &CliSecurityLevel) -> Argon2Params {
    match security_level {
        CliSecurityLevel::Interactive => Argon2Params::interactive(),
        CliSecurityLevel::Balanced => Argon2Params::default(),
        CliSecurityLevel::Sensitive => Argon2Params::sensitive(),
    }
}

/// Get a passphrase securely with retry logic
pub fn get_passphrase_with_retry(prompt: &str) -> KeyResult<String> {
    print!("{}", prompt);
    io::stdout().flush().map_err(|e| KeyError::StorageError(format!("Failed to flush stdout: {}", e)))?;
    
    read_password().map_err(|e| KeyError::AuthenticationError(format!("Failed to read passphrase: {}", e)))
}

/// Get default passphrase for a given operation
pub fn get_operation_passphrase(_operation: &str, passphrase: Option<String>) -> KeyResult<String> {
    match passphrase {
        Some(p) => Ok(p),
        None => get_secure_passphrase().map_err(|e| KeyError::AuthenticationError(format!("Failed to get passphrase: {}", e))),
    }
}

/// Validate private key input and convert to bytes
pub fn validate_private_key_input(
    private_key: Option<String>,
    private_key_file: Option<PathBuf>,
) -> KeyResult<[u8; 32]> {
    let key_bytes = match (private_key, private_key_file) {
        (Some(key_str), None) => parse_key_input(&key_str, true)
            .map_err(|e| KeyError::InvalidKey(format!("Invalid private key format: {}", e)))?,
        (None, Some(file_path)) => {
            let content = fs::read_to_string(&file_path)
                .map_err(|e| KeyError::StorageError(format!("Failed to read private key file: {}", e)))?;
            parse_key_input(content.trim(), true)
                .map_err(|e| KeyError::InvalidKey(format!("Invalid private key in file: {}", e)))?
        }
        (Some(_), Some(_)) => {
            return Err(KeyError::ConfigurationError("Cannot specify both --private-key and --private-key-file".to_string()));
        }
        (None, None) => {
            return Err(KeyError::ConfigurationError("Must specify either --private-key or --private-key-file".to_string()));
        }
    };

    Ok(key_bytes)
}

/// Validate public key input and convert to bytes
pub fn validate_public_key_input(
    public_key: Option<String>,
    public_key_file: Option<PathBuf>,
) -> KeyResult<[u8; 32]> {
    let key_bytes = match (public_key, public_key_file) {
        (Some(key_str), None) => parse_key_input(&key_str, false)
            .map_err(|e| KeyError::InvalidKey(format!("Invalid public key format: {}", e)))?,
        (None, Some(file_path)) => {
            let content = fs::read_to_string(&file_path)
                .map_err(|e| KeyError::StorageError(format!("Failed to read public key file: {}", e)))?;
            parse_key_input(content.trim(), false)
                .map_err(|e| KeyError::InvalidKey(format!("Invalid public key in file: {}", e)))?
        }
        (Some(_), Some(_)) => {
            return Err(KeyError::ConfigurationError("Cannot specify both --public-key and --public-key-file".to_string()));
        }
        (None, None) => {
            return Err(KeyError::ConfigurationError("Must specify either --public-key or --public-key-file".to_string()));
        }
    };

    Ok(key_bytes)
}

/// Format and output a key using the specified format and output options
pub fn format_and_output_key(
    key_bytes: &[u8; 32],
    format: &KeyFormat,
    output_file: Option<&PathBuf>,
    key_type: &str,
    is_private: bool,
    show_info: bool,
) -> KeyResult<()> {
    let formatted_key = format_key(key_bytes, format, is_private)
        .map_err(|e| KeyError::InvalidKey(format!("Failed to format key: {}", e)))?;
    
    output_key(
        &formatted_key,
        output_file,
        key_type,
        0,
        1,
        show_info,
    ).map_err(|e| KeyError::StorageError(format!("Failed to output key: {}", e)))?;

    Ok(())
}

/// Format and output a key with specific index and total for batch operations
#[allow(clippy::too_many_arguments)]
pub fn format_and_output_key_with_index(
    key_bytes: &[u8; 32],
    format: &KeyFormat,
    output_file: Option<&PathBuf>,
    key_type: &str,
    is_private: bool,
    show_info: bool,
    index: usize,
    total: usize,
) -> KeyResult<()> {
    let formatted_key = format_key(key_bytes, format, is_private)
        .map_err(|e| KeyError::InvalidKey(format!("Failed to format key: {}", e)))?;
    
    output_key(
        &formatted_key,
        output_file,
        key_type,
        index as u32,
        total as u32,
        show_info,
    ).map_err(|e| KeyError::StorageError(format!("Failed to output key: {}", e)))?;

    Ok(())
}

/// Set secure file permissions (600 - owner read/write only)
pub fn set_secure_file_permissions(file_path: &PathBuf) -> KeyResult<()> {
    let mut perms = fs::metadata(file_path)
        .map_err(|e| KeyError::StorageError(format!("Failed to get file metadata: {}", e)))?
        .permissions();
    perms.set_mode(0o600);
    fs::set_permissions(file_path, perms)
        .map_err(|e| KeyError::StorageError(format!("Failed to set file permissions: {}", e)))?;
    
    Ok(())
}

/// Generate a secure nonce for cryptographic operations
pub fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

/// Confirm a dangerous operation with user input
pub fn confirm_operation(message: &str, force: bool) -> KeyResult<bool> {
    if force {
        return Ok(true);
    }

    print!("{} (y/N): ", message);
    io::stdout().flush()
        .map_err(|e| KeyError::StorageError(format!("Failed to flush stdout: {}", e)))?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)
        .map_err(|e| KeyError::StorageError(format!("Failed to read input: {}", e)))?;
    
    Ok(input.trim().to_lowercase() == "y")
}

/// Check if a key file exists in storage
pub fn key_exists_in_storage(key_id: &str, storage_path: &Path) -> bool {
    storage_path.join(format!("{}.json", key_id)).exists()
}

/// Validate that a key ID is safe for filesystem use
pub fn validate_key_id(key_id: &str) -> KeyResult<()> {
    if key_id.is_empty() {
        return Err(KeyError::ConfigurationError("Key ID cannot be empty".to_string()));
    }

    // Check for invalid characters that could cause filesystem issues
    if key_id.contains('/') || key_id.contains('\\') || key_id.contains('.') || key_id.contains(' ') {
        return Err(KeyError::ConfigurationError(
            "Key ID cannot contain path separators, dots, or spaces".to_string()
        ));
    }

    // Check length constraints
    if key_id.len() > 100 {
        return Err(KeyError::ConfigurationError("Key ID too long (max 100 characters)".to_string()));
    }

    Ok(())
}

/// Clear sensitive data from memory (best effort)
pub fn clear_sensitive_data<T: Default>(data: &mut T) {
    *data = T::default();
}