//! HTTP API endpoints for cryptographic initialization
//!
//! This module provides REST endpoints for programmatic database cryptographic
//! initialization, including both random key generation and passphrase-based
//! key derivation workflows.

use crate::config::crypto::{CryptoConfig, MasterKeyConfig, KeyDerivationConfig, SecurityLevel};
use crate::datafold_node::crypto_init::{
    initialize_database_crypto, get_crypto_init_status, validate_crypto_config_for_init,
    is_crypto_init_needed, CryptoInitError
};
use crate::datafold_node::http_server::AppState;
use actix_web::{web, HttpResponse, Result as ActixResult};
use log::{info, warn, debug, error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use base64;

/// Standard API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: ApiError) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

/// API error response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: HashMap<String, serde_json::Value>,
}

impl ApiError {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details: HashMap::new(),
        }
    }

    pub fn with_details(mut self, key: &str, value: serde_json::Value) -> Self {
        self.details.insert(key.to_string(), value);
        self
    }
}

/// Convert CryptoInitError to ApiError
impl From<CryptoInitError> for ApiError {
    fn from(error: CryptoInitError) -> Self {
        match error {
            CryptoInitError::AlreadyInitialized => {
                ApiError::new("CRYPTO_ALREADY_INITIALIZED", "Database already has crypto metadata initialized")
            }
            CryptoInitError::InvalidConfig(msg) => {
                ApiError::new("INVALID_CRYPTO_CONFIG", &msg)
            }
            CryptoInitError::Config(config_error) => {
                ApiError::new("CONFIG_ERROR", &format!("Configuration error: {}", config_error))
            }
            CryptoInitError::Crypto(crypto_error) => {
                ApiError::new("CRYPTO_ERROR", &format!("Cryptographic operation failed: {}", crypto_error))
            }
            CryptoInitError::Database(db_error) => {
                ApiError::new("DATABASE_ERROR", &format!("Database operation failed: {}", db_error))
            }
            CryptoInitError::Sled(sled_error) => {
                ApiError::new("DATABASE_ERROR", &format!("Database error: {}", sled_error))
            }
        }
    }
}

/// Request body for passphrase-based crypto initialization
#[derive(Debug, Serialize, Deserialize)]
pub struct PassphraseInitRequest {
    pub passphrase: String,
    #[serde(default = "default_security_level")]
    pub security_level: SecurityLevel,
    pub custom_params: Option<CustomArgon2Params>,
}

fn default_security_level() -> SecurityLevel {
    SecurityLevel::Interactive
}

/// Custom Argon2 parameters for advanced users
#[derive(Debug, Serialize, Deserialize)]
pub struct CustomArgon2Params {
    pub memory_cost: Option<u32>,
    pub time_cost: Option<u32>,
    pub parallelism: Option<u32>,
}

/// Request body for crypto configuration validation
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateConfigRequest {
    pub method: String, // "random" or "passphrase"
    pub passphrase: Option<String>,
    #[serde(default = "default_security_level")]
    pub security_level: SecurityLevel,
    pub custom_params: Option<CustomArgon2Params>,
}

/// Response data for successful crypto initialization
#[derive(Debug, Serialize, Deserialize)]
pub struct CryptoInitResponse {
    pub initialized: bool,
    pub derivation_method: String,
    pub public_key: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Response data for crypto status endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct CryptoStatusResponse {
    pub initialized: bool,
    pub version: Option<u32>,
    pub algorithm: Option<String>,
    pub derivation_method: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub integrity_verified: Option<bool>,
}

/// Response data for crypto validation endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct CryptoValidationResponse {
    pub valid: bool,
    pub warnings: Vec<String>,
}

