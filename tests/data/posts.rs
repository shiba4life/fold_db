use serde_json::{json, Value};
use fold_db::schema::types::{Schema, FieldType};
use fold_db::folddb::FoldDB;
use uuid::Uuid;

pub fn create_test_posts(fold_db: &mut FoldDB) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let posts = vec![
        ("Performance Optimization", "Some tips for optimizing Rust code without sacrificing safety..."),
        ("Testing Strategies", "Here's how I structure tests in my Rust projects..."),
        ("Database Design Patterns", "Implementing an atom-based database for versioning..."),
        ("Web Development with Rust", "Comparing different web frameworks in the Rust ecosystem..."),
        ("Building CLI Tools", "Created my first CLI tool in Rust using clap..."),
        ("Error Handling Best Practices", "Today I learned about better error handling patterns in Rust..."),
        ("Async Programming", "Exploring async/await in Rust and how it compares to other languages..."),
        ("Immutable Data Structures", "Here's why immutable data structures are great for version control..."),
        ("API Design", "Exploring different API design patterns in Rust..."),
        ("Exploring Rust's Memory Safety", "Just discovered how Rust's ownership system prevents common bugs...")
    ];

    // Create schema
    let mut schema = Schema::new("user_posts".to_string());
    
    // Add posts field with default permissions
    let posts_field = fold_db::schema::types::SchemaField::new(
        "W1".to_string(),
        Uuid::new_v4().to_string(),
        FieldType::Collection,
    );
    schema.add_field("posts".to_string(), posts_field);

    // Create JSON posts
    let mut json_posts = Vec::new();
    for (i, (title, content)) in posts.into_iter().enumerate() {
        let timestamp = (chrono::Utc::now() - chrono::Duration::hours(i as i64)).timestamp();
        let post = json!({
            "title": title,
            "content": content,
            "timestamp": timestamp
        });
        json_posts.push(post);
    }

    // Load schema and set field value
    fold_db.load_schema(schema)?;
    fold_db.set_field_value("user_posts", "posts", json!(json_posts), "system_init".to_string())?;
    
    Ok(json_posts)
}
