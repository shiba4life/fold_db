//! Pure Event-Driven AtomManager Module
//!
//! This module contains the AtomManager implementation broken down into logical components:
//! - Main AtomManager struct and interface
//! - Event processing threads
//! - Request handlers
//! - Field processing utilities
//! - Helper methods

mod event_processing;
mod request_handlers;
mod field_processing;
mod helpers;

use crate::atom::{Atom, AtomRef, AtomRefCollection, AtomRefRange};
use crate::db_operations::DbOperations;
use crate::fold_db_core::infrastructure::message_bus::MessageBus;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

/// Re-export unified statistics from shared stats module
pub use crate::fold_db_core::shared::EventDrivenAtomStats;

/// Pure event-driven AtomManager that only communicates via events
pub struct AtomManager {
    pub(crate) db_ops: Arc<DbOperations>,
    pub(crate) atoms: Arc<Mutex<HashMap<String, Atom>>>,
    pub(crate) ref_atoms: Arc<Mutex<HashMap<String, AtomRef>>>,
    pub(crate) ref_collections: Arc<Mutex<HashMap<String, AtomRefCollection>>>,
    pub(crate) ref_ranges: Arc<Mutex<HashMap<String, AtomRefRange>>>,
    pub(crate) message_bus: Arc<MessageBus>,
    pub(crate) stats: Arc<Mutex<EventDrivenAtomStats>>,
    pub(crate) event_threads: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl AtomManager {
    pub fn new(db_ops: DbOperations, message_bus: Arc<MessageBus>) -> Self {
        let mut atoms = HashMap::new();
        let mut ref_atoms = HashMap::new();
        let mut ref_collections = HashMap::new();
        let mut ref_ranges = HashMap::new();

        // Load existing data from database
        for result in db_ops.db().iter().flatten() {
            let key_str = String::from_utf8_lossy(result.0.as_ref());
            let bytes = result.1.as_ref();

            if let Some(stripped) = key_str.strip_prefix("atom:") {
                if let Ok(atom) = serde_json::from_slice(bytes) {
                    atoms.insert(stripped.to_string(), atom);
                }
            } else if let Some(stripped) = key_str.strip_prefix("ref:") {
                if let Ok(atom_ref) = serde_json::from_slice::<AtomRef>(bytes) {
                    ref_atoms.insert(stripped.to_string(), atom_ref);
                } else if let Ok(collection) = serde_json::from_slice::<AtomRefCollection>(bytes) {
                    ref_collections.insert(stripped.to_string(), collection);
                } else if let Ok(range) = serde_json::from_slice::<AtomRefRange>(bytes) {
                    ref_ranges.insert(stripped.to_string(), range);
                }
            }
        }

        let manager = Self {
            db_ops: Arc::new(db_ops),
            atoms: Arc::new(Mutex::new(atoms)),
            ref_atoms: Arc::new(Mutex::new(ref_atoms)),
            ref_collections: Arc::new(Mutex::new(ref_collections)),
            ref_ranges: Arc::new(Mutex::new(ref_ranges)),
            message_bus: Arc::clone(&message_bus),
            stats: Arc::new(Mutex::new(EventDrivenAtomStats::new())),
            event_threads: Arc::new(Mutex::new(Vec::new())),
        };

        // Start pure event-driven processing
        manager.start_event_processing();
        manager
    }

    /// Public API methods for direct access (for backward compatibility)
    pub fn create_atom(
        &self,
        schema_name: &str,
        source_pub_key: String,
        content: serde_json::Value,
    ) -> Result<Atom, Box<dyn std::error::Error>> {
        helpers::create_atom(&self.db_ops, schema_name, source_pub_key, content)
    }

    pub fn update_atom_ref(
        &self,
        aref_uuid: &str,
        atom_uuid: String,
        source_pub_key: String,
    ) -> Result<AtomRef, Box<dyn std::error::Error>> {
        helpers::update_atom_ref(&self.db_ops, aref_uuid, atom_uuid, source_pub_key)
    }

    pub fn update_atom_ref_range(
        &self,
        aref_uuid: &str,
        atom_uuid: String,
        key: String,
        source_pub_key: String,
    ) -> Result<AtomRefRange, Box<dyn std::error::Error>> {
        helpers::update_atom_ref_range(&self.db_ops, aref_uuid, atom_uuid, key, source_pub_key)
    }

    pub fn get_atom_history(
        &self,
        aref_uuid: &str,
    ) -> Result<Vec<crate::atom::Atom>, Box<dyn std::error::Error>> {
        helpers::get_atom_history(&self.db_ops, aref_uuid)
    }

    /// Get current statistics for testing
    pub fn get_stats(&self) -> EventDrivenAtomStats {
        self.stats.lock().unwrap().clone()
    }
}

impl Clone for AtomManager {
    fn clone(&self) -> Self {
        Self {
            db_ops: Arc::clone(&self.db_ops),
            atoms: Arc::clone(&self.atoms),
            ref_atoms: Arc::clone(&self.ref_atoms),
            ref_collections: Arc::clone(&self.ref_collections),
            ref_ranges: Arc::clone(&self.ref_ranges),
            message_bus: Arc::clone(&self.message_bus),
            stats: Arc::clone(&self.stats),
            event_threads: Arc::clone(&self.event_threads),
        }
    }
}