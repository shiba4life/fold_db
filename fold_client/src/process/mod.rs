//! Process module for FoldClient
//!
//! This module provides functionality for managing sandboxed processes.

use crate::auth::AppRegistration;
use crate::Result;
use crate::FoldClientError;
use crate::sandbox::{Sandbox, SandboxConfig, create_sandbox};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Child;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

/// Process manager for FoldClient
pub struct ProcessManager {
    /// Directory where app data is stored
    app_data_dir: PathBuf,
    /// Whether to allow network access for apps
    allow_network: bool,
    /// Whether to allow file system access for apps
    allow_filesystem: bool,
    /// Maximum memory usage for apps (in MB)
    max_memory_mb: Option<u64>,
    /// Maximum CPU usage for apps (in percent)
    max_cpu_percent: Option<u32>,
    /// Active processes
    processes: Arc<Mutex<HashMap<String, ManagedProcess>>>,
}

/// A managed process
struct ManagedProcess {
    /// App registration
    app: AppRegistration,
    /// Child process
    child: Child,
    /// Working directory
    working_dir: PathBuf,
}

impl ProcessManager {
    /// Create a new process manager
    pub fn new(
        app_data_dir: PathBuf,
        allow_network: bool,
        allow_filesystem: bool,
        max_memory_mb: Option<u64>,
        max_cpu_percent: Option<u32>,
    ) -> Result<Self> {
        // Create the app data directory if it doesn't exist
        std::fs::create_dir_all(&app_data_dir)?;

        Ok(Self {
            app_data_dir,
            allow_network,
            allow_filesystem,
            max_memory_mb,
            max_cpu_percent,
            processes: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Launch an app
    pub fn launch_app(
        &self,
        app: AppRegistration,
        program: &str,
        args: &[&str],
    ) -> Result<()> {
        // Create the working directory for the app
        let working_dir = self.app_data_dir.join(&app.app_id);
        std::fs::create_dir_all(&working_dir)?;

        // Create environment variables for the app
        let mut env_vars = HashMap::new();
        env_vars.insert("FOLD_CLIENT_APP_ID".to_string(), app.app_id.clone());
        env_vars.insert("FOLD_CLIENT_APP_TOKEN".to_string(), app.token.clone());

        // Create the sandbox configuration
        let config = SandboxConfig {
            working_dir: working_dir.clone(),
            allow_network: self.allow_network,
            allow_filesystem: self.allow_filesystem,
            max_memory_mb: self.max_memory_mb,
            max_cpu_percent: self.max_cpu_percent,
            env_vars,
        };

        // Create the sandbox
        let sandbox = create_sandbox(config)?;

        // Launch the app in the sandbox
        let child = sandbox.run_command(program, args)?;

        // Store the process
        let mut processes = self.processes.lock().unwrap();
        processes.insert(
            app.app_id.clone(),
            ManagedProcess {
                app,
                child,
                working_dir,
            },
        );

        Ok(())
    }

    /// Terminate an app
    pub fn terminate_app(&self, app_id: &str) -> Result<()> {
        // Get the process
        let mut processes = self.processes.lock().unwrap();
        let process = processes
            .remove(app_id)
            .ok_or_else(|| FoldClientError::Process(format!("App not found: {}", app_id)))?;

        // Terminate the process
        let mut child = process.child;
        child.kill().map_err(|e| {
            FoldClientError::Process(format!("Failed to terminate app {}: {}", app_id, e))
        })?;

        Ok(())
    }

    /// Check if an app is running
    pub fn is_app_running(&self, app_id: &str) -> Result<bool> {
        // Get the process
        let processes = self.processes.lock().unwrap();
        let running = processes.contains_key(app_id);
        Ok(running)
    }

    /// Get the list of running apps
    pub fn list_running_apps(&self) -> Result<Vec<String>> {
        // Get the list of app IDs
        let processes = self.processes.lock().unwrap();
        let app_ids = processes.keys().cloned().collect();
        Ok(app_ids)
    }

    /// Clean up resources for terminated apps
    pub fn cleanup(&self) -> Result<()> {
        // Get the list of processes
        let mut processes = self.processes.lock().unwrap();

        // Check each process
        let mut terminated = Vec::new();
        for (app_id, process) in processes.iter_mut() {
            // Try to get the exit status
            match process.child.try_wait() {
                Ok(Some(_)) => {
                    // Process has terminated
                    terminated.push(app_id.clone());
                }
                Ok(None) => {
                    // Process is still running
                }
                Err(e) => {
                    // Error checking process status
                    log::error!("Error checking process status for app {}: {}", app_id, e);
                    terminated.push(app_id.clone());
                }
            }
        }

        // Remove terminated processes
        for app_id in terminated {
            processes.remove(&app_id);
        }

        Ok(())
    }
}
