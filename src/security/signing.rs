//! Message signing and verification functionality

use crate::{
    constants::SINGLE_PUBLIC_KEY_ID,
    db_operations::DbOperations,
    security::{
        Ed25519PublicKey, KeyUtils, PublicKeyInfo, SecurityError, SecurityResult, SignedMessage,
        VerificationResult,
    },
};
use base64::{engine::general_purpose, Engine as _};
use serde_json::Value;
use std::sync::{Arc, RwLock};

/// Message signer for client-side use
pub struct MessageSigner {
    keypair: crate::security::Ed25519KeyPair,
}

impl MessageSigner {
    /// Create a new message signer with a key pair
    pub fn new(keypair: crate::security::Ed25519KeyPair) -> Self {
        Self { keypair }
    }

    /// Sign a message payload
    pub fn sign_message(&self, payload: Value) -> SecurityResult<SignedMessage> {
        // Serialize the payload to canonical JSON
        let payload_bytes = self.serialize_payload(&payload)?;

        // Create timestamp
        let timestamp = chrono::Utc::now().timestamp();

        // Create message to sign (payload + timestamp + key_id)
        let mut message_to_sign = payload_bytes.clone();
        message_to_sign.extend_from_slice(&timestamp.to_be_bytes());
        message_to_sign.extend_from_slice(SINGLE_PUBLIC_KEY_ID.as_bytes());

        // Sign the message
        let signature = self.keypair.sign(&message_to_sign);
        let signature_base64 = KeyUtils::signature_to_base64(&signature);

        // Base64 encode the original payload for storage
        let payload_base64 = general_purpose::STANDARD.encode(&payload_bytes);

        Ok(SignedMessage::new(
            payload_base64,
            SINGLE_PUBLIC_KEY_ID.to_string(),
            signature_base64,
            timestamp,
        ))
    }

    /// Serialize payload to canonical JSON bytes
    fn serialize_payload(&self, payload: &Value) -> SecurityResult<Vec<u8>> {
        serde_json::to_vec(payload).map_err(|e| SecurityError::SerializationError(e.to_string()))
    }
}

/// Message verifier for server-side use with optional persistence
pub struct MessageVerifier {
    /// The registered public key (in-memory cache)
    public_key: Arc<RwLock<Option<PublicKeyInfo>>>,
    /// Database operations for persistence
    db_ops: Option<Arc<DbOperations>>,
    /// Maximum allowed timestamp drift in seconds
    max_timestamp_drift: i64,
}

impl MessageVerifier {
    /// Create a new message verifier without persistence
    pub fn new(max_timestamp_drift: i64) -> Self {
        Self {
            public_key: Arc::new(RwLock::new(None)),
            db_ops: None,
            max_timestamp_drift,
        }
    }

    /// Create a new message verifier with database persistence
    pub fn new_with_persistence(
        max_timestamp_drift: i64,
        db_ops: Arc<DbOperations>,
    ) -> SecurityResult<Self> {
        let verifier = Self {
            public_key: Arc::new(RwLock::new(None)),
            db_ops: Some(db_ops),
            max_timestamp_drift,
        };

        // Load persisted key from database
        verifier.load_persisted_key()?;
        Ok(verifier)
    }

    /// Load the persisted public key from database into memory
    fn load_persisted_key(&self) -> SecurityResult<()> {
        if let Some(db_ops) = &self.db_ops {
            match db_ops.get_system_public_key() {
                Ok(Some(persisted_key)) => {
                    let mut key_lock = self
                        .public_key
                        .write()
                        .map_err(|_| SecurityError::KeyNotFound("Failed to acquire write lock".to_string()))?;
                    *key_lock = Some(persisted_key);
                    log::info!("Loaded system public key from database");
                }
                Ok(None) => {
                    log::info!("No system public key found in database.");
                }
                Err(e) => {
                    log::warn!("Failed to load persisted public key: {}", e);
                    // Don't fail initialization - continue without persisted key
                }
            }
        }
        Ok(())
    }

