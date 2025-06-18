//! Crypto and key management command handling for DataFold CLI
//! 
//! This module handles all cryptographic operations including key generation,
//! storage, rotation, backup/restore, and crypto initialization/validation.

use crate::cli::args::Commands;
use crate::cli::commands::{crypto, keys};
use crate::DataFoldNode;

/// Handle crypto and key management commands that don't require a node
pub async fn handle_crypto_command_standalone(command: &Commands, config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::CryptoValidate { config_file } => {
            crypto::handle_crypto_validate(config_file.clone(), config_path)
        }
        Commands::GenerateKey {
            format,
            private_key_file,
            public_key_file,
            count,
            public_only,
            private_only,
        } => {
            keys::handle_generate_key(
                format.clone(),
                private_key_file.clone(),
                public_key_file.clone(),
                *count,
                *public_only,
                *private_only,
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        }
        Commands::DeriveKey {
            format,
            private_key_file,
            public_key_file,
            security_level,
            public_only,
            private_only,
            passphrase,
        } => {
            keys::handle_derive_key(
                format.clone(),
                private_key_file.clone(),
                public_key_file.clone(),
                security_level.clone(),
                *public_only,
                *private_only,
                passphrase.clone(),
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        }
        Commands::ExtractPublicKey {
            private_key,
            private_key_file,
            format,
            output_file,
        } => {
            keys::handle_extract_public_key(
                private_key.clone(),
                private_key_file.clone(),
                format.clone(),
                output_file.clone(),
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        }
        Commands::VerifyKey {
            private_key,
            private_key_file,
            public_key,
            public_key_file,
        } => {
            keys::handle_verify_key(
                private_key.clone(),
                private_key_file.clone(),
                public_key.clone(),
                public_key_file.clone(),
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        }
        Commands::StoreKey {
            key_id,
            private_key,
            private_key_file,
            storage_dir,
            force,
            security_level,
            passphrase,
        } => {
            keys::handle_store_key(
                key_id.clone(),
                private_key.clone(),
                private_key_file.clone(),
                storage_dir.clone(),
                *force,
                security_level.clone(),
                passphrase.clone(),
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        }
        Commands::RetrieveKey {
            key_id,
            storage_dir,
            format,
            output_file,
            public_only,
        } => {
            keys::handle_retrieve_key(
                key_id.clone(),
                storage_dir.clone(),
                format.clone(),
                output_file.clone(),
                *public_only,
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        }
        Commands::DeleteKey {
            key_id,
            storage_dir,
            force,
        } => {
            keys::handle_delete_key(key_id.clone(), storage_dir.clone(), *force)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        }
        Commands::ListKeys {
            storage_dir,
            verbose,
        } => {
            keys::handle_list_keys(storage_dir.clone(), *verbose)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        }
        Commands::DeriveFromMaster {
            master_key_id,
            context,
            child_key_id,
            storage_dir,
            security_level,
            format,
            output_only,
            force,
        } => {
            keys::handle_derive_from_master(
                master_key_id.clone(),
                context.clone(),
                child_key_id.clone(),
                storage_dir.clone(),
                security_level.clone(),
                format.clone(),
                *output_only,
                *force,
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        }
        Commands::RotateKey {
            key_id,
            storage_dir,
            security_level,
            method,
            keep_backup,
            force,
        } => {
            keys::handle_rotate_key(
                key_id.clone(),
                storage_dir.clone(),
                security_level.clone(),
                method.clone(),
                *keep_backup,
                *force,
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        }
        Commands::ListKeyVersions {
            key_id,
            storage_dir,
            verbose,
        } => {
            keys::handle_list_key_versions(key_id.clone(), storage_dir.clone(), *verbose)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        }
        Commands::BackupKey {
            key_id,
            storage_dir,
            backup_file,
            backup_passphrase,
        } => {
            keys::handle_backup_key(
                key_id.clone(),
                storage_dir.clone(),
                backup_file.clone(),
                *backup_passphrase,
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        }
        Commands::RestoreKey {
            backup_file,
            key_id,
            storage_dir,
            force,
        } => {
            keys::handle_restore_key(
                backup_file.clone(),
                key_id.clone(),
                storage_dir.clone(),
                *force,
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        }
        Commands::ExportKey {
            key_id,
            storage_dir,
            export_file,
            format,
            export_passphrase,
            include_metadata,
        } => {
            keys::handle_export_key(
                key_id.clone(),
                storage_dir.clone(),
                export_file.clone(),
                format.clone(),
                *export_passphrase,
                *include_metadata,
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        }
        Commands::ImportKey {
            export_file,
            key_id,
            storage_dir,
            force,
            verify_integrity,
        } => {
            keys::handle_import_key(
                export_file.clone(),
                key_id.clone(),
                storage_dir.clone(),
                *force,
                *verify_integrity,
            ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        }
        _ => {
            Err("Command is not a standalone crypto command".into())
        }
    }
}

/// Handle crypto commands that require a node
pub fn handle_crypto_command_with_node(command: &Commands, node: &mut DataFoldNode) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::CryptoInit {
            method,
            security_level,
            force,
        } => {
            crypto::handle_crypto_init(method.clone(), security_level.clone(), *force, node)?;
            Ok(())
        }
        Commands::CryptoStatus {} => {
            crypto::handle_crypto_status(node)?;
            Ok(())
        }
        _ => {
            Err("Command is not a node-based crypto command".into())
        }
    }
}

/// Check if a command is a standalone crypto command (doesn't need node)
pub fn is_standalone_crypto_command(command: &Commands) -> bool {
    matches!(command,
        Commands::CryptoValidate { .. }
        | Commands::GenerateKey { .. }
        | Commands::DeriveKey { .. }
        | Commands::ExtractPublicKey { .. }
        | Commands::VerifyKey { .. }
        | Commands::StoreKey { .. }
        | Commands::RetrieveKey { .. }
        | Commands::DeleteKey { .. }
        | Commands::ListKeys { .. }
        | Commands::DeriveFromMaster { .. }
        | Commands::RotateKey { .. }
        | Commands::ListKeyVersions { .. }
        | Commands::BackupKey { .. }
        | Commands::RestoreKey { .. }
        | Commands::ExportKey { .. }
        | Commands::ImportKey { .. }
    )
}

/// Check if a command is a node-based crypto command
pub fn is_node_crypto_command(command: &Commands) -> bool {
    matches!(command,
        Commands::CryptoInit { .. }
        | Commands::CryptoStatus { .. }
    )
}