//! FoldDB Core Coordinator
//!
//! This module contains the main FoldDB struct and initialization logic.
//! It coordinates between different managers and services but delegates
//! specific operations to the operations modules.

use crate::atom::AtomRefBehavior;
use crate::fold_db_core::infrastructure::factory::InfrastructureFactory;
use crate::fold_db_core::infrastructure::init::{init_orchestrator, init_transform_manager};
use crate::fold_db_core::infrastructure::message_bus::{
    AtomRefUpdateRequest, MessageBus, SystemInitializationRequest,
};
use crate::fold_db_core::managers::AtomManager;
use crate::fold_db_core::operations::EncryptionOperations;
use crate::fold_db_core::orchestration::TransformOrchestrator;
use crate::fold_db_core::services::field_retrieval::service::FieldRetrievalService;
use crate::fold_db_core::transform_manager::TransformManager;
use crate::db_operations::DbOperations;
use crate::permissions::PermissionWrapper;
use crate::schema::core::{SchemaCore, SchemaState};
use crate::schema::{Schema, SchemaError};
use log::info;
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// The main database coordinator that manages schemas, permissions, and data storage.
pub struct FoldDB {
    pub(crate) atom_manager: AtomManager,
    pub(crate) field_retrieval_service: FieldRetrievalService,
    pub(crate) schema_manager: Arc<SchemaCore>,
    pub(crate) transform_manager: Arc<TransformManager>,
    pub(crate) transform_orchestrator: Arc<TransformOrchestrator>,
    /// Shared database operations
    pub(crate) db_ops: Arc<DbOperations>,
    permission_wrapper: PermissionWrapper,
    /// Message bus for event-driven communication
    pub(crate) message_bus: Arc<MessageBus>,
    /// Event monitor for system-wide observability
    pub(crate) event_monitor: Arc<crate::fold_db_core::infrastructure::event_monitor::EventMonitor>,
    /// Stateful encryption operations coordinator
    pub(crate) encryption_operations: Arc<Mutex<EncryptionOperations>>,
}

