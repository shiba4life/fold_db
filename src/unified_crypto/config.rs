//! # Unified Cryptographic Configuration
//!
//! This module provides centralized configuration management for the unified
//! cryptographic system. It supports crypto-agility, security policy enforcement,
//! and dynamic configuration validation.

use crate::security_types::SecurityLevel;
use crate::unified_crypto::{
    error::{UnifiedCryptoError, UnifiedCryptoResult},
    types::{Algorithm, CipherSuite, HashAlgorithm, KeyDerivationFunction, SignatureAlgorithm},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Master key configuration for legacy compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MasterKeyConfig {
    /// Random generated master key
    Random,
    /// Passphrase-derived master key
    Passphrase { passphrase: String },
    /// External key source
    External { key_source: String },
}

impl Default for MasterKeyConfig {
    fn default() -> Self {
        Self::Random
    }
}

/// Legacy key derivation configuration for compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyKeyDerivationConfig {
    /// Default key derivation function
    pub default_kdf: KeyDerivationFunction,
    /// Supported KDF algorithms
    pub supported_kdfs: Vec<KeyDerivationFunction>,
    /// Argon2 parameters
    pub argon2_params: Argon2Params,
    /// PBKDF2 iteration count
    pub pbkdf2_iterations: u32,
    /// Salt length in bytes
    pub salt_length: usize,
}

impl Default for LegacyKeyDerivationConfig {
    fn default() -> Self {
        Self {
            default_kdf: KeyDerivationFunction::Argon2id,
            supported_kdfs: vec![KeyDerivationFunction::Argon2id],
            argon2_params: Argon2Params::default(),
            pbkdf2_iterations: 100_000,
            salt_length: 32,
        }
    }
}

impl LegacyKeyDerivationConfig {
    /// Validate the key derivation configuration
    pub fn validate(&self) -> UnifiedCryptoResult<()> {
        self.argon2_params.validate()?;
        
        if self.salt_length < 16 {
            return Err(UnifiedCryptoError::Configuration {
                message: "Salt length must be at least 16 bytes".to_string(),
            });
        }
        
        if self.pbkdf2_iterations < 10_000 {
            return Err(UnifiedCryptoError::Configuration {
                message: "PBKDF2 iterations must be at least 10,000".to_string(),
            });
        }
        
        Ok(())
    }

    /// Convert to Argon2 parameters
    pub fn to_argon2_params(&self) -> UnifiedCryptoResult<Argon2Params> {
        Ok(self.argon2_params.clone())
    }

    /// Apply security level defaults to key derivation
    pub fn apply_security_level(&mut self, level: SecurityLevel) {
        self.argon2_params.apply_security_level(level);
        
        match level {
            SecurityLevel::High => {
                self.pbkdf2_iterations = self.pbkdf2_iterations.max(200_000);
                self.salt_length = self.salt_length.max(32);
            }
            _ => {} // Standard and Basic use defaults
        }
    }
}

/// Type alias for legacy compatibility
pub type KeyDerivationConfig = LegacyKeyDerivationConfig;

impl LegacyKeyDerivationConfig {
    /// Create a sensitive configuration for legacy compatibility
    pub fn sensitive() -> Self {
        let mut config = Self::default();
        config.argon2_params = Argon2Params::sensitive();
        config
    }

    /// Create configuration for a specific security level
    pub fn for_security_level(level: SecurityLevel) -> Self {
        let mut config = Self::default();
        config.apply_security_level(level);
        config
    }

    /// Expose memory_cost for legacy compatibility
    pub fn memory_cost(&self) -> u32 {
        self.argon2_params.memory_cost
    }

    /// Expose time_cost for legacy compatibility
    pub fn time_cost(&self) -> u32 {
        self.argon2_params.time_cost
    }

    /// Expose parallelism for legacy compatibility
    pub fn parallelism(&self) -> u32 {
        self.argon2_params.parallelism
    }
}

