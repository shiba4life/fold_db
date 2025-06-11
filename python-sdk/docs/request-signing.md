# DataFold Python SDK - Request Signing

This document provides comprehensive information about using the DataFold Python SDK's RFC 9421 HTTP Message Signatures implementation with Ed25519 digital signatures.

## Overview

The DataFold Python SDK provides automatic request signing functionality that authenticates HTTP requests using digital signatures according to the RFC 9421 HTTP Message Signatures specification. This enables secure communication with DataFold's signature-protected API endpoints.

### Key Features

- **RFC 9421 Compliant**: Full implementation of HTTP Message Signatures specification
- **Ed25519 Cryptography**: Uses Ed25519 digital signatures for security and performance
- **Multiple Security Profiles**: Configurable security levels from minimal to strict
- **HTTP Client Integration**: Seamless integration with requests library
- **High Performance**: <10ms signing operations (typically ~0.2ms)
- **Comprehensive Error Handling**: Detailed error messages with error codes
- **Cross-Platform**: Works on Windows, macOS, and Linux

## Quick Start

### Basic Usage

```python
from datafold_sdk import (
    generate_key_pair,
    create_signing_config,
    RFC9421Signer,
    SignableRequest,
    HttpMethod
)

# 1. Generate Ed25519 key pair
key_pair = generate_key_pair()

# 2. Create signing configuration
config = (create_signing_config()
          .key_id("my-client-id")
          .private_key(key_pair.private_key)
          .profile("standard")
          .build())

# 3. Create signer
signer = RFC9421Signer(config)

# 4. Sign a request
request = SignableRequest(
    method=HttpMethod.POST,
    url="https://api.datafold.com/api/crypto/keys/register",
    headers={"content-type": "application/json"},
    body='{"client_id": "my-client", "public_key": "..."}'
)

result = signer.sign_request(request)

# 5. Use the signed headers
print(f"Signature-Input: {result.signature_input}")
print(f"Signature: {result.signature}")
```

### HTTP Client Integration

```python
from datafold_sdk import create_signing_session, generate_key_pair, create_from_profile

# Set up signing
key_pair = generate_key_pair()
config = create_from_profile("standard", "my-client", key_pair.private_key)

# Create signing session
session = create_signing_session(config)

# All requests are automatically signed
response = session.post(
    "https://api.datafold.com/api/data/create",
    json={"data": "example"}
)
```

## Security Profiles

The SDK provides three pre-configured security profiles:

### Minimal Profile
- **Use Case**: Low-latency scenarios, internal APIs
- **Components**: @method, @target-uri only
- **Content Digest**: Disabled
- **Nonce Validation**: Disabled

```python
config = create_from_profile("minimal", "client-id", private_key)
```

### Standard Profile (Recommended)
- **Use Case**: Most applications, balanced security
- **Components**: @method, @target-uri, content-type, content-digest
- **Content Digest**: SHA-256
- **Nonce Validation**: Enabled

```python
config = create_from_profile("standard", "client-id", private_key)
```

### Strict Profile
- **Use Case**: High-security scenarios, sensitive operations
- **Components**: @method, @target-uri, content-type, content-length, user-agent, authorization, content-digest
- **Content Digest**: SHA-512
- **Nonce Validation**: Enabled (no custom nonces)

```python
config = create_from_profile("strict", "client-id", private_key)
```

## Configuration

### Manual Configuration

```python
from datafold_sdk import create_signing_config, SignatureComponents

config = (create_signing_config()
          .key_id("my-client")
          .private_key(private_key)
          .algorithm("ed25519")  # Default
          .method(True)          # Include @method
          .target_uri(True)      # Include @target-uri
          .headers(["content-type", "authorization"])
          .content_digest(True)  # Include content digest
          .build())
```

### Custom Components

```python
from datafold_sdk import SignatureComponents

components = SignatureComponents(
    method=True,
    target_uri=True,
    headers=["content-type", "x-api-key"],
    content_digest=True
)

config = (create_signing_config()
          .key_id("custom-client")
          .private_key(private_key)
          .components(components)
          .build())
```

## HTTP Integration

### Signing Session

The `SigningSession` class wraps a `requests.Session` and automatically signs outgoing requests:

