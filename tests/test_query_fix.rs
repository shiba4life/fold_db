use fold_db::{DataFoldNode, NodeConfig};
use fold_db::testing::Operation;
use serde_json::{json, Value};
use tempfile::tempdir;

#[tokio::main]
async fn main() {
    // Create a test node
    let dir = tempdir().unwrap();
    let config = NodeConfig {
        storage_path: dir.path().to_path_buf(),
        default_trust_distance: 1,
        network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
    };
    let mut node = DataFoldNode::new(config).unwrap();
    
    // Load the schema
    let schema_str = std::fs::read_to_string("src/datafold_node/examples/user_profile_schema.json")
        .expect("Failed to read schema file");
    
    let schema: fold_db::schema::Schema = serde_json::from_str(&schema_str)
        .expect("Failed to parse schema JSON");
    
    node.load_schema(schema).expect("Failed to load schema");
    
    // Create test data
    let mutation_str = r#"{
        "type": "mutation",
        "schema": "UserProfile",
        "mutation_type": "create",
        "data": {
            "username": "testuser",
            "email": "test@example.com",
            "full_name": "Test User",
            "bio": "Test bio",
            "age": 30,
            "location": "Test Location"
        }
    }"#;
    
    let mutation: Value = serde_json::from_str(mutation_str).expect("Failed to parse mutation");
    
    // Execute the mutation
    let operation_str = mutation.to_string();
    let operation: Operation = serde_json::from_str(&operation_str)
        .expect("Failed to parse operation");
    
    node.execute_operation(operation).expect("Failed to execute mutation");
    
    // Execute a query
    let query_str = r#"{
        "type": "query",
        "schema": "UserProfile",
        "fields": ["username", "email", "bio"],
        "filter": null
    }"#;
    
    let query: Value = serde_json::from_str(query_str).expect("Failed to parse query");
    
    // Execute the query
    let operation_str = query.to_string();
    let operation: Operation = serde_json::from_str(&operation_str)
        .expect("Failed to parse operation");
    
    let result = node.execute_operation(operation).expect("Failed to execute query");
    
    println!("Query result: {}", serde_json::to_string_pretty(&result).unwrap());
}
