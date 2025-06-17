//! Transform Manager - Main coordinator for transform management system
//!
//! This is now a thin coordinator that delegates complex logic to specialized modules:
//! - state: In-memory state management
//! - registration: Transform loading and registration
//! - execution: Transform execution logic
//! - orchestration: Event-driven orchestration
//! - metrics: Logging and diagnostics
//!
//! CURRENT ARCHITECTURE RESPONSIBILITIES:
//! - Transform Registration: Delegates to registration module
//! - Transform Execution: Delegates to execution module
//! - Dependency Tracking: Handled by state module
//! - Schema Monitoring: Managed by orchestration module
//!
//! ORCHESTRATION IS HANDLED BY TransformOrchestrator:
//! - TransformOrchestrator listens for FieldValueSet events directly
//! - TransformOrchestrator determines which transforms to execute
//! - TransformOrchestrator calls TransformManager for actual execution

use super::execution::TransformExecutionManager;
use super::orchestration::{TransformOrchestrationManager, OrchestrationUtils};
use super::registration::TransformRegistrationManager;
use super::state::TransformManagerState;
use super::types::TransformRunner;
use crate::db_operations::DbOperations;
use crate::fold_db_core::infrastructure::message_bus::MessageBus;
use crate::schema::types::{SchemaError, Transform, TransformRegistration};
use log::{error, info};
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// TransformManager: Main coordinator for transform management
pub struct TransformManager {
    /// Direct database operations (consistent with other components)
    pub(super) db_ops: Arc<DbOperations>,
    /// In-memory state management
    state: Arc<TransformManagerState>,
    /// Transform registration manager
    registration_manager: TransformRegistrationManager,
    /// Transform execution manager
    execution_manager: TransformExecutionManager,
    /// Orchestration manager
    #[allow(dead_code)]
    orchestration_manager: TransformOrchestrationManager,
    /// Message bus for event-driven communication
    pub(super) _message_bus: Arc<MessageBus>,
}

impl TransformManager {
    /// Creates a new TransformManager instance with unified database operations
    pub fn new(
        db_ops: Arc<DbOperations>,
        message_bus: Arc<MessageBus>,
    ) -> Result<Self, SchemaError> {
        info!("🚀 Initializing TransformManager with refactored architecture");

        // Create shared state
        let state = Arc::new(TransformManagerState::new());

        // Create specialized managers
        let registration_manager = TransformRegistrationManager::new(Arc::clone(&db_ops), Arc::clone(&state));
        let execution_manager = TransformExecutionManager::new(Arc::clone(&db_ops));
        let orchestration_manager = TransformOrchestrationManager::new(Arc::clone(&db_ops), Arc::clone(&message_bus));

        // Load persisted mappings
        let (
            aref_to_transforms,
            transform_to_arefs,
            transform_input_names,
            field_to_transforms,
            transform_to_fields,
            transform_outputs,
        ) = TransformRegistrationManager::load_persisted_mappings_static(&db_ops)?;

        // Initialize state with loaded mappings
        {
            *crate::fold_db_core::transform_manager::utils::TransformUtils::write_lock(&state.aref_to_transforms, "aref_to_transforms")? = aref_to_transforms;
            *crate::fold_db_core::transform_manager::utils::TransformUtils::write_lock(&state.transform_to_arefs, "transform_to_arefs")? = transform_to_arefs;
            *crate::fold_db_core::transform_manager::utils::TransformUtils::write_lock(&state.transform_input_names, "transform_input_names")? = transform_input_names;
            *crate::fold_db_core::transform_manager::utils::TransformUtils::write_lock(&state.field_to_transforms, "field_to_transforms")? = field_to_transforms;
            *crate::fold_db_core::transform_manager::utils::TransformUtils::write_lock(&state.transform_to_fields, "transform_to_fields")? = transform_to_fields;
            *crate::fold_db_core::transform_manager::utils::TransformUtils::write_lock(&state.transform_outputs, "transform_outputs")? = transform_outputs;
        }

        // Load transforms from database
        let loaded_transforms = registration_manager.load_transforms_from_database()?;

        // Log loaded field mappings during initialization
        {
            let field_to_transforms = crate::fold_db_core::transform_manager::utils::TransformUtils::read_lock(&state.field_to_transforms, "field_to_transforms")?;
            info!(
                "🔍 DEBUG TransformManager::new(): Loaded field_to_transforms with {} entries:",
                field_to_transforms.len()
            );
            for (field_key, transforms) in field_to_transforms.iter() {
                info!("  📋 '{}' -> {:?}", field_key, transforms);
            }
            if field_to_transforms.is_empty() {
                info!("⚠️ DEBUG TransformManager::new(): No field mappings loaded from database!");
            }
        }

        // Start the orchestration system
        orchestration_manager.start_orchestration_system()?;

        info!(
            "✅ TransformManager initialized successfully with {} transforms",
            loaded_transforms.len()
        );

        Ok(Self {
            db_ops,
            state,
            registration_manager,
            execution_manager,
            orchestration_manager,
            _message_bus: message_bus,
        })
    }

    /// Returns true if a transform with the given id is registered.
    pub fn transform_exists(&self, transform_id: &str) -> Result<bool, SchemaError> {
        self.registration_manager.transform_exists(transform_id)
    }