/// Request body for public key registration
#[derive(Debug, Serialize, Deserialize)]
pub struct PublicKeyRegistrationRequest {
    pub client_id: Option<String>,
    pub user_id: Option<String>,
    pub public_key: String, // Hex-encoded Ed25519 public key
    pub key_name: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

/// Response data for successful public key registration
#[derive(Debug, Serialize, Deserialize)]
pub struct PublicKeyRegistrationResponse {
    pub registration_id: String,
    pub client_id: String,
    pub public_key: String,
    pub key_name: Option<String>,
    pub registered_at: chrono::DateTime<chrono::Utc>,
    pub status: String,
}

/// Response data for public key status/lookup
#[derive(Debug, Serialize, Deserialize)]
pub struct PublicKeyStatusResponse {
    pub registration_id: String,
    pub client_id: String,
    pub public_key: String,
    pub key_name: Option<String>,
    pub registered_at: chrono::DateTime<chrono::Utc>,
    pub status: String,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

/// Request body for signature verification
#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureVerificationRequest {
    pub client_id: String,
    pub message: String, // Base64 or hex-encoded message data
    pub signature: String, // Hex-encoded Ed25519 signature (64 bytes)
    pub message_encoding: Option<String>, // "base64", "hex", or "utf8" (default: "utf8")
    pub metadata: Option<HashMap<String, String>>, // Optional context metadata
}

/// Response data for signature verification
#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureVerificationResponse {
    pub verified: bool,
    pub client_id: String,
    pub public_key: String,
    pub verified_at: chrono::DateTime<chrono::Utc>,
    pub message_hash: String, // SHA256 hash of the message for audit purposes
}

/// Stored public key registration record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyRegistration {
    pub registration_id: String,
    pub client_id: String,
    pub user_id: Option<String>,
    pub public_key_bytes: [u8; 32], // Ed25519 public key
    pub key_name: Option<String>,
    pub metadata: HashMap<String, String>,
    pub registered_at: chrono::DateTime<chrono::Utc>,
    pub status: String, // "active", "revoked", "suspended"
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

pub const PUBLIC_KEY_REGISTRATIONS_TREE: &str = "public_key_registrations";
pub const CLIENT_KEY_INDEX_TREE: &str = "client_key_index";

/// Initialize database crypto with randomly generated keys
/// 
/// POST /api/crypto/init/random
pub async fn init_random_key(
    app_state: web::Data<AppState>,
) -> ActixResult<HttpResponse> {
    info!("API request: Initialize crypto with random key");

    let response = async {
        // Get database operations
        let node = app_state.node.lock().await;
        let db = node.db.lock()
            .map_err(|_| ApiError::new("INTERNAL_ERROR", "Cannot lock database mutex"))?;
        let db_ops = db.db_ops();
        drop(db);
        drop(node);

        // Create crypto configuration for random key
        let crypto_config = CryptoConfig::with_random_key();

        // Check if crypto initialization is needed
        let needs_init = is_crypto_init_needed(db_ops.clone(), Some(&crypto_config))
            .map_err(ApiError::from)?;

        if !needs_init {
            return Err(ApiError::new("CRYPTO_ALREADY_INITIALIZED", "Database already has crypto metadata initialized"));
        }

        // Perform crypto initialization
        let context = initialize_database_crypto(db_ops, &crypto_config)
            .map_err(ApiError::from)?;

        // Extract values before moving context
        let derivation_method = context.derivation_method.clone();
        let public_key_bytes = context.public_key().to_bytes();
        let created_at = context.metadata().created_at;

        info!("Random key crypto initialization completed successfully");

        Ok(CryptoInitResponse {
            initialized: true,
            derivation_method,
            public_key: hex::encode(public_key_bytes),
            created_at,
        })
    }.await;

    match response {
        Ok(data) => {
            debug!("Random key initialization successful");
            Ok(HttpResponse::Ok().json(ApiResponse::success(data)))
        }
        Err(error) => {
            error!("Random key initialization failed: {}", error.message);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(error)))
        }
    }
}

