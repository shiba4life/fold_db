//! Message signing and verification functionality

use crate::security::{
    SecurityError, SecurityResult, SignedMessage, PublicKeyInfo, VerificationResult,
    Ed25519PublicKey, KeyUtils,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Message signer for client-side use
pub struct MessageSigner {
    keypair: crate::security::Ed25519KeyPair,
    public_key_id: String,
}

impl MessageSigner {
    /// Create a new message signer with a key pair
    pub fn new(keypair: crate::security::Ed25519KeyPair, public_key_id: String) -> Self {
        Self {
            keypair,
            public_key_id,
        }
    }
    
    /// Sign a message payload
    pub fn sign_message(&self, payload: Value) -> SecurityResult<SignedMessage> {
        // Serialize the payload to canonical JSON
        let payload_bytes = self.serialize_payload(&payload)?;
        
        // Create timestamp
        let timestamp = chrono::Utc::now().timestamp();
        
        // Create message to sign (payload + timestamp + key_id)
        let mut message_to_sign = payload_bytes;
        message_to_sign.extend_from_slice(&timestamp.to_be_bytes());
        message_to_sign.extend_from_slice(self.public_key_id.as_bytes());
        
        // Sign the message
        let signature = self.keypair.sign(&message_to_sign);
        let signature_base64 = KeyUtils::signature_to_base64(&signature);
        
        Ok(SignedMessage::new(
            payload,
            signature_base64,
            self.public_key_id.clone(),
            timestamp,
        ))
    }
    
    /// Sign a message with a nonce for additional security
    pub fn sign_message_with_nonce(&self, payload: Value, nonce: String) -> SecurityResult<SignedMessage> {
        let mut signed_message = self.sign_message(payload)?;
        signed_message.nonce = Some(nonce);
        Ok(signed_message)
    }
    
    /// Serialize payload to canonical JSON bytes
    fn serialize_payload(&self, payload: &Value) -> SecurityResult<Vec<u8>> {
        serde_json::to_vec(payload)
            .map_err(|e| SecurityError::SerializationError(e.to_string()))
    }
}

/// Message verifier for server-side use with optional persistence
pub struct MessageVerifier {
    /// Registered public keys (in-memory cache)
    public_keys: Arc<RwLock<HashMap<String, PublicKeyInfo>>>,
    /// Database operations for persistence
    db_ops: Option<Arc<crate::db_operations::DbOperations>>,
    /// Maximum allowed timestamp drift in seconds
    max_timestamp_drift: i64,
}

impl MessageVerifier {
    /// Create a new message verifier without persistence
    pub fn new(max_timestamp_drift: i64) -> Self {
        Self {
            public_keys: Arc::new(RwLock::new(HashMap::new())),
            db_ops: None,
            max_timestamp_drift,
        }
    }

    /// Create a new message verifier with database persistence
    pub fn new_with_persistence(
        max_timestamp_drift: i64,
        db_ops: Arc<crate::db_operations::DbOperations>
    ) -> SecurityResult<Self> {
        let verifier = Self {
            public_keys: Arc::new(RwLock::new(HashMap::new())),
            db_ops: Some(db_ops.clone()),
            max_timestamp_drift,
        };

        // Load persisted keys from database
        verifier.load_persisted_keys()?;
        Ok(verifier)
    }

    /// Load all persisted public keys from database into memory
    fn load_persisted_keys(&self) -> SecurityResult<()> {
        if let Some(db_ops) = &self.db_ops {
            match db_ops.get_all_public_keys() {
                Ok(persisted_keys) => {
                    let mut keys = self.public_keys.write()
                        .map_err(|_| SecurityError::KeyNotFound("Failed to acquire write lock".to_string()))?;
                    
                    for key_info in persisted_keys {
                        keys.insert(key_info.id.clone(), key_info);
                    }
                    
                    log::info!("Loaded {} public keys from database", keys.len());
                    Ok(())
                }
                Err(e) => {
                    log::warn!("Failed to load persisted public keys: {}", e);
                    // Don't fail initialization - continue without persisted keys
                    Ok(())
                }
            }
        } else {
            Ok(())
        }
    }

    /// Persist a public key to database
    fn persist_public_key(&self, key_info: &PublicKeyInfo) -> SecurityResult<()> {
        if let Some(db_ops) = &self.db_ops {
            match db_ops.store_public_key(key_info) {
                Ok(()) => {
                    log::debug!("Persisted public key: {}", key_info.id);
                    Ok(())
                }
                Err(e) => {
                    log::error!("Failed to persist public key {}: {}", key_info.id, e);
                    // Don't fail the operation - key is still in memory
                    Ok(())
                }
            }
        } else {
            Ok(())
        }
    }
    
    /// Register a public key with automatic persistence
    pub fn register_public_key(&self, key_info: PublicKeyInfo) -> SecurityResult<()> {
        // Store in memory first
        {
            let mut keys = self.public_keys.write()
                .map_err(|_| SecurityError::KeyNotFound("Failed to acquire write lock".to_string()))?;
            keys.insert(key_info.id.clone(), key_info.clone());
        }

        // Then persist to database
        self.persist_public_key(&key_info)?;

        log::info!("Registered public key: {}", key_info.id);
        Ok(())
    }
    
    /// Remove a public key from both memory and database
    pub fn remove_public_key(&self, key_id: &str) -> SecurityResult<()> {
        // Remove from memory
        {
            let mut keys = self.public_keys.write()
                .map_err(|_| SecurityError::KeyNotFound("Failed to acquire write lock".to_string()))?;
            keys.remove(key_id);
        }

        // Remove from database
        if let Some(db_ops) = &self.db_ops {
            match db_ops.delete_public_key(key_id) {
                Ok(_) => log::debug!("Removed public key from database: {}", key_id),
                Err(e) => log::error!("Failed to remove public key from database {}: {}", key_id, e),
            }
        }

        log::info!("Removed public key: {}", key_id);
        Ok(())
    }
    
    /// Get public key info by ID
    pub fn get_public_key(&self, key_id: &str) -> SecurityResult<Option<PublicKeyInfo>> {
        let keys = self.public_keys.read()
            .map_err(|_| SecurityError::KeyNotFound("Failed to acquire read lock".to_string()))?;
        
        Ok(keys.get(key_id).cloned())
    }
    
    /// List all registered public keys
    pub fn list_public_keys(&self) -> SecurityResult<Vec<PublicKeyInfo>> {
        let keys = self.public_keys.read()
            .map_err(|_| SecurityError::KeyNotFound("Failed to acquire read lock".to_string()))?;
        
        Ok(keys.values().cloned().collect())
    }
    
    /// Verify a signed message
    pub fn verify_message(&self, signed_message: &SignedMessage) -> SecurityResult<VerificationResult> {
        // Get the public key info
        let key_info = match self.get_public_key(&signed_message.public_key_id)? {
            Some(info) => info,
            None => return Ok(VerificationResult::failure(
                format!("Public key not found: {}", signed_message.public_key_id)
            )),
        };
        
        // Check if key is valid (not expired, active, etc.)
        if !key_info.is_valid() {
            return Ok(VerificationResult::failure(
                "Public key is not valid (expired or inactive)".to_string()
            ));
        }
        
        // Check timestamp validity
        let timestamp_valid = self.is_timestamp_valid(signed_message.timestamp);
        
        // Parse the public key
        let public_key = match Ed25519PublicKey::from_base64(&key_info.public_key) {
            Ok(key) => key,
            Err(e) => return Ok(VerificationResult::failure(
                format!("Invalid public key format: {}", e)
            )),
        };
        
        // Parse the signature
        let signature = match KeyUtils::signature_from_base64(&signed_message.signature) {
            Ok(sig) => sig,
            Err(e) => return Ok(VerificationResult::failure(
                format!("Invalid signature format: {}", e)
            )),
        };
        
        // Recreate the message that was signed
        let message_to_verify = match self.create_message_to_verify(signed_message) {
            Ok(msg) => msg,
            Err(e) => return Ok(VerificationResult::failure(
                format!("Failed to recreate message: {}", e)
            )),
        };
        
        // Verify the signature
        let signature_valid = public_key.verify(&message_to_verify, &signature);
        
        if signature_valid {
            Ok(VerificationResult::success(key_info, timestamp_valid))
        } else {
            Ok(VerificationResult::failure("Signature verification failed".to_string()))
        }
    }
    
    /// Check if timestamp is within acceptable range
    fn is_timestamp_valid(&self, timestamp: i64) -> bool {
        let now = chrono::Utc::now().timestamp();
        let diff = (now - timestamp).abs();
        diff <= self.max_timestamp_drift
    }
    
    /// Recreate the message that was signed for verification
    fn create_message_to_verify(&self, signed_message: &SignedMessage) -> SecurityResult<Vec<u8>> {
        // Serialize the payload to canonical JSON
        let payload_bytes = serde_json::to_vec(&signed_message.payload)
            .map_err(|e| SecurityError::SerializationError(e.to_string()))?;
        
        // Recreate the signed message (payload + timestamp + key_id)
        let mut message_to_verify = payload_bytes;
        message_to_verify.extend_from_slice(&signed_message.timestamp.to_be_bytes());
        message_to_verify.extend_from_slice(signed_message.public_key_id.as_bytes());
        
        Ok(message_to_verify)
    }
    
    /// Verify a message and require specific permissions
    pub fn verify_message_with_permissions(
        &self,
        signed_message: &SignedMessage,
        required_permissions: &[String],
    ) -> SecurityResult<VerificationResult> {
        let mut result = self.verify_message(signed_message)?;
        
        if result.is_valid {
            if let Some(key_info) = &result.public_key_info {
                // Check if the key has all required permissions
                for required_perm in required_permissions {
                    if !key_info.permissions.contains(required_perm) {
                        result.is_valid = false;
                        result.error = Some(format!(
                            "Missing required permission: {}",
                            required_perm
                        ));
                        break;
                    }
                }
            }
        }
        
        Ok(result)
    }

}

/// Utility functions for message signing
pub struct SigningUtils;

impl SigningUtils {
    /// Create a message signer from a base64-encoded secret key
    pub fn create_signer_from_secret(
        secret_key_base64: &str,
        public_key_id: String,
    ) -> SecurityResult<MessageSigner> {
        use base64::{Engine as _, engine::general_purpose};
        
        let secret_bytes = general_purpose::STANDARD.decode(secret_key_base64)
            .map_err(|e| SecurityError::KeyGenerationFailed(e.to_string()))?;
        
        let keypair = crate::security::Ed25519KeyPair::from_secret_key(&secret_bytes)?;
        
        Ok(MessageSigner::new(keypair, public_key_id))
    }
    
    /// Extract the owner ID from a verified message
    pub fn get_message_owner(verification_result: &VerificationResult) -> Option<String> {
        verification_result.public_key_info.as_ref().map(|info| info.owner_id.clone())
    }
    
    /// Check if a verification result indicates success
    pub fn is_verification_successful(result: &VerificationResult) -> bool {
        result.is_valid && result.timestamp_valid
    }
}

#[cfg(test)]
mod persistence_tests {
    use super::*;
    use crate::testing_utils::TestDatabaseFactory;

    #[test]
    fn test_message_verifier_persistence() {
        let (db_ops, _) = TestDatabaseFactory::create_test_environment().unwrap();

        // Create verifier with persistence
        let verifier = MessageVerifier::new_with_persistence(300, db_ops.clone()).unwrap();

        // Register a key
        let key_info = PublicKeyInfo::new(
            "test_key".to_string(),
            "test_public_key".to_string(),
            "test_owner".to_string(),
            vec!["read".to_string()],
        );
        verifier.register_public_key(key_info.clone()).unwrap();

        // Verify key is in memory
        let retrieved = verifier.get_public_key("test_key").unwrap();
        assert!(retrieved.is_some());

        // Create new verifier instance to simulate restart
        let verifier2 = MessageVerifier::new_with_persistence(300, db_ops).unwrap();

        // Verify key was loaded from database
        let retrieved2 = verifier2.get_public_key("test_key").unwrap();
        assert!(retrieved2.is_some());
        assert_eq!(retrieved2.unwrap().id, "test_key");
    }

    #[test]
    fn test_remove_public_key_persistence() {
        let (db_ops, _) = TestDatabaseFactory::create_test_environment().unwrap();

        let verifier = MessageVerifier::new_with_persistence(300, db_ops.clone()).unwrap();

        // Register and then remove a key
        let key_info = PublicKeyInfo::new(
            "test_key".to_string(),
            "test_public_key".to_string(),
            "test_owner".to_string(),
            vec!["read".to_string()],
        );
        verifier.register_public_key(key_info).unwrap();
        verifier.remove_public_key("test_key").unwrap();

        // Create new verifier to check persistence
        let verifier2 = MessageVerifier::new_with_persistence(300, db_ops).unwrap();
        let retrieved = verifier2.get_public_key("test_key").unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_graceful_database_failure() {
        // Test that MessageVerifier continues to work even if database operations fail
        let verifier = MessageVerifier::new(300); // No database
        
        let key_info = PublicKeyInfo::new(
            "test_key".to_string(),
            "test_public_key".to_string(),
            "test_owner".to_string(),
            vec!["read".to_string()],
        );
        
        // Should work fine without database
        verifier.register_public_key(key_info).unwrap();
        let retrieved = verifier.get_public_key("test_key").unwrap();
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_persistence_load_multiple_keys() {
        let (db_ops, _) = TestDatabaseFactory::create_test_environment().unwrap();

        let verifier = MessageVerifier::new_with_persistence(300, db_ops.clone()).unwrap();

        // Register multiple keys
        for i in 0..5 {
            let key_info = PublicKeyInfo::new(
                format!("test_key_{}", i),
                format!("test_public_key_{}", i),
                format!("test_owner_{}", i),
                vec!["read".to_string()],
            );
            verifier.register_public_key(key_info).unwrap();
        }

        // Create new verifier to test loading
        let verifier2 = MessageVerifier::new_with_persistence(300, db_ops).unwrap();

        // Verify all keys were loaded
        let all_keys = verifier2.list_public_keys().unwrap();
        assert_eq!(all_keys.len(), 5);

        for i in 0..5 {
            let key_id = format!("test_key_{}", i);
            let retrieved = verifier2.get_public_key(&key_id).unwrap();
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap().id, key_id);
        }
    }

    #[test]
    fn test_security_manager_with_persistence() {
        use crate::security::utils::SecurityManager;
        
        let (db_ops, _) = TestDatabaseFactory::create_test_environment().unwrap();

        // Create config with encryption disabled for testing
        let config = crate::security::SecurityConfig {
            require_tls: false,
            require_signatures: true,
            encrypt_at_rest: false,
            master_key: None,
        };
        let manager = SecurityManager::new_with_persistence(config, db_ops.clone()).unwrap();

        // Generate a client keypair
        let keypair = crate::security::Ed25519KeyPair::generate().unwrap();
        
        // Register the public key
        let registration_request = crate::security::KeyRegistrationRequest {
            public_key: keypair.public_key_base64(),
            owner_id: "test_user".to_string(),
            permissions: vec!["read".to_string()],
            metadata: std::collections::HashMap::new(),
            expires_at: None,
        };
        
        let response = manager.register_public_key(registration_request).unwrap();
        assert!(response.success);
        assert!(response.public_key_id.is_some());

        // Create new manager to test persistence
        let config2 = crate::security::SecurityConfig {
            require_tls: false,
            require_signatures: true,
            encrypt_at_rest: false,
            master_key: None,
        };
        let manager2 = SecurityManager::new_with_persistence(config2, db_ops).unwrap();

        // Verify key was persisted
        let public_key_id = response.public_key_id.unwrap();
        let retrieved = manager2.get_public_key(&public_key_id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().owner_id, "test_user");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::Ed25519KeyPair;
    
    #[test]
    fn test_message_signing_and_verification() {
        // Generate a key pair
        let keypair = Ed25519KeyPair::generate().unwrap();
        let public_key = crate::security::Ed25519PublicKey::from_bytes(&keypair.public_key_bytes()).unwrap();
        let key_id = KeyUtils::generate_key_id(&public_key);
        
        // Create signer and verifier
        let signer = MessageSigner::new(keypair, key_id.clone());
        let verifier = MessageVerifier::new(300); // 5 minute drift
        
        // Register the public key
        let key_info = PublicKeyInfo::new(
            key_id,
            public_key.to_base64(),
            "test_user".to_string(),
            vec!["read".to_string(), "write".to_string()],
        );
        verifier.register_public_key(key_info).unwrap();
        
        // Create and sign a message
        let payload = serde_json::json!({
            "action": "test",
            "data": "hello world"
        });
        
        let signed_message = signer.sign_message(payload).unwrap();
        
        // Verify the message
        let result = verifier.verify_message(&signed_message).unwrap();
        assert!(result.is_valid);
        assert!(result.timestamp_valid);
        assert!(result.public_key_info.is_some());
    }
    
    #[test]
    fn test_permission_verification() {
        // Generate a key pair
        let keypair = Ed25519KeyPair::generate().unwrap();
        let public_key = crate::security::Ed25519PublicKey::from_bytes(&keypair.public_key_bytes()).unwrap();
        let key_id = KeyUtils::generate_key_id(&public_key);
        
        // Create signer and verifier
        let signer = MessageSigner::new(keypair, key_id.clone());
        let verifier = MessageVerifier::new(300);
        
        // Register the public key with limited permissions
        let key_info = PublicKeyInfo::new(
            key_id,
            public_key.to_base64(),
            "test_user".to_string(),
            vec!["read".to_string()], // Only read permission
        );
        verifier.register_public_key(key_info).unwrap();
        
        // Create and sign a message
        let payload = serde_json::json!({"action": "read"});
        let signed_message = signer.sign_message(payload).unwrap();
        
        // Verify with read permission (should succeed)
        let result = verifier.verify_message_with_permissions(
            &signed_message,
            &["read".to_string()]
        ).unwrap();
        assert!(result.is_valid);
        
        // Verify with write permission (should fail)
        let result = verifier.verify_message_with_permissions(
            &signed_message,
            &["write".to_string()]
        ).unwrap();
        assert!(!result.is_valid);
        assert!(result.error.is_some());
    }
    
    #[test]
    fn test_timestamp_validation() {
        let verifier = MessageVerifier::new(60); // 1 minute drift
        
        let now = chrono::Utc::now().timestamp();
        
        // Current timestamp should be valid
        assert!(verifier.is_timestamp_valid(now));
        
        // Timestamp 30 seconds ago should be valid
        assert!(verifier.is_timestamp_valid(now - 30));
        
        // Timestamp 30 seconds in future should be valid
        assert!(verifier.is_timestamp_valid(now + 30));
        
        // Timestamp 2 minutes ago should be invalid
        assert!(!verifier.is_timestamp_valid(now - 120));
        
        // Timestamp 2 minutes in future should be invalid
        assert!(!verifier.is_timestamp_valid(now + 120));
    }
}