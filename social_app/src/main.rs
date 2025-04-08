use serde_json::{json, Value};
use log::info;
use uuid::Uuid;
use chrono::Utc;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// Simple TCP client for connecting directly to the DataFold node
struct NodeClient {
    stream: TcpStream,
}

impl NodeClient {
    // Connect to the DataFold node
    async fn connect(host: &str, port: u16) -> Result<Self> {
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect(&addr).await?;
        Ok(Self { stream })
    }
    
    // Send a request to the DataFold node
    async fn send_request(&mut self, operation: &str, params: Value) -> Result<Value> {
        // Create the request
        let request = json!({
            "operation": operation,
            "params": params
        });
        
        // Serialize the request
        let request_bytes = serde_json::to_vec(&request)?;
        
        // Try to send the request and read the response, with reconnection if needed
        let max_retries = 3;
        for attempt in 0..max_retries {
            if attempt > 0 {
                info!("Retrying request (attempt {}/{})", attempt + 1, max_retries);
                // Reconnect if this is a retry
                let host = "127.0.0.1";
                let port = 9000;
                match TcpStream::connect(format!("{}:{}", host, port)).await {
                    Ok(stream) => {
                        self.stream = stream;
                        info!("Reconnected to DataFold Node");
                    },
                    Err(e) => {
                        info!("Failed to reconnect: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        continue;
                    }
                }
            }
            
            // Send the request length
            if let Err(e) = self.stream.write_u32(request_bytes.len() as u32).await {
                info!("Error sending request length: {}", e);
                continue;
            }
            
            // Send the request
            if let Err(e) = self.stream.write_all(&request_bytes).await {
                info!("Error sending request: {}", e);
                continue;
            }
            
            // Read the response length
            let response_len = match self.stream.read_u32().await {
                Ok(len) => len as usize,
                Err(e) => {
                    info!("Error reading response length: {}", e);
                    continue;
                }
            };
            
            // Read the response
            let mut response_bytes = vec![0u8; response_len];
            if let Err(e) = self.stream.read_exact(&mut response_bytes).await {
                info!("Error reading response: {}", e);
                continue;
            }
            
            // Deserialize the response
            match serde_json::from_slice(&response_bytes) {
                Ok(response) => return Ok(response),
                Err(e) => {
                    info!("Error deserializing response: {}", e);
                    continue;
                }
            }
        }
        
        // If we get here, all retries failed
        Err(format!("Failed to send request after {} retries", max_retries).into())
    }
    
    // List available schemas
    async fn list_schemas(&mut self) -> Result<Vec<String>> {
        let result = self.send_request("list_schemas", Value::Null).await?;
        let schemas = result
            .as_array()
            .ok_or_else(|| "Invalid response format".to_string())?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        Ok(schemas)
    }
    
    // Query data from a schema
    async fn query(
        &mut self,
        schema: &str,
        fields: &[&str],
        filter: Option<Value>,
    ) -> Result<Vec<Value>> {
        let params = json!({
            "schema": schema,
            "fields": fields,
            "filter": filter,
        });
        
        let result = self.send_request("query", params).await?;
        
        // Handle the case where the result might be directly an array
        if let Some(results) = result.as_array() {
            return Ok(results.clone());
        }
        
        // Otherwise, try to extract the results field
        let results = result
            .get("results")
            .ok_or_else(|| "Invalid response format".to_string())?
            .as_array()
            .ok_or_else(|| "Invalid response format".to_string())?
            .clone();
        Ok(results)
    }
    
    // Create data in a schema
    async fn create(&mut self, schema: &str, data: Value) -> Result<String> {
        let params = json!({
            "schema": schema,
            "mutation_type": "create",
            "data": data,
        });
        
        let result = self.send_request("mutation", params).await?;
        
        // Handle the case where the result might be directly a success value
        if let Some(success) = result.get("success") {
            if success.as_bool() == Some(true) {
                if let Some(id) = result.get("id") {
                    if let Some(id_str) = id.as_str() {
                        return Ok(id_str.to_string());
                    }
                }
                // If we can't get the ID, return a placeholder
                return Ok("success".to_string());
            }
        }
        
        // If we get here, something went wrong
        Err(format!("Failed to create data: {:?}", result).into())
    }
    
    // Discover remote nodes
    async fn discover_nodes(&mut self) -> Result<Vec<Value>> {
        let result = self.send_request("discover_nodes", Value::Null).await?;
        
        // Handle the case where the result might be directly an array
        if let Some(nodes) = result.as_array() {
            return Ok(nodes.clone());
        }
        
        // If we get here, something went wrong
        Err("Invalid response format for discover_nodes".into())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    info!("Starting Social App with direct connection to DataFold Node");

    // Connect directly to the DataFold node
    let host = "127.0.0.1";
    let port = 9000;
    info!("Connecting to DataFold Node at {}:{}", host, port);
    let mut client = NodeClient::connect(host, port).await?;
    info!("Connected to DataFold Node successfully");

    // List available schemas
    info!("Listing available schemas...");
    let schemas = client.list_schemas().await?;
    info!("Available schemas: {:?}", schemas);

    // Create schemas if they don't exist
    if !schemas.contains(&"user".to_string()) {
        create_user_schema(&mut client).await?;
    }

    if !schemas.contains(&"post".to_string()) {
        create_post_schema(&mut client).await?;
    }

    if !schemas.contains(&"comment".to_string()) {
        create_comment_schema(&mut client).await?;
    }

    // Create a test user
    info!("Creating a test user...");
    let user_id = Uuid::new_v4().to_string();
    let username = "alice";
    let full_name = "Alice Johnson";
    let bio = "Software engineer and blockchain enthusiast";

    let user_data = json!({
        "id": user_id,
        "username": username,
        "full_name": full_name,
        "bio": bio,
        "created_at": Utc::now().to_rfc3339()
    });

    match client.create("user", user_data).await {
        Ok(id) => info!("User created with ID: {}", id),
        Err(e) => {
            if e.to_string().contains("already exists") {
                info!("User already exists, continuing...");
            } else {
                return Err(format!("Failed to create user: {}", e).into());
            }
        }
    }

    // Query the user
    info!("Querying the user...");
    let filter = Some(json!({
        "field": "username",
        "operator": "eq",
        "value": username
    }));
    let users = client.query("user", &["id", "username", "full_name", "bio"], filter).await?;
    
    if users.is_empty() {
        return Err("User not found".into());
    }
    
    let user = &users[0];
    info!("Found user: {}", user);
    
    // Use the user ID from the query result
    let user_id = user["id"].as_str().unwrap_or(&user_id).to_string();

    // Create a post
    info!("Creating a post...");
    let post_id = Uuid::new_v4().to_string();
    let post_title = "Hello DataFold Network";
    let post_content = "This is my first post on the decentralized social network!";

    let post_data = json!({
        "id": post_id,
        "title": post_title,
        "content": post_content,
        "author_id": user_id,
        "created_at": Utc::now().to_rfc3339()
    });

    let post_result = client.create("post", post_data).await?;
    info!("Post created with ID: {}", post_result);

    // Query all posts
    info!("Querying all posts...");
    let posts = client.query("post", &["id", "title", "content", "author_id", "created_at"], None).await?;
    info!("Found {} posts", posts.len());
    
    for (i, post) in posts.iter().enumerate() {
        info!("Post {}: {}", i + 1, post);
    }

    // Add a comment to the post
    info!("Adding a comment to the post...");
    let comment_id = Uuid::new_v4().to_string();
    let comment_content = "Great post! Looking forward to more content.";

    let comment_data = json!({
        "id": comment_id,
        "content": comment_content,
        "author_id": user_id,
        "post_id": post_id,
        "created_at": Utc::now().to_rfc3339()
    });

    let comment_result = client.create("comment", comment_data).await?;
    info!("Comment created with ID: {}", comment_result);

    // Query comments for the post
    info!("Querying comments for the post...");
    let filter = Some(json!({
        "field": "post_id",
        "operator": "eq",
        "value": post_id
    }));
    let comments = client.query("comment", &["id", "content", "author_id", "post_id", "created_at"], filter).await?;
    info!("Found {} comments for post {}", comments.len(), post_id);
    
    for (i, comment) in comments.iter().enumerate() {
        info!("Comment {}: {}", i + 1, comment);
    }

    // Discover remote nodes
    info!("Discovering remote nodes...");
    let nodes = client.discover_nodes().await?;
    info!("Discovered {} nodes", nodes.len());
    
    for (i, node) in nodes.iter().enumerate() {
        info!("Node {}: {}", i + 1, node);
    }

    // Remote node querying is not implemented in this simplified version
    info!("Remote node querying is not implemented in this simplified version");

    info!("Social App completed successfully!");
    
    Ok(())
}

async fn create_user_schema(client: &mut NodeClient) -> Result<()> {
    info!("Creating user schema...");
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

    let result = client.send_request("create_schema", json!({ "schema": user_schema })).await?;
    info!("User schema creation result: {:?}", result);
    Ok(())
}

async fn create_post_schema(client: &mut NodeClient) -> Result<()> {
    info!("Creating post schema...");
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

    let result = client.send_request("create_schema", json!({ "schema": post_schema })).await?;
    info!("Post schema creation result: {:?}", result);
    Ok(())
}

async fn create_comment_schema(client: &mut NodeClient) -> Result<()> {
    info!("Creating comment schema...");
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

    let result = client.send_request("create_schema", json!({ "schema": comment_schema })).await?;
    info!("Comment schema creation result: {:?}", result);
    Ok(())
}
