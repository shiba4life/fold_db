use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

use crate::error::{AppSdkError, AppSdkResult};
use crate::isolation::{
    NetworkIsolation, MicroVMConfig, LinuxContainerConfig, WasmSandboxConfig,
    ResourceLimits, MicroVMType, DriveConfig, VsockConfig, SocketPath, MountPoint
};
use crate::permissions::AppPermissions;
use crate::types::NodeConnection;

/// Container for a social app
#[derive(Debug)]
pub struct SocialAppContainer {
    /// Unique identifier for this app instance
    pub app_id: String,
    
    /// Public key for authentication
    pub app_public_key: String,
    
    /// Permissions granted to this app
    pub permissions: AppPermissions,
    
    /// Connection to the DataFold node (ONLY communication channel)
    pub node_connection: NodeConnection,
    
    /// Network isolation configuration
    pub network_isolation: NetworkIsolation,
    
    /// Resource limits
    pub resource_limits: ResourceLimits,
    
    /// Container status
    pub status: ContainerStatus,
    
    /// Container configuration
    pub config: ContainerConfig,
    
    /// Container process ID
    pub process_id: Option<u32>,
}

impl SocialAppContainer {
    /// Create a new social app container
    pub fn new(
        app_id: &str,
        app_public_key: &str,
        permissions: AppPermissions,
        config: ContainerConfig,
    ) -> Self {
        // Create a default node connection using a Unix socket
        let socket_path = format!("/var/run/datafold/app_{}.sock", app_id);
        let node_connection = NodeConnection::UnixSocket(socket_path);
        
        // Create default network isolation
        let network_isolation = NetworkIsolation::maximum();
        
        // Create default resource limits
        let resource_limits = ResourceLimits::default();
        
        Self {
            app_id: app_id.to_string(),
            app_public_key: app_public_key.to_string(),
            permissions,
            node_connection,
            network_isolation,
            resource_limits,
            status: ContainerStatus::Created,
            config,
            process_id: None,
        }
    }

    /// Start the container
    pub async fn start(&mut self) -> AppSdkResult<()> {
        // Check if the container is already running
        if self.status == ContainerStatus::Running {
            return Err(AppSdkError::Container("Container is already running".to_string()));
        }
        
        // Clone the isolation type to avoid borrowing issues
        let isolation_type = self.config.isolation_type.clone();
        
        // Start the container based on the isolation type
        match isolation_type {
            IsolationType::MicroVM(vm_config) => {
                self.start_microvm(&vm_config).await?;
            },
            IsolationType::LinuxContainer(container_config) => {
                self.start_linux_container(&container_config).await?;
            },
            IsolationType::WasmSandbox(wasm_config) => {
                self.start_wasm_sandbox(&wasm_config).await?;
            },
        }
        
        // Update the status
        self.status = ContainerStatus::Running;
        
        Ok(())
    }

    /// Stop the container
    pub async fn stop(&mut self) -> AppSdkResult<()> {
        // Check if the container is running
        if self.status != ContainerStatus::Running {
            return Err(AppSdkError::Container("Container is not running".to_string()));
        }
        
        // Clone the isolation type to avoid borrowing issues
        let isolation_type = self.config.isolation_type.clone();
        
        // Stop the container based on the isolation type
        match isolation_type {
            IsolationType::MicroVM(_) => {
                self.stop_microvm().await?;
            },
            IsolationType::LinuxContainer(_) => {
                self.stop_linux_container().await?;
            },
            IsolationType::WasmSandbox(_) => {
                self.stop_wasm_sandbox().await?;
            },
        }
        
        // Update the status
        self.status = ContainerStatus::Stopped;
        
        Ok(())
    }

    /// Start a MicroVM
    async fn start_microvm(&mut self, vm_config: &MicroVMConfig) -> AppSdkResult<()> {
        // In a real implementation, this would start a MicroVM using the specified configuration
        // For now, we'll just log that we're starting a MicroVM
        println!("Starting MicroVM for app {}", self.app_id);
        println!("VM type: {:?}", vm_config.vm_type);
        println!("vCPUs: {}", vm_config.vcpu_count);
        println!("Memory: {} MB", vm_config.memory_mb);
        
        // Set a dummy process ID
        self.process_id = Some(1000);
        
        Ok(())
    }

    /// Stop a MicroVM
    async fn stop_microvm(&mut self) -> AppSdkResult<()> {
        // In a real implementation, this would stop the MicroVM
        // For now, we'll just log that we're stopping the MicroVM
        println!("Stopping MicroVM for app {}", self.app_id);
        
        // Clear the process ID
        self.process_id = None;
        
        Ok(())
    }

    /// Start a Linux container
    async fn start_linux_container(&mut self, _container_config: &LinuxContainerConfig) -> AppSdkResult<()> {
        // In a real implementation, this would start a Linux container using the specified configuration
        // For now, we'll just log that we're starting a Linux container
        println!("Starting Linux container for app {}", self.app_id);
        // println!("Network namespace: {}", _container_config.network_namespace);
        // println!("Dropped capabilities: {:?}", _container_config.dropped_capabilities);
        
        // Set a dummy process ID
        self.process_id = Some(2000);
        
        Ok(())
    }

