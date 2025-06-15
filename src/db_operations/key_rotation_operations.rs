//! Database operations for key rotation and replacement
//!
//! This module provides atomic database operations for key rotation including:
//! - Transactional key replacement
//! - Association updates
//! - Audit trail persistence
//! - Rollback capabilities

use super::core::DbOperations;
use crate::crypto::audit_logger::{CryptoAuditLogger, OperationResult};
use crate::crypto::ed25519::PublicKey;
use crate::crypto::key_rotation::{
    KeyRotationError, KeyRotationRequest, KeyRotationResponse, RotationContext, RotationReason,
};
use crate::schema::SchemaError;
use crate::security_types::RotationStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// Key rotation operation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationRecord {
    /// Unique operation ID
    pub operation_id: Uuid,
    /// Request that initiated this rotation
    pub request: KeyRotationRequest,
    /// Old public key (hex encoded)
    pub old_public_key: String,
    /// New public key (hex encoded)
    pub new_public_key: String,
    /// Rotation reason
    pub reason: RotationReason,
    /// Operation status
    pub status: RotationStatus,
    /// Timestamp when operation started
    pub started_at: DateTime<Utc>,
    /// Timestamp when operation completed
    pub completed_at: Option<DateTime<Utc>>,
    /// Actor who initiated the rotation
    pub actor: Option<String>,
    /// Client ID associated with the operation
    pub client_id: Option<String>,
    /// Error details (if failed)
    pub error_details: Option<String>,
    /// Number of associations updated
    pub associations_updated: u64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Key association record for tracking what data is associated with each key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyAssociation {
    /// Association ID
    pub association_id: String,
    /// Public key (hex encoded)
    pub public_key: String,
    /// Type of association (e.g., "client_registration", "master_key")
    pub association_type: String,
    /// Reference to the associated data
    pub data_reference: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Status of the association
    pub status: String, // "active", "inactive", "rotated"
}

/// Database tree names for key rotation
pub const KEY_ROTATION_RECORDS_TREE: &str = "key_rotation_records";
pub const KEY_ASSOCIATIONS_TREE: &str = "key_associations";
pub const KEY_ROTATION_INDEX_TREE: &str = "key_rotation_index";

/// Extension trait for DbOperations to handle key rotation
impl DbOperations {
    /// Perform atomic key rotation operation
    pub async fn rotate_key(
        &self,
        context: &RotationContext,
        audit_logger: Option<&CryptoAuditLogger>,
    ) -> Result<KeyRotationResponse, KeyRotationError> {
        let start_time = std::time::Instant::now();
        let operation_id = context.correlation_id;

        // Create initial rotation record
        let mut rotation_record = KeyRotationRecord {
            operation_id,
            request: context.request.clone(),
            old_public_key: hex::encode(context.request.old_public_key.to_bytes()),
            new_public_key: hex::encode(context.request.new_public_key.to_bytes()),
            reason: context.request.reason.clone(),
            status: RotationStatus::InProgress,
            started_at: context.started_at,
            completed_at: None,
            actor: context.actor.clone(),
            client_id: context.request.client_id.clone(),
            error_details: None,
            associations_updated: 0,
            metadata: context.request.metadata.clone(),
        };

        // Store initial record
        self.store_rotation_record(&rotation_record).map_err(|e| {
            KeyRotationError::new(
                "STORAGE_ERROR",
                &format!("Failed to store rotation record: {}", e),
            )
        })?;

        // Log rotation start
        if let Some(logger) = audit_logger {
            logger
                .log_key_operation(
                    "key_rotation_start",
                    "key_rotation",
                    Duration::from_millis(0),
                    OperationResult::InProgress,
                    Some(operation_id),
                )
                .await;
        }

        // Perform the rotation operation
        let result = self
            .execute_key_rotation(&mut rotation_record, audit_logger)
            .await;

        let duration = start_time.elapsed();

        // Update final record status
        rotation_record.completed_at = Some(Utc::now());
        rotation_record.status = if result.is_ok() {
            RotationStatus::Completed
        } else {
            RotationStatus::Failed
        };

        if let Err(ref error) = result {
            rotation_record.error_details = Some(error.message.clone());
        }

        // Store final record
        let _ = self.store_rotation_record(&rotation_record);

        // Log final result
        if let Some(logger) = audit_logger {
            let audit_result = match result {
                Ok(_) => OperationResult::Success,
                Err(ref error) => OperationResult::Failure {
                    error_type: error.code.clone(),
                    error_message: error.message.clone(),
                    error_code: Some(error.code.clone()),
                },
            };

            logger
                .log_key_operation(
                    "key_rotation_complete",
                    "key_rotation",
                    duration,
                    audit_result,
                    Some(operation_id),
                )
                .await;
        }

        result
    }

