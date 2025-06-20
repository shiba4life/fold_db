//! # Backup Security Cryptographic Operations
//!
//! This module provides high-level backup and recovery operations with strong cryptographic
//! protection. It handles encrypted backup creation, secure recovery workflows, and backup
//! integrity verification with comprehensive audit logging.

use crate::unified_crypto::{UnifiedCrypto, UnifiedCryptoResult, UnifiedCryptoError, CryptoAuditEvent};
use crate::unified_crypto::types::{EncryptedData, KeyId, Algorithm, HashAlgorithm};
use crate::unified_crypto::audit::CryptoAuditLogger;
use std::sync::Arc;
use std::time::SystemTime;
use serde::{Serialize, Deserialize};

/// Backup security policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSecurityPolicy {
    /// Default encryption algorithm for backups
    pub default_encryption: Algorithm,
    /// Enable backup compression
    pub enable_compression: bool,
    /// Maximum backup file size
    pub max_backup_size: u64,
    /// Backup retention period
    pub retention_period_days: u32,
    /// Enable backup verification
    pub enable_verification: bool,
}

impl Default for BackupSecurityPolicy {
    fn default() -> Self {
        Self {
            default_encryption: Algorithm::Aes256Gcm,
            enable_compression: true,
            max_backup_size: 10 * 1024 * 1024 * 1024, // 10GB
            retention_period_days: 90,
            enable_verification: true,
        }
    }
}

/// Encrypted backup metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedBackup {
    /// Backup identifier
    pub backup_id: String,
    /// Backup creation timestamp
    pub created_at: SystemTime,
    /// Encrypted backup data
    pub encrypted_data: EncryptedData,
    /// Backup metadata
    pub metadata: BackupMetadata,
    /// Backup integrity hash
    pub integrity_hash: Vec<u8>,
}

/// Backup metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    /// Original data size
    pub original_size: u64,
    /// Compressed size (if compression enabled)
    pub compressed_size: Option<u64>,
    /// Backup type
    pub backup_type: BackupType,
    /// Source description
    pub source_description: String,
    /// Backup version
    pub version: u32,
}

/// Types of backups
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BackupType {
    /// Full backup
    Full,
    /// Incremental backup
    Incremental,
    /// Differential backup
    Differential,
    /// Configuration backup
    Configuration,
}

/// Backup operations coordinator
pub struct BackupOperations {
    /// Reference to the unified crypto system
    crypto: Arc<UnifiedCrypto>,
    /// Backup security policy
    policy: BackupSecurityPolicy,
    /// Audit logger for backup operations
    audit_logger: Arc<CryptoAuditLogger>,
}

impl BackupOperations {
    /// Create new backup operations coordinator
    pub fn new(crypto: Arc<UnifiedCrypto>) -> UnifiedCryptoResult<Self> {
        let policy = BackupSecurityPolicy::default();
        let audit_logger = crypto.audit_logger().clone();

        audit_logger.log_crypto_event(CryptoAuditEvent::backup_operations_initialized())?;

        Ok(Self {
            crypto,
            policy,
            audit_logger: audit_logger,
        })
    }

    /// Create an encrypted backup
    ///
    /// # Arguments
    /// * `data` - Data to backup
    /// * `source_description` - Description of the backup source
    /// * `backup_type` - Type of backup
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<EncryptedBackup>` - Encrypted backup or error
    pub fn create_encrypted_backup(
        &self,
        data: &[u8],
        source_description: &str,
        backup_type: BackupType,
    ) -> UnifiedCryptoResult<EncryptedBackup> {
        // Validate backup size
        if data.len() as u64 > self.policy.max_backup_size {
            return Err(UnifiedCryptoError::InvalidInput {
                message: format!("Backup size {} exceeds maximum {}", data.len(), self.policy.max_backup_size),
            });
        }

        let backup_id = self.generate_backup_id()?;

        // Log backup creation start
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::backup_creation_start(&backup_id, data.len())
        )?;

        // Apply compression if enabled
        let (final_data, compressed_size) = if self.policy.enable_compression {
            // In production, implement actual compression
            (data.to_vec(), Some(data.len() as u64))
        } else {
            (data.to_vec(), None)
        };

        // Generate encryption key for backup
        let key_pair = self.crypto.generate_keypair()?;
        
        // Encrypt backup data
        let encrypted_data = self.crypto.encrypt(&final_data, &key_pair.public_key)?;

        // Calculate integrity hash
        let integrity_hash = self.crypto.hash(encrypted_data.ciphertext(), HashAlgorithm::Sha256)?;

        let backup = EncryptedBackup {
            backup_id: backup_id.clone(),
            created_at: SystemTime::now(),
            encrypted_data,
            metadata: BackupMetadata {
                original_size: data.len() as u64,
                compressed_size,
                backup_type,
                source_description: source_description.to_string(),
                version: 1,
            },
            integrity_hash,
        };

        // Log successful backup creation
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::backup_creation_success(&backup_id)
        )?;

        Ok(backup)
    }

    /// Restore data from an encrypted backup
    ///
    /// # Arguments
    /// * `backup` - Encrypted backup to restore
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<Vec<u8>>` - Restored data or error
    pub fn restore_from_backup(&self, backup: &EncryptedBackup) -> UnifiedCryptoResult<Vec<u8>> {
        // Log restore start
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::backup_restore_start(&backup.backup_id)
        )?;

        // Verify backup integrity if enabled
        if self.policy.enable_verification {
            self.verify_backup_integrity(backup)?;
        }

        // Get decryption key
        let key_id = backup.encrypted_data.key_id();
        let key_pair = self.crypto.key_manager().load_keypair(key_id)?;

        // Decrypt backup data
        let decrypted_data = self.crypto.decrypt(&backup.encrypted_data, &key_pair.private_key)?;

        // Apply decompression if needed
        let final_data = if backup.metadata.compressed_size.is_some() {
            // In production, implement actual decompression
            decrypted_data
        } else {
            decrypted_data
        };

        // Log successful restore
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::backup_restore_success(&backup.backup_id)
        )?;

        Ok(final_data)
    }

    /// Verify backup integrity
    fn verify_backup_integrity(&self, backup: &EncryptedBackup) -> UnifiedCryptoResult<()> {
        let calculated_hash = self.crypto.hash(backup.encrypted_data.ciphertext(), HashAlgorithm::Sha256)?;
        
        if calculated_hash != backup.integrity_hash {
            return Err(UnifiedCryptoError::IntegrityError {
                message: "Backup integrity verification failed".to_string(),
            });
        }

        Ok(())
    }

    /// Generate a unique backup identifier
    fn generate_backup_id(&self) -> UnifiedCryptoResult<String> {
        let random_bytes = self.crypto.primitives.generate_random_bytes(16)?;
        let backup_id_hash = self.crypto.hash(&random_bytes, HashAlgorithm::Sha256)?;
        Ok(format!("backup_{}", hex::encode(&backup_id_hash[..8])))
    }
}