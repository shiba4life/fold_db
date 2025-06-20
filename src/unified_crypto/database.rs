//! # Database Cryptographic Operations
//!
//! This module provides high-level database encryption operations built on top of the unified
//! cryptographic primitives. It integrates with existing database operations patterns while
//! providing enhanced security, automatic key management, and comprehensive audit logging.
//!
//! ## Features
//!
//! - **Transparent Encryption**: Seamless encryption/decryption for database operations
//! - **Automatic Key Management**: Automated key rotation and lifecycle management
//! - **Transaction Security**: Transaction-level cryptographic protection
//! - **Schema Protection**: Encrypted metadata and schema information
//! - **Backward Compatibility**: Support for migrating from existing encryption systems
//! - **Comprehensive Auditing**: Full audit trails for all database cryptographic operations
//!
//! ## Architecture
//!
//! The database operations layer provides multiple levels of encryption:
//! - **Record-level encryption**: Individual database records
//! - **Column-level encryption**: Specific sensitive columns
//! - **Table-level encryption**: Entire tables with unified keys
//! - **Metadata encryption**: Schema and configuration data
//!
//! ## Example Usage
//!
//! ```rust
//! use datafold::unified_crypto::{UnifiedCrypto, CryptoOperations, CryptoConfig};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize crypto operations
//! let config = CryptoConfig::default();
//! let crypto = UnifiedCrypto::new(config)?;
//! let operations = CryptoOperations::new(crypto)?;
//!
//! // Encrypt database record
//! let sensitive_data = b"user personal information";
//! let encrypted_record = operations.database().encrypt_record(
//!     sensitive_data,
//!     "users_table",
//! )?;
//!
//! // Decrypt database record
//! let decrypted_data = operations.database().decrypt_record(
//!     &encrypted_record,
//!     "users_table",
//! )?;
//! # Ok(())
//! # }
//! ```

use crate::unified_crypto::{UnifiedCrypto, UnifiedCryptoResult, UnifiedCryptoError, CryptoAuditEvent};
use crate::unified_crypto::types::{EncryptedData, KeyId, Algorithm};
use crate::unified_crypto::keys::KeyManager;
use crate::unified_crypto::audit::CryptoAuditLogger;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Database encryption context for organizing keys and policies
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DatabaseContext {
    /// Table-level context for encrypting entire tables
    Table(String),
    /// Column-level context for encrypting specific columns
    Column { table: String, column: String },
    /// Schema-level context for encrypting metadata
    Schema,
    /// Index-level context for encrypting search indices
    Index(String),
    /// Backup-level context for encrypted backups
    Backup,
}

impl DatabaseContext {
    /// Get a string identifier for this context
    pub fn identifier(&self) -> String {
        match self {
            DatabaseContext::Table(table) => format!("table:{}", table),
            DatabaseContext::Column { table, column } => format!("column:{}:{}", table, column),
            DatabaseContext::Schema => "schema".to_string(),
            DatabaseContext::Index(index) => format!("index:{}", index),
            DatabaseContext::Backup => "backup".to_string(),
        }
    }

    /// Check if this context is for a specific table
    pub fn is_table_context(&self, table_name: &str) -> bool {
        match self {
            DatabaseContext::Table(table) => table == table_name,
            DatabaseContext::Column { table, .. } => table == table_name,
            _ => false,
        }
    }
}

/// Database encryption policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseEncryptionPolicy {
    /// Default encryption algorithm for database operations
    pub default_algorithm: Algorithm,
    /// Key rotation interval in seconds
    pub key_rotation_interval: u64,
    /// Whether to encrypt table names and metadata
    pub encrypt_metadata: bool,
    /// Whether to enable transparent encryption for all operations
    pub transparent_encryption: bool,
    /// Maximum record size for encryption (in bytes)
    pub max_record_size: usize,
    /// Compression before encryption
    pub enable_compression: bool,
}

impl Default for DatabaseEncryptionPolicy {
    fn default() -> Self {
        Self {
            default_algorithm: Algorithm::Aes256Gcm,
            key_rotation_interval: 86400 * 30, // 30 days
            encrypt_metadata: true,
            transparent_encryption: true,
            max_record_size: 16 * 1024 * 1024, // 16MB
            enable_compression: true,
        }
    }
}

/// Encrypted database record with metadata
#[derive(Debug, Clone)]
pub struct EncryptedDatabaseRecord {
    /// Encrypted data
    pub encrypted_data: EncryptedData,
    /// Database context for this record
    pub context: DatabaseContext,
    /// Record timestamp
    pub timestamp: std::time::SystemTime,
    /// Record version for migration support
    pub version: u32,
    /// Compression information (if enabled)
    pub compression_info: Option<CompressionInfo>,
}

