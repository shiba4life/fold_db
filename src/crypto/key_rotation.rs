//! Key rotation and replacement protocol for DataFold
//!
//! This module provides secure key rotation capabilities including:
//! - Cryptographic proof of ownership for old keys
//! - Atomic replacement of key associations
//! - Comprehensive audit logging
//! - Transactional integrity

use super::audit_logger::{AuditSeverity, CryptoAuditLogger, OperationResult};
use super::ed25519::{PrivateKey, PublicKey};
use super::error::{CryptoError, CryptoResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Maximum allowed time drift for rotation requests (5 minutes)
pub const MAX_TIMESTAMP_DRIFT: Duration = Duration::from_secs(300);

/// Maximum rotation request lifetime (30 minutes)
pub const MAX_REQUEST_LIFETIME: Duration = Duration::from_secs(1800);

/// Key rotation reason codes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RotationReason {
    /// Scheduled key rotation
    Scheduled,
    /// Key compromise suspected
    Compromise,
    /// Security policy requirement
    Policy,
    /// User-initiated rotation
    UserInitiated,
    /// Migration to new algorithm
    Migration,
    /// System maintenance
    Maintenance,
}

impl RotationReason {
    /// Get the severity level for audit logging
    pub fn audit_severity(&self) -> AuditSeverity {
        match self {
            RotationReason::Compromise => AuditSeverity::Critical,
            RotationReason::Policy | RotationReason::Migration => AuditSeverity::Warning,
            RotationReason::Scheduled
            | RotationReason::UserInitiated
            | RotationReason::Maintenance => AuditSeverity::Info,
        }
    }
}

/// Key rotation request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationRequest {
    /// Current (old) public key being replaced
    pub old_public_key: PublicKey,
    /// New public key to replace the old one
    pub new_public_key: PublicKey,
    /// Reason for rotation
    pub reason: RotationReason,
    /// Request timestamp (Unix timestamp in milliseconds)
    pub timestamp: u64,
    /// Cryptographic nonce to prevent replay attacks (hex encoded)
    pub nonce: String,
    /// Client identifier (optional, for multi-client scenarios)
    pub client_id: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Signature of the request using the old private key (hex encoded)
    pub signature: String,
}

impl KeyRotationRequest {
    /// Create a new key rotation request
    pub fn new(
        old_private_key: &PrivateKey,
        new_public_key: PublicKey,
        reason: RotationReason,
        client_id: Option<String>,
        metadata: HashMap<String, String>,
    ) -> CryptoResult<Self> {
        let old_public_key = old_private_key.public_key();

        // Validate that new key is different from old key
        if new_public_key.to_bytes() == old_public_key.to_bytes() {
            return Err(CryptoError::InvalidInput(
                "New public key must be different from old public key".to_string(),
            ));
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| CryptoError::InvalidInput(format!("Failed to get timestamp: {}", e)))?
            .as_millis() as u64;

        // Generate cryptographic nonce
        let mut nonce_bytes = [0u8; 32];
        use rand::RngCore;
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = hex::encode(nonce_bytes);

        let mut request = Self {
            old_public_key,
            new_public_key,
            reason,
            timestamp,
            nonce,
            client_id,
            metadata,
            signature: String::new(), // Will be computed below
        };

        // Sign the request with the old private key
        let message = request.signing_message()?;
        let signature_bytes = old_private_key.sign(&message)?;
        request.signature = hex::encode(signature_bytes);

        Ok(request)
    }

    /// Get the message that should be signed for this request
    pub fn signing_message(&self) -> CryptoResult<Vec<u8>> {
        use std::collections::BTreeMap;

        // Create deterministic message for signing
        let sorted_metadata: BTreeMap<String, String> = self
            .metadata
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let signing_data = serde_json::json!({
            "old_public_key": hex::encode(self.old_public_key.to_bytes()),
            "new_public_key": hex::encode(self.new_public_key.to_bytes()),
            "reason": self.reason,
            "timestamp": self.timestamp,
            "nonce": &self.nonce,
            "client_id": self.client_id,
            "metadata": sorted_metadata
        });

        serde_json::to_vec(&signing_data).map_err(|e| {
            CryptoError::InvalidInput(format!("Failed to serialize signing message: {}", e))
        })
    }

    /// Verify the signature on this request
    pub fn verify_signature(&self) -> CryptoResult<()> {
        let message = self.signing_message()?;
        let signature_bytes = hex::decode(&self.signature)
            .map_err(|e| CryptoError::InvalidInput(format!("Invalid signature hex: {}", e)))?;

        if signature_bytes.len() != 64 {
            return Err(CryptoError::InvalidInput(
                "Signature must be 64 bytes".to_string(),
            ));
        }

        let mut signature_array = [0u8; 64];
        signature_array.copy_from_slice(&signature_bytes);

        self.old_public_key.verify(&message, &signature_array)
    }

