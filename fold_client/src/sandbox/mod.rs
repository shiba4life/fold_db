//! Sandbox module for FoldClient
//!
//! This module provides platform-specific sandbox implementations for isolating
//! applications from the system.

use crate::Result;
use std::process::Child;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::LinuxSandbox as PlatformSandbox;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::MacOSSandbox as PlatformSandbox;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsSandbox as PlatformSandbox;

/// Configuration for the sandbox
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Working directory for the sandboxed process
    pub working_dir: std::path::PathBuf,
    /// Whether to allow network access
    pub allow_network: bool,
    /// Whether to allow file system access
    pub allow_filesystem: bool,
    /// Maximum memory usage in MB
    pub max_memory_mb: Option<u64>,
    /// Maximum CPU usage in percent
    pub max_cpu_percent: Option<u32>,
    /// Environment variables to set
    pub env_vars: std::collections::HashMap<String, String>,
}

/// Sandbox trait for platform-specific sandbox implementations
pub trait Sandbox {
    /// Create a new sandbox with the specified configuration
    fn new(config: SandboxConfig) -> Result<Self>
    where
        Self: Sized;

    /// Run a command in the sandbox
    fn run_command(&self, program: &str, args: &[&str]) -> Result<Child>;
}

/// Create a new sandbox for the current platform
pub fn create_sandbox(config: SandboxConfig) -> Result<PlatformSandbox> {
    PlatformSandbox::new(config)
}
