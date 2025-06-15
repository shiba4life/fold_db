//! HTTP API endpoints for key rotation and replacement
//!
//! This module provides REST endpoints for secure key rotation operations including:
//! - Key rotation request validation
//! - Atomic key replacement
//! - Audit trail creation
//! - Error handling and recovery

use crate::crypto::audit_logger::{CryptoAuditLogger, OperationResult, SecurityEventDetails};
use crate::crypto::key_rotation::{
    KeyRotationRequest, KeyRotationValidator, RotationContext, RotationReason,
};
use crate::datafold_node::crypto_routes::{ApiError, ApiResponse};
use crate::datafold_node::http_server::AppState;
use crate::db_operations::key_rotation_operations::KeyRotationRecord;
use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
use chrono::{DateTime, Utc};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Request body for key rotation endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyRotationApiRequest {
    /// The complete key rotation request (includes signature)
    pub rotation_request: KeyRotationRequest,
    /// Optional actor identifier
    pub actor: Option<String>,
    /// Force rotation even with warnings (default: false)
    pub force: Option<bool>,
}

/// Response data for successful key rotation
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyRotationApiResponse {
    /// Whether the rotation was successful
    pub success: bool,
    /// New key identifier/fingerprint
    pub new_key_id: String,
    /// Confirmation that old key was invalidated
    pub old_key_invalidated: bool,
    /// Operation correlation ID for audit trail
    pub correlation_id: Uuid,
    /// Response timestamp
    pub timestamp: DateTime<Utc>,
    /// Any warnings or notes
    pub warnings: Vec<String>,
    /// Number of associations updated
    pub associations_updated: u64,
}

/// Request body for key rotation status lookup
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyRotationStatusRequest {
    /// Operation correlation ID
    pub correlation_id: Uuid,
}

/// Response data for key rotation status
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyRotationStatusResponse {
    /// Operation correlation ID
    pub correlation_id: Uuid,
    /// Current status of the rotation
    pub status: String,
    /// Old public key (hex encoded)
    pub old_public_key: String,
    /// New public key (hex encoded)
    pub new_public_key: String,
    /// Rotation reason
    pub reason: RotationReason,
    /// When the operation started
    pub started_at: DateTime<Utc>,
    /// When the operation completed (if applicable)
    pub completed_at: Option<DateTime<Utc>>,
    /// Number of associations updated
    pub associations_updated: u64,
    /// Error details (if failed)
    pub error_details: Option<String>,
}

/// Request body for key rotation history lookup
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyRotationHistoryRequest {
    /// Public key to get history for (hex encoded)
    pub public_key: String,
    /// Maximum number of records to return (default: 50)
    pub limit: Option<usize>,
}

/// Response data for key rotation history
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyRotationHistoryResponse {
    /// Public key that was queried
    pub public_key: String,
    /// List of rotation records
    pub rotations: Vec<KeyRotationHistoryEntry>,
    /// Total number of rotations found
    pub total_count: usize,
}

/// Individual rotation history entry
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyRotationHistoryEntry {
    /// Operation correlation ID
    pub correlation_id: Uuid,
    /// Operation status
    pub status: String,
    /// Old public key (hex encoded)
    pub old_public_key: String,
    /// New public key (hex encoded)
    pub new_public_key: String,
    /// Rotation reason
    pub reason: RotationReason,
    /// When the operation started
    pub started_at: DateTime<Utc>,
    /// When the operation completed
    pub completed_at: Option<DateTime<Utc>>,
    /// Actor who initiated the rotation
    pub actor: Option<String>,
}

/// Perform key rotation operation
///
/// POST /api/crypto/keys/rotate
pub async fn rotate_key(
    app_state: web::Data<AppState>,
    request: web::Json<KeyRotationApiRequest>,
    http_request: HttpRequest,
) -> ActixResult<HttpResponse> {
    info!("API request: Key rotation initiated");

    let start_time = std::time::Instant::now();
    let rotation_request = &request.rotation_request;

    // Log security event for key rotation attempt
    let client_ip = http_request
        .connection_info()
        .peer_addr()
        .unwrap_or("unknown")
        .to_string();

    #[allow(clippy::await_holding_lock)]
    let response = async {
        // Get database operations
        let node = app_state.node.lock().await;
        let db = node
            .db
            .lock()
            .map_err(|_| ApiError::new("INTERNAL_ERROR", "Cannot lock database mutex"))?;
        let db_ops = db.db_ops();
        drop(db);
        drop(node);

        // Create audit logger
        let audit_logger = CryptoAuditLogger::with_default_config();

        // Create validator
        let validator = KeyRotationValidator::new(Some(audit_logger.clone()));

        // Validate the rotation request
        debug!("Validating key rotation request");
        let validation_result = validator.validate_request(rotation_request).await;

        if !validation_result.is_valid {
            // Log security event for invalid rotation attempt
            let security_details = SecurityEventDetails {
                event_type: "invalid_key_rotation_attempt".to_string(),
                threat_level: "medium".to_string(),
                source: Some(client_ip.clone()),
                target: Some(hex::encode(rotation_request.old_public_key.to_bytes())),
                security_metadata: HashMap::from([
                    ("errors".to_string(), validation_result.errors.join("; ")),
                    (
                        "reason".to_string(),
                        format!("{:?}", rotation_request.reason),
                    ),
                ]),
            };

            audit_logger
                .log_security_event(
                    "invalid_key_rotation",
                    security_details,
                    OperationResult::Failure {
                        error_type: "ValidationError".to_string(),
                        error_message: validation_result.errors.join("; "),
                        error_code: Some("INVALID_ROTATION_REQUEST".to_string()),
                    },
                    None,
                )
                .await;

            return Err(ApiError::new(
                "INVALID_ROTATION_REQUEST",
                &validation_result.errors.join("; "),
            )
            .with_details(
                "validation_errors",
                serde_json::json!(validation_result.errors),
            ));
        }

        // Check for warnings and force flag
        if !validation_result.warnings.is_empty() && !request.force.unwrap_or(false) {
            return Err(ApiError::new(
                "ROTATION_WARNINGS",
                "Rotation has warnings - use force=true to proceed",
            )
            .with_details("warnings", serde_json::json!(validation_result.warnings)));
        }

        // Create rotation context
        let context = RotationContext::new(
            rotation_request.clone(),
            validation_result.clone(),
            request.actor.clone(),
        );

        // Log security event for successful validation
        let security_details = SecurityEventDetails {
            event_type: "key_rotation_validated".to_string(),
            threat_level: "info".to_string(),
            source: Some(client_ip),
            target: Some(hex::encode(rotation_request.old_public_key.to_bytes())),
            security_metadata: HashMap::from([
                (
                    "correlation_id".to_string(),
                    context.correlation_id.to_string(),
                ),
                (
                    "reason".to_string(),
                    format!("{:?}", rotation_request.reason),
                ),
                (
                    "actor".to_string(),
                    request.actor.clone().unwrap_or("anonymous".to_string()),
                ),
            ]),
        };

        audit_logger
            .log_security_event(
                "key_rotation_validated",
                security_details,
                OperationResult::Success,
                Some(context.correlation_id),
            )
            .await;

        // Perform the rotation
        info!(
            "Executing key rotation for correlation ID: {}",
            context.correlation_id
        );
        let rotation_result = db_ops.rotate_key(&context, Some(&audit_logger)).await;

        match rotation_result {
            Ok(response) => {
                info!(
                    "Key rotation completed successfully: {}",
                    context.correlation_id
                );

                Ok(KeyRotationApiResponse {
                    success: response.success,
                    new_key_id: response.new_key_id,
                    old_key_invalidated: response.old_key_invalidated,
                    correlation_id: context.correlation_id,
                    timestamp: response.timestamp,
                    warnings: response.warnings,
                    associations_updated: 0, // Will be populated by the actual operation
                })
            }
            Err(rotation_error) => {
                error!(
                    "Key rotation failed: {} - {}",
                    rotation_error.code, rotation_error.message
                );

                Err(ApiError::new(&rotation_error.code, &rotation_error.message)
                    .with_details(
                        "correlation_id",
                        serde_json::Value::String(context.correlation_id.to_string()),
                    )
                    .with_details("details", serde_json::json!(rotation_error.details)))
            }
        }
    }
    .await;

    let duration = start_time.elapsed();
    debug!("Key rotation request processed in {:?}", duration);

    match response {
        Ok(data) => {
            info!("Key rotation API request successful");
            Ok(HttpResponse::Ok().json(ApiResponse::success(data)))
        }
        Err(error) => {
            error!("Key rotation API request failed: {}", error.message);
            match error.code.as_str() {
                "INVALID_ROTATION_REQUEST" | "ROTATION_WARNINGS" => {
                    Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(error)))
                }
                "KEY_NOT_FOUND" => {
                    Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(error)))
                }
                "KEY_ALREADY_EXISTS" => {
                    Ok(HttpResponse::Conflict().json(ApiResponse::<()>::error(error)))
                }
                "TRANSACTION_FAILED" | "STORAGE_ERROR" => {
                    Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(error)))
                }
                _ => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(error))),
            }
        }
    }
}

/// Get key rotation operation status
///
/// POST /api/crypto/keys/rotate/status
pub async fn get_rotation_status(
    app_state: web::Data<AppState>,
    request: web::Json<KeyRotationStatusRequest>,
) -> ActixResult<HttpResponse> {
    debug!(
        "API request: Get key rotation status for {}",
        request.correlation_id
    );

    let response = async {
        // Get database operations
        let node = app_state.node.lock().await;
        let db = node
            .db
            .lock()
            .map_err(|_| ApiError::new("INTERNAL_ERROR", "Cannot lock database mutex"))?;
        let db_ops = db.db_ops();
        drop(db);
        drop(node);

        // Look up the rotation record
        match db_ops.get_rotation_record(&request.correlation_id) {
            Ok(Some(record)) => Ok(KeyRotationStatusResponse {
                correlation_id: record.operation_id,
                status: format!("{:?}", record.status),
                old_public_key: record.old_public_key,
                new_public_key: record.new_public_key,
                reason: record.reason,
                started_at: record.started_at,
                completed_at: record.completed_at,
                associations_updated: record.associations_updated,
                error_details: record.error_details,
            }),
            Ok(None) => Err(ApiError::new(
                "ROTATION_NOT_FOUND",
                "No rotation record found for the given correlation ID",
            )),
            Err(e) => Err(ApiError::new(
                "DATABASE_ERROR",
                &format!("Failed to retrieve rotation record: {}", e),
            )),
        }
    }
    .await;

    match response {
        Ok(data) => {
            debug!("Key rotation status retrieved successfully");
            Ok(HttpResponse::Ok().json(ApiResponse::success(data)))
        }
        Err(error) => {
            error!("Failed to get rotation status: {}", error.message);
            match error.code.as_str() {
                "ROTATION_NOT_FOUND" => {
                    Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(error)))
                }
                _ => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(error))),
            }
        }
    }
}

/// Get key rotation history for a public key
///
/// POST /api/crypto/keys/rotate/history
pub async fn get_rotation_history(
    app_state: web::Data<AppState>,
    request: web::Json<KeyRotationHistoryRequest>,
) -> ActixResult<HttpResponse> {
    debug!(
        "API request: Get key rotation history for key: {}",
        request.public_key
    );

    let response = async {
        // Validate hex-encoded public key
        if request.public_key.len() != 64 {
            return Err(ApiError::new(
                "INVALID_PUBLIC_KEY",
                "Public key must be 64 hex characters",
            ));
        }

        if hex::decode(&request.public_key).is_err() {
            return Err(ApiError::new(
                "INVALID_PUBLIC_KEY",
                "Public key must be valid hex",
            ));
        }

        // Get database operations
        let node = app_state.node.lock().await;
        let db = node
            .db
            .lock()
            .map_err(|_| ApiError::new("INTERNAL_ERROR", "Cannot lock database mutex"))?;
        let db_ops = db.db_ops();
        drop(db);
        drop(node);

        // Get rotation history
        match db_ops.get_rotation_history(&request.public_key) {
            Ok(records) => {
                let limit = request.limit.unwrap_or(50);
                let limited_records: Vec<KeyRotationRecord> =
                    records.into_iter().take(limit).collect();

                let history_entries: Vec<KeyRotationHistoryEntry> = limited_records
                    .iter()
                    .map(|record| KeyRotationHistoryEntry {
                        correlation_id: record.operation_id,
                        status: format!("{:?}", record.status),
                        old_public_key: record.old_public_key.clone(),
                        new_public_key: record.new_public_key.clone(),
                        reason: record.reason.clone(),
                        started_at: record.started_at,
                        completed_at: record.completed_at,
                        actor: record.actor.clone(),
                    })
                    .collect();

                Ok(KeyRotationHistoryResponse {
                    public_key: request.public_key.clone(),
                    rotations: history_entries,
                    total_count: limited_records.len(),
                })
            }
            Err(e) => Err(ApiError::new(
                "DATABASE_ERROR",
                &format!("Failed to retrieve rotation history: {}", e),
            )),
        }
    }
    .await;

    match response {
        Ok(data) => {
            debug!("Key rotation history retrieved successfully");
            Ok(HttpResponse::Ok().json(ApiResponse::success(data)))
        }
        Err(error) => {
            error!("Failed to get rotation history: {}", error.message);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(error)))
        }
    }
}

/// Get key rotation statistics
///
/// GET /api/crypto/keys/rotate/stats
pub async fn get_rotation_statistics(app_state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    debug!("API request: Get key rotation statistics");

    let response = async {
        // Get database operations
        let node = app_state.node.lock().await;
        let db = node
            .db
            .lock()
            .map_err(|_| ApiError::new("INTERNAL_ERROR", "Cannot lock database mutex"))?;
        let db_ops = db.db_ops();
        drop(db);
        drop(node);

        // Get rotation statistics
        match db_ops.get_rotation_statistics() {
            Ok(stats) => Ok(stats),
            Err(e) => Err(ApiError::new(
                "DATABASE_ERROR",
                &format!("Failed to retrieve rotation statistics: {}", e),
            )),
        }
    }
    .await;

    match response {
        Ok(data) => {
            debug!("Key rotation statistics retrieved successfully");
            Ok(HttpResponse::Ok().json(ApiResponse::success(data)))
        }
        Err(error) => {
            error!("Failed to get rotation statistics: {}", error.message);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(error)))
        }
    }
}

/// Validate a key rotation request without executing it
///
/// POST /api/crypto/keys/rotate/validate
pub async fn validate_rotation_request(
    request: web::Json<KeyRotationRequest>,
    _http_request: HttpRequest,
) -> ActixResult<HttpResponse> {
    debug!("API request: Validate key rotation request");

    // Create audit logger
    let audit_logger = CryptoAuditLogger::with_default_config();

    // Create validator
    let validator = KeyRotationValidator::new(Some(audit_logger));

    // Validate the rotation request
    let validation_result = validator.validate_request(&request).await;

    let request_id = match request.request_id() {
        Ok(id) => id,
        Err(_) => "unknown".to_string(),
    };

    let response_data = serde_json::json!({
        "valid": validation_result.is_valid,
        "errors": validation_result.errors,
        "warnings": validation_result.warnings,
        "request_id": request_id,
    });

    debug!("Key rotation validation completed");
    Ok(HttpResponse::Ok().json(ApiResponse::success(response_data)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::ed25519::generate_master_keypair;
    use crate::crypto::key_rotation::RotationReason;
    use std::collections::HashMap;

    #[test]
    fn test_key_rotation_api_request_serialization() {
        let old_keypair = generate_master_keypair().unwrap();
        let new_keypair = generate_master_keypair().unwrap();
        let old_private_key = old_keypair.private_key();

        let rotation_request = KeyRotationRequest::new(
            &old_private_key,
            new_keypair.public_key().clone(),
            RotationReason::Scheduled,
            Some("test-client".to_string()),
            HashMap::new(),
        )
        .unwrap();

        let api_request = KeyRotationApiRequest {
            rotation_request,
            actor: Some("test-actor".to_string()),
            force: Some(false),
        };

        // Test serialization
        let json = serde_json::to_string(&api_request).unwrap();
        assert!(!json.is_empty());

        // Test deserialization
        let deserialized: KeyRotationApiRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.actor, api_request.actor);
        assert_eq!(deserialized.force, api_request.force);
    }

    #[test]
    fn test_key_rotation_status_request_serialization() {
        let status_request = KeyRotationStatusRequest {
            correlation_id: Uuid::new_v4(),
        };

        let json = serde_json::to_string(&status_request).unwrap();
        let deserialized: KeyRotationStatusRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.correlation_id, status_request.correlation_id);
    }

    #[test]
    fn test_key_rotation_history_request_serialization() {
        let history_request = KeyRotationHistoryRequest {
            public_key: "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                .to_string(),
            limit: Some(25),
        };

        let json = serde_json::to_string(&history_request).unwrap();
        let deserialized: KeyRotationHistoryRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.public_key, history_request.public_key);
        assert_eq!(deserialized.limit, history_request.limit);
    }
}
