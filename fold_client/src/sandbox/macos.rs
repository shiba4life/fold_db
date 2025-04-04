//! macOS-specific sandbox implementation
//!
//! This module provides a sandbox implementation for macOS using the sandbox-exec command.

use crate::Result;
use crate::FoldClientError;
use crate::sandbox::{Sandbox, SandboxConfig};
use std::fs::File;
use std::io::Write;
use std::process::{Child, Command, Stdio};

/// macOS sandbox implementation
pub struct MacOSSandbox {
    /// Sandbox configuration
    config: SandboxConfig,
    /// Path to the sandbox profile
    profile_path: std::path::PathBuf,
}

impl Sandbox for MacOSSandbox {
    fn new(config: SandboxConfig) -> Result<Self> {
        // Create a temporary directory for the sandbox profile
        let profile_dir = std::env::temp_dir().join("fold_client_sandbox");
        std::fs::create_dir_all(&profile_dir)?;

        // Create a unique profile path
        let profile_path = profile_dir.join(format!("sandbox_{}.sb", std::process::id()));

        // Create the sandbox
        let sandbox = Self {
            config,
            profile_path,
        };

        // Generate the sandbox profile
        sandbox.generate_profile()?;

        Ok(sandbox)
    }

    fn run_command(&self, program: &str, args: &[&str]) -> Result<Child> {
        // Create the command
        let mut command = Command::new("sandbox-exec");
        command
            .arg("-f")
            .arg(&self.profile_path)
            .arg(program)
            .args(args)
            .current_dir(&self.config.working_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set environment variables
        for (key, value) in &self.config.env_vars {
            command.env(key, value);
        }

        // Set resource limits
        if let Some(memory_mb) = self.config.max_memory_mb {
            command.env("MEMMON_LIMIT", memory_mb.to_string());
        }

        if let Some(cpu_percent) = self.config.max_cpu_percent {
            command.env("CPUMON_LIMIT", cpu_percent.to_string());
        }

        // Launch the command
        let child = command
            .spawn()
            .map_err(|e| FoldClientError::Sandbox(format!("Failed to launch command: {}", e)))?;

        Ok(child)
    }
}

impl MacOSSandbox {
    /// Generate a sandbox profile for the application
    fn generate_profile(&self) -> Result<()> {
        // Create the sandbox profile
        let mut profile = String::new();

        // Add the version
        profile.push_str("(version 1)\n");

        // Add default rules
        profile.push_str("(allow default)\n");

        // Network access rules
        if !self.config.allow_network {
            profile.push_str("(deny network*)\n");
        }

        // File system access rules
        if !self.config.allow_filesystem {
            // Allow access to the working directory
            profile.push_str(&format!(
                "(allow file-read* file-write* (subpath \"{}\"))\n",
                self.config.working_dir.to_string_lossy()
            ));

            // Deny access to other directories
            profile.push_str("(deny file-read* file-write* (subpath \"/\"))\n");
        }

        // Add resource limits
        if let Some(memory_mb) = self.config.max_memory_mb {
            profile.push_str(&format!("(limit memory {}))\n", memory_mb * 1024 * 1024));
        }

        if let Some(cpu_percent) = self.config.max_cpu_percent {
            profile.push_str(&format!("(limit cpu {})\n", cpu_percent));
        }

        // Write the profile to a file
        let mut file = File::create(&self.profile_path)?;
        file.write_all(profile.as_bytes())?;

        Ok(())
    }
}

impl Drop for MacOSSandbox {
    fn drop(&mut self) {
        // Clean up the profile file
        let _ = std::fs::remove_file(&self.profile_path);
    }
}
