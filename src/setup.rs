use std::sync::Arc;
use crate::folddb::FoldDB;

pub fn initialize_database_with_path(path: &str) -> Result<Arc<FoldDB>, Box<dyn std::error::Error>> {
    let fold_db = FoldDB::new(path)?;
    Ok(Arc::new(fold_db))
}