impl FoldDB {
    /// Creates a new FoldDB instance with the specified storage path.
    pub fn new(path: &str) -> sled::Result<Self> {
        let db = match sled::open(path) {
            Ok(db) => db,
            Err(e) => {
                if e.to_string().contains("No such file or directory") {
                    sled::open(path)?
                } else {
                    return Err(e);
                }
            }
        };

        let db_ops =
            DbOperations::new(db.clone()).map_err(|e| sled::Error::Unsupported(e.to_string()))?;
        let orchestrator_tree = db_ops.orchestrator_tree.clone();

        // Initialize message bus
        let message_bus = InfrastructureFactory::create_message_bus();

        // Initialize components via event-driven system initialization
        let correlation_id = Uuid::new_v4().to_string();
        let init_request = SystemInitializationRequest {
            correlation_id: correlation_id.clone(),
            db_path: path.to_string(),
            orchestrator_config: None,
        };

        // Send system initialization request via message bus
        if let Err(e) = message_bus.publish(init_request) {
            return Err(sled::Error::Unsupported(format!(
                "Failed to initialize system via events: {}",
                e
            )));
        }

        // Create managers using event-driven initialization only
        let db_ops_arc = Arc::new(db_ops.clone());
        let atom_manager = AtomManager::new(db_ops.clone(), Arc::clone(&message_bus));
        let schema_manager = Arc::new(
            SchemaCore::new(path, Arc::clone(&db_ops_arc), Arc::clone(&message_bus))
                .map_err(|e| sled::Error::Unsupported(e.to_string()))?,
        );

        // Use standard initialization but with deprecated closures that recommend events
        let transform_manager =
            init_transform_manager(Arc::new(db_ops.clone()), Arc::clone(&message_bus))?;
        let orchestrator = init_orchestrator(
            &FieldRetrievalService::new(Arc::clone(&message_bus)),
            transform_manager.clone(),
            orchestrator_tree,
            Arc::clone(&message_bus),
            Arc::new(db_ops.clone()),
        )?;

        info!("Loading schema states from disk during FoldDB initialization");
        if let Err(e) = schema_manager.discover_and_load_all_schemas() {
            info!("Failed to load schema states: {}", e);
        } else {
            // After loading schema states, we need to ensure approved schemas are moved from 'available'
            // to 'schemas' HashMap so that map_fields() can find them
            if let Ok(approved_schemas) =
                schema_manager.list_schemas_by_state(SchemaState::Approved)
            {
                info!("Moving {} approved schemas from 'available' to 'schemas' HashMap for field mapping", approved_schemas.len());

                // Move approved schemas from available to schemas HashMap
                for schema_name in &approved_schemas {
                    if let Err(e) = schema_manager.ensure_approved_schema_in_schemas(schema_name) {
                        info!(
                            "Failed to move approved schema '{}' to schemas HashMap: {}",
                            schema_name, e
                        );
                    }
                }

                // Now proceed with field mapping for all approved schemas
                for schema_name in approved_schemas {
                    if let Ok(atom_refs) = schema_manager.map_fields(&schema_name) {
                        // Persist each atom ref using event-driven communication
                        for atom_ref in atom_refs {
                            let aref_uuid = atom_ref.uuid().to_string();
                            let atom_uuid = atom_ref.get_atom_uuid().clone();

                            // Send AtomRefUpdateRequest via message bus
                            let correlation_id = Uuid::new_v4().to_string();
                            let update_request = AtomRefUpdateRequest {
                                correlation_id: correlation_id.clone(),
                                aref_uuid: aref_uuid.clone(),
                                atom_uuid,
                                source_pub_key: "system".to_string(),
                                aref_type: "Single".to_string(), // Default type for schema initialization
                                additional_data: None,
                            };

                            if let Err(e) = message_bus.publish(update_request) {
                                info!(
                                    "Failed to publish AtomRefUpdateRequest for schema '{}': {}",
                                    schema_name, e
                                );
                            }
                        }
                    }
                }
            }
        }

        // Create and start EventMonitor for system-wide observability
        let event_monitor = Arc::new(crate::fold_db_core::infrastructure::event_monitor::EventMonitor::new(
            &message_bus,
        ));
        info!("Started EventMonitor for system-wide event tracking");

        // AtomManager operates via direct method calls, not event consumption.
        // Event-driven components:
        // - EventMonitor: System observability and statistics
        // - TransformOrchestrator: Automatic transform triggering based on field changes

        Ok(Self {
            atom_manager,
            field_retrieval_service: FieldRetrievalService::new(Arc::clone(&message_bus)),
            schema_manager,
            transform_manager,
            transform_orchestrator: orchestrator,
            db_ops: Arc::new(db_ops.clone()),
            permission_wrapper: PermissionWrapper::new(),
            message_bus,
            event_monitor,
            encryption_operations: Arc::new(Mutex::new(EncryptionOperations::new(Arc::new(db_ops.clone())))),
        })
    }

    /// Retrieves or generates and persists the node identifier.
    pub fn get_node_id(&self) -> Result<String, sled::Error> {
        self.db_ops
            .get_node_id()
            .map_err(|e| sled::Error::Unsupported(e.to_string()))
    }

    /// Retrieves the list of permitted schemas for the given node.
    pub fn get_schema_permissions(&self, node_id: &str) -> Vec<String> {
        self.db_ops
            .get_schema_permissions(node_id)
            .unwrap_or_default()
    }

    /// Sets the permitted schemas for the given node.
    pub fn set_schema_permissions(&self, node_id: &str, schemas: &[String]) -> sled::Result<()> {
        self.db_ops
            .set_schema_permissions(node_id, schemas)
            .map_err(|e| sled::Error::Unsupported(e.to_string()))
    }

    // ========== CONSOLIDATED SCHEMA API - DELEGATES TO SCHEMA_CORE ==========

    /// Load schema from JSON string (creates Available schema)
    pub fn load_schema_from_json(&mut self, json_str: &str) -> Result<(), SchemaError> {
        // Delegate to working schema_manager implementation
        self.schema_manager.load_schema_from_json(json_str)
    }