    /// Check if the request timestamp is within acceptable bounds
    pub fn is_timestamp_valid(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let request_time = self.timestamp;
        let diff = if now > request_time {
            Duration::from_millis(now - request_time)
        } else {
            Duration::from_millis(request_time - now)
        };

        diff <= MAX_TIMESTAMP_DRIFT
    }

    /// Check if the request is still within its lifetime
    pub fn is_within_lifetime(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let age = Duration::from_millis(now.saturating_sub(self.timestamp));
        age <= MAX_REQUEST_LIFETIME
    }

    /// Validate the new public key format and security
    pub fn validate_new_key(&self) -> CryptoResult<()> {
        // Check that new key is different from old key
        if self.new_public_key.to_bytes() == self.old_public_key.to_bytes() {
            return Err(CryptoError::InvalidInput(
                "New public key must be different from old public key".to_string(),
            ));
        }

        // Check for cryptographically weak keys (already done in PublicKey::from_bytes)
        // Additional validation could be added here if needed

        Ok(())
    }

    /// Get the request ID (deterministic hash)
    pub fn request_id(&self) -> CryptoResult<String> {
        let mut hasher = Sha256::new();
        hasher.update(self.old_public_key.to_bytes());
        hasher.update(self.new_public_key.to_bytes());
        hasher.update(self.nonce.as_bytes());
        hasher.update(self.timestamp.to_le_bytes());

        Ok(hex::encode(hasher.finalize())[..16].to_string())
    }
}

/// Key rotation response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationResponse {
    /// Whether the rotation was successful
    pub success: bool,
    /// New key identifier/fingerprint
    pub new_key_id: String,
    /// Confirmation that old key was invalidated
    pub old_key_invalidated: bool,
    /// Audit trail reference for this operation
    pub audit_trail_id: Uuid,
    /// Response timestamp
    pub timestamp: DateTime<Utc>,
    /// Any warnings or notes
    pub warnings: Vec<String>,
}

/// Key rotation error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationError {
    /// Error code
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Detailed error information
    pub details: HashMap<String, serde_json::Value>,
    /// Correlation ID for troubleshooting
    pub correlation_id: Option<Uuid>,
}

impl KeyRotationError {
    /// Create a new rotation error
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details: HashMap::new(),
            correlation_id: None,
        }
    }

    /// Add detail to the error
    pub fn with_detail(mut self, key: &str, value: serde_json::Value) -> Self {
        self.details.insert(key.to_string(), value);
        self
    }

    /// Add correlation ID
    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
}

/// Key rotation validation result
#[derive(Debug, Clone)]
pub struct RotationValidationResult {
    /// Whether the request is valid
    pub is_valid: bool,
    /// Validation errors (if any)
    pub errors: Vec<String>,
    /// Validation warnings (if any)
    pub warnings: Vec<String>,
}

/// Core key rotation validator
pub struct KeyRotationValidator {
    /// Audit logger for validation events
    audit_logger: Option<CryptoAuditLogger>,
}

impl KeyRotationValidator {
    /// Create a new validator
    pub fn new(audit_logger: Option<CryptoAuditLogger>) -> Self {
        Self { audit_logger }
    }

    /// Validate a key rotation request
    pub async fn validate_request(&self, request: &KeyRotationRequest) -> RotationValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Verify signature
        if let Err(e) = request.verify_signature() {
            errors.push(format!("Invalid signature: {}", e));
        }

        // Check timestamp validity
        if !request.is_timestamp_valid() {
            errors.push("Request timestamp is outside acceptable window".to_string());
        }

        // Check request lifetime
        if !request.is_within_lifetime() {
            errors.push("Request has exceeded maximum lifetime".to_string());
        }

        // Validate new key
        if let Err(e) = request.validate_new_key() {
            errors.push(format!("Invalid new public key: {}", e));
        }

        // Add warnings for security-critical reasons
        match request.reason {
            RotationReason::Compromise => {
                warnings.push("Key compromise detected - immediate rotation required".to_string());
            }
            RotationReason::Migration => {
                warnings.push("Algorithm migration in progress".to_string());
            }
            _ => {}
        }

        let is_valid = errors.is_empty();

