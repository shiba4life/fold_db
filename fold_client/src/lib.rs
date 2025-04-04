//! FoldClient - A client for DataFold that provides sandboxed access to the node API
//!
//! FoldClient acts as a mediator between applications and the DataFold node,
//! providing sandboxed access to the node API. It handles authentication,
//! permission enforcement, and sandboxing of applications.

pub mod auth;
pub mod ipc;
pub mod node;
pub mod process;
pub mod sandbox;

use std::error::Error;
use std::fmt;
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
#[derive(Debug)]
pub enum FoldClientError {
    /// Authentication error
    Auth(String),
    /// IPC error
    Ipc(String),
    /// Node communication error
    Node(String),
    /// Process management error
    Process(String),
    /// Sandbox error
    Sandbox(String),
    /// I/O error
    Io(std::io::Error),
    /// Serialization error
    Serialization(String),
    /// Configuration error
    Config(String),
}

impl fmt::Display for FoldClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FoldClientError::Auth(msg) => write!(f, "Authentication error: {}", msg),
            FoldClientError::Ipc(msg) => write!(f, "IPC error: {}", msg),
            FoldClientError::Node(msg) => write!(f, "Node communication error: {}", msg),
            FoldClientError::Process(msg) => write!(f, "Process management error: {}", msg),
            FoldClientError::Sandbox(msg) => write!(f, "Sandbox error: {}", msg),
            FoldClientError::Io(err) => write!(f, "I/O error: {}", err),
            FoldClientError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            FoldClientError::Config(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl Error for FoldClientError {}

impl From<std::io::Error> for FoldClientError {
    fn from(err: std::io::Error) -> Self {
        FoldClientError::Io(err)
    }
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
    /// Whether to allow network access for apps
    pub allow_network_access: bool,
    /// Whether to allow file system access for apps
    pub allow_filesystem_access: bool,
    /// Maximum memory usage for apps (in MB)
    pub max_memory_mb: Option<u64>,
    /// Maximum CPU usage for apps (in percent)
    pub max_cpu_percent: Option<u32>,
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
            allow_network_access: false,
            allow_filesystem_access: false,
            max_memory_mb: Some(1024),
            max_cpu_percent: Some(50),
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
            config.allow_network_access,
            config.allow_filesystem_access,
            config.max_memory_mb,
            config.max_cpu_percent,
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
        let cleanup_task = tokio::spawn(async move {
            loop {
                // Clean up terminated processes
                if let Err(e) = process_manager.cleanup() {
                    log::error!("Error cleaning up processes: {}", e);
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
        for app_id in self.process_manager.list_running_apps()? {
            if let Err(e) = self.process_manager.terminate_app(&app_id) {
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
        self.process_manager.launch_app(app, program, args)
    }
    
    /// Terminate an app
    pub async fn terminate_app(&self, app_id: &str) -> Result<()> {
        // Terminate the app
        self.process_manager.terminate_app(app_id)
    }
    
    /// Check if an app is running
    pub async fn is_app_running(&self, app_id: &str) -> Result<bool> {
        // Check if the app is running
        self.process_manager.is_app_running(app_id)
    }
    
    /// Get the list of running apps
    pub async fn list_running_apps(&self) -> Result<Vec<String>> {
        // Get the list of running apps
        self.process_manager.list_running_apps()
    }
}
