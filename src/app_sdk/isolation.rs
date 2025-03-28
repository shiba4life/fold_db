use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Network isolation configuration for a social app container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIsolation {
    /// Container uses a separate network namespace
    pub isolated_namespace: bool,
    
    /// No direct network interfaces except loopback
    pub network_interfaces: Vec<String>,
    
    /// Firewall rules block all outbound connections
    pub firewall_rules: Vec<FirewallRule>,
}

impl NetworkIsolation {
    /// Create a new network isolation configuration with maximum isolation
    pub fn maximum() -> Self {
        Self {
            isolated_namespace: true,
            network_interfaces: vec!["lo".to_string()], // Only loopback interface
            firewall_rules: vec![
                FirewallRule::new("OUTPUT", "DROP", None), // Drop all outbound traffic
                FirewallRule::new("OUTPUT", "ACCEPT", Some("lo")), // Allow loopback traffic
            ],
        }
    }

    /// Create a new network isolation configuration with no isolation
    pub fn none() -> Self {
        Self {
            isolated_namespace: false,
            network_interfaces: vec![],
            firewall_rules: vec![],
        }
    }

    /// Add a firewall rule
    pub fn add_firewall_rule(mut self, rule: FirewallRule) -> Self {
        self.firewall_rules.push(rule);
        self
    }

    /// Add a network interface
    pub fn add_network_interface(mut self, interface: &str) -> Self {
        self.network_interfaces.push(interface.to_string());
        self
    }
}

/// Firewall rule for network isolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    /// Chain (INPUT, OUTPUT, FORWARD)
    pub chain: String,
    
    /// Action (ACCEPT, DROP, REJECT)
    pub action: String,
    
    /// Interface (optional)
    pub interface: Option<String>,
}

impl FirewallRule {
    /// Create a new firewall rule
    pub fn new(chain: &str, action: &str, interface: Option<&str>) -> Self {
        Self {
            chain: chain.to_string(),
            action: action.to_string(),
            interface: interface.map(|s| s.to_string()),
        }
    }
}

/// MicroVM configuration for app isolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroVMConfig {
    /// VM type (Firecracker, Kata Containers, etc.)
    pub vm_type: MicroVMType,
    
    /// VM resources
    pub vcpu_count: u32,
    pub memory_mb: u32,
    
    /// Network configuration - only allow communication with host
    pub network_config: NetworkConfig,
    
    /// Vsock for communication with host
    pub vsock_config: VsockConfig,
    
    /// Kernel parameters to restrict networking
    pub kernel_params: Vec<String>,
    
    /// Root filesystem
    pub root_fs: String,
    
    /// Additional drives
    pub additional_drives: Vec<DriveConfig>,
}

impl MicroVMConfig {
    /// Create a new MicroVM configuration with default values
    pub fn new(vm_type: MicroVMType, root_fs: &str) -> Self {
        Self {
            vm_type,
            vcpu_count: 1,
            memory_mb: 128,
            network_config: NetworkConfig::none(),
            vsock_config: VsockConfig::default(),
            kernel_params: vec![
                "console=ttyS0".to_string(),
                "reboot=k".to_string(),
                "panic=1".to_string(),
                "pci=off".to_string(),
                "nomodules".to_string(),
                "ip=none".to_string(),
            ],
            root_fs: root_fs.to_string(),
            additional_drives: vec![],
        }
    }

    /// Set the number of vCPUs
    pub fn with_vcpu_count(mut self, count: u32) -> Self {
        self.vcpu_count = count;
        self
    }

    /// Set the memory size in MB
    pub fn with_memory_mb(mut self, memory: u32) -> Self {
        self.memory_mb = memory;
        self
    }

    /// Set the network configuration
    pub fn with_network_config(mut self, config: NetworkConfig) -> Self {
        self.network_config = config;
        self
    }

    /// Set the vsock configuration
    pub fn with_vsock_config(mut self, config: VsockConfig) -> Self {
        self.vsock_config = config;
        self
    }

    /// Add a kernel parameter
    pub fn add_kernel_param(mut self, param: &str) -> Self {
        self.kernel_params.push(param.to_string());
        self
    }

    /// Add an additional drive
    pub fn add_drive(mut self, drive: DriveConfig) -> Self {
        self.additional_drives.push(drive);
        self
    }
}

/// MicroVM types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MicroVMType {
    Firecracker,
    KataContainers,
    Cloud9VM,
    Weave,
}

/// Network configuration for MicroVM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Network interfaces
    pub interfaces: Vec<NetworkInterface>,
}

impl NetworkConfig {
    /// Create a network configuration with no interfaces
    pub fn none() -> Self {
        Self {
            interfaces: vec![],
        }
    }

    /// Add a network interface
    pub fn add_interface(mut self, interface: NetworkInterface) -> Self {
        self.interfaces.push(interface);
        self
    }
}

/// Network interface configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// Interface name
    pub name: String,
    
    /// Interface type
    pub interface_type: NetworkInterfaceType,
    
    /// MAC address
    pub mac_address: Option<String>,
    
    /// Host device (for host-device type)
    pub host_device: Option<String>,
}

/// Network interface types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkInterfaceType {
    Tap,
    Macvtap,
    HostDevice,
}

