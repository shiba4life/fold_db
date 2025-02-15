use std::process::Command;
use crate::datafold_node::error::NodeError;
use crate::datafold_node::config::DockerConfig;

#[derive(Debug, Clone)]
pub struct ContainerState {
    /// Container ID
    pub id: String,
    /// Container status
    pub status: ContainerStatus,
    /// Network ID if using isolated network
    pub network_id: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ContainerStatus {
    Created,
    Running,
    Stopped,
    Failed(String),
}

pub(crate) fn check_docker_available() -> Result<(), NodeError> {
    Command::new("docker")
        .arg("--version")
        .output()
        .map_err(|e| NodeError::DockerError(format!("Docker not available: {}", e)))?;
    Ok(())
}

pub(crate) fn create_container(
    image: &str,
    config: &DockerConfig,
) -> Result<String, NodeError> {
    let mut cmd = Command::new("docker");
    cmd.arg("create")
        .arg("--memory")
        .arg(format!("{}b", config.memory_limit))
        .arg("--cpus")
        .arg(config.cpu_limit.to_string())
        .arg("--network")
        .arg(if config.network_config.network_isolated {
            "none"
        } else {
            "bridge"
        });

    // Add environment variables
    for (key, value) in &config.environment {
        cmd.arg("-e").arg(format!("{}={}", key, value));
    }

    // Add port mappings
    for (container_port, host_port) in &config.network_config.exposed_ports {
        cmd.arg("-p").arg(format!("{}:{}", host_port, container_port));
    }

    cmd.arg(image);

    let output = cmd
        .output()
        .map_err(|e| NodeError::DockerError(format!("Failed to create container: {}", e)))?;

    if !output.status.success() {
        return Err(NodeError::DockerError(format!(
            "Container creation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub(crate) fn start_container(container_id: &str) -> Result<(), NodeError> {
    let output = Command::new("docker")
        .args(["start", container_id])
        .output()
        .map_err(|e| NodeError::DockerError(format!("Failed to start container: {}", e)))?;

    if !output.status.success() {
        return Err(NodeError::DockerError(format!(
            "Container start failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

pub(crate) fn stop_container(container_id: &str) -> Result<(), NodeError> {
    let output = Command::new("docker")
        .args(["stop", container_id])
        .output()
        .map_err(|e| NodeError::DockerError(format!("Failed to stop container: {}", e)))?;

    if !output.status.success() {
        return Err(NodeError::DockerError(format!(
            "Container stop failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

pub(crate) fn remove_container(container_id: &str) -> Result<(), NodeError> {
    let output = Command::new("docker")
        .args(["rm", container_id])
        .output()
        .map_err(|e| NodeError::DockerError(format!("Failed to remove container: {}", e)))?;

    if !output.status.success() {
        return Err(NodeError::DockerError(format!(
            "Container removal failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

pub(crate) fn cleanup_network(network_id: &str) {
    let _ = Command::new("docker")
        .args(["network", "rm", network_id])
        .output();
}