/// Initialize database crypto with passphrase-derived keys
/// 
/// POST /api/crypto/init/passphrase
pub async fn init_passphrase_key(
    app_state: web::Data<AppState>,
    request: web::Json<PassphraseInitRequest>,
) -> ActixResult<HttpResponse> {
    info!("API request: Initialize crypto with passphrase-derived key");

    let response = async {
        // Validate passphrase
        if request.passphrase.is_empty() {
            return Err(ApiError::new("INVALID_PASSPHRASE", "Passphrase cannot be empty"));
        }

        if request.passphrase.len() > 1024 {
            return Err(ApiError::new("INVALID_PASSPHRASE", "Passphrase is too long (max 1024 characters)"));
        }

        // Get database operations
        let node = app_state.node.lock().await;
        let db = node.db.lock()
            .map_err(|_| ApiError::new("INTERNAL_ERROR", "Cannot lock database mutex"))?;
        let db_ops = db.db_ops();
        drop(db);
        drop(node);

        // Create crypto configuration for passphrase
        let mut key_derivation = KeyDerivationConfig::for_security_level(request.security_level);
        
        // Apply custom parameters if provided
        if let Some(custom_params) = &request.custom_params {
            if let Some(memory_cost) = custom_params.memory_cost {
                key_derivation.memory_cost = memory_cost;
            }
            if let Some(time_cost) = custom_params.time_cost {
                key_derivation.time_cost = time_cost;
            }
            if let Some(parallelism) = custom_params.parallelism {
                key_derivation.parallelism = parallelism;
            }
        }

        let crypto_config = CryptoConfig {
            enabled: true,
            master_key: MasterKeyConfig::Passphrase {
                passphrase: request.passphrase.clone(),
            },
            key_derivation,
        };

        // Validate configuration
        validate_crypto_config_for_init(&crypto_config)
            .map_err(ApiError::from)?;

        // Check if crypto initialization is needed
        let needs_init = is_crypto_init_needed(db_ops.clone(), Some(&crypto_config))
            .map_err(ApiError::from)?;

        if !needs_init {
            return Err(ApiError::new("CRYPTO_ALREADY_INITIALIZED", "Database already has crypto metadata initialized"));
        }

        // Perform crypto initialization
        let context = initialize_database_crypto(db_ops, &crypto_config)
            .map_err(ApiError::from)?;

        // Extract values before moving context
        let derivation_method = context.derivation_method.clone();
        let public_key_bytes = context.public_key().to_bytes();
        let created_at = context.metadata().created_at;

        info!("Passphrase-based crypto initialization completed successfully");

        Ok(CryptoInitResponse {
            initialized: true,
            derivation_method,
            public_key: hex::encode(public_key_bytes),
            created_at,
        })
    }.await;

    match response {
        Ok(data) => {
            debug!("Passphrase initialization successful");
            Ok(HttpResponse::Ok().json(ApiResponse::success(data)))
        }
        Err(error) => {
            error!("Passphrase initialization failed: {}", error.message);
            match error.code.as_str() {
                "CRYPTO_ALREADY_INITIALIZED" => {
                    Ok(HttpResponse::Conflict().json(ApiResponse::<()>::error(error)))
                }
                "INVALID_PASSPHRASE" | "INVALID_CRYPTO_CONFIG" => {
                    Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(error)))
                }
                _ => {
                    Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(error)))
                }
            }
        }
    }
}

/// Get current crypto initialization status
/// 
/// GET /api/crypto/status
pub async fn get_crypto_status(
    app_state: web::Data<AppState>,
) -> ActixResult<HttpResponse> {
    debug!("API request: Get crypto status");

    let response: Result<CryptoStatusResponse, ApiError> = async {
        // Get database operations
        let node = app_state.node.lock().await;
        let db = node.db.lock()
            .map_err(|_| ApiError::new("INTERNAL_ERROR", "Cannot lock database mutex"))?;
        let db_ops = db.db_ops();
        drop(db);
        drop(node);

        // Get crypto status
        let status = get_crypto_init_status(db_ops)
            .map_err(ApiError::from)?;

        Ok(CryptoStatusResponse {
            initialized: status.initialized,
            version: status.version,
            algorithm: status.algorithm,
            derivation_method: status.derivation_method,
            created_at: status.created_at,
            integrity_verified: status.integrity_verified,
        })
    }.await;

    match response {
        Ok(data) => {
            debug!("Crypto status retrieved successfully");
            Ok(HttpResponse::Ok().json(ApiResponse::success(data)))
        }
        Err(error) => {
            error!("Failed to get crypto status: {}", error.message);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(error)))
        }
    }
}