/// Top-level unified cryptographic configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    /// General cryptographic settings
    pub general: GeneralConfig,
    /// Cryptographic primitives configuration
    pub primitives: PrimitivesConfig,
    /// Key management configuration
    pub keys: KeyConfig,
    /// Security policy configuration
    pub policy: CryptoPolicy,
    /// Audit logging configuration
    pub audit: AuditConfig,
    /// Performance and optimization settings
    pub performance: PerformanceConfig,
    /// Master key configuration (for legacy compatibility)
    pub master_key: MasterKeyConfig,
    /// Key derivation configuration (for legacy compatibility)
    pub key_derivation: LegacyKeyDerivationConfig,
    /// Crypto system enabled flag (for legacy compatibility)
    pub enabled: bool,
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            primitives: PrimitivesConfig::default(),
            keys: KeyConfig::default(),
            policy: CryptoPolicy::default(),
            audit: AuditConfig::default(),
            performance: PerformanceConfig::default(),
            master_key: MasterKeyConfig::default(),
            key_derivation: LegacyKeyDerivationConfig::default(),
            enabled: true,
        }
    }
}


impl CryptoConfig {
    /// Create a configuration for the specified security level
    pub fn for_security_level(level: SecurityLevel) -> Self {
        let mut config = Self::default();
        config.general.security_level = level;
        config.apply_security_level_defaults();
        config
    }

    /// Create config with random key (for legacy compatibility)
    pub fn with_random_key() -> Self {
        let mut config = Self::default();
        config.master_key = MasterKeyConfig::Random;
        config
    }

    /// Create config with enhanced security (for legacy compatibility)
    pub fn with_enhanced_security(passphrase: String) -> Self {
        let mut config = Self::default();
        config.master_key = MasterKeyConfig::Passphrase { passphrase };
        config.general.security_level = SecurityLevel::High;
        config
    }

    /// Create config with passphrase (for legacy compatibility)
    pub fn with_passphrase(passphrase: String) -> Self {
        let mut config = Self::default();
        config.master_key = MasterKeyConfig::Passphrase { passphrase };
        config
    }

    /// Validate configuration (for legacy compatibility)
    pub fn validate(&self) -> UnifiedCryptoResult<()> {
        self.validate_security()
    }

    /// Validate the configuration for security compliance
    pub fn validate_security(&self) -> UnifiedCryptoResult<()> {
        // Validate general configuration
        self.general.validate()?;
        
        // Validate primitives configuration
        self.primitives.validate(&self.general.security_level)?;
        
        // Validate key configuration
        self.keys.validate(&self.general.security_level)?;
        
        // Validate policy configuration
        self.policy.validate()?;
        
        // Validate audit configuration
        self.audit.validate()?;
        
        // Validate performance configuration
        self.performance.validate()?;
        
        // Cross-component validation
        self.validate_cross_component()?;
        
        Ok(())
    }

    /// Apply security level defaults to all components
    fn apply_security_level_defaults(&mut self) {
        let level = self.general.security_level;
        
        // Apply to primitives
        self.primitives.apply_security_level(level);
        
        // Apply to keys
        self.keys.apply_security_level(level);
        
        // Apply to policy
        self.policy.apply_security_level(level);
        
        // Apply to audit
        self.audit.apply_security_level(level);
    }

    /// Validate cross-component configuration consistency
    fn validate_cross_component(&self) -> UnifiedCryptoResult<()> {
        // Ensure default algorithms are supported
        if !self.primitives.supported_algorithms.contains(&self.keys.default_algorithm) {
            return Err(UnifiedCryptoError::Configuration {
                message: format!(
                    "Default key algorithm {:?} not in supported algorithms",
                    self.keys.default_algorithm
                ),
            });
        }

        // Ensure policy limits are reasonable
        if let Some(max_key_age) = self.policy.max_key_age {
            if max_key_age < Duration::from_secs(3600) {
                return Err(UnifiedCryptoError::Configuration {
                    message: "Maximum key age must be at least 1 hour".to_string(),
                });
            }
        }

        Ok(())
    }
}

