// Sandbox module for secure Docker container management
//
// This module provides functionality for running third-party Docker containers
// in a secure, isolated environment with restricted access to the Datafold API.

mod manager;
mod security;
mod config;
mod types;

pub use manager::SandboxManager;
pub use security::SecurityMiddleware;
pub use config::SandboxConfig;
pub use types::{ContainerInfo, ResourceLimits, SandboxError, SandboxResult, Request, Response, ContainerStatus, OperationType};