/// Validate crypto configuration without initializing
/// 
/// POST /api/crypto/validate
pub async fn validate_crypto_config(
    request: web::Json<ValidateConfigRequest>,
) -> ActixResult<HttpResponse> {
    debug!("API request: Validate crypto configuration");

    let response = async {
        let mut warnings = Vec::new();

        // Create crypto config based on method
        let crypto_config = match request.method.as_str() {
            "random" => CryptoConfig::with_random_key(),
            "passphrase" => {
                let passphrase = request.passphrase.clone()
                    .ok_or_else(|| ApiError::new("MISSING_PASSPHRASE", "Passphrase is required for passphrase method"))?;

                if passphrase.is_empty() {
                    return Err(ApiError::new("INVALID_PASSPHRASE", "Passphrase cannot be empty"));
                }

                if passphrase.len() < 8 {
                    warnings.push("Passphrase is shorter than recommended minimum of 8 characters".to_string());
                }

                if passphrase.len() < 12 {
                    warnings.push("Consider using a passphrase of at least 12 characters for better security".to_string());
                }

                if passphrase.len() > 1024 {
                    return Err(ApiError::new("INVALID_PASSPHRASE", "Passphrase is too long (max 1024 characters)"));
                }

                // Check for common weak patterns
                if passphrase.chars().all(|c| c.is_ascii_digit()) {
                    warnings.push("Passphrase contains only numbers - consider adding letters and symbols".to_string());
                }

                if passphrase.chars().all(|c| c.is_ascii_alphabetic()) {
                    warnings.push("Passphrase contains only letters - consider adding numbers and symbols".to_string());
                }

                let mut key_derivation = KeyDerivationConfig::for_security_level(request.security_level);
                
                // Apply custom parameters if provided
                if let Some(custom_params) = &request.custom_params {
                    if let Some(memory_cost) = custom_params.memory_cost {
                        key_derivation.memory_cost = memory_cost;
                    }
                    if let Some(time_cost) = custom_params.time_cost {
                        key_derivation.time_cost = time_cost;
                    }
                    if let Some(parallelism) = custom_params.parallelism {
                        key_derivation.parallelism = parallelism;
                    }
                }

                CryptoConfig {
                    enabled: true,
                    master_key: MasterKeyConfig::Passphrase { passphrase },
                    key_derivation,
                }
            }
            _ => {
                return Err(ApiError::new("INVALID_METHOD", "Method must be 'random' or 'passphrase'"));
            }
        };

        // Validate the configuration
        validate_crypto_config_for_init(&crypto_config)
            .map_err(ApiError::from)?;

        // Check security level warnings
        match request.security_level {
            SecurityLevel::Interactive => {
                warnings.push("Interactive security level is optimized for user experience - consider Balanced or Sensitive for higher security".to_string());
            }
            SecurityLevel::Balanced => {
                // Good balance, no warning needed
            }
            SecurityLevel::Sensitive => {
                warnings.push("Sensitive security level provides maximum security but may take longer to initialize".to_string());
            }
        }

        Ok(CryptoValidationResponse {
            valid: true,
            warnings,
        })
    }.await;

    match response {
        Ok(data) => {
            debug!("Crypto configuration validation successful");
            Ok(HttpResponse::Ok().json(ApiResponse::success(data)))
        }
        Err(error) => {
            warn!("Crypto configuration validation failed: {}", error.message);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(error)))
        }
    }
}

