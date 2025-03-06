use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::num::NonZeroUsize;
use lru::LruCache;
use serde::{Deserialize, Serialize};
use crate::permissions::types::policy::TrustDistance;
use crate::error::{FoldDbError, NetworkErrorKind};

/// Configuration for the WebAuthManager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthConfig {
    /// Maximum number of cached verifications
    pub cache_size: usize,
    /// Duration in seconds for which verifications are cached
    pub cache_ttl_seconds: u64,
    /// Default trust level for new keys
    pub default_trust_level: u32,
}

impl Default for WebAuthConfig {
    fn default() -> Self {
        Self {
            cache_size: 1000,
            cache_ttl_seconds: 300, // 5 minutes
            default_trust_level: 5,
        }
    }
}

/// Represents a public key for authentication
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PublicKey(pub String);

/// Represents a signature for request verification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Signature {
    /// The signature value
    pub value: String,
    /// Timestamp when the signature was created
    pub timestamp: u64,
    /// Nonce to prevent replay attacks
    pub nonce: String,
}

/// Represents a web request with authentication information
#[derive(Debug, Clone)]
pub struct WebRequest {
    /// The public key used for authentication
    pub public_key: PublicKey,
    /// The signature of the request
    pub signature: Signature,
    /// The request path
    pub path: String,
    /// The request method
    pub method: String,
    /// The request body
    pub body: Option<String>,
}

/// Manages web authentication using public key cryptography
pub struct WebAuthManager {
    /// Store verified public keys with their trust levels
    verified_keys: HashMap<PublicKey, u32>,
    /// Cache for recent verifications
    verification_cache: LruCache<Signature, Instant>,
    /// Configuration
    config: WebAuthConfig,
}

impl WebAuthManager {
    /// Create a new WebAuthManager with the given configuration
    pub fn new(config: WebAuthConfig) -> Self {
        let cache_size = NonZeroUsize::new(config.cache_size)
            .unwrap_or_else(|| NonZeroUsize::new(1).unwrap());
            
        Self {
            verified_keys: HashMap::new(),
            verification_cache: LruCache::new(cache_size),
            config,
        }
    }

    /// Create a new WebAuthManager with default configuration
    pub fn default() -> Self {
        Self::new(WebAuthConfig::default())
    }

    /// Verify a request signature
    pub async fn verify_request(&mut self, request: &WebRequest) -> Result<u32, FoldDbError> {
        // Check if the signature is in the cache
        if let Some(timestamp) = self.verification_cache.get(&request.signature) {
            if timestamp.elapsed() < Duration::from_secs(self.config.cache_ttl_seconds) {
                // Signature is valid and not expired
                return self.get_trust_level(&request.public_key);
            }
        }

        // Verify the signature
        if !self.verify_signature(&request.public_key, &request.signature, request) {
            return Err(FoldDbError::Network(NetworkErrorKind::Authentication("Invalid signature".to_string())));
        }

        // Add to cache
        self.verification_cache.put(request.signature.clone(), Instant::now());

        // Return trust level
        self.get_trust_level(&request.public_key)
    }

    /// Register a new public key with a trust level
    pub fn register_key(&mut self, key: PublicKey, trust_level: u32) -> Result<(), FoldDbError> {
        self.verified_keys.insert(key, trust_level);
        Ok(())
    }

    /// Remove an existing key
    pub fn remove_key(&mut self, key: &PublicKey) -> Result<(), FoldDbError> {
        if self.verified_keys.remove(key).is_none() {
            return Err(FoldDbError::Network(NetworkErrorKind::Authentication("Key not found".to_string())));
        }
        Ok(())
    }

    /// Get the trust level for a public key
    fn get_trust_level(&self, key: &PublicKey) -> Result<u32, FoldDbError> {
        self.verified_keys.get(key).copied().ok_or_else(|| {
            FoldDbError::Network(NetworkErrorKind::Authentication("Public key not registered".to_string()))
        })
    }

    /// Verify a signature against a public key
    fn verify_signature(&self, public_key: &PublicKey, signature: &Signature, request: &WebRequest) -> bool {
        // TODO: Implement actual signature verification using a crypto library
        // For now, we'll just return true for testing purposes
        
        // Check if the timestamp is recent (within 5 minutes)
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        if current_time - signature.timestamp > 300 {
            return false;
        }
        
        // In a real implementation, we would:
        // 1. Create a message from the request (path, method, body, timestamp, nonce)
        // 2. Verify the signature against the public key
        // 3. Check for replay attacks using the nonce
        
        true
    }

    /// Convert a trust level to a TrustDistance
    pub fn trust_level_to_distance(trust_level: u32) -> TrustDistance {
        if trust_level == 0 {
            TrustDistance::NoRequirement
        } else {
            TrustDistance::Distance(trust_level)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_verify_key() {
        let mut manager = WebAuthManager::default();
        let key = PublicKey("test_key".to_string());
        let trust_level = 3;

        // Register key
        manager.register_key(key.clone(), trust_level).unwrap();

        // Create a test request
        let request = WebRequest {
            public_key: key.clone(),
            signature: Signature {
                value: "test_signature".to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                nonce: "test_nonce".to_string(),
            },
            path: "/api/test".to_string(),
            method: "GET".to_string(),
            body: None,
        };

        // Verify request
        let result = manager.verify_request(&request).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), trust_level);
    }

    #[test]
    fn test_remove_key() {
        let mut manager = WebAuthManager::default();
        let key = PublicKey("test_key".to_string());
        let trust_level = 3;

        // Register key
        manager.register_key(key.clone(), trust_level).unwrap();

        // Remove key
        let result = manager.remove_key(&key);
        assert!(result.is_ok());

        // Try to get trust level for removed key
        let result = manager.get_trust_level(&key);
        assert!(result.is_err());
    }
}