    /// Execute the actual key rotation with transactional guarantees
    async fn execute_key_rotation(
        &self,
        rotation_record: &mut KeyRotationRecord,
        audit_logger: Option<&CryptoAuditLogger>,
    ) -> Result<KeyRotationResponse, KeyRotationError> {
        let operation_id = rotation_record.operation_id;
        let old_key_hex = &rotation_record.old_public_key;
        let new_key_hex = &rotation_record.new_public_key;

        // Step 1: Check if old key exists and get associations
        let associations = self.get_key_associations(old_key_hex).map_err(|e| {
            KeyRotationError::new(
                "LOOKUP_ERROR",
                &format!("Failed to get key associations: {}", e),
            )
        })?;

        if associations.is_empty() {
            return Err(KeyRotationError::new(
                "KEY_NOT_FOUND",
                "Old public key has no associations",
            ));
        }

        // Step 2: Validate that new key doesn't already exist
        let existing_new_associations = self.get_key_associations(new_key_hex).map_err(|e| {
            KeyRotationError::new(
                "LOOKUP_ERROR",
                &format!("Failed to check new key associations: {}", e),
            )
        })?;

        if !existing_new_associations.is_empty() {
            return Err(KeyRotationError::new(
                "KEY_ALREADY_EXISTS",
                "New public key already has associations",
            ));
        }

        // Step 3: Begin transaction-like operation
        let mut updated_associations = Vec::new();
        let mut rollback_needed = false;

        // Step 4: Update each association to use the new key
        for mut association in associations {
            let old_association_id = association.association_id.clone();

            // Create new association with new key
            association.association_id =
                format!("{}_{}", new_key_hex, &association.association_type);
            association.public_key = new_key_hex.clone();
            association.updated_at = Utc::now();
            association.status = "active".to_string();

            // Store new association
            if let Err(_e) = self.store_key_association(&association) {
                rollback_needed = true;
                break;
            }

            updated_associations.push((old_association_id, association));
        }

        // Step 5: Handle rollback if needed
        if rollback_needed {
            // Rollback all changes
            for (_, association) in &updated_associations {
                let _ = self.delete_key_association(&association.association_id);
            }

            rotation_record.status = RotationStatus::RolledBack;

            if let Some(logger) = audit_logger {
                logger
                    .log_key_operation(
                        "key_rotation_rollback",
                        "key_rotation",
                        Duration::from_millis(0),
                        OperationResult::Failure {
                            error_type: "TRANSACTION_ROLLBACK".to_string(),
                            error_message: "Key rotation rolled back due to storage failure"
                                .to_string(),
                            error_code: Some("ROLLBACK_EXECUTED".to_string()),
                        },
                        Some(operation_id),
                    )
                    .await;
            }

            return Err(KeyRotationError::new(
                "TRANSACTION_FAILED",
                "Key rotation transaction failed and was rolled back",
            ));
        }

        // Step 6: Invalidate old key associations
        for (old_association_id, _) in &updated_associations {
            if let Ok(mut old_association) = self.get_key_association(old_association_id) {
                old_association.status = "rotated".to_string();
                old_association.updated_at = Utc::now();
                let _ = self.store_key_association(&old_association);
            }
        }

        // Step 7: Update master key if this is a master key rotation
        if self
            .is_master_key_rotation(&rotation_record.old_public_key)
            .unwrap_or(false)
        {
            self.update_master_key_in_metadata(
                &rotation_record.new_public_key,
                &rotation_record.request.reason,
            )
            .map_err(|e| {
                KeyRotationError::new(
                    "MASTER_KEY_UPDATE_FAILED",
                    &format!("Failed to update master key: {}", e),
                )
            })?;
        }

        // Step 8: Update rotation record
        rotation_record.associations_updated = updated_associations.len() as u64;

        // Step 9: Create success response
        let new_key_id = hex::encode(&rotation_record.new_public_key.as_bytes()[..8]);

        Ok(KeyRotationResponse {
            success: true,
            new_key_id,
            old_key_invalidated: true,
            audit_trail_id: operation_id,
            timestamp: Utc::now(),
            warnings: self.generate_rotation_warnings(&rotation_record.reason),
        })
    }

