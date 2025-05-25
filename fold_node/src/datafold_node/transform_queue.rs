use crate::error::{FoldDbError, FoldDbResult};
use super::DataFoldNode;

impl DataFoldNode {
    /// Add a transform to the queue
    pub fn add_transform_to_queue(&self, transform_id: &str) -> FoldDbResult<()> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        db.transform_orchestrator.add_transform(transform_id, "manual")?;
        Ok(())
    }

    /// Get information about the transform queue
    pub fn get_transform_queue_info(&self) -> FoldDbResult<serde_json::Value> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        let queue_length = db.transform_orchestrator.len()?;
        let is_empty = db.transform_orchestrator.is_empty()?;
        let mut queue = Vec::new();
        let mut current = queue_length;
        while current > 0 {
            if let Some(Ok(id)) = db.transform_orchestrator.process_one() {
                queue.push(id.to_string());
            }
            current -= 1;
        }
        Ok(serde_json::json!({
            "queue": queue,
            "length": queue_length,
            "isEmpty": is_empty
        }))
    }
}


