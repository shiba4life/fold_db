# Authentication

DataFold supports multiple authentication methods across its HTTP REST API and TCP protocol interfaces to ensure secure access to data and operations.

## Authentication Methods

### 1. API Key Authentication (HTTP)
Simple bearer token authentication for HTTP requests.

**Usage:**
```bash
curl -H "Authorization: Bearer your-api-key" \
  http://localhost:9001/api/schemas
```

**Configuration:**
API keys are configured in the node configuration file:

```json
{
  "auth": {
    "api_keys": [
      {
        "key": "your-api-key",
        "permissions": ["read", "write"],
        "expires": "2024-12-31T23:59:59Z"
      }
    ]
  }
}
```

### 2. Signature-Based Authentication (HTTP)
Cryptographic signature authentication using Ed25519 key pairs.

**Usage:**
```bash
curl -H "X-Signature: ed25519:signature-hash" \
  -H "X-Public-Key: ed25519:public-key" \
  -H "X-Timestamp: 1609459200" \
  http://localhost:9001/api/schemas
```

**Headers:**
- `X-Public-Key`: Ed25519 public key (base64-encoded)
- `X-Signature`: Ed25519 signature of the request (base64-encoded)
- `X-Timestamp`: Unix timestamp of the request

**Signature Calculation:**
```python
import ed25519
import json
import time

# Create signature payload
timestamp = int(time.time())
method = "GET"
path = "/api/schemas"
body = ""  # Empty for GET requests

payload = f"{method}|{path}|{body}|{timestamp}"

# Sign with private key
private_key = ed25519.SigningKey(private_key_bytes)
signature = private_key.sign(payload.encode('utf-8'))

# Headers
headers = {
    "X-Public-Key": f"ed25519:{base64.b64encode(public_key_bytes).decode()}",
    "X-Signature": f"ed25519:{base64.b64encode(signature).decode()}",
    "X-Timestamp": str(timestamp)
}
```

### 3. Public Key Authentication (TCP)
Direct public key authentication in TCP protocol messages.

**Usage:**
```json
{
  "app_id": "my-app",
  "operation": "query",
  "params": {...},
  "public_key": "ed25519:public-key",
  "signature": "ed25519:signature",
  "timestamp": 1609459200
}
```

**Signature Process:**
1. Create payload from operation parameters
2. Add timestamp to prevent replay attacks
3. Sign payload with Ed25519 private key
4. Include public key and signature in message

## Key Management

### Key Generation
Generate Ed25519 key pairs for authentication:

```bash
# Using DataFold CLI
datafold_cli crypto generate-keypair --output keys/

# Using OpenSSL
openssl genpkey -algorithm Ed25519 -out private.pem
openssl pkey -in private.pem -pubout -out public.pem
```

### Key Storage
Store private keys securely:

```bash
# Set proper file permissions
chmod 600 private.pem

# Use environment variables
export DATAFOLD_PRIVATE_KEY="$(cat private.pem)"

# Use dedicated key management systems for production
```

### Key Rotation
Regular key rotation for enhanced security:

```json
{
  "auth": {
    "key_rotation": {
      "enabled": true,
      "rotation_interval_days": 90,
      "grace_period_days": 7
    }
  }
}
```

## Authentication Flow

### HTTP Request Authentication
1. **Generate signature** from request components
2. **Add headers** with public key, signature, and timestamp
3. **Send request** to DataFold HTTP API
4. **Server validates** signature and timestamp
5. **Access granted** if authentication succeeds

### TCP Message Authentication
1. **Prepare message** with operation parameters
2. **Add timestamp** to message
3. **Sign message** with private key
4. **Send message** with public key and signature
5. **Server validates** and processes if authentic

## Security Considerations

### Replay Attack Prevention
- **Timestamp validation**: Requests older than 5 minutes are rejected
- **Nonce support**: Optional nonce field for additional replay protection
- **Signature uniqueness**: Each signature is unique due to timestamp inclusion

### Key Security
- **Private key protection**: Store private keys securely
- **Public key verification**: Verify public keys before trusting
- **Key rotation**: Rotate keys regularly
- **Revocation**: Support for key revocation when compromised

### Network Security
- **TLS encryption**: Use HTTPS for HTTP API in production
- **Certificate validation**: Validate server certificates
- **Network isolation**: Restrict network access where possible

## Configuration Examples

### Node Configuration
```json
{
  "auth": {
    "required": true,
    "methods": ["api_key", "signature"],
    "api_keys": [
      {
        "key": "prod-api-key-123",
        "permissions": ["read"],
        "description": "Read-only production key"
      }
    ],
    "trusted_public_keys": [
      {
        "key": "ed25519:ABC123...",
        "permissions": ["read", "write"],
        "description": "Admin public key"
      }
    ],
    "signature_validation": {
      "max_timestamp_skew_seconds": 300,
      "require_nonce": false
    }
  }
}
```

### Client Configuration
```json
{
  "client": {
    "auth": {
      "method": "signature",
      "private_key_path": "keys/private.pem",
      "public_key": "ed25519:ABC123..."
    }
  }
}
```

## Error Responses

### Authentication Errors
- `AUTHENTICATION_REQUIRED`: No authentication provided
- `INVALID_SIGNATURE`: Signature validation failed
- `EXPIRED_TIMESTAMP`: Request timestamp too old
- `INVALID_PUBLIC_KEY`: Public key format invalid
- `KEY_NOT_TRUSTED`: Public key not in trusted list

**Example Error Response:**
```json
{
  "error": {
    "code": "INVALID_SIGNATURE",
    "message": "Signature validation failed",
    "details": {
      "public_key": "ed25519:ABC123...",
      "timestamp": 1609459200,
      "expected_format": "method|path|body|timestamp"
    }
  }
}
```

## Testing Authentication

### CLI Testing
```bash
# Test API key
curl -H "Authorization: Bearer test-key" \
  http://localhost:9001/api/health

# Test signature (using CLI helper)
datafold_cli auth test-signature \
  --private-key keys/private.pem \
  --url http://localhost:9001/api/schemas
```

### Integration Testing
```python
import requests
import ed25519
import base64
import time

def test_signature_auth():
    # Load keys
    with open('private.pem', 'rb') as f:
        private_key = ed25519.SigningKey(f.read())
    
    public_key = private_key.get_verifying_key()
    
    # Create request
    timestamp = int(time.time())
    method = "GET"
    path = "/api/schemas"
    body = ""
    
    payload = f"{method}|{path}|{body}|{timestamp}"
    signature = private_key.sign(payload.encode('utf-8'))
    
    # Send request
    headers = {
        "X-Public-Key": f"ed25519:{base64.b64encode(public_key.to_bytes()).decode()}",
        "X-Signature": f"ed25519:{base64.b64encode(signature).decode()}",
        "X-Timestamp": str(timestamp)
    }
    
    response = requests.get("http://localhost:9001/api/schemas", headers=headers)
    assert response.status_code == 200
```

## Related Documentation

- [CLI Authentication Guide](../guides/cli-authentication.md) - Detailed CLI setup
- [Security Best Practices](../guides/security-best-practices.md) - Production security
- [Key Management](../guides/key-rotation.md) - Key rotation procedures
- [Permissions & Payments API](./permissions-payments-api.md) - Access control beyond authentication
- [Error Handling](./error-handling.md) - Authentication error troubleshooting

## Return to Index

[← Back to API Reference Index](./index.md)