        // Log validation result
        if let Some(ref logger) = self.audit_logger {
            let result = if is_valid {
                OperationResult::Success
            } else {
                OperationResult::Failure {
                    error_type: "ValidationError".to_string(),
                    error_message: errors.join("; "),
                    error_code: Some("ROTATION_VALIDATION_FAILED".to_string()),
                }
            };

            logger
                .log_key_operation(
                    "validate_rotation_request",
                    "key_rotation",
                    Duration::from_millis(0), // Validation is instant
                    result,
                    None,
                )
                .await;
        }

        RotationValidationResult {
            is_valid,
            errors,
            warnings,
        }
    }
}

/// Key rotation context for database operations
#[derive(Debug, Clone)]
pub struct RotationContext {
    /// Correlation ID for tracking
    pub correlation_id: Uuid,
    /// Request being processed
    pub request: KeyRotationRequest,
    /// Validation result
    pub validation: RotationValidationResult,
    /// Processing start time
    pub started_at: DateTime<Utc>,
    /// Actor performing the rotation
    pub actor: Option<String>,
}

impl RotationContext {
    /// Create new rotation context
    pub fn new(
        request: KeyRotationRequest,
        validation: RotationValidationResult,
        actor: Option<String>,
    ) -> Self {
        Self {
            correlation_id: Uuid::new_v4(),
            request,
            validation,
            started_at: Utc::now(),
            actor,
        }
    }

    /// Get the old public key fingerprint
    pub fn old_key_id(&self) -> String {
        hex::encode(&self.request.old_public_key.to_bytes()[..8])
    }

    /// Get the new public key fingerprint  
    pub fn new_key_id(&self) -> String {
        hex::encode(&self.request.new_public_key.to_bytes()[..8])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::ed25519::generate_master_keypair;

    #[test]
    fn test_key_rotation_request_creation() {
        let old_keypair = generate_master_keypair().unwrap();
        let new_keypair = generate_master_keypair().unwrap();
        let old_private_key = old_keypair.private_key();

        let request = KeyRotationRequest::new(
            &old_private_key,
            new_keypair.public_key().clone(),
            RotationReason::Scheduled,
            Some("test-client".to_string()),
            HashMap::new(),
        )
        .unwrap();

        assert_eq!(
            request.old_public_key.to_bytes(),
            old_keypair.public_key_bytes()
        );
        assert_eq!(
            request.new_public_key.to_bytes(),
            new_keypair.public_key_bytes()
        );
        assert_eq!(request.reason, RotationReason::Scheduled);
        assert!(request.is_timestamp_valid());
        assert!(request.is_within_lifetime());
    }

    #[test]
    fn test_signature_verification() {
        let old_keypair = generate_master_keypair().unwrap();
        let new_keypair = generate_master_keypair().unwrap();
        let old_private_key = old_keypair.private_key();

        let request = KeyRotationRequest::new(
            &old_private_key,
            new_keypair.public_key().clone(),
            RotationReason::UserInitiated,
            None,
            HashMap::new(),
        )
        .unwrap();

        // Should verify correctly
        assert!(request.verify_signature().is_ok());

        // Create corrupted request
        let mut corrupted_request = request.clone();
        // Corrupt the signature by changing it to an invalid hex string
        corrupted_request.signature = "invalid_signature".to_string();

        // Should fail verification
        assert!(corrupted_request.verify_signature().is_err());
    }

    #[test]
    fn test_new_key_validation() {
        let keypair = generate_master_keypair().unwrap();
        let private_key = keypair.private_key();

        // Should fail with same key
        let invalid_request = KeyRotationRequest::new(
            &private_key,
            keypair.public_key().clone(), // Same key!
            RotationReason::Scheduled,
            None,
            HashMap::new(),
        );

        assert!(invalid_request.is_err());
    }

    #[tokio::test]
    async fn test_rotation_validator() {
        let old_keypair = generate_master_keypair().unwrap();
        let new_keypair = generate_master_keypair().unwrap();
        let old_private_key = old_keypair.private_key();

        let request = KeyRotationRequest::new(
            &old_private_key,
            new_keypair.public_key().clone(),
            RotationReason::Policy,
            None,
            HashMap::new(),
        )
        .unwrap();

        let validator = KeyRotationValidator::new(None);
        let result = validator.validate_request(&request).await;

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_rotation_reason_severity() {
        assert_eq!(
            RotationReason::Compromise.audit_severity(),
            AuditSeverity::Critical
        );
        assert_eq!(
            RotationReason::Policy.audit_severity(),
            AuditSeverity::Warning
        );
        assert_eq!(
            RotationReason::Scheduled.audit_severity(),
            AuditSeverity::Info
        );
    }
}
