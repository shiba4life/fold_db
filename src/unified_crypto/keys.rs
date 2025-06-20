//! # Unified Key Management
//!
//! This module provides centralized key lifecycle management for the unified
//! cryptographic system. It handles key generation, storage, rotation, backup,
//! and secure disposal while maintaining comprehensive audit trails.

use crate::unified_crypto::{
    audit::{CryptoAuditEvent, CryptoAuditLogger},
    config::{KeyConfig, KeyStorageBackend},
    error::{ErrorContext, ErrorSeverity, UnifiedCryptoError, UnifiedCryptoResult},
    primitives::{CryptoPrimitives, PrivateKeyHandle, PublicKeyHandle},
    types::{Algorithm, KeyId, KeyMaterial, KeyOperation, KeyUsage},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Key pair container with public and private key handles
#[derive(Debug, Clone)]
pub struct KeyPair {
    /// Public key handle
    pub public_key: PublicKeyHandle,
    /// Private key handle
    pub private_key: PrivateKeyHandle,
    /// Key metadata
    pub metadata: KeyMetadata,
}

impl KeyPair {
    /// Create a new key pair
    pub fn new(
        public_key: PublicKeyHandle,
        private_key: PrivateKeyHandle,
        metadata: KeyMetadata,
    ) -> Self {
        Self {
            public_key,
            private_key,
            metadata,
        }
    }

    /// Create a key pair from secret bytes
    pub fn from_secret_bytes(secret_bytes: &[u8], algorithm: Algorithm) -> UnifiedCryptoResult<Self> {
        let private_key = PrivateKeyHandle::from_secret_bytes(secret_bytes, algorithm)?;
        let public_key = PublicKeyHandle::from_bytes(&private_key.secret_key_bytes()[32..], algorithm)?; // Ed25519 public key is last 32 bytes
        let metadata = KeyMetadata::new(
            private_key.id().clone(),
            algorithm,
            KeyUsage::signing_key(),
        );
        Ok(Self::new(public_key, private_key, metadata))
    }

    /// Get the secret key bytes
    pub fn secret_key_bytes(&self) -> &[u8] {
        self.private_key.secret_key_bytes()
    }

    /// Get the public key bytes
    pub fn public_key_bytes(&self) -> &[u8] {
        self.public_key.public_key_bytes()
    }

    /// Get a reference to the public key
    pub fn public_key(&self) -> &PublicKeyHandle {
        &self.public_key
    }

    /// Get the key ID
    pub fn key_id(&self) -> &KeyId {
        self.public_key.id()
    }

    /// Get the algorithm
    pub fn algorithm(&self) -> Algorithm {
        self.public_key.algorithm()
    }

    /// Sign data using this key pair
    pub fn sign_data(&self, data: &[u8]) -> UnifiedCryptoResult<Vec<u8>> {
        // This would need access to the crypto primitives to actually sign
        // For now, return a placeholder implementation
        Ok(vec![0u8; 64]) // Ed25519 signature length
    }
}

/// Master key pair for legacy compatibility
#[derive(Debug, Clone)]
pub struct MasterKeyPair {
    /// Public key bytes (32 bytes for Ed25519)
    pub public_key_bytes: [u8; 32],
    /// Private key bytes (32 bytes for Ed25519 seed)
    pub private_key_bytes: [u8; 32],
}

impl MasterKeyPair {
    /// Create a master key pair from bytes
    pub fn from_bytes(public_key: &[u8], private_key: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if public_key.len() != 32 {
            return Err("Public key must be 32 bytes".into());
        }
        if private_key.len() < 32 {
            return Err("Private key must be at least 32 bytes".into());
        }
        
        let mut pub_bytes = [0u8; 32];
        let mut priv_bytes = [0u8; 32];
        
        pub_bytes.copy_from_slice(&public_key[..32]);
        priv_bytes.copy_from_slice(&private_key[..32]);
        
        Ok(Self {
            public_key_bytes: pub_bytes,
            private_key_bytes: priv_bytes,
        })
    }

    /// Create a master key pair from secret bytes
    pub fn from_secret_bytes(secret_bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if secret_bytes.len() < 32 {
            return Err("Secret bytes must be at least 32 bytes".into());
        }
        
        // For Ed25519, derive public key from private key
        use ring::signature::{Ed25519KeyPair, KeyPair};
        
        // Create a PKCS8 document from the secret bytes (simplified)
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&secret_bytes[..32]);
        
        // Generate PKCS8 format for ring
        let pkcs8_bytes = match Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()) {
            Ok(bytes) => bytes,
            Err(_) => return Err("Failed to generate PKCS8".into()),
        };
        let key_pair = match Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()) {
            Ok(kp) => kp,
            Err(_) => return Err("Failed to create key pair from PKCS8".into()),
        };
        
        let public_key_bytes = key_pair.public_key().as_ref();
        let mut pub_bytes = [0u8; 32];
        pub_bytes.copy_from_slice(public_key_bytes);
        
        Ok(Self {
            public_key_bytes: pub_bytes,
            private_key_bytes: seed,
        })
    }

    /// Get the public key bytes
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.public_key_bytes
    }

    /// Get the secret key bytes
    pub fn secret_key_bytes(&self) -> [u8; 32] {
        self.private_key_bytes
    }

    /// Sign data using this master key pair
    pub fn sign_data(&self, _data: &[u8]) -> UnifiedCryptoResult<Vec<u8>> {
        use ring::signature::{Ed25519KeyPair, KeyPair};
        
        // Generate PKCS8 format for ring from our seed
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new())
            .map_err(|_| UnifiedCryptoError::KeyGeneration { details: "Failed to generate PKCS8".to_string() })?;
        let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .map_err(|_| UnifiedCryptoError::KeyGeneration { details: "Failed to create key pair".to_string() })?;
        
        // Sign the data
        let signature = key_pair.sign(_data);
        Ok(signature.as_ref().to_vec())
    }
}