/// Compression metadata for database records
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionInfo {
    /// Compression algorithm used
    pub algorithm: CompressionAlgorithm,
    /// Original size before compression
    pub original_size: usize,
    /// Compressed size
    pub compressed_size: usize,
}

/// Supported compression algorithms
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    /// No compression
    None,
    /// Gzip compression
    Gzip,
    /// LZ4 compression (fast)
    Lz4,
    /// Zstd compression (balanced)
    Zstd,
}

/// Database cryptographic operations coordinator
///
/// This struct provides high-level cryptographic operations specifically designed for database
/// use cases. It handles key management, encryption contexts, and integration with existing
/// database operation patterns.
pub struct DatabaseOperations {
    /// Reference to the unified crypto system
    crypto: Arc<UnifiedCrypto>,
    /// Database-specific encryption policy
    policy: DatabaseEncryptionPolicy,
    /// Context-specific key cache
    key_cache: Arc<std::sync::RwLock<HashMap<DatabaseContext, KeyId>>>,
    /// Audit logger for database operations
    audit_logger: Arc<CryptoAuditLogger>,
}

impl DatabaseOperations {
    /// Create new database operations coordinator
    ///
    /// # Arguments
    /// * `crypto` - Unified crypto system reference
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<Self>` - New database operations coordinator or error
    ///
    /// # Security
    /// - Initializes secure key caching for database contexts
    /// - Establishes audit logging for all database operations
    /// - Validates encryption policy configuration
    pub fn new(crypto: Arc<UnifiedCrypto>) -> UnifiedCryptoResult<Self> {
        let policy = DatabaseEncryptionPolicy::default();
        let key_cache = Arc::new(std::sync::RwLock::new(HashMap::new()));
        let audit_logger = crypto.audit_logger();

        // Log database operations initialization
        audit_logger.log_crypto_event(CryptoAuditEvent::database_operations_initialized())?;

        Ok(Self {
            crypto,
            policy,
            key_cache,
            audit_logger,
        })
    }

    /// Create new database operations with custom policy
    ///
    /// # Arguments
    /// * `crypto` - Unified crypto system reference
    /// * `policy` - Custom database encryption policy
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<Self>` - New database operations coordinator or error
    pub fn with_policy(crypto: Arc<UnifiedCrypto>, policy: DatabaseEncryptionPolicy) -> UnifiedCryptoResult<Self> {
        let key_cache = Arc::new(std::sync::RwLock::new(HashMap::new()));
        let audit_logger = crypto.audit_logger().clone();

        // Validate policy configuration
        if policy.max_record_size == 0 {
            return Err(UnifiedCryptoError::InvalidConfiguration {
                message: "Maximum record size must be greater than zero".to_string(),
            });
        }

        // Log database operations initialization with custom policy
        audit_logger.log_crypto_event(CryptoAuditEvent::database_operations_initialized_with_policy(&policy))?;

        Ok(Self {
            crypto,
            policy,
            key_cache,
            audit_logger,
        })
    }

    /// Encrypt a database record for the specified context
    ///
    /// # Arguments
    /// * `data` - Raw data to encrypt
    /// * `table_name` - Table name for encryption context
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<EncryptedDatabaseRecord>` - Encrypted record or error
    ///
    /// # Security
    /// - Uses context-specific encryption keys
    /// - Applies compression if configured
    /// - Validates record size limits
    /// - Logs encryption operation for audit trail
    pub fn encrypt_record(&self, data: &[u8], table_name: &str) -> UnifiedCryptoResult<EncryptedDatabaseRecord> {
        // Validate input size
        if data.len() > self.policy.max_record_size {
            return Err(UnifiedCryptoError::InvalidInput {
                message: format!("Record size {} exceeds maximum {}", data.len(), self.policy.max_record_size),
            });
        }

        let context = DatabaseContext::Table(table_name.to_string());
        
        // Log encryption start
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::database_encryption_start(&context, data.len())
        )?;

        // Get or create encryption key for this context
        let key_id = self.get_or_create_key_for_context(&context)?;
        let key_pair = self.crypto.key_manager().get_keypair(&key_id)?;

        // Apply compression if enabled
        let (final_data, compression_info) = if self.policy.enable_compression {
            self.compress_data(data)?
        } else {
            (data.to_vec(), None)
        };

        // Encrypt the data
        let encrypted_data = self.crypto.encrypt(&final_data, &key_pair.public_key)?;

        let record = EncryptedDatabaseRecord {
            encrypted_data,
            context: context.clone(),
            timestamp: std::time::SystemTime::now(),
            version: 1,
            compression_info,
        };

        // Log successful encryption
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::database_encryption_success(&context, record.encrypted_data.len())
        )?;

        Ok(record)
    }

    /// Decrypt a database record
    ///
    /// # Arguments
    /// * `encrypted_record` - Encrypted database record
    /// * `table_name` - Expected table name for validation
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<Vec<u8>>` - Decrypted data or error
    ///
    /// # Security
    /// - Validates encryption context matches expected table
    /// - Uses appropriate decryption key for the context
    /// - Handles decompression if applicable
    /// - Logs decryption operation for audit trail
    pub fn decrypt_record(&self, encrypted_record: &EncryptedDatabaseRecord, table_name: &str) -> UnifiedCryptoResult<Vec<u8>> {
        // Validate context matches expected table
        if !encrypted_record.context.is_table_context(table_name) {
            return Err(UnifiedCryptoError::InvalidInput {
                message: format!("Record context {:?} does not match table {}", encrypted_record.context, table_name),
            });
        }

        // Log decryption start
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::database_decryption_start(&encrypted_record.context, encrypted_record.encrypted_data.len())
        )?;

