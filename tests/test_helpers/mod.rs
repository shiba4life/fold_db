pub mod schema_builder;
pub mod operation_builder;

use std::path::PathBuf;
use uuid::Uuid;
use fold_db::FoldDB;

pub fn get_test_db_path() -> String {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let tmp_dir = current_dir.join("tmp");
    
    // Create tmp directory and ensure it exists
    std::fs::create_dir_all(&tmp_dir).expect("Failed to create tmp directory");
    
    let db_path = tmp_dir.join(format!("fold_db_test_{}", Uuid::new_v4()));
    
    // Create the database directory
    std::fs::create_dir_all(&db_path).expect("Failed to create database directory");
    
    db_path.to_string_lossy().into_owned()
}

pub fn cleanup_test_db(path: &str) {
    let path = std::path::Path::new(path);
    if path.exists() {
        for _ in 0..3 {  // Try up to 3 times
            if std::fs::remove_dir_all(path).is_ok() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

pub fn setup_test_db() -> (FoldDB, String) {
    let db_path = get_test_db_path();
    let db = FoldDB::new(&db_path).expect("Failed to create test database");
    (db, db_path)
}

pub fn setup_and_allow_schema(db: &mut FoldDB, schema_name: &str) -> Result<(), fold_db::schema::SchemaError> {
    db.allow_schema(schema_name)
}