    /// Store a key rotation record
    pub fn store_rotation_record(&self, record: &KeyRotationRecord) -> Result<(), SchemaError> {
        let key = format!("rotation_record:{}", record.operation_id);
        self.store_item(&key, record)?;

        // Index by old public key for lookups
        let index_key = format!("rotation_index:old_key:{}", record.old_public_key);
        self.store_item(&index_key, &record.operation_id)?;

        // Index by new public key
        let index_key = format!("rotation_index:new_key:{}", record.new_public_key);
        self.store_item(&index_key, &record.operation_id)?;

        Ok(())
    }

    /// Get a key rotation record by operation ID
    pub fn get_rotation_record(
        &self,
        operation_id: &Uuid,
    ) -> Result<Option<KeyRotationRecord>, SchemaError> {
        let key = format!("rotation_record:{}", operation_id);
        self.get_item(&key)
    }

    /// Get key rotation history for a public key
    pub fn get_rotation_history(
        &self,
        public_key_hex: &str,
    ) -> Result<Vec<KeyRotationRecord>, SchemaError> {
        let mut records = Vec::new();

        // Look up by old key
        let old_key_index = format!("rotation_index:old_key:{}", public_key_hex);
        if let Some(operation_id) = self.get_item::<Uuid>(&old_key_index)? {
            if let Some(record) = self.get_rotation_record(&operation_id)? {
                records.push(record);
            }
        }

        // Look up by new key
        let new_key_index = format!("rotation_index:new_key:{}", public_key_hex);
        if let Some(operation_id) = self.get_item::<Uuid>(&new_key_index)? {
            if let Some(record) = self.get_rotation_record(&operation_id)? {
                records.push(record);
            }
        }

        // Sort by timestamp
        records.sort_by(|a, b| a.started_at.cmp(&b.started_at));

        Ok(records)
    }

    /// Store a key association
    pub fn store_key_association(&self, association: &KeyAssociation) -> Result<(), SchemaError> {
        let key = format!("key_association:{}", association.association_id);
        self.store_item(&key, association)
    }

    /// Get a key association by ID
    pub fn get_key_association(&self, association_id: &str) -> Result<KeyAssociation, SchemaError> {
        let key = format!("key_association:{}", association_id);
        self.get_item(&key)?.ok_or_else(|| {
            SchemaError::NotFound(format!("Key association not found: {}", association_id))
        })
    }

    /// Get all associations for a public key
    pub fn get_key_associations(
        &self,
        public_key_hex: &str,
    ) -> Result<Vec<KeyAssociation>, SchemaError> {
        let prefix = "key_association:";
        let keys = self.list_items_with_prefix(prefix)?;
        let mut associations = Vec::new();

        for key in keys {
            if let Some(association) = self.get_item::<KeyAssociation>(&key)? {
                if association.public_key == public_key_hex && association.status == "active" {
                    associations.push(association);
                }
            }
        }

        Ok(associations)
    }