```python
from datafold_sdk import create_signing_session

# Create signing session
session = create_signing_session(signing_config)

# Configure signing
session.configure_signing(config, auto_sign=True)

# Make signed requests
response = session.get("https://api.datafold.com/api/data")
response = session.post("https://api.datafold.com/api/data", json=data)

# Disable signing temporarily
session.disable_signing()
response = session.get("https://example.com/public")  # Not signed

# Re-enable signing
session.enable_signing()
```

### DataFold HTTP Client Integration

```python
from datafold_sdk import DataFoldHttpClient, ServerConfig

# Create HTTP client with signing
server_config = ServerConfig(base_url="https://api.datafold.com")
client = DataFoldHttpClient(server_config, signing_config=config)

# Register public key (automatically signed)
registration = client.register_public_key(
    key_pair=key_pair,
    client_id="my-client",
    key_name="Production Key"
)
```

### Manual Request Signing

```python
from datafold_sdk import sign_request, SignableRequest, HttpMethod

request = SignableRequest(
    method=HttpMethod.PUT,
    url="https://api.datafold.com/api/resource/123",
    headers={"content-type": "application/json"},
    body='{"updated": true}'
)

# Sign the request
result = sign_request(request, config)

# Apply headers to your HTTP library
import requests
response = requests.put(
    request.url,
    headers={**request.headers, **result.headers},
    data=request.body
)
```

## Performance

The signing implementation is optimized for high performance:

- **Target**: <10ms per signing operation
- **Typical**: ~0.2ms per signing operation
- **Throughput**: >1000 requests/second
- **Memory**: Minimal overhead

### Performance Monitoring

```python
import time
from datafold_sdk import PerformanceTimer

timer = PerformanceTimer()
result = signer.sign_request(request)
elapsed_ms = timer.elapsed_ms()

print(f"Signing completed in {elapsed_ms:.2f}ms")
```

## Error Handling

The SDK provides comprehensive error handling with specific error codes:

### Common Errors

```python
from datafold_sdk import SigningError, SigningErrorCodes

try:
    result = signer.sign_request(request)
except SigningError as e:
    print(f"Signing failed: {e.message}")
    print(f"Error code: {e.code}")
    print(f"Details: {e.details}")
    
    # Handle specific errors
    if e.code == SigningErrorCodes.INVALID_URL:
        print("Fix the request URL")
    elif e.code == SigningErrorCodes.CANONICAL_MESSAGE_FAILED:
        print("Check required headers")
```

### Error Codes

- `INVALID_CONFIG`: Configuration validation failed
- `INVALID_PRIVATE_KEY`: Private key format is invalid
- `INVALID_REQUEST`: Request validation failed
- `INVALID_URL`: URL format is invalid
- `SIGNING_FAILED`: Cryptographic signing failed
- `CANONICAL_MESSAGE_FAILED`: Message construction failed
- `CRYPTOGRAPHY_UNAVAILABLE`: Required cryptography library not available

## Utilities

### Nonce and Timestamp Generation

```python
from datafold_sdk import generate_nonce, generate_timestamp, format_rfc3339_timestamp

# Generate nonce (UUID v4)
nonce = generate_nonce()
print(f"Nonce: {nonce}")

# Generate timestamp
timestamp = generate_timestamp()
rfc3339 = format_rfc3339_timestamp(timestamp)
print(f"Timestamp: {timestamp} ({rfc3339})")
```

### Content Digest Calculation

```python
from datafold_sdk import calculate_content_digest, DigestAlgorithm

content = '{"message": "Hello, World!"}'
digest = calculate_content_digest(content, DigestAlgorithm.SHA256)

print(f"Algorithm: {digest.algorithm}")
print(f"Digest: {digest.digest}")
print(f"Header Value: {digest.header_value}")
```

### URL Parsing

```python
from datafold_sdk import parse_url

url_parts = parse_url("https://api.example.com/path?param=value")
print(f"Target URI: {url_parts['target_uri']}")  # "/path?param=value"
print(f"Origin: {url_parts['origin']}")          # "https://api.example.com"
```

## Key Management

### Key Generation

```python
from datafold_sdk import generate_key_pair

# Generate new Ed25519 key pair
key_pair = generate_key_pair()

print(f"Private Key (hex): {key_pair.private_key.hex()}")
print(f"Public Key (hex): {key_pair.public_key.hex()}")

# Store securely (example)
with open("private_key.pem", "w") as f:
    from datafold_sdk import format_key
    pem_key = format_key(key_pair.private_key, "pem")
    f.write(pem_key)
```

