//! Transform Manager - Main coordinator for transform management system
//!
//! LEGACY REMOVAL COMPLETE (Task 32-5)
//! This module now uses only the UnifiedTransformManager, with all legacy
//! components removed. All functionality is provided through the unified system.
//!
//! CURRENT ARCHITECTURE RESPONSIBILITIES:
//! - Transform Registration: Handled by UnifiedTransformManager
//! - Transform Execution: Handled by UnifiedTransformManager  
//! - Dependency Tracking: Handled by unified state management
//! - Schema Monitoring: Managed by unified orchestration

use super::types::TransformRunner;
use crate::db_operations::DbOperations;
use crate::fold_db_core::infrastructure::message_bus::MessageBus;
use crate::schema::types::{SchemaError, Transform, TransformRegistration, Field};
use crate::transform_execution::{
    UnifiedTransformManager, TransformConfig, TransformDefinition, TransformId,
};
use log::{error, info, warn};
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// TransformManager: Main coordinator for transform management
///
/// LEGACY REMOVAL COMPLETE: This now uses only UnifiedTransformManager
/// with all legacy components removed for a clean, consolidated architecture.
pub struct TransformManager {
    /// Direct database operations (consistent with other components)
    pub(super) db_ops: Arc<DbOperations>,
    /// Unified transform manager (sole delegation target)
    unified_manager: UnifiedTransformManager,
    /// Message bus for event-driven communication
    pub(super) _message_bus: Arc<MessageBus>,
}

impl TransformManager {
    /// Creates a new TransformManager instance with unified architecture only
    pub fn new(
        db_ops: Arc<DbOperations>,
        message_bus: Arc<MessageBus>,
    ) -> Result<Self, SchemaError> {
        info!("🚀 Initializing TransformManager with unified architecture only (Task 32-5 legacy removal complete)");

        // Initialize unified transform manager with default configuration
        let unified_config = TransformConfig::default();
        let unified_manager = UnifiedTransformManager::new(Arc::clone(&db_ops), unified_config)
            .map_err(|e| {
                error!("Failed to initialize UnifiedTransformManager: {:?}", e);
                SchemaError::InvalidData(format!("Unified manager initialization failed: {:?}", e))
            })?;

        // Load existing transforms into unified manager
        let transform_ids = db_ops.list_transforms()?;
        let mut loaded_count = 0;
        
        for transform_id in transform_ids {
            if let Ok(Some(transform)) = db_ops.get_transform(&transform_id) {
                let unified_definition = TransformDefinition {
                    id: transform_id.clone(),
                    transform: transform.clone(),
                    inputs: transform.get_inputs().to_vec(),
                    metadata: {
                        let mut meta = std::collections::HashMap::new();
                        meta.insert("migration_source".to_string(), "initialization".to_string());
                        meta
                    },
                };
                
                if unified_manager.register_transform(unified_definition).is_ok() {
                    loaded_count += 1;
                    info!("✅ Loaded transform '{}' into unified manager", transform_id);
                } else {
                    warn!("⚠️ Failed to load transform '{}' into unified manager", transform_id);
                }
            }
        }

        info!(
            "✅ TransformManager initialized successfully with {} transforms in unified manager",
            loaded_count
        );

        Ok(Self {
            db_ops,
            unified_manager,
            _message_bus: message_bus,
        })
    }

    /// Returns true if a transform with the given id is registered.
    ///
    /// LEGACY REMOVAL: Now uses only unified manager
    pub fn transform_exists(&self, transform_id: &str) -> Result<bool, SchemaError> {
        let transform_id_unified: TransformId = transform_id.to_string();
        let unified_transforms = self.unified_manager.list_transforms(None);
        
        // Check if transform exists in unified manager
        let exists_in_unified = unified_transforms.iter().any(|meta| meta.id == transform_id_unified);
        
        Ok(exists_in_unified)
    }

    /// List all registered transforms.
    ///
    /// LEGACY REMOVAL: Now uses only unified manager  
    pub fn list_transforms(&self) -> Result<HashMap<String, Transform>, SchemaError> {
        // Get transforms from unified manager
        let unified_transforms = self.unified_manager.list_transforms(None);
        
        // Convert unified metadata to legacy format for backward compatibility
        let mut result = HashMap::new();
        for metadata in unified_transforms {
            // Get the Transform from database using the metadata ID
            if let Ok(Some(transform)) = self.db_ops.get_transform(&metadata.id) {
                result.insert(metadata.id.clone(), transform);
            }
        }
        
        Ok(result)
    }

    /// Gets all transforms that depend on the specified atom reference.
    /// 
    /// LEGACY REMOVAL: Now uses unified manager's dependency tracking
    pub fn get_dependent_transforms(
        &self,
        aref_uuid: &str,
    ) -> Result<HashSet<String>, SchemaError> {
        // Use unified manager to find transforms with this dependency
        let all_transforms = self.unified_manager.list_transforms(None);
        let mut dependent_transforms = HashSet::new();
        
        for metadata in all_transforms {
            // Check if any transform depends on this atom reference
            if let Ok(Some(transform)) = self.db_ops.get_transform(&metadata.id) {
                if transform.get_inputs().contains(&aref_uuid.to_string()) {
                    dependent_transforms.insert(metadata.id);
                }
            }
        }
        
        Ok(dependent_transforms)
    }