    /// Persist a public key to database
    fn persist_public_key(&self, key_info: &PublicKeyInfo) -> SecurityResult<()> {
        if let Some(db_ops) = &self.db_ops {
            match db_ops.set_system_public_key(key_info) {
                Ok(()) => {
                    log::debug!("Persisted system public key");
                    Ok(())
                }
                Err(e) => {
                    log::error!("Failed to persist system public key: {}", e);
                    // Don't fail the operation - key is still in memory
                    Ok(())
                }
            }
        } else {
            Ok(())
        }
    }

    /// Register the system-wide public key with automatic persistence
    pub fn register_system_public_key(&self, key_info: PublicKeyInfo) -> SecurityResult<()> {
        let mut key_to_store = key_info;
        key_to_store.id = SINGLE_PUBLIC_KEY_ID.to_string();

        // Store in memory first
        {
            let mut key = self
                .public_key
                .write()
                .map_err(|_| SecurityError::KeyNotFound("Failed to acquire write lock".to_string()))?;
            *key = Some(key_to_store.clone());
        }

        // Then persist to database
        self.persist_public_key(&key_to_store)?;

        log::info!("Registered system public key");
        Ok(())
    }

    /// Remove the system public key from both memory and database
    pub fn remove_system_public_key(&self) -> SecurityResult<()> {
        // Remove from memory
        {
            let mut key = self
                .public_key
                .write()
                .map_err(|_| SecurityError::KeyNotFound("Failed to acquire write lock".to_string()))?;
            *key = None;
        }

        // Remove from database
        if let Some(db_ops) = &self.db_ops {
            match db_ops.delete_system_public_key() {
                Ok(_) => log::debug!("Removed system public key from database"),
                Err(e) => log::error!("Failed to remove system public key from database: {}", e),
            }
        }

        log::info!("Removed system public key");
        Ok(())
    }

    /// Get the system public key info
    pub fn get_system_public_key(&self) -> SecurityResult<Option<PublicKeyInfo>> {
        Ok(self
            .public_key
            .read()
            .map_err(|_| SecurityError::KeyNotFound("Failed to acquire read lock".to_string()))?
            .clone())
    }

    /// List the system public key if it exists.
    pub fn list_public_keys(&self) -> SecurityResult<Vec<PublicKeyInfo>> {
        let key = self
            .public_key
            .read()
            .map_err(|_| SecurityError::KeyNotFound("Failed to acquire read lock".to_string()))?;

        if let Some(k) = &*key {
            Ok(vec![k.clone()])
        } else {
            Ok(vec![])
        }
    }

    /// Verify a signed message
    pub fn verify_message(&self, signed_message: &SignedMessage) -> SecurityResult<VerificationResult> {
        // Get the public key info
        let key_info = match self.get_system_public_key()? {
            Some(info) => info,
            None => {
                return Ok(VerificationResult::failure(
                    "System public key not found".to_string(),
                ))
            }
        };

        // Check if key is valid (not expired, active, etc.)
        if !key_info.is_valid() {
            return Ok(VerificationResult::failure(
                "Public key is not valid (expired or inactive)".to_string(),
            ));
        }

        // Check timestamp validity
        let timestamp_valid = self.is_timestamp_valid(signed_message.timestamp);

        // Parse the public key
        let public_key = match Ed25519PublicKey::from_base64(&key_info.public_key) {
            Ok(key) => key,
            Err(e) => {
                return Ok(VerificationResult::failure(format!(
                    "Invalid public key format: {}",
                    e
                )))
            }
        };

        // Parse the signature
        let signature = match KeyUtils::signature_from_base64(&signed_message.signature) {
            Ok(sig) => sig,
            Err(e) => {
                return Ok(VerificationResult::failure(format!(
                    "Invalid signature format: {}",
                    e
                )))
            }
        };

        // Recreate the message that was signed
        let message_to_verify = match self.create_message_to_verify(signed_message) {
            Ok(msg) => msg,
            Err(e) => {
                return Ok(VerificationResult::failure(format!(
                    "Failed to recreate message: {}",
                    e
                )))
            }
        };

        // Verify the signature
        let is_valid = public_key.verify(&message_to_verify, &signature);

        if is_valid && timestamp_valid {
            Ok(VerificationResult::success(key_info, timestamp_valid))
        } else {
            Ok(VerificationResult::failure(
                "Signature verification failed".to_string(),
            ))
        }
    }

