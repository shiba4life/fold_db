use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::tempdir;

use fold_client::sandbox::{Sandbox, SandboxConfig};

#[cfg(target_os = "macos")]
use fold_client::sandbox::PlatformSandbox as MacOSSandbox;

#[cfg(target_os = "linux")]
use fold_client::sandbox::PlatformSandbox as LinuxSandbox;

#[cfg(target_os = "windows")]
use fold_client::sandbox::PlatformSandbox as WindowsSandbox;

// Helper function to create a temporary directory for testing
fn create_temp_sandbox_dir() -> (PathBuf, tempfile::TempDir) {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let sandbox_dir = temp_dir.path().to_path_buf();
    (sandbox_dir, temp_dir)
}

#[test]
fn test_sandbox_config_creation() {
    let (working_dir, _temp_dir) = create_temp_sandbox_dir();
    
    let mut env_vars = HashMap::new();
    env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());
    
    let config = SandboxConfig {
        working_dir: working_dir.clone(),
        allow_network: false,
        allow_filesystem: true,
        max_memory_mb: Some(1024),
        max_cpu_percent: Some(50),
        env_vars,
    };
    
    assert_eq!(config.working_dir, working_dir, "Working directory does not match");
    assert_eq!(config.allow_network, false, "Network access flag does not match");
    assert_eq!(config.allow_filesystem, true, "File system access flag does not match");
    assert_eq!(config.max_memory_mb, Some(1024), "Memory limit does not match");
    assert_eq!(config.max_cpu_percent, Some(50), "CPU limit does not match");
    assert_eq!(config.env_vars.get("TEST_VAR"), Some(&"test_value".to_string()), "Environment variable does not match");
}

#[test]
#[cfg(target_os = "macos")]
fn test_macos_sandbox_creation() {
    let (working_dir, _temp_dir) = create_temp_sandbox_dir();
    
    let config = SandboxConfig {
        working_dir,
        allow_network: false,
        allow_filesystem: true,
        max_memory_mb: Some(1024),
        max_cpu_percent: Some(50),
        env_vars: HashMap::new(),
    };
    
    let sandbox = MacOSSandbox::new(config);
    assert!(sandbox.is_ok(), "Failed to create macOS sandbox");
}

#[test]
#[cfg(target_os = "linux")]
fn test_linux_sandbox_creation() {
    let (working_dir, _temp_dir) = create_temp_sandbox_dir();
    
    let config = SandboxConfig {
        working_dir,
        allow_network: false,
        allow_filesystem: true,
        max_memory_mb: Some(1024),
        max_cpu_percent: Some(50),
        env_vars: HashMap::new(),
    };
    
    let sandbox = LinuxSandbox::new(config);
    assert!(sandbox.is_ok(), "Failed to create Linux sandbox");
}

#[test]
#[cfg(target_os = "windows")]
fn test_windows_sandbox_creation() {
    let (working_dir, _temp_dir) = create_temp_sandbox_dir();
    
    let config = SandboxConfig {
        working_dir,
        allow_network: false,
        allow_filesystem: true,
        max_memory_mb: Some(1024),
        max_cpu_percent: Some(50),
        env_vars: HashMap::new(),
    };
    
    let sandbox = WindowsSandbox::new(config);
    assert!(sandbox.is_ok(), "Failed to create Windows sandbox");
}

#[test]
fn test_sandbox_run_command_api() {
    let (working_dir, _temp_dir) = create_temp_sandbox_dir();
    
    let config = SandboxConfig {
        working_dir,
        allow_network: false,
        allow_filesystem: true,
        max_memory_mb: Some(1024),
        max_cpu_percent: Some(50),
        env_vars: HashMap::new(),
    };
    
    #[cfg(target_os = "macos")]
    let sandbox = MacOSSandbox::new(config).expect("Failed to create macOS sandbox");
    
    #[cfg(target_os = "linux")]
    let sandbox = LinuxSandbox::new(config).expect("Failed to create Linux sandbox");
    
    #[cfg(target_os = "windows")]
    let sandbox = WindowsSandbox::new(config).expect("Failed to create Windows sandbox");
    
    // Test that the run_command API works without actually executing the command
    // This is a more unit-test approach that doesn't depend on the actual sandbox execution
    
    #[cfg(unix)]
    let result = sandbox.run_command("echo", &["Hello, World!"]);
    
    #[cfg(windows)]
    let result = sandbox.run_command("cmd", &["/c", "echo", "Hello, World!"]);
    
    // We only check that the API returns a Result<Child> without error
    assert!(result.is_ok(), "Failed to run command in sandbox");
}

#[test]
fn test_sandbox_with_env_vars_api() {
    let (working_dir, _temp_dir) = create_temp_sandbox_dir();
    
    let mut env_vars = HashMap::new();
    env_vars.insert("TEST_ENV_VAR".to_string(), "test_value".to_string());
    
    let config = SandboxConfig {
        working_dir,
        allow_network: false,
        allow_filesystem: true,
        max_memory_mb: Some(1024),
        max_cpu_percent: Some(50),
        env_vars,
    };
    
    #[cfg(target_os = "macos")]
    let sandbox = MacOSSandbox::new(config).expect("Failed to create macOS sandbox");
    
    #[cfg(target_os = "linux")]
    let sandbox = LinuxSandbox::new(config).expect("Failed to create Linux sandbox");
    
    #[cfg(target_os = "windows")]
    let sandbox = WindowsSandbox::new(config).expect("Failed to create Windows sandbox");
    
    // Test that the run_command API works with environment variables
    // without actually executing the command
    
    #[cfg(unix)]
    let result = sandbox.run_command("sh", &["-c", "echo $TEST_ENV_VAR"]);
    
    #[cfg(windows)]
    let result = sandbox.run_command("cmd", &["/c", "echo", "%TEST_ENV_VAR%"]);
    
    // We only check that the API returns a Result<Child> without error
    assert!(result.is_ok(), "Failed to run command in sandbox");
}
