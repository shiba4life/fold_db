use super::types::TransformRunner;
use crate::db_operations::DbOperations;
use crate::fold_db_core::infrastructure::message_bus::MessageBus;
use crate::schema::types::{SchemaError, Transform};
use log::info;
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
    /// Thread handle for monitoring TransformTriggered events
    pub(super) _transform_triggered_consumer_thread: Option<thread::JoinHandle<()>>,
    /// Thread handle for processing TransformTriggerRequest events
    pub(super) _transform_trigger_request_thread: Option<thread::JoinHandle<()>>,
    /// Thread handle for processing TransformExecutionRequest events
    pub(super) _transform_execution_request_thread: Option<thread::JoinHandle<()>>,
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

        // Field mappings will be auto-registered during reload_transforms()
        // which is triggered by SchemaChanged events, avoiding duplicate registration

        // Set up monitoring of TransformTriggered events for orchestrated execution
        let transform_triggered_consumer_thread = Self::setup_transform_triggered_monitoring(
            Arc::clone(&message_bus),
        );

        // Set up request/response processing threads
        let transform_trigger_request_thread = Self::setup_transform_trigger_request_processing(
            Arc::clone(&message_bus),
        );

        let transform_execution_request_thread = Self::setup_transform_execution_request_processing(
            Arc::clone(&message_bus),
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
            _transform_triggered_consumer_thread: Some(transform_triggered_consumer_thread),
            _transform_trigger_request_thread: Some(transform_trigger_request_thread),
            _transform_execution_request_thread: Some(transform_execution_request_thread),
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
        Ok(field_to_transforms.get(&key).cloned().unwrap_or_default())
    }
}

impl TransformRunner for TransformManager {
    /// Legacy method maintained for API compatibility - redirects to event-driven execution
    /// This method publishes a TransformExecutionRequest event instead of direct execution
    fn execute_transform_now(&self, transform_id: &str) -> Result<JsonValue, SchemaError> {
        log::info!("🔄 execute_transform_now called for {} - using event-driven execution", transform_id);
        
        // Publish a TransformExecutionRequest event
        let execution_request = crate::fold_db_core::infrastructure::message_bus::TransformExecutionRequest {
            correlation_id: format!("api_request_{}", transform_id),
        };
        
        match self.message_bus.publish(execution_request) {
            Ok(_) => {
                log::info!("📢 Published TransformExecutionRequest for {}", transform_id);
                // Return a placeholder indicating the event was published
                Ok(serde_json::json!({
                    "status": "execution_requested",
                    "transform_id": transform_id,
                    "method": "event_driven"
                }))
            }
            Err(e) => {
                log::error!("❌ Failed to publish TransformExecutionRequest for {}: {}", transform_id, e);
                Err(SchemaError::InvalidData(format!("Failed to request transform execution: {}", e)))
            }
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
