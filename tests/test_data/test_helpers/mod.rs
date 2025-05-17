pub mod operation_builder;
pub mod schema_builder;

// Re-export testing utilities for all tests
use fold_node::FoldDB;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

#[allow(dead_code)]
static CLEANUP_LOCK: Mutex<()> = Mutex::new(());

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn cleanup_tmp_dir() {
    let _lock = CLEANUP_LOCK.lock().unwrap();
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let tmp_dir = current_dir.join("tmp");

    // First ensure the directory exists
    let _ = fs::create_dir_all(&tmp_dir);

    // Remove all contents with retries
    let cleanup_contents = || -> std::io::Result<()> {
        if let Ok(entries) = fs::read_dir(&tmp_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let _ = fs::remove_dir_all(entry.path());
                }
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
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Directory not empty",
                ))
            }
        } else {
            Ok(())
        }
    };

    if let Err(e) = retry_with_backoff(verify_empty, 5) {
        eprintln!("Warning: Directory may not be empty: {}", e);
    }
}

#[allow(dead_code)]
pub fn setup_test_db() -> (FoldDB, String) {
    let db_path = get_test_db_path();
    let db = FoldDB::new(&db_path).expect("Failed to create test database");
    (db, db_path)
}

#[allow(dead_code)]
pub fn setup_and_allow_schema(
    db: &mut FoldDB,
    schema_name: &str,
) -> Result<(), fold_node::testing::SchemaError> {
    db.allow_schema(schema_name)
}
