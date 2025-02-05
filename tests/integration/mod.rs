pub mod schema_tests;
pub mod mapper_tests;
pub mod permissions_tests;
pub mod posts_tests;
pub mod profile_tests;

use std::sync::Arc;
use fold_db::folddb::FoldDB;
use serde_json::json;

pub fn setup_test_db() -> Arc<FoldDB> {
    let db = FoldDB::new(&crate::get_test_db_path("db")).unwrap();
    Arc::new(db)
}

pub fn setup_test_db_with_data() -> Arc<FoldDB> {
    let mut db = FoldDB::new(&crate::get_test_db_path("db_with_data")).unwrap();
    
    // Setup test data
    crate::data::profile::setup_profile_schema(&mut db).unwrap();
    crate::data::posts::create_test_posts(&mut db).unwrap();
    
    Arc::new(db)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_setup() {
        let db = setup_test_db();
        assert!(db.get_field_value("profile", "name").is_err());
    }

    #[test]
    fn test_db_with_data() {
        let db = setup_test_db_with_data();
        let name = db.get_field_value("profile", "name").unwrap();
        let bio = db.get_field_value("profile", "bio").unwrap();
        
        assert_eq!(name, json!("John Doe"));
        assert_eq!(bio, json!("A software engineer"));
    }
}
