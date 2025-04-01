use datafold_sdk::{
    SocialAppContainer, ContainerConfig, AppPermissions, FieldPermissions,
    MicroVMConfig, MicroVMType
};
use datafold_sdk::isolation::ResourceLimits;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create permissions
    let permissions = AppPermissions::new()
        .allow_schemas(&["user", "post", "comment"])
        .with_field_permissions("user", FieldPermissions::new()
            .allow_reads(&["id", "name", "email", "created_at"])
            .allow_writes(&["name", "email"]))
        .with_field_permissions("post", FieldPermissions::new()
            .allow_reads(&["id", "title", "content", "author_id", "created_at"])
            .allow_writes(&["title", "content"]))
        .with_max_trust_distance(2);
    
    // Create resource limits
    let resource_limits = ResourceLimits {
        max_cpu_percent: 25.0,
        max_memory_mb: 128,
        max_storage_mb: 512,
        max_concurrent_ops: 5,
    };
    
    // Create VM configuration
    let vm_config = MicroVMConfig::new(
        MicroVMType::Firecracker,
        "/var/lib/datafold/vm-images/minimal-rootfs.ext4"
    )
    .with_vcpu_count(1)
    .with_memory_mb(128);
    
    // Create container configuration
    let config = ContainerConfig::new_microvm(
        PathBuf::from("/path/to/social-app"),
        vm_config
    )
    .add_env_var("APP_ENV", "production")
    .add_env_var("LOG_LEVEL", "info")
    .add_args(&["--no-analytics", "--cache-enabled"]);
    
    println!("Creating social app container...");
    let mut container = SocialAppContainer::new(
        "social-app-1",
        "app-public-key",
        permissions,
        config
    );
    
    println!("Starting container...");
    container.start().await?;
    println!("Container started with process ID: {:?}", container.process_id);
    
    println!("Container status: {:?}", container.get_status());
    println!("Is container running? {}", container.is_running());
    
    println!("Getting resource usage...");
    let usage = container.get_resource_usage()?;
    println!("CPU: {}%, Memory: {} MB, Storage: {} MB, Concurrent Ops: {}", 
        usage.cpu_percent, usage.memory_mb, usage.storage_mb, usage.concurrent_ops);
    
    println!("Stopping container...");
    container.stop().await?;
    println!("Container stopped");
    
    println!("Container status: {:?}", container.get_status());
    println!("Is container running? {}", container.is_running());
    
    Ok(())
}
