use serde_json::Value;
use fold_db::setup;
use super::super::data::posts;

#[tokio::test]
async fn test_posts() -> Result<(), Box<dyn std::error::Error>> {
    let fold_db = setup::initialize_database_with_path(":memory:posts")?;
    posts::create_test_posts(&fold_db)?;
    
    // Get all posts
    let mut posts = Vec::new();
    for i in 1..=10 {
        if let Ok(post) = fold_db.get_field_value("user_posts", &format!("post_{}", i)) {
            posts.push(post);
        }
    }
    
    assert!(!posts.is_empty());
    
    // Verify specific posts exist
    let post_titles: Vec<String> = posts.iter()
        .filter_map(|p| p.get("title").and_then(Value::as_str).map(String::from))
        .collect();
    
    assert!(post_titles.contains(&"Performance Optimization".to_string()));
    assert!(post_titles.contains(&"Testing Strategies".to_string()));

    // Verify posts are sorted by timestamp (latest first)
    let timestamps: Vec<i64> = posts.iter()
        .filter_map(|p| p.get("timestamp").and_then(Value::as_i64))
        .collect();
    
    // Check if timestamps are in descending order
    for i in 1..timestamps.len() {
        assert!(timestamps[i-1] >= timestamps[i]);
    }

    Ok(())
}