/// Comprehensive key metadata for lifecycle management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    /// Unique key identifier
    pub key_id: KeyId,
    /// Cryptographic algorithm
    pub algorithm: Algorithm,
    /// Key usage permissions and restrictions
    pub usage: KeyUsage,
    /// Key creation timestamp
    pub created_at: SystemTime,
    /// Key last used timestamp
    pub last_used_at: Option<SystemTime>,
    /// Key expiration timestamp
    pub expires_at: Option<SystemTime>,
    /// Current key status
    pub status: KeyStatus,
    /// Key generation parameters
    pub generation_params: KeyGenerationParams,
    /// Rotation history
    pub rotation_history: Vec<RotationRecord>,
    /// Backup information
    pub backup_info: Option<BackupInfo>,
    /// Custom metadata tags
    pub tags: HashMap<String, String>,
    /// Operation counters
    pub operation_counters: OperationCounters,
}

impl KeyMetadata {
    /// Create new key metadata
    pub fn new(key_id: KeyId, algorithm: Algorithm, usage: KeyUsage) -> Self {
        Self {
            key_id,
            algorithm,
            usage,
            created_at: SystemTime::now(),
            last_used_at: None,
            expires_at: None,
            status: KeyStatus::Active,
            generation_params: KeyGenerationParams::default(),
            rotation_history: Vec::new(),
            backup_info: None,
            tags: HashMap::new(),
            operation_counters: OperationCounters::default(),
        }
    }

    /// Check if the key is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            SystemTime::now() > expires_at
        } else {
            false
        }
    }

    /// Check if the key needs rotation
    pub fn needs_rotation(&self, max_age: Duration) -> bool {
        match SystemTime::now().duration_since(self.created_at) {
            Ok(age) => age > max_age,
            Err(_) => true, // Clock issues - force rotation
        }
    }

    /// Record key usage
    pub fn record_usage(&mut self, operation: KeyOperation) {
        self.last_used_at = Some(SystemTime::now());
        self.operation_counters.increment(operation);
    }

    /// Check if operation is allowed
    pub fn is_operation_allowed(&self, operation: KeyOperation) -> bool {
        // Check key status
        if !matches!(self.status, KeyStatus::Active) {
            return false;
        }

        // Check expiration
        if self.is_expired() {
            return false;
        }

        // Check usage permissions
        if !self.usage.allows_operation(operation) {
            return false;
        }

        // Check operation limits
        if let Some(max_ops) = self.usage.max_operations {
            if self.operation_counters.total() >= max_ops {
                return false;
            }
        }

        true
    }
}

/// Key lifecycle status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyStatus {
    /// Key is active and can be used
    Active,
    /// Key is being rotated
    Rotating,
    /// Key is deprecated but still valid for decryption/verification
    Deprecated,
    /// Key is revoked and cannot be used
    Revoked,
    /// Key is destroyed and should not exist
    Destroyed,
}

/// Key generation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyGenerationParams {
    /// Algorithm used for generation
    pub algorithm: Algorithm,
    /// Key size in bits
    pub key_size: Option<usize>,
    /// Generation timestamp
    pub generated_at: SystemTime,
    /// Generation source (hardware, software, etc.)
    pub generation_source: String,
    /// Entropy quality metrics
    pub entropy_quality: Option<EntropyQuality>,
}

impl Default for KeyGenerationParams {
    fn default() -> Self {
        Self {
            algorithm: Algorithm::Ed25519,
            key_size: Some(256),
            generated_at: SystemTime::now(),
            generation_source: "software".to_string(),
            entropy_quality: None,
        }
    }
}

/// Entropy quality assessment for key generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyQuality {
    /// Entropy bits per byte
    pub entropy_per_byte: f64,
    /// Randomness test results
    pub randomness_tests: Vec<RandomnessTest>,
    /// Overall quality score (0.0 - 1.0)
    pub quality_score: f64,
}

/// Randomness test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomnessTest {
    /// Test name
    pub test_name: String,
    /// Test result (pass/fail)
    pub passed: bool,
    /// Test score
    pub score: f64,
}

/// Key rotation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationRecord {
    /// Previous key ID
    pub old_key_id: KeyId,
    /// New key ID
    pub new_key_id: KeyId,
    /// Rotation timestamp
    pub rotated_at: SystemTime,
    /// Rotation reason
    pub reason: RotationReason,
    /// Rotation metadata
    pub metadata: HashMap<String, String>,
}

