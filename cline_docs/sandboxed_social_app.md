# Sandboxed Social App Implementation

## Overview

We've implemented a solution to provide a sandbox guarantee for applications that need to access the DataFold node API. This ensures that only processes initiated by our FoldClient can access the node API, and it enforces permissions and resource limits on those processes.

## Architecture

The architecture consists of three main components:

1. **DataFold Node**: The node that provides the API for accessing and manipulating data.
2. **FoldClient**: A mediator between applications and the DataFold node that provides sandboxed access to the node API.
3. **Sandboxed Applications**: Applications that run in the FoldClient sandbox and communicate with the DataFold node through the FoldClient.

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│                 │     │                 │     │                 │
│  Sandboxed App  │◄────┤   FoldClient    │◄────┤  DataFold Node  │
│                 │     │                 │     │                 │
└─────────────────┘     └─────────────────┘     └─────────────────┘
        ▲                       ▲                       ▲
        │                       │                       │
        │                       │                       │
        │                       │                       │
┌───────┴───────┐     ┌─────────┴─────────┐   ┌─────────┴─────────┐
│  Restricted   │     │  Authentication   │   │   Node API with   │
│  Environment  │     │  & Permissions    │   │   Cryptographic   │
│               │     │                   │   │   Verification    │
└───────────────┘     └───────────────────┘   └───────────────────┘
```

## FoldClient Implementation

The FoldClient is implemented as a Rust library and executable that provides the following features:

1. **Sandboxed Environment**: Applications run in a sandboxed environment with restricted access to system resources.
2. **Network Isolation**: Applications can be prevented from accessing the network directly.
3. **File System Isolation**: Applications can be restricted to a specific directory.
4. **Resource Limits**: Applications can have memory and CPU usage limits.
5. **Permission Enforcement**: Applications can only perform operations they have been granted permission for.
6. **Cryptographic Authentication**: All communication is authenticated using cryptographic signatures.
7. **Cross-Platform**: Works on Linux, macOS, and Windows.

### Sandbox Implementation

The sandbox implementation is platform-specific:

- **Linux**: Uses namespaces, cgroups, and seccomp to isolate applications.
- **macOS**: Uses the sandbox-exec command to create a sandboxed environment.
- **Windows**: Uses job objects and integrity levels to restrict applications.

### IPC Mechanism

Applications communicate with the FoldClient using a secure IPC mechanism:

- **Unix Domain Sockets**: On Linux and macOS, Unix domain sockets are used for IPC.
- **Named Pipes**: On Windows, named pipes are used for IPC.

### Authentication and Authorization

The FoldClient uses cryptographic signatures to authenticate requests to the DataFold node:

1. Each application is registered with the FoldClient and receives a unique ID and token.
2. The FoldClient generates a keypair for each application.
3. Applications use their token to authenticate with the FoldClient.
4. The FoldClient signs requests to the DataFold node with its private key.
5. The DataFold node verifies the signature using the FoldClient's public key.

## Social App Integration

The social app has been modified to use the FoldClient for accessing the DataFold node API:

1. The social app is registered with the FoldClient and receives a unique ID and token.
2. The social app is launched by the FoldClient in a sandboxed environment.
3. The social app uses the FoldClient's IPC mechanism to communicate with the DataFold node.
4. The FoldClient enforces permissions and resource limits on the social app.

### Example Code

Here's an example of how the social app uses the FoldClient:

```rust
use fold_client::ipc::client::{IpcClient, Result};
use serde_json::json;
use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Get the app ID and token from environment variables
    let app_id = env::var("FOLD_CLIENT_APP_ID")
        .expect("FOLD_CLIENT_APP_ID environment variable not set");
    let token = env::var("FOLD_CLIENT_APP_TOKEN")
        .expect("FOLD_CLIENT_APP_TOKEN environment variable not set");

    // Get the socket directory
    let socket_dir = if let Ok(dir) = env::var("FOLD_CLIENT_SOCKET_DIR") {
        PathBuf::from(dir)
    } else {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        home_dir.join(".datafold").join("sockets")
    };

    // Connect to the FoldClient
    let mut client = IpcClient::connect(&socket_dir, &app_id, &token).await?;

    // List available schemas
    let schemas = client.list_schemas().await?;
    println!("Available schemas: {:?}", schemas);

    // Query users
    if schemas.contains(&"user".to_string()) {
        let users = client.query("user", &["id", "username", "full_name"], None).await?;
        println!("Users: {:?}", users);
    }

    // Create a new post
    let post_id = uuid::Uuid::new_v4().to_string();
    let post_data = json!({
        "id": post_id,
        "user_id": "user123",
        "content": "Hello, world!",
        "created_at": chrono::Utc::now().to_rfc3339(),
    });
    let result = client.create("post", post_data).await?;
    println!("Post created with ID: {}", result);

    Ok(())
}
```

## DataFold Node Integration

The DataFold node has been modified to verify signatures from the FoldClient:

1. The DataFold node is configured with the FoldClient's public key.
2. The DataFold node verifies the signature of each request using the FoldClient's public key.
3. If the signature is valid, the DataFold node processes the request.
4. If the signature is invalid, the DataFold node rejects the request.

### Example Code

Here's an example of how the DataFold node verifies signatures:

```rust
// Add a function to verify signatures
fn verify_signature(app_id: &str, message: &[u8], signature: &[u8], public_key: &[u8]) -> bool {
    use ed25519_dalek::{PublicKey, Signature, Verifier};
    
    // Parse the public key
    let public_key = match PublicKey::from_bytes(public_key) {
        Ok(key) => key,
        Err(_) => return false,
    };
    
    // Parse the signature
    let signature = match Signature::from_bytes(signature) {
        Ok(sig) => sig,
        Err(_) => return false,
    };
    
    // Verify the signature
    match public_key.verify(message, &signature) {
        Ok(_) => true,
        Err(_) => false,
    }
}

