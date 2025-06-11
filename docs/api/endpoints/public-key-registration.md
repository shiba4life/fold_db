# Public Key Registration API

The Public Key Registration API allows clients to register their Ed25519 public keys with DataFold for signature authentication. This is the first step in setting up signature-based authentication.

## Base URL

All endpoints are prefixed with `/api/crypto/keys`

**Production**: `https://api.datafold.com/api/crypto/keys`  
**Development**: `http://localhost:9001/api/crypto/keys`

## Authentication

Public key registration endpoints do not require authentication (chicken-and-egg problem). However, they are rate-limited to prevent abuse.

## Endpoints Overview

| Endpoint | Method | Purpose | Auth Required |
|----------|--------|---------|---------------|
| [`/register`](#register-public-key) | POST | Register a new public key | No |
| [`/status/{client_id}`](#get-registration-status) | GET | Get registration status | No |
| [`/update/{client_id}`](#update-registration) | PUT | Update key metadata | Yes |
| [`/revoke/{client_id}`](#revoke-registration) | DELETE | Revoke a public key | Yes |

## Register Public Key

Register a client's Ed25519 public key for authentication and signature verification.

### Request

**Endpoint**: `POST /api/crypto/keys/register`

**Headers**:
```http
Content-Type: application/json
```

**Request Body**:
```json
{
  "client_id": "optional-client-identifier",
  "user_id": "optional-user-identifier", 
  "public_key": "hex-encoded-ed25519-public-key",
  "key_name": "optional-human-readable-key-name",
  "metadata": {
    "optional": "metadata",
    "environment": "production",
    "version": "1.0.0",
    "description": "Production API key for service X"
  }
}
```

**Request Fields**:

| Field | Type | Required | Description | Constraints |
|-------|------|----------|-------------|-------------|
| `client_id` | string | No | Unique identifier for the client | 1-64 chars, alphanumeric + hyphens |
| `user_id` | string | No | Associated user identifier | 1-128 chars |
| `public_key` | string | **Yes** | 64-character hex-encoded Ed25519 public key | Exactly 64 hex characters (32 bytes) |
| `key_name` | string | No | Human-readable name for the key | 1-128 chars |
| `metadata` | object | No | Additional key-value metadata | Max 10 keys, values <256 chars each |

**Public Key Format**:
- **Algorithm**: Ed25519 (Edwards-curve Digital Signature Algorithm)
- **Size**: 32 bytes (256 bits)
- **Encoding**: Hexadecimal (64 characters)
- **Example**: `"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"`

### Response

**Success Response** (201 Created):
```json
{
  "success": true,
  "data": {
    "registration_id": "reg_7f9e8a5b-1234-5678-9abc-def123456789",
    "client_id": "client-identifier-123",
    "public_key": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    "key_name": "Production API Key",
    "registered_at": "2025-06-09T23:27:09.123456Z",
    "status": "active",
    "expires_at": null,
    "metadata": {
      "environment": "production",
      "version": "1.0.0",
      "description": "Production API key for service X"
    }
  }
}
```

**Response Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `registration_id` | string | UUID v4 identifier for this registration |
| `client_id` | string | Client identifier (auto-generated if not provided) |
| `public_key` | string | Hex-encoded public key as registered |
| `key_name` | string | Human-readable key name |
| `registered_at` | string | ISO 8601 timestamp of registration |
| `status` | string | Current status: `"active"`, `"pending"`, `"revoked"` |
| `expires_at` | string\|null | Expiration timestamp (null = no expiration) |
| `metadata` | object | Registered metadata |

### Error Responses

#### 400 Bad Request - Invalid Public Key Format
```json
{
  "success": false,
  "error": {
    "code": "INVALID_PUBLIC_KEY",
    "message": "Ed25519 public key must be exactly 32 bytes (64 hex characters)",
    "details": {
      "provided_length": 62,
      "expected_length": 64,
      "format": "hexadecimal"
    }
  }
}
```

#### 409 Conflict - Client Already Registered
```json
{
  "success": false,
  "error": {
    "code": "CLIENT_ALREADY_REGISTERED",
    "message": "Client already has a registered public key. Use update endpoint to change keys.",
    "details": {
      "existing_client_id": "client-identifier-123",
      "registered_at": "2025-06-09T20:15:30Z",
      "update_endpoint": "/api/crypto/keys/update/client-identifier-123"
    }
  }
}
```

#### 409 Conflict - Duplicate Public Key
```json
{
  "success": false,
  "error": {
    "code": "DUPLICATE_PUBLIC_KEY",
    "message": "This public key is already registered to another client",
    "details": {
      "conflict_type": "duplicate_key",
      "guidance": "Each public key can only be registered once. Generate a new keypair."
    }
  }
}
```

#### 422 Unprocessable Entity - Invalid Metadata
```json
{
  "success": false,
  "error": {
    "code": "INVALID_METADATA",
    "message": "Metadata validation failed",
    "details": {
      "errors": [
        "metadata.environment: must be one of [development, staging, production]",
        "metadata.description: exceeds maximum length of 256 characters"
      ]
    }
  }
}
```

#### 429 Too Many Requests - Rate Limited
```json
{
  "success": false,
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Registration rate limit exceeded. Please try again later.",
    "details": {
      "retry_after": 300,
      "limit": 10,
      "window": 3600,
      "remaining": 0
    }
  }
}
```

### Example Requests

#### Basic Registration
```bash
curl -X POST https://api.datafold.com/api/crypto/keys/register \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "my-app-prod",
    "public_key": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    "key_name": "Production Key"
  }'
```

#### Registration with Full Metadata
```bash
curl -X POST https://api.datafold.com/api/crypto/keys/register \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "analytics-service-v2",
    "user_id": "service-account-analytics",
    "public_key": "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456",
    "key_name": "Analytics Service Production Key",
    "metadata": {
      "environment": "production",
      "service": "analytics",
      "version": "2.1.0",
      "team": "data-platform",
      "contact": "data-team@company.com",
      "deployment": "kubernetes-cluster-1",
      "region": "us-east-1"
    }
  }'
```

#### Auto-Generated Client ID
```bash
curl -X POST https://api.datafold.com/api/crypto/keys/register \
  -H "Content-Type: application/json" \
  -d '{
    "public_key": "f1e2d3c4b5a6978812345678901234567890fedcba1234567890fedcba123456",
    "key_name": "Development Key",
    "metadata": {
      "environment": "development",
      "generated_by": "dev-setup-script"
    }
  }'
```

## Get Registration Status

Retrieve the registration status and details for a client's public key.

### Request

**Endpoint**: `GET /api/crypto/keys/status/{client_id}`

**Path Parameters**:
- `client_id` (string, required): The client identifier to look up

**Headers**:
```http
Accept: application/json
```

### Response

**Success Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "registration_id": "reg_7f9e8a5b-1234-5678-9abc-def123456789",
    "client_id": "my-app-prod",
    "public_key": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    "key_name": "Production Key",
    "registered_at": "2025-06-09T23:27:09.123456Z",
    "status": "active",
    "last_used": "2025-06-09T23:45:15.789123Z",
    "usage_count": 1543,
    "expires_at": null,
    "metadata": {
      "environment": "production",
      "version": "1.0.0"
    }
  }
}
```

### Error Responses

#### 404 Not Found - Client Not Registered
```json
{
  "success": false,
  "error": {
    "code": "CLIENT_NOT_FOUND",
    "message": "No public key registered for this client",
    "details": {
      "client_id": "unknown-client",
      "suggestion": "Register the public key first using POST /api/crypto/keys/register"
    }
  }
}
```

### Example Requests

```bash
# Check registration status
curl https://api.datafold.com/api/crypto/keys/status/my-app-prod

# Check with verbose output
curl -v https://api.datafold.com/api/crypto/keys/status/my-app-prod
```

## Update Registration

Update metadata and settings for an existing public key registration.

### Request

**Endpoint**: `PUT /api/crypto/keys/update/{client_id}`

**Authentication**: Required (must be signed with the registered private key)

**Headers**:
```http
Content-Type: application/json
Signature-Input: sig1=("@method" "@target-uri" "content-type" "content-digest");created=1718904473;nonce="550e8400e29b41d4a716446655440000";keyid="my-app-prod";alg="ed25519"
Signature: sig1=:iP8QqVo8mfq7UQWMxVYuEl5HPgQrydV7F+Gv2jA8O7N7vCN9WOPbVjFnvUjNh8QXi2Hm2cTEfqEi3JqOgRDnlJ8w=:
Content-Digest: sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:
```

**Request Body**:
```json
{
  "key_name": "Updated Production Key Name",
  "metadata": {
    "environment": "production",
    "version": "2.0.0",
    "updated_reason": "Version upgrade",
    "contact": "new-team@company.com"
  }
}
```

**Updatable Fields**:
- `key_name`: Human-readable name
- `metadata`: Metadata object (replaces existing metadata)

**Non-Updatable Fields**:
- `client_id`: Cannot be changed
- `public_key`: Cannot be changed (use key rotation instead)
- `user_id`: Cannot be changed

### Response

**Success Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "registration_id": "reg_7f9e8a5b-1234-5678-9abc-def123456789",
    "client_id": "my-app-prod",
    "public_key": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    "key_name": "Updated Production Key Name",
    "registered_at": "2025-06-09T23:27:09.123456Z",
    "updated_at": "2025-06-10T10:15:30.456789Z",
    "status": "active",
    "metadata": {
      "environment": "production",
      "version": "2.0.0",
      "updated_reason": "Version upgrade",
      "contact": "new-team@company.com"
    }
  }
}
```

## Revoke Registration

Permanently revoke a public key registration. This action cannot be undone.

### Request

**Endpoint**: `DELETE /api/crypto/keys/revoke/{client_id}`

**Authentication**: Required (must be signed with the registered private key)

**Headers**:
```http
Signature-Input: sig1=("@method" "@target-uri");created=1718904473;nonce="550e8400e29b41d4a716446655440000";keyid="my-app-prod";alg="ed25519"
Signature: sig1=:iP8QqVo8mfq7UQWMxVYuEl5HPgQrydV7F+Gv2jA8O7N7vCN9WOPbVjFnvUjNh8QXi2Hm2cTEfqEi3JqOgRDnlJ8w=:
```

**Request Body** (optional):
```json
{
  "reason": "Key compromise suspected",
  "confirm": true
}
```

### Response

**Success Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "client_id": "my-app-prod",
    "status": "revoked",
    "revoked_at": "2025-06-10T11:30:45.123456Z",
    "reason": "Key compromise suspected"
  }
}
```

## Rate Limiting

Public key registration endpoints are rate-limited to prevent abuse:

| Endpoint | Rate Limit | Window | Burst |
|----------|------------|--------|--------|
| Register | 10 requests | 1 hour | 3 requests |
| Status | 100 requests | 1 hour | 20 requests |
| Update | 20 requests | 1 hour | 5 requests |
| Revoke | 5 requests | 1 hour | 2 requests |

Rate limit headers are included in responses:
```http
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 7
X-RateLimit-Reset: 1618884473
X-RateLimit-Window: 3600
```

## Security Considerations

### Public Key Validation
- All public keys must be valid Ed25519 public keys (32 bytes)
- Keys are validated using the `ed25519-dalek` library
- Invalid key formats are rejected with detailed error messages
- Weak or test keys are detected and rejected

### Duplicate Prevention
- Each client can only register one public key at a time
- Each public key can only be registered once across all clients
- Attempts to register duplicate keys return `409 Conflict`
- SHA-256 hashes are used for efficient duplicate detection

### Storage Security
- Public keys are stored with SHA-256 hashes for efficient duplicate detection
- All registration data includes integrity timestamps
- Database operations are atomic to prevent partial registrations
- Metadata is sanitized and validated before storage

### Access Control
- Registration endpoints are public (no auth required)
- Update and revoke endpoints require signature authentication
- Rate limiting prevents abuse of public endpoints
- Audit logging tracks all registration activities

## Integration Notes

### Client Libraries
This API is designed to integrate with:
- **JavaScript SDK**: [`@datafold/sdk`](../sdks/javascript/README.md)
- **Python SDK**: [`datafold-sdk`](../sdks/python/README.md)
- **CLI Tool**: [`datafold-cli`](../sdks/cli/README.md)

### Workflow Integration
1. **Development**: Generate keypair → Register public key → Test authentication
2. **CI/CD**: Store private key in secrets → Configure authentication → Run tests
3. **Production**: Secure key storage → Register with metadata → Monitor usage

### Monitoring and Observability
- **Metrics**: Registration rate, success/failure rates, key usage patterns
- **Logging**: All registration activities with full audit trail
- **Alerting**: Failed registrations, rate limit violations, security events

## Related APIs

- **[Signature Verification](signature-verification.md)** - Verify digital signatures using registered keys
- **[Error Codes Reference](error-codes.md)** - Complete error code documentation
- **[Authentication Overview](../authentication/overview.md)** - How signature authentication works

## Examples and Tutorials

- **[Getting Started Guide](../authentication/getting-started.md)** - Complete setup walkthrough
- **[JavaScript Examples](../sdks/javascript/examples.md)** - Working code examples
- **[Python Examples](../sdks/python/examples.md)** - Python integration examples
- **[CLI Examples](../sdks/cli/README.md)** - Command-line usage

---

**Next Steps**: After registering your public key, learn how to [sign requests](../authentication/request-signing.md) and [verify signatures](signature-verification.md).