/// General cryptographic settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Overall security level
    pub security_level: SecurityLevel,
    /// Whether to enable FIPS compliance mode
    pub fips_mode: bool,
    /// Whether to enable hardware acceleration
    pub hardware_acceleration: bool,
    /// Whether to enable strict validation
    pub strict_validation: bool,
    /// Version of the configuration format
    pub config_version: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            security_level: SecurityLevel::Standard,
            fips_mode: false,
            hardware_acceleration: true,
            strict_validation: true,
            config_version: "1.0".to_string(),
        }
    }
}

impl GeneralConfig {
    pub fn validate(&self) -> UnifiedCryptoResult<()> {
        // Validate config version
        if self.config_version != "1.0" {
            return Err(UnifiedCryptoError::Configuration {
                message: format!("Unsupported config version: {}", self.config_version),
            });
        }

        Ok(())
    }
}

/// Cryptographic primitives configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimitivesConfig {
    /// Supported cryptographic algorithms
    pub supported_algorithms: Vec<Algorithm>,
    /// Default encryption configuration
    pub encryption: EncryptionConfig,
    /// Default signing configuration
    pub signing: SigningConfig,
    /// Default hashing configuration
    pub hashing: HashingConfig,
    /// Key derivation configuration
    pub key_derivation: LegacyKeyDerivationConfig,
}

impl Default for PrimitivesConfig {
    fn default() -> Self {
        Self {
            supported_algorithms: vec![
                Algorithm::Ed25519,
                Algorithm::Aes256Gcm,
                Algorithm::ChaCha20Poly1305,
                Algorithm::Sha256,
                Algorithm::Sha3_256,
                Algorithm::Blake3,
                Algorithm::Argon2id,
                Algorithm::Hkdf,
            ],
            encryption: EncryptionConfig::default(),
            signing: SigningConfig::default(),
            hashing: HashingConfig::default(),
            key_derivation: LegacyKeyDerivationConfig::default(),
        }
    }
}

impl PrimitivesConfig {
    fn validate(&self, security_level: &SecurityLevel) -> UnifiedCryptoResult<()> {
        // Ensure all algorithms are approved for the security level
        for algorithm in &self.supported_algorithms {
            if !algorithm.is_approved_for_level(*security_level) {
                return Err(UnifiedCryptoError::Configuration {
                    message: format!(
                        "Algorithm {:?} not approved for security level {:?}",
                        algorithm, security_level
                    ),
                });
            }
        }

        // Validate sub-configurations
        self.encryption.validate()?;
        self.signing.validate()?;
        self.hashing.validate()?;
        self.key_derivation.validate()?;

        Ok(())
    }

    fn apply_security_level(&mut self, level: SecurityLevel) {
        // Filter algorithms based on security level
        self.supported_algorithms.retain(|alg| alg.is_approved_for_level(level));
        
        // Apply to sub-configurations
        self.encryption.apply_security_level(level);
        self.signing.apply_security_level(level);
        self.hashing.apply_security_level(level);
        self.key_derivation.apply_security_level(level);
    }
}

/// Encryption algorithm configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Default cipher suite
    pub default_cipher: CipherSuite,
    /// Supported cipher suites
    pub supported_ciphers: Vec<CipherSuite>,
    /// Key size in bits (for symmetric encryption)
    pub key_size: usize,
    /// Whether to use additional authenticated data
    pub use_aad: bool,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            default_cipher: CipherSuite::Aes256Gcm,
            supported_ciphers: vec![CipherSuite::Aes256Gcm, CipherSuite::ChaCha20Poly1305],
            key_size: 256,
            use_aad: true,
        }
    }
}

impl EncryptionConfig {
    pub fn validate(&self) -> UnifiedCryptoResult<()> {
        // Ensure default cipher is in supported list
        if !self.supported_ciphers.contains(&self.default_cipher) {
            return Err(UnifiedCryptoError::Configuration {
                message: "Default cipher not in supported ciphers list".to_string(),
            });
        }

        // Validate key size
        if self.key_size < 128 {
            return Err(UnifiedCryptoError::Configuration {
                message: "Key size must be at least 128 bits".to_string(),
            });
        }

        Ok(())
    }