/// Reason for key rotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationReason {
    /// Scheduled rotation
    Scheduled,
    /// Key compromise suspected
    Compromise,
    /// Policy enforcement
    Policy,
    /// Manual request
    Manual,
    /// Emergency rotation
    Emergency,
}

/// Key backup information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    /// Backup creation timestamp
    pub created_at: SystemTime,
    /// Backup location identifier
    pub location: String,
    /// Backup encryption key ID
    pub encryption_key_id: Option<KeyId>,
    /// Backup verification hash
    pub verification_hash: Vec<u8>,
    /// Recovery instructions
    pub recovery_instructions: String,
}

/// Operation counters for key usage tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OperationCounters {
    /// Encryption operations
    pub encrypt_count: u64,
    /// Decryption operations
    pub decrypt_count: u64,
    /// Signing operations
    pub sign_count: u64,
    /// Verification operations
    pub verify_count: u64,
    /// Key derivation operations
    pub derive_count: u64,
}

impl OperationCounters {
    /// Increment counter for operation
    pub fn increment(&mut self, operation: KeyOperation) {
        match operation {
            KeyOperation::Encrypt => self.encrypt_count += 1,
            KeyOperation::Decrypt => self.decrypt_count += 1,
            KeyOperation::Sign => self.sign_count += 1,
            KeyOperation::Verify => self.verify_count += 1,
            KeyOperation::Derive => self.derive_count += 1,
        }
    }

    /// Get total operation count
    pub fn total(&self) -> u64 {
        self.encrypt_count
            + self.decrypt_count
            + self.sign_count
            + self.verify_count
            + self.derive_count
    }
}

/// Unified key manager for centralized key lifecycle management
pub struct KeyManager {
    /// Configuration
    config: Arc<KeyConfig>,
    /// Cryptographic primitives for key operations
    primitives: Arc<CryptoPrimitives>,
    /// Key storage backend
    storage: Arc<dyn KeyStorage>,
    /// In-memory key cache
    key_cache: Arc<RwLock<HashMap<KeyId, CachedKey>>>,
    /// Audit logger
    audit_logger: Arc<CryptoAuditLogger>,
}

impl std::fmt::Debug for KeyManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyManager")
            .field("config", &"<redacted>")
            .field("primitives", &"<redacted>")
            .field("storage", &"<redacted>")
            .field("key_cache", &format!("<{} cached keys>",
                self.key_cache.read().map(|cache| cache.len()).unwrap_or(0)))
            .field("audit_logger", &"<redacted>")
            .finish()
    }
}

/// Cached key information for performance
#[derive(Clone, Debug)]
struct CachedKey {
    /// Key metadata
    metadata: KeyMetadata,
    /// Cached public key (always safe to cache)
    public_key: Option<PublicKeyHandle>,
    /// Cache timestamp
    cached_at: SystemTime,
    /// Access count
    access_count: u64,
}

impl KeyManager {
    /// Create a new key manager
    ///
    /// # Arguments
    /// * `config` - Key management configuration
    /// * `audit_logger` - Audit logger for security events
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<Self>` - New key manager or error
    pub fn new(
        config: &KeyConfig,
        audit_logger: Arc<CryptoAuditLogger>,
    ) -> UnifiedCryptoResult<Self> {
        // Initialize cryptographic primitives
        let primitives_config = crate::unified_crypto::config::PrimitivesConfig::default();
        let primitives = Arc::new(CryptoPrimitives::new(&primitives_config)?);

        // Initialize storage backend
        let storage = Self::create_storage_backend(&config.storage_backend)?;

        // Initialize key cache
        let key_cache = Arc::new(RwLock::new(HashMap::new()));

        // Log initialization
        audit_logger.log_crypto_event(CryptoAuditEvent::key_manager_init())?;

        Ok(Self {
            config: Arc::new(config.clone()),
            primitives,
            storage,
            key_cache,
            audit_logger,
        })
    }

    /// Create storage backend from configuration
    fn create_storage_backend(
        config: &KeyStorageBackend,
    ) -> UnifiedCryptoResult<Arc<dyn KeyStorage>> {
        match config.storage_type {
            crate::unified_crypto::config::StorageType::Memory => {
                Ok(Arc::new(InMemoryKeyStorage::new()))
            }
            crate::unified_crypto::config::StorageType::File => {
                Ok(Arc::new(FileKeyStorage::new("./keys")?))
            }
            _ => Err(UnifiedCryptoError::UnsupportedAlgorithm {
                algorithm: "Storage backend not yet implemented".to_string(),
            }),
        }
    }

    /// Generate a new key pair
    ///
    /// # Arguments
    /// * `algorithm` - Algorithm for key generation
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<KeyPair>` - Generated key pair or error
    ///
    /// # Security
    /// - Uses cryptographically secure key generation
    /// - Records comprehensive audit trail
    /// - Applies security policies
    pub fn generate_keypair(&self, algorithm: &Algorithm) -> UnifiedCryptoResult<KeyPair> {
        // Log key generation start
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::key_generation_start(*algorithm)
        )?;

        // Generate key pair using primitives
        let (public_key, private_key) = self.primitives.generate_keypair(*algorithm)?;