/// Vsock configuration for MicroVM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsockConfig {
    /// Vsock ID
    pub vsock_id: String,
    
    /// Guest CID
    pub guest_cid: u32,
    
    /// Unix domain socket path
    pub uds_path: String,
}

impl Default for VsockConfig {
    fn default() -> Self {
        Self {
            vsock_id: "vsock-default".to_string(),
            guest_cid: 3,
            uds_path: "/tmp/firecracker-vsock.sock".to_string(),
        }
    }
}

/// Drive configuration for MicroVM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveConfig {
    /// Path on host
    pub path_on_host: String,
    
    /// Is read-only
    pub is_read_only: bool,
    
    /// Is root device
    pub is_root_device: bool,
}

impl DriveConfig {
    /// Create a new drive configuration
    pub fn new(path: &str, read_only: bool, root_device: bool) -> Self {
        Self {
            path_on_host: path.to_string(),
            is_read_only: read_only,
            is_root_device: root_device,
        }
    }
}

/// Linux container configuration for app isolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinuxContainerConfig {
    /// Use network namespace isolation
    pub network_namespace: bool,
    
    /// Drop all network capabilities except local communication
    pub dropped_capabilities: Vec<String>,
    
    /// Only allow communication with the DataFold node socket
    pub allowed_sockets: Vec<SocketPath>,
    
    /// Seccomp filters to block network-related syscalls
    pub seccomp_filters: Vec<SeccompRule>,
    
    /// Resource limits
    pub resource_limits: ResourceLimits,
    
    /// Mount points
    pub mounts: Vec<MountPoint>,
}

impl LinuxContainerConfig {
    /// Create a new Linux container configuration with maximum isolation
    pub fn maximum() -> Self {
        Self {
            network_namespace: true,
            dropped_capabilities: vec![
                "NET_ADMIN".to_string(),
                "NET_RAW".to_string(),
                "NET_BROADCAST".to_string(),
            ],
            allowed_sockets: vec![],
            seccomp_filters: vec![],
            resource_limits: ResourceLimits::default(),
            mounts: vec![],
        }
    }

    /// Add an allowed socket
    pub fn add_allowed_socket(mut self, socket: SocketPath) -> Self {
        self.allowed_sockets.push(socket);
        self
    }

    /// Add a seccomp filter
    pub fn add_seccomp_filter(mut self, filter: SeccompRule) -> Self {
        self.seccomp_filters.push(filter);
        self
    }

    /// Set resource limits
    pub fn with_resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.resource_limits = limits;
        self
    }

    /// Add a mount point
    pub fn add_mount(mut self, mount: MountPoint) -> Self {
        self.mounts.push(mount);
        self
    }
}

/// Socket path for container configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketPath {
    /// Path on host
    pub host_path: String,
    
    /// Path in container
    pub container_path: String,
}

impl SocketPath {
    /// Create a new socket path
    pub fn new(host_path: &str, container_path: &str) -> Self {
        Self {
            host_path: host_path.to_string(),
            container_path: container_path.to_string(),
        }
    }
}

/// Seccomp rule for container configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeccompRule {
    /// Syscall name
    pub syscall: String,
    
    /// Action (ALLOW, DENY, KILL)
    pub action: SeccompAction,
}

impl SeccompRule {
    /// Create a new seccomp rule
    pub fn new(syscall: &str, action: SeccompAction) -> Self {
        Self {
            syscall: syscall.to_string(),
            action,
        }
    }
}

/// Seccomp actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SeccompAction {
    Allow,
    Deny,
    Kill,
}

/// Resource limits for container configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum CPU usage percentage
    pub max_cpu_percent: f32,
    
    /// Maximum memory usage in MB
    pub max_memory_mb: u32,
    
    /// Maximum storage in MB
    pub max_storage_mb: u32,
    
    /// Maximum number of concurrent operations
    pub max_concurrent_ops: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_percent: 50.0,
            max_memory_mb: 256,
            max_storage_mb: 1024,
            max_concurrent_ops: 10,
        }
    }
}

/// Mount point for container configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountPoint {
    /// Source path on host
    pub source: String,
    
    /// Target path in container
    pub target: String,
    
    /// Is read-only
    pub read_only: bool,
}

impl MountPoint {
    /// Create a new mount point
    pub fn new(source: &str, target: &str, read_only: bool) -> Self {
        Self {
            source: source.to_string(),
            target: target.to_string(),
            read_only,
        }
    }
}

/// WebAssembly sandbox configuration for app isolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmSandboxConfig {
    /// No network access in WASM imports
    pub network_imports: Vec<String>,
    
    /// Only allow communication through provided DataFold API
    pub allowed_imports: Vec<String>,
    
    /// Memory isolation
    pub memory_isolation: bool,
    
    /// Resource limits
    pub resource_limits: ResourceLimits,
}

impl WasmSandboxConfig {
    /// Create a new WebAssembly sandbox configuration with maximum isolation
    pub fn maximum() -> Self {
        Self {
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
        }
    }

    /// Add an allowed import
    pub fn add_allowed_import(mut self, import: &str) -> Self {
        self.allowed_imports.push(import.to_string());
        self
    }

    /// Set resource limits
    pub fn with_resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.resource_limits = limits;
        self
    }
}