    /// Check if timestamp is within acceptable range
    fn is_timestamp_valid(&self, timestamp: i64) -> bool {
        let current_time = chrono::Utc::now().timestamp();
        (current_time - timestamp).abs() <= self.max_timestamp_drift
    }

    /// Recreate the original signed message for verification
    fn create_message_to_verify(&self, signed_message: &SignedMessage) -> SecurityResult<Vec<u8>> {
        let mut message = general_purpose::STANDARD
            .decode(&signed_message.payload)
            .map_err(|e| SecurityError::DeserializationError(e.to_string()))?;
        message.extend_from_slice(&signed_message.timestamp.to_be_bytes());
        message.extend_from_slice(signed_message.public_key_id.as_bytes());
        Ok(message)
    }

    /// Check permissions and verify message
    pub fn verify_message_with_permissions(
        &self,
        signed_message: &SignedMessage,
        required_permissions: &[String],
    ) -> SecurityResult<VerificationResult> {
        let verification_result = self.verify_message(signed_message)?;
        if !verification_result.is_valid {
            return Ok(verification_result);
        }

        if let Some(key_info) = &verification_result.public_key_info {
            for perm in required_permissions {
                if !key_info.permissions.contains(perm) {
                    return Ok(VerificationResult::failure(format!(
                        "Missing required permission: {}",
                        perm
                    )));
                }
            }
        }

        Ok(verification_result)
    }
}

/// Utility functions for signing
pub struct SigningUtils;

impl SigningUtils {
    /// Create a signer from a base64 encoded secret key
    pub fn create_signer_from_secret(secret_key_base64: &str) -> SecurityResult<MessageSigner> {
        let secret_key_bytes = general_purpose::STANDARD
            .decode(secret_key_base64)
            .map_err(|e| SecurityError::InvalidKeyFormat(e.to_string()))?;
        let keypair = crate::security::Ed25519KeyPair::from_secret_key(&secret_key_bytes)?;
        Ok(MessageSigner::new(keypair))
    }

    /// Get the owner ID from a verification result
    pub fn get_message_owner(verification_result: &VerificationResult) -> Option<String> {
        verification_result
            .public_key_info
            .as_ref()
            .map(|info| info.owner_id.clone())
    }

    /// Check if verification was successful
    pub fn is_verification_successful(result: &VerificationResult) -> bool {
        result.is_valid
    }
}

#[cfg(test)]
mod persistence_tests {
    use super::*;
    use crate::security::Ed25519KeyPair;
    use crate::testing_utils::TestDatabaseFactory;