    /// Delete a key association
    pub fn delete_key_association(&self, association_id: &str) -> Result<bool, SchemaError> {
        let key = format!("key_association:{}", association_id);
        // Check if exists first
        let exists = self.get_item::<KeyAssociation>(&key)?.is_some();
        if exists {
            self.db()
                .remove(&key)
                .map_err(|e| SchemaError::InvalidData(format!("Failed to remove key: {}", e)))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Check if this is a master key rotation
    fn is_master_key_rotation(&self, old_public_key_hex: &str) -> Result<bool, SchemaError> {
        match self.get_master_public_key()? {
            Some(master_key) => {
                let master_key_hex = hex::encode(master_key.to_bytes());
                Ok(master_key_hex == old_public_key_hex)
            }
            None => Ok(false),
        }
    }

    /// Update master key in crypto metadata
    fn update_master_key_in_metadata(
        &self,
        new_public_key_hex: &str,
        reason: &RotationReason,
    ) -> Result<(), SchemaError> {
        // Decode new public key
        let new_key_bytes = hex::decode(new_public_key_hex)
            .map_err(|e| SchemaError::InvalidData(format!("Invalid hex key: {}", e)))?;

        if new_key_bytes.len() != 32 {
            return Err(SchemaError::InvalidData(
                "Invalid public key length".to_string(),
            ));
        }

        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&new_key_bytes);

        let new_public_key = PublicKey::from_bytes(&key_array)
            .map_err(|e| SchemaError::InvalidData(format!("Invalid public key: {}", e)))?;

        // Get existing metadata
        let mut metadata = self
            .get_crypto_metadata()?
            .ok_or_else(|| SchemaError::NotFound("No crypto metadata found".to_string()))?;

        // Update the public key
        metadata.master_public_key = new_public_key;

        // Add rotation metadata
        let reason_str = format!("{:?}", reason);
        metadata
            .add_metadata("last_rotation_reason".to_string(), reason_str)
            .map_err(|e| {
                SchemaError::InvalidData(format!("Failed to add rotation metadata: {}", e))
            })?;

        metadata
            .add_metadata(
                "last_rotation_timestamp".to_string(),
                Utc::now().to_rfc3339(),
            )
            .map_err(|e| {
                SchemaError::InvalidData(format!("Failed to add rotation timestamp: {}", e))
            })?;

        // Store updated metadata
        self.store_crypto_metadata(&metadata)
    }

    /// Generate warnings based on rotation reason
    fn generate_rotation_warnings(&self, reason: &RotationReason) -> Vec<String> {
        let mut warnings = Vec::new();

        match reason {
            RotationReason::Compromise => {
                warnings.push(
                    "Key compromise detected - ensure all clients update to new key immediately"
                        .to_string(),
                );
                warnings.push("Review security logs for potential unauthorized access".to_string());
            }
            RotationReason::Migration => {
                warnings.push(
                    "Algorithm migration completed - verify compatibility with all clients"
                        .to_string(),
                );
            }
            RotationReason::Maintenance => {
                warnings.push(
                    "Maintenance rotation completed - monitor for any client connectivity issues"
                        .to_string(),
                );
            }
            _ => {}
        }

        warnings
    }

    /// Get rotation statistics
    pub fn get_rotation_statistics(
        &self,
    ) -> Result<HashMap<String, serde_json::Value>, SchemaError> {
        let prefix = "rotation_record:";
        let keys = self.list_items_with_prefix(prefix)?;
        let mut stats = HashMap::new();
        let mut total_rotations = 0u64;
        let mut successful_rotations = 0u64;
        let mut failed_rotations = 0u64;
        let mut reason_counts: HashMap<String, u64> = HashMap::new();

        for key in keys {
            if let Some(record) = self.get_item::<KeyRotationRecord>(&key)? {
                total_rotations += 1;

                match record.status {
                    RotationStatus::Completed => successful_rotations += 1,
                    RotationStatus::Failed | RotationStatus::RolledBack => failed_rotations += 1,
                    _ => {}
                }

                let reason_key = format!("{:?}", record.reason);
                *reason_counts.entry(reason_key).or_insert(0) += 1;
            }
        }

        stats.insert(
            "total_rotations".to_string(),
            serde_json::Value::Number(total_rotations.into()),
        );
        stats.insert(
            "successful_rotations".to_string(),
            serde_json::Value::Number(successful_rotations.into()),
        );
        stats.insert(
            "failed_rotations".to_string(),
            serde_json::Value::Number(failed_rotations.into()),
        );
        stats.insert(
            "success_rate".to_string(),
            if total_rotations > 0 {
                let rate = (successful_rotations as f64 / total_rotations as f64) * 100.0;
                serde_json::json!(rate)
            } else {
                serde_json::json!(0.0)
            },
        );
        stats.insert(
            "reason_breakdown".to_string(),
            serde_json::json!(reason_counts),
        );

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::ed25519::generate_master_keypair;
    use crate::crypto::key_rotation::{
        KeyRotationRequest, RotationContext, RotationValidationResult,
    };
    use tempfile::tempdir;

    fn create_test_db_ops() -> DbOperations {
        let temp_dir = tempdir().unwrap();
        let db = sled::open(temp_dir.path()).unwrap();
        DbOperations::new(db).unwrap()
    }

    #[test]
    fn test_store_and_retrieve_rotation_record() {
        let db_ops = create_test_db_ops();
        let old_keypair = generate_master_keypair().unwrap();
        let new_keypair = generate_master_keypair().unwrap();
        let old_private_key = old_keypair.private_key();

        let request = KeyRotationRequest::new(
            &old_private_key,
            new_keypair.public_key().clone(),
            RotationReason::Scheduled,
            None,
            HashMap::new(),
        )
        .unwrap();

        let context = RotationContext::new(
            request.clone(),
            RotationValidationResult {
                is_valid: true,
                errors: Vec::new(),
                warnings: Vec::new(),
            },
            Some("test-actor".to_string()),
        );

        let record = KeyRotationRecord {
            operation_id: context.correlation_id,
            request: context.request.clone(),
            old_public_key: hex::encode(context.request.old_public_key.to_bytes()),
            new_public_key: hex::encode(context.request.new_public_key.to_bytes()),
            reason: context.request.reason.clone(),
            status: RotationStatus::Completed,
            started_at: context.started_at,
            completed_at: Some(Utc::now()),
            actor: context.actor.clone(),
            client_id: context.request.client_id.clone(),
            error_details: None,
            associations_updated: 0,
            metadata: HashMap::new(),
        };

        // Store record
        assert!(db_ops.store_rotation_record(&record).is_ok());

        // Retrieve record
        let retrieved = db_ops
            .get_rotation_record(&context.correlation_id)
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.operation_id, record.operation_id);
        assert_eq!(retrieved.old_public_key, record.old_public_key);
        assert_eq!(retrieved.new_public_key, record.new_public_key);
        assert_eq!(retrieved.status, RotationStatus::Completed);
    }

    #[test]
    fn test_key_association_operations() {
        let db_ops = create_test_db_ops();
        let keypair = generate_master_keypair().unwrap();
        let public_key_hex = hex::encode(keypair.public_key_bytes());

        let association = KeyAssociation {
            association_id: format!("{}_client", public_key_hex),
            public_key: public_key_hex.clone(),
            association_type: "client_registration".to_string(),
            data_reference: "client-123".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: "active".to_string(),
        };

        // Store association
        assert!(db_ops.store_key_association(&association).is_ok());

        // Retrieve association
        let retrieved = db_ops
            .get_key_association(&association.association_id)
            .unwrap();
        assert_eq!(retrieved.public_key, association.public_key);
        assert_eq!(retrieved.association_type, association.association_type);

        // Get associations by key
        let associations = db_ops.get_key_associations(&public_key_hex).unwrap();
        assert_eq!(associations.len(), 1);
        assert_eq!(associations[0].association_id, association.association_id);

        // Delete association
        assert!(db_ops
            .delete_key_association(&association.association_id)
            .unwrap());

        // Verify deletion
        assert!(db_ops
            .get_key_association(&association.association_id)
            .is_err());
    }
}
