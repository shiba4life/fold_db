use datafold_sdk::{
    DataFoldClient,
    types::NodeConnection,
};
use serde_json::json;
use uuid::Uuid;

use crate::Logger;
use crate::node_setup::TestNode;

/// User data for a test node
pub struct UserData {
    pub id: String,
    pub username: String,
    pub full_name: String,
    pub bio: String,
    pub email: String,
}

/// Post data for a test node
pub struct PostData {
    pub id: String,
    pub title: String,
    pub content: String,
    pub author_id: String,
}

/// Add test data to both nodes
pub async fn add_test_data(_node1: &TestNode, _node2: &TestNode, logger: &mut Logger) -> Result<(UserData, PostData, UserData, PostData), Box<dyn std::error::Error>> {
    logger.log("\nAdding different data to each node...");
    
    // Create clients for both nodes
    let client1 = create_client_for_node(1, 8001);
    let client2 = create_client_for_node(2, 8002);
    
    // Add data to Node 1
    let (alice, alice_post) = add_data_to_node(
        &client1, 
        "alice", 
        "Alice Johnson", 
        "Node 1 user", 
        "node1_user@example.com",
        "Hello from Node 1",
        "This post is stored on Node 1",
        1,
        logger
    ).await?;
    
    // Add data to Node 2
    let (bob, bob_post) = add_data_to_node(
        &client2, 
        "bob", 
        "Bob Smith", 
        "Node 2 user", 
        "node2_user@example.com",
        "Hello from Node 2",
        "This post is stored on Node 2",
        2,
        logger
    ).await?;
    
    logger.log("Data added to both nodes");
    
    Ok((alice, alice_post, bob, bob_post))
}

/// Add user and post data to a node
async fn add_data_to_node(
    client: &DataFoldClient,
    username: &str,
    full_name: &str,
    bio: &str,
    email: &str,
    post_title: &str,
    post_content: &str,
    node_number: u8,
    logger: &mut Logger
) -> Result<(UserData, PostData), Box<dyn std::error::Error>> {
    // Create user data
    let user_id = Uuid::new_v4().to_string();
    logger.log(&format!("Adding user to Node {}: {}", node_number, user_id));
    
    let user_data = UserData {
        id: user_id.clone(),
        username: username.to_string(),
        full_name: full_name.to_string(),
        bio: bio.to_string(),
        email: email.to_string(),
    };
    
    let user_mutation = client.send_request(
        datafold_sdk::types::AppRequest::new(
            client.get_app_id(),
            None,
            "mutation",
            json!({
                "schema": "user",
                "mutation_type": "create",
                "data": {
                    "id": user_data.id.clone(),
                    "username": user_data.username,
                    "full_name": user_data.full_name,
                    "bio": user_data.bio,
                    "email": user_data.email
                }
            }),
            "private-key-placeholder",
        )
    ).await;
    
    match user_mutation {
        Ok(_) => logger.log(&format!("User added to Node {}", node_number)),
        Err(e) => logger.log(&format!("Error adding user to Node {}: {}", node_number, e)),
    }
    
    // Create post data
    let post_id = Uuid::new_v4().to_string();
    logger.log(&format!("Adding post to Node {}: {}", node_number, post_id));
    
    let post_data = PostData {
        id: post_id.clone(),
        title: post_title.to_string(),
        content: post_content.to_string(),
        author_id: user_id.clone(),
    };
    
    let post_mutation = client.send_request(
        datafold_sdk::types::AppRequest::new(
            client.get_app_id(),
            None,
            "mutation",
            json!({
                "schema": "post",
                "mutation_type": "create",
                "data": {
                    "id": post_data.id.clone(),
                    "title": post_data.title,
                    "content": post_data.content,
                    "author_id": post_data.author_id
                }
            }),
            "private-key-placeholder",
        )
    ).await;
    
    match post_mutation {
        Ok(_) => logger.log(&format!("Post added to Node {}", node_number)),
        Err(e) => logger.log(&format!("Error adding post to Node {}: {}", node_number, e)),
    }
    
    Ok((user_data, post_data))
}

/// Create a client for a node
fn create_client_for_node(_node_number: u8, port: u16) -> DataFoldClient {
    let connection = NodeConnection::TcpSocket("127.0.0.1".to_string(), port);
    DataFoldClient::with_connection(
        "schema-creator",
        "private-key-placeholder",
        "public-key-placeholder",
        connection,
    )
}
