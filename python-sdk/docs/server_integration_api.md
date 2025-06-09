# DataFold Server Integration API

This document describes the Python SDK's integration with DataFold server for public key registration and signature verification.

## Overview

The DataFold Python SDK provides seamless integration with DataFold server through HTTP APIs, enabling:

- **Public Key Registration**: Register Ed25519 public keys with the server for client authentication
- **Signature Verification**: Verify digital signatures using server-side validation
- **Session Management**: Maintain client sessions with automatic retry and error handling
- **Key Storage Integration**: Combine server registration with local secure storage

## Quick Start

### Basic Setup

```python
from datafold_sdk import quick_setup

# Create client session with automatic registration
session = quick_setup("http://localhost:9001", client_id="my-client")

# Sign a message
message = "Hello DataFold!"
signature = session.sign_message(message)

# Verify with server
result = session.verify_with_server(message, signature)
print(f"Verification result: {result.verified}")
```

### Complete Workflow

```python
from datafold_sdk import DataFoldClient, generate_key_pair

# Create client
client = DataFoldClient("http://localhost:9001")

# Create new session with registration
session = client.create_new_session(
    client_id="my-application",
    key_name="primary-key",
    auto_register=True,
    save_to_storage=True
)

# Use the session
message = "Important data to sign"
signature = session.sign_message(message)
result = session.verify_with_server(message, signature)

print(f"Registration ID: {session.registration.registration_id}")
print(f"Signature verified: {result.verified}")
```

## HTTP Client API

### DataFoldHttpClient

Low-level HTTP client for direct server communication.

```python
from datafold_sdk import create_client, generate_key_pair

# Create HTTP client
client = create_client("http://localhost:9001", timeout=30.0)

# Generate key pair
key_pair = generate_key_pair()

# Register public key
registration = client.register_public_key(
    key_pair=key_pair,
    client_id="my-client",
    user_id="user123",
    key_name="primary-key",
    metadata={"app_version": "1.0.0"}
)

print(f"Registered: {registration.registration_id}")
```

#### Methods

##### `register_public_key(key_pair, client_id=None, user_id=None, key_name=None, metadata=None)`

Register a public key with the DataFold server.

**Parameters:**
- `key_pair` (Ed25519KeyPair): Key pair to register
- `client_id` (str, optional): Client identifier (auto-generated if not provided)
- `user_id` (str, optional): User identifier
- `key_name` (str, optional): Human-readable key name
- `metadata` (dict, optional): Additional metadata

**Returns:** `PublicKeyRegistration` object

**Example:**
```python
registration = client.register_public_key(
    key_pair=my_key_pair,
    client_id="web-app-v1",
    key_name="production-key"
)
```

##### `get_key_status(client_id)`

Retrieve registration status for a client.

**Parameters:**
- `client_id` (str): Client identifier

**Returns:** `PublicKeyRegistration` object

**Example:**
```python
status = client.get_key_status("my-client")
print(f"Status: {status.status}")
print(f"Last used: {status.last_used}")
```

##### `verify_signature(client_id, message, signature, message_encoding='utf8')`

Verify a digital signature using the server.

**Parameters:**
- `client_id` (str): Client identifier
- `message` (str|bytes): Original message
- `signature` (bytes): Ed25519 signature (64 bytes)
- `message_encoding` (str): Message encoding ('utf8', 'hex', 'base64')

**Returns:** `SignatureVerificationResult` object

**Example:**
```python
result = client.verify_signature(
    client_id="my-client",
    message="Hello World",
    signature=signature_bytes
)
print(f"Verified: {result.verified}")
```

## High-Level Integration API

### DataFoldClient

High-level client that combines key management with server integration.

```python
from datafold_sdk import DataFoldClient

client = DataFoldClient(
    server_url="http://localhost:9001",
    timeout=30.0,
    verify_ssl=True,
    retry_attempts=3
)
```

#### Methods

##### `create_new_session(**kwargs)`

Create a new client session with fresh key pair.

**Parameters:**
- `client_id` (str, optional): Client identifier
- `user_id` (str, optional): User identifier  
- `key_name` (str, optional): Key name for storage
- `metadata` (dict, optional): Registration metadata
- `auto_register` (bool, default=True): Auto-register with server
- `save_to_storage` (bool, default=True): Save to local storage

**Returns:** `ClientSession` object

##### `load_session_from_storage(storage_key, client_id=None, auto_check_status=True)`

Load existing session from storage.

**Parameters:**
- `storage_key` (str): Storage key for saved key pair
- `client_id` (str, optional): Client identifier
- `auto_check_status` (bool, default=True): Check server status

**Returns:** `ClientSession` object

### ClientSession

Represents a complete client identity with server integration.

#### Methods

##### `sign_message(message)`

Sign a message using the client's private key.

**Parameters:**
- `message` (str|bytes): Message to sign

**Returns:** `bytes` - Ed25519 signature

##### `verify_with_server(message, signature, message_encoding='utf8')`

Verify signature using DataFold server.

**Parameters:**
- `message` (str|bytes): Original message
- `signature` (bytes): Signature to verify
- `message_encoding` (str): Message encoding

