use super::types::TransformRunner;
use crate::db_operations::DbOperations;
use crate::fold_db_core::infrastructure::message_bus::MessageBus;
use crate::schema::types::{SchemaError, Transform};
use log::{info, error};
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::thread;

pub(super) const AREF_TO_TRANSFORMS_KEY: &str = "map_aref_to_transforms";
pub(super) const TRANSFORM_TO_AREFS_KEY: &str = "map_transform_to_arefs";
pub(super) const TRANSFORM_INPUT_NAMES_KEY: &str = "map_transform_input_names";
pub(super) const FIELD_TO_TRANSFORMS_KEY: &str = "map_field_to_transforms";
pub(super) const TRANSFORM_TO_FIELDS_KEY: &str = "map_transform_to_fields";
pub(super) const TRANSFORM_OUTPUTS_KEY: &str = "map_transform_outputs";

pub struct TransformManager {
    /// Direct database operations (consistent with other components)
    pub(super) db_ops: Arc<DbOperations>,
    /// In-memory cache of registered transforms
    pub(super) registered_transforms: RwLock<HashMap<String, Transform>>,
    /// Maps atom reference UUIDs to the transforms that depend on them
    pub(super) aref_to_transforms: RwLock<HashMap<String, HashSet<String>>>,
    /// Maps transform IDs to their dependent atom reference UUIDs
    pub(super) transform_to_arefs: RwLock<HashMap<String, HashSet<String>>>,
    /// Maps transform IDs to input field names keyed by atom ref UUID
    pub(super) transform_input_names: RwLock<HashMap<String, HashMap<String, String>>>,
    /// Maps schema.field keys to transforms triggered by them
    pub(super) field_to_transforms: RwLock<HashMap<String, HashSet<String>>>,
    /// Maps transform IDs to the fields that trigger them
    pub(super) transform_to_fields: RwLock<HashMap<String, HashSet<String>>>,
    /// Maps transform IDs to their output atom reference UUIDs
    pub(super) transform_outputs: RwLock<HashMap<String, String>>,
    /// Message bus for event-driven communication
    pub(super) message_bus: Arc<MessageBus>,
    /// Thread handle for processing TransformTriggered events
    pub(super) _transform_triggered_thread: Option<thread::JoinHandle<()>>,
    /// Thread handle for monitoring SchemaChanged events
    pub(super) _schema_changed_consumer_thread: Option<thread::JoinHandle<()>>,
}

impl TransformManager {
    /// Creates a new TransformManager instance with unified database operations
    pub fn new(
        db_ops: std::sync::Arc<crate::db_operations::DbOperations>,
        message_bus: Arc<MessageBus>,
    ) -> Result<Self, SchemaError> {
        // Load any persisted transforms using direct database operations
        let mut registered_transforms = HashMap::new();
        
        let transform_ids = db_ops.list_transforms()?;

        for transform_id in transform_ids {
            match db_ops.get_transform(&transform_id) {
                Ok(Some(transform)) => {
                    info!(
                        "📋 Loading transform '{}' with inputs: {:?}, output: {}",
                        transform_id, transform.get_inputs(), transform.get_output()
                    );
                    registered_transforms.insert(transform_id, transform);
                }
                Ok(None) => {
                    log::warn!(
                        "Transform '{}' not found in storage during initialization",
                        transform_id
                    );
                }
                Err(e) => {
                    log::error!(
                        "Failed to load transform '{}' during initialization: {}",
                        transform_id,
                        e
                    );
                    return Err(e);
                }
            }
        }

        // Load mappings using direct database operations
        let (
            aref_to_transforms,
            transform_to_arefs,
            transform_input_names,
            field_to_transforms,
            transform_to_fields,
            transform_outputs,
        ) = Self::load_persisted_mappings_direct(&db_ops)?;
        
        // DEBUG: Log loaded field mappings during initialization
        info!("🔍 DEBUG TransformManager::new(): Loaded field_to_transforms with {} entries:", field_to_transforms.len());
        for (field_key, transforms) in &field_to_transforms {
            info!("  📋 '{}' -> {:?}", field_key, transforms);
        }
        if field_to_transforms.is_empty() {
            info!("⚠️ DEBUG TransformManager::new(): No field mappings loaded from database!");
        }

        // Field mappings will be auto-registered during reload_transforms()
        // which is triggered by SchemaChanged events, avoiding duplicate registration

        // FieldValueSet monitoring is now handled directly by TransformOrchestrator
        // TransformTriggerRequest and TransformExecutionRequest processing is also obsolete
        // since TransformOrchestrator now handles transforms directly

        // Set up processing of TransformTriggered events
        let transform_triggered_thread = Self::setup_transform_triggered_processing(
            Arc::clone(&message_bus),
            Some(Arc::clone(&db_ops)),
        );

        // Set up monitoring of SchemaChanged events to reload transforms
        let schema_changed_consumer_thread = Self::setup_schema_changed_monitoring(
            Arc::clone(&message_bus),
            Arc::clone(&db_ops),
        );

        Ok(Self {
            db_ops,
            registered_transforms: RwLock::new(registered_transforms),
            aref_to_transforms: RwLock::new(aref_to_transforms),
            transform_to_arefs: RwLock::new(transform_to_arefs),
            transform_input_names: RwLock::new(transform_input_names),
            field_to_transforms: RwLock::new(field_to_transforms),
            transform_to_fields: RwLock::new(transform_to_fields),
            transform_outputs: RwLock::new(transform_outputs),
            message_bus,
            _transform_triggered_thread: Some(transform_triggered_thread),
            _schema_changed_consumer_thread: Some(schema_changed_consumer_thread),
        })
    }

