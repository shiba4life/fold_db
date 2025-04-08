//! Docker module for FoldClient
//!
//! This module provides functionality for managing Docker containers.

use crate::Result;
use crate::FoldClientError;
use crate::DockerConfig;
use async_trait::async_trait;
use bollard::container::{
    Config as ContainerConfig, CreateContainerOptions, ListContainersOptions,
    RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
};
use bollard::models::{HostConfig, Mount, MountTypeEnum};
use bollard::Docker;
use std::collections::HashMap;
use std::path::Path;

/// Docker container manager
#[derive(Clone)]
pub struct DockerManager {
    /// Docker client
    docker: Docker,
    /// Docker configuration
    config: DockerConfig,
}

impl DockerManager {
    /// Create a new Docker manager
    pub async fn new(config: DockerConfig) -> Result<Self> {
        // Connect to Docker
        let docker = match config.docker_host {
            Some(ref _host) => Docker::connect_with_local_defaults()
                .map_err(|e| FoldClientError::Docker(format!("Failed to connect to Docker: {}", e)))?,
            None => Docker::connect_with_local_defaults()
                .map_err(|e| FoldClientError::Docker(format!("Failed to connect to Docker: {}", e)))?,
        };

        // Verify Docker connection
        docker
            .ping()
            .await
            .map_err(|e| FoldClientError::Docker(format!("Failed to ping Docker: {}", e)))?;

        Ok(Self { docker, config })
    }

    /// Create a container for an app
    pub async fn create_container(
        &self,
        app_id: &str,
        working_dir: &Path,
        program: &str,
        args: &[&str],
        env_vars: &HashMap<String, String>,
        allow_network: bool,
        memory_limit: Option<u64>,
        cpu_limit: Option<u64>,
    ) -> Result<String> {
        // Create container options
        let options = CreateContainerOptions {
            name: format!("fold_client_{}", app_id),
            platform: None,
        };

        // Create host config
        let mut host_config = HostConfig {
            auto_remove: Some(self.config.auto_remove),
            memory: memory_limit.or(Some(self.config.default_memory_limit * 1024 * 1024)).map(|m| m as i64),
            memory_swap: memory_limit.map(|m| (m * 2 * 1024 * 1024) as i64),
            cpu_shares: cpu_limit.or(Some(self.config.default_cpu_limit)).map(|c| c as i64),
            network_mode: Some(if allow_network {
                self.config.network.clone()
            } else {
                "none".to_string()
            }),
            mounts: Some(vec![Mount {
                target: Some("/app".to_string()),
                source: Some(working_dir.to_string_lossy().to_string()),
                typ: Some(MountTypeEnum::BIND),
                read_only: Some(false),
                ..Default::default()
            }]),
            ..Default::default()
        };

        // Set storage limit if supported
        if let Some(storage_limit) = Some(self.config.default_storage_limit * 1024 * 1024) {
            host_config.storage_opt = Some(HashMap::from([
                ("size".to_string(), storage_limit.to_string()),
            ]));
        }

        // Create environment variables
        let env = env_vars
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>();

        // Create container config
        let container_config = ContainerConfig {
            image: Some(self.config.base_image.clone()),
            cmd: Some(
                std::iter::once(program.to_string())
                    .chain(args.iter().map(|s| s.to_string()))
                    .collect(),
            ),
            env: Some(env),
            working_dir: Some("/app".to_string()),
            host_config: Some(host_config),
            ..Default::default()
        };

        // Create the container
        let response = self
            .docker
            .create_container(Some(options), container_config)
            .await
            .map_err(|e| {
                FoldClientError::Docker(format!("Failed to create container: {}", e))
            })?;

        // Return the container ID
        Ok(response.id)
    }

    /// Start a container
    pub async fn start_container(&self, container_id: &str) -> Result<()> {
        self.docker
            .start_container(container_id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| {
                FoldClientError::Docker(format!("Failed to start container: {}", e))
            })?;

        Ok(())
    }

    /// Stop a container
    pub async fn stop_container(&self, container_id: &str) -> Result<()> {
        let options = StopContainerOptions {
            t: 10, // 10 seconds timeout
        };

        self.docker
            .stop_container(container_id, Some(options))
            .await
            .map_err(|e| {
                FoldClientError::Docker(format!("Failed to stop container: {}", e))
            })?;

        Ok(())
    }

    /// Remove a container
    pub async fn remove_container(&self, container_id: &str) -> Result<()> {
        let options = RemoveContainerOptions {
            force: true,
            ..Default::default()
        };

        self.docker
            .remove_container(container_id, Some(options))
            .await
            .map_err(|e| {
                FoldClientError::Docker(format!("Failed to remove container: {}", e))
            })?;

        Ok(())
    }

