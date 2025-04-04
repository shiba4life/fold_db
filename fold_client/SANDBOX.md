# FoldClient Sandbox Implementation

This document provides a detailed explanation of how the FoldClient sandbox implementation works across different platforms.

## Overview

The FoldClient sandbox implementation provides a secure environment for applications to run in, with the following features:

- **Process Isolation**: Applications run in separate processes with restricted capabilities.
- **Network Isolation**: Applications can be prevented from accessing the network directly.
- **File System Isolation**: Applications can be restricted to a specific directory.
- **Resource Limits**: Applications can have memory and CPU usage limits.
- **IPC Mechanism**: Applications communicate with the FoldClient using a secure IPC mechanism.

## Cross-Platform Implementation

The sandbox implementation is platform-specific, with different mechanisms used on Linux, macOS, and Windows.

### Linux Implementation

On Linux, the sandbox uses the following mechanisms:

1. **Namespaces**: Linux namespaces are used to isolate the application from the rest of the system.
   - `CLONE_NEWPID`: Creates a new PID namespace, isolating the process tree.
   - `CLONE_NEWNET`: Creates a new network namespace, isolating the network stack (if network access is disabled).
   - `CLONE_NEWNS`: Creates a new mount namespace, isolating the file system (if file system access is disabled).
   - `CLONE_NEWIPC`: Creates a new IPC namespace, isolating IPC resources.
   - `CLONE_NEWUTS`: Creates a new UTS namespace, isolating hostname and domain name.
   - `CLONE_NEWUSER`: Creates a new user namespace, isolating user and group IDs.

2. **Cgroups**: Control groups are used to limit resource usage.
   - Memory limits: Restricts the amount of memory the application can use.
   - CPU limits: Restricts the amount of CPU time the application can use.

3. **Seccomp**: Secure computing mode is used to restrict system calls.
   - Only allows a limited set of system calls, preventing the application from performing dangerous operations.

4. **Mount Namespace**: If file system access is disabled, a private `/tmp` directory is mounted to prevent access to the host file system.

### macOS Implementation

On macOS, the sandbox uses the following mechanisms:

1. **Sandbox-exec**: The `sandbox-exec` command is used to create a sandboxed environment.
   - A custom sandbox profile is generated based on the sandbox configuration.
   - The profile specifies what operations are allowed or denied.

2. **Sandbox Profiles**: The sandbox profile includes rules for:
   - Network access: Denies network access if disabled.
   - File system access: Restricts file system access to the working directory if disabled.
   - Resource limits: Sets memory and CPU limits.

3. **Resource Limits**: Environment variables are used to set resource limits.
   - `MEMMON_LIMIT`: Sets the memory limit.
   - `CPUMON_LIMIT`: Sets the CPU limit.

### Windows Implementation

On Windows, the sandbox uses the following mechanisms:

1. **Job Objects**: Job objects are used to group processes and control their resource usage.
   - `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`: Terminates all processes in the job when the job object is closed.
   - `JOB_OBJECT_LIMIT_JOB_MEMORY`: Limits the total memory usage of all processes in the job.
   - `JOB_OBJECT_LIMIT_PROCESS_MEMORY`: Limits the memory usage of each process in the job.
   - `JOB_OBJECT_LIMIT_ACTIVE_PROCESS`: Limits the number of processes in the job.

2. **Windows Firewall**: The Windows Firewall is used to block network access if disabled.
   - A firewall rule is created to block outbound connections from the application.

3. **Integrity Levels**: Integrity levels are used to restrict what the application can do.
   - The application runs at a low integrity level, preventing it from modifying high integrity resources.

## IPC Mechanism

Applications communicate with the FoldClient using a secure IPC mechanism:

1. **Unix Domain Sockets**: On Linux and macOS, Unix domain sockets are used for IPC.
   - Each application has its own socket, created in the app socket directory.
   - The socket path is based on the application ID.

2. **Named Pipes**: On Windows, named pipes are used for IPC.
   - Each application has its own named pipe, created in the app socket directory.
   - The pipe name is based on the application ID.

3. **Message Format**: Messages are serialized as JSON and include:
   - Request ID: A unique identifier for the request.
   - App ID: The ID of the application.
   - Token: The authentication token for the application.
   - Operation: The operation to perform.
   - Parameters: The parameters for the operation.
   - Signature: An optional cryptographic signature.

4. **Authentication**: The FoldClient verifies the application's token and signature before processing requests.
   - The token is verified against the stored token for the application.
   - The signature is verified using the application's public key.

5. **Permission Enforcement**: The FoldClient checks if the application has permission to perform the requested operation.
   - Permissions are specified when registering the application.
   - The FoldClient rejects requests for operations the application doesn't have permission for.

## Resource Limits

The FoldClient can enforce resource limits on applications:

1. **Memory Limits**: Restricts the amount of memory the application can use.
   - On Linux: Uses cgroups to set memory limits.
   - On macOS: Uses the sandbox profile to set memory limits.
   - On Windows: Uses job objects to set memory limits.

2. **CPU Limits**: Restricts the amount of CPU time the application can use.
   - On Linux: Uses cgroups to set CPU limits.
   - On macOS: Uses the sandbox profile to set CPU limits.
   - On Windows: Uses job objects to set CPU limits.

## Security Considerations

The FoldClient sandbox implementation provides strong security guarantees, but there are some considerations to keep in mind:

1. **Root/Administrator Access**: The sandbox may not be effective against applications running with root or administrator privileges.
   - On Linux: Root can break out of namespaces and cgroups.
   - On macOS: Root can bypass the sandbox.
   - On Windows: Administrators can bypass job objects and integrity levels.

2. **Kernel Vulnerabilities**: The sandbox relies on the security of the underlying operating system.
   - Kernel vulnerabilities could potentially be exploited to escape the sandbox.

3. **IPC Security**: The IPC mechanism is secure, but relies on the security of the underlying operating system.
   - Unix domain sockets and named pipes are generally secure, but could potentially be accessed by other processes with sufficient privileges.

4. **Resource Exhaustion**: While the sandbox can limit resource usage, it may not prevent all forms of resource exhaustion.
   - Applications could still potentially exhaust resources within their limits.

5. **Side-Channel Attacks**: The sandbox may not protect against side-channel attacks.
   - Applications could potentially use side-channel attacks to extract information from other processes.

## Best Practices

To maximize the security of the FoldClient sandbox:

1. **Principle of Least Privilege**: Only grant applications the permissions they need.
   - Restrict network access unless necessary.
   - Restrict file system access unless necessary.
   - Set tight resource limits.

2. **Regular Updates**: Keep the operating system and FoldClient up to date.
   - Security vulnerabilities are regularly discovered and patched.

3. **Monitoring**: Monitor application behavior for signs of compromise.
   - Unusual resource usage patterns could indicate a compromise.

4. **Secure Configuration**: Configure the FoldClient securely.
   - Use secure default settings.
   - Review and adjust settings as needed.

5. **Defense in Depth**: Use multiple layers of security.
   - The sandbox is just one layer of security.
   - Use other security measures as well, such as firewalls, intrusion detection systems, and regular security audits.