    fn apply_security_level(&mut self, level: SecurityLevel) {
        match level {
            SecurityLevel::High => {
                // Only the most secure ciphers
                self.supported_ciphers = vec![CipherSuite::Aes256Gcm, CipherSuite::ChaCha20Poly1305];
                self.default_cipher = CipherSuite::Aes256Gcm;
                self.key_size = 256;
                self.use_aad = true;
            }
            _ => {} // Standard and Basic use defaults
        }
    }
}

/// Digital signature configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningConfig {
    /// Default signature algorithm
    pub default_algorithm: SignatureAlgorithm,
    /// Supported signature algorithms
    pub supported_algorithms: Vec<SignatureAlgorithm>,
    /// Whether to include key ID in signatures
    pub include_key_id: bool,
}

impl Default for SigningConfig {
    fn default() -> Self {
        Self {
            default_algorithm: SignatureAlgorithm::Ed25519,
            supported_algorithms: vec![
                SignatureAlgorithm::Ed25519,
                SignatureAlgorithm::RsaPssSha256,
                SignatureAlgorithm::RsaPssSha3_256,
            ],
            include_key_id: true,
        }
    }
}

impl SigningConfig {
    pub fn validate(&self) -> UnifiedCryptoResult<()> {
        // Ensure default algorithm is in supported list
        if !self.supported_algorithms.contains(&self.default_algorithm) {
            return Err(UnifiedCryptoError::Configuration {
                message: "Default signing algorithm not in supported algorithms list".to_string(),
            });
        }

        Ok(())
    }

    fn apply_security_level(&mut self, level: SecurityLevel) {
        match level {
            SecurityLevel::High => {
                // Only the most secure algorithms
                self.supported_algorithms = vec![
                    SignatureAlgorithm::Ed25519,
                    SignatureAlgorithm::RsaPssSha3_256,
                ];
                self.default_algorithm = SignatureAlgorithm::Ed25519;
                self.include_key_id = true;
            }
            _ => {} // Standard and Basic use defaults
        }
    }
}

/// Hash algorithm configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashingConfig {
    /// Default hash algorithm
    pub default_algorithm: HashAlgorithm,
    /// Supported hash algorithms
    pub supported_algorithms: Vec<HashAlgorithm>,
    /// Whether to use keyed hashing (HMAC)
    pub use_keyed_hashing: bool,
}

impl Default for HashingConfig {
    fn default() -> Self {
        Self {
            default_algorithm: HashAlgorithm::Sha256,
            supported_algorithms: vec![
                HashAlgorithm::Sha256,
                HashAlgorithm::Sha3_256,
                HashAlgorithm::Blake3,
            ],
            use_keyed_hashing: false,
        }
    }
}

impl HashingConfig {
    pub fn validate(&self) -> UnifiedCryptoResult<()> {
        // Ensure default algorithm is in supported list
        if !self.supported_algorithms.contains(&self.default_algorithm) {
            return Err(UnifiedCryptoError::Configuration {
                message: "Default hash algorithm not in supported algorithms list".to_string(),
            });
        }

        Ok(())
    }

    fn apply_security_level(&mut self, level: SecurityLevel) {
        match level {
            SecurityLevel::High => {
                // Only the most secure algorithms
                self.supported_algorithms = vec![HashAlgorithm::Sha3_256, HashAlgorithm::Blake3];
                self.default_algorithm = HashAlgorithm::Sha3_256;
                self.use_keyed_hashing = true;
            }
            _ => {} // Standard and Basic use defaults
        }
    }
}

/// Argon2 key derivation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Argon2Params {
    /// Memory cost in KB
    pub memory_cost: u32,
    /// Time cost (iterations)
    pub time_cost: u32,
    /// Parallelism (lanes)
    pub parallelism: u32,
    /// Output length in bytes
    pub output_length: usize,
}

impl Default for Argon2Params {
    fn default() -> Self {
        Self {
            memory_cost: 65536, // 64 MB
            time_cost: 3,
            parallelism: 1,
            output_length: 32,
        }
    }
}

