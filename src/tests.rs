use crate::graphql::{build_schema, FIELD_QUERY, FIELD_HISTORY_QUERY, UPDATE_FIELDS_MUTATION};
use crate::setup;

use std::time::{SystemTime, UNIX_EPOCH};

pub async fn run_example_tests() -> Result<(), Box<dyn std::error::Error>> {
    // Use a unique path based on timestamp to avoid conflicts
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Create tmp directory if it doesn't exist
    std::fs::create_dir_all("tmp")?;
    let db_path = format!("tmp/fold_db_{}", timestamp);
    
    // Initialize the database with example data
    let fold_db = setup::initialize_database_with_path(&db_path)?;

    // Build the GraphQL schema
    let schema = build_schema(fold_db);

    println!("Executing example queries...\n");

    // Query initial state
    let response = schema.execute(FIELD_QUERY).await;
    println!("Query response:\n{}", serde_json::to_string_pretty(&response.data)?);

    // Query history
    let history_response = schema.execute(FIELD_HISTORY_QUERY).await;
    println!("\nHistory response:\n{}", serde_json::to_string_pretty(&history_response.data)?);

    // Update fields
    let mutation_response = schema.execute(UPDATE_FIELDS_MUTATION).await;
    println!("\nMutation response:\n{}", serde_json::to_string_pretty(&mutation_response.data)?);

    // Verify updates
    println!("\nVerifying updates with another query:");
    let verify_response = schema.execute(FIELD_QUERY).await;
    println!("{}", serde_json::to_string_pretty(&verify_response.data)?);

    // Verify history after updates
    println!("\nVerifying history after updates:");
    let verify_history = schema.execute(FIELD_HISTORY_QUERY).await;
    println!("{}", serde_json::to_string_pretty(&verify_history.data)?);

    Ok(())
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_field_operations() -> Result<(), Box<dyn std::error::Error>> {
        let fold_db = setup::initialize_database_with_path(":memory:")?;
        let schema = build_schema(fold_db);

        // Test initial field values
        let response = schema.execute(FIELD_QUERY).await;
        let json = serde_json::to_string_pretty(&response.data)?;
        assert!(json.contains("john_doe"));
        assert!(json.contains("Software engineer and Rust enthusiast"));

        // Test field updates
        let mutation_response = schema.execute(UPDATE_FIELDS_MUTATION).await;
        let mutation_json = mutation_response.data.to_string();
        assert!(mutation_json.contains("true"));
        assert!(mutation_json.contains("true")); // Both updates should succeed

        // Verify updates
        let verify_response = schema.execute(FIELD_QUERY).await;
        let json = serde_json::to_string_pretty(&verify_response.data)?;
        assert!(json.contains("new_username"));
        assert!(json.contains("Full-stack developer with a passion for Rust"));

        // Verify history
        let history_response = schema.execute(FIELD_HISTORY_QUERY).await;
        let json = serde_json::to_string_pretty(&history_response.data)?;
        // Username history
        assert!(json.contains("new_username"));
        assert!(json.contains("john_doe"));
        // Bio history
        assert!(json.contains("Full-stack developer with a passion for Rust"));
        assert!(json.contains("Software engineer and Rust enthusiast"));

        Ok(())
    }
}