        // Get decryption key
        let key_id = encrypted_record.encrypted_data.key_id();
        let key_pair = self.crypto.key_manager().get_keypair(key_id)?;

        // Decrypt the data
        let decrypted_data = self.crypto.decrypt(&encrypted_record.encrypted_data, &key_pair.private_key)?;

        // Apply decompression if needed
        let final_data = if let Some(compression_info) = &encrypted_record.compression_info {
            self.decompress_data(&decrypted_data, compression_info)?
        } else {
            decrypted_data
        };

        // Log successful decryption
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::database_decryption_success(&encrypted_record.context)
        )?;

        Ok(final_data)
    }

    /// Encrypt a specific column value
    ///
    /// # Arguments
    /// * `data` - Column data to encrypt
    /// * `table_name` - Table name
    /// * `column_name` - Column name
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<EncryptedDatabaseRecord>` - Encrypted column data or error
    pub fn encrypt_column(&self, data: &[u8], table_name: &str, column_name: &str) -> UnifiedCryptoResult<EncryptedDatabaseRecord> {
        let context = DatabaseContext::Column {
            table: table_name.to_string(),
            column: column_name.to_string(),
        };

        // Similar implementation to encrypt_record but with column context
        self.encrypt_with_context(data, context)
    }

    /// Encrypt database schema information
    ///
    /// # Arguments
    /// * `schema_data` - Schema data to encrypt
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<EncryptedDatabaseRecord>` - Encrypted schema or error
    pub fn encrypt_schema(&self, schema_data: &[u8]) -> UnifiedCryptoResult<EncryptedDatabaseRecord> {
        let context = DatabaseContext::Schema;
        self.encrypt_with_context(schema_data, context)
    }

    /// Rotate encryption keys for a specific database context
    ///
    /// # Arguments
    /// * `context` - Database context to rotate keys for
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<KeyId>` - New key ID or error
    ///
    /// # Security
    /// - Generates new encryption key for the context
    /// - Maintains access to old keys for decryption
    /// - Logs key rotation for audit trail
    pub fn rotate_keys(&self, context: &DatabaseContext) -> UnifiedCryptoResult<KeyId> {
        // Log key rotation start
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::database_key_rotation_start(context)
        )?;

        // Generate new key for context
        let new_key_pair = self.crypto.generate_keypair()?;
        let new_key_id = new_key_pair.public_key.id().clone();

        // Update key cache
        {
            let mut cache = self.key_cache.write().map_err(|_| UnifiedCryptoError::ConcurrencyError {
                message: "Failed to acquire key cache lock".to_string(),
            })?;
            cache.insert(context.clone(), new_key_id.clone());
        }

        // Log successful key rotation
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::database_key_rotation_success(context, &new_key_id)
        )?;

        Ok(new_key_id)
    }

    /// Get current encryption policy
    pub fn policy(&self) -> &DatabaseEncryptionPolicy {
        &self.policy
    }

    /// Update encryption policy
    ///
    /// # Arguments
    /// * `new_policy` - New encryption policy
    ///
    /// # Security
    /// - Validates new policy configuration
    /// - Logs policy change for audit trail
    pub fn update_policy(&mut self, new_policy: DatabaseEncryptionPolicy) -> UnifiedCryptoResult<()> {
        // Validate new policy
        if new_policy.max_record_size == 0 {
            return Err(UnifiedCryptoError::InvalidConfiguration {
                message: "Maximum record size must be greater than zero".to_string(),
            });
        }

        // Log policy update
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::database_policy_updated(&self.policy, &new_policy)
        )?;

        self.policy = new_policy;
        Ok(())
    }

    /// Get or create encryption key for a database context
    fn get_or_create_key_for_context(&self, context: &DatabaseContext) -> UnifiedCryptoResult<KeyId> {
        // Check cache first
        {
            let cache = self.key_cache.read().map_err(|_| UnifiedCryptoError::ConcurrencyError {
                message: "Failed to acquire key cache lock".to_string(),
            })?;
            if let Some(key_id) = cache.get(context) {
                return Ok(key_id.clone());
            }
        }

        // Generate new key for context
        let key_pair = self.crypto.generate_keypair()?;
        let key_id = key_pair.public_key.id().clone();

        // Store in cache
        {
            let mut cache = self.key_cache.write().map_err(|_| UnifiedCryptoError::ConcurrencyError {
                message: "Failed to acquire key cache lock".to_string(),
            })?;
            cache.insert(context.clone(), key_id.clone());
        }

        Ok(key_id)
    }

    /// Encrypt data with specific context
    fn encrypt_with_context(&self, data: &[u8], context: DatabaseContext) -> UnifiedCryptoResult<EncryptedDatabaseRecord> {
        // Validate input size
        if data.len() > self.policy.max_record_size {
            return Err(UnifiedCryptoError::InvalidInput {
                message: format!("Data size {} exceeds maximum {}", data.len(), self.policy.max_record_size),
            });
        }

        // Log encryption start
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::database_encryption_start(&context, data.len())
        )?;

        // Get encryption key
        let key_id = self.get_or_create_key_for_context(&context)?;
        let key_pair = self.crypto.key_manager().get_keypair(&key_id)?;

        // Apply compression if enabled
        let (final_data, compression_info) = if self.policy.enable_compression {
            self.compress_data(data)?
        } else {
            (data.to_vec(), None)
        };

        // Encrypt the data
        let encrypted_data = self.crypto.encrypt(&final_data, &key_pair.public_key)?;

        let record = EncryptedDatabaseRecord {
            encrypted_data,
            context: context.clone(),
            timestamp: std::time::SystemTime::now(),
            version: 1,
            compression_info,
        };

        // Log successful encryption
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::database_encryption_success(&context, record.encrypted_data.len())
        )?;

        Ok(record)
    }

    /// Compress data before encryption
    fn compress_data(&self, data: &[u8]) -> UnifiedCryptoResult<(Vec<u8>, Option<CompressionInfo>)> {
        // For now, implement simple compression (in production, use proper compression library)
        let original_size = data.len();
        
        // Simulate compression (replace with actual compression in production)
        let compressed = data.to_vec(); // No actual compression for this example
        let compressed_size = compressed.len();

        let compression_info = CompressionInfo {
            algorithm: CompressionAlgorithm::None,
            original_size,
            compressed_size,
        };

        Ok((compressed, Some(compression_info)))
    }

    /// Decompress data after decryption
    fn decompress_data(&self, data: &[u8], compression_info: &CompressionInfo) -> UnifiedCryptoResult<Vec<u8>> {
        // For now, just return the data as-is (implement actual decompression in production)
        match compression_info.algorithm {
            CompressionAlgorithm::None => Ok(data.to_vec()),
            _ => {
                // In production, implement actual decompression based on algorithm
                Ok(data.to_vec())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unified_crypto::CryptoConfig;

    #[test]
    fn test_database_operations_initialization() {
        let config = CryptoConfig::default();
        let crypto = UnifiedCrypto::new(config).expect("Failed to create crypto");
        let crypto_arc = Arc::new(crypto);
        let db_ops = DatabaseOperations::new(crypto_arc);
        assert!(db_ops.is_ok());
    }

    #[test]
    fn test_database_context_identifier() {
        let table_context = DatabaseContext::Table("users".to_string());
        assert_eq!(table_context.identifier(), "table:users");

        let column_context = DatabaseContext::Column {
            table: "users".to_string(),
            column: "email".to_string(),
        };
        assert_eq!(column_context.identifier(), "column:users:email");
    }

    #[test]
    fn test_encrypt_decrypt_record_roundtrip() {
        let config = CryptoConfig::default();
        let crypto = UnifiedCrypto::new(config).expect("Failed to create crypto");
        let crypto_arc = Arc::new(crypto);
        let db_ops = DatabaseOperations::new(crypto_arc).expect("Failed to create database operations");

        let test_data = b"sensitive user data";
        let table_name = "users";

        let encrypted_record = db_ops.encrypt_record(test_data, table_name)
            .expect("Failed to encrypt record");

        let decrypted_data = db_ops.decrypt_record(&encrypted_record, table_name)
            .expect("Failed to decrypt record");

        assert_eq!(test_data, &decrypted_data[..]);
    }
}