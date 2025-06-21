//! Security utility functions and helpers

use crate::security::{
    SecurityError, SecurityResult, SignedMessage, PublicKeyInfo, KeyRegistrationRequest,
    KeyRegistrationResponse, MessageVerifier, EncryptionManager, ConditionalEncryption,
    Ed25519KeyPair, Ed25519PublicKey, KeyUtils,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Security manager that combines all security functionality
pub struct SecurityManager {
    /// Message verifier for signature verification
    pub verifier: Arc<MessageVerifier>,
    /// Conditional encryption for data at rest
    pub encryption: Arc<ConditionalEncryption>,
    /// Security configuration
    pub config: crate::security::SecurityConfig,
}

impl SecurityManager {
    /// Create a new security manager without persistence
    pub fn new(config: crate::security::SecurityConfig) -> SecurityResult<Self> {
        let verifier = Arc::new(MessageVerifier::new(300)); // 5 minute timestamp drift
        
        let encryption = Arc::new(ConditionalEncryption::new(
            config.encrypt_at_rest,
            config.master_key,
        )?);
        
        Ok(Self {
            verifier,
            encryption,
            config,
        })
    }

    /// Create a new security manager with database persistence
    pub fn new_with_persistence(
        config: crate::security::SecurityConfig,
        db_ops: Arc<crate::db_operations::DbOperations>
    ) -> SecurityResult<Self> {
        let verifier = Arc::new(MessageVerifier::new_with_persistence(300, db_ops)?);
        
        let encryption = Arc::new(ConditionalEncryption::new(
            config.encrypt_at_rest,
            config.master_key,
        )?);
        
        Ok(Self {
            verifier,
            encryption,
            config,
        })
    }
    
    /// Register a new public key
    pub fn register_public_key(&self, request: KeyRegistrationRequest) -> SecurityResult<KeyRegistrationResponse> {
        // Validate the public key format
        let public_key = Ed25519PublicKey::from_base64(&request.public_key)
            .map_err(|e| SecurityError::InvalidPublicKey(e.to_string()))?;
        
        // Generate a unique key ID
        let key_id = KeyUtils::generate_key_id(&public_key);
        
        // Create public key info, using the validated and re-encoded key
        let mut key_info = PublicKeyInfo::new(
            key_id.clone(),
            public_key.to_base64(), // Use the validated key, re-encoded
            request.owner_id,
            request.permissions,
        );
        
        // Add metadata
        for (k, v) in request.metadata {
            key_info = key_info.with_metadata(k, v);
        }
        
        // Set expiration if provided
        if let Some(expires_at) = request.expires_at {
            key_info = key_info.with_expiration(expires_at);
        }
        
        // Register with the verifier
        self.verifier.register_public_key(key_info)?;
        
        Ok(KeyRegistrationResponse {
            success: true,
            public_key_id: Some(key_id),
            error: None,
        })
    }
    
    /// Verify a signed message
    pub fn verify_message(&self, signed_message: &SignedMessage) -> SecurityResult<crate::security::VerificationResult> {
        if !self.config.require_signatures {
            // If signatures are not required, create a mock successful result
            return Ok(crate::security::VerificationResult {
                is_valid: true,
                public_key_info: None,
                error: None,
                timestamp_valid: true,
            });
        }
        
        self.verifier.verify_message(signed_message)
    }
    
    /// Verify a message with required permissions
    pub fn verify_message_with_permissions(
        &self,
        signed_message: &SignedMessage,
        required_permissions: &[String],
    ) -> SecurityResult<crate::security::VerificationResult> {
        if !self.config.require_signatures {
            // If signatures are not required, create a mock successful result
            return Ok(crate::security::VerificationResult {
                is_valid: true,
                public_key_info: None,
                error: None,
                timestamp_valid: true,
            });
        }
        
        self.verifier.verify_message_with_permissions(signed_message, required_permissions)
    }
    
    /// Encrypt data if encryption is enabled
    pub fn encrypt_data(&self, data: &[u8]) -> SecurityResult<Option<crate::security::EncryptedData>> {
        self.encryption.maybe_encrypt(data)
    }
    
    /// Encrypt JSON data if encryption is enabled
    pub fn encrypt_json(&self, json_data: &Value) -> SecurityResult<Option<crate::security::EncryptedData>> {
        self.encryption.maybe_encrypt_json(json_data)
    }
    
    /// Decrypt data
    pub fn decrypt_data(&self, encrypted_data: &crate::security::EncryptedData) -> SecurityResult<Vec<u8>> {
        self.encryption.maybe_decrypt(encrypted_data)
    }
    
    /// Decrypt JSON data
    pub fn decrypt_json(&self, encrypted_data: &crate::security::EncryptedData) -> SecurityResult<Value> {
        self.encryption.maybe_decrypt_json(encrypted_data)
    }
    
    /// Check if encryption is enabled
    pub fn is_encryption_enabled(&self) -> bool {
        self.encryption.is_encryption_enabled()
    }
    
    /// List all registered public keys
    pub fn list_public_keys(&self) -> SecurityResult<Vec<PublicKeyInfo>> {
        self.verifier.list_public_keys()
    }
    
    /// Remove a public key
    pub fn remove_public_key(&self, key_id: &str) -> SecurityResult<()> {
        self.verifier.remove_public_key(key_id)
    }
    
    /// Get public key info by ID
    pub fn get_public_key(&self, key_id: &str) -> SecurityResult<Option<PublicKeyInfo>> {
        self.verifier.get_public_key(key_id)
    }
}

/// Client-side security utilities
pub struct ClientSecurity;

impl ClientSecurity {
    /// Generate a new key pair for a client
    pub fn generate_client_keypair() -> SecurityResult<Ed25519KeyPair> {
        Ed25519KeyPair::generate()
    }
    
    /// Create a signer from a key pair
    pub fn create_signer(keypair: Ed25519KeyPair, public_key_id: String) -> crate::security::MessageSigner {
        crate::security::MessageSigner::new(keypair, public_key_id)
    }
    
    /// Create a key registration request
    pub fn create_registration_request(
        keypair: &Ed25519KeyPair,
        owner_id: String,
        permissions: Vec<String>,
    ) -> KeyRegistrationRequest {
        KeyRegistrationRequest {
            public_key: keypair.public_key_base64(),
            owner_id,
            permissions,
            metadata: HashMap::new(),
            expires_at: None,
        }
    }
    
    /// Create a signed message from a payload
    pub fn sign_message(
        signer: &crate::security::MessageSigner,
        payload: Value,
    ) -> SecurityResult<SignedMessage> {
        signer.sign_message(payload)
    }
    
    /// Generate example client code
    pub fn generate_client_example() -> String {
        r#"
// Example client-side usage
use datafold::security::*;

// 1. Generate a key pair
let keypair = ClientSecurity::generate_client_keypair()?;

// 2. Create a registration request
let registration_request = ClientSecurity::create_registration_request(
    &keypair,
    "user123".to_string(),
    vec!["read".to_string(), "write".to_string()],
);

// 3. Send registration request to server and get public_key_id back
// let response = send_registration_request(registration_request).await?;
// let public_key_id = response.public_key_id.unwrap();

// 4. Create a message signer
let signer = ClientSecurity::create_signer(keypair, public_key_id);

// 5. Sign messages before sending to server
let payload = serde_json::json!({
    "action": "create_user",
    "data": {
        "username": "alice",
        "email": "alice@example.com"
    }
});

let signed_message = ClientSecurity::sign_message(&signer, payload)?;

// 6. Send signed message to server
// let response = send_signed_message(signed_message).await?;
"#.to_string()
    }
}

/// Server-side security middleware
pub struct SecurityMiddleware {
    manager: Arc<SecurityManager>,
}

impl SecurityMiddleware {
    /// Create new security middleware
    pub fn new(manager: Arc<SecurityManager>) -> Self {
        Self { manager }
    }
    
    /// Validate an incoming request
    pub fn validate_request(
        &self,
        signed_message: &SignedMessage,
        required_permissions: Option<&[String]>,
    ) -> SecurityResult<String> {
        let result = if let Some(perms) = required_permissions {
            self.manager.verify_message_with_permissions(signed_message, perms)?
        } else {
            self.manager.verify_message(signed_message)?
        };
        
        if !result.is_valid {
            return Err(SecurityError::SignatureVerificationFailed(
                result.error.unwrap_or("Invalid signature".to_string())
            ));
        }
        
        if !result.timestamp_valid {
            return Err(SecurityError::SignatureVerificationFailed(
                "Invalid timestamp".to_string()
            ));
        }
        
        // Return the owner ID if available
        Ok(result.public_key_info
            .map(|info| info.owner_id)
            .unwrap_or_else(|| "anonymous".to_string()))
    }
    
    /// Extract and validate a signed message from JSON
    pub fn extract_signed_message(&self, json_data: &Value) -> SecurityResult<SignedMessage> {
        serde_json::from_value(json_data.clone())
            .map_err(|e| SecurityError::DeserializationError(e.to_string()))
    }
    
    /// Wrap a response with optional encryption
    pub fn prepare_response(&self, response_data: &Value) -> SecurityResult<Value> {
        if let Some(encrypted) = self.manager.encrypt_json(response_data)? {
            // If encryption is enabled, return encrypted response
            Ok(serde_json::to_value(encrypted)
                .map_err(|e| SecurityError::SerializationError(e.to_string()))?)
        } else {
            // If encryption is disabled, return raw response
            Ok(response_data.clone())
        }
    }
}

/// Configuration helpers
pub struct SecurityConfigBuilder {
    config: crate::security::SecurityConfig,
}

impl SecurityConfigBuilder {
    /// Create a new config builder
    pub fn new() -> Self {
        Self {
            config: crate::security::SecurityConfig::default(),
        }
    }
    
    /// Set TLS requirement
    pub fn require_tls(mut self, require: bool) -> Self {
        self.config.require_tls = require;
        self
    }
    
    /// Set signature requirement
    pub fn require_signatures(mut self, require: bool) -> Self {
        self.config.require_signatures = require;
        self
    }
    
    /// Enable encryption with a generated key
    pub fn enable_encryption(mut self) -> Self {
        self.config.encrypt_at_rest = true;
        self.config.master_key = Some(EncryptionManager::generate_master_key());
        self
    }
    
    /// Enable encryption with a specific key
    pub fn enable_encryption_with_key(mut self, key: [u8; 32]) -> Self {
        self.config.encrypt_at_rest = true;
        self.config.master_key = Some(key);
        self
    }
    
    /// Enable encryption with a password-derived key
    pub fn enable_encryption_with_password(mut self, password: &str, salt: &[u8]) -> SecurityResult<Self> {
        let key = crate::security::EncryptionUtils::derive_key_from_password(password, salt)?;
        self.config.encrypt_at_rest = true;
        self.config.master_key = Some(key);
        Ok(self)
    }
    
    /// Build the configuration
    pub fn build(self) -> crate::security::SecurityConfig {
        self.config
    }
}

impl Default for SecurityConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_security_manager() {
        let config = SecurityConfigBuilder::new()
            .require_signatures(true)
            .enable_encryption()
            .build();
        
        let manager = SecurityManager::new(config).unwrap();
        
        // Generate a client keypair
        let keypair = ClientSecurity::generate_client_keypair().unwrap();
        
        // Register the public key
        let registration_request = ClientSecurity::create_registration_request(
            &keypair,
            "test_user".to_string(),
            vec!["read".to_string()],
        );
        
        let response = manager.register_public_key(registration_request).unwrap();
        assert!(response.success);
        assert!(response.public_key_id.is_some());
        
        let public_key_id = response.public_key_id.unwrap();
        
        // Create a signer and sign a message
        let signer = ClientSecurity::create_signer(keypair, public_key_id);
        let payload = serde_json::json!({"action": "test"});
        let signed_message = ClientSecurity::sign_message(&signer, payload).unwrap();
        
        // Verify the message
        let result = manager.verify_message(&signed_message).unwrap();
        assert!(result.is_valid);
    }
    
    #[test]
    fn test_security_middleware() {
        let config = SecurityConfigBuilder::new()
            .require_signatures(true)
            .enable_encryption()
            .build();
        
        let manager = Arc::new(SecurityManager::new(config).unwrap());
        let middleware = SecurityMiddleware::new(manager.clone());
        
        // Register a key
        let keypair = ClientSecurity::generate_client_keypair().unwrap();
        let registration_request = ClientSecurity::create_registration_request(
            &keypair,
            "test_user".to_string(),
            vec!["read".to_string(), "write".to_string()],
        );
        
        let response = manager.register_public_key(registration_request).unwrap();
        let public_key_id = response.public_key_id.unwrap();
        
        // Create a signed message
        let signer = ClientSecurity::create_signer(keypair, public_key_id);
        let payload = serde_json::json!({"action": "test"});
        let signed_message = ClientSecurity::sign_message(&signer, payload).unwrap();
        
        // Validate with middleware
        let owner_id = middleware.validate_request(&signed_message, Some(&["read".to_string()])).unwrap();
        assert_eq!(owner_id, "test_user");
        
        // Should fail with insufficient permissions
        let result = middleware.validate_request(&signed_message, Some(&["admin".to_string()]));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_config_builder() {
        let config = SecurityConfigBuilder::new()
            .require_tls(true)
            .require_signatures(true)
            .enable_encryption()
            .build();
        
        assert!(config.require_tls);
        assert!(config.require_signatures);
        assert!(config.encrypt_at_rest);
        assert!(config.master_key.is_some());
    }
}
