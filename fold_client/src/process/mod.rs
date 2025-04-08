//! Process module for FoldClient
//!
//! This module provides functionality for managing Docker-sandboxed processes.

use crate::auth::AppRegistration;
use crate::docker::DockerManager;
use crate::DockerConfig;
use crate::Result;
use crate::FoldClientError;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Process manager for FoldClient
pub struct ProcessManager {
    /// Directory where app data is stored
    app_data_dir: PathBuf,
    /// Docker configuration
    docker_config: DockerConfig,
    /// Docker manager
    docker_manager: Arc<Mutex<Option<DockerManager>>>,
    /// Active containers
    containers: Arc<Mutex<HashMap<String, String>>>,
}

impl ProcessManager {
    /// Create a new process manager
    pub fn new(
        app_data_dir: PathBuf,
        docker_config: DockerConfig,
    ) -> Result<Self> {
        // Create the app data directory if it doesn't exist
        std::fs::create_dir_all(&app_data_dir)?;

        Ok(Self {
            app_data_dir,
            docker_config,
            docker_manager: Arc::new(Mutex::new(None)),
            containers: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Initialize the Docker manager
    async fn init_docker_manager(&self) -> Result<()> {
        // Check if Docker manager is already initialized
        {
            let docker_manager_lock = self.docker_manager.lock().unwrap();
            if docker_manager_lock.is_some() {
                return Ok(());
            }
        }
        
        // Initialize Docker manager
        let docker_manager = DockerManager::new(self.docker_config.clone()).await?;
        
        // Store Docker manager
        let mut docker_manager_lock = self.docker_manager.lock().unwrap();
        if docker_manager_lock.is_none() {
            *docker_manager_lock = Some(docker_manager);
        }
        
        Ok(())
    }

    /// Get the Docker manager
    async fn get_docker_manager(&self) -> Result<Arc<DockerManager>> {
        self.init_docker_manager().await?;
        let docker_manager = {
            let docker_manager_lock = self.docker_manager.lock().unwrap();
            docker_manager_lock.as_ref().unwrap().clone()
        };
        Ok(Arc::new(docker_manager))
    }

    /// Launch an app
    pub async fn launch_app(
        &self,
        app: AppRegistration,
        program: &str,
        args: &[&str],
    ) -> Result<()> {
        // Create the working directory for the app
        let working_dir = self.app_data_dir.join(&app.app_id);
        std::fs::create_dir_all(&working_dir)?;

        // Create environment variables for the app
        let mut env_vars = HashMap::new();
        env_vars.insert("FOLD_CLIENT_APP_ID".to_string(), app.app_id.clone());
        env_vars.insert("FOLD_CLIENT_APP_TOKEN".to_string(), app.token.clone());
        env_vars.insert("FOLD_CLIENT_SOCKET_DIR".to_string(), "/tmp/fold_client_sockets".to_string());

        // Get the Docker manager
        let docker_manager = self.get_docker_manager().await?;

        // Create container parameters
        let container_params = crate::docker::ContainerParams {
            app_id: &app.app_id,
            working_dir: &working_dir,
            program,
            args,
            env_vars: &env_vars,
            allow_network: self.docker_config.default_allow_network,
            memory_limit: None,
            cpu_limit: None,
        };

        // Create and start the container
        let container_id = docker_manager
            .create_container(container_params)
            .await?;

        docker_manager.start_container(&container_id).await?;

        // Store the container ID
        let mut containers = self.containers.lock().unwrap();
        containers.insert(app.app_id.clone(), container_id);

        Ok(())
    }

    /// Terminate an app
    pub async fn terminate_app(&self, app_id: &str) -> Result<()> {
        // Get the container ID
        let container_id = {
            let containers = self.containers.lock().unwrap();
            containers
                .get(app_id)
                .cloned()
                .ok_or_else(|| FoldClientError::Process(format!("App not found: {}", app_id)))?
        };

        // Get the Docker manager
        let docker_manager = self.get_docker_manager().await?;

        // Stop and remove the container
        docker_manager.stop_container(&container_id).await?;
        docker_manager.remove_container(&container_id).await?;

        // Remove the container from the map
        let mut containers = self.containers.lock().unwrap();
        containers.remove(app_id);

        Ok(())
    }

    /// Check if an app is running
    pub async fn is_app_running(&self, app_id: &str) -> Result<bool> {
        // Get the container ID
        let container_id = {
            let containers = self.containers.lock().unwrap();
            match containers.get(app_id) {
                Some(id) => id.clone(),
                None => return Ok(false),
            }
        };

        // Get the Docker manager
        let docker_manager = self.get_docker_manager().await?;

        // Check if the container is running
        docker_manager.is_container_running(&container_id).await
    }

    /// Get the list of running apps
    pub async fn list_running_apps(&self) -> Result<Vec<String>> {
        // Get the Docker manager
        let docker_manager = self.get_docker_manager().await?;

        // Get the list of containers
        let container_ids = docker_manager.list_containers().await?;

        // Map container IDs to app IDs
        let containers = self.containers.lock().unwrap();
        let app_ids = containers
            .iter()
            .filter(|(_, container_id)| container_ids.contains(container_id))
            .map(|(app_id, _)| app_id.clone())
            .collect();

        Ok(app_ids)
    }

    /// Clean up resources for terminated apps
    pub async fn cleanup(&self) -> Result<()> {
        // Get the Docker manager
        let docker_manager = self.get_docker_manager().await?;

        // Clean up containers
        docker_manager.cleanup_containers().await?;

        // Get the list of containers
        let container_ids = docker_manager.list_containers().await?;

        // Update the containers map
        {
            let mut containers = self.containers.lock().unwrap();
            // Remove containers that no longer exist
            containers.retain(|_, container_id| container_ids.contains(container_id));
        }

        Ok(())
    }
}
