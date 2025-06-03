use fold_node::db_operations::core::DbOperations;
use fold_node::schema::SchemaCore;
use std::sync::Arc;
use tempfile::NamedTempFile;

fn create_test_db_ops() -> Arc<DbOperations> {
    let db = sled::Config::new().temporary(true).open().unwrap();
    Arc::new(DbOperations::new(db).unwrap())
}

#[test]
fn test_new_invalid_path() {
    let file = NamedTempFile::new().unwrap();
    let db_ops = create_test_db_ops();
    let result = SchemaCore::new(file.path().to_str().unwrap(), db_ops);
    assert!(result.is_err());
}