impl Argon2Params {
    /// Create new Argon2 parameters with custom values
    pub fn new(memory_cost: u32, time_cost: u32, parallelism: u32) -> Result<Self, UnifiedCryptoError> {
        let params = Self {
            memory_cost,
            time_cost,
            parallelism,
            output_length: 32,
        };
        params.validate()?;
        Ok(params)
    }

    /// Create Argon2 parameters for interactive use (fast)
    pub fn interactive() -> Self {
        Self {
            memory_cost: 16384, // 16 MB
            time_cost: 1,
            parallelism: 1,
            output_length: 32,
        }
    }

    /// Create Argon2 parameters for sensitive operations (strong)
    pub fn sensitive() -> Self {
        Self {
            memory_cost: 131072, // 128 MB
            time_cost: 5,
            parallelism: 2,
            output_length: 32,
        }
    }

    pub fn validate(&self) -> UnifiedCryptoResult<()> {
        if self.memory_cost < 4096 {
            return Err(UnifiedCryptoError::Configuration {
                message: "Argon2 memory cost must be at least 4096 KB".to_string(),
            });
        }

        if self.time_cost < 1 {
            return Err(UnifiedCryptoError::Configuration {
                message: "Argon2 time cost must be at least 1".to_string(),
            });
        }

        if self.parallelism < 1 {
            return Err(UnifiedCryptoError::Configuration {
                message: "Argon2 parallelism must be at least 1".to_string(),
            });
        }

        if self.output_length < 16 {
            return Err(UnifiedCryptoError::Configuration {
                message: "Argon2 output length must be at least 16 bytes".to_string(),
            });
        }

        Ok(())
    }

    fn apply_security_level(&mut self, level: SecurityLevel) {
        match level {
            SecurityLevel::High => {
                self.memory_cost = self.memory_cost.max(131072); // 128 MB
                self.time_cost = self.time_cost.max(5);
                self.output_length = self.output_length.max(32);
            }
            SecurityLevel::Standard => {
                self.memory_cost = self.memory_cost.max(65536); // 64 MB
                self.time_cost = self.time_cost.max(3);
                self.output_length = self.output_length.max(32);
            }
            _ => {} // Basic and Low use defaults
        }
    }
}

/// Key management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyConfig {
    /// Default algorithm for new keys
    pub default_algorithm: Algorithm,
    /// Key storage backend
    pub storage_backend: KeyStorageBackend,
    /// Key rotation settings
    pub rotation: KeyRotationConfig,
    /// Key backup settings
    pub backup: KeyBackupConfig,
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self {
            default_algorithm: Algorithm::Ed25519,
            storage_backend: KeyStorageBackend::default(),
            rotation: KeyRotationConfig::default(),
            backup: KeyBackupConfig::default(),
        }
    }
}

impl KeyConfig {
    /// Create a KeyConfig optimized for a specific security level
    pub fn for_security_level(security_level: SecurityLevel) -> Self {
        let mut config = Self::default();
        
        // Adjust configuration based on security level
        match security_level {
            SecurityLevel::Basic => {
                // Use default settings for basic security
            }
            SecurityLevel::Low => {
                // Minimal security settings
                config.default_algorithm = Algorithm::Ed25519;
            }
            SecurityLevel::Standard => {
                // Enhanced settings for standard security
                config.default_algorithm = Algorithm::Ed25519;
            }
            SecurityLevel::High => {
                // Maximum security settings
                config.default_algorithm = Algorithm::Ed25519;
            }
        }
        
        config
    }

    pub fn validate(&self, security_level: &SecurityLevel) -> UnifiedCryptoResult<()> {
        // Validate default algorithm for security level
        if !self.default_algorithm.is_approved_for_level(*security_level) {
            return Err(UnifiedCryptoError::Configuration {
                message: format!(
                    "Default algorithm {:?} not approved for security level {:?}",
                    self.default_algorithm, security_level
                ),
            });
        }

        // Validate sub-configurations
        self.storage_backend.validate()?;
        self.rotation.validate()?;
        self.backup.validate()?;

        Ok(())
    }

