//! Security-related types and data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A signed message sent from client to backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedMessage {
    /// The actual message payload (JSON)
    pub payload: serde_json::Value,
    /// Base64-encoded Ed25519 signature of the payload
    pub signature: String,
    /// Unique identifier for the public key used to sign this message
    pub public_key_id: String,
    /// Timestamp when the message was signed (for replay protection)
    pub timestamp: i64,
    /// Optional nonce for additional replay protection
    pub nonce: Option<String>,
}

impl SignedMessage {
    /// Create a new signed message
    pub fn new(
        payload: serde_json::Value,
        signature: String,
        public_key_id: String,
        timestamp: i64,
    ) -> Self {
        Self {
            payload,
            signature,
            public_key_id,
            timestamp,
            nonce: None,
        }
    }
    
    /// Add a nonce for additional security
    pub fn with_nonce(mut self, nonce: String) -> Self {
        self.nonce = Some(nonce);
        self
    }
}

/// Public key information stored on the backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyInfo {
    /// Unique identifier for this public key
    pub id: String,
    /// Base64-encoded Ed25519 public key
    pub public_key: String,
    /// User or client ID associated with this key
    pub owner_id: String,
    /// When this key was registered
    pub created_at: i64,
    /// Optional expiration timestamp
    pub expires_at: Option<i64>,
    /// Whether this key is currently active
    pub is_active: bool,
    /// Permissions associated with this key
    pub permissions: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl PublicKeyInfo {
    /// Create a new public key info
    pub fn new(
        id: String,
        public_key: String,
        owner_id: String,
        permissions: Vec<String>,
    ) -> Self {
        Self {
            id,
            public_key,
            owner_id,
            created_at: chrono::Utc::now().timestamp(),
            expires_at: None,
            is_active: true,
            permissions,
            metadata: HashMap::new(),
        }
    }
    
    /// Check if this key is currently valid
    pub fn is_valid(&self) -> bool {
        if !self.is_active {
            return false;
        }
        
        if let Some(expires_at) = self.expires_at {
            return chrono::Utc::now().timestamp() < expires_at;
        }
        
        true
    }
    
    /// Add metadata to this key
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Set expiration time
    pub fn with_expiration(mut self, expires_at: i64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
}

/// Encrypted data container for at-rest storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Base64-encoded encrypted data
    pub data: String,
    /// Base64-encoded nonce/IV used for encryption
    pub nonce: String,
    /// Tag for authentication (AES-GCM)
    pub tag: String,
    /// Encryption algorithm used
    pub algorithm: String,
    /// Timestamp when data was encrypted
    pub encrypted_at: i64,
}

impl EncryptedData {
    /// Create a new encrypted data container
    pub fn new(data: String, nonce: String, tag: String) -> Self {
        Self {
            data,
            nonce,
            tag,
            algorithm: "AES-256-GCM".to_string(),
            encrypted_at: chrono::Utc::now().timestamp(),
        }
    }
}

/// Key registration request from client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRegistrationRequest {
    /// Base64-encoded Ed25519 public key
    pub public_key: String,
    /// User or client ID
    pub owner_id: String,
    /// Requested permissions
    pub permissions: Vec<String>,
    /// Optional metadata
    pub metadata: HashMap<String, String>,
    /// Optional expiration timestamp
    pub expires_at: Option<i64>,
}

/// Key registration response from backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRegistrationResponse {
    /// Whether registration was successful
    pub success: bool,
    /// Unique identifier assigned to the public key
    pub public_key_id: Option<String>,
    /// Error message if registration failed
    pub error: Option<String>,
}

/// Message verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether the signature is valid
    pub is_valid: bool,
    /// Public key info used for verification
    pub public_key_info: Option<PublicKeyInfo>,
    /// Error message if verification failed
    pub error: Option<String>,
    /// Whether the message timestamp is within acceptable range
    pub timestamp_valid: bool,
}

impl VerificationResult {
    /// Create a successful verification result
    pub fn success(public_key_info: PublicKeyInfo, timestamp_valid: bool) -> Self {
        Self {
            is_valid: true,
            public_key_info: Some(public_key_info),
            error: None,
            timestamp_valid,
        }
    }
    
    /// Create a failed verification result
    pub fn failure(error: String) -> Self {
        Self {
            is_valid: false,
            public_key_info: None,
            error: Some(error),
            timestamp_valid: false,
        }
    }
}