    /// Stop a Linux container
    async fn stop_linux_container(&mut self) -> AppSdkResult<()> {
        // In a real implementation, this would stop the Linux container
        // For now, we'll just log that we're stopping the Linux container
        println!("Stopping Linux container for app {}", self.app_id);
        
        // Clear the process ID
        self.process_id = None;
        
        Ok(())
    }

    /// Start a WebAssembly sandbox
    async fn start_wasm_sandbox(&mut self, _wasm_config: &WasmSandboxConfig) -> AppSdkResult<()> {
        // In a real implementation, this would start a WebAssembly sandbox using the specified configuration
        // For now, we'll just log that we're starting a WebAssembly sandbox
        println!("Starting WebAssembly sandbox for app {}", self.app_id);
        // println!("Memory isolation: {}", _wasm_config.memory_isolation);
        // println!("Allowed imports: {:?}", _wasm_config.allowed_imports);
        
        // Set a dummy process ID
        self.process_id = Some(3000);
        
        Ok(())
    }

    /// Stop a WebAssembly sandbox
    async fn stop_wasm_sandbox(&mut self) -> AppSdkResult<()> {
        // In a real implementation, this would stop the WebAssembly sandbox
        // For now, we'll just log that we're stopping the WebAssembly sandbox
        println!("Stopping WebAssembly sandbox for app {}", self.app_id);
        
        // Clear the process ID
        self.process_id = None;
        
        Ok(())
    }

    /// Get the container status
    pub fn get_status(&self) -> ContainerStatus {
        self.status
    }

    /// Check if the container is running
    pub fn is_running(&self) -> bool {
        self.status == ContainerStatus::Running
    }

    /// Get the container's resource usage
    pub fn get_resource_usage(&self) -> AppSdkResult<ResourceUsage> {
        // In a real implementation, this would get the actual resource usage
        // For now, we'll just return dummy values
        Ok(ResourceUsage {
            cpu_percent: 10.0,
            memory_mb: 50,
            storage_mb: 100,
            concurrent_ops: 2,
        })
    }
}

/// Container status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerStatus {
    Created,
    Running,
    Stopped,
    Failed,
}

/// Container configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    /// App binary path
    pub app_binary_path: PathBuf,
    
    /// Isolation type
    pub isolation_type: IsolationType,
    
    /// Environment variables
    pub env_vars: HashMap<String, String>,
    
    /// Working directory
    pub working_dir: PathBuf,
    
    /// Command-line arguments
    pub args: Vec<String>,
}

impl ContainerConfig {
    /// Create a new container configuration with MicroVM isolation
    pub fn new_microvm(
        app_binary_path: PathBuf,
        vm_config: MicroVMConfig,
    ) -> Self {
        Self {
            app_binary_path,
            isolation_type: IsolationType::MicroVM(vm_config),
            env_vars: HashMap::new(),
            working_dir: PathBuf::from("/app"),
            args: vec![],
        }
    }

    /// Create a new container configuration with Linux container isolation
    pub fn new_linux_container(
        app_binary_path: PathBuf,
        container_config: LinuxContainerConfig,
    ) -> Self {
        Self {
            app_binary_path,
            isolation_type: IsolationType::LinuxContainer(container_config),
            env_vars: HashMap::new(),
            working_dir: PathBuf::from("/app"),
            args: vec![],
        }
    }

    /// Create a new container configuration with WebAssembly sandbox isolation
    pub fn new_wasm_sandbox(
        app_binary_path: PathBuf,
        wasm_config: WasmSandboxConfig,
    ) -> Self {
        Self {
            app_binary_path,
            isolation_type: IsolationType::WasmSandbox(wasm_config),
            env_vars: HashMap::new(),
            working_dir: PathBuf::from("/app"),
            args: vec![],
        }
    }

    /// Add an environment variable
    pub fn add_env_var(mut self, key: &str, value: &str) -> Self {
        self.env_vars.insert(key.to_string(), value.to_string());
        self
    }

    /// Set the working directory
    pub fn with_working_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = dir;
        self
    }

    /// Add a command-line argument
    pub fn add_arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    /// Add multiple command-line arguments
    pub fn add_args(mut self, args: &[&str]) -> Self {
        for arg in args {
            self.args.push(arg.to_string());
        }
        self
    }
}

/// Isolation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IsolationType {
    MicroVM(MicroVMConfig),
    LinuxContainer(LinuxContainerConfig),
    WasmSandbox(WasmSandboxConfig),
}

/// Resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU usage percentage
    pub cpu_percent: f32,
    
    /// Memory usage in MB
    pub memory_mb: u32,
    
    /// Storage usage in MB
    pub storage_mb: u32,
    
    /// Number of concurrent operations
    pub concurrent_ops: u32,
}

