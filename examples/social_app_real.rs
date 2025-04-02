use std::time::Duration;
use datafold_sdk::{
    client::DataFoldClient,
    types::{NodeConnection, QueryFilter},
    mutation_builder::MutationType,
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("DataFold Social App - Real Node Example");
    println!("=======================================");

    // Check if a DataFold node is already running
    println!("\nMake sure you have a DataFold node running!");
    println!("You can start one with: cargo run --bin datafold_node -- --port 9000");
    println!("Waiting 3 seconds before continuing...");
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Create a client for the app with a mock connection
    let connection = NodeConnection::UnixSocket("mock://localhost:9000".to_string());
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

    // Create schemas if they don't exist
    if !schemas.contains(&"user".to_string()) {
        println!("\n3. Creating user schema...");
        let user_schema = json!({
            "name": "user",
            "fields": [
                { "name": "id", "field_type": "string", "required": true },
                { "name": "username", "field_type": "string", "required": true },
                { "name": "full_name", "field_type": "string", "required": false },
                { "name": "bio", "field_type": "string", "required": false },
                { "name": "created_at", "field_type": "string", "required": true }
            ]
        });

        let request = client.send_request(
            datafold_sdk::types::AppRequest::new(
                client.get_app_id(),
                None,
                "create_schema",
                json!({ "schema": user_schema }),
                "private-key-placeholder",
            )
        ).await?;

        println!("User schema creation result: {:?}", request);
    }

    if !schemas.contains(&"post".to_string()) {
        println!("\n4. Creating post schema...");
        let post_schema = json!({
            "name": "post",
            "fields": [
                { "name": "id", "field_type": "string", "required": true },
                { "name": "title", "field_type": "string", "required": true },
                { "name": "content", "field_type": "string", "required": true },
                { "name": "author_id", "field_type": "string", "required": true },
                { "name": "created_at", "field_type": "string", "required": true }
            ]
        });

        let request = client.send_request(
            datafold_sdk::types::AppRequest::new(
                client.get_app_id(),
                None,
                "create_schema",
                json!({ "schema": post_schema }),
                "private-key-placeholder",
            )
        ).await?;

        println!("Post schema creation result: {:?}", request);
    }

    if !schemas.contains(&"comment".to_string()) {
        println!("\n5. Creating comment schema...");
        let comment_schema = json!({
            "name": "comment",
            "fields": [
                { "name": "id", "field_type": "string", "required": true },
                { "name": "content", "field_type": "string", "required": true },
                { "name": "author_id", "field_type": "string", "required": true },
                { "name": "post_id", "field_type": "string", "required": true },
                { "name": "created_at", "field_type": "string", "required": true }
            ]
        });

        let request = client.send_request(
            datafold_sdk::types::AppRequest::new(
                client.get_app_id(),
                None,
                "create_schema",
                json!({ "schema": comment_schema }),
                "private-key-placeholder",
            )
        ).await?;

        println!("Comment schema creation result: {:?}", request);
    }

    // Create a test user
    println!("\n6. Creating a test user...");
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
    println!("\n7. Querying the user...");
    let query_result = client.query("user")
        .select(&["id", "username", "full_name", "bio"])
        .filter(QueryFilter::eq("username", json!(username)))
        .execute()
        .await?;

    println!("Query results: {:?}", query_result.results);

    // Create a post
    println!("\n8. Creating a post...");
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
    println!("\n9. Querying all posts...");
    let query_result = client.query("post")
        .select(&["id", "title", "content", "author_id", "created_at"])
        .execute()
        .await?;

    println!("Posts: {:?}", query_result.results);

    // Add a comment to the post
    println!("\n10. Adding a comment to the post...");
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
    println!("\n11. Querying comments for the post...");
    let query_result = client.query("comment")
        .select(&["id", "content", "author_id", "post_id", "created_at"])
        .filter(QueryFilter::eq("post_id", json!(post_id)))
        .execute()
        .await?;

    println!("Comments: {:?}", query_result.results);

    // Create another user
    println!("\n12. Creating another user...");
    let user2_id = uuid::Uuid::new_v4().to_string();
    let username2 = "bob";
    let full_name2 = "Bob Smith";
    let bio2 = "Blockchain developer and crypto enthusiast";

    let mutation_result = client.mutate("user")
        .operation(MutationType::Create)
        .set("id", json!(user2_id))
        .set("username", json!(username2))
        .set("full_name", json!(full_name2))
        .set("bio", json!(bio2))
        .set("created_at", json!(chrono::Utc::now().to_rfc3339()))
        .execute()
        .await?;

    println!("User creation result: success={}, id={:?}", mutation_result.success, mutation_result.id);

    // Create a post from the second user
    println!("\n13. Creating a post from the second user...");
    let post2_id = uuid::Uuid::new_v4().to_string();
    let post2_title = "Decentralized Social Networks";
    let post2_content = "Decentralized social networks are the future of online communication!";

    let mutation_result = client.mutate("post")
        .operation(MutationType::Create)
        .set("id", json!(post2_id))
        .set("title", json!(post2_title))
        .set("content", json!(post2_content))
        .set("author_id", json!(user2_id))
        .set("created_at", json!(chrono::Utc::now().to_rfc3339()))
        .execute()
        .await?;

    println!("Post creation result: success={}, id={:?}", mutation_result.success, mutation_result.id);

    // Add a comment from the first user to the second user's post
    println!("\n14. Adding a comment from the first user to the second user's post...");
    let comment2_id = uuid::Uuid::new_v4().to_string();
    let comment2_content = "I completely agree! Decentralization is key to user privacy.";

    let mutation_result = client.mutate("comment")
        .operation(MutationType::Create)
        .set("id", json!(comment2_id))
        .set("content", json!(comment2_content))
        .set("author_id", json!(user_id))
        .set("post_id", json!(post2_id))
        .set("created_at", json!(chrono::Utc::now().to_rfc3339()))
        .execute()
        .await?;

    println!("Comment creation result: success={}, id={:?}", mutation_result.success, mutation_result.id);

    // Query all users
    println!("\n15. Querying all users...");
    let query_result = client.query("user")
        .select(&["id", "username", "full_name", "bio", "created_at"])
        .execute()
        .await?;

    println!("Users: {:?}", query_result.results);

    // Discover remote nodes
    println!("\n16. Discovering remote nodes...");
    let nodes = client.discover_nodes().await?;
    println!("Discovered nodes: {:?}", nodes);

    // If there are remote nodes, query them
    if !nodes.is_empty() {
        let remote_node_id = &nodes[0].id;
        println!("\n17. Executing a query on remote node {}...", remote_node_id);
        
        let remote_query_result = client.query_on_node("user", remote_node_id)
            .select(&["id", "username", "full_name"])
            .execute()
            .await?;

        println!("Remote query results: {:?}", remote_query_result.results);
    } else {
        println!("\n17. No remote nodes found. Skipping remote query.");
        println!("To test with multiple nodes, run the examples/run_network_nodes.sh script.");
    }

    println!("\nSocial App Example completed successfully!");
    
    Ok(())
}
