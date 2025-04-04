//! Linux-specific sandbox implementation
//!
//! This module provides a sandbox implementation for Linux using namespaces, cgroups, and seccomp.

use crate::Result;
use crate::FoldClientError;
use crate::sandbox::{Sandbox, SandboxConfig};
use nix::sched::{clone, CloneFlags};
use nix::sys::signal::Signal;
use nix::unistd::Pid;
use std::ffi::CString;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};

/// Linux sandbox implementation
pub struct LinuxSandbox {
    /// Sandbox configuration
    config: SandboxConfig,
}

impl Sandbox for LinuxSandbox {
    fn new(config: SandboxConfig) -> Result<Self> {
        Ok(Self { config })
    }

    fn run_command(&self, program: &str, args: &[&str]) -> Result<Child> {
        // Create the command
        let mut command = Command::new(program);
        command
            .args(args)
            .current_dir(&self.config.working_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set environment variables
        for (key, value) in &self.config.env_vars {
            command.env(key, value);
        }

        // Apply sandbox restrictions
        unsafe {
            command.pre_exec(|| {
                // Create a new namespace for the process
                let mut clone_flags = CloneFlags::empty();

                // Always create a new PID namespace
                clone_flags.insert(CloneFlags::CLONE_NEWPID);

                // If network access is not allowed, create a new network namespace
                if !self.config.allow_network {
                    clone_flags.insert(CloneFlags::CLONE_NEWNET);
                }

                // If file system access is not allowed, create a new mount namespace
                if !self.config.allow_filesystem {
                    clone_flags.insert(CloneFlags::CLONE_NEWNS);
                }

                // Create a new IPC namespace
                clone_flags.insert(CloneFlags::CLONE_NEWIPC);

                // Create a new UTS namespace
                clone_flags.insert(CloneFlags::CLONE_NEWUTS);

                // Create a new user namespace
                clone_flags.insert(CloneFlags::CLONE_NEWUSER);

                // Apply the namespace changes
                if let Err(e) = nix::sched::unshare(clone_flags) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to create namespaces: {}", e),
                    ));
                }

                // If file system access is not allowed, mount a private /tmp
                if !self.config.allow_filesystem {
                    // Mount a private /tmp
                    if let Err(e) = nix::mount::mount(
                        Some("none"),
                        "/tmp",
                        Some("tmpfs"),
                        nix::mount::MsFlags::MS_NOSUID | nix::mount::MsFlags::MS_NODEV,
                        Some("size=10M"),
                    ) {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Failed to mount private /tmp: {}", e),
                        ));
                    }
                }

                // Apply resource limits
                if let Some(memory_mb) = self.config.max_memory_mb {
                    // Set the memory limit using cgroups
                    // This is a simplified example - in practice, you would use the cgroups API
                    let memory_limit = memory_mb * 1024 * 1024;
                    let cgroup_path = format!("/sys/fs/cgroup/memory/fold_client/{}", std::process::id());
                    
                    // Create the cgroup directory
                    if let Err(e) = std::fs::create_dir_all(&cgroup_path) {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Failed to create cgroup directory: {}", e),
                        ));
                    }
                    
                    // Set the memory limit
                    if let Err(e) = std::fs::write(
                        format!("{}/memory.limit_in_bytes", cgroup_path),
                        memory_limit.to_string(),
                    ) {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Failed to set memory limit: {}", e),
                        ));
                    }
                    
                    // Add the current process to the cgroup
                    if let Err(e) = std::fs::write(
                        format!("{}/cgroup.procs", cgroup_path),
                        std::process::id().to_string(),
                    ) {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Failed to add process to cgroup: {}", e),
                        ));
                    }
                }

                if let Some(cpu_percent) = self.config.max_cpu_percent {
                    // Set the CPU limit using cgroups
                    // This is a simplified example - in practice, you would use the cgroups API
                    let cgroup_path = format!("/sys/fs/cgroup/cpu/fold_client/{}", std::process::id());
                    
                    // Create the cgroup directory
                    if let Err(e) = std::fs::create_dir_all(&cgroup_path) {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Failed to create cgroup directory: {}", e),
                        ));
                    }
                    
                    // Set the CPU limit
                    let cpu_quota = (cpu_percent as u64 * 1000) / 100;
                    if let Err(e) = std::fs::write(
                        format!("{}/cpu.cfs_quota_us", cgroup_path),
                        cpu_quota.to_string(),
                    ) {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Failed to set CPU limit: {}", e),
                        ));
                    }
                    
                    // Add the current process to the cgroup
                    if let Err(e) = std::fs::write(
                        format!("{}/cgroup.procs", cgroup_path),
                        std::process::id().to_string(),
                    ) {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Failed to add process to cgroup: {}", e),
                        ));
                    }
                }

                Ok(())
            });
        }

        // Launch the command
        let child = command
            .spawn()
            .map_err(|e| FoldClientError::Sandbox(format!("Failed to launch command: {}", e)))?;

        Ok(child)
    }
}