    /// Check if a container is running
    pub async fn is_container_running(&self, container_id: &str) -> Result<bool> {
        let options = ListContainersOptions::<String> {
            all: true,
            filters: HashMap::from([("id".to_string(), vec![container_id.to_string()])]),
            ..Default::default()
        };

        let containers = self
            .docker
            .list_containers(Some(options))
            .await
            .map_err(|e| {
                FoldClientError::Docker(format!("Failed to list containers: {}", e))
            })?;

        if containers.is_empty() {
            return Ok(false);
        }

        let container = &containers[0];
        let state = container.state.as_deref().unwrap_or("");

        Ok(state == "running")
    }

    /// List all containers created by FoldClient
    pub async fn list_containers(&self) -> Result<Vec<String>> {
        let options = ListContainersOptions::<String> {
            all: true,
            filters: HashMap::from([("name".to_string(), vec!["fold_client_".to_string()])]),
            ..Default::default()
        };

        let containers = self
            .docker
            .list_containers(Some(options))
            .await
            .map_err(|e| {
                FoldClientError::Docker(format!("Failed to list containers: {}", e))
            })?;

        let container_ids = containers
            .iter()
            .filter_map(|c| c.id.clone())
            .collect();

        Ok(container_ids)
    }

    /// Clean up stopped containers
    pub async fn cleanup_containers(&self) -> Result<()> {
        let options = ListContainersOptions::<String> {
            all: true,
            filters: HashMap::from([
                ("name".to_string(), vec!["fold_client_".to_string()]),
                ("status".to_string(), vec!["exited".to_string(), "dead".to_string()]),
            ]),
            ..Default::default()
        };

        let containers = self
            .docker
            .list_containers(Some(options))
            .await
            .map_err(|e| {
                FoldClientError::Docker(format!("Failed to list containers: {}", e))
            })?;

        for container in containers {
            if let Some(id) = container.id {
                let remove_options = RemoveContainerOptions {
                    force: false,
                    ..Default::default()
                };

                if let Err(e) = self.docker.remove_container(&id, Some(remove_options)).await {
                    log::warn!("Failed to remove container {}: {}", id, e);
                }
            }
        }

        Ok(())
    }
}

/// Trait for container managers
#[async_trait]
pub trait ContainerManager {
    /// Create a container
    async fn create_container(
        &self,
        app_id: &str,
        working_dir: &Path,
        program: &str,
        args: &[&str],
        env_vars: &HashMap<String, String>,
        allow_network: bool,
        memory_limit: Option<u64>,
        cpu_limit: Option<u64>,
    ) -> Result<String>;

    /// Start a container
    async fn start_container(&self, container_id: &str) -> Result<()>;

    /// Stop a container
    async fn stop_container(&self, container_id: &str) -> Result<()>;

    /// Remove a container
    async fn remove_container(&self, container_id: &str) -> Result<()>;

    /// Check if a container is running
    async fn is_container_running(&self, container_id: &str) -> Result<bool>;

    /// List all containers
    async fn list_containers(&self) -> Result<Vec<String>>;

    /// Clean up containers
    async fn cleanup_containers(&self) -> Result<()>;
}

#[async_trait]
impl ContainerManager for DockerManager {
    async fn create_container(
        &self,
        app_id: &str,
        working_dir: &Path,
        program: &str,
        args: &[&str],
        env_vars: &HashMap<String, String>,
        allow_network: bool,
        memory_limit: Option<u64>,
        cpu_limit: Option<u64>,
    ) -> Result<String> {
        self.create_container(
            app_id,
            working_dir,
            program,
            args,
            env_vars,
            allow_network,
            memory_limit,
            cpu_limit,
        )
        .await
    }

    async fn start_container(&self, container_id: &str) -> Result<()> {
        self.start_container(container_id).await
    }

    async fn stop_container(&self, container_id: &str) -> Result<()> {
        self.stop_container(container_id).await
    }

    async fn remove_container(&self, container_id: &str) -> Result<()> {
        self.remove_container(container_id).await
    }

    async fn is_container_running(&self, container_id: &str) -> Result<bool> {
        self.is_container_running(container_id).await
    }

    async fn list_containers(&self) -> Result<Vec<String>> {
        self.list_containers().await
    }

    async fn cleanup_containers(&self) -> Result<()> {
        self.cleanup_containers().await
    }
}
