use serde_json::Value;
use fold_db::setup;
use super::super::data::profile;

#[tokio::test]
async fn test_profile_operations() -> Result<(), Box<dyn std::error::Error>> {
    let fold_db = setup::initialize_database_with_path(":memory:profile")?;
    profile::create_test_profile(&fold_db)?;

    // Test initial profile
    let username = fold_db.get_field_value("user_profile", "username")?;
    let bio = fold_db.get_field_value("user_profile", "bio")?;
    
    assert_eq!(username.as_str().unwrap(), "john_doe");
    assert_eq!(bio.as_str().unwrap(), "Software engineer and Rust enthusiast");

    // Test profile updates
    fold_db.update_field_value(
        "user_profile",
        "username",
        Value::String("new_username".to_string()),
        "test".to_string(),
    )?;
    
    fold_db.update_field_value(
        "user_profile",
        "bio",
        Value::String("Full-stack developer with a passion for Rust".to_string()),
        "test".to_string(),
    )?;

    // Verify updates
    let updated_username = fold_db.get_field_value("user_profile", "username")?;
    let updated_bio = fold_db.get_field_value("user_profile", "bio")?;
    
    assert_eq!(updated_username.as_str().unwrap(), "new_username");
    assert_eq!(updated_bio.as_str().unwrap(), "Full-stack developer with a passion for Rust");

    Ok(())
}
