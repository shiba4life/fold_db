/// Configuration for the network layer
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Local listening address
    pub listen_address: String,
    /// Request timeout in seconds
    pub request_timeout: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_address: "/ip4/0.0.0.0/tcp/0".to_string(),
            request_timeout: 30,
        }
    }
}
