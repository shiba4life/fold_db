use std::sync::Arc;
use fold_db::folddb::FoldDB;
use serde_json::{json, Value};

#[test]
fn test_posts_setup() {
    let mut db = FoldDB::new(&crate::get_test_db_path("posts")).unwrap();
    let posts = crate::data::posts::create_test_posts(&mut db).unwrap();

    // Test posts were created
    let stored_posts = db.get_field_value("user_posts", "posts").unwrap();
    let stored_posts = stored_posts.as_array().unwrap();

    assert_eq!(stored_posts.len(), posts.len());
    
    // Test post content
    for (stored, original) in stored_posts.iter().zip(posts.iter()) {
        assert_eq!(stored["title"], original["title"]);
        assert_eq!(stored["content"], original["content"]);
    }
}

#[test]
fn test_posts_update() {
    let mut db = FoldDB::new(&crate::get_test_db_path("posts_update")).unwrap();
    crate::data::posts::create_test_posts(&mut db).unwrap();

    // Create new post
    let new_post = json!({
        "title": "New Post",
        "content": "This is a new post",
        "timestamp": chrono::Utc::now().timestamp()
    });

    // Get current posts and add new one
    let mut current_posts = db.get_field_value("user_posts", "posts").unwrap();
    let mut posts_array = current_posts.as_array().unwrap().clone();
    posts_array.push(new_post.clone());

    // Update posts
    db.set_field_value(
        "user_posts",
        "posts",
        json!(posts_array),
        "test".to_string(),
    ).unwrap();

    // Test updated posts
    let stored_posts = db.get_field_value("user_posts", "posts").unwrap();
    let stored_posts = stored_posts.as_array().unwrap();

    assert_eq!(stored_posts.last().unwrap()["title"], new_post["title"]);
    assert_eq!(stored_posts.last().unwrap()["content"], new_post["content"]);
}