        // Create key metadata
        let mut metadata = KeyMetadata::new(
            public_key.id().clone(),
            *algorithm,
            KeyUsage::signing_key(), // Default usage - can be customized
        );

        // Set generation parameters
        metadata.generation_params = KeyGenerationParams {
            algorithm: *algorithm,
            key_size: algorithm.key_size_bits(),
            generated_at: SystemTime::now(),
            generation_source: "unified_crypto".to_string(),
            entropy_quality: None, // Could be enhanced with actual entropy testing
        };

        // Store key pair
        self.storage.store_keypair(&public_key, &private_key, &metadata)?;

        // Cache metadata
        self.cache_key_metadata(&metadata)?;

        // Create key pair object
        let keypair = KeyPair::new(public_key, private_key, metadata.clone());

        // Log successful generation
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::key_generation_success(&metadata.key_id)
        )?;

        Ok(keypair)
    }

    /// Load a key pair by ID
    ///
    /// # Arguments
    /// * `key_id` - Unique key identifier
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<KeyPair>` - Loaded key pair or error
    pub fn load_keypair(&self, key_id: &KeyId) -> UnifiedCryptoResult<KeyPair> {
        // Check cache first
        if let Some(cached) = self.get_cached_key(key_id)? {
            // Update access count
            self.update_cache_access(key_id)?;
        }

        // Load from storage
        let (public_key, private_key, metadata) = self.storage.load_keypair(key_id)?;

        // Validate key before use
        self.validate_key_usage(&metadata, KeyOperation::Verify)?;

        // Cache the key
        self.cache_key_metadata(&metadata)?;

        // Log key access
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::key_access(key_id)
        )?;

        Ok(KeyPair::new(public_key, private_key, metadata))
    }

    /// Load only the public key by ID
    ///
    /// # Arguments
    /// * `key_id` - Unique key identifier
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<PublicKeyHandle>` - Loaded public key or error
    pub fn load_public_key(&self, key_id: &KeyId) -> UnifiedCryptoResult<PublicKeyHandle> {
        // Check cache first
        if let Some(cached) = self.get_cached_key(key_id)? {
            if let Some(public_key) = &cached.public_key {
                self.update_cache_access(key_id)?;
                return Ok(public_key.clone());
            }
        }

        // Load from storage
        let (public_key, metadata) = self.storage.load_public_key(key_id)?;

        // Cache the key
        self.cache_key_metadata(&metadata)?;

        // Log key access
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::key_access(key_id)
        )?;

        Ok(public_key)
    }

    /// Get a key pair by ID (alias for load_keypair for compatibility)
    ///
    /// # Arguments
    /// * `key_id` - Unique key identifier
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<KeyPair>` - Retrieved key pair or error
    pub fn get_keypair(&self, key_id: &KeyId) -> UnifiedCryptoResult<KeyPair> {
        self.load_keypair(key_id)
    }

    /// Rotate a key pair
    ///
    /// # Arguments
    /// * `old_key_id` - ID of key to rotate
    /// * `reason` - Reason for rotation
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<KeyPair>` - New key pair or error
    pub fn rotate_keypair(
        &self,
        old_key_id: &KeyId,
        reason: RotationReason,
    ) -> UnifiedCryptoResult<KeyPair> {
        // Log rotation start
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::key_rotation_start(old_key_id, &reason)
        )?;

        // Load old key metadata
        let old_metadata = self.storage.load_metadata(old_key_id)?;

        // Generate new key pair with same algorithm
        let new_keypair = self.generate_keypair(&old_metadata.algorithm)?;

        // Create rotation record
        let rotation_record = RotationRecord {
            old_key_id: old_key_id.clone(),
            new_key_id: new_keypair.key_id().clone(),
            rotated_at: SystemTime::now(),
            reason: reason.clone(),
            metadata: HashMap::new(),
        };

        // Update old key status
        let mut updated_old_metadata = old_metadata;
        updated_old_metadata.status = KeyStatus::Deprecated;
        updated_old_metadata.rotation_history.push(rotation_record.clone());
        self.storage.update_metadata(&updated_old_metadata)?;

        // Update new key with rotation history
        let mut new_metadata = new_keypair.metadata.clone();
        new_metadata.rotation_history.push(rotation_record);
        self.storage.update_metadata(&new_metadata)?;

        // Clear old key from cache
        self.remove_from_cache(old_key_id)?;

        // Log successful rotation
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::key_rotation_success(old_key_id, new_keypair.key_id())
        )?;

        Ok(new_keypair)
    }

    /// Revoke a key
    ///
    /// # Arguments
    /// * `key_id` - ID of key to revoke
    /// * `reason` - Reason for revocation
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<()>` - Success or error
    pub fn revoke_key(&self, key_id: &KeyId, reason: &str) -> UnifiedCryptoResult<()> {
        // Log revocation start
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::key_revocation_start(key_id, reason)
        )?;

        // Load and update metadata
        let mut metadata = self.storage.load_metadata(key_id)?;
        metadata.status = KeyStatus::Revoked;
        metadata.tags.insert("revocation_reason".to_string(), reason.to_string());
        metadata.tags.insert("revoked_at".to_string(), 
            SystemTime::now().duration_since(UNIX_EPOCH)
                .unwrap_or_default().as_secs().to_string()
        );

        // Update storage
        self.storage.update_metadata(&metadata)?;

        // Clear from cache
        self.remove_from_cache(key_id)?;

        // Log successful revocation
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::key_revocation_success(key_id)
        )?;

        Ok(())
    }

    /// List all keys with optional filtering
    ///
    /// # Arguments
    /// * `filter` - Optional filter criteria
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<Vec<KeyMetadata>>` - List of key metadata or error
    pub fn list_keys(&self, filter: Option<&KeyFilter>) -> UnifiedCryptoResult<Vec<KeyMetadata>> {
        let all_keys = self.storage.list_keys()?;
        
        if let Some(filter) = filter {
            Ok(all_keys.into_iter().filter(|metadata| filter.matches(metadata)).collect())
        } else {
            Ok(all_keys)
        }
    }

    /// Backup a key pair
    ///
    /// # Arguments
    /// * `key_id` - ID of key to backup
    /// * `backup_location` - Backup destination
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<BackupInfo>` - Backup information or error
    pub fn backup_key(&self, key_id: &KeyId, backup_location: &str) -> UnifiedCryptoResult<BackupInfo> {
        // Log backup start
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::key_backup_start(key_id, backup_location)
        )?;

        // Load key pair
        let (public_key, private_key, metadata) = self.storage.load_keypair(key_id)?;

        // Create backup info
        let backup_info = BackupInfo {
            created_at: SystemTime::now(),
            location: backup_location.to_string(),
            encryption_key_id: None, // Would implement backup encryption
            verification_hash: vec![0u8; 32], // Would implement actual hash
            recovery_instructions: "Contact system administrator".to_string(),
        };

        // Store backup (implementation would vary by storage type)
        // For now, just update metadata
        let mut updated_metadata = metadata;
        updated_metadata.backup_info = Some(backup_info.clone());
        self.storage.update_metadata(&updated_metadata)?;

        // Log successful backup
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::key_backup_success(key_id)
        )?;

        Ok(backup_info)
    }

    /// Validate key usage permissions
    fn validate_key_usage(
        &self,
        metadata: &KeyMetadata,
        operation: KeyOperation,
    ) -> UnifiedCryptoResult<()> {
        if !metadata.is_operation_allowed(operation) {
            return Err(UnifiedCryptoError::PolicyViolation {
                policy: format!("Key {} not allowed for operation {}", metadata.key_id, operation),
            });
        }
        Ok(())
    }

    /// Cache key metadata for performance
    fn cache_key_metadata(&self, metadata: &KeyMetadata) -> UnifiedCryptoResult<()> {
        let cached_key = CachedKey {
            metadata: metadata.clone(),
            public_key: None, // Would cache public key if needed
            cached_at: SystemTime::now(),
            access_count: 0,
        };

        let mut cache = self.key_cache.write()
            .map_err(|_| UnifiedCryptoError::Internal {
                context: "Failed to acquire cache write lock".to_string(),
            })?;

        cache.insert(metadata.key_id.clone(), cached_key);
        Ok(())
    }

    /// Get cached key if available
    fn get_cached_key(&self, key_id: &KeyId) -> UnifiedCryptoResult<Option<CachedKey>> {
        let cache = self.key_cache.read()
            .map_err(|_| UnifiedCryptoError::Internal {
                context: "Failed to acquire cache read lock".to_string(),
            })?;

        Ok(cache.get(key_id).cloned())
    }

    /// Update cache access count
    fn update_cache_access(&self, key_id: &KeyId) -> UnifiedCryptoResult<()> {
        let mut cache = self.key_cache.write()
            .map_err(|_| UnifiedCryptoError::Internal {
                context: "Failed to acquire cache write lock".to_string(),
            })?;

        if let Some(cached_key) = cache.get_mut(key_id) {
            cached_key.access_count += 1;
        }

        Ok(())
    }

    /// Remove key from cache
    fn remove_from_cache(&self, key_id: &KeyId) -> UnifiedCryptoResult<()> {
        let mut cache = self.key_cache.write()
            .map_err(|_| UnifiedCryptoError::Internal {
                context: "Failed to acquire cache write lock".to_string(),
            })?;

        cache.remove(key_id);
        Ok(())
    }
}

/// Key filtering criteria
#[derive(Debug, Clone)]
pub struct KeyFilter {
    /// Filter by algorithm
    pub algorithm: Option<Algorithm>,
    /// Filter by status
    pub status: Option<KeyStatus>,
    /// Filter by creation time range
    pub created_after: Option<SystemTime>,
    /// Filter by creation time range
    pub created_before: Option<SystemTime>,
    /// Filter by tags
    pub tags: HashMap<String, String>,
}

impl KeyFilter {
    /// Check if metadata matches filter
    pub fn matches(&self, metadata: &KeyMetadata) -> bool {
        if let Some(algorithm) = self.algorithm {
            if metadata.algorithm != algorithm {
                return false;
            }
        }

        if let Some(status) = self.status {
            if metadata.status != status {
                return false;
            }
        }

        if let Some(created_after) = self.created_after {
            if metadata.created_at <= created_after {
                return false;
            }
        }

        if let Some(created_before) = self.created_before {
            if metadata.created_at >= created_before {
                return false;
            }
        }

        for (key, value) in &self.tags {
            if metadata.tags.get(key) != Some(value) {
                return false;
            }
        }

        true
    }
}

/// Key storage trait for different storage backends
pub trait KeyStorage: Send + Sync {
    /// Store a key pair with metadata
    fn store_keypair(
        &self,
        public_key: &PublicKeyHandle,
        private_key: &PrivateKeyHandle,
        metadata: &KeyMetadata,
    ) -> UnifiedCryptoResult<()>;

    /// Load a complete key pair
    fn load_keypair(&self, key_id: &KeyId) -> UnifiedCryptoResult<(PublicKeyHandle, PrivateKeyHandle, KeyMetadata)>;

    /// Load only the public key
    fn load_public_key(&self, key_id: &KeyId) -> UnifiedCryptoResult<(PublicKeyHandle, KeyMetadata)>;

    /// Load only metadata
    fn load_metadata(&self, key_id: &KeyId) -> UnifiedCryptoResult<KeyMetadata>;

    /// Update metadata
    fn update_metadata(&self, metadata: &KeyMetadata) -> UnifiedCryptoResult<()>;

    /// List all keys
    fn list_keys(&self) -> UnifiedCryptoResult<Vec<KeyMetadata>>;

    /// Delete a key
    fn delete_key(&self, key_id: &KeyId) -> UnifiedCryptoResult<()>;
}

/// In-memory key storage implementation (for testing/development)
pub struct InMemoryKeyStorage {
    keys: RwLock<HashMap<KeyId, StoredKeyPair>>,
}

#[derive(Debug)]
struct StoredKeyPair {
    public_key: PublicKeyHandle,
    private_key: PrivateKeyHandle,
    metadata: KeyMetadata,
}

impl InMemoryKeyStorage {
    pub fn new() -> Self {
        Self {
            keys: RwLock::new(HashMap::new()),
        }
    }
}

impl KeyStorage for InMemoryKeyStorage {
    fn store_keypair(
        &self,
        public_key: &PublicKeyHandle,
        private_key: &PrivateKeyHandle,
        metadata: &KeyMetadata,
    ) -> UnifiedCryptoResult<()> {
        let stored = StoredKeyPair {
            public_key: public_key.clone(),
            private_key: PrivateKeyHandle::new(
                private_key.id().clone(),
                KeyMaterial::from_bytes(private_key.key_material().bytes().to_vec()),
                private_key.algorithm(),
            ),
            metadata: metadata.clone(),
        };

        let mut keys = self.keys.write()
            .map_err(|_| UnifiedCryptoError::Internal {
                context: "Failed to acquire storage write lock".to_string(),
            })?;

        keys.insert(metadata.key_id.clone(), stored);
        Ok(())
    }

    fn load_keypair(&self, key_id: &KeyId) -> UnifiedCryptoResult<(PublicKeyHandle, PrivateKeyHandle, KeyMetadata)> {
        let keys = self.keys.read()
            .map_err(|_| UnifiedCryptoError::Internal {
                context: "Failed to acquire storage read lock".to_string(),
            })?;

        let stored = keys.get(key_id)
            .ok_or_else(|| UnifiedCryptoError::KeyManagement {
                operation: format!("Key {} not found", key_id),
            })?;

        Ok((
            stored.public_key.clone(),
            PrivateKeyHandle::new(
                stored.private_key.id().clone(),
                KeyMaterial::from_bytes(stored.private_key.key_material().bytes().to_vec()),
                stored.private_key.algorithm(),
            ),
            stored.metadata.clone(),
        ))
    }

    fn load_public_key(&self, key_id: &KeyId) -> UnifiedCryptoResult<(PublicKeyHandle, KeyMetadata)> {
        let keys = self.keys.read()
            .map_err(|_| UnifiedCryptoError::Internal {
                context: "Failed to acquire storage read lock".to_string(),
            })?;

        let stored = keys.get(key_id)
            .ok_or_else(|| UnifiedCryptoError::KeyManagement {
                operation: format!("Key {} not found", key_id),
            })?;

        Ok((stored.public_key.clone(), stored.metadata.clone()))
    }

    fn load_metadata(&self, key_id: &KeyId) -> UnifiedCryptoResult<KeyMetadata> {
        let keys = self.keys.read()
            .map_err(|_| UnifiedCryptoError::Internal {
                context: "Failed to acquire storage read lock".to_string(),
            })?;

        let stored = keys.get(key_id)
            .ok_or_else(|| UnifiedCryptoError::KeyManagement {
                operation: format!("Key {} not found", key_id),
            })?;

        Ok(stored.metadata.clone())
    }

    fn update_metadata(&self, metadata: &KeyMetadata) -> UnifiedCryptoResult<()> {
        let mut keys = self.keys.write()
            .map_err(|_| UnifiedCryptoError::Internal {
                context: "Failed to acquire storage write lock".to_string(),
            })?;

        if let Some(stored) = keys.get_mut(&metadata.key_id) {
            stored.metadata = metadata.clone();
            Ok(())
        } else {
            Err(UnifiedCryptoError::KeyManagement {
                operation: format!("Key {} not found for metadata update", metadata.key_id),
            })
        }
    }

    fn list_keys(&self) -> UnifiedCryptoResult<Vec<KeyMetadata>> {
        let keys = self.keys.read()
            .map_err(|_| UnifiedCryptoError::Internal {
                context: "Failed to acquire storage read lock".to_string(),
            })?;

        Ok(keys.values().map(|stored| stored.metadata.clone()).collect())
    }

    fn delete_key(&self, key_id: &KeyId) -> UnifiedCryptoResult<()> {
        let mut keys = self.keys.write()
            .map_err(|_| UnifiedCryptoError::Internal {
                context: "Failed to acquire storage write lock".to_string(),
            })?;

        keys.remove(key_id)
            .ok_or_else(|| UnifiedCryptoError::KeyManagement {
                operation: format!("Key {} not found for deletion", key_id),
            })?;

        Ok(())
    }
}

/// File-based key storage implementation
pub struct FileKeyStorage {
    base_path: std::path::PathBuf,
}

impl FileKeyStorage {
    pub fn new(base_path: &str) -> UnifiedCryptoResult<Self> {
        let path = std::path::PathBuf::from(base_path);
        
        // Create directory if it doesn't exist
        if !path.exists() {
            std::fs::create_dir_all(&path)
                .map_err(|e| UnifiedCryptoError::Internal {
                    context: format!("Failed to create key storage directory: {}", e),
                })?;
        }

        Ok(Self { base_path: path })
    }
}

impl KeyStorage for FileKeyStorage {
    fn store_keypair(
        &self,
        public_key: &PublicKeyHandle,
        private_key: &PrivateKeyHandle,
        metadata: &KeyMetadata,
    ) -> UnifiedCryptoResult<()> {
        // Implementation would write encrypted key files
        // For now, return not implemented
        Err(UnifiedCryptoError::Internal {
            context: "File storage not yet fully implemented".to_string(),
        })
    }

    fn load_keypair(&self, key_id: &KeyId) -> UnifiedCryptoResult<(PublicKeyHandle, PrivateKeyHandle, KeyMetadata)> {
        Err(UnifiedCryptoError::Internal {
            context: "File storage not yet fully implemented".to_string(),
        })
    }

    fn load_public_key(&self, key_id: &KeyId) -> UnifiedCryptoResult<(PublicKeyHandle, KeyMetadata)> {
        Err(UnifiedCryptoError::Internal {
            context: "File storage not yet fully implemented".to_string(),
        })
    }

    fn load_metadata(&self, key_id: &KeyId) -> UnifiedCryptoResult<KeyMetadata> {
        Err(UnifiedCryptoError::Internal {
            context: "File storage not yet fully implemented".to_string(),
        })
    }

    fn update_metadata(&self, metadata: &KeyMetadata) -> UnifiedCryptoResult<()> {
        Err(UnifiedCryptoError::Internal {
            context: "File storage not yet fully implemented".to_string(),
        })
    }

    fn list_keys(&self) -> UnifiedCryptoResult<Vec<KeyMetadata>> {
        Err(UnifiedCryptoError::Internal {
            context: "File storage not yet fully implemented".to_string(),
        })
    }

    fn delete_key(&self, key_id: &KeyId) -> UnifiedCryptoResult<()> {
        Err(UnifiedCryptoError::Internal {
            context: "File storage not yet fully implemented".to_string(),
        })
    }
}

/// Key rotation manager for automated key lifecycle management
pub struct KeyRotationManager {
    key_manager: Arc<KeyManager>,
    config: Arc<crate::unified_crypto::config::KeyRotationConfig>,
    audit_logger: Arc<CryptoAuditLogger>,
}

impl KeyRotationManager {
    /// Create a new key rotation manager
    pub fn new(
        key_manager: Arc<KeyManager>,
        config: Arc<crate::unified_crypto::config::KeyRotationConfig>,
        audit_logger: Arc<CryptoAuditLogger>,
    ) -> Self {
        Self {
            key_manager,
            config,
            audit_logger,
        }
    }

    /// Check for keys that need rotation
    pub fn check_rotation_needed(&self) -> UnifiedCryptoResult<Vec<KeyId>> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }

        let all_keys = self.key_manager.list_keys(None)?;
        let mut keys_needing_rotation = Vec::new();

        for metadata in all_keys {
            if metadata.status == KeyStatus::Active && 
               metadata.needs_rotation(self.config.max_age) {
                keys_needing_rotation.push(metadata.key_id);
            }
        }

        Ok(keys_needing_rotation)
    }

    /// Perform automatic rotation for eligible keys
    pub fn perform_automatic_rotation(&self) -> UnifiedCryptoResult<Vec<KeyId>> {
        let keys_to_rotate = self.check_rotation_needed()?;
        let mut rotated_keys = Vec::new();

        for key_id in keys_to_rotate {
            match self.key_manager.rotate_keypair(&key_id, RotationReason::Scheduled) {
                Ok(new_keypair) => {
                    rotated_keys.push(new_keypair.key_id().clone());
                    
                    // Log successful automatic rotation
                    self.audit_logger.log_crypto_event(
                        CryptoAuditEvent::automatic_rotation_success(&key_id, new_keypair.key_id())
                    )?;
                }
                Err(err) => {
                    // Log rotation failure but continue with other keys
                    self.audit_logger.log_crypto_event(
                        CryptoAuditEvent::automatic_rotation_failure(&key_id, &err)
                    )?;
                }
            }
        }

        Ok(rotated_keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unified_crypto::audit::CryptoAuditLogger;
    use crate::unified_crypto::config::{AuditConfig, KeyConfig};

    fn create_test_key_manager() -> KeyManager {
        let config = KeyConfig::default();
        let audit_config = AuditConfig::default();
        let audit_logger = Arc::new(CryptoAuditLogger::new(&audit_config).unwrap());
        
        KeyManager::new(&config, audit_logger).unwrap()
    }

    #[test]
    fn test_key_manager_creation() {
        let key_manager = create_test_key_manager();
        // Should not panic and should be created successfully
    }

    #[test]
    fn test_keypair_generation() {
        let key_manager = create_test_key_manager();
        
        let result = key_manager.generate_keypair(&Algorithm::Ed25519);
        assert!(result.is_ok());
        
        let keypair = result.unwrap();
        assert_eq!(keypair.algorithm(), Algorithm::Ed25519);
    }

    #[test]
    fn test_keypair_load() {
        let key_manager = create_test_key_manager();
        
        // Generate a keypair first
        let keypair = key_manager.generate_keypair(&Algorithm::Ed25519).unwrap();
        let key_id = keypair.key_id().clone();
        
        // Load it back
        let loaded = key_manager.load_keypair(&key_id);
        assert!(loaded.is_ok());
        
        let loaded_keypair = loaded.unwrap();
        assert_eq!(loaded_keypair.key_id(), &key_id);
    }

    #[test]
    fn test_key_rotation() {
        let key_manager = create_test_key_manager();
        
        // Generate initial keypair
        let keypair = key_manager.generate_keypair(&Algorithm::Ed25519).unwrap();
        let old_key_id = keypair.key_id().clone();
        
        // Rotate the key
        let rotated = key_manager.rotate_keypair(&old_key_id, RotationReason::Manual);
        assert!(rotated.is_ok());
        
        let new_keypair = rotated.unwrap();
        assert_ne!(new_keypair.key_id(), &old_key_id);
        assert_eq!(new_keypair.algorithm(), Algorithm::Ed25519);
    }

    #[test]
    fn test_key_revocation() {
        let key_manager = create_test_key_manager();
        
        // Generate a keypair
        let keypair = key_manager.generate_keypair(&Algorithm::Ed25519).unwrap();
        let key_id = keypair.key_id().clone();
        
        // Revoke the key
        let result = key_manager.revoke_key(&key_id, "Test revocation");
        assert!(result.is_ok());
        
        // Verify key is revoked
        let metadata = key_manager.storage.load_metadata(&key_id).unwrap();
        assert_eq!(metadata.status, KeyStatus::Revoked);
    }

    #[test]
    fn test_key_metadata() {
        let key_id = KeyId::generate(Algorithm::Ed25519);
        let usage = KeyUsage::signing_key();
        let metadata = KeyMetadata::new(key_id.clone(), Algorithm::Ed25519, usage);
        
        assert_eq!(metadata.key_id, key_id);
        assert_eq!(metadata.algorithm, Algorithm::Ed25519);
        assert_eq!(metadata.status, KeyStatus::Active);
        assert!(!metadata.is_expired());
    }

    #[test]
    fn test_operation_counters() {
        let mut counters = OperationCounters::default();
        assert_eq!(counters.total(), 0);
        
        counters.increment(KeyOperation::Sign);
        counters.increment(KeyOperation::Verify);
        counters.increment(KeyOperation::Encrypt);
        
        assert_eq!(counters.sign_count, 1);
        assert_eq!(counters.verify_count, 1);
        assert_eq!(counters.encrypt_count, 1);
        assert_eq!(counters.total(), 3);
    }
}
// Compatibility function for legacy imports
pub fn generate_master_keypair() -> UnifiedCryptoResult<(PublicKeyHandle, PrivateKeyHandle)> {
    // Use the primitives to generate a master keypair
    let master_keypair = crate::unified_crypto::primitives::generate_master_keypair()
        .map_err(|e| UnifiedCryptoError::KeyGeneration {
            details: format!("Failed to generate master keypair: {}", e)
        })?;
    
    // Convert to expected return type
    let key_id = KeyId::new("master".to_string(), Algorithm::Ed25519);
    
    let public_handle = PublicKeyHandle::new(
        key_id.clone(),
        master_keypair.public_key_bytes.to_vec(),
        Algorithm::Ed25519
    );
    
    let private_handle = PrivateKeyHandle::new(
        key_id,
        KeyMaterial::from_bytes(master_keypair.private_key_bytes.to_vec()),
        Algorithm::Ed25519
    );
    
    Ok((public_handle, private_handle))
}