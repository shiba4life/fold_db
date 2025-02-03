use std::collections::HashMap;
use serde_json::json;
use fold_db::schema::InternalSchema;
use fold_db::folddb::FoldDB;

pub fn create_test_posts(fold_db: &FoldDB) -> Result<Vec<String>, Box<dyn std::error::Error>> {
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

    let mut arefs = Vec::new();
    for (i, (title, content)) in posts.into_iter().enumerate() {
        let timestamp = (chrono::Utc::now() - chrono::Duration::hours(i as i64)).timestamp();
        let post = json!({
            "title": title,
            "content": content,
            "timestamp": timestamp
        });

        let atom = fold_db.create_atom(
            post.to_string(),
            "post".to_string(),
            "system_init".to_string(),
            None,
        )?;
        let aref = fold_db.create_atom_ref(&atom)?;
        arefs.push(aref);
    }

    // Create and load schema
    let mut posts_fields = HashMap::new();
    for (i, aref) in arefs.iter().enumerate() {
        posts_fields.insert(format!("post_{}", i + 1), aref.clone());
    }

    let mut schema = InternalSchema::new();
    schema.fields = posts_fields;
    fold_db.load_schema("user_posts", schema).map_err(|e| e.to_string())?;

    Ok(arefs)
}
