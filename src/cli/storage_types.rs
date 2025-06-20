//! Storage and configuration type definitions
//! 
//! This module contains all storage-related types, backup formats,
//! and API communication structures for the DataFold CLI.

use crate::unified_crypto::config::Argon2Params;

/// Secure key storage configuration
#[derive(Debug, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

/// Key versioning metadata for rotation tracking
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct KeyVersionMetadata {
    /// Version number (starts at 1)
    pub version: u32,
    /// Previous version number (None for initial version)
    pub previous_version: Option<u32>,
    /// Creation timestamp for this version
    pub created_at: String,
    /// Derivation method used for this version
    pub derivation_method: String,
    /// Salt used for key derivation (32 bytes)
    pub salt: [u8; 32],
    /// Argon2 parameters used for this version
    pub argon2_params: StoredArgon2Params,
}

/// Enhanced key storage with versioning support
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct VersionedKeyStorageConfig {
    /// Current active version metadata
    pub current_version: KeyVersionMetadata,
    /// Encrypted key data for current version
    pub encrypted_key: Vec<u8>,
    /// Nonce used for encryption (12 bytes for AES-GCM)
    pub nonce: [u8; 12],
    /// Version of storage format
    pub version: u32,
}

/// Key backup format for secure export/import
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct KeyBackupFormat {
    /// Encrypted key storage configuration
    pub key_config: KeyStorageConfig,
    /// Additional backup metadata
    pub backup_metadata: BackupMetadata,
    /// Backup format version
    pub backup_version: u32,
    /// Optional additional encryption layer
    pub additional_encryption: Option<AdditionalEncryption>,
}

/// Backup metadata
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BackupMetadata {
    /// Original key identifier
    pub key_id: String,
    /// Backup creation timestamp
    pub created_at: String,
    /// Source system identifier
    pub source_system: String,
    /// Backup description
    pub description: Option<String>,
}

/// Additional encryption layer for enhanced security
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AdditionalEncryption {
    /// Encrypted data
    pub encrypted_data: Vec<u8>,
    /// Nonce for additional encryption (12 bytes)
    pub nonce: [u8; 12],
    /// Salt for additional key derivation (32 bytes)
    pub salt: [u8; 32],
    /// Argon2 parameters for additional encryption
    pub argon2_params: StoredArgon2Params,
}

/// Enhanced key export format with metadata
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EnhancedKeyExportFormat {
    /// Key data (encrypted)
    pub key_data: Vec<u8>,
    /// Export metadata
    pub metadata: ExportKeyMetadata,
    /// Enhanced KDF parameters
    pub kdf_params: EnhancedKdfParams,
    /// Export format version
    pub version: u32,
}

/// Enhanced KDF parameters for export
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EnhancedKdfParams {
    /// Primary salt (32 bytes)
    pub primary_salt: [u8; 32],
    /// Secondary salt for export passphrase (32 bytes)
    pub export_salt: [u8; 32],
    /// Argon2 parameters for primary encryption
    pub primary_argon2: StoredArgon2Params,
    /// Argon2 parameters for export encryption
    pub export_argon2: StoredArgon2Params,
}

/// Export key metadata
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ExportKeyMetadata {
    /// Original key identifier
    pub key_id: String,
    /// Export timestamp
    pub exported_at: String,
    /// Key version (if versioned)
    pub key_version: Option<u32>,
    /// Export description
    pub description: Option<String>,
}

/// API response wrapper
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

/// API error information
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Public key registration request
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PublicKeyRegistrationRequest {
    pub client_id: String,
    pub public_key: String,
    pub user_id: Option<String>,
    pub key_name: Option<String>,
}

/// Public key registration response
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PublicKeyRegistrationResponse {
    pub client_id: String,
    pub public_key: String,
    pub registered_at: String,
    pub status: String,
    pub key_id: Option<String>,
}

/// Public key status response
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PublicKeyStatusResponse {
    pub client_id: String,
    pub registered: bool,
    pub public_key: Option<String>,
    pub registered_at: Option<String>,
    pub status: String,
}

/// Signature verification request
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SignatureVerificationRequest {
    pub client_id: String,
    pub message: String,
    pub signature: String,
    pub message_encoding: String,
}

/// Signature verification response
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SignatureVerificationResponse {
    pub valid: bool,
    pub message: String,
    pub verified_at: String,
}