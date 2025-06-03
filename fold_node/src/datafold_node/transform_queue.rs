use super::DataFoldNode;
use crate::error::{FoldDbError, FoldDbResult};

impl DataFoldNode {
    /// Add a transform to the queue
    pub fn add_transform_to_queue(&self, transform_id: &str) -> FoldDbResult<()> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        db.transform_orchestrator
            .add_transform(transform_id, "manual")?;
        Ok(())
    }

    /// Get information about the transform queue
    pub fn get_transform_queue_info(&self) -> FoldDbResult<serde_json::Value> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        let queue = db.transform_orchestrator.list_queued_transforms()?;
        let queue_length = queue.len();
        let is_empty = db.transform_orchestrator.is_empty()?;
        Ok(serde_json::json!({
            "queue": queue,
            "length": queue_length,
            "isEmpty": is_empty
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datafold_node::config::NodeConfig;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn queue_info_works() {
        let dir = tempdir().unwrap();
        let config = NodeConfig {
            storage_path: dir.path().to_path_buf(),
            default_trust_distance: 1,
            network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
        };
        let node = DataFoldNode::new(config).unwrap();
        let info = node.get_transform_queue_info().unwrap();
        assert!(info.get("queue").is_some());
    }

    #[test]
    fn queue_info_does_not_run_transforms() {
        let dir = tempdir().unwrap();
        let config = NodeConfig {
            storage_path: dir.path().to_path_buf(),
            default_trust_distance: 1,
            network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
        };
        let mut node = DataFoldNode::new(config).unwrap();

        node.load_schema_from_file("src/datafold_node/samples/data/TransformBase.json")
            .unwrap();
        node.approve_schema("TransformBase").unwrap();
        node.load_schema_from_file("src/datafold_node/samples/data/TransformSchema.json")
            .unwrap();
        node.approve_schema("TransformSchema").unwrap();

        node.add_transform_to_queue("TransformSchema.result")
            .unwrap();
        let initial_len = {
            let db = node.db.lock().unwrap();
            db.transform_orchestrator.len().unwrap()
        };
        assert_eq!(initial_len, 1);

        let info = node.get_transform_queue_info().unwrap();
        let after_len = {
            let db = node.db.lock().unwrap();
            db.transform_orchestrator.len().unwrap()
        };
        assert_eq!(after_len, initial_len);
        assert_eq!(info["queue"], json!(["TransformSchema.result"]));

        node.process_transform_queue().unwrap();
        let final_len = {
            let db = node.db.lock().unwrap();
            db.transform_orchestrator.len().unwrap()
        };
        assert_eq!(final_len, 0);
    }
}
