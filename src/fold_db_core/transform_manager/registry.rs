//! Legacy registry module - functionality moved to registration.rs
//!
//! This module is kept for backward compatibility but delegates
//! all functionality to the new registration module.

use super::registration::TransformRegistrationManager;
use super::state::TransformManagerState;
use crate::db_operations::DbOperations;
use crate::schema::types::{SchemaError, Transform};
use std::sync::Arc;

/// Legacy registry functionality - now delegates to registration module
pub struct TransformRegistry {
    registration_manager: TransformRegistrationManager,
}

impl TransformRegistry {
    /// Create a new registry manager
    pub fn new(db_ops: Arc<DbOperations>, state: Arc<TransformManagerState>) -> Self {
        Self {
            registration_manager: TransformRegistrationManager::new(db_ops, state),
        }
    }

    /// Register transform with auto-detection - delegates to registration manager
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

    /// Unregister transform - delegates to registration manager
    pub fn unregister_transform(&self, transform_id: &str) -> Result<bool, SchemaError> {
        self.registration_manager.unregister_transform(transform_id)
    }
}
