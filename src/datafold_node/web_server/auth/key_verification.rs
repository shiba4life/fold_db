use std::sync::Arc;
use warp::{Filter, Rejection, Reply};
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use crate::datafold_node::web_server::auth::web_auth_manager::{WebAuthManager, PublicKey, Signature, WebRequest};
use crate::datafold_node::web_server::types::ApiErrorResponse;
use crate::error::{FoldDbError, NetworkErrorKind};

/// Request with authentication headers
#[derive(Debug, Deserialize)]
pub struct AuthenticatedRequest {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Authentication headers for requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthHeaders {
    /// The public key used for authentication
    #[serde(rename = "X-Public-Key")]
    pub public_key: String,
    /// The signature of the request
    #[serde(rename = "X-Signature")]
    pub signature: String,
    /// Timestamp when the signature was created
    #[serde(rename = "X-Timestamp")]
    pub timestamp: u64,
    /// Nonce to prevent replay attacks
    #[serde(rename = "X-Nonce")]
    pub nonce: String,
}

/// Middleware for key verification
pub struct KeyVerificationMiddleware {
    auth_manager: Arc<Mutex<WebAuthManager>>,
}

impl KeyVerificationMiddleware {
    /// Create a new KeyVerificationMiddleware
    pub fn new(auth_manager: Arc<Mutex<WebAuthManager>>) -> Self {
        Self { auth_manager }
    }

    /// Create a warp filter that verifies request signatures
    pub fn with_auth(
        &self,
    ) -> impl Filter<Extract = (u32,), Error = Rejection> + Clone {
        let auth_manager = Arc::clone(&self.auth_manager);
        
        warp::header::headers_cloned()
            .and(warp::path::full())
            .and(warp::method())
            .and(warp::body::bytes())
            .and_then(move |headers: warp::http::HeaderMap, path: warp::path::FullPath, method: warp::http::Method, body: bytes::Bytes| {
                let auth_manager = Arc::clone(&auth_manager);
                
                async move {
                    // Extract authentication headers
                    let public_key = match headers.get("X-Public-Key") {
                        Some(value) => value.to_str().unwrap_or_default().to_string(),
                        None => return Err(warp::reject::custom(ApiErrorResponse::new("Missing X-Public-Key header"))),
                    };
                    
                    let signature = match headers.get("X-Signature") {
                        Some(value) => value.to_str().unwrap_or_default().to_string(),
                        None => return Err(warp::reject::custom(ApiErrorResponse::new("Missing X-Signature header"))),
                    };
                    
                    let timestamp = match headers.get("X-Timestamp") {
                        Some(value) => {
                            match value.to_str().unwrap_or_default().parse::<u64>() {
                                Ok(ts) => ts,
                                Err(_) => return Err(warp::reject::custom(ApiErrorResponse::new("Invalid X-Timestamp header"))),
                            }
                        },
                        None => return Err(warp::reject::custom(ApiErrorResponse::new("Missing X-Timestamp header"))),
                    };
                    
                    let nonce = match headers.get("X-Nonce") {
                        Some(value) => value.to_str().unwrap_or_default().to_string(),
                        None => return Err(warp::reject::custom(ApiErrorResponse::new("Missing X-Nonce header"))),
                    };
                    
                    // Create WebRequest for verification
                    let request = WebRequest {
                        public_key: PublicKey(public_key),
                        signature: Signature {
                            value: signature,
                            timestamp,
                            nonce,
                        },
                        path: path.as_str().to_string(),
                        method: method.as_str().to_string(),
                        body: if body.is_empty() { None } else { Some(String::from_utf8_lossy(&body).to_string()) },
                    };
                    
                    // Verify the request
                    let mut auth_manager = auth_manager.lock().await;
                    match auth_manager.verify_request(&request).await {
                        Ok(trust_level) => Ok(trust_level),
                        Err(e) => Err(warp::reject::custom(ApiErrorResponse::new(format!("Authentication failed: {}", e)))),
                    }
                }
            })
    }

    /// Extract and validate public key from headers
    pub fn extract_key(headers: &warp::http::HeaderMap) -> Result<PublicKey, Rejection> {
        match headers.get("X-Public-Key") {
            Some(value) => {
                let key_str = value.to_str().map_err(|_| {
                    warp::reject::custom(ApiErrorResponse::new("Invalid X-Public-Key header"))
                })?;
                
                Ok(PublicKey(key_str.to_string()))
            },
            None => Err(warp::reject::custom(ApiErrorResponse::new("Missing X-Public-Key header"))),
        }
    }

    /// Validate signature format and timestamp
    pub fn validate_signature(signature: &Signature) -> Result<(), Rejection> {
        // Check if the timestamp is recent (within 5 minutes)
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        if current_time - signature.timestamp > 300 {
            return Err(warp::reject::custom(ApiErrorResponse::new("Signature timestamp expired")));
        }
        
        // In a real implementation, we would also:
        // 1. Check signature format
        // 2. Verify nonce uniqueness
        
        Ok(())
    }
}

/// Warp filter that adds authentication to a route
pub fn with_auth(
    auth_manager: Arc<Mutex<WebAuthManager>>,
) -> impl Filter<Extract = (u32,), Error = Rejection> + Clone {
    KeyVerificationMiddleware::new(auth_manager).with_auth()
}

#[cfg(test)]
mod tests {
    use super::*;
    use warp::http::HeaderValue;
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_extract_key() {
        let mut headers = warp::http::HeaderMap::new();
        headers.insert("X-Public-Key", HeaderValue::from_static("test_key"));
        
        let result = KeyVerificationMiddleware::extract_key(&headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PublicKey("test_key".to_string()));
    }
    
    #[tokio::test]
    async fn test_validate_signature() {
        // Valid signature (recent timestamp)
        let valid_signature = Signature {
            value: "test_signature".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            nonce: "test_nonce".to_string(),
        };
        
        let result = KeyVerificationMiddleware::validate_signature(&valid_signature);
        assert!(result.is_ok());
        
        // Invalid signature (old timestamp)
        let invalid_signature = Signature {
            value: "test_signature".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() - 600, // 10 minutes ago
            nonce: "test_nonce".to_string(),
        };
        
        let result = KeyVerificationMiddleware::validate_signature(&invalid_signature);
        assert!(result.is_err());
    }
}
