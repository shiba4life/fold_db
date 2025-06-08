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
        let mut key_derivation = KeyDerivationConfig::for_security_level(request.security_level.clone());
        
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

                let mut key_derivation = KeyDerivationConfig::for_security_level(request.security_level.clone());
                
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