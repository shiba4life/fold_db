//! Cryptography command definitions
//! 
//! This module contains all CLI commands related to cryptographic operations,
//! including key generation, storage, rotation, and management.

use clap::Subcommand;
use std::path::PathBuf;
use super::cli_types::{CryptoMethod, CliSecurityLevel, KeyFormat, RotationMethod, ExportFormat};

/// Cryptography-related CLI commands
#[derive(Subcommand, Debug)]
pub enum CryptoCommands {
    /// Initialize database cryptography
    CryptoInit {
        /// Crypto initialization method
        #[arg(long, value_enum, default_value = "random")]
        method: CryptoMethod,
        /// Security level for key derivation (when using passphrase)
        #[arg(long, value_enum, default_value = "balanced")]
        security_level: CliSecurityLevel,
        /// Force re-initialization even if crypto is already initialized
        #[arg(long)]
        force: bool,
    },
    /// Check database crypto initialization status
    CryptoStatus {},
    /// Validate crypto configuration
    CryptoValidate {
        /// Path to configuration file to validate (defaults to CLI config)
        #[arg(long)]
        config_file: Option<PathBuf>,
    },
    /// Generate a new Ed25519 keypair for client-side key management
    GenerateKey {
        /// Output format for the generated keys
        #[arg(long, value_enum, default_value = "hex")]
        format: KeyFormat,
        /// Output file for private key (optional, defaults to stdout)
        #[arg(long)]
        private_key_file: Option<PathBuf>,
        /// Output file for public key (optional, defaults to stdout)
        #[arg(long)]
        public_key_file: Option<PathBuf>,
        /// Generate multiple keypairs (batch mode)
        #[arg(long, default_value = "1")]
        count: u32,
        /// Only output the public key (useful for registration)
        #[arg(long)]
        public_only: bool,
        /// Only output the private key (use with caution)
        #[arg(long)]
        private_only: bool,
    },
    /// Derive Ed25519 keypair from passphrase using secure key derivation
    DeriveKey {
        /// Output format for the derived keys
        #[arg(long, value_enum, default_value = "hex")]
        format: KeyFormat,
        /// Output file for private key (optional, defaults to stdout)
        #[arg(long)]
        private_key_file: Option<PathBuf>,
        /// Output file for public key (optional, defaults to stdout)
        #[arg(long)]
        public_key_file: Option<PathBuf>,
        /// Security level for key derivation
        #[arg(long, value_enum, default_value = "balanced")]
        security_level: CliSecurityLevel,
        /// Only output the public key
        #[arg(long)]
        public_only: bool,
        /// Only output the private key (use with caution)
        #[arg(long)]
        private_only: bool,
        /// Passphrase for key derivation (if not provided, will prompt)
        #[arg(long)]
        passphrase: Option<String>,
    },
    /// Extract public key from private key
    ExtractPublicKey {
        /// Private key input (hex, base64, or PEM format)
        #[arg(long)]
        private_key: Option<String>,
        /// Private key file path
        #[arg(long)]
        private_key_file: Option<PathBuf>,
        /// Output format for the public key
        #[arg(long, value_enum, default_value = "hex")]
        format: KeyFormat,
        /// Output file for public key (optional, defaults to stdout)
        #[arg(long)]
        output_file: Option<PathBuf>,
    },
    /// Verify that a keypair is valid and matches
    VerifyKey {
        /// Private key input (hex, base64, or PEM format)
        #[arg(long)]
        private_key: Option<String>,
        /// Private key file path
        #[arg(long)]
        private_key_file: Option<PathBuf>,
        /// Public key input (hex, base64, or PEM format)
        #[arg(long)]
        public_key: Option<String>,
        /// Public key file path
        #[arg(long)]
        public_key_file: Option<PathBuf>,
    },
    /// Store a private key securely in encrypted storage
    StoreKey {
        /// Key identifier/name for storage
        #[arg(long, required = true)]
        key_id: String,
        /// Private key input (hex, base64, or PEM format)
        #[arg(long)]
        private_key: Option<String>,
        /// Private key file path
        #[arg(long)]
        private_key_file: Option<PathBuf>,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Force overwrite if key already exists
        #[arg(long)]
        force: bool,
        /// Security level for encryption
        #[arg(long, value_enum, default_value = "balanced")]
        security_level: CliSecurityLevel,
        /// Passphrase for key encryption (if not provided, will prompt)
        #[arg(long)]
        passphrase: Option<String>,
    },
    /// Retrieve a private key from encrypted storage
    RetrieveKey {
        /// Key identifier/name to retrieve
        #[arg(long, required = true)]
        key_id: String,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Output format for the retrieved key
        #[arg(long, value_enum, default_value = "hex")]
        format: KeyFormat,
        /// Output file for retrieved key (optional, defaults to stdout)
        #[arg(long)]
        output_file: Option<PathBuf>,
        /// Only output the public key derived from stored private key
        #[arg(long)]
        public_only: bool,
    },
    /// Delete a key from encrypted storage
    DeleteKey {
        /// Key identifier/name to delete
        #[arg(long, required = true)]
        key_id: String,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Force deletion without confirmation
        #[arg(long)]
        force: bool,
    },
    /// List all stored keys
    ListKeys {
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Show detailed information about each key
        #[arg(long)]
        verbose: bool,
    },
    /// Derive a child key from a master key using HKDF (BLAKE3)
    DeriveFromMaster {
        /// Master key identifier to derive from
        #[arg(long, required = true)]
        master_key_id: String,
        /// Derivation context/info string
        #[arg(long, required = true)]
        context: String,
        /// Child key identifier for storage
        #[arg(long, required = true)]
        child_key_id: String,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Security level for child key encryption
        #[arg(long, value_enum, default_value = "balanced")]
        security_level: CliSecurityLevel,
        /// Output format for the derived key (if not stored)
        #[arg(long, value_enum, default_value = "hex")]
        format: KeyFormat,
        /// Only output the key, don't store it
        #[arg(long)]
        output_only: bool,
        /// Force overwrite if child key already exists
        #[arg(long)]
        force: bool,
    },
    /// Rotate a stored key to a new version
    RotateKey {
        /// Key identifier to rotate
        #[arg(long, required = true)]
        key_id: String,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Security level for the new key version
        #[arg(long, value_enum, default_value = "balanced")]
        security_level: CliSecurityLevel,
        /// Rotation method
        #[arg(long, value_enum, default_value = "regenerate")]
        method: RotationMethod,
        /// Keep previous version as backup
        #[arg(long)]
        keep_backup: bool,
        /// Force rotation without confirmation
        #[arg(long)]
        force: bool,
    },
    /// List all versions of a stored key
    ListKeyVersions {
        /// Key identifier to list versions for
        #[arg(long, required = true)]
        key_id: String,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Show detailed version information
        #[arg(long)]
        verbose: bool,
    },
    /// Create encrypted backup of a key
    BackupKey {
        /// Key identifier to backup
        #[arg(long, required = true)]
        key_id: String,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Backup file path
        #[arg(long, required = true)]
        backup_file: PathBuf,
        /// Additional passphrase for backup encryption (optional)
        #[arg(long)]
        backup_passphrase: bool,
    },
    /// Restore key from encrypted backup
    RestoreKey {
        /// Backup file path
        #[arg(long, required = true)]
        backup_file: PathBuf,
        /// Key identifier for restored key
        #[arg(long, required = true)]
        key_id: String,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Force overwrite if key already exists
        #[arg(long)]
        force: bool,
    },
    /// Export key with encrypted passphrase protection
    ExportKey {
        /// Key identifier to export
        #[arg(long, required = true)]
        key_id: String,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Export file path
        #[arg(long, required = true)]
        export_file: PathBuf,
        /// Export format (json or binary)
        #[arg(long, value_enum, default_value = "json")]
        format: ExportFormat,
        /// Use additional export passphrase for enhanced security
        #[arg(long)]
        export_passphrase: bool,
        /// Include key metadata in export
        #[arg(long)]
        include_metadata: bool,
    },
    /// Import key from encrypted export file
    ImportKey {
        /// Export file path
        #[arg(long, required = true)]
        export_file: PathBuf,
        /// Key identifier for imported key
        #[arg(long, required = true)]
        key_id: String,
        /// Storage directory (defaults to ~/.datafold/keys)
        #[arg(long)]
        storage_dir: Option<PathBuf>,
        /// Force overwrite if key already exists
        #[arg(long)]
        force: bool,
        /// Verify key integrity after import
        #[arg(long, default_value = "true")]
        verify_integrity: bool,
    },
}