use std::time::Duration;
use datafold_sdk::{
    client::DataFoldClient,
    types::{NodeConnection, QueryFilter},
    mutation_builder::MutationType,
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("DataFold Social App - Mock Example");
    println!("==================================");

    // Create a client for the app with a mock connection
    let connection = NodeConnection::UnixSocket("mock".to_string());
    let client = DataFoldClient::with_connection(
        "social-app",
        "private-key-placeholder",
        "public-key-placeholder",
        connection,
    );

    println!("\n1. Client created for app: {}", client.get_app_id());

    // Discover available schemas
    println!("\n2. Discovering local schemas...");
    let schemas = client.discover_local_schemas().await?;
    println!("Available schemas: {:?}", schemas);

    // Create a test user
    println!("\n3. Creating a test user...");
    let user_id = uuid::Uuid::new_v4().to_string();
    let username = "alice";
    let full_name = "Alice Johnson";
    let bio = "Software engineer and blockchain enthusiast";

    let mutation_result = client.mutate("user")
        .operation(MutationType::Create)
        .set("id", json!(user_id))
        .set("username", json!(username))
        .set("full_name", json!(full_name))
        .set("bio", json!(bio))
        .set("created_at", json!(chrono::Utc::now().to_rfc3339()))
        .execute()
        .await?;

    println!("User creation result: success={}, id={:?}", mutation_result.success, mutation_result.id);

    // Query the user
    println!("\n4. Querying the user...");
    let query_result = client.query("user")
        .select(&["id", "username", "full_name", "bio"])
        .filter(QueryFilter::eq("username", json!(username)))
        .execute()
        .await?;

    println!("Query results: {:?}", query_result.results);

    // Create a post
    println!("\n5. Creating a post...");
    let post_id = uuid::Uuid::new_v4().to_string();
    let post_title = "Hello DataFold Network";
    let post_content = "This is my first post on the decentralized social network!";

    let mutation_result = client.mutate("post")
        .operation(MutationType::Create)
        .set("id", json!(post_id))
        .set("title", json!(post_title))
        .set("content", json!(post_content))
        .set("author_id", json!(user_id))
        .set("created_at", json!(chrono::Utc::now().to_rfc3339()))
        .execute()
        .await?;

    println!("Post creation result: success={}, id={:?}", mutation_result.success, mutation_result.id);

    // Query all posts
    println!("\n6. Querying all posts...");
    let query_result = client.query("post")
        .select(&["id", "title", "content", "author_id", "created_at"])
        .execute()
        .await?;

    println!("Posts: {:?}", query_result.results);

    // Add a comment to the post
    println!("\n7. Adding a comment to the post...");
    let comment_id = uuid::Uuid::new_v4().to_string();
    let comment_content = "Great post! Looking forward to more content.";

    let mutation_result = client.mutate("comment")
        .operation(MutationType::Create)
        .set("id", json!(comment_id))
        .set("content", json!(comment_content))
        .set("author_id", json!(user_id))
        .set("post_id", json!(post_id))
        .set("created_at", json!(chrono::Utc::now().to_rfc3339()))
        .execute()
        .await?;

    println!("Comment creation result: success={}, id={:?}", mutation_result.success, mutation_result.id);

    // Query comments for the post
    println!("\n8. Querying comments for the post...");
    let query_result = client.query("comment")
        .select(&["id", "content", "author_id", "post_id", "created_at"])
        .filter(QueryFilter::eq("post_id", json!(post_id)))
        .execute()
        .await?;

    println!("Comments: {:?}", query_result.results);

    // Discover remote nodes
    println!("\n9. Discovering remote nodes...");
    let nodes = client.discover_nodes().await?;
    println!("Discovered nodes: {:?}", nodes);

    // If there are remote nodes, query them
    if !nodes.is_empty() {
        let remote_node_id = &nodes[0].id;
        println!("\n10. Executing a query on remote node {}...", remote_node_id);
        
        let remote_query_result = client.query_on_node("user", remote_node_id)
            .select(&["id", "username", "full_name"])
            .execute()
            .await?;

        println!("Remote query results: {:?}", remote_query_result.results);
    } else {
        println!("\n10. No remote nodes found. Skipping remote query.");
    }

    println!("\nSocial App Example completed successfully!");
    
    Ok(())
}