    /// Load schema from file (creates Available schema)
    pub fn load_schema_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), SchemaError> {
        // Delegate to working schema_manager implementation
        self.schema_manager
            .load_schema_from_file(path.as_ref().to_str().unwrap())
    }

    /// Add a schema to available schemas (for testing compatibility)
    pub fn add_schema_available(&mut self, schema: Schema) -> Result<(), SchemaError> {
        // Delegate to working schema_manager implementation
        self.schema_manager.add_schema_available(schema)
    }

    /// Approve a schema for queries and mutations (for testing compatibility)
    pub fn approve_schema(&mut self, schema_name: &str) -> Result<(), SchemaError> {
        // Delegate to working schema_manager implementation
        self.schema_manager.approve_schema(schema_name)
    }

    /// Mark a schema as unloaded without removing transforms.
    pub fn unload_schema(&self, schema_name: &str) -> Result<(), SchemaError> {
        self.schema_manager.unload_schema(schema_name)
    }

    /// Get a schema by name - public accessor for testing
    pub fn get_schema(
        &self,
        schema_name: &str,
    ) -> Result<Option<crate::schema::Schema>, SchemaError> {
        self.schema_manager.get_schema(schema_name)
    }

    // ========== ACCESSOR METHODS ==========

    /// Provides access to the underlying database operations
    pub fn db_ops(&self) -> Arc<DbOperations> {
        Arc::clone(&self.db_ops)
    }

    /// Provides access to the field retrieval service for testing
    pub fn field_retrieval_service(&self) -> &FieldRetrievalService {
        &self.field_retrieval_service
    }

    /// Provides access to the atom manager for testing
    pub fn atom_manager(&self) -> &AtomManager {
        &self.atom_manager
    }

    /// Provides access to the event monitor for observability
    pub fn event_monitor(&self) -> Arc<crate::fold_db_core::infrastructure::event_monitor::EventMonitor> {
        Arc::clone(&self.event_monitor)
    }

    /// Get current event statistics from the event monitor
    pub fn get_event_statistics(&self) -> crate::fold_db_core::infrastructure::event_monitor::EventStatistics {
        self.event_monitor.get_statistics()
    }

    /// Log a summary of all system activity since FoldDB was created
    pub fn log_event_summary(&self) {
        self.event_monitor.log_summary()
    }

    /// Get the message bus for publishing events (for testing)
    pub fn message_bus(&self) -> Arc<MessageBus> {
        Arc::clone(&self.message_bus)
    }

    /// Get the transform manager for testing transform functionality
    pub fn transform_manager(&self) -> Arc<TransformManager> {
        Arc::clone(&self.transform_manager)
    }

    /// Get the schema manager for testing schema functionality
    pub fn schema_manager(&self) -> Arc<SchemaCore> {
        Arc::clone(&self.schema_manager)
    }

    /// Register a transform with the system
    pub fn register_transform(
        &self,
        _transform: crate::schema::types::Transform,
    ) -> Result<(), SchemaError> {
        // For now, return error since TransformRegistration is expected, not Transform
        Err(SchemaError::InvalidData(
            "Transform registration not yet implemented - needs TransformRegistration type"
                .to_string(),
        ))
    }

    /// List all registered transforms
    pub fn list_transforms(
        &self,
    ) -> Result<std::collections::HashMap<String, crate::schema::types::Transform>, SchemaError> {
        self.transform_manager.list_transforms()
    }

    /// Execute a transform by ID using direct execution
    /// This executes the transform immediately and returns the result
    pub fn run_transform(&self, transform_id: &str) -> Result<serde_json::Value, SchemaError> {
        use crate::fold_db_core::transform_manager::types::TransformRunner;
        log::info!(
            "🔄 run_transform called for {} - using direct execution",
            transform_id
        );

        // Use direct execution through the transform manager
        match TransformRunner::execute_transform_now(&*self.transform_manager, transform_id) {
            Ok(result) => Ok(result),
            Err(e) => Err(SchemaError::InvalidData(e.to_string())),
        }
    }

    /// Process any pending transforms in the queue
    pub fn process_transform_queue(&self) {
        // Transform orchestrator processing is handled automatically by events
        // self.transform_orchestrator.process_pending_transforms();
    }

    /// Internal accessor for permission wrapper
    #[allow(dead_code)]
    pub(crate) fn permission_wrapper(&self) -> &PermissionWrapper {
        &self.permission_wrapper
    }
}