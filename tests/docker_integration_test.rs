use std::path::PathBuf;
use std::process::Command;
use std::fs;
use std::env;
use std::io;
use std::path::Path;

use fold_db::{DataFoldNode, NodeConfig};
use fold_db::testing::{
    Schema,
    Query,
    Mutation,
    SchemaField,
    PermissionsPolicy,
    TrustDistance,
    FieldPaymentConfig,
    TrustDistanceScaling,
};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

fn copy_dir_recursive(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    if !dst.as_ref().exists() {
        fs::create_dir_all(&dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.as_ref().join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

#[test]
fn test_docker_node_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Check if Docker is available
    if let Err(_) = Command::new("docker").arg("--version").output() {
        println!("Skipping docker integration test - Docker not available");
        return Ok(());
    }
    // Create temporary test directory
    let test_dir = env::temp_dir().join("datafold_test");
    fs::create_dir_all(&test_dir)?;
    
    // Create test schema with proper field configurations
    let mut schema = Schema::new("test_schema".to_string());
    
    // Create permissions policy with trust distance
    let permissions = PermissionsPolicy::new(
        TrustDistance::Distance(1),  // read policy
        TrustDistance::Distance(1),  // write policy
    );
    
    // Create payment config
    let payment_config = FieldPaymentConfig::new(
        1.0,  // base multiplier
        TrustDistanceScaling::None,
        None, // min payment
    )?;
    
    // Add fields with proper configuration
    schema.add_field(
        "id".to_string(),
        SchemaField::new(
            permissions.clone(),
            payment_config.clone(),
            HashMap::new(),
        ).with_ref_atom_uuid(Uuid::new_v4().to_string()),
    );
    
    schema.add_field(
        "data".to_string(),
        SchemaField::new(
            permissions,
            payment_config,
            HashMap::new(),
        ).with_ref_atom_uuid(Uuid::new_v4().to_string()),
    );
    
    // Create test_data directory and write schema
    fs::create_dir_all(test_dir.join("test_data"))?;
    fs::write(
        test_dir.join("test_data/schema.json"),
        serde_json::to_string_pretty(&schema)?
    )?;
    
    // Copy project files to test directory
    let cargo_toml = fs::read_to_string("Cargo.toml")?;
    fs::write(test_dir.join("Cargo.toml"), cargo_toml)?;
    
    fs::create_dir_all(test_dir.join("src"))?;
    copy_dir_recursive("src", &test_dir.join("src"))?;
    
    // Create Dockerfile
    let dockerfile = r#"
# Build stage
FROM rust:1.70-slim as builder
WORKDIR /app
COPY Cargo.toml .
COPY src src/
COPY test_data test_data/
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim
WORKDIR /app
COPY --from=builder /app/target/release/datafold_node .
COPY ./test_data/schema.json /app/config/schema.json

RUN useradd -r -u 1001 -g root datafold
USER datafold

VOLUME ["/app/data"]
EXPOSE 8080

ENV MEMORY_LIMIT=1g
ENV CPU_LIMIT=1.0

ENTRYPOINT ["./datafold_node"]
"#;
    
    fs::write(test_dir.join("Dockerfile"), dockerfile)?;
    
    // Create docker-compose.yml
    let docker_compose = r#"
version: '3.8'
services:
  test_node:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - ./data:/app/data
    environment:
      - NODE_CONFIG=/app/config/node_config.json
    deploy:
      resources:
        limits:
          cpus: '1.0'
          memory: 1G
"#;
    
    fs::write(test_dir.join("docker-compose.yml"), docker_compose)?;
    
    // Create node config
    let node_config = r#"{
        "storage_path": "/app/data",
        "default_trust_distance": 1,
        "docker": {
            "memory_limit": 1073741824,
            "cpu_limit": 1.0,
            "environment": {},
            "network_config": {
                "network_isolated": true,
                "exposed_ports": {}
            }
        }
    }"#;
    
    fs::create_dir_all(test_dir.join("config"))?;
    fs::write(test_dir.join("config/node_config.json"), node_config)?;
    
    // Build and start container
    Command::new("docker-compose")
        .arg("up")
        .arg("-d")
        .current_dir(&test_dir)
        .status()?;
    
    // Initialize test client
    let node_config = NodeConfig {
        storage_path: PathBuf::from("/app/data"),
        default_trust_distance: 1,
        docker: fold_db::datafold_node::DockerConfig::default(),
    };
    
    let mut node = DataFoldNode::new(node_config)?;
    
    // Load schema
    node.load_schema(schema)?;
    
    // Test mutation
    let mut fields_and_values = HashMap::new();
    fields_and_values.insert("id".to_string(), json!("test1"));
    fields_and_values.insert("data".to_string(), json!("test data"));
    
    let mutation = Mutation::new(
        "test_schema".to_string(),
        fields_and_values,
        "test_key".to_string(),
        1,
    );
    
    node.mutate(mutation)?;
    
    // Wait a moment for the mutation to be processed
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    // Test query
    let query = Query::new(
        "test_schema".to_string(),
        vec!["id".to_string(), "data".to_string()],
        "test_key".to_string(),
        1,
    );
    
    let results = node.query(query)?;
    
    // Check that we got at least one successful result
    assert!(!results.is_empty());
    
    // Find and verify the expected result
    let found = results.iter().any(|result| {
        if let Ok(value) = result {
            value.get("id").and_then(|id| id.as_str()) == Some("test1") &&
            value.get("data").and_then(|data| data.as_str()) == Some("test data")
        } else {
            false
        }
    });
    assert!(found, "Expected test data not found in query results");
    
    // Cleanup
    Command::new("docker-compose")
        .arg("down")
        .current_dir(&test_dir)
        .status()?;
    
    fs::remove_dir_all(&test_dir)?;
    
    Ok(())
}
