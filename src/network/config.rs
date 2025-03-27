/// Configuration for the network layer
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Local listening address
    pub listen_address: String,
    /// Request timeout in seconds
    pub request_timeout: u64,
    /// Enable mDNS discovery
    pub enable_mdns: bool,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Connection keep-alive interval in seconds
    pub keep_alive_interval: u64,
    /// Maximum message size in bytes
    pub max_message_size: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_address: "/ip4/0.0.0.0/tcp/0".to_string(),
            request_timeout: 30,
            enable_mdns: true,
            max_connections: 50,
            keep_alive_interval: 20,
            max_message_size: 1_000_000, // 1MB
        }
    }
}

impl NetworkConfig {
    /// Create a new network configuration with the specified listen address
    pub fn new(listen_address: &str) -> Self {
        let mut config = Self::default();
        config.listen_address = listen_address.to_string();
        config
    }
    
    /// Set the request timeout in seconds
    pub fn with_request_timeout(mut self, timeout: u64) -> Self {
        self.request_timeout = timeout;
        self
    }
    
    /// Enable or disable mDNS discovery
    pub fn with_mdns(mut self, enable: bool) -> Self {
        self.enable_mdns = enable;
        self
    }
    
    /// Set the maximum number of concurrent connections
    pub fn with_max_connections(mut self, max: usize) -> Self {
        self.max_connections = max;
        self
    }
    
    /// Set the connection keep-alive interval in seconds
    pub fn with_keep_alive_interval(mut self, interval: u64) -> Self {
        self.keep_alive_interval = interval;
        self
    }
    
    /// Set the maximum message size in bytes
    pub fn with_max_message_size(mut self, size: usize) -> Self {
        self.max_message_size = size;
        self
    }
}