### Key Loading

```python
from datafold_sdk import parse_key

# Load from PEM file
with open("private_key.pem", "r") as f:
    pem_data = f.read()
    private_key = parse_key(pem_data, "pem")

# Load from hex string
hex_key = "abc123..."
private_key = parse_key(hex_key, "hex")
```

## RFC 9421 Compliance

The implementation strictly follows RFC 9421 HTTP Message Signatures:

### Signature Format

```
Signature-Input: sig1=("@method" "@target-uri" "content-type" "content-digest");created=1234567890;keyid="client-001";alg="ed25519";nonce="uuid-v4"
Signature: sig1=:base64-encoded-signature:
```

### Canonical Message

```
"@method": POST
"@target-uri": /api/endpoint
"content-type": application/json
"content-digest": sha-256=:hash:
"@signature-params": ("@method" "@target-uri" "content-type" "content-digest");created=1234567890;keyid="client-001";alg="ed25519";nonce="uuid-v4"
```

### Supported Components

- **@method**: HTTP method (GET, POST, etc.)
- **@target-uri**: Request path and query string
- **Headers**: Any HTTP headers (case-insensitive)
- **content-digest**: SHA-256 or SHA-512 hash of request body

## Security Considerations

### Best Practices

1. **Key Storage**: Store private keys securely using OS keychain or HSM
2. **Key Rotation**: Regularly rotate signing keys
3. **Nonce Validation**: Use unique nonces to prevent replay attacks
4. **HTTPS Only**: Always use HTTPS for signed requests
5. **Profile Selection**: Choose appropriate security profile for your use case

### Threat Mitigation

- **Replay Attacks**: Prevented by unique nonces and timestamps
- **Man-in-the-Middle**: Mitigated by signing request components
- **Request Tampering**: Detected by content digest validation
- **Key Compromise**: Limited by key rotation and scope

## Troubleshooting

### Common Issues

#### "Cryptography package not available"
```bash
pip install cryptography>=41.0.0
```

#### "Required header not found"
```python
# Ensure all required headers are present
request = SignableRequest(
    method=HttpMethod.POST,
    url="https://api.example.com/test",
    headers={
        "content-type": "application/json",
        "authorization": "Bearer token"  # If required by config
    },
    body='{"data": true}'
)
```

#### "Invalid private key format"
```python
# Ensure private key is exactly 32 bytes
assert len(private_key) == 32
assert isinstance(private_key, bytes)
```

### Debug Mode

```python
import logging

# Enable debug logging
logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger('datafold_sdk.signing')

# This will show detailed signing information
result = signer.sign_request(request)
```

## Examples

See `examples/request_signing_example.py` for comprehensive examples covering:

- Basic signing workflow
- Security profile usage
- HTTP client integration
- Performance benchmarking
- Utility functions
- Error handling

## API Reference

For complete API documentation, see the inline documentation in the source code or generate documentation using:

```bash
cd python-sdk
python -c "import datafold_sdk; help(datafold_sdk.signing)"
```

## Migration from Other Libraries

### From PyJWT

```python
# Old (PyJWT)
import jwt
token = jwt.encode(payload, private_key, algorithm="EdDSA")

# New (DataFold SDK)
from datafold_sdk import create_signing_config, RFC9421Signer
config = create_signing_config().key_id("client").private_key(private_key).build()
signer = RFC9421Signer(config)
result = signer.sign_request(request)
```

### From Manual Signing

```python
# Old (manual)
import hashlib
import base64
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

message = f"{method}\n{url}\n{headers}"
signature = private_key.sign(message.encode())

# New (DataFold SDK)
from datafold_sdk import sign_request
result = sign_request(request, config)
```

## Support

For questions, issues, or feature requests:

1. Check the [examples](../examples/) directory
2. Review this documentation
3. Open an issue on the project repository
4. Contact the DataFold team

## Version Compatibility

- **Python**: 3.8+
- **Cryptography**: 41.0.0+
- **Requests**: 2.31.0+ (optional, for HTTP integration)
- **RFC 9421**: Full compliance
- **Ed25519**: Standard implementation

## License

This implementation is part of the DataFold SDK and is licensed under the same terms as the main project.