/// Helper function to create a Firecracker VM
pub async fn create_firecracker_vm(
    app_id: &str,
    app_binary: &str,
) -> AppSdkResult<String> {
    // In a real implementation, this would create a Firecracker VM
    // For now, we'll just log that we're creating a VM and return a dummy VM ID
    println!("Creating Firecracker VM for app {}", app_id);
    println!("App binary: {}", app_binary);
    
    // Create a VM configuration
    let _vm_config = MicroVMConfig {
        vm_type: MicroVMType::Firecracker,
        vcpu_count: 1,
        memory_mb: 128,
        network_config: crate::isolation::NetworkConfig::none(),
        vsock_config: VsockConfig {
            vsock_id: format!("vsock-{}", app_id),
            guest_cid: 3,
            uds_path: format!("/tmp/firecracker-{}.sock", app_id),
        },
        kernel_params: vec![
            "console=ttyS0".to_string(),
            "reboot=k".to_string(),
            "panic=1".to_string(),
            "pci=off".to_string(),
            "nomodules".to_string(),
            "ip=none".to_string(),
        ],
        root_fs: "/var/lib/datafold/vm-images/minimal-rootfs.ext4".to_string(),
        additional_drives: vec![
            DriveConfig {
                path_on_host: format!("/var/lib/datafold/apps/{}/drive.ext4", app_id),
                is_read_only: false,
                is_root_device: false,
            },
        ],
    };
    
    // Return a dummy VM ID
    Ok(format!("vm-{}", app_id))
}

/// Helper function to create a Kata container
pub async fn create_kata_container(
    app_id: &str,
    app_binary: &str,
) -> AppSdkResult<String> {
    // In a real implementation, this would create a Kata container
    // For now, we'll just log that we're creating a container and return a dummy container ID
    println!("Creating Kata container for app {}", app_id);
    println!("App binary: {}", app_binary);
    
    // Create a container configuration
    let _container_config = ContainerConfig {
        app_binary_path: PathBuf::from(app_binary),
        isolation_type: IsolationType::MicroVM(MicroVMConfig {
            vm_type: MicroVMType::KataContainers,
            vcpu_count: 1,
            memory_mb: 128,
            network_config: crate::isolation::NetworkConfig::none(),
            vsock_config: VsockConfig::default(),
            kernel_params: vec![
                "console=ttyS0".to_string(),
                "reboot=k".to_string(),
                "panic=1".to_string(),
                "pci=off".to_string(),
                "nomodules".to_string(),
                "ip=none".to_string(),
            ],
            root_fs: "/var/lib/datafold/vm-images/minimal-rootfs.ext4".to_string(),
            additional_drives: vec![],
        }),
        env_vars: HashMap::new(),
        working_dir: PathBuf::from("/app"),
        args: vec![],
    };
    
    // Return a dummy container ID
    Ok(format!("container-{}", app_id))
}

/// Helper function to create an isolated container
pub async fn create_isolated_container(
    app_id: &str,
    app_binary: &str,
) -> AppSdkResult<String> {
    // In a real implementation, this would create an isolated container
    // For now, we'll just log that we're creating a container and return a dummy container ID
    println!("Creating isolated container for app {}", app_id);
    println!("App binary: {}", app_binary);
    
    // Create a container configuration
    let _container_config = ContainerConfig {
        app_binary_path: PathBuf::from(app_binary),
        isolation_type: IsolationType::LinuxContainer(LinuxContainerConfig {
            network_namespace: true,
            dropped_capabilities: vec![
                "NET_ADMIN".to_string(),
                "NET_RAW".to_string(),
                "NET_BROADCAST".to_string(),
            ],
            allowed_sockets: vec![
                SocketPath {
                    host_path: format!("/var/run/datafold/app_{}.sock", app_id),
                    container_path: "/var/run/datafold/node.sock".to_string(),
                },
            ],
            seccomp_filters: vec![],
            resource_limits: ResourceLimits::default(),
            mounts: vec![
                MountPoint {
                    source: app_binary.to_string(),
                    target: "/app/binary".to_string(),
                    read_only: true,
                },
            ],
        }),
        env_vars: HashMap::new(),
        working_dir: PathBuf::from("/app"),
        args: vec![],
    };
    
    // Return a dummy container ID
    Ok(format!("container-{}", app_id))
}

/// Helper function to create a WebAssembly sandbox
pub async fn create_wasm_sandbox(
    app_id: &str,
    wasm_module: &[u8],
) -> AppSdkResult<String> {
    // In a real implementation, this would create a WebAssembly sandbox
    // For now, we'll just log that we're creating a sandbox and return a dummy sandbox ID
    println!("Creating WebAssembly sandbox for app {}", app_id);
    println!("WASM module size: {} bytes", wasm_module.len());
    
    // Create a sandbox configuration
    let _sandbox_config = WasmSandboxConfig {
        network_imports: vec![],
        allowed_imports: vec![
            "query_local".to_string(),
            "query_remote".to_string(),
            "mutate_local".to_string(),
            "mutate_remote".to_string(),
            "discover_nodes".to_string(),
            "discover_schemas".to_string(),
        ],
        memory_isolation: true,
        resource_limits: ResourceLimits::default(),
    };
    
    // Return a dummy sandbox ID
    Ok(format!("sandbox-{}", app_id))
}