    fn apply_security_level(&mut self, level: SecurityLevel) {
        match level {
            SecurityLevel::High => {
                self.default_algorithm = Algorithm::Ed25519;
            }
            _ => {} // Other levels use defaults
        }

        self.storage_backend.apply_security_level(level);
        self.rotation.apply_security_level(level);
        self.backup.apply_security_level(level);
    }
}

/// Key storage backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStorageBackend {
    /// Storage type
    pub storage_type: StorageType,
    /// Encryption settings for stored keys
    pub encryption: StorageEncryption,
    /// Access control settings
    pub access_control: AccessControlConfig,
}

impl Default for KeyStorageBackend {
    fn default() -> Self {
        Self {
            storage_type: StorageType::File,
            encryption: StorageEncryption::default(),
            access_control: AccessControlConfig::default(),
        }
    }
}

impl KeyStorageBackend {
    pub fn validate(&self) -> UnifiedCryptoResult<()> {
        self.encryption.validate()?;
        self.access_control.validate()?;
        Ok(())
    }

    fn apply_security_level(&mut self, level: SecurityLevel) {
        self.encryption.apply_security_level(level);
        self.access_control.apply_security_level(level);
    }
}

/// Storage type options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageType {
    /// File-based storage
    File,
    /// In-memory storage (not persistent)
    Memory,
    /// Hardware security module
    Hsm,
    /// Database storage
    Database,
}

/// Storage encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEncryption {
    /// Whether to encrypt stored keys
    pub enabled: bool,
    /// Encryption algorithm for storage
    pub algorithm: Algorithm,
    /// Key derivation for storage encryption
    pub key_derivation: KeyDerivationFunction,
}

impl Default for StorageEncryption {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: Algorithm::Aes256Gcm,
            key_derivation: KeyDerivationFunction::Argon2id,
        }
    }
}

impl StorageEncryption {
    pub fn validate(&self) -> UnifiedCryptoResult<()> {
        if self.enabled {
            // Ensure encryption algorithm is appropriate for storage
            match self.algorithm {
                Algorithm::Aes256Gcm | Algorithm::ChaCha20Poly1305 => {}
                _ => {
                    return Err(UnifiedCryptoError::Configuration {
                        message: format!(
                            "Algorithm {:?} not suitable for storage encryption",
                            self.algorithm
                        ),
                    });
                }
            }
        }
        Ok(())
    }

    fn apply_security_level(&mut self, level: SecurityLevel) {
        match level {
            SecurityLevel::High => {
                self.enabled = true;
                self.algorithm = Algorithm::Aes256Gcm;
                self.key_derivation = KeyDerivationFunction::Argon2id;
            }
            _ => {} // Standard and Basic use defaults
        }
    }
}

/// Access control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlConfig {
    /// Whether to enforce access controls
    pub enabled: bool,
    /// Required permissions by operation
    pub permissions: HashMap<String, Vec<String>>,
}

impl Default for AccessControlConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            permissions: HashMap::new(),
        }
    }
}

impl AccessControlConfig {
    pub fn validate(&self) -> UnifiedCryptoResult<()> {
        // Basic validation - could be extended
        Ok(())
    }

    fn apply_security_level(&mut self, level: SecurityLevel) {
        match level {
            SecurityLevel::High => {
                self.enabled = true;
            }
            _ => {} // Standard and Basic use defaults
        }
    }
}

/// Key rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationConfig {
    /// Whether automatic rotation is enabled
    pub enabled: bool,
    /// Rotation interval
    pub interval: Duration,
    /// Maximum key age before forced rotation
    pub max_age: Duration,
}

impl Default for KeyRotationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(30 * 24 * 3600), // 30 days
            max_age: Duration::from_secs(90 * 24 * 3600),   // 90 days
        }
    }
}

impl KeyRotationConfig {
    pub fn validate(&self) -> UnifiedCryptoResult<()> {
        if self.enabled {
            if self.interval > self.max_age {
                return Err(UnifiedCryptoError::Configuration {
                    message: "Rotation interval cannot exceed max age".to_string(),
                });
            }

            if self.interval < Duration::from_secs(24 * 3600) {
                return Err(UnifiedCryptoError::Configuration {
                    message: "Rotation interval must be at least 24 hours".to_string(),
                });
            }
        }
        Ok(())
    }

