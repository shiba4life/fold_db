//! API request/response structures for DataFold CLI
//! 
//! This module contains all the API data structures used for communication
//! with DataFold servers, extracted from the main CLI binary for better
//! organization and reusability.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// API error structure from server responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: HashMap<String, serde_json::Value>,
}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

/// Public key registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyRegistrationRequest {
    pub client_id: Option<String>,
    pub user_id: Option<String>,
    pub public_key: String,
    pub key_name: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

/// Public key registration response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyRegistrationResponse {
    pub registration_id: String,
    pub client_id: String,
    pub public_key: String,
    pub key_name: Option<String>,
    pub registered_at: String,
    pub status: String,
}

/// Public key status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyStatusResponse {
    pub registration_id: String,
    pub client_id: String,
    pub public_key: String,
    pub key_name: Option<String>,
    pub registered_at: String,
    pub status: String,
    pub last_used: Option<String>,
}

/// Signature verification request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureVerificationRequest {
    pub client_id: String,
    pub message: String,
    pub signature: String,
    pub message_encoding: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

/// Signature verification response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureVerificationResponse {
    pub verified: bool,
    pub client_id: String,
    pub public_key: String,
    pub verified_at: String,
    pub message_hash: String,
}