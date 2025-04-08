//! FoldClient - A client for DataFold that provides Docker-sandboxed access to the node API
//!
//! FoldClient acts as a mediator between applications and the DataFold node,
//! providing Docker-sandboxed access to the node API. It handles authentication,
//! permission enforcement, and sandboxing of applications.

pub mod auth;
pub mod ipc;
pub mod node;
pub mod docker;
pub mod process;

use std::sync::Arc;

use auth::AuthManager;
use auth::AppRegistration;
use crate::ipc::server::IpcServer;
use node::NodeClient;
use node::NodeConnection;
use process::ProcessManager;

/// Result type for FoldClient operations
pub type Result<T> = std::result::Result<T, FoldClientError>;

/// Error type for FoldClient operations
#[derive(Debug, thiserror::Error)]
pub enum FoldClientError {
    #[error("Authentication error: {0}")]
    Auth(String),
    
    #[error("IPC error: {0}")]
    Ipc(String),
    
    #[error("Node communication error: {0}")]
    Node(String),
    
    #[error("Process management error: {0}")]
    Process(String),
    
    #[error("Docker error: {0}")]
    Docker(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Configuration for the FoldClient
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FoldClientConfig {
    /// Path to the DataFold node socket
    pub node_socket_path: Option<String>,
    
    /// Host and port for the DataFold node TCP connection
    pub node_tcp_address: Option<(String, u16)>,
    
    /// Path to the directory where app sockets will be created
    pub app_socket_dir: std::path::PathBuf,
    
    /// Path to the directory where app data will be stored
    pub app_data_dir: std::path::PathBuf,
    
    /// Docker configuration
    pub docker_config: DockerConfig,
    
    /// Private key content for authentication with the DataFold node
    pub private_key: Option<String>,
}

/// Docker configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DockerConfig {
    /// Docker API URL (e.g., "unix:///var/run/docker.sock" or "tcp://localhost:2375")
    pub docker_host: Option<String>,
    
    /// Docker network to use for containers
    pub network: String,
    
    /// Default CPU limit for containers (in CPU shares, where 1024 = 1 CPU)
    pub default_cpu_limit: u64,
    
    /// Default memory limit for containers (in MB)
    pub default_memory_limit: u64,
    
    /// Default storage limit for containers (in MB)
    pub default_storage_limit: u64,
    
    /// Whether to enable network access for containers by default
    pub default_allow_network: bool,
    
    /// Base image to use for containers
    pub base_image: String,
    
    /// Whether to auto-remove containers when they exit
    pub auto_remove: bool,
}

impl Default for DockerConfig {
    fn default() -> Self {
        Self {
            docker_host: None, // Use default Docker socket
            network: "bridge".to_string(),
            default_cpu_limit: 1024, // 1 CPU
            default_memory_limit: 512, // 512 MB
            default_storage_limit: 1024, // 1 GB
            default_allow_network: false,
            base_image: "alpine:latest".to_string(),
            auto_remove: true,
        }
    }
}

impl Default for FoldClientConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
        let datafold_dir = home_dir.join(".datafold");
        
        Self {
            node_socket_path: None,
            node_tcp_address: Some(("127.0.0.1".to_string(), 9000)),
            app_socket_dir: datafold_dir.join("sockets"),
            app_data_dir: datafold_dir.join("app_data"),
            docker_config: DockerConfig::default(),
            private_key: None,
        }
    }
}

/// The main FoldClient struct
pub struct FoldClient {
    /// Configuration for the FoldClient
    config: FoldClientConfig,
    
    /// Authentication manager
    auth_manager: Arc<AuthManager>,
    
    /// Node client
    node_client: Arc<NodeClient>,
    
    /// IPC server
    ipc_server: Option<IpcServer>,
    
    /// Process manager
    process_manager: Arc<ProcessManager>,
    
    /// Cleanup task
    cleanup_task: Option<tokio::task::JoinHandle<()>>,
}

