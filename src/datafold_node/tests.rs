use super::*;
use std::process::Command;
use tempfile::tempdir;

fn create_test_config() -> NodeConfig {
    let dir = tempdir().unwrap();
    NodeConfig {
        storage_path: dir.path().to_path_buf(),
        default_trust_distance: 1,
        docker: DockerConfig::default(),
    }
}

#[test]
fn test_node_creation() {
    let config = create_test_config();
    let node = DataFoldNode::new(config);
    assert!(node.is_ok());
}

#[test]
fn test_add_trusted_node() {
    let config = create_test_config();
    let mut node = DataFoldNode::new(config).unwrap();

    assert!(node.add_trusted_node("test_node").is_ok());
    assert!(node.get_trusted_nodes().contains_key("test_node"));
    assert!(node.remove_trusted_node("test_node").is_ok());
    assert!(!node.get_trusted_nodes().contains_key("test_node"));
}

#[test]
fn test_docker_app_lifecycle() {
    // Skip if Docker is not available
    if Command::new("docker").arg("--version").output().is_err() {
        println!("Skipping docker test - Docker not available");
        return;
    }

    let mut config = create_test_config();
    
    // Configure Docker settings
    config.docker.memory_limit = 256 * 1024 * 1024; // 256MB
    config.docker.cpu_limit = 0.5;
    config.docker.environment.insert("TEST_ENV".to_string(), "test_value".to_string());
    config.docker.network_config.exposed_ports.insert(8080, 8081);

    let mut node = DataFoldNode::new(config).unwrap();

    // Test loading app
    let app_id = "test-app";
    let container_id = node.load_docker_app("hello-world:latest", app_id).unwrap();
    assert!(!container_id.is_empty());

    // Test getting status
    let status = node.get_docker_app_status(app_id).unwrap();
    assert!(matches!(status, Some(ContainerStatus::Running)));

    // Test removing app
    assert!(node.remove_docker_app(app_id).is_ok());

    // Verify app is removed
    let status = node.get_docker_app_status(app_id).unwrap();
    assert!(status.is_none());
}

#[test]
fn test_docker_app_error_handling() {
    // Skip if Docker is not available
    if Command::new("docker").arg("--version").output().is_err() {
        println!("Skipping docker test - Docker not available");
        return;
    }

    let config = create_test_config();
    let mut node = DataFoldNode::new(config).unwrap();

    // Test loading non-existent image
    let result = node.load_docker_app("non-existent-image:latest", "test-app");
    assert!(matches!(result, Err(NodeError::DockerError(_))));

    // Test removing non-existent app
    let result = node.remove_docker_app("non-existent-app");
    assert!(result.is_ok()); // Should succeed silently if app doesn't exist
}
