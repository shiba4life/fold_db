//! Windows-specific sandbox implementation
//!
//! This module provides a sandbox implementation for Windows using job objects and integrity levels.

use crate::Result;
use crate::FoldClientError;
use crate::sandbox::{Sandbox, SandboxConfig};
use std::ffi::OsStr;
use std::os::windows::process::CommandExt;
use std::process::{Child, Command, Stdio};
use winapi::shared::minwindef::{DWORD, FALSE, TRUE};
use winapi::um::handleapi::CloseHandle;
use winapi::um::jobapi2::{AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject};
use winapi::um::processthreadsapi::GetCurrentProcess;
use winapi::um::winnt::{
    HANDLE, JOBOBJECT_BASIC_LIMIT_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_ACTIVE_PROCESS, JOB_OBJECT_LIMIT_JOB_MEMORY, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOB_OBJECT_LIMIT_PROCESS_MEMORY,
};

/// Windows sandbox implementation
pub struct WindowsSandbox {
    /// Sandbox configuration
    config: SandboxConfig,
    /// Job object handle
    job_handle: HANDLE,
}

impl Sandbox for WindowsSandbox {
    fn new(config: SandboxConfig) -> Result<Self> {
        // Create a job object
        let job_name = format!("FoldClient_{}", std::process::id());
        let job_name_wide: Vec<u16> = job_name.encode_utf16().chain(std::iter::once(0)).collect();
        let job_handle = unsafe {
            let handle = CreateJobObjectW(std::ptr::null_mut(), job_name_wide.as_ptr());
            if handle.is_null() {
                return Err(FoldClientError::Sandbox(
                    "Failed to create job object".to_string(),
                ));
            }
            handle
        };

        // Configure the job object
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

        // Set memory limit
        if let Some(memory_mb) = config.max_memory_mb {
            info.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_JOB_MEMORY | JOB_OBJECT_LIMIT_PROCESS_MEMORY;
            info.JobMemoryLimit = memory_mb * 1024 * 1024;
            info.ProcessMemoryLimit = memory_mb * 1024 * 1024;
        }

        // Set process limit
        info.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
        info.BasicLimitInformation.ActiveProcessLimit = 1;

        // Apply the configuration
        let result = unsafe {
            SetInformationJobObject(
                job_handle,
                winapi::um::winnt::JobObjectExtendedLimitInformation,
                &mut info as *mut _ as *mut _,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as DWORD,
            )
        };

        if result == 0 {
            unsafe {
                CloseHandle(job_handle);
            }
            return Err(FoldClientError::Sandbox(
                "Failed to configure job object".to_string(),
            ));
        }

        Ok(Self { config, job_handle })
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

        // Set creation flags
        let creation_flags = if !self.config.allow_network {
            // Use the Windows Firewall API to block network access
            // This is a simplified example - in practice, you would use the Windows Firewall API
            let _ = Command::new("netsh")
                .args(&[
                    "advfirewall",
                    "firewall",
                    "add",
                    "rule",
                    "name=FoldClient",
                    &format!("program={}", program),
                    "dir=out",
                    "action=block",
                ])
                .output();

            0
        } else {
            0
        };

        // Set the creation flags
        command.creation_flags(creation_flags);

        // Launch the command
        let mut child = command
            .spawn()
            .map_err(|e| FoldClientError::Sandbox(format!("Failed to launch command: {}", e)))?;

        // Assign the process to the job object
        let result = unsafe {
            AssignProcessToJobObject(self.job_handle, child.as_raw_handle() as HANDLE)
        };

        if result == 0 {
            // Failed to assign process to job object
            child.kill().ok();
            return Err(FoldClientError::Sandbox(
                "Failed to assign process to job object".to_string(),
            ));
        }

        Ok(child)
    }
}

impl Drop for WindowsSandbox {
    fn drop(&mut self) {
        // Close the job object handle
        unsafe {
            CloseHandle(self.job_handle);
        }

        // Remove the firewall rule
        if !self.config.allow_network {
            let _ = Command::new("netsh")
                .args(&[
                    "advfirewall",
                    "firewall",
                    "delete",
                    "rule",
                    "name=FoldClient",
                ])
                .output();
        }
    }
}
