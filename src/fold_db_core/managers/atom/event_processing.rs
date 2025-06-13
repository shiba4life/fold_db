//! Event processing threads for AtomManager

use super::AtomManager;
use crate::fold_db_core::infrastructure::message_bus::{
    AtomCreateRequest, AtomRefCreateRequest, AtomRefUpdateRequest, AtomUpdateRequest,
    FieldValueSetRequest,
};
use log::{error, info, warn};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

impl AtomManager {
    /// Start background event processing threads for request/response handling
    pub(super) fn start_event_processing(&self) {
        info!("🚀 Starting AtomManager pure event processing");

        let mut threads = self.event_threads.lock().unwrap();

        // Thread 1: AtomCreateRequest processing
        let atom_create_thread = self.start_atom_create_processing();
        threads.push(atom_create_thread);

        // Thread 2: AtomUpdateRequest processing
        let atom_update_thread = self.start_atom_update_processing();
        threads.push(atom_update_thread);

        // Thread 3: AtomRefCreateRequest processing
        let atomref_create_thread = self.start_atomref_create_processing();
        threads.push(atomref_create_thread);

        // Thread 4: AtomRefUpdateRequest processing
        let atomref_update_thread = self.start_atomref_update_processing();
        threads.push(atomref_update_thread);

        // Thread 5: FieldValueSetRequest processing - CRITICAL MUTATION BUG FIX
        let fieldvalueset_thread = self.start_fieldvalueset_processing();
        threads.push(fieldvalueset_thread);

        // DIAGNOSTIC LOG: All handlers now implemented
        info!("🔍 DIAGNOSTIC: AtomManager event threads - AtomCreateRequest: ✅, AtomUpdateRequest: ✅, AtomRefCreateRequest: ✅, AtomRefUpdateRequest: ✅, FieldValueSetRequest: ✅ FIXED");

        info!(
            "✅ AtomManager started {} event processing threads",
            threads.len()
        );
    }

    /// Process AtomCreateRequest events
    fn start_atom_create_processing(&self) -> JoinHandle<()> {
        let mut consumer = self.message_bus.subscribe::<AtomCreateRequest>();
        let manager = self.clone();

        thread::spawn(move || {
            info!("⚛️ AtomCreateRequest processor started");

            loop {
                match consumer.recv_timeout(Duration::from_millis(100)) {
                    Ok(request) => {
                        if let Err(e) = manager.handle_atom_create_request(request) {
                            error!("❌ Error processing AtomCreateRequest: {}", e);
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // Continue waiting
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        warn!("⚠️ AtomCreateRequest channel disconnected");
                        break;
                    }
                }
            }
        })
    }

    /// Process AtomUpdateRequest events
    fn start_atom_update_processing(&self) -> JoinHandle<()> {
        let mut consumer = self.message_bus.subscribe::<AtomUpdateRequest>();
        let manager = self.clone();

        thread::spawn(move || {
            info!("🔄 AtomUpdateRequest processor started");

            loop {
                match consumer.recv_timeout(Duration::from_millis(100)) {
                    Ok(request) => {
                        if let Err(e) = manager.handle_atom_update_request(request) {
                            error!("❌ Error processing AtomUpdateRequest: {}", e);
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // Continue waiting
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        warn!("⚠️ AtomUpdateRequest channel disconnected");
                        break;
                    }
                }
            }
        })
    }

    /// Process AtomRefCreateRequest events
    fn start_atomref_create_processing(&self) -> JoinHandle<()> {
        let mut consumer = self.message_bus.subscribe::<AtomRefCreateRequest>();
        let manager = self.clone();

        thread::spawn(move || {
            info!("🔗 AtomRefCreateRequest processor started");

            loop {
                match consumer.recv_timeout(Duration::from_millis(100)) {
                    Ok(request) => {
                        if let Err(e) = manager.handle_atomref_create_request(request) {
                            error!("❌ Error processing AtomRefCreateRequest: {}", e);
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // Continue waiting
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        warn!("⚠️ AtomRefCreateRequest channel disconnected");
                        break;
                    }
                }
            }
        })
    }

    /// Process AtomRefUpdateRequest events
    fn start_atomref_update_processing(&self) -> JoinHandle<()> {
        let mut consumer = self.message_bus.subscribe::<AtomRefUpdateRequest>();
        let manager = self.clone();

        thread::spawn(move || {
            info!("🔄 AtomRefUpdateRequest processor started");

            loop {
                match consumer.recv_timeout(Duration::from_millis(100)) {
                    Ok(request) => {
                        if let Err(e) = manager.handle_atomref_update_request(request) {
                            error!("❌ Error processing AtomRefUpdateRequest: {}", e);
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // Continue waiting
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        warn!("⚠️ AtomRefUpdateRequest channel disconnected");
                        break;
                    }
                }
            }
        })
    }

    /// Process FieldValueSetRequest events - CRITICAL MUTATION BUG FIX
    fn start_fieldvalueset_processing(&self) -> JoinHandle<()> {
        let mut consumer = self.message_bus.subscribe::<FieldValueSetRequest>();
        let manager = self.clone();

        thread::spawn(move || {
            info!("📝 FieldValueSetRequest processor started - CRITICAL MUTATION BUG FIX");

            loop {
                match consumer.recv_timeout(Duration::from_millis(100)) {
                    Ok(request) => {
                        if let Err(e) = manager.handle_fieldvalueset_request(request) {
                            error!("❌ Error processing FieldValueSetRequest: {}", e);
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // Continue waiting
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        warn!("⚠️ FieldValueSetRequest channel disconnected");
                        break;
                    }
                }
            }
        })
    }
}
