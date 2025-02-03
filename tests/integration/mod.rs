pub mod profile_tests;
pub mod posts_tests;
pub mod api_tests;

use fold_db::setup;
use std::time::{SystemTime, UNIX_EPOCH};
use super::data::{profile, posts};

pub async fn run_example_tests() -> Result<(), Box<dyn std::error::Error>> {
    // Use a unique path based on timestamp to avoid conflicts
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Create tmp directory if it doesn't exist
    std::fs::create_dir_all("tmp")?;
    let db_path = format!("tmp/fold_db_{}", timestamp);
    
    // Initialize the database
    let fold_db = setup::initialize_database_with_path(&db_path)?;
    
    // Create test data
    profile::create_test_profile(&fold_db)?;
    posts::create_test_posts(&fold_db)?;

    println!("Running example operations...\n");

    // Print some example data
    let username = fold_db.get_field_value("user_profile", "username")?;
    let bio = fold_db.get_field_value("user_profile", "bio")?;
    
    println!("Current user profile:");
    println!("Username: {}", username);
    println!("Bio: {}", bio);

    println!("\nDatabase initialized successfully at: {}", db_path);

    Ok(())
}
