use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use super::config::SandboxConfig;
use super::security::SecurityMiddleware;
use super::types::{ContainerInfo, ContainerStatus, Request, ResourceLimits, Response, SandboxError, SandboxResult};

/// Manager for sandboxed containers
pub struct SandboxManager {
    /// Network ID for the internal Docker network
    network_id: String,
    /// Map of container IDs to container information
    containers: Arc<Mutex<HashMap<String, ContainerInfo>>>,
    /// Security middleware
    security: Arc<SecurityMiddleware>,
    /// Configuration
    config: SandboxConfig,
}

impl SandboxManager {
    /// Creates a new sandbox manager
    pub fn new(config: SandboxConfig) -> SandboxResult<Self> {
        // Create security middleware
        let security = Arc::new(SecurityMiddleware::new(config.clone()));

        // Create the internal Docker network if it doesn't exist
        let network_id = Self::create_network(&config.network_name)?;

        Ok(Self {
            network_id,
            containers: Arc::new(Mutex::new(HashMap::new())),
            security,
            config,
        })
    }

    /// Creates the internal Docker network
    fn create_network(network_name: &str) -> SandboxResult<String> {
        // Check if network already exists
        let output = Command::new("docker")
            .args(["network", "ls", "--filter", &format!("name={}", network_name), "--format", "{{.ID}}"])
            .output()
            .map_err(|e| SandboxError::Docker(format!("Failed to list networks: {}", e)))?;

        let network_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !network_id.is_empty() {
            return Ok(network_id);
        }

        // Create the network
        let output = Command::new("docker")
            .args(["network", "create", "--internal", network_name])
            .output()
            .map_err(|e| SandboxError::Docker(format!("Failed to create network: {}", e)))?;

        if !output.status.success() {
            return Err(SandboxError::Docker(format!(
                "Failed to create network: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Registers a container with the sandbox manager
    pub fn register_container(&self, container_id: &str, name: &str, image: &str, resource_limits: Option<ResourceLimits>) -> SandboxResult<()> {
        let mut containers = self.containers
            .lock()
            .map_err(|_| SandboxError::Internal("Failed to lock containers".to_string()))?;

        // Check if container is already registered
        if containers.contains_key(container_id) {
            return Err(SandboxError::Config(format!(
                "Container {} is already registered",
                container_id
            )));
        }

        // Create container info
        let container_info = ContainerInfo {
            id: container_id.to_string(),
            name: name.to_string(),
            image: image.to_string(),
            status: ContainerStatus::Unknown,
            created_at: SystemTime::now(),
            labels: HashMap::new(),
            resource_limits: resource_limits.unwrap_or_else(|| self.config.default_resource_limits.clone()),
            network_id: Some(self.network_id.clone()),
        };

        // Register container
        containers.insert(container_id.to_string(), container_info);

        Ok(())
    }

    /// Starts a container
    pub fn start_container(&self, container_id: &str) -> SandboxResult<()> {
        // Check if container is registered
        let containers = self.containers
            .lock()
            .map_err(|_| SandboxError::Internal("Failed to lock containers".to_string()))?;

        let container_info = containers.get(container_id)
            .ok_or_else(|| SandboxError::ContainerNotFound(container_id.to_string()))?;

        // Build Docker run command
        let mut cmd = Command::new("docker");
        cmd.arg("run")
            .arg("-d")  // Detached mode
            .arg("--rm")  // Remove container when it exits
            .arg(format!("--name={}", container_info.name))
            .arg(format!("--network={}", self.network_id));

        // Add security options
        if self.config.security_options.drop_all_capabilities {
            cmd.arg("--cap-drop=ALL");
        }

        for cap in &self.config.security_options.add_capabilities {
            cmd.arg(format!("--cap-add={}", cap));
        }

        if self.config.security_options.no_new_privileges {
            cmd.arg("--security-opt=no-new-privileges");
        }

        if self.config.security_options.read_only_root_fs {
            cmd.arg("--read-only");
        }

        if let Some(profile) = &self.config.security_options.seccomp_profile {
            cmd.arg(format!("--security-opt=seccomp={}", profile));
        }

        if let Some(profile) = &self.config.security_options.apparmor_profile {
            cmd.arg(format!("--security-opt=apparmor={}", profile));
        }

        // Add resource limits
        if let Some(cpu) = container_info.resource_limits.cpu_limit {
            cmd.arg(format!("--cpus={}", cpu));
        }

        if let Some(memory) = container_info.resource_limits.memory_limit {
            cmd.arg(format!("--memory={}b", memory));
        }

        if let Some(pids) = container_info.resource_limits.pids_limit {
            cmd.arg(format!("--pids-limit={}", pids));
        }

        // Add environment variables
        for (key, value) in &self.config.environment_variables {
            cmd.arg(format!("--env={}={}", key, value));
        }

        // Add volume mounts
        for mount in &self.config.volume_mounts {
            let mut mount_arg = format!("--volume={}:{}", mount.source.display(), mount.target.display());
            if mount.read_only {
                mount_arg.push_str(":ro");
            }
            cmd.arg(mount_arg);
        }

        // Add API environment variables
        if self.config.use_unix_socket {
            if let Some(socket_path) = &self.config.api_socket_path {
                cmd.arg(format!("--volume={}:/datafold.sock", socket_path.display()));
                cmd.arg("--env=DATAFOLD_API_SOCKET=/datafold.sock");
            }
        } else {
            cmd.arg(format!("--env=DATAFOLD_API_HOST={}", self.config.api_host));
            cmd.arg(format!("--env=DATAFOLD_API_PORT={}", self.config.api_port));
        }

        // Add container image
        cmd.arg(&container_info.image);

        // Run the container
        let output = cmd.output()
            .map_err(|e| SandboxError::Docker(format!("Failed to start container: {}", e)))?;

        if !output.status.success() {
            return Err(SandboxError::Docker(format!(
                "Failed to start container: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Update container status
        drop(containers);  // Release lock before acquiring it again
        let mut containers = self.containers
            .lock()
            .map_err(|_| SandboxError::Internal("Failed to lock containers".to_string()))?;

        if let Some(container) = containers.get_mut(container_id) {
            container.status = ContainerStatus::Running;
        }

        Ok(())
    }

    /// Stops a container
    pub fn stop_container(&self, container_id: &str) -> SandboxResult<()> {
        // Check if container is registered
        let containers = self.containers
            .lock()
            .map_err(|_| SandboxError::Internal("Failed to lock containers".to_string()))?;

        let container_info = containers.get(container_id)
            .ok_or_else(|| SandboxError::ContainerNotFound(container_id.to_string()))?;

        // Stop the container
        let output = Command::new("docker")
            .args(["stop", &container_info.id])
            .output()
            .map_err(|e| SandboxError::Docker(format!("Failed to stop container: {}", e)))?;

        if !output.status.success() {
            return Err(SandboxError::Docker(format!(
                "Failed to stop container: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Update container status
        drop(containers);  // Release lock before acquiring it again
        let mut containers = self.containers
            .lock()
            .map_err(|_| SandboxError::Internal("Failed to lock containers".to_string()))?;

        if let Some(container) = containers.get_mut(container_id) {
            container.status = ContainerStatus::Stopped;
        }

        Ok(())
    }

    /// Removes a container
    pub fn remove_container(&self, container_id: &str) -> SandboxResult<()> {
        // Check if container is registered
        let mut containers = self.containers
            .lock()
            .map_err(|_| SandboxError::Internal("Failed to lock containers".to_string()))?;

        let container_info = containers.get(container_id)
            .ok_or_else(|| SandboxError::ContainerNotFound(container_id.to_string()))?;

        // Remove the container
        let output = Command::new("docker")
            .args(["rm", "-f", &container_info.id])
            .output()
            .map_err(|e| SandboxError::Docker(format!("Failed to remove container: {}", e)))?;

        if !output.status.success() {
            return Err(SandboxError::Docker(format!(
                "Failed to remove container: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Remove container from registry
        containers.remove(container_id);

        Ok(())
    }

    /// Gets information about a container
    pub fn get_container_info(&self, container_id: &str) -> SandboxResult<ContainerInfo> {
        let containers = self.containers
            .lock()
            .map_err(|_| SandboxError::Internal("Failed to lock containers".to_string()))?;

        containers.get(container_id)
            .cloned()
            .ok_or_else(|| SandboxError::ContainerNotFound(container_id.to_string()))
    }

    /// Lists all registered containers
    pub fn list_containers(&self) -> SandboxResult<Vec<ContainerInfo>> {
        let containers = self.containers
            .lock()
            .map_err(|_| SandboxError::Internal("Failed to lock containers".to_string()))?;

        Ok(containers.values().cloned().collect())
    }

    /// Validates a request from a container
    pub fn validate_request(&self, container_id: &str, request: &Request) -> SandboxResult<()> {
        // Check if container is registered
        let containers = self.containers
            .lock()
            .map_err(|_| SandboxError::Internal("Failed to lock containers".to_string()))?;

        if !containers.contains_key(container_id) {
            return Err(SandboxError::ContainerNotFound(container_id.to_string()));
        }

        // Check rate limits
        self.security.check_rate_limit(container_id)?;

        // Validate request
        self.security.validate_request(request)
    }

    /// Proxies a request to the Datafold API
    pub fn proxy_request(&self, request: Request) -> SandboxResult<Response> {
        // Validate request
        self.validate_request(&request.container_id, &request)?;

        // Proxy request based on configuration
        if self.config.use_unix_socket {
            self.proxy_request_unix_socket(request)
        } else {
            self.proxy_request_network(request)
        }
    }

    /// Proxies a request using Unix socket
    fn proxy_request_unix_socket(&self, request: Request) -> SandboxResult<Response> {
        // Check if socket path is configured
        let socket_path = self.config.api_socket_path.as_ref()
            .ok_or_else(|| SandboxError::Config("Unix socket path not configured".to_string()))?;

        // Build curl command for Unix socket
        let mut cmd = Command::new("curl");
        cmd.arg("--unix-socket")
            .arg(socket_path)
            .arg("-X")
            .arg(&request.method)
            .arg(format!("http://localhost{}", request.path));

        // Add headers
        for (key, value) in &request.headers {
            cmd.arg("-H").arg(format!("{}: {}", key, value));
        }

        // Add request body if present
        if let Some(body) = &request.body {
            cmd.arg("-d").arg(String::from_utf8_lossy(body).to_string());
        }

        // Execute curl command
        let output = cmd.output()
            .map_err(|e| SandboxError::Network(format!("Failed to execute curl command: {}", e)))?;

        // Parse response
        let status = if output.status.success() { 200 } else { 500 };
        let body = if output.status.success() {
            output.stdout.clone()
        } else {
            output.stderr.clone()
        };

        Ok(Response {
            status,
            headers: HashMap::new(),
            body: Some(body),
        })
    }

    /// Proxies a request using network
    fn proxy_request_network(&self, request: Request) -> SandboxResult<Response> {
        // Build curl command for network
        let mut cmd = Command::new("curl");
        cmd.arg("-X")
            .arg(&request.method)
            .arg(format!("http://{}:{}{}", self.config.api_host, self.config.api_port, request.path));

        // Add headers
        for (key, value) in &request.headers {
            cmd.arg("-H").arg(format!("{}: {}", key, value));
        }

        // Add request body if present
        if let Some(body) = &request.body {
            cmd.arg("-d").arg(String::from_utf8_lossy(body).to_string());
        }

        // Execute curl command
        let output = cmd.output()
            .map_err(|e| SandboxError::Network(format!("Failed to execute curl command: {}", e)))?;

        // Parse response
        let status = if output.status.success() { 200 } else { 500 };
        let body = if output.status.success() {
            output.stdout.clone()
        } else {
            output.stderr.clone()
        };

        Ok(Response {
            status,
            headers: HashMap::new(),
            body: Some(body),
        })
    }

    /// Gets the network ID
    pub fn get_network_id(&self) -> &str {
        &self.network_id
    }

    /// Gets the configuration
    pub fn get_config(&self) -> &SandboxConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires Docker to be running
    fn test_create_network() {
        let network_name = "test_datafold_network";
        let network_id = SandboxManager::create_network(network_name).unwrap();
        assert!(!network_id.is_empty());

        // Clean up
        let _ = Command::new("docker")
            .args(["network", "rm", network_name])
            .output();
    }
}
