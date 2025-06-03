#![allow(dead_code)]
pub mod node_operations;
pub mod operation_builder;
pub mod schema_builder;

// Re-export testing utilities for all tests
use fold_node::FoldDB;
use fold_node::{DataFoldNode, NodeConfig};
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;
use uuid::Uuid;

static CLEANUP_LOCK: Mutex<()> = Mutex::new(());

fn retry_with_backoff<F, T, E>(mut f: F, retries: u32) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    let mut attempt = 0;
    loop {
        match f() {
            ok @ Ok(_) => return ok,
            Err(_) if attempt < retries => {
                attempt += 1;
                thread::sleep(Duration::from_millis(100 * attempt as u64));
                continue;
            }
            err => return err,
        }
    }
}

pub fn get_test_db_path() -> String {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let tmp_dir = current_dir.join("tmp");

    // Create tmp directory and ensure it exists
    fs::create_dir_all(&tmp_dir).expect("Failed to create tmp directory");

    // Replace any potentially problematic characters in the UUID
    let safe_uuid = Uuid::new_v4().to_string().replace("-", "_");
    let db_path = tmp_dir.join(format!("test_db_{}", safe_uuid));

    // Create the database directory
    fs::create_dir_all(&db_path).expect("Failed to create database directory");

    // Create schemas subdirectory with proper error handling
    let schemas_dir = db_path.join("schemas");
    match fs::create_dir_all(&schemas_dir) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Warning: Failed to create schemas directory: {}", e);
            // Try an alternative approach
            let schemas_path = db_path.to_string_lossy().into_owned() + "/schemas";
            fs::create_dir_all(schemas_path)
                .expect("Failed to create schemas directory with alternative method");
        }
    }

    db_path.to_string_lossy().into_owned()
}

pub fn cleanup_test_db(path: &str) {
    let _lock = CLEANUP_LOCK.lock().unwrap();
    let path = Path::new(path);
    if path.exists() {
        for _ in 0..3 {
            // Try up to 3 times
            if fs::remove_dir_all(path).is_ok() {
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}

pub fn cleanup_tmp_dir() {
    let _lock = CLEANUP_LOCK.lock().unwrap();
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let tmp_dir = current_dir.join("tmp");

    // First ensure the directory exists
    let _ = fs::create_dir_all(&tmp_dir);

    // Remove all contents with retries
    let cleanup_contents = || -> std::io::Result<()> {
        if let Ok(entries) = fs::read_dir(&tmp_dir) {
            for entry in entries.flatten() {
                let _ = fs::remove_dir_all(entry.path());
            }
        }
        Ok(())
    };

    if let Err(e) = retry_with_backoff(cleanup_contents, 5) {
        eprintln!("Warning: Failed to clean contents: {}", e);
    }

    // Verify the directory is empty
    let verify_empty = || -> std::io::Result<()> {
        if let Ok(entries) = fs::read_dir(&tmp_dir) {
            if entries.count() == 0 {
                Ok(())
            } else {
                Err(std::io::Error::other("Directory not empty"))
            }
        } else {
            Ok(())
        }
    };

    if let Err(e) = retry_with_backoff(verify_empty, 5) {
        eprintln!("Warning: Directory may not be empty: {}", e);
    }
}

pub fn setup_test_db() -> (FoldDB, String) {
    let db_path = get_test_db_path();
    let db = FoldDB::new(&db_path).expect("Failed to create test database");
    (db, db_path)
}

pub fn setup_and_allow_schema(
    db: &mut FoldDB,
    schema_name: &str,
) -> Result<(), fold_node::testing::SchemaError> {
    db.allow_schema(schema_name)
}

pub fn create_test_node() -> DataFoldNode {
    let dir = tempdir().expect("temp dir");
    let config = NodeConfig {
        storage_path: dir.path().to_path_buf(),
        default_trust_distance: 1,
        network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
    };
    DataFoldNode::new(config).expect("Failed to create test node")
}

/// Helper function to set up a test node with proper schema permissions
pub fn create_test_node_with_schema_permissions(schema_names: &[&str]) -> DataFoldNode {
    let node = create_test_node();
    let node_id = node.get_node_id().to_string();
    let schema_strings: Vec<String> = schema_names.iter().map(|s| s.to_string()).collect();

    // Set schema permissions
    node.set_schema_permissions(&node_id, &schema_strings)
        .unwrap();

    node
}

/// Helper function to load and approve a schema for testing
pub fn load_and_approve_schema(
    node: &mut DataFoldNode,
    schema_path: &str,
    schema_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    node.load_schema_from_file(schema_path)?;
    node.approve_schema(schema_name)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fold_node::testing::Schema;

    #[test]
    fn test_create_test_node() {
        let mut node1 = create_test_node();
        let id1 = node1.get_node_id().to_string();
        let node2 = create_test_node();
        let id2 = node2.get_node_id().to_string();
        assert_ne!(id1, id2, "nodes should have unique ids");

        let schema = Schema::new("helper_test".to_string());
        assert!(node1.add_schema_available(schema).is_ok());
        assert!(node1.approve_schema("helper_test").is_ok());
        assert!(node1.get_schema("helper_test").unwrap().is_some());
    }
}