    /// List all registered transforms.
    pub fn list_transforms(&self) -> Result<HashMap<String, Transform>, SchemaError> {
        self.registration_manager.list_transforms()
    }

    /// Gets all transforms that depend on the specified atom reference.
    pub fn get_dependent_transforms(
        &self,
        aref_uuid: &str,
    ) -> Result<HashSet<String>, SchemaError> {
        self.registration_manager.get_dependent_transforms(aref_uuid)
    }

    /// Gets all atom references that a transform depends on.
    pub fn get_transform_inputs(&self, transform_id: &str) -> Result<HashSet<String>, SchemaError> {
        self.registration_manager.get_transform_inputs(transform_id)
    }

    /// Gets the output atom reference for a transform.
    pub fn get_transform_output(&self, transform_id: &str) -> Result<Option<String>, SchemaError> {
        self.registration_manager.get_transform_output(transform_id)
    }

    /// Gets all transforms that should run when the specified field is updated.
    pub fn get_transforms_for_field(
        &self,
        schema_name: &str,
        field_name: &str,
    ) -> Result<HashSet<String>, SchemaError> {
        self.registration_manager.get_transforms_for_field(schema_name, field_name)
    }

    /// Register transform using event-driven database operations only
    pub fn register_transform_event_driven(
        &self,
        registration: TransformRegistration,
    ) -> Result<(), SchemaError> {
        self.registration_manager.register_transform_event_driven(registration)
    }

    /// Registers a transform with automatic input dependency detection.
    pub fn register_transform_auto(
        &self,
        transform_id: String,
        transform: Transform,
        output_aref: String,
        schema_name: String,
        field_name: String,
    ) -> Result<(), SchemaError> {
        self.registration_manager.register_transform_auto(
            transform_id,
            transform,
            output_aref,
            schema_name,
            field_name,
        )
    }

    /// Unregisters a transform using direct database operations.
    pub fn unregister_transform(&self, transform_id: &str) -> Result<bool, SchemaError> {
        self.registration_manager.unregister_transform(transform_id)
    }

    /// Reload transforms from database - called when new transforms are registered
    pub fn reload_transforms(&self) -> Result<(), SchemaError> {
        self.registration_manager.reload_transforms()
    }

    /// Persist mappings to database
    pub fn persist_mappings_direct(&self) -> Result<(), SchemaError> {
        self.state.persist_mappings(&self.db_ops)
    }

    /// Load persisted mappings from database (static method for backward compatibility)
    #[allow(clippy::type_complexity)]
    pub fn load_persisted_mappings_direct(
        db_ops: &Arc<DbOperations>,
    ) -> Result<(
        HashMap<String, HashSet<String>>,   // aref_to_transforms
        HashMap<String, HashSet<String>>,   // transform_to_arefs  
        HashMap<String, HashMap<String, String>>, // transform_input_names
        HashMap<String, HashSet<String>>,   // field_to_transforms
        HashMap<String, HashSet<String>>,   // transform_to_fields
        HashMap<String, String>,            // transform_outputs
    ), SchemaError> {
        TransformRegistrationManager::load_persisted_mappings_static(db_ops)
    }

    /// Execute transform directly using transform_id (DEPRECATED)
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

    /// Static execution methods (for backward compatibility with orchestration system)
    pub fn execute_single_transform(
        transform_id: &str,
        transform: &Transform,
        db_ops: &Arc<DbOperations>,
    ) -> Result<JsonValue, SchemaError> {
        TransformExecutionManager::execute_single_transform(transform_id, transform, db_ops)
    }

    /// Static result storage method (for backward compatibility with orchestration system)
    pub fn store_transform_result_generic(
        db_ops: &Arc<DbOperations>,
        transform: &Transform,
        result: &JsonValue,
    ) -> Result<(), SchemaError> {
        TransformExecutionManager::store_transform_result_generic(db_ops, transform, result)
    }

    /// Start orchestration system (utility method)
    pub fn start_orchestration_system(
        db_ops: Arc<DbOperations>,
        message_bus: Arc<MessageBus>,
    ) -> Result<(), SchemaError> {
        OrchestrationUtils::initialize_orchestration(db_ops, message_bus)
    }
}

impl TransformRunner for TransformManager {
    /// Execute a transform now using the execution manager
    fn execute_transform_now(&self, transform_id: &str) -> Result<JsonValue, SchemaError> {
        info!("🚀 TransformManager: Executing transform now: {}", transform_id);

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

        // Execute using the execution manager
        let result = self.execution_manager.execute_transform(transform_id, &transform)?;

        // Store the result using the execution manager
        self.execution_manager.store_result(&transform, &result)?;

        info!(
            "✅ Transform '{}' executed successfully: {}",
            transform_id, result
        );
        Ok(result)
    }

    fn transform_exists(&self, transform_id: &str) -> Result<bool, SchemaError> {
        self.registration_manager.transform_exists(transform_id)
    }

    fn get_transforms_for_field(
        &self,
        schema_name: &str,
        field_name: &str,
    ) -> Result<HashSet<String>, SchemaError> {
        self.registration_manager.get_transforms_for_field(schema_name, field_name)
    }
}
