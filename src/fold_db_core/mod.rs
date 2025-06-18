//! FoldDB Core - Event-driven database system
//!
//! This module contains the core components of the FoldDB system organized
//! into logical groups for better maintainability and understanding:
//!
//! - **coordinator**: Main FoldDB struct and initialization logic
//! - **operations/**: Split operation modules (mutations, queries, encryption)
//! - **managers/**: Core managers for different aspects of data management
//! - **services/**: Service layer components for operations
//! - **infrastructure/**: Foundation components (message bus, initialization, etc.)
//! - **orchestration/**: Coordination and orchestration components
//! - **shared/**: Common utilities and shared components
//! - **transform_manager/**: Transform system (already well-organized)
//! - **core_tests**: Test suite for core functionality

// Module declarations
pub mod coordinator;
pub mod core_tests;
pub mod infrastructure;
pub mod managers;
pub mod operations;
pub mod orchestration;
pub mod services;
pub mod shared;
pub mod transform_manager;

// Re-export key components for backwards compatibility
pub use coordinator::FoldDB;
pub use infrastructure::{EventMonitor, MessageBus};
pub use managers::AtomManager;
pub use operations::{EncryptionOperations, MutationOperations, QueryOperations};
pub use orchestration::TransformOrchestrator;
pub use services::field_retrieval::service::FieldRetrievalService;
pub use shared::*;
pub use transform_manager::TransformManager;

// Import infrastructure components that are used internally
use infrastructure::message_bus::{
    AtomCreateResponse, AtomRefCreateResponse, FieldUpdateResponse, FieldValueSetResponse,
    SchemaApprovalResponse, SchemaLoadResponse,
};

/// Unified response type for all operations
#[derive(Debug, Clone)]
pub enum OperationResponse {
    FieldValueSetResponse(FieldValueSetResponse),
    FieldUpdateResponse(FieldUpdateResponse),
    SchemaLoadResponse(SchemaLoadResponse),
    SchemaApprovalResponse(SchemaApprovalResponse),
    AtomCreateResponse(AtomCreateResponse),
    AtomRefCreateResponse(AtomRefCreateResponse),
    Error(String),
    Timeout,
}

// Extended FoldDB implementation that uses the operations modules
impl FoldDB {
    /// Write schema operation using mutation operations module
    pub fn write_schema(&mut self, mutation: crate::schema::types::Mutation) -> Result<(), crate::schema::SchemaError> {
        let mut mutation_ops = MutationOperations::new(
            self.schema_manager(),
            crate::permissions::PermissionWrapper::new(),
            self.message_bus(),
        );
        mutation_ops.write_schema(mutation)
    }

    /// Query operation using query operations module
    pub fn query(&self, query: crate::schema::types::Query) -> Result<serde_json::Value, crate::schema::SchemaError> {
        let query_ops = QueryOperations::new(
            self.schema_manager(),
            crate::permissions::PermissionWrapper::new(),
            self.db_ops(),
        );
        query_ops.query(query)
    }

    /// Query a Range schema using query operations module
    pub fn query_range_schema(&self, query: crate::schema::types::Query) -> Result<serde_json::Value, crate::schema::SchemaError> {
        let query_ops = QueryOperations::new(
            self.schema_manager(),
            crate::permissions::PermissionWrapper::new(),
            self.db_ops(),
        );
        query_ops.query_range_schema(query)
    }

    /// Query a schema (compatibility method) using query operations module
    pub fn query_schema(&self, query: crate::schema::types::Query) -> Vec<Result<serde_json::Value, crate::schema::SchemaError>> {
        let query_ops = QueryOperations::new(
            self.schema_manager(),
            crate::permissions::PermissionWrapper::new(),
            self.db_ops(),
        );
        query_ops.query_schema(query)
    }

    /// Enable encryption for atom storage with the given master key pair
    pub fn enable_atom_encryption(
        &mut self,
        master_keypair: &crate::crypto::MasterKeyPair,
    ) -> Result<(), crate::schema::SchemaError> {
        let mut encryption_ops = EncryptionOperations::new(self.db_ops());
        encryption_ops.enable_atom_encryption(master_keypair, &mut self.atom_manager)
    }

    /// Enable encryption for atom storage with crypto config
    pub fn enable_atom_encryption_with_config(
        &mut self,
        master_keypair: &crate::crypto::MasterKeyPair,
        crypto_config: &crate::config::crypto::CryptoConfig,
    ) -> Result<(), crate::schema::SchemaError> {
        let mut encryption_ops = EncryptionOperations::new(self.db_ops());
        encryption_ops.enable_atom_encryption_with_config(master_keypair, crypto_config, &mut self.atom_manager)
    }

    /// Disable encryption for atom storage (fallback to unencrypted)
    pub fn disable_atom_encryption(&mut self) {
        let mut encryption_ops = EncryptionOperations::new(self.db_ops());
        encryption_ops.disable_atom_encryption();
    }

    /// Check if atom encryption is enabled
    pub fn is_atom_encryption_enabled(&self) -> bool {
        let encryption_ops = EncryptionOperations::new(self.db_ops());
        encryption_ops.is_atom_encryption_enabled()
    }

    /// Get encryption statistics
    pub fn get_encryption_stats(
        &self,
    ) -> Result<std::collections::HashMap<String, u64>, crate::schema::SchemaError> {
        let encryption_ops = EncryptionOperations::new(self.db_ops());
        encryption_ops.get_encryption_stats()
    }

    /// Migrate existing unencrypted atoms to encrypted format
    pub fn migrate_atoms_to_encrypted(&mut self) -> Result<u64, crate::schema::SchemaError> {
        let mut encryption_ops = EncryptionOperations::new(self.db_ops());
        encryption_ops.migrate_atoms_to_encrypted()
    }

    /// Get a reference to the encryption wrapper for advanced operations
    pub fn encryption_wrapper(&self) -> Option<&std::sync::Arc<crate::db_operations::EncryptionWrapper>> {
        // This method needs to be handled differently since we can't access the internal encryption wrapper
        // from the operations module. For now, return None.
        // TODO: Refactor to properly expose encryption wrapper through operations module
        None
    }
}