    fn apply_security_level(&mut self, level: SecurityLevel) {
        match level {
            SecurityLevel::High => {
                self.enabled = true;
                self.interval = Duration::from_secs(7 * 24 * 3600); // 7 days
                self.max_age = Duration::from_secs(30 * 24 * 3600); // 30 days
            }
            SecurityLevel::Standard => {
                self.enabled = true;
                self.interval = Duration::from_secs(14 * 24 * 3600); // 14 days
                self.max_age = Duration::from_secs(60 * 24 * 3600);  // 60 days
            }
            _ => {} // Basic and Low use defaults
        }
    }
}

/// Key backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBackupConfig {
    /// Whether backup is enabled
    pub enabled: bool,
    /// Backup encryption (double encryption)
    pub encryption: bool,
    /// Backup verification
    pub verification: bool,
}

impl Default for KeyBackupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            encryption: true,
            verification: true,
        }
    }
}

impl KeyBackupConfig {
    pub fn validate(&self) -> UnifiedCryptoResult<()> {
        // Basic validation
        Ok(())
    }

    fn apply_security_level(&mut self, level: SecurityLevel) {
        match level {
            SecurityLevel::High => {
                self.enabled = true;
                self.encryption = true;
                self.verification = true;
            }
            _ => {} // Standard and Basic use defaults
        }
    }
}

/// Security policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoPolicy {
    /// Minimum security level required
    pub min_security_level: SecurityLevel,
    /// Maximum key age
    pub max_key_age: Option<Duration>,
    /// Rate limiting settings
    pub rate_limits: RateLimitConfig,
    /// Compliance settings
    pub compliance: ComplianceConfig,
}

impl Default for CryptoPolicy {
    fn default() -> Self {
        Self {
            min_security_level: SecurityLevel::Standard,
            max_key_age: Some(Duration::from_secs(365 * 24 * 3600)), // 1 year
            rate_limits: RateLimitConfig::default(),
            compliance: ComplianceConfig::default(),
        }
    }
}

impl CryptoPolicy {
    pub fn validate(&self) -> UnifiedCryptoResult<()> {
        self.rate_limits.validate()?;
        self.compliance.validate()?;
        Ok(())
    }

    fn apply_security_level(&mut self, level: SecurityLevel) {
        self.min_security_level = level;
        
        match level {
            SecurityLevel::High => {
                self.max_key_age = Some(Duration::from_secs(30 * 24 * 3600)); // 30 days
            }
            SecurityLevel::Standard => {
                self.max_key_age = Some(Duration::from_secs(90 * 24 * 3600)); // 90 days
            }
            _ => {} // Basic and Low use defaults
        }

        self.rate_limits.apply_security_level(level);
        self.compliance.apply_security_level(level);
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Whether rate limiting is enabled
    pub enabled: bool,
    /// Maximum operations per second
    pub max_ops_per_second: u32,
    /// Burst allowance
    pub burst_allowance: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_ops_per_second: 100,
            burst_allowance: 10,
        }
    }
}

impl RateLimitConfig {
    pub fn validate(&self) -> UnifiedCryptoResult<()> {
        if self.enabled && self.max_ops_per_second == 0 {
            return Err(UnifiedCryptoError::Configuration {
                message: "Rate limit max ops per second must be greater than 0".to_string(),
            });
        }
        Ok(())
    }

    fn apply_security_level(&mut self, level: SecurityLevel) {
        match level {
            SecurityLevel::High => {
                self.enabled = true;
                self.max_ops_per_second = 50;
                self.burst_allowance = 5;
            }
            SecurityLevel::Standard => {
                self.enabled = true;
                self.max_ops_per_second = 75;
                self.burst_allowance = 7;
            }
            _ => {} // Basic and Low use defaults
        }
    }
}

/// Compliance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    /// FIPS 140-2 compliance
    pub fips_140_2: bool,
    /// Common Criteria compliance
    pub common_criteria: bool,
    /// SOC 2 compliance
    pub soc2: bool,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            fips_140_2: false,
            common_criteria: false,
            soc2: false,
        }
    }
}

