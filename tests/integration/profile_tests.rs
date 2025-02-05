use fold_db::folddb::FoldDB;
use serde_json::json;

#[test]
fn test_profile_setup() {
    let mut db = FoldDB::new(&crate::get_test_db_path("profile")).unwrap();
    crate::data::profile::setup_profile_schema(&mut db).unwrap();

    // Test initial values
    let name = db.get_field_value("profile", "name").unwrap();
    let bio = db.get_field_value("profile", "bio").unwrap();
    
    assert_eq!(name, json!("John Doe"));
    assert_eq!(bio, json!("A software engineer"));
}

#[test]
fn test_profile_update() {
    let mut db = FoldDB::new(&crate::get_test_db_path("profile_update")).unwrap();
    crate::data::profile::setup_profile_schema(&mut db).unwrap();

    // Update values
    db.set_field_value(
        "profile",
        "name",
        json!("Jane Doe"),
        "test".to_string(),
    ).unwrap();

    db.set_field_value(
        "profile",
        "bio",
        json!("A data scientist"),
        "test".to_string(),
    ).unwrap();

    // Test updated values
    let name = db.get_field_value("profile", "name").unwrap();
    let bio = db.get_field_value("profile", "bio").unwrap();
    
    assert_eq!(name, json!("Jane Doe"));
    assert_eq!(bio, json!("A data scientist"));
}