/// Register a client's public key for authentication and signature verification
///
/// POST /api/crypto/keys/register
pub async fn register_public_key(
    app_state: web::Data<AppState>,
    request: web::Json<PublicKeyRegistrationRequest>,
) -> ActixResult<HttpResponse> {
    info!("API request: Register public key for client authentication");

    let response = async {
        // Validate required fields
        if request.public_key.is_empty() {
            return Err(ApiError::new("INVALID_PUBLIC_KEY", "Public key cannot be empty"));
        }

        // Decode and validate public key
        let public_key_bytes = hex::decode(&request.public_key)
            .map_err(|_| ApiError::new("INVALID_PUBLIC_KEY", "Public key must be valid hex-encoded bytes"))?;
        
        if public_key_bytes.len() != 32 {
            return Err(ApiError::new("INVALID_PUBLIC_KEY", "Ed25519 public key must be exactly 32 bytes"));
        }

        let mut key_bytes_array = [0u8; 32];
        key_bytes_array.copy_from_slice(&public_key_bytes);

        // Validate the public key by creating a PublicKey instance
        let _public_key = crate::crypto::PublicKey::from_bytes(&key_bytes_array)
            .map_err(|_| ApiError::new("INVALID_PUBLIC_KEY", "Invalid Ed25519 public key format"))?;

        // Generate client ID if not provided
        let client_id = request.client_id.clone()
            .unwrap_or_else(|| format!("client_{}", Uuid::new_v4()));

        // Check if client already has a registered key
        let node = app_state.node.lock().await;
        let db = node.db.lock()
            .map_err(|_| ApiError::new("INTERNAL_ERROR", "Cannot lock database mutex"))?;
        let db_ops = db.db_ops();
        drop(db);
        drop(node);

        // Check for existing registration by client_id
        let client_key_lookup = format!("{}:{}", CLIENT_KEY_INDEX_TREE, &client_id);
        if let Ok(Some(_)) = db_ops.get_item::<String>(&client_key_lookup) {
            return Err(ApiError::new("CLIENT_ALREADY_REGISTERED",
                "Client already has a registered public key. Use update endpoint to change keys."));
        }

        // Check for duplicate public key
        let public_key_hash = {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(key_bytes_array);
            hex::encode(hasher.finalize())
        };

        // Search for existing public key by iterating through registrations
        let tree_prefix = format!("{}:", PUBLIC_KEY_REGISTRATIONS_TREE);
        for (_, value) in db_ops.db().scan_prefix(tree_prefix.as_bytes()).flatten() {
            if let Ok(registration) = serde_json::from_slice::<PublicKeyRegistration>(&value) {
                let existing_hash = {
                    use sha2::{Sha256, Digest};
                    let mut hasher = Sha256::new();
                    hasher.update(registration.public_key_bytes);
                    hex::encode(hasher.finalize())
                };
                if existing_hash == public_key_hash {
                    return Err(ApiError::new("DUPLICATE_PUBLIC_KEY",
                        "This public key is already registered by another client"));
                }
            }
        }

        // Create registration record
        let registration_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        
        let registration = PublicKeyRegistration {
            registration_id: registration_id.clone(),
            client_id: client_id.clone(),
            user_id: request.user_id.clone(),
            public_key_bytes: key_bytes_array,
            key_name: request.key_name.clone(),
            metadata: request.metadata.clone().unwrap_or_default(),
            registered_at: now,
            status: "active".to_string(),
            last_used: None,
        };

        // Store registration in database
        let registration_key = format!("{}:{}", PUBLIC_KEY_REGISTRATIONS_TREE, &registration_id);
        
        db_ops.store_item(&registration_key, &registration)
            .map_err(|e| ApiError::new("DATABASE_ERROR", &format!("Failed to store registration: {}", e)))?;

        // Store client -> registration_id index
        let client_index_key = format!("{}:{}", CLIENT_KEY_INDEX_TREE, &client_id);
        db_ops.store_item(&client_index_key, &registration_id)
            .map_err(|e| ApiError::new("DATABASE_ERROR", &format!("Failed to store client index: {}", e)))?;

        info!("Public key registered successfully for client: {}", client_id);

        Ok(PublicKeyRegistrationResponse {
            registration_id,
            client_id,
            public_key: request.public_key.clone(),
            key_name: request.key_name.clone(),
            registered_at: now,
            status: "active".to_string(),
        })
    }.await;

    match response {
        Ok(data) => {
            debug!("Public key registration successful");
            Ok(HttpResponse::Created().json(ApiResponse::success(data)))
        }
        Err(error) => {
            error!("Public key registration failed: {}", error.message);
            match error.code.as_str() {
                "CLIENT_ALREADY_REGISTERED" | "DUPLICATE_PUBLIC_KEY" => {
                    Ok(HttpResponse::Conflict().json(ApiResponse::<()>::error(error)))
                }
                "INVALID_PUBLIC_KEY" => {
                    Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(error)))
                }
                _ => {
                    Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(error)))
                }
            }
        }
    }
}

