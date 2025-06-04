//! Common test utilities and fixtures for TransformManager tests
//!
//! This module provides shared functionality for both integration and unit tests,
//! including test fixtures, mock data generation, and helper functions.

use fold_node::db_operations::DbOperations;
use fold_node::fold_db_core::infrastructure::message_bus::MessageBus;
use fold_node::fold_db_core::transform_manager::TransformManager;
use fold_node::schema::types::{Transform, TransformRegistration, SchemaError};
use std::sync::Arc;
use tempfile::TempDir;

/// Common test fixture that can be used across integration and unit tests
pub struct CommonTestFixture {
    pub transform_manager: TransformManager,
    pub message_bus: Arc<MessageBus>,
    pub db_ops: Arc<DbOperations>,
    pub _temp_dir: TempDir,
}

impl CommonTestFixture {
    /// Create a new test fixture with initialized components
    pub fn new() -> Result<Self, SchemaError> {
        let temp_dir = tempfile::tempdir().map_err(|e| {
            SchemaError::InvalidData(format!("Failed to create temp directory: {}", e))
        })?;
        
        // Create temporary sled database
        let db = sled::Config::new()
            .path(temp_dir.path())
            .temporary(true)
            .open()
            .map_err(|e| {
                SchemaError::InvalidData(format!("Failed to open temporary database: {}", e))
            })?;
        let db_ops = Arc::new(DbOperations::new(db).map_err(|e| {
            SchemaError::InvalidData(format!("Failed to create DbOperations: {}", e))
        })?);
        let message_bus = Arc::new(MessageBus::new());
        
        let transform_manager = TransformManager::new(
            Arc::clone(&db_ops),
            Arc::clone(&message_bus),
        )?;

        Ok(Self {
            transform_manager,
            message_bus,
            db_ops,
            _temp_dir: temp_dir,
        })
    }

    /// Create a sample transform for testing
    pub fn create_sample_transform() -> Transform {
        Transform::new(
            "input1".to_string(),
            "test.output".to_string(),
        )
    }

    /// Create a sample transform registration with all required fields
    pub fn create_sample_registration() -> TransformRegistration {
        TransformRegistration {
            transform_id: "test_transform".to_string(),
            transform: Self::create_sample_transform(),
            input_arefs: vec!["aref1".to_string(), "aref2".to_string()],
            input_names: vec!["input1".to_string(), "input2".to_string()],
            trigger_fields: vec!["test.field1".to_string(), "test.field2".to_string()],
            output_aref: "output_aref".to_string(),
            schema_name: "test".to_string(),
            field_name: "output".to_string(),
        }
    }

    /// Create a transform with a specific ID for testing
    pub fn create_named_transform(transform_id: &str) -> Transform {
        Transform::new(
            "input1".to_string(),
            format!("test.{}", transform_id).to_string(),
        )
    }

    /// Create a registration with a specific transform ID
    pub fn create_named_registration(transform_id: &str) -> TransformRegistration {
        TransformRegistration {
            transform_id: transform_id.to_string(),
            transform: Self::create_named_transform(transform_id),
            input_arefs: vec![format!("{}_aref1", transform_id), format!("{}_aref2", transform_id)],
            input_names: vec!["input1".to_string(), "input2".to_string()],
            trigger_fields: vec![format!("test.{}_field", transform_id)],
            output_aref: format!("{}_output_aref", transform_id),
            schema_name: "test".to_string(),
            field_name: transform_id.to_string(),
        }
    }
}

/// Generate test correlation IDs for event testing
pub fn generate_test_correlation_id(prefix: &str) -> String {
    format!("{}_{}", prefix, uuid::Uuid::new_v4())
}

/// Helper function to wait for async operations in tests
pub async fn wait_for_async_operation() {
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}