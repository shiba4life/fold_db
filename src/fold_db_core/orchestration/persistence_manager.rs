//! Persistence management component for the Transform Orchestrator
//! 
//! Handles state persistence logic using sled database operations,
//! extracted from the main TransformOrchestrator for better separation of concerns.

use sled::Tree;
use log::{error, info};
use crate::schema::SchemaError;
use super::queue_manager::QueueState;

/// Manages persistence operations for queue state
pub struct PersistenceManager {
    tree: Tree,
}

impl PersistenceManager {
    /// Create a new PersistenceManager with the given sled tree
    pub fn new(tree: Tree) -> Self {
        Self { tree }
    }

    /// Save the current queue state to persistent storage
    pub fn save_state(&self, state: &QueueState) -> Result<(), SchemaError> {
        info!("💾 SAVE_STATE START - saving orchestrator state to disk");
        
        info!("📋 Current state to persist - queue length: {}, queued count: {}, processed count: {}",
            state.queue.len(), state.queued.len(), state.processed.len());
        info!("📋 Queue items: {:?}", state.queue);
        info!("📋 Queued set: {:?}", state.queued);
        info!("📋 Processed set: {:?}", state.processed);
        
        // Use consistent serialization pattern from SerializationHelper
        let state_bytes = serde_json::to_vec(state).map_err(|e| {
            let error_msg = format!("Failed to serialize orchestrator state: {}", e);
            error!("❌ {}", error_msg);
            SchemaError::InvalidData(error_msg)
        })?;
        
        info!("💾 Inserting state into tree (size: {} bytes)", state_bytes.len());
        self.tree.insert("state", state_bytes).map_err(|e| {
            error!("❌ Failed to insert orchestrator state into tree: {}", e);
            SchemaError::InvalidData(format!("Failed to persist orchestrator state: {}", e))
        })?;
        
        info!("✅ SAVE_STATE COMPLETE - state saved successfully");
        Ok(())
    }

    /// Load queue state from persistent storage
    pub fn load_state(&self) -> Result<QueueState, SchemaError> {
        info!("📖 LOAD_STATE START - loading orchestrator state from disk");
        
        let state = self.tree
            .get("state")
            .map_err(|e| {
                error!("❌ Failed to get state from tree: {}", e);
                SchemaError::InvalidData(format!("Failed to load state: {}", e))
            })?
            .map(|v| serde_json::from_slice::<QueueState>(&v))
            .transpose()
            .map_err(|e| {
                let error_msg = format!("Failed to deserialize orchestrator state: {}", e);
                error!("❌ {}", error_msg);
                SchemaError::InvalidData(error_msg)
            })?
            .unwrap_or_else(|| {
                info!("📋 No existing state found, creating new empty state");
                QueueState::default()
            });

        info!("📖 LOAD_STATE COMPLETE - loaded state with queue length: {}, queued count: {}, processed count: {}",
            state.queue.len(), state.queued.len(), state.processed.len());
        info!("📋 Loaded queue items: {:?}", state.queue);
        info!("📋 Loaded queued set: {:?}", state.queued);
        info!("📋 Loaded processed set: {:?}", state.processed);
        
        Ok(state)
    }

    /// Flush changes to disk to ensure persistence
    pub fn flush(&self) -> Result<(), SchemaError> {
        info!("💾 Flushing tree to disk");
        self.tree.flush().map_err(|e| {
            error!("❌ Failed to flush orchestrator state to disk: {}", e);
            SchemaError::InvalidData(format!("Failed to flush orchestrator state: {}", e))
        })?;
        
        info!("✅ Tree flushed successfully");
        Ok(())
    }

    /// Save state and immediately flush to disk for guaranteed persistence
    pub fn save_and_flush(&self, state: &QueueState) -> Result<(), SchemaError> {
        self.save_state(state)?;
        self.flush()?;
        Ok(())
    }

    /// Check if state exists in persistent storage
    pub fn state_exists(&self) -> Result<bool, SchemaError> {
        let exists = self.tree
            .get("state")
            .map_err(|e| {
                error!("❌ Failed to check state existence: {}", e);
                SchemaError::InvalidData(format!("Failed to check state existence: {}", e))
            })?
            .is_some();
        
        info!("🔍 State exists in storage: {}", exists);
        Ok(exists)
    }

    /// Clear all persistent state (useful for testing or reset operations)
    pub fn clear_state(&self) -> Result<(), SchemaError> {
        info!("🗑️ Clearing persistent state");
        
        self.tree.remove("state").map_err(|e| {
            error!("❌ Failed to clear state: {}", e);
            SchemaError::InvalidData(format!("Failed to clear state: {}", e))
        })?;
        
        self.flush()?;
        info!("✅ State cleared successfully");
        Ok(())
    }

    /// Get the underlying tree for advanced operations (use carefully)
    pub fn get_tree(&self) -> &Tree {
        &self.tree
    }

