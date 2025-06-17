//! Legacy persistence module - functionality moved to state.rs
//!
//! This module is kept for backward compatibility but delegates
//! all functionality to the new state module.

use super::state::TransformManagerState;
use crate::db_operations::DbOperations;
use crate::schema::types::SchemaError;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Legacy persistence functionality - now delegates to state module
pub struct TransformPersistence {
    state: Arc<TransformManagerState>,
    db_ops: Arc<DbOperations>,
}

impl TransformPersistence {
    /// Create a new persistence manager
    pub fn new(db_ops: Arc<DbOperations>, state: Arc<TransformManagerState>) -> Self {
        Self { state, db_ops }
    }

    /// Persist mappings - delegates to state module
    pub fn persist_mappings_direct(&self) -> Result<(), SchemaError> {
        self.state.persist_mappings(&self.db_ops)
    }

    /// Load persisted mappings - delegates to state module
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
        let state = TransformManagerState::new();
        state.load_persisted_mappings(db_ops)?;

        // Extract the loaded data
        let aref_to_transforms = crate::fold_db_core::transform_manager::utils::TransformUtils::read_lock(&state.aref_to_transforms, "aref_to_transforms")?.clone();
        let transform_to_arefs = crate::fold_db_core::transform_manager::utils::TransformUtils::read_lock(&state.transform_to_arefs, "transform_to_arefs")?.clone();
        let transform_input_names = crate::fold_db_core::transform_manager::utils::TransformUtils::read_lock(&state.transform_input_names, "transform_input_names")?.clone();
        let field_to_transforms = crate::fold_db_core::transform_manager::utils::TransformUtils::read_lock(&state.field_to_transforms, "field_to_transforms")?.clone();
        let transform_to_fields = crate::fold_db_core::transform_manager::utils::TransformUtils::read_lock(&state.transform_to_fields, "transform_to_fields")?.clone();
        let transform_outputs = crate::fold_db_core::transform_manager::utils::TransformUtils::read_lock(&state.transform_outputs, "transform_outputs")?.clone();

        Ok((
            aref_to_transforms,
            transform_to_arefs,
            transform_input_names,
            field_to_transforms,
            transform_to_fields,
            transform_outputs,
        ))
    }
}
