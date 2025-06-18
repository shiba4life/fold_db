//! Key extraction and verification operations
//! 
//! This module handles extracting public keys from private keys and
//! verifying that keypairs are valid and functional.

use crate::cli::args::KeyFormat;
use crate::cli::commands::keys::error::{KeyError, KeyResult};
use crate::cli::commands::keys::utils::{
    format_and_output_key, validate_private_key_input, validate_public_key_input,
};
use crate::crypto::ed25519::{generate_master_keypair_from_seed};
use log::info;
use std::path::PathBuf;

/// Handle extracting public key from private key
pub fn handle_extract_public_key(
    private_key: Option<String>,
    private_key_file: Option<PathBuf>,
    format: KeyFormat,
    output_file: Option<PathBuf>,
) -> KeyResult<()> {
    // Get private key bytes using utility function
    let private_key_bytes = validate_private_key_input(private_key, private_key_file)?;

    // Create keypair from private key
    let keypair = generate_master_keypair_from_seed(&private_key_bytes)
        .map_err(|e| KeyError::InvalidKey(format!("Invalid private key: {}", e)))?;

    let public_key_bytes = keypair.public_key_bytes();

    // Format and output the public key
    format_and_output_key(
        &public_key_bytes,
        &format,
        output_file.as_ref(),
        "public",
        false,
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
) -> KeyResult<()> {
    // Get private key bytes using utility function
    let private_key_bytes = validate_private_key_input(private_key, private_key_file)?;

    // Get public key bytes using utility function
    let public_key_bytes = validate_public_key_input(public_key, public_key_file)?;

    // Create keypair from private key
    let keypair = generate_master_keypair_from_seed(&private_key_bytes)
        .map_err(|e| KeyError::InvalidKey(format!("Invalid private key: {}", e)))?;

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
        return Err(KeyError::ValidationError("Keypair verification failed".to_string()));
    }

    // Test signing and verification to ensure the keypair is fully functional
    let test_message = b"DataFold Ed25519 keypair verification test";
    let signature = keypair
        .sign_data(test_message)
        .map_err(|e| KeyError::CryptographicError(format!("Failed to sign test message: {}", e)))?;

    keypair
        .verify_data(test_message, &signature)
        .map_err(|e| KeyError::CryptographicError(format!("Failed to verify test signature: {}", e)))?;

    println!("✅ Functional verification successful: keypair can sign and verify");
    info!("✅ Functional verification successful: keypair can sign and verify");

    Ok(())
}

/// Verify the integrity of a key by attempting to create a keypair and test functionality
pub fn verify_key_integrity(key_bytes: &[u8; 32]) -> KeyResult<()> {
    // Generate keypair from key to verify it's valid
    let keypair = generate_master_keypair_from_seed(key_bytes)
        .map_err(|e| KeyError::ValidationError(format!("Key integrity verification failed: {}", e)))?;

    // Test signing and verification
    let test_message = b"DataFold key integrity verification test";
    let signature = keypair
        .sign_data(test_message)
        .map_err(|e| KeyError::ValidationError(format!("Key functionality test failed: {}", e)))?;

    keypair
        .verify_data(test_message, &signature)
        .map_err(|e| KeyError::ValidationError(format!("Key verification test failed: {}", e)))?;

    Ok(())
}