    /// Create a backup of the current state with a timestamp key
    pub fn backup_state(&self) -> Result<String, SchemaError> {
        let current_state = self.load_state()?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| {
                error!("❌ Failed to get timestamp: {}", e);
                SchemaError::InvalidData("Failed to get timestamp".to_string())
            })?
            .as_secs();
        
        let backup_key = format!("state_backup_{}", timestamp);
        
        let state_bytes = serde_json::to_vec(&current_state).map_err(|e| {
            let error_msg = format!("Failed to serialize state for backup: {}", e);
            error!("❌ {}", error_msg);
            SchemaError::InvalidData(error_msg)
        })?;
        
        self.tree.insert(&backup_key, state_bytes).map_err(|e| {
            error!("❌ Failed to save backup: {}", e);
            SchemaError::InvalidData(format!("Failed to save backup: {}", e))
        })?;
        
        self.flush()?;
        info!("✅ State backed up with key: {}", backup_key);
        Ok(backup_key)
    }

    /// Restore state from a backup key
    pub fn restore_from_backup(&self, backup_key: &str) -> Result<(), SchemaError> {
        info!("🔄 Restoring state from backup: {}", backup_key);
        
        let backup_data = self.tree
            .get(backup_key)
            .map_err(|e| {
                error!("❌ Failed to get backup data: {}", e);
                SchemaError::InvalidData(format!("Failed to get backup data: {}", e))
            })?
            .ok_or_else(|| {
                error!("❌ Backup key not found: {}", backup_key);
                SchemaError::InvalidData(format!("Backup key not found: {}", backup_key))
            })?;
        
        let state: QueueState = serde_json::from_slice(&backup_data).map_err(|e| {
            let error_msg = format!("Failed to deserialize backup data: {}", e);
            error!("❌ {}", error_msg);
            SchemaError::InvalidData(error_msg)
        })?;
        
        self.save_and_flush(&state)?;
        info!("✅ State restored from backup: {}", backup_key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fold_db_core::orchestration::queue_manager::QueueItem;

    fn create_test_tree() -> Tree {
        let config = sled::Config::new().temporary(true);
        let db = config.open().expect("Failed to create test database");
        db.open_tree("test_persistence").expect("Failed to create test tree")
    }

    #[test]
    fn test_save_and_load_state() {
        let tree = create_test_tree();
        let manager = PersistenceManager::new(tree);
        
        // Create test state
        let mut test_state = QueueState::default();
        test_state.queue.push_back(QueueItem {
            id: "test_transform".to_string(),
            mutation_hash: "test_hash".to_string(),
        });
        test_state.queued.insert("test_transform|test_hash".to_string());
        test_state.processed.insert("processed_transform|processed_hash".to_string());
        
        // Save state
        manager.save_state(&test_state).unwrap();
        manager.flush().unwrap();
        
        // Load state
        let loaded_state = manager.load_state().unwrap();
        
        // Verify state matches
        assert_eq!(loaded_state.queue.len(), 1);
        assert_eq!(loaded_state.queued.len(), 1);
        assert_eq!(loaded_state.processed.len(), 1);
        assert_eq!(loaded_state.queue[0].id, "test_transform");
        assert!(loaded_state.queued.contains("test_transform|test_hash"));
        assert!(loaded_state.processed.contains("processed_transform|processed_hash"));
    }

    #[test]
    fn test_state_exists() {
        let tree = create_test_tree();
        let manager = PersistenceManager::new(tree);
        
        // Initially no state
        assert!(!manager.state_exists().unwrap());
        
        // Save state
        let state = QueueState::default();
        manager.save_state(&state).unwrap();
        
        // Now state exists
        assert!(manager.state_exists().unwrap());
    }

    #[test]
    fn test_clear_state() {
        let tree = create_test_tree();
        let manager = PersistenceManager::new(tree);
        
        // Save state
        let state = QueueState::default();
        manager.save_and_flush(&state).unwrap();
        assert!(manager.state_exists().unwrap());
        
        // Clear state
        manager.clear_state().unwrap();
        assert!(!manager.state_exists().unwrap());
    }

    #[test]
    fn test_backup_and_restore() {
        let tree = create_test_tree();
        let manager = PersistenceManager::new(tree);
        
        // Create and save initial state
        let mut initial_state = QueueState::default();
        initial_state.queue.push_back(QueueItem {
            id: "initial_transform".to_string(),
            mutation_hash: "initial_hash".to_string(),
        });
        manager.save_and_flush(&initial_state).unwrap();
        
        // Create backup
        let backup_key = manager.backup_state().unwrap();
        
        // Modify state
        let mut modified_state = QueueState::default();
        modified_state.queue.push_back(QueueItem {
            id: "modified_transform".to_string(),
            mutation_hash: "modified_hash".to_string(),
        });
        manager.save_and_flush(&modified_state).unwrap();
        
        // Restore from backup
        manager.restore_from_backup(&backup_key).unwrap();
        
        // Verify restoration
        let restored_state = manager.load_state().unwrap();
        assert_eq!(restored_state.queue.len(), 1);
        assert_eq!(restored_state.queue[0].id, "initial_transform");
    }
}