**Returns:** `SignatureVerificationResult` object

##### `save_to_storage(key_name=None)`

Save key pair to secure storage.

**Parameters:**
- `key_name` (str, optional): Storage key name

**Returns:** `str` - Storage key identifier

## Configuration

### Server Configuration

```python
from datafold_sdk import ServerConfig, DataFoldHttpClient

config = ServerConfig(
    base_url="https://api.datafold.com/",
    timeout=30.0,
    verify_ssl=True,
    retry_attempts=3,
    retry_backoff_factor=0.3,
    max_retry_delay=10.0
)

client = DataFoldHttpClient(config)
```

### SSL Configuration

For development with self-signed certificates:

```python
client = DataFoldClient(
    server_url="https://localhost:9001",
    verify_ssl=False  # Only for development!
)
```

## Error Handling

### Exception Types

- `ServerCommunicationError`: Network or HTTP errors
- `ValidationError`: Invalid input parameters
- `Ed25519KeyError`: Key operation failures
- `StorageError`: Local storage failures

### Example Error Handling

```python
from datafold_sdk import ServerCommunicationError, ValidationError

try:
    session = client.create_new_session(client_id="my-app")
    signature = session.sign_message("test")
    result = session.verify_with_server("test", signature)
    
except ServerCommunicationError as e:
    print(f"Server error: {e}")
    print(f"Error details: {e.details}")
    
except ValidationError as e:
    print(f"Validation error: {e}")
    
except Exception as e:
    print(f"Unexpected error: {e}")
```

### Retry Logic

The HTTP client automatically retries failed requests:

- **Status codes**: 429, 500, 502, 503, 504
- **Network errors**: Connection timeouts, DNS failures
- **Backoff strategy**: Exponential backoff with jitter
- **Configuration**: Customizable retry attempts and delays

## API Endpoints

The SDK communicates with the following DataFold server endpoints:

- `POST /api/crypto/keys/register` - Register public key
- `GET /api/crypto/keys/status/{client_id}` - Get key status  
- `POST /api/crypto/signatures/verify` - Verify signature

### Request/Response Format

All API calls use JSON format with standardized response structure:

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

Error responses:

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable message",
    "details": { ... }
  }
}
```

## Security Considerations

### Transport Security

- Use HTTPS in production (`verify_ssl=True`)
- Validate server certificates
- Consider certificate pinning for high-security applications

### Key Management

- Private keys never leave the client
- Public keys are registered for authentication
- Signatures provide message integrity and authenticity

### Best Practices

1. **Use unique client IDs** for each application instance
2. **Store keys securely** using the built-in storage system
3. **Handle network errors** gracefully with retries
4. **Validate responses** from server APIs
5. **Log security events** for audit trails

## Examples

### Web Application Integration

```python
from datafold_sdk import DataFoldClient
import os

# Production configuration
client = DataFoldClient(
    server_url=os.environ["DATAFOLD_SERVER_URL"],
    timeout=30.0,
    verify_ssl=True
)

# Create session for user
session = client.create_new_session(
    client_id=f"webapp-{user_id}",
    user_id=user_id,
    key_name=f"user-{user_id}-key",
    metadata={
        "app_version": "1.2.3",
        "user_agent": request.headers.get("User-Agent")
    }
)

# Sign API request
api_data = {"action": "transfer", "amount": 100}
signature = session.sign_message(json.dumps(api_data))

# Include signature in API call
headers = {
    "X-Client-ID": session.client_id,
    "X-Signature": signature.hex(),
    "Content-Type": "application/json"
}
```

### CLI Tool Integration

```python
import argparse
from datafold_sdk import load_existing_client

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--server", required=True)
    parser.add_argument("--client-id", required=True)
    parser.add_argument("--message", required=True)
    args = parser.parse_args()
    
    # Load existing client
    session = load_existing_client(
        server_url=args.server,
        storage_key=f"cli-{args.client_id}",
        client_id=args.client_id
    )
    
    # Sign and verify message
    signature = session.sign_message(args.message)
    result = session.verify_with_server(args.message, signature)
    
    print(f"Message: {args.message}")
    print(f"Signature: {signature.hex()}")
    print(f"Verified: {result.verified}")

if __name__ == "__main__":
    main()
```

### Testing and Development

```python
from datafold_sdk import register_and_verify_workflow

# Complete test workflow
session, result = register_and_verify_workflow(
    server_url="http://localhost:9001",
    message="test message",
    client_id="test-client"
)

print(f"Registration: {session.registration.registration_id}")
print(f"Verification: {result.verified}")
```

## Performance Notes

- **Connection Pooling**: HTTP client reuses connections
- **Concurrent Requests**: Thread-safe for multiple operations
- **Caching**: Registration status cached in ClientSession
- **Timeouts**: Configurable request timeouts prevent hanging

## Migration Guide

For applications upgrading from local-only key management:

1. **Add server configuration** to existing code
2. **Update key generation** to include registration
3. **Replace local verification** with server verification
4. **Handle network errors** in signature workflows
5. **Test connectivity** with development server

See the examples directory for complete migration examples.