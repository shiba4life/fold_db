use std::path::PathBuf;
use fold_db::{
    DataFoldClient, SocialAppContainer, ContainerConfig, AppPermissions, FieldPermissions,
    MicroVMConfig, MicroVMType, LinuxContainerConfig, WasmSandboxConfig,
    QueryFilter, AppSdkMutationType
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("DataFold Social App SDK Example");
    println!("===============================");

    // Create a client for the app
    let client = DataFoldClient::new(
        "example-social-app",
        "private-key-placeholder",
        "public-key-placeholder",
    );

    println!("\n1. Client created for app: {}", client.get_app_id());

    // Discover available schemas
    println!("\n2. Discovering local schemas...");
    let schemas = client.discover_local_schemas().await?;
    println!("Available schemas: {:?}", schemas);

    // Create app permissions
    println!("\n3. Setting up app permissions...");
    let permissions = AppPermissions::new()
        .allow_schemas(&["user", "post", "comment"])
        .with_field_permissions(
            "user",
            FieldPermissions::new()
                .allow_reads(&["id", "username", "full_name", "bio"])
                .allow_writes(&["bio"]),
        )
        .with_field_permissions(
            "post",
            FieldPermissions::new()
                .allow_read_writes(&["id", "title", "content", "author_id", "created_at"]),
        )
        .with_field_permissions(
            "comment",
            FieldPermissions::new()
                .allow_read_writes(&["id", "content", "author_id", "post_id", "created_at"]),
        )
        .allow_remote_nodes(&["node1", "node2"])
        .with_max_trust_distance(2);

    println!("Permissions set up for schemas: {:?}", permissions.allowed_schemas);

    // Create container configurations for different isolation types
    println!("\n4. Creating container configurations...");

    // MicroVM with Firecracker
    let firecracker_config = ContainerConfig::new_microvm(
        PathBuf::from("/path/to/app/binary"),
        MicroVMConfig::new(MicroVMType::Firecracker, "/var/lib/datafold/vm-images/minimal-rootfs.ext4")
            .with_vcpu_count(1)
            .with_memory_mb(128),
    );
    println!("Created Firecracker MicroVM configuration");

    // Linux container
    let container_config = ContainerConfig::new_linux_container(
        PathBuf::from("/path/to/app/binary"),
        LinuxContainerConfig::maximum(),
    );
    println!("Created Linux container configuration");

    // WebAssembly sandbox
    let wasm_config = ContainerConfig::new_wasm_sandbox(
        PathBuf::from("/path/to/app/wasm"),
        WasmSandboxConfig::maximum(),
    );
    println!("Created WebAssembly sandbox configuration");

    // Create a social app container
    println!("\n5. Creating a social app container...");
    let container = SocialAppContainer::new(
        "example-social-app",
        "public-key-placeholder",
        permissions,
        firecracker_config,
    );
    println!("Container created with ID: {}", container.app_id);

    // In a real implementation, we would start the container
    // For this example, we'll just simulate it
    println!("\n6. Starting the container (simulated)...");
    // container.start().await?;
    println!("Container started successfully");

    // Example query using the client
    println!("\n7. Executing a query...");
    let query_result = client.query("user")
        .select(&["id", "username", "full_name", "bio"])
        .filter(QueryFilter::eq("username", serde_json::json!("testuser")))
        .execute()
        .await?;

    println!("Query results: {:?}", query_result.results);

    // Example mutation using the client
    println!("\n8. Executing a mutation...");
    let mutation_result = client.mutate("post")
        .operation(AppSdkMutationType::Create)
        .set("title", serde_json::json!("Hello DataFold Network"))
        .set("content", serde_json::json!("This is my first post on the decentralized social network!"))
        .set("author_id", serde_json::json!("user123"))
        .execute()
        .await?;

    println!("Mutation result: success={}, id={:?}", mutation_result.success, mutation_result.id);

    // Discover remote nodes
    println!("\n9. Discovering remote nodes...");
    let nodes = client.discover_nodes().await?;
    println!("Discovered nodes: {:?}", nodes);

    // Query on a remote node
    println!("\n10. Executing a query on a remote node...");
    let remote_query_result = client.query_on_node("user", "node1")
        .select(&["id", "username", "full_name"])
        .execute()
        .await?;

    println!("Remote query results: {:?}", remote_query_result.results);

    // In a real implementation, we would stop the container
    // For this example, we'll just simulate it
    println!("\n11. Stopping the container (simulated)...");
    // container.stop().await?;
    println!("Container stopped successfully");

    println!("\nExample completed successfully!");
    Ok(())
}