/// Get public key registration status by client ID
///
/// GET /api/crypto/keys/status/{client_id}
pub async fn get_public_key_status(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    let client_id = path.into_inner();
    debug!("API request: Get public key status for client: {}", client_id);

    let response: Result<PublicKeyStatusResponse, ApiError> = async {
        let node = app_state.node.lock().await;
        let db = node.db.lock()
            .map_err(|_| ApiError::new("INTERNAL_ERROR", "Cannot lock database mutex"))?;
        let db_ops = db.db_ops();
        drop(db);
        drop(node);

        // Look up registration ID by client ID
        let client_index_key = format!("{}:{}", CLIENT_KEY_INDEX_TREE, &client_id);
        let registration_id_str = db_ops.get_item::<String>(&client_index_key)
            .map_err(|e| ApiError::new("DATABASE_ERROR", &format!("Failed to lookup client: {}", e)))?
            .ok_or_else(|| ApiError::new("CLIENT_NOT_FOUND", "No public key registered for this client"))?;

        // Get registration record
        let registration_key = format!("{}:{}", PUBLIC_KEY_REGISTRATIONS_TREE, &registration_id_str);
        let registration: PublicKeyRegistration = db_ops.get_item(&registration_key)
            .map_err(|e| ApiError::new("DATABASE_ERROR", &format!("Failed to get registration: {}", e)))?
            .ok_or_else(|| ApiError::new("REGISTRATION_NOT_FOUND", "Registration record not found"))?;

        Ok(PublicKeyStatusResponse {
            registration_id: registration.registration_id,
            client_id: registration.client_id,
            public_key: hex::encode(registration.public_key_bytes),
            key_name: registration.key_name,
            registered_at: registration.registered_at,
            status: registration.status,
            last_used: registration.last_used,
        })
    }.await;

    match response {
        Ok(data) => {
            debug!("Public key status retrieved successfully");
            Ok(HttpResponse::Ok().json(ApiResponse::success(data)))
        }
        Err(error) => {
            error!("Failed to get public key status: {}", error.message);
            match error.code.as_str() {
                "CLIENT_NOT_FOUND" | "REGISTRATION_NOT_FOUND" => {
                    Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(error)))
                }
                _ => {
                    Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(error)))
                }
            }
        }
    }
}

