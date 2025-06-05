//! Common test utilities for integration tests
//!
//! This module provides test fixtures directly for use by integration tests.
//! Each integration test file is a separate crate, so we can't import from test_utils.rs.

use fold_node::db_operations::DbOperations;
use fold_node::fold_db_core::infrastructure::message_bus::MessageBus;
use fold_node::fold_db_core::transform_manager::TransformManager;
use fold_node::datafold_node::config::NodeConfig;
use fold_node::datafold_node::DataFoldNode;
use fold_node::schema::types::SchemaError;
use std::sync::Arc;
use tempfile::TempDir;
use uuid::Uuid;

/// Basic test fixture for unit tests
pub struct TestFixture {
    pub transform_manager: Arc<TransformManager>,
    pub message_bus: Arc<MessageBus>,
    pub db_ops: Arc<DbOperations>,
    pub _temp_dir: TempDir,
}

/// Extended test fixture for full integration testing with schemas
pub struct CommonTestFixture {
    pub common: TestFixture,
    pub node: DataFoldNode,
    pub _temp_dir: TempDir,
}

impl TestFixture {
    /// Create a new test fixture with standard database and message bus setup
    pub fn new() -> Result<Self, SchemaError> {
        let temp_dir = tempfile::tempdir().map_err(|e| {
            SchemaError::InvalidData(format!("Failed to create temp directory: {}", e))
        })?;
        
        // Unified database setup
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

        // Unified MessageBus creation
        let message_bus = Arc::new(MessageBus::new());
        
        let transform_manager = TransformManager::new(
            Arc::clone(&db_ops),
            Arc::clone(&message_bus),
        )?;

        Ok(Self {
            transform_manager: Arc::new(transform_manager),
            message_bus,
            db_ops,
            _temp_dir: temp_dir,
        })
    }
}

impl CommonTestFixture {
    /// Create test fixture with schemas loaded for end-to-end testing
    pub async fn new_with_schemas() -> Result<CommonTestFixture, SchemaError> {
        let temp_dir = tempfile::tempdir().map_err(|e| {
            SchemaError::InvalidData(format!("Failed to create temp directory: {}", e))
        })?;

        // Unified NodeConfig setup
        let config = NodeConfig::new(temp_dir.path().to_path_buf());
        let node = DataFoldNode::load(config).await.map_err(|e| {
            SchemaError::InvalidData(format!("Failed to load DataFoldNode: {}", e))
        })?;

        let node_clone = node.clone();
        {
            let fold_db = node_clone.get_fold_db().map_err(|e|
                SchemaError::InvalidData(format!("Failed to get FoldDB from node: {}", e))
            )?;

            // Set up schemas for transform testing
            fold_db.schema_manager().approve_schema("TransformBase")
                .map_err(|e| SchemaError::InvalidData(format!("Failed to approve TransformBase schema: {}", e)))?;
            fold_db.schema_manager().approve_schema("TransformSchema")
                .map_err(|e| SchemaError::InvalidData(format!("Failed to approve TransformSchema schema: {}", e)))?;

            // Reload transforms to pick up new schemas
            fold_db.transform_manager().reload_transforms()
                .map_err(|e| SchemaError::InvalidData(format!("Failed to reload transforms: {}", e)))?;
        } // Drop the lock here

        // Wait for async initialization
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        Ok(Self::new_from_node(node, temp_dir))
    }

    /// Create new CommonTestFixture for simple cases
    pub fn new() -> Result<Self, SchemaError> {
        let temp_dir = tempfile::tempdir().map_err(|e| {
            SchemaError::InvalidData(format!("Failed to create temp directory: {}", e))
        })?;
        
        let basic_fixture = TestFixture::new()?;
        
        // Create a minimal node for compatibility
        let config = NodeConfig::new(temp_dir.path().to_path_buf());
        let node = DataFoldNode::new(config).map_err(|e| {
            SchemaError::InvalidData(format!("Failed to create DataFoldNode: {}", e))
        })?;

        Ok(Self {
            common: basic_fixture,
            node,
            _temp_dir: temp_dir,
        })
    }

    /// Create from existing DataFoldNode
    fn new_from_node(node: DataFoldNode, temp_dir: TempDir) -> Self {
        let node_clone = node.clone();
        let fold_db = node_clone.get_fold_db().expect("FoldDB should be available");
        
        // Extract components for common fixture
        let message_bus = fold_db.message_bus().clone();
        let transform_manager = fold_db.transform_manager().clone();
        let db_ops = fold_db.db_ops();

        let common = TestFixture {
            transform_manager,
            message_bus,
            db_ops,
            _temp_dir: tempfile::tempdir().expect("Should create temp dir"),
        };

        Self {
            common,
            node,
            _temp_dir: temp_dir,
        }
    }

    /// Create a sample transform registration for testing
    pub fn create_sample_registration() -> fold_node::schema::types::TransformRegistration {
        use fold_node::schema::types::{TransformRegistration, Transform};

        let transform = Transform {
            inputs: vec!["input1".to_string()],
            logic: "input1 + 10".to_string(),
            output: "result".to_string(),
            parsed_expression: None,
        };

        TransformRegistration {
            transform_id: "test_transform".to_string(),
            transform,
            input_arefs: vec!["input1_aref".to_string()],
            input_names: vec!["input1".to_string()],
            trigger_fields: vec!["input1".to_string()],
            output_aref: "result_aref".to_string(),
            schema_name: "test_schema".to_string(),
            field_name: "result".to_string(),
        }
    }
}

/// Generate unique correlation IDs for test tracking
pub fn generate_test_correlation_id() -> String {
    format!("test-{}", Uuid::new_v4())
}

/// Wait for async operations to complete with timeout
pub async fn wait_for_async_operation() {
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}