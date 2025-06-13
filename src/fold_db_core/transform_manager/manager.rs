use super::types::TransformRunner;
use crate::db_operations::DbOperations;
use crate::fold_db_core::infrastructure::message_bus::MessageBus;
use crate::fold_db_core::transform_manager::utils::*;
use crate::schema::types::{SchemaError, Transform};
use log::{error, info};
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

/// TransformManager: Handles transform execution and registration
///
/// CURRENT ARCHITECTURE RESPONSIBILITIES:
/// - Transform Registration: Manages loading and storing of transforms
/// - Transform Execution: Executes individual transforms when requested
/// - Dependency Tracking: Maintains mappings between fields and transforms
/// - Schema Monitoring: Reloads transforms when schemas change
///
/// ORCHESTRATION IS HANDLED BY TransformOrchestrator:
/// - TransformOrchestrator listens for FieldValueSet events directly
/// - TransformOrchestrator determines which transforms to execute
/// - TransformOrchestrator calls TransformManager for actual execution
///
/// This separation provides clean responsibilities:
/// - TransformOrchestrator: Orchestration and event handling
/// - TransformManager: Execution and registration
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
    pub(super) _message_bus: Arc<MessageBus>,
    /// Thread handle for monitoring SchemaChanged events to reload transforms
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
                        transform_id,
                        transform.get_inputs(),
                        transform.get_output()
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
        info!(
            "🔍 DEBUG TransformManager::new(): Loaded field_to_transforms with {} entries:",
            field_to_transforms.len()
        );
        for (field_key, transforms) in &field_to_transforms {
            info!("  📋 '{}' -> {:?}", field_key, transforms);
        }
        if field_to_transforms.is_empty() {
            info!("⚠️ DEBUG TransformManager::new(): No field mappings loaded from database!");
        }

        // Field mappings will be auto-registered during reload_transforms()
        // which is triggered by SchemaChanged events, avoiding duplicate registration

        // Start the orchestration system to handle TransformTriggered events
        Self::start_orchestration_system(Arc::clone(&db_ops), Arc::clone(&message_bus))?;

        // Monitoring removed during aggressive cleanup

        Ok(Self {
            db_ops,
            registered_transforms: RwLock::new(registered_transforms),
            aref_to_transforms: RwLock::new(aref_to_transforms),
            transform_to_arefs: RwLock::new(transform_to_arefs),
            transform_input_names: RwLock::new(transform_input_names),
            field_to_transforms: RwLock::new(field_to_transforms),
            transform_to_fields: RwLock::new(transform_to_fields),
            transform_outputs: RwLock::new(transform_outputs),
            _message_bus: message_bus,
            _schema_changed_consumer_thread: None,
        })
    }

    /// Returns true if a transform with the given id is registered.
    pub fn transform_exists(&self, transform_id: &str) -> Result<bool, SchemaError> {
        let registered_transforms = self
            .registered_transforms
            .read()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire read lock".to_string()))?;
        Ok(registered_transforms.contains_key(transform_id))
    }

    /// List all registered transforms.
    pub fn list_transforms(&self) -> Result<HashMap<String, Transform>, SchemaError> {
        let registered_transforms = self
            .registered_transforms
            .read()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire read lock".to_string()))?;
        Ok(registered_transforms.clone())
    }

    /// Gets all transforms that depend on the specified atom reference.
    pub fn get_dependent_transforms(
        &self,
        aref_uuid: &str,
    ) -> Result<HashSet<String>, SchemaError> {
        let aref_to_transforms = self
            .aref_to_transforms
            .read()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire read lock".to_string()))?;
        Ok(aref_to_transforms
            .get(aref_uuid)
            .cloned()
            .unwrap_or_default())
    }

    /// Gets all atom references that a transform depends on.
    pub fn get_transform_inputs(&self, transform_id: &str) -> Result<HashSet<String>, SchemaError> {
        let transform_to_arefs = self
            .transform_to_arefs
            .read()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire read lock".to_string()))?;
        Ok(transform_to_arefs
            .get(transform_id)
            .cloned()
            .unwrap_or_default())
    }

    /// Gets the output atom reference for a transform.
    pub fn get_transform_output(&self, transform_id: &str) -> Result<Option<String>, SchemaError> {
        let transform_outputs = self
            .transform_outputs
            .read()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire read lock".to_string()))?;
        Ok(transform_outputs.get(transform_id).cloned())
    }

    /// Gets all transforms that should run when the specified field is updated.
    pub fn get_transforms_for_field(
        &self,
        schema_name: &str,
        field_name: &str,
    ) -> Result<HashSet<String>, SchemaError> {
        let key = format!("{}.{}", schema_name, field_name);
        let field_to_transforms = self
            .field_to_transforms
            .read()
            .map_err(|_| SchemaError::InvalidData("Failed to acquire read lock".to_string()))?;

        let result = field_to_transforms.get(&key).cloned().unwrap_or_default();

        // DEBUG: Log field mapping lookup
        info!(
            "🔍 DEBUG TransformManager: Looking up transforms for '{}' - found {} transforms: {:?}",
            key,
            result.len(),
            result
        );

        // DEBUG: Log all field mappings for diagnostics
        if result.is_empty() {
            LoggingHelper::log_field_mappings_state(
                &field_to_transforms,
                "TransformManager::get_transforms_for_field",
            );
        }

        Ok(result)
    }

    /// Start the orchestration system to handle TransformTriggered events
    fn start_orchestration_system(
        db_ops: Arc<crate::db_operations::DbOperations>,
        message_bus: Arc<MessageBus>,
    ) -> Result<(), SchemaError> {
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
        struct SimpleTransformRunner {
            db_ops: Arc<crate::db_operations::DbOperations>,
        }

        impl crate::fold_db_core::transform_manager::types::TransformRunner for SimpleTransformRunner {
            fn execute_transform_now(
                &self,
                transform_id: &str,
            ) -> Result<serde_json::Value, SchemaError> {
                // Load and execute the transform directly
                if let Ok(Some(transform)) = self.db_ops.get_transform(transform_id) {
                    let result = crate::fold_db_core::transform_manager::manager::TransformManager::execute_single_transform(
                        transform_id,
                        &transform,
                        &self.db_ops
                    )?;

                    // Store the result
                    crate::fold_db_core::transform_manager::manager::TransformManager::store_transform_result_generic(
                        &self.db_ops,
                        &transform,
                        &result
                    )?;

                    Ok(result)
                } else {
                    Err(SchemaError::InvalidData(format!(
                        "Transform '{}' not found",
                        transform_id
                    )))
                }
            }

            fn transform_exists(&self, transform_id: &str) -> Result<bool, SchemaError> {
                Ok(self.db_ops.get_transform(transform_id)?.is_some())
            }

            fn get_transforms_for_field(
                &self,
                schema_name: &str,
                field_name: &str,
            ) -> Result<std::collections::HashSet<String>, SchemaError> {
                // Load field-to-transforms mapping from database
                let field_key = format!("{}.{}", schema_name, field_name);

                match self.db_ops.get_transform_mapping(FIELD_TO_TRANSFORMS_KEY) {
                    Ok(Some(mapping_bytes)) => {
                        if let Ok(field_to_transforms) = serde_json::from_slice::<
                            std::collections::HashMap<String, std::collections::HashSet<String>>,
                        >(&mapping_bytes)
                        {
                            Ok(field_to_transforms
                                .get(&field_key)
                                .cloned()
                                .unwrap_or_default())
                        } else {
                            info!("⚠️ Failed to deserialize field_to_transforms mapping, returning empty set");
                            Ok(std::collections::HashSet::new())
                        }
                    }
                    Ok(None) => {
                        info!("ℹ️ No field_to_transforms mapping found in database");
                        Ok(std::collections::HashSet::new())
                    }
                    Err(e) => {
                        error!("❌ Failed to load field_to_transforms mapping: {}", e);
                        Err(SchemaError::InvalidData(format!(
                            "Failed to load field mapping: {}",
                            e
                        )))
                    }
                }
            }
        }

        let transform_runner = Arc::new(SimpleTransformRunner {
            db_ops: Arc::clone(&db_ops),
        });

        // Start the EventMonitor to handle TransformTriggered events
        let persistence =
            crate::fold_db_core::orchestration::persistence_manager::PersistenceManager::new(tree);
        let _event_monitor = crate::fold_db_core::orchestration::event_monitor::EventMonitor::new(
            Arc::clone(&message_bus),
            transform_runner,
            persistence,
        );

        // Store the event monitor in a static variable so it doesn't get dropped
        use once_cell::sync::Lazy;
        use std::sync::Mutex;

        static EVENT_MONITOR: Lazy<
            Mutex<Option<crate::fold_db_core::orchestration::event_monitor::EventMonitor>>,
        > = Lazy::new(|| Mutex::new(None));

        if let Ok(mut monitor) = EVENT_MONITOR.lock() {
            *monitor = Some(_event_monitor);
            info!("✅ Orchestration system started successfully");
        } else {
            error!("❌ Failed to store EventMonitor in static variable");
        }

        Ok(())
    }
    /// Execute transform directly using transform_id (unified approach)
    /// MADE PRIVATE - Only ExecutionCoordinator should call this
    pub fn execute_transform_with_db(
        transform_id: &str,
        _message_bus: &Arc<MessageBus>,
        db_ops: Option<&Arc<crate::db_operations::DbOperations>>,
    ) -> (usize, bool, Option<String>) {
        info!(
            "🚀 TransformManager: Executing transform directly: {}",
            transform_id
        );

        // Get database operations
        let _db_ops = match db_ops {
            Some(ops) => {
                info!("✅ Database operations available");
                ops
            }
            None => {
                error!("❌ No database operations provided for transform execution");
                return (
                    0_usize,
                    false,
                    Some("Database operations required".to_string()),
                );
            }
        };

        // Execute directly without helper dependency
        error!("❌ DEPRECATED: TransformManager::execute_transform_with_db is no longer used");
        error!("❌ All execution should go through TransformOrchestrator -> ExecutionCoordinator");
        let success = false;
        let error_msg =
            Some("Direct transform execution through TransformManager is deprecated".to_string());

        if success {
            info!("🎯 Transform execution completed successfully");
            (1_usize, true, None)
        } else {
            (0_usize, false, error_msg)
        }
    }
}