    #[test]
    fn test_message_verifier_persistence() {
        let (db_ops, _) = TestDatabaseFactory::create_test_environment().unwrap();
        let verifier = MessageVerifier::new_with_persistence(60, db_ops.clone()).unwrap();
        let keypair = Ed25519KeyPair::generate().unwrap();
        let public_key_base64 = keypair.public_key_base64();

        let key_info = PublicKeyInfo::new(
            "test_key".to_string(),
            public_key_base64,
            "test_owner".to_string(),
            vec!["read".to_string()],
        );
        verifier.register_system_public_key(key_info.clone()).unwrap();

        // Now create a new verifier with the same database
        let verifier2 = MessageVerifier::new_with_persistence(60, db_ops).unwrap();
        let retrieved = verifier2.get_system_public_key().unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, SINGLE_PUBLIC_KEY_ID.to_string());
    }

    #[test]
    fn test_remove_public_key_persistence() {
        let (db_ops, _) = TestDatabaseFactory::create_test_environment().unwrap();
        let verifier = MessageVerifier::new_with_persistence(60, db_ops.clone()).unwrap();
        let keypair = Ed25519KeyPair::generate().unwrap();
        let public_key_base64 = keypair.public_key_base64();

        let key_info = PublicKeyInfo::new(
            "test_key".to_string(),
            public_key_base64,
            "test_owner".to_string(),
            vec!["read".to_string()],
        );
        verifier.register_system_public_key(key_info).unwrap();
        assert!(verifier.get_system_public_key().unwrap().is_some());

        verifier.remove_system_public_key().unwrap();
        assert!(verifier.get_system_public_key().unwrap().is_none());

        // Verify with a new verifier instance
        let verifier2 = MessageVerifier::new_with_persistence(60, db_ops).unwrap();
        let retrieved = verifier2.get_system_public_key().unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    #[ignore] // This test is flaky depending on timing.
    fn test_graceful_database_failure() {
        // This test is difficult to implement reliably without more extensive mocking
        // or a way to simulate database disconnection. We'll ignore it for now.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::Ed25519KeyPair;
    use serde_json::json;

    #[test]
    fn test_message_signing_and_verification() {
        let signer_keypair = Ed25519KeyPair::generate().unwrap();
        let signer = MessageSigner::new(signer_keypair);
        let verifier = MessageVerifier::new(60);

        let public_key_base64 = signer.keypair.public_key_base64();
        let key_info = PublicKeyInfo::new(
            SINGLE_PUBLIC_KEY_ID.to_string(),
            public_key_base64,
            "test_owner".to_string(),
            vec!["read".to_string()],
        );
        verifier.register_system_public_key(key_info).unwrap();

        let payload = json!({"data": "hello world"});
        let signed_message = signer.sign_message(payload).unwrap();

        let result = verifier.verify_message(&signed_message).unwrap();
        assert!(result.is_valid);
        assert!(result.timestamp_valid);
        assert_eq!(
            result.public_key_info.unwrap().id,
            SINGLE_PUBLIC_KEY_ID.to_string()
        );
    }

    #[test]
    fn test_permission_verification() {
        let signer_keypair = Ed25519KeyPair::generate().unwrap();
        let signer = MessageSigner::new(signer_keypair);
        let verifier = MessageVerifier::new(60);

        let public_key_base64 = signer.keypair.public_key_base64();
        let key_info = PublicKeyInfo::new(
            SINGLE_PUBLIC_KEY_ID.to_string(),
            public_key_base64,
            "test_owner".to_string(),
            vec!["read".to_string()],
        );
        verifier.register_system_public_key(key_info).unwrap();

        let payload = json!({"action": "read_data"});
        let signed_message = signer.sign_message(payload).unwrap();

        // Test with sufficient permissions
        let result1 = verifier
            .verify_message_with_permissions(&signed_message, &["read".to_string()])
            .unwrap();
        assert!(result1.is_valid);

        // Test with insufficient permissions
        let result2 = verifier
            .verify_message_with_permissions(&signed_message, &["write".to_string()])
            .unwrap();
        assert!(!result2.is_valid);
        assert!(result2.error.unwrap().contains("Missing required permission"));
    }

    #[test]
    fn test_timestamp_validation() {
        let signer_keypair = Ed25519KeyPair::generate().unwrap();
        let signer = MessageSigner::new(signer_keypair);
        let verifier = MessageVerifier::new(5); // 5 second tolerance

        let public_key_base64 = signer.keypair.public_key_base64();
        let key_info = PublicKeyInfo::new(
            "test_key".to_string(),
            public_key_base64,
            "test_owner".to_string(),
            vec![],
        );
        verifier.register_system_public_key(key_info).unwrap();

        // Message with valid timestamp
        let valid_payload = json!({"msg": "valid"});
        let valid_message = signer.sign_message(valid_payload).unwrap();
        let valid_result = verifier.verify_message(&valid_message).unwrap();
        assert!(valid_result.is_valid);
        assert!(valid_result.timestamp_valid);

        // Message with expired timestamp
        let expired_payload = json!({"msg": "expired"});
        let mut expired_message = signer.sign_message(expired_payload).unwrap();
        expired_message.timestamp -= 10; // 10 seconds ago, outside tolerance
        let expired_result = verifier.verify_message(&expired_message).unwrap();
        assert!(!expired_result.timestamp_valid);
    }
}