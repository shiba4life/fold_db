//! Cryptography-related command handlers
//! 
//! This module contains handlers for crypto initialization, status checking,
//! and validation commands.

use crate::cli::args::{CliSecurityLevel, CryptoMethod};
use crate::cli::utils::key_utils::get_secure_passphrase;
use crate::unified_crypto::config::{CryptoConfig, MasterKeyConfig};
use crate::datafold_node::crypto::{
    initialize_database_crypto, validate_crypto_config_for_init,
    CryptoInitStatus, get_crypto_init_status,
};
use crate::security_types::SecurityLevel;
use crate::{load_node_config, DataFoldNode};
use log::{error, info, warn};
use std::path::PathBuf;

/// Handle database crypto initialization
pub fn handle_crypto_init(
    method: CryptoMethod,
    security_level: CliSecurityLevel,
    force: bool,
    node: &mut DataFoldNode,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting database crypto initialization");

    // Get the actual SecurityLevel from the CLI wrapper
    let security_level: SecurityLevel = security_level.into();

    // Check if crypto is already initialized
    let fold_db = node.get_fold_db()?;
    let db_ops = fold_db.db_ops();
    let status = get_crypto_init_status(db_ops.clone(), None)
        .map_err(|e| format!("Failed to check crypto status: {}", e))?;

    if status.initialized && !force {
        info!("Database crypto is already initialized and completed");
        info!("Crypto initialization is healthy and verified");
        return Ok(());
    }
    
    if status.initialized && force {
        warn!("Forcing crypto re-initialization on already initialized database");
    }
    
    if let Some(ref error) = status.error_message {
        warn!("Previous crypto initialization failed: {}", error);
        if !force {
            return Err("Previous crypto initialization failed. Use --force to retry.".into());
        }
    }

    // Get passphrase if needed
    let passphrase = match method {
        CryptoMethod::Random => None,
        CryptoMethod::Passphrase => Some(get_secure_passphrase()?),
    };

    // Create crypto configuration
    let crypto_config = match method {
        CryptoMethod::Random => {
            info!("Using random key generation");
            {
                let mut config = CryptoConfig::for_security_level(security_level);
                config.master_key = MasterKeyConfig::Random;
                config
            }
        }
        CryptoMethod::Passphrase => {
            let passphrase = passphrase.unwrap(); // Safe since we just set it
            info!(
                "Using passphrase-based key derivation with {} security level",
                security_level.as_str()
            );
            {
                let mut config = CryptoConfig::for_security_level(security_level);
                config.master_key = MasterKeyConfig::Passphrase { passphrase };
                config
            }
        }
    };

    // Validate configuration
    validate_crypto_config_for_init(&crypto_config)
        .map_err(|e| format!("Crypto configuration validation failed: {}", e))?;
    info!("Crypto configuration validated successfully");

    // Perform initialization
    match initialize_database_crypto(db_ops, &crypto_config) {
        Ok(context) => {
            info!("✅ Database crypto initialization completed successfully!");
            info!("Derivation method: {}", context.derivation_method);
            info!("Master public key stored in database metadata");

            // Verify the initialization was successful
            let fold_db = node.get_fold_db()?;
            let final_status = get_crypto_init_status(fold_db.db_ops(), Some(&crypto_config))
                .map_err(|e| format!("Failed to verify crypto initialization: {}", e))?;

            if final_status.initialized {
                info!("✅ Crypto initialization verified successfully");
            } else {
                error!("❌ Crypto initialization verification failed");
                return Err("Crypto initialization verification failed".into());
            }
        }
        Err(e) => {
            error!("❌ Crypto initialization failed: {}", e);
            return Err(format!("Crypto initialization failed: {}", e).into());
        }
    }

    Ok(())
}

/// Handle checking database crypto status
pub fn handle_crypto_status(node: &mut DataFoldNode) -> Result<(), Box<dyn std::error::Error>> {
    info!("Checking database crypto initialization status");

    let fold_db = node.get_fold_db()?;
    let db_ops = fold_db.db_ops();
    let status = get_crypto_init_status(db_ops, None)
        .map_err(|e| format!("Failed to get crypto status: {}", e))?;

    info!("Crypto Status: {}", status.summary());

    if status.initialized {
        info!("  Initialized: ✅ Yes");
        info!(
            "  Algorithm: {}",
            status.algorithm.as_deref().unwrap_or("Unknown")
        );
        info!(
            "  Derivation Method: {}",
            status.derivation_method.as_deref().unwrap_or("Unknown")
        );
        info!("  Version: {}", status.version.unwrap_or(0));

        if let Some(created_at) = status.created_at {
            info!("  Created: {}", created_at.format("%Y-%m-%d %H:%M:%S UTC"));
        }

        match status.integrity_verified {
            Some(true) => info!("  Integrity: ✅ Verified"),
            Some(false) => warn!("  Integrity: ❌ Failed verification"),
            None => info!("  Integrity: ⚠️  Not checked"),
        }

        if status.is_healthy() {
            info!("🟢 Overall Status: Healthy");
        } else {
            warn!("🟡 Overall Status: Issues detected");
        }
    } else {
        info!("  Initialized: ❌ No");
        info!("🔴 Overall Status: Not initialized");
    }

    Ok(())
}

/// Handle validating crypto configuration
pub fn handle_crypto_validate(
    config_file: Option<PathBuf>,
    default_config_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = config_file
        .as_deref()
        .map(|p| p.to_str().unwrap_or(default_config_path))
        .unwrap_or(default_config_path);

    info!("Validating crypto configuration in: {}", config_path);

    // Load the node configuration
    let node_config = load_node_config(Some(config_path), None)?;

    // Check if crypto configuration exists
    if let Some(crypto_config) = &node_config.crypto {
        info!("Found crypto configuration");

        // Validate the configuration
        match validate_crypto_config_for_init(crypto_config) {
            Ok(()) => {
                info!("✅ Crypto configuration is valid");

                // Show configuration details
                info!("Configuration details:");
                info!("  Enabled: {}", crypto_config.enabled);

                if crypto_config.enabled {
                    match &crypto_config.master_key {
                        MasterKeyConfig::Random => {
                            info!("  Master Key: Random generation");
                        }
                        MasterKeyConfig::Passphrase { .. } => {
                            info!("  Master Key: Passphrase-based derivation");

                            info!("  Key Derivation: Argon2id parameters");
                            info!(
                                "    Memory Cost: {} KB",
                                crypto_config.key_derivation.argon2_params.memory_cost
                            );
                            info!(
                                "    Time Cost: {} iterations",
                                crypto_config.key_derivation.argon2_params.time_cost
                            );
                            info!(
                                "    Parallelism: {} threads",
                                crypto_config.key_derivation.argon2_params.parallelism
                            );
                        }
                        MasterKeyConfig::External { key_source } => {
                            info!("  Master Key: External source ({})", key_source);
                        }
                    }
                }
            }
            Err(e) => {
                error!("❌ Crypto configuration validation failed: {}", e);
                return Err(format!("Crypto configuration validation failed: {}", e).into());
            }
        }
    } else {
        info!("No crypto configuration found in node config");
        info!("ℹ️  Crypto will be disabled by default");
    }

    Ok(())
}