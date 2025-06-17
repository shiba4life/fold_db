//! Transform orchestration system setup and management
//!
//! This module handles:
//! - Orchestration system initialization
//! - Event monitor setup
//! - Transform runner implementation
//! - Event-driven execution coordination

use super::config::*;
use crate::db_operations::DbOperations;
use crate::fold_db_core::infrastructure::message_bus::MessageBus;
use crate::fold_db_core::transform_manager::types::TransformRunner;
use crate::schema::types::SchemaError;
use log::{error, info};
use serde_json::Value as JsonValue;
use std::collections::HashSet;
use std::sync::Arc;
use once_cell::sync::Lazy;
use std::sync::Mutex;

/// Static storage for the event monitor to prevent it from being dropped
static EVENT_MONITOR: Lazy<
    Mutex<Option<crate::fold_db_core::orchestration::event_monitor::EventMonitor>>,
> = Lazy::new(|| Mutex::new(None));

/// Transform orchestration system manager
pub struct TransformOrchestrationManager {
    db_ops: Arc<DbOperations>,
    message_bus: Arc<MessageBus>,
}

impl TransformOrchestrationManager {
    /// Create a new orchestration manager
    pub fn new(db_ops: Arc<DbOperations>, message_bus: Arc<MessageBus>) -> Self {
        Self { db_ops, message_bus }
    }

    /// Start the orchestration system to handle TransformTriggered events
    pub fn start_orchestration_system(&self) -> Result<(), SchemaError> {
        info!("🚀 Starting orchestration system for TransformTriggered event handling");

        // Create a temporary tree for the orchestration system
        let temp_config = sled::Config::new().temporary(true);
        let temp_db = temp_config.open().map_err(|e| {
            SchemaError::InvalidData(format!(
                "Failed to create temporary database for orchestration: {}",
                e
            ))
        })?;
        let tree = temp_db.open_tree("orchestration").map_err(|e| {
            SchemaError::InvalidData(format!("Failed to create orchestration tree: {}", e))
        })?;

        // Create a simple transform runner wrapper for the manager
        let transform_runner = Arc::new(SimpleTransformRunner::new(Arc::clone(&self.db_ops)));

        // Start the EventMonitor to handle TransformTriggered events
        let persistence =
            crate::fold_db_core::orchestration::persistence_manager::PersistenceManager::new(tree);
        let event_monitor = crate::fold_db_core::orchestration::event_monitor::EventMonitor::new(
            Arc::clone(&self.message_bus),
            transform_runner,
            persistence,
        );

        // Store the event monitor in a static variable so it doesn't get dropped
        if let Ok(mut monitor) = EVENT_MONITOR.lock() {
            *monitor = Some(event_monitor);
            info!("✅ Orchestration system started successfully");
        } else {
            error!("❌ Failed to store EventMonitor in static variable");
            return Err(SchemaError::InvalidData(
                "Failed to store EventMonitor in static variable".to_string(),
            ));
        }

        Ok(())
    }

    /// Stop the orchestration system (for cleanup)
    pub fn stop_orchestration_system() -> Result<(), SchemaError> {
        info!("🛑 Stopping orchestration system");
        
        if let Ok(mut monitor) = EVENT_MONITOR.lock() {
            if monitor.take().is_some() {
                info!("✅ Orchestration system stopped successfully");
            } else {
                info!("ℹ️ Orchestration system was not running");
            }
        } else {
            error!("❌ Failed to access EventMonitor for shutdown");
            return Err(SchemaError::InvalidData(
                "Failed to access EventMonitor for shutdown".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if the orchestration system is running
    pub fn is_orchestration_running() -> bool {
        EVENT_MONITOR
            .lock()
            .map(|monitor| monitor.is_some())
            .unwrap_or(false)
    }
}

/// Simple transform runner implementation for the orchestration system
pub struct SimpleTransformRunner {
    db_ops: Arc<DbOperations>,
}

impl SimpleTransformRunner {
    /// Create a new simple transform runner
    pub fn new(db_ops: Arc<DbOperations>) -> Self {
        Self { db_ops }
    }

    /// Execute a single transform directly
    fn execute_single_transform(
        &self,
        transform_id: &str,
        transform: &crate::schema::types::Transform,
    ) -> Result<JsonValue, SchemaError> {
        // Use the execution manager's static method
        super::execution::TransformExecutionManager::execute_single_transform(
            transform_id,
            transform,
            &self.db_ops,
        )
    }

    /// Store transform result using the execution module
    fn store_transform_result(
        &self,
        transform: &crate::schema::types::Transform,
        result: &JsonValue,
    ) -> Result<(), SchemaError> {
        // Use the execution manager's static method
        super::execution::TransformExecutionManager::store_transform_result_generic(
            &self.db_ops,
            transform,
            result,
        )
    }
}

impl TransformRunner for SimpleTransformRunner {
    /// Execute a transform now with full execution and result storage
    fn execute_transform_now(&self, transform_id: &str) -> Result<JsonValue, SchemaError> {
        info!("🚀 SimpleTransformRunner: Executing transform '{}'", transform_id);

        // Load and execute the transform directly
        if let Ok(Some(transform)) = self.db_ops.get_transform(transform_id) {
            let result = self.execute_single_transform(transform_id, &transform)?;

            // Store the result
            self.store_transform_result(&transform, &result)?;

            info!("✅ SimpleTransformRunner: Transform '{}' executed successfully", transform_id);
            Ok(result)
        } else {
            let error_msg = format!("Transform '{}' not found", transform_id);
            error!("❌ SimpleTransformRunner: {}", error_msg);
            Err(SchemaError::InvalidData(error_msg))
        }
    }

    /// Check if a transform exists in the database
    fn transform_exists(&self, transform_id: &str) -> Result<bool, SchemaError> {
        Ok(self.db_ops.get_transform(transform_id)?.is_some())
    }

    /// Get transforms for a field by loading from database
    fn get_transforms_for_field(
        &self,
        schema_name: &str,
        field_name: &str,
    ) -> Result<HashSet<String>, SchemaError> {
        // Load field-to-transforms mapping from database
        let field_key = format!("{}.{}", schema_name, field_name);

        match self.db_ops.get_transform_mapping(FIELD_TO_TRANSFORMS_KEY) {
            Ok(Some(mapping_bytes)) => {
                if let Ok(field_to_transforms) = serde_json::from_slice::<
                    std::collections::HashMap<String, HashSet<String>>,
                >(&mapping_bytes)
                {
                    let result = field_to_transforms
                        .get(&field_key)
                        .cloned()
                        .unwrap_or_default();
                    
                    info!(
                        "🔍 SimpleTransformRunner: Found {} transforms for field '{}'",
                        result.len(),
                        field_key
                    );
                    Ok(result)
                } else {
                    info!("⚠️ SimpleTransformRunner: Failed to deserialize field_to_transforms mapping, returning empty set");
                    Ok(HashSet::new())
                }
            }
            Ok(None) => {
                info!("ℹ️ SimpleTransformRunner: No field_to_transforms mapping found in database");
                Ok(HashSet::new())
            }
            Err(e) => {
                error!("❌ SimpleTransformRunner: Failed to load field_to_transforms mapping: {}", e);
                Err(SchemaError::InvalidData(format!(
                    "Failed to load field mapping: {}",
                    e
                )))
            }
        }
    }
}

/// Utility functions for orchestration system management
pub struct OrchestrationUtils;

impl OrchestrationUtils {
    /// Initialize orchestration system with provided dependencies
    pub fn initialize_orchestration(
        db_ops: Arc<DbOperations>,
        message_bus: Arc<MessageBus>,
    ) -> Result<(), SchemaError> {
        let orchestration_manager = TransformOrchestrationManager::new(db_ops, message_bus);
        orchestration_manager.start_orchestration_system()
    }

    /// Shutdown orchestration system
    pub fn shutdown_orchestration() -> Result<(), SchemaError> {
        TransformOrchestrationManager::stop_orchestration_system()
    }

    /// Check orchestration system status
    pub fn check_orchestration_status() -> bool {
        TransformOrchestrationManager::is_orchestration_running()
    }

    /// Log orchestration system status
    pub fn log_orchestration_status() {
        if Self::check_orchestration_status() {
            info!("✅ Orchestration system is running");
        } else {
            info!("❌ Orchestration system is not running");
        }
    }
}