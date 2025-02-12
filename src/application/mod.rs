//! Application layer module for DataFold Node
//! Provides Unix Domain Socket based communication for containerized applications

mod client;
mod server;
mod types;

pub use client::DataFoldClient;
pub use server::SocketServer;
pub use types::{ApiRequest, ApiResponse, ClientError, ResponseStatus, SocketConfig};

use std::path::PathBuf;
use std::time::Duration;

/// Default socket path
pub const DEFAULT_SOCKET_PATH: &str = "/var/run/datafold/datafold.sock";
/// Default socket permissions (660 - rw-rw----)
pub const DEFAULT_SOCKET_PERMISSIONS: u32 = 0o660;
/// Default buffer size for socket operations
pub const DEFAULT_BUFFER_SIZE: usize = 8192;
/// Default timeout for client operations
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Configuration for client connections
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Path to the Unix Domain Socket
    pub socket_path: PathBuf,
    /// Timeout for operations
    pub timeout: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            socket_path: PathBuf::from(DEFAULT_SOCKET_PATH),
            timeout: DEFAULT_TIMEOUT,
        }
    }
}