impl TransformRunner for TransformManager {
    /// DEPRECATED: Direct execution removed - use TransformOrchestrator::add_transform() instead
    /// This method now only queues the transform for execution by the orchestrator
    fn execute_transform_now(&self, transform_id: &str) -> Result<JsonValue, SchemaError> {
        info!(
            "🚀 TransformManager: Executing transform now: {}",
            transform_id
        );

        // Load the transform from the database
        let transform = match self.db_ops.get_transform(transform_id) {
            Ok(Some(transform)) => transform,
            Ok(None) => {
                error!("❌ Transform '{}' not found", transform_id);
                return Err(SchemaError::InvalidData(format!(
                    "Transform '{}' not found",
                    transform_id
                )));
            }
            Err(e) => {
                error!("❌ Failed to load transform '{}': {}", transform_id, e);
                return Err(SchemaError::InvalidData(format!(
                    "Failed to load transform: {}",
                    e
                )));
            }
        };

        // Execute the transform using the execution module (call as static method)
        let result =
            TransformManager::execute_single_transform(transform_id, &transform, &self.db_ops)?;

        // Store the result (call as static method)
        Self::store_transform_result_generic(&self.db_ops, &transform, &result)?;

        info!(
            "✅ Transform '{}' executed successfully: {}",
            transform_id, result
        );
        Ok(result)
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
