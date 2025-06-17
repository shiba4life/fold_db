//! Legacy loading module - functionality moved to registration.rs
//!
//! This module is kept for backward compatibility but delegates
//! all functionality to the new registration module.

use super::registration::TransformRegistrationManager;
use super::state::TransformManagerState;
use crate::db_operations::DbOperations;
use crate::schema::types::{SchemaError, TransformRegistration};
use std::sync::Arc;

/// Legacy loading functionality - now delegates to registration module
pub struct TransformLoading {
    registration_manager: TransformRegistrationManager,
}

impl TransformLoading {
    /// Create a new loading manager
    pub fn new(db_ops: Arc<DbOperations>, state: Arc<TransformManagerState>) -> Self {
        Self {
            registration_manager: TransformRegistrationManager::new(db_ops, state),
        }
    }

    /// Reload transforms - delegates to registration manager
    pub fn reload_transforms(&self) -> Result<(), SchemaError> {
        self.registration_manager.reload_transforms()
    }

    /// Register transform - delegates to registration manager
    pub fn register_transform_event_driven(
        &self,
        registration: TransformRegistration,
    ) -> Result<(), SchemaError> {
        self.registration_manager.register_transform_event_driven(registration)
    }
}
