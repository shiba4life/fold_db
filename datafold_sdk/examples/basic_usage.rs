use datafold_sdk::{DataFoldClient, QueryFilter};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let client = DataFoldClient::new("example-app", "private-key", "public-key");
    
    println!("Discovering local schemas...");
    let schemas = client.discover_local_schemas().await?;
    println!("Available schemas: {:?}", schemas);
    
    println!("\nQuerying user data...");
    let query_result = client.query("user")
        .select(&["id", "name", "email"])
        .filter(QueryFilter::eq("name", json!("John Doe")))
        .execute()
        .await?;
    
    println!("Query results: {:?}", query_result);
    
    println!("\nCreating a new user...");
    let mutation_result = client.mutate("user")
        .set("name", json!("Jane Doe"))
        .set("email", json!("jane@example.com"))
        .execute()
        .await?;
    
    println!("Mutation result: {:?}", mutation_result);
    
    println!("\nDiscovering remote nodes...");
    let nodes = client.discover_nodes().await?;
    println!("Remote nodes: {:?}", nodes);
    
    if !nodes.is_empty() {
        let node_id = &nodes[0].id;
        println!("\nDiscovering schemas on remote node {}...", node_id);
        let remote_schemas = client.discover_remote_schemas(node_id).await?;
        println!("Remote schemas: {:?}", remote_schemas);
        
        if !remote_schemas.is_empty() {
            let schema_name = &remote_schemas[0];
            println!("\nQuerying remote data from schema {}...", schema_name);
            let remote_query_result = client.query_on_node(schema_name, node_id)
                .select(&["id", "name"])
                .execute()
                .await?;
            
            println!("Remote query results: {:?}", remote_query_result);
        }
    }
    
    Ok(())
}