    /// Gets all atom references that a transform depends on.
    ///
    /// LEGACY REMOVAL: Now uses unified manager
    pub fn get_transform_inputs(&self, transform_id: &str) -> Result<HashSet<String>, SchemaError> {
        if let Ok(Some(transform)) = self.db_ops.get_transform(transform_id) {
            Ok(transform.get_inputs().iter().cloned().collect())
        } else {
            Ok(HashSet::new())
        }
    }

    /// Gets the output atom reference for a transform.
    ///
    /// LEGACY REMOVAL: Now uses unified manager
    pub fn get_transform_output(&self, transform_id: &str) -> Result<Option<String>, SchemaError> {
        if let Ok(Some(transform)) = self.db_ops.get_transform(transform_id) {
            Ok(Some(transform.get_output().to_string()))
        } else {
            Ok(None)
        }
    }

    /// Gets all transforms that should run when the specified field is updated.
    ///
    /// LEGACY REMOVAL: Now uses unified manager with database lookup
    pub fn get_transforms_for_field(
        &self,
        schema_name: &str,
        field_name: &str,
    ) -> Result<HashSet<String>, SchemaError> {
        let field_key = format!("{}.{}", schema_name, field_name);
        let all_transforms = self.unified_manager.list_transforms(None);
        let mut matching_transforms = HashSet::new();
        
        for metadata in all_transforms {
            if let Ok(Some(transform)) = self.db_ops.get_transform(&metadata.id) {
                // Check if this transform has the field as input or dependency
                if transform.get_inputs().contains(&field_key) ||
                   transform.analyze_dependencies().contains(&field_key) {
                    matching_transforms.insert(metadata.id);
                }
            }
        }
        
        Ok(matching_transforms)
    }

    /// Register transform using event-driven database operations only
    ///
    /// LEGACY REMOVAL: Now uses only unified manager
    pub fn register_transform_event_driven(
        &self,
        registration: TransformRegistration,
    ) -> Result<(), SchemaError> {
        // CONSOLIDATED: Direct delegation to unified manager
        info!("📝 TransformManager: Delegating event-driven registration to unified manager: {}", registration.transform_id);
        
        // Load transform from database
        let transform = match self.db_ops.get_transform(&registration.transform_id) {
            Ok(Some(transform)) => transform,
            Ok(None) => {
                return Err(SchemaError::InvalidData(format!(
                    "Transform '{}' not found in database",
                    registration.transform_id
                )));
            },
            Err(e) => {
                return Err(SchemaError::InvalidData(format!(
                    "Failed to get transform from database: {:?}",
                    e
                )));
            }
        };
        
        // Create unified definition and delegate
        let unified_definition = TransformDefinition {
            id: registration.transform_id.clone(),
            transform: transform.clone(),
            inputs: transform.get_inputs().to_vec(),
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("migration_source".to_string(), "event_driven".to_string());
                meta.insert("schema_name".to_string(), registration.schema_name);
                meta.insert("field_name".to_string(), registration.field_name);
                meta
            },
        };
        
        use crate::transform_execution::error::error_conversion::ErrorConversion;
        self.unified_manager.register_transform(unified_definition)
            .map_transform_execution_error()?;
            