impl ComplianceConfig {
    pub fn validate(&self) -> UnifiedCryptoResult<()> {
        // Compliance validation could check for required algorithms, etc.
        Ok(())
    }

    fn apply_security_level(&mut self, level: SecurityLevel) {
        match level {
            SecurityLevel::High => {
                // Critical level may require compliance certifications
                self.soc2 = true;
            }
            _ => {} // Other levels use defaults
        }
    }
}

/// Audit logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Whether audit logging is enabled
    pub enabled: bool,
    /// Log level for audit events
    pub log_level: AuditLogLevel,
    /// Whether to include sensitive operation details
    pub include_sensitive_details: bool,
    /// Maximum log file size before rotation
    pub max_log_size: u64,
    /// Number of log files to retain
    pub log_retention_count: u32,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_level: AuditLogLevel::Info,
            include_sensitive_details: false,
            max_log_size: 100 * 1024 * 1024, // 100 MB
            log_retention_count: 10,
        }
    }
}

impl AuditConfig {
    pub fn validate(&self) -> UnifiedCryptoResult<()> {
        if self.max_log_size < 1024 * 1024 {
            return Err(UnifiedCryptoError::Configuration {
                message: "Max log size must be at least 1 MB".to_string(),
            });
        }

        if self.log_retention_count == 0 {
            return Err(UnifiedCryptoError::Configuration {
                message: "Log retention count must be at least 1".to_string(),
            });
        }

        Ok(())
    }

    fn apply_security_level(&mut self, level: SecurityLevel) {
        match level {
            SecurityLevel::High => {
                self.enabled = true;
                self.log_level = AuditLogLevel::Debug;
                self.include_sensitive_details = false; // Never include sensitive details
            }
            _ => {} // Standard and Basic use defaults
        }
    }
}

/// Audit log levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AuditLogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Performance and optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Whether to enable memory pooling
    pub memory_pooling: bool,
    /// Whether to enable operation caching
    pub operation_caching: bool,
    /// Thread pool size for parallel operations
    pub thread_pool_size: usize,
    /// Operation timeout in seconds
    pub operation_timeout: Duration,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            memory_pooling: true,
            operation_caching: false, // Disabled by default for security
            thread_pool_size: 4,
            operation_timeout: Duration::from_secs(30),
        }
    }
}

impl PerformanceConfig {
    pub fn validate(&self) -> UnifiedCryptoResult<()> {
        if self.thread_pool_size == 0 {
            return Err(UnifiedCryptoError::Configuration {
                message: "Thread pool size must be at least 1".to_string(),
            });
        }

        if self.operation_timeout < Duration::from_secs(1) {
            return Err(UnifiedCryptoError::Configuration {
                message: "Operation timeout must be at least 1 second".to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_validation() {
        let config = CryptoConfig::default();
        assert!(config.validate_security().is_ok());
    }

    #[test]
    fn test_security_level_application() {
        let mut config = CryptoConfig::for_security_level(SecurityLevel::High);
        assert_eq!(config.general.security_level, SecurityLevel::High);
        assert!(config.validate_security().is_ok());
    }

    #[test]
    fn test_cross_component_validation() {
        let mut config = CryptoConfig::default();
        config.keys.default_algorithm = Algorithm::Blake3; // Not a valid key algorithm
        config.primitives.supported_algorithms = vec![Algorithm::Ed25519]; // Doesn't include Blake3
        
        assert!(config.validate_security().is_err());
    }

    #[test]
    fn test_argon2_params_validation() {
        let mut params = Argon2Params::default();
        params.memory_cost = 1000; // Too low
        assert!(params.validate().is_err());
        
        params.memory_cost = 65536;
        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_rate_limit_validation() {
        let mut rate_limits = RateLimitConfig::default();
        rate_limits.max_ops_per_second = 0; // Invalid
        assert!(rate_limits.validate().is_err());
        
        rate_limits.max_ops_per_second = 100;
        assert!(rate_limits.validate().is_ok());
    }
}