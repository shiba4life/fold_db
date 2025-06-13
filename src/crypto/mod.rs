//! # Cryptographic Utilities for DataFold
//!
//! This module provides cryptographic functionality for DataFold's database master key encryption
//! system. It includes secure key generation, management, and serialization for Ed25519 keys.
//!
//! ## Security Features
//!
//! * Secure Ed25519 key pair generation using cryptographically secure random number generation
//! * Automatic memory zeroization for sensitive key material
//! * Safe serialization and deserialization with proper error handling
//! * Integration with Argon2 for passphrase-based key derivation
//!
//! ## Example Usage
//!
//! ```rust
//! use datafold::crypto::ed25519::{generate_master_keypair, MasterKeyPair};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Generate a new master key pair for database encryption
//! let master_keys = generate_master_keypair()?;
//!
//! // Extract public key for storage
//! let public_key_bytes = master_keys.public_key_bytes();
//!
//! // Sign database operations
//! let signature = master_keys.sign_data(b"database-operation-data")?;
//! # Ok(())
//! # }
//! ```

pub mod argon2;
pub mod audit_logger;
pub mod ed25519;
pub mod enhanced_error;
pub mod error;
pub mod key_rotation;
pub mod key_rotation_audit;
pub mod key_rotation_security;
pub mod rotation_threat_monitor;
pub mod security_monitor;

// Re-export commonly used types
pub use argon2::{
    derive_key, derive_master_keypair, derive_master_keypair_default, generate_salt,
    generate_salt_and_derive_keypair, Argon2Params, DerivedKey, Salt,
};
pub use audit_logger::{
    audit_decryption_operation, audit_encryption_operation, audit_security_event,
    get_global_audit_logger, init_global_audit_logger, AuditConfig, AuditEvent, AuditEventType,
    AuditSeverity, CryptoAuditLogger, OperationResult, PerformanceMetrics, SecurityEventDetails,
};
pub use ed25519::{generate_master_keypair, MasterKeyPair, PublicKey};
pub use enhanced_error::{
    EnhancedCryptoError, EnhancedCryptoResult, ErrorContext, ErrorSeverity, RecoveryAction,
};
pub use error::{CryptoError, CryptoResult};
pub use key_rotation::{
    KeyRotationError, KeyRotationRequest, KeyRotationResponse, KeyRotationValidator,
    RotationContext, RotationReason, RotationValidationResult, MAX_REQUEST_LIFETIME,
    MAX_TIMESTAMP_DRIFT,
};
pub use key_rotation_audit::{
    GeolocationInfo, KeyRotationAuditEventType, KeyRotationAuditLogger,
    KeyRotationSecurityMetadata, RotationAuditCorrelation, RotationStatus, SessionInfo,
    TamperProofAuditEntry,
};
pub use key_rotation_security::{
    IpRestrictionConfig, KeyRotationSecurityManager, KeyRotationSecurityPolicy, RateLimitConfig,
    RiskAssessmentConfig, SecurityEvaluationResult, SessionSecurityConfig, TimeRestrictionConfig,
};
pub use rotation_threat_monitor::{
    RemediationAction, RotationThreatDetection, RotationThreatMonitor, RotationThreatMonitorConfig,
    RotationThreatPattern, ThreatStatusSummary,
};
pub use security_monitor::{
    get_global_security_monitor, init_global_security_monitor, CryptoSecurityMonitor,
    SecurityDetection, SecurityMonitorConfig, SecurityPattern, ThreatLevel,
};