        info!("✅ Successfully delegated transform '{}' registration to unified manager", registration.transform_id);
        Ok(())
    }

    /// CONSOLIDATED: Direct delegation to unified manager
    pub fn register_transform_auto(
        &self,
        transform_id: String,
        transform: Transform,
        output_aref: String,
        schema_name: String,
        field_name: String,
    ) -> Result<(), SchemaError> {
        info!("📝 TransformManager: Delegating auto-registration to unified manager: {}", transform_id);
        
        // Create unified definition and delegate - no duplicate logic
        let unified_definition = TransformDefinition {
            id: transform_id.clone(),
            transform: transform.clone(),
            inputs: transform.get_inputs().to_vec(),
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("schema_name".to_string(), schema_name.clone());
                meta.insert("field_name".to_string(), field_name.clone());
                meta.insert("output_aref".to_string(), output_aref.clone());
                meta.insert("migration_source".to_string(), "auto_registration".to_string());
                meta
            },
        };
        
        use crate::transform_execution::error::error_conversion::ErrorConversion;
        self.unified_manager.register_transform(unified_definition)
            .map_transform_execution_error()?;
            
        info!("✅ Successfully delegated auto-registration of '{}' to unified manager", transform_id);
        Ok(())
    }

    /// Unregisters a transform using direct database operations.
    ///
    /// LEGACY REMOVAL: Now uses only unified manager
    pub fn unregister_transform(&self, transform_id: &str) -> Result<bool, SchemaError> {
        let transform_id_unified: TransformId = transform_id.to_string();
        
        // Remove from unified manager
        self.unified_manager.remove_transform(transform_id_unified)
            .map_err(|e| {
                error!("Failed to unregister from unified manager: {:?}", e);
                SchemaError::InvalidData(format!("Unified unregistration failed: {:?}", e))
            })?;
            
        info!("✅ Successfully unregistered transform '{}' from unified manager", transform_id);
        Ok(true)
    }

    /// Reload transforms from database - now delegates to unified manager
    pub fn reload_transforms(&self) -> Result<(), SchemaError> {
        info!("🔄 Reloading transforms into unified manager");
        
        let transform_ids = self.db_ops.list_transforms()?;
        let mut reloaded_count = 0;
        
        for transform_id in transform_ids {
            if let Ok(Some(transform)) = self.db_ops.get_transform(&transform_id) {
                let unified_definition = TransformDefinition {
                    id: transform_id.clone(),
                    transform: transform.clone(),
                    inputs: transform.get_inputs().to_vec(),
                    metadata: {
                        let mut meta = std::collections::HashMap::new();
                        meta.insert("migration_source".to_string(), "reload".to_string());
                        meta
                    },
                };
                
                if self.unified_manager.register_transform(unified_definition).is_ok() {
                    reloaded_count += 1;
                }
            }
        }
        
        info!("✅ Reloaded {} transforms into unified manager", reloaded_count);
        Ok(())
    }



    /// Store transform result (static method for backward compatibility)
    pub fn store_transform_result_generic(
        db_ops: &Arc<DbOperations>,
        transform: &Transform,
        result: &JsonValue,
    ) -> Result<(), SchemaError> {
        if let Some(dot_pos) = transform.get_output().find('.') {
            let schema_name = &transform.get_output()[..dot_pos];
            let field_name = &transform.get_output()[dot_pos + 1..];
            
            // Create atom for the result
            let atom = db_ops.create_atom(
                schema_name,
                "TRANSFORM_SYSTEM".to_string(),
                None,
                result.clone(),
                None,
            )?;
            
            // Update field reference
            let mut schema = db_ops.get_schema(schema_name)?.ok_or_else(|| {
                SchemaError::InvalidData(format!("Schema '{}' not found", schema_name))
            })?;
            
            if let Some(field) = schema.fields.get_mut(field_name) {
                let ref_uuid = match field.ref_atom_uuid() {
                    Some(existing_ref) => existing_ref.clone(),
                    None => {
                        let new_ref_uuid = uuid::Uuid::new_v4().to_string();
                        field.set_ref_atom_uuid(new_ref_uuid.clone());
                        new_ref_uuid
                    }
                };
                
                // Create/update AtomRef
                let atom_ref = crate::atom::AtomRef::new(
                    atom.uuid().to_string(), 
                    "TRANSFORM_SYSTEM".to_string()
                );
                db_ops.store_item(&format!("ref:{}", ref_uuid), &atom_ref)?;
                
                // Save updated schema
                db_ops.store_schema(schema_name, &schema)?;
                
                info!("✅ Stored transform result for {}.{} -> atom {}", schema_name, field_name, atom.uuid());
            }
            
            Ok(())
        } else {
            Err(SchemaError::InvalidField(format!(
                "Invalid output field format '{}', expected 'Schema.field'",
                transform.get_output()
            )))
        }
    }
}

impl TransformRunner for TransformManager {
    /// Execute a transform now using the unified manager
    fn execute_transform_now(&self, transform_id: &str) -> Result<JsonValue, SchemaError> {
        info!("🚀 TransformManager: Delegating to unified manager: {}", transform_id);

        // CONSOLIDATED: Use unified manager for all execution logic
        // This eliminates the duplicate execution implementation that was here
        use crate::transform_execution::types::{TransformInput, ExecutionContext};
        use crate::transform_execution::error::error_conversion::ErrorConversion;
        
        // Load the transform to get output info for context
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

        // Create minimal context - unified manager handles input resolution
        let (schema_name, field_name) = if let Some(dot_pos) = transform.get_output().find('.') {
            (transform.get_output()[..dot_pos].to_string(), transform.get_output()[dot_pos + 1..].to_string())
        } else {
            ("unknown".to_string(), transform_id.to_string())
        };

        let context = ExecutionContext {
            schema_name,
            field_name,
            atom_ref: None,
            timestamp: std::time::SystemTime::now(),
            additional_data: HashMap::new(),
        };

        // Empty input - unified manager will resolve all inputs internally
        let input = TransformInput {
            values: HashMap::new(),
            context
        };

        // Delegate to unified manager - no duplicate logic
        let result = self.unified_manager.execute_transform(transform_id.to_string(), input)
            .map_transform_execution_error()?;

        // Store the result using static method
        Self::store_transform_result_generic(&self.db_ops, &transform, &result.value)?;

        info!(
            "✅ Transform '{}' executed successfully via unified manager: {}",
            transform_id, result.value
        );
        Ok(result.value)
    }

    fn transform_exists(&self, transform_id: &str) -> Result<bool, SchemaError> {
        self.transform_exists(transform_id)
    }

    fn get_transforms_for_field(
        &self,
        schema_name: &str,
        field_name: &str,
    ) -> Result<HashSet<String>, SchemaError> {
        self.get_transforms_for_field(schema_name, field_name)
    }
}