impl FoldClient {
    /// Create a new FoldClient with the default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(FoldClientConfig::default())
    }
    
    /// Create a new FoldClient with the specified configuration
    pub fn with_config(config: FoldClientConfig) -> Result<Self> {
        // Create the authentication manager
        let auth_manager = Arc::new(AuthManager::new(config.app_data_dir.join("auth"))?);
        
        // Create the node client
        let node_connection = if let Some(socket_path) = &config.node_socket_path {
            NodeConnection::UnixSocket(socket_path.clone().into())
        } else if let Some((host, port)) = &config.node_tcp_address {
            NodeConnection::TcpSocket(host.clone(), *port)
        } else {
            return Err(FoldClientError::Config("No node connection specified".to_string()));
        };
        let node_client = Arc::new(NodeClient::new(node_connection, auth_manager.clone()));
        
        // Create the process manager
        let process_manager = Arc::new(ProcessManager::new(
            config.app_data_dir.clone(),
            config.docker_config.clone(),
        )?);
        
        Ok(Self {
            config,
            auth_manager,
            node_client,
            ipc_server: None,
            process_manager,
            cleanup_task: None,
        })
    }
    
    /// Start the FoldClient
    pub async fn start(&mut self) -> Result<()> {
        // Create the IPC server
        let mut ipc_server = IpcServer::new(
            self.config.app_socket_dir.clone(),
            self.auth_manager.clone(),
            self.node_client.clone(),
        )?;
        
        // Start the IPC server
        ipc_server.start().await?;
        
        // Store the IPC server
        self.ipc_server = Some(ipc_server);
        
        // Start the cleanup task
        let process_manager = self.process_manager.clone();
        let cleanup_task = tokio::task::spawn(async move {
            loop {
                // Clean up terminated containers
                let cleanup_result = process_manager.cleanup().await;
                if let Err(e) = cleanup_result {
                    log::error!("Error cleaning up containers: {}", e);
                }
                
                // Sleep for a while
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        });
        self.cleanup_task = Some(cleanup_task);
        
        Ok(())
    }
    
    /// Stop the FoldClient
    pub async fn stop(&mut self) -> Result<()> {
        // Stop the cleanup task
        if let Some(task) = self.cleanup_task.take() {
            task.abort();
        }
        
        // Stop the IPC server
        if let Some(mut ipc_server) = self.ipc_server.take() {
            ipc_server.stop().await?;
        }
        
        // Terminate all running apps
        for app_id in self.process_manager.list_running_apps().await? {
            if let Err(e) = self.process_manager.terminate_app(&app_id).await {
                log::error!("Error terminating app {}: {}", app_id, e);
            }
        }
        
        Ok(())
    }
    
    /// Register a new app
    pub async fn register_app(&mut self, app_name: &str, permissions: &[&str]) -> Result<AppRegistration> {
        // Register the app with the authentication manager
        let app = self.auth_manager.register_app(app_name, permissions)?;
        
        // Create a socket for the app
        if let Some(ref mut ipc_server) = self.ipc_server {
            let (tx, _) = tokio::sync::mpsc::channel(100);
            ipc_server.start_app_server(&app.app_id, tx).await?;
        }
        
        Ok(app)
    }
    
    /// Launch an app
    pub async fn launch_app(&self, app_id: &str, program: &str, args: &[&str]) -> Result<()> {
        // Get the app registration
        let app = self.auth_manager.get_app(app_id)?;
        
        // Launch the app
        self.process_manager.launch_app(app, program, args).await
    }
    
    /// Terminate an app
    pub async fn terminate_app(&self, app_id: &str) -> Result<()> {
        // Terminate the app
        self.process_manager.terminate_app(app_id).await
    }
    
    /// Check if an app is running
    pub async fn is_app_running(&self, app_id: &str) -> Result<bool> {
        // Check if the app is running
        self.process_manager.is_app_running(app_id).await
    }
    
    /// Get the list of running apps
    pub async fn list_running_apps(&self) -> Result<Vec<String>> {
        // Get the list of running apps
        self.process_manager.list_running_apps().await
    }
}
