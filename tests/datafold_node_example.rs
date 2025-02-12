use fold_db::fees::types::{FieldPaymentConfig, TrustDistanceScaling};
use fold_db::permissions::types::policy::{PermissionsPolicy, TrustDistance};
use fold_db::schema::types::fields::SchemaField;
use fold_db::schema::types::{Mutation, Query};
use fold_db::schema::{Schema, SchemaError};
use fold_db::{DataFoldNode, NodeConfig};
use serde_json::json;
use tempfile::tempdir;
use uuid;

#[test]
fn test_complete_node_workflow() {
    // 1. Initialize the node
    let dir = tempdir().unwrap();
    let config = NodeConfig {
        storage_path: dir.path().to_path_buf(),
        default_trust_distance: 1,
    };
    let mut node = DataFoldNode::new(config).unwrap();
    println!("✓ Node initialized successfully");

    // 2. Create and load a schema
    let mut schema = Schema::new("blog_post".to_string());

    // Add title field
    let title_field = SchemaField {
        ref_atom_uuid: uuid::Uuid::new_v4().to_string(),
        permission_policy: PermissionsPolicy::new(
            TrustDistance::Distance(1), // Anyone with trust distance 1 can read
            TrustDistance::Distance(1), // Only trusted users can write
        ),
        payment_config: FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
    };
    schema.add_field("title".to_string(), title_field);

    // Add content field
    let content_field = SchemaField {
        ref_atom_uuid: uuid::Uuid::new_v4().to_string(),
        permission_policy: PermissionsPolicy::new(
            TrustDistance::Distance(1),
            TrustDistance::Distance(1),
        ),
        payment_config: FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
    };
    schema.add_field("content".to_string(), content_field);

    // Load schema
    node.load_schema(schema).unwrap();
    node.allow_schema("blog_post").unwrap();
    println!("✓ Schema loaded successfully");

    // 3. Create a blog post
    let create_post = Mutation {
        schema_name: "blog_post".to_string(),
        pub_key: "author_key".to_string(),
        trust_distance: 1,
        fields_and_values: vec![
            ("title".to_string(), json!("My First Blog Post")),
            (
                "content".to_string(),
                json!("This is the content of my first blog post."),
            ),
        ]
        .into_iter()
        .collect(),
    };

    node.mutate(create_post).unwrap();
    println!("✓ Blog post created successfully");

    // 4. Query the blog post
    let query = Query {
        schema_name: "blog_post".to_string(),
        pub_key: "reader_key".to_string(),
        fields: vec!["title".to_string(), "content".to_string()],
        trust_distance: 1,
    };

    let results = node.query(query).unwrap();
    println!("✓ Blog post queried successfully");

    // 5. Verify the results
    assert_eq!(results.len(), 2);
    for result in results {
        let value = result.unwrap();
        match value.as_str() {
            Some("My First Blog Post") => println!("✓ Title verified"),
            Some("This is the content of my first blog post.") => println!("✓ Content verified"),
            _ => panic!("Unexpected value: {}", value),
        }
    }

    // 6. Update the blog post
    let update_post = Mutation {
        schema_name: "blog_post".to_string(),
        pub_key: "author_key".to_string(),
        trust_distance: 1,
        fields_and_values: vec![(
            "content".to_string(),
            json!("This is the updated content of my first blog post."),
        )]
        .into_iter()
        .collect(),
    };

    node.mutate(update_post).unwrap();
    println!("✓ Blog post updated successfully");

    // 7. Query the updated content
    let query = Query {
        schema_name: "blog_post".to_string(),
        pub_key: "reader_key".to_string(),
        fields: vec!["content".to_string()],
        trust_distance: 1,
    };

    let results = node.query(query).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].as_ref().unwrap().as_str().unwrap(),
        "This is the updated content of my first blog post."
    );
    println!("✓ Updated content verified");

    // 8. Test permission handling
    let unauthorized_query = Query {
        schema_name: "blog_post".to_string(),
        pub_key: "untrusted_key".to_string(),
        fields: vec!["content".to_string()],
        trust_distance: 2, // Higher trust distance (less trusted)
    };

    let results = node.query(unauthorized_query).unwrap();
    assert!(results[0].is_err()); // Should fail due to insufficient trust
    println!("✓ Permission handling verified");

    println!("\n✓ Complete node workflow test passed successfully!");
}
