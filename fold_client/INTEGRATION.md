# Integrating FoldClient with DataFold Node

This document provides instructions for integrating the FoldClient with the DataFold node to provide a sandbox guarantee for applications.

## Overview

The FoldClient acts as a mediator between applications and the DataFold node, providing sandboxed access to the node API. It ensures that only processes initiated by the FoldClient can access the node API, and it enforces permissions and resource limits on those processes.

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│                 │     │                 │     │                 │
│  Sandboxed App  │◄────┤   FoldClient    │◄────┤  DataFold Node  │
│                 │     │                 │     │                 │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

## Integration Steps

### 1. Install FoldClient

First, you need to install the FoldClient:

```bash
# Clone the repository
git clone https://github.com/datafold/fold_client.git

# Build the FoldClient
cd fold_client
cargo build --release
```

### 2. Configure FoldClient

Create a configuration file for the FoldClient:

```json
{
  "node_tcp_address": ["127.0.0.1", 9000],
  "app_socket_dir": "/path/to/app/sockets",
  "app_data_dir": "/path/to/app/data",
  "allow_network_access": false,
  "allow_filesystem_access": true,
  "max_memory_mb": 1024,
  "max_cpu_percent": 50
}
```

### 3. Start FoldClient

Start the FoldClient with the configuration file:

```bash
./target/release/fold_client start --config config.json
```

### 4. Register an App

Register an application with the FoldClient:

```bash
./target/release/fold_client register-app --name "My App" --permissions "list_schemas,query,mutation"
```

This will output the app ID and token, which you'll need to provide to the application.

### 5. Launch the App

Launch the application with the FoldClient:

```bash
./target/release/fold_client launch-app --id "app-id" --program "/path/to/app" --args "arg1,arg2"
```

### 6. Modify the DataFold Node

To ensure that only the FoldClient can access the DataFold node API, you need to modify the node to verify signatures from the FoldClient.

#### 6.1. Add Signature Verification to the Node

Modify the `tcp_server.rs` file in the DataFold node to verify signatures from the FoldClient:

```rust
// src/datafold_node/tcp_server.rs

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
    // In a real implementation, this would be stored securely
    let public_key = get_fold_client_public_key()?;
    
    // Verify the signature
    if !verify_signature(app_id, message.as_bytes(), &signature, &public_key) {
        return Err(FoldDbError::Config("Invalid signature".to_string()));
    }
    
    // Process the request as usual
    // ...
}

// Function to get the FoldClient's public key
fn get_fold_client_public_key() -> FoldDbResult<Vec<u8>> {
    // In a real implementation, this would be stored securely
    // For now, we'll use a placeholder
    Ok(vec![
        // Public key bytes
    ])
}
```

#### 6.2. Configure the Node to Trust the FoldClient

Configure the DataFold node to trust the FoldClient's public key:

```bash
# Generate a key pair for the FoldClient
openssl genpkey -algorithm ed25519 -out fold_client_private.pem
openssl pkey -in fold_client_private.pem -pubout -out fold_client_public.pem

# Configure the node to trust the FoldClient's public key
# This will depend on your node's configuration system
```

### 7. Test the Integration

Test the integration by running a sandboxed application:

```bash
# Start the DataFold node
cargo run --bin datafold_node -- --port 9000

# Start the FoldClient
./target/release/fold_client start --config config.json

# Register an app
./target/release/fold_client register-app --name "Test App" --permissions "list_schemas,query,mutation"

# Launch the app
./target/release/fold_client launch-app --id "app-id" --program "./target/debug/examples/sandboxed_app" --args ""
```

## Security Considerations

### Public Key Distribution

The FoldClient's public key needs to be distributed to the DataFold node securely. This can be done in several ways:

1. **Manual Configuration**: The public key is manually configured on the node.
2. **Certificate Authority**: The public key is signed by a trusted certificate authority.
3. **Key Exchange Protocol**: The FoldClient and node use a key exchange protocol to establish trust.

### Token Security

The app tokens generated by the FoldClient need to be kept secure. If an attacker obtains an app token, they could potentially impersonate the app.

### Signature Verification

The DataFold node must verify signatures from the FoldClient to ensure that only the FoldClient can access the node API.

## Best Practices

### Use Unix Domain Sockets

For local communication between the FoldClient and the DataFold node, use Unix domain sockets instead of TCP sockets. Unix domain sockets provide better security and performance for local communication.

### Restrict Permissions

Only grant applications the permissions they need. For example, if an application only needs to query data, don't grant it permission to mutate data.

### Monitor Application Behavior

Monitor application behavior for signs of compromise. Unusual resource usage patterns or unexpected API calls could indicate a compromise.

### Regular Updates

Keep the FoldClient and DataFold node up to date with the latest security patches.

### Defense in Depth

Use multiple layers of security. The FoldClient is just one layer of security. Use other security measures as well, such as firewalls, intrusion detection systems, and regular security audits.