/// Verify a digital signature from a registered client
///
/// POST /api/crypto/signatures/verify
pub async fn verify_signature(
    app_state: web::Data<AppState>,
    request: web::Json<SignatureVerificationRequest>,
) -> ActixResult<HttpResponse> {
    info!("API request: Verify digital signature for client: {}", request.client_id);

    let response = async {
        // Validate required fields
        if request.client_id.is_empty() {
            return Err(ApiError::new("INVALID_CLIENT_ID", "Client ID cannot be empty"));
        }

        if request.message.is_empty() {
            return Err(ApiError::new("INVALID_MESSAGE", "Message cannot be empty"));
        }

        if request.signature.is_empty() {
            return Err(ApiError::new("INVALID_SIGNATURE", "Signature cannot be empty"));
        }

        // Decode signature from hex
        let signature_bytes = hex::decode(&request.signature)
            .map_err(|_| ApiError::new("INVALID_SIGNATURE", "Signature must be valid hex-encoded bytes"))?;
        
        if signature_bytes.len() != 64 {
            return Err(ApiError::new("INVALID_SIGNATURE", "Ed25519 signature must be exactly 64 bytes"));
        }

        let mut signature_array = [0u8; 64];
        signature_array.copy_from_slice(&signature_bytes);

        // Decode message according to specified encoding
        let message_encoding = request.message_encoding.as_deref().unwrap_or("utf8");
        let message_bytes = match message_encoding {
            "utf8" => request.message.as_bytes().to_vec(),
            "hex" => hex::decode(&request.message)
                .map_err(|_| ApiError::new("INVALID_MESSAGE", "Invalid hex-encoded message"))?,
            "base64" => {
                use base64::{Engine as _, engine::general_purpose};
                general_purpose::STANDARD.decode(&request.message)
                    .map_err(|_| ApiError::new("INVALID_MESSAGE", "Invalid base64-encoded message"))?
            },
            _ => return Err(ApiError::new("INVALID_ENCODING", "Message encoding must be 'utf8', 'hex', or 'base64'")),
        };

        // Get database operations
        let node = app_state.node.lock().await;
        let db = node.db.lock()
            .map_err(|_| ApiError::new("INTERNAL_ERROR", "Cannot lock database mutex"))?;
        let db_ops = db.db_ops();
        drop(db);
        drop(node);

        // Look up registration by client ID
        let client_index_key = format!("{}:{}", CLIENT_KEY_INDEX_TREE, &request.client_id);
        let registration_id_str = db_ops.get_item::<String>(&client_index_key)
            .map_err(|e| ApiError::new("DATABASE_ERROR", &format!("Failed to lookup client: {}", e)))?
            .ok_or_else(|| ApiError::new("CLIENT_NOT_FOUND", "No public key registered for this client"))?;

        // Get registration record
        let registration_key = format!("{}:{}", PUBLIC_KEY_REGISTRATIONS_TREE, &registration_id_str);
        let mut registration: PublicKeyRegistration = db_ops.get_item(&registration_key)
            .map_err(|e| ApiError::new("DATABASE_ERROR", &format!("Failed to get registration: {}", e)))?
            .ok_or_else(|| ApiError::new("REGISTRATION_NOT_FOUND", "Registration record not found"))?;

        // Check if the public key is active
        if registration.status != "active" {
            return Err(ApiError::new("KEY_NOT_ACTIVE", &format!("Public key status is '{}', only 'active' keys can verify signatures", registration.status)));
        }

        // Create PublicKey instance for verification
        let public_key = crate::crypto::PublicKey::from_bytes(&registration.public_key_bytes)
            .map_err(|e| ApiError::new("INVALID_PUBLIC_KEY", &format!("Failed to load public key: {}", e)))?;

        // Verify the signature
        let verification_result = public_key.verify(&message_bytes, &signature_array);
        let verified = verification_result.is_ok();

        if !verified {
            warn!("Signature verification failed for client: {}", request.client_id);
            return Err(ApiError::new("SIGNATURE_VERIFICATION_FAILED", "Digital signature verification failed"));
        }

        // Update last_used timestamp
        let now = chrono::Utc::now();
        registration.last_used = Some(now);
        
        db_ops.store_item(&registration_key, &registration)
            .map_err(|e| ApiError::new("DATABASE_ERROR", &format!("Failed to update registration: {}", e)))?;

        // Compute message hash for audit trail
        let message_hash = {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(&message_bytes);
            hex::encode(hasher.finalize())
        };

        info!("Signature verification successful for client: {}", request.client_id);

        Ok(SignatureVerificationResponse {
            verified: true,
            client_id: request.client_id.clone(),
            public_key: hex::encode(registration.public_key_bytes),
            verified_at: now,
            message_hash,
        })
    }.await;

    match response {
        Ok(data) => {
            debug!("Signature verification successful");
            Ok(HttpResponse::Ok().json(ApiResponse::success(data)))
        }
        Err(error) => {
            error!("Signature verification failed: {}", error.message);
            match error.code.as_str() {
                "CLIENT_NOT_FOUND" | "REGISTRATION_NOT_FOUND" => {
                    Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(error)))
                }
                "INVALID_CLIENT_ID" | "INVALID_MESSAGE" | "INVALID_SIGNATURE" | "INVALID_ENCODING" => {
                    Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(error)))
                }
                "KEY_NOT_ACTIVE" => {
                    Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(error)))
                }
                "SIGNATURE_VERIFICATION_FAILED" => {
                    Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(error)))
                }
                _ => {
                    Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(error)))
                }
            }
        }
    }
}