    /// Returns true if a transform with the given id is registered.
    pub fn transform_exists(&self, transform_id: &str) -> Result<bool, SchemaError> {
        let registered_transforms = self.registered_transforms.read().map_err(|_| {
            SchemaError::InvalidData("Failed to acquire registered_transforms lock".to_string())
        })?;
        Ok(registered_transforms.contains_key(transform_id))
    }

    /// List all registered transforms.
    pub fn list_transforms(&self) -> Result<HashMap<String, Transform>, SchemaError> {
        let registered_transforms = self.registered_transforms.read().map_err(|_| {
            SchemaError::InvalidData("Failed to acquire registered_transforms lock".to_string())
        })?;
        Ok(registered_transforms.clone())
    }

    /// Gets all transforms that depend on the specified atom reference.
    pub fn get_dependent_transforms(
        &self,
        aref_uuid: &str,
    ) -> Result<HashSet<String>, SchemaError> {
        let aref_to_transforms = self.aref_to_transforms.read().map_err(|_| {
            SchemaError::InvalidData("Failed to acquire aref_to_transforms lock".to_string())
        })?;
        Ok(match aref_to_transforms.get(aref_uuid) {
            Some(transform_set) => transform_set.clone(),
            None => HashSet::new(),
        })
    }

    /// Gets all atom references that a transform depends on.
    pub fn get_transform_inputs(&self, transform_id: &str) -> Result<HashSet<String>, SchemaError> {
        let transform_to_arefs = self.transform_to_arefs.read().map_err(|_| {
            SchemaError::InvalidData("Failed to acquire transform_to_arefs lock".to_string())
        })?;
        Ok(match transform_to_arefs.get(transform_id) {
            Some(aref_set) => aref_set.clone(),
            None => HashSet::new(),
        })
    }

    /// Gets the output atom reference for a transform.
    pub fn get_transform_output(&self, transform_id: &str) -> Result<Option<String>, SchemaError> {
        let transform_outputs = self.transform_outputs.read().map_err(|_| {
            SchemaError::InvalidData("Failed to acquire transform_outputs lock".to_string())
        })?;
        Ok(transform_outputs.get(transform_id).cloned())
    }

    /// Gets all transforms that should run when the specified field is updated.
    pub fn get_transforms_for_field(
        &self,
        schema_name: &str,
        field_name: &str,
    ) -> Result<HashSet<String>, SchemaError> {
        let key = format!("{}.{}", schema_name, field_name);
        let field_to_transforms = self.field_to_transforms.read().map_err(|_| {
            SchemaError::InvalidData("Failed to acquire field_to_transforms lock".to_string())
        })?;
        
        let result = field_to_transforms.get(&key).cloned().unwrap_or_default();
        
        // DEBUG: Log field mapping lookup
        info!(
            "🔍 DEBUG TransformManager: Looking up transforms for '{}' - found {} transforms: {:?}",
            key, result.len(), result
        );
        
        // DEBUG: Log all field mappings for diagnostics
        if result.is_empty() {
            info!("🔍 DEBUG TransformManager: All field mappings available:");
            for (field_key, transforms) in field_to_transforms.iter() {
                info!("  📋 '{}' -> {:?}", field_key, transforms);
            }
            if field_to_transforms.is_empty() {
                error!("❌ DEBUG TransformManager: No field mappings found at all!");
            }
        }
        
        Ok(result)
    }
    /// Execute transform directly using transform_id (simplified approach)
    pub fn execute_transform_with_db(
        transform_id: &str,
        message_bus: &Arc<MessageBus>,
        db_ops: Option<&Arc<crate::db_operations::DbOperations>>,
    ) -> (usize, bool, Option<String>) {
        info!("🚀 TransformManager: Executing transform directly: {}", transform_id);
        info!("🔍 Starting transform execution flow...");
        
        // Get database operations
        let db_ops = match db_ops {
            Some(ops) => {
                info!("✅ Database operations available");
                ops
            }
            None => {
                error!("❌ No database operations provided for transform execution");
                return (0_usize, false, Some("Database operations required".to_string()));
            }
        };
        
        // Load the transform from the database
        info!("📋 Loading transform '{}' from database...", transform_id);
        let transform = match db_ops.get_transform(transform_id) {
            Ok(Some(transform)) => {
                info!("✅ Transform '{}' loaded successfully", transform_id);
                info!("🔧 Transform logic: {}", transform.logic);
                info!("📥 Transform inputs: {:?}", transform.get_inputs());
                info!("📤 Transform output: {}", transform.get_output());
                transform
            }
            Ok(None) => {
                error!("❌ Transform '{}' not found in database", transform_id);
                return (0_usize, false, Some(format!("Transform '{}' not found", transform_id)));
            }
            Err(e) => {
                error!("❌ Failed to load transform '{}': {}", transform_id, e);
                return (0_usize, false, Some(format!("Failed to load transform: {}", e)));
            }
        };
        
        // Execute the transform directly
        info!("🚀 Executing transform logic...");
        match Self::execute_single_transform(transform_id, &transform, db_ops) {
            Ok(result) => {
                info!("✅ Transform '{}' executed successfully: {}", transform_id, result);
                
                // Store the result
                info!("💾 Storing transform result...");
                if let Err(e) = Self::store_transform_result_generic(db_ops, &transform, &result) {
                    error!("❌ Failed to store result for transform '{}': {}", transform_id, e);
                    return (0_usize, false, Some(format!("Failed to store result: {}", e)));
                }
                
                info!("✅ Transform result stored successfully");
                
                // Publish TransformExecuted event
                info!("📢 Publishing TransformExecuted event...");
                let executed_event = crate::fold_db_core::infrastructure::message_bus::TransformExecuted {
                    transform_id: transform_id.to_string(),
                    result: format!("computed_result: {}", result),
                };
                
                match message_bus.publish(executed_event) {
                    Ok(_) => {
                        info!("✅ Published TransformExecuted event for: {}", transform_id);
                        info!("🎯 Transform execution completed successfully");
                        (1_usize, true, None)
                    }
                    Err(e) => {
                        error!("❌ Failed to publish TransformExecuted event for {}: {}", transform_id, e);
                        (1_usize, true, Some(format!("Failed to publish event: {}", e)))
                    }
                }
            }
            Err(e) => {
                error!("❌ Transform '{}' execution failed: {}", transform_id, e);
                (0_usize, false, Some(format!("Transform execution failed: {}", e)))
            }
        }
    }
}

impl TransformRunner for TransformManager {
    /// Execute transform directly using simplified transform_id approach
    fn execute_transform_now(&self, transform_id: &str) -> Result<JsonValue, SchemaError> {
        info!("🚀 execute_transform_now called for transform_id: {}", transform_id);
        info!("🔄 Starting transform execution flow...");
        
        // Use simplified direct execution with transform_id
        info!("🔧 Delegating to execute_transform_with_db...");
        let (transforms_executed, success, error_msg) = TransformManager::execute_transform_with_db(
            transform_id,
            &self.message_bus,
            Some(&self.db_ops)
        );
        
        if success {
            info!("✅ Transform '{}' execution completed successfully", transform_id);
            info!("📊 Transforms executed: {}", transforms_executed);
            let result = serde_json::json!({
                "status": "executed_directly",
                "transform_id": transform_id,
                "transforms_executed": transforms_executed,
                "method": "simplified_direct_execution"
            });
            info!("🎯 Final result returned: {}", result);
            Ok(result)
        } else {
            let error_message = error_msg.unwrap_or_else(|| "Unknown execution error".to_string());
            error!("❌ Transform '{}' execution failed: {}", transform_id, error_message);
            Err(SchemaError::InvalidData(format!("Transform execution failed: {}", error_message)))
        }
    }

    fn transform_exists(&self, transform_id: &str) -> Result<bool, SchemaError> {
        let registered_transforms = self.registered_transforms.read().map_err(|_| {
            SchemaError::InvalidData("Failed to acquire registered_transforms lock".to_string())
        })?;
        Ok(registered_transforms.contains_key(transform_id))
    }

    fn get_transforms_for_field(
        &self,
        schema_name: &str,
        field_name: &str,
    ) -> Result<HashSet<String>, SchemaError> {
        let key = format!("{}.{}", schema_name, field_name);
        let field_to_transforms = self.field_to_transforms.read().map_err(|_| {
            SchemaError::InvalidData("Failed to acquire field_to_transforms lock".to_string())
        })?;
        Ok(field_to_transforms.get(&key).cloned().unwrap_or_default())
    }
}