// Modify the process_request function to verify signatures
async fn process_request(
    request: &Value,
    node: Arc<Mutex<DataFoldNode>>,
) -> FoldDbResult<Value> {
    // Extract the app ID and signature from the request
    let app_id = request.get("app_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| FoldDbError::Config("Missing app_id".to_string()))?;
        
    let signature_base64 = request.get("signature")
        .and_then(|v| v.as_str())
        .ok_or_else(|| FoldDbError::Config("Missing signature".to_string()))?;
        
    // Decode the signature
    let signature = base64::decode(signature_base64)
        .map_err(|e| FoldDbError::Config(format!("Invalid signature: {}", e)))?;
        
    // Create a canonical representation of the request for verification
    let mut request_for_verification = request.clone();
    if let Some(obj) = request_for_verification.as_object_mut() {
        obj.remove("signature");
    }
    
    let message = serde_json::to_string(&request_for_verification)
        .map_err(|e| FoldDbError::Config(format!("Failed to serialize request: {}", e)))?;
        
    // Get the public key for the FoldClient
    let public_key = get_fold_client_public_key()?;
    
    // Verify the signature
    if !verify_signature(app_id, message.as_bytes(), &signature, &public_key) {
        return Err(FoldDbError::Config("Invalid signature".to_string()));
    }
    
    // Process the request as usual
    // ...
}
```

## Security Considerations

The FoldClient provides a strong sandbox guarantee, but there are some security considerations to keep in mind:

1. **Root/Administrator Access**: The sandbox may not be effective against applications running with root or administrator privileges.
2. **Kernel Vulnerabilities**: The sandbox relies on the security of the underlying operating system.
3. **IPC Security**: The IPC mechanism is secure, but relies on the security of the underlying operating system.
4. **Resource Exhaustion**: While the sandbox can limit resource usage, it may not prevent all forms of resource exhaustion.
5. **Side-Channel Attacks**: The sandbox may not protect against side-channel attacks.

## Future Improvements

1. **Enhanced Sandboxing**: Implement more advanced sandboxing techniques, such as seccomp-bpf on Linux.
2. **Fine-Grained Permissions**: Implement more fine-grained permissions for applications.
3. **Resource Monitoring**: Implement real-time monitoring of resource usage.
4. **Audit Logging**: Implement comprehensive audit logging for all operations.
5. **Multi-User Support**: Implement support for multiple users with different permissions.
