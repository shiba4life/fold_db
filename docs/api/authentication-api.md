# DataFold API Authentication Documentation

## Overview

All DataFold API requests require mandatory RFC 9421 HTTP Message Signatures using Ed25519 cryptography. **Authentication cannot be disabled** and is required for every API operation.

## ⚠️ Critical Requirements

**Mandatory Authentication**: Every API request must include:
- `Signature-Input` header with RFC 9421 signature input parameters
- `Signature` header with base64-encoded Ed25519 signature
- Valid timestamp within allowed time window
- Unique nonce for replay prevention
- Proper content digest for requests with body content

**No Bypass**: There are no development modes, debug endpoints, or configuration options that allow unauthenticated requests.

## Authentication Headers

### Required Headers

All authenticated requests must include these headers:

```http
POST /api/v1/data HTTP/1.1
Host: api.datafold.com
Content-Type: application/json
Content-Digest: sha-256=:uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek=:
Signature-Input: sig1=("@method" "@target-uri" "content-type" "content-digest");created=1640995200;keyid="client-key-123";alg="ed25519";nonce="550e8400-e29b-41d4-a716-446655440000"
Signature: sig1=:K2qGT5srn2OGbOIDzQ6kYT+ruaycnDAAUpKv+ePFfD6tk4B9q8G7su1BtomrqLCtmgLN4aTiNs2l+Lh3gBQIiw==:
X-Correlation-ID: req-550e8400-e29b-41d4-a716-446655440000

{
  "query": "SELECT * FROM users WHERE active = true"
}
```

### Header Specifications

#### Signature-Input Header

Format: `sig1=(<components>);<parameters>`

**Required Components:**
- `@method` - HTTP method (GET, POST, etc.)
- `@target-uri` - Full request URI
- `content-type` - Content-Type header (for requests with body)
- `content-digest` - Content digest (for requests with body)

**Required Parameters:**
- `created` - Unix timestamp when signature was created
- `keyid` - Client key identifier registered with server
- `alg` - Signature algorithm (must be "ed25519")
- `nonce` - Unique nonce (UUID4 format recommended)

Example:
```
Signature-Input: sig1=("@method" "@target-uri" "content-type" "content-digest");created=1640995200;keyid="client-key-123";alg="ed25519";nonce="550e8400-e29b-41d4-a716-446655440000"
```

#### Signature Header

Format: `sig1=:<base64-signature>:`

The signature is calculated over the canonical message constructed from the signature components.

Example:
```
Signature: sig1=:K2qGT5srn2OGbOIDzQ6kYT+ruaycnDAAUpKv+ePFfD6tk4B9q8G7su1BtomrqLCtmgLN4aTiNs2l+Lh3gBQIiw==:
```

#### Content-Digest Header

For requests with body content, calculate SHA-256 digest:

Format: `sha-256=:<base64-digest>:`

Example:
```
Content-Digest: sha-256=:uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek=:
```

#### X-Correlation-ID Header

Optional but recommended for request tracing:

```
X-Correlation-ID: req-550e8400-e29b-41d4-a716-446655440000
```

## API Endpoints

### Authentication Management

#### Register Public Key

Register a client's public key with the server.

```http
POST /api/v1/auth/keys HTTP/1.1
```

**Request Body:**
```json
{
  "key_id": "client-key-123",
  "public_key": "302a300506032b657003210000112233445566778899aabbccddeeff",
  "client_id": "my-application",
  "description": "Production client key",
  "security_profile": "strict"
}
```

**Response:**
```json
{
  "success": true,
  "key_id": "client-key-123",
  "status": "active",
  "registered_at": "2024-01-01T00:00:00Z",
  "expires_at": "2025-01-01T00:00:00Z"
}
```

**Error Responses:**
```json
{
  "error": "INVALID_PUBLIC_KEY",
  "message": "Public key format is invalid",
  "correlation_id": "req-550e8400-e29b-41d4-a716-446655440000"
}
```

#### List Registered Keys

List all registered public keys for a client.

```http
GET /api/v1/auth/keys HTTP/1.1
```

**Query Parameters:**
- `client_id` (optional) - Filter by client ID
- `status` (optional) - Filter by status (active, revoked, expired)

**Response:**
```json
{
  "keys": [
    {
      "key_id": "client-key-123",
      "client_id": "my-application", 
      "status": "active",
      "registered_at": "2024-01-01T00:00:00Z",
      "last_used": "2024-01-15T12:30:00Z"
    }
  ],
  "total": 1
}
```

#### Revoke Public Key

Revoke a registered public key.

```http
DELETE /api/v1/auth/keys/{key_id} HTTP/1.1
```

**Request Body:**
```json
{
  "reason": "key_compromise",
  "immediate": true
}
```

**Response:**
```json
{
  "success": true,
  "key_id": "client-key-123",
  "status": "revoked",
  "revoked_at": "2024-01-15T15:45:00Z"
}
```

### Data Operations

All data operations require mandatory authentication with the headers shown above.

#### Query Data

Execute a query against DataFold schemas.

```http
POST /api/v1/query HTTP/1.1
Host: api.datafold.com
Content-Type: application/json
Content-Digest: sha-256=:queryDigestHere=:
Signature-Input: sig1=("@method" "@target-uri" "content-type" "content-digest");created=1640995200;keyid="client-key-123";alg="ed25519";nonce="unique-nonce-here"
Signature: sig1=:signatureHere:

{
  "schema": "users",
  "query": {
    "fields": ["id", "name", "email"],
    "filters": {
      "active": true
    },
    "limit": 100
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": 1,
      "name": "John Doe",
      "email": "john@example.com"
    }
  ],
  "total": 1,
  "correlation_id": "req-550e8400-e29b-41d4-a716-446655440000"
}
```

#### Create/Update Data

Mutate data in DataFold schemas.

```http
POST /api/v1/mutate HTTP/1.1
Host: api.datafold.com
Content-Type: application/json
Content-Digest: sha-256=:mutationDigestHere=:
Signature-Input: sig1=("@method" "@target-uri" "content-type" "content-digest");created=1640995200;keyid="client-key-123";alg="ed25519";nonce="unique-nonce-here"
Signature: sig1=:signatureHere:

{
  "schema": "users",
  "operation": "create",
  "data": {
    "name": "Jane Smith",
    "email": "jane@example.com",
    "active": true
  }
}
```

**Response:**
```json
{
  "success": true,
  "operation": "create",
  "id": 2,
  "data": {
    "id": 2,
    "name": "Jane Smith", 
    "email": "jane@example.com",
    "active": true,
    "created_at": "2024-01-15T16:00:00Z"
  },
  "correlation_id": "req-550e8400-e29b-41d4-a716-446655440000"
}
```

### Schema Management

#### List Schemas

Get available schemas.

```http
GET /api/v1/schemas HTTP/1.1
Host: api.datafold.com
Signature-Input: sig1=("@method" "@target-uri");created=1640995200;keyid="client-key-123";alg="ed25519";nonce="unique-nonce-here"
Signature: sig1=:signatureHere:
```

**Response:**
```json
{
  "schemas": [
    {
      "name": "users",
      "version": "1.0",
      "fields": [
        {"name": "id", "type": "integer", "primary_key": true},
        {"name": "name", "type": "string", "required": true},
        {"name": "email", "type": "string", "required": true}
      ]
    }
  ]
}
```

#### Create Schema

Create a new schema.

```http
POST /api/v1/schemas HTTP/1.1
Host: api.datafold.com
Content-Type: application/json
Content-Digest: sha-256=:schemaDigestHere=:
Signature-Input: sig1=("@method" "@target-uri" "content-type" "content-digest");created=1640995200;keyid="client-key-123";alg="ed25519";nonce="unique-nonce-here"
Signature: sig1=:signatureHere:

{
  "name": "products",
  "version": "1.0",
  "fields": [
    {"name": "id", "type": "integer", "primary_key": true},
    {"name": "name", "type": "string", "required": true},
    {"name": "price", "type": "decimal", "required": true}
  ]
}
```

## Authentication Error Responses

### Common Error Codes

All authentication errors return structured error responses:

#### MISSING_HEADERS (400)
```json
{
  "error": "MISSING_HEADERS",
  "message": "Missing required authentication headers. Please include Signature-Input and Signature headers.",
  "missing_headers": ["signature-input", "signature"],
  "correlation_id": "req-550e8400-e29b-41d4-a716-446655440000",
  "documentation": "https://docs.datafold.dev/authentication/setup"
}
```

#### INVALID_SIGNATURE_FORMAT (400)
```json
{
  "error": "INVALID_SIGNATURE_FORMAT", 
  "message": "Invalid signature format. Please verify your signature encoding and header format.",
  "details": "Signature header must follow RFC 9421 format",
  "correlation_id": "req-550e8400-e29b-41d4-a716-446655440000",
  "documentation": "https://docs.datafold.dev/authentication/signature-format"
}
```

#### SIGNATURE_VERIFICATION_FAILED (401)
```json
{
  "error": "SIGNATURE_VERIFICATION_FAILED",
  "message": "Signature verification failed. Please check your signature calculation and key.", 
  "key_id": "client-key-123",
  "correlation_id": "req-550e8400-e29b-41d4-a716-446655440000",
  "documentation": "https://docs.datafold.dev/authentication/troubleshooting"
}
```

#### TIMESTAMP_VALIDATION_FAILED (401)
```json
{
  "error": "TIMESTAMP_VALIDATION_FAILED",
  "message": "Request timestamp invalid. Please ensure timestamp is within allowed time window.",
  "timestamp": 1640995200,
  "current_time": 1640995800,
  "max_age": 300,
  "correlation_id": "req-550e8400-e29b-41d4-a716-446655440000",
  "documentation": "https://docs.datafold.dev/authentication/troubleshooting#timestamp-issues"
}
```

#### NONCE_VALIDATION_FAILED (401)
```json
{
  "error": "NONCE_VALIDATION_FAILED",
  "message": "Request validation failed. Please use a unique nonce for each request.",
  "nonce": "550e8400-e29b-41d4-a716-446655440000",
  "reason": "nonce_already_used",
  "correlation_id": "req-550e8400-e29b-41d4-a716-446655440000",
  "documentation": "https://docs.datafold.dev/authentication/troubleshooting#nonce-validation"
}
```

#### PUBLIC_KEY_LOOKUP_FAILED (401)
```json
{
  "error": "PUBLIC_KEY_LOOKUP_FAILED",
  "message": "Authentication failed. Please verify your key ID is registered.",
  "key_id": "client-key-123",
  "correlation_id": "req-550e8400-e29b-41d4-a716-446655440000",
  "documentation": "https://docs.datafold.dev/authentication/key-management"
}
```

#### RATE_LIMIT_EXCEEDED (429)
```json
{
  "error": "RATE_LIMIT_EXCEEDED",
  "message": "Rate limit exceeded. Please reduce request frequency and try again later.",
  "client_id": "my-application",
  "retry_after": 60,
  "correlation_id": "req-550e8400-e29b-41d4-a716-446655440000",
  "documentation": "https://docs.datafold.dev/authentication/rate-limits"
}
```

## SDK Examples

### JavaScript/TypeScript

```typescript
import { DataFoldApiClient, SecurityProfile } from '@datafold/api-client';

const client = new DataFoldApiClient({
  baseUrl: 'https://api.datafold.com',
  authenticationRequired: true, // Always true, cannot be disabled
  securityProfile: SecurityProfile.STRICT,
  credentials: {
    keyId: 'client-key-123',
    privateKey: await loadPrivateKey('client-key-123'),
    clientId: 'my-application'
  }
});

// All requests are automatically signed
try {
  const response = await client.query({
    schema: 'users',
    query: {
      fields: ['id', 'name', 'email'],
      filters: { active: true }
    }
  });
  
  console.log('Query result:', response.data);
} catch (error) {
  if (error.code === 'SIGNATURE_VERIFICATION_FAILED') {
    console.error('Authentication failed:', error.message);
    // Handle authentication error
  }
}
```

### Python

```python
from datafold_api import DataFoldApiClient, SecurityProfile
import os

client = DataFoldApiClient(
    base_url='https://api.datafold.com',
    authentication_required=True,  # Always True, cannot be disabled
    security_profile=SecurityProfile.STRICT,
    credentials={
        'key_id': 'client-key-123',
        'private_key': load_private_key('client-key-123'),
        'client_id': 'my-application'
    }
)

# All requests are automatically signed
try:
    response = client.query(
        schema='users',
        query={
            'fields': ['id', 'name', 'email'],
            'filters': {'active': True}
        }
    )
    
    print('Query result:', response['data'])
except SignatureAuthenticationError as e:
    print(f'Authentication failed: {e.message}')
    print(f'Correlation ID: {e.correlation_id}')
```

### cURL Examples

#### Basic Query with Authentication

```bash
#!/bin/bash

# Configuration
SERVER_URL="https://api.datafold.com"
KEY_ID="client-key-123"
PRIVATE_KEY_FILE="~/.datafold/keys/client-key-123.pem"
NONCE=$(uuidgen)
TIMESTAMP=$(date +%s)

# Request body
BODY='{"schema":"users","query":{"fields":["id","name"],"limit":10}}'

# Calculate content digest
CONTENT_DIGEST="sha-256=$(echo -n "$BODY" | openssl dgst -sha256 -binary | base64)"

# Create signature input
SIGNATURE_INPUT="sig1=(\"@method\" \"@target-uri\" \"content-type\" \"content-digest\");created=$TIMESTAMP;keyid=\"$KEY_ID\";alg=\"ed25519\";nonce=\"$NONCE\""

# Create canonical message (simplified - actual implementation more complex)
CANONICAL_MESSAGE="\"@method\": POST
\"@target-uri\": $SERVER_URL/api/v1/query  
\"content-type\": application/json
\"content-digest\": $CONTENT_DIGEST
\"@signature-params\": $SIGNATURE_INPUT"

# Sign the canonical message (requires Ed25519 signing tool)
SIGNATURE=$(echo -n "$CANONICAL_MESSAGE" | openssl pkeyutl -sign -inkey "$PRIVATE_KEY_FILE" | base64)

# Make the request
curl -X POST "$SERVER_URL/api/v1/query" \
  -H "Content-Type: application/json" \
  -H "Content-Digest: $CONTENT_DIGEST" \
  -H "Signature-Input: $SIGNATURE_INPUT" \
  -H "Signature: sig1=:$SIGNATURE:" \
  -H "X-Correlation-ID: req-$NONCE" \
  -d "$BODY"
```

## Security Considerations

### Time Synchronization

Ensure all clients have properly synchronized clocks:

```bash
# Install NTP
sudo apt install ntp

# Enable time synchronization
sudo systemctl enable ntp

# Force synchronization
sudo ntpdate -s time.nist.gov
```

### Key Management

- Store private keys securely (HSM, secure vault, encrypted storage)
- Use separate keys for different environments
- Implement regular key rotation procedures
- Monitor key usage and access patterns

### Network Security

- Always use HTTPS/TLS for API communications
- Verify server certificates
- Use proper firewall rules
- Monitor for suspicious network activity

### Request Security

- Use unique nonces for every request
- Include appropriate signature components
- Validate response signatures if server supports it
- Log authentication events for audit purposes

## Rate Limiting

DataFold implements rate limiting for authentication requests:

- **Standard Profile**: 1000 requests per minute per client
- **Strict Profile**: 500 requests per minute per client  
- **Failed Attempts**: 10 failed authentications per 5 minutes

When rate limits are exceeded, implement exponential backoff:

```javascript
async function retryWithBackoff(requestFn, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await requestFn();
    } catch (error) {
      if (error.code === 'RATE_LIMIT_EXCEEDED' && i < maxRetries - 1) {
        const delay = Math.min(1000 * Math.pow(2, i), 30000);
        await new Promise(resolve => setTimeout(resolve, delay));
        continue;
      }
      throw error;
    }
  }
}
```

## Monitoring and Alerting

Monitor these authentication metrics:

- Authentication success/failure rates
- Signature verification latency
- Rate limiting events
- Key usage patterns
- Timestamp validation failures
- Nonce validation failures

Set up alerts for:

- High authentication failure rates
- Unusual request patterns
- Key compromise indicators
- Performance degradation

## Compliance and Auditing

DataFold authentication supports compliance requirements:

- **SOC 2**: Comprehensive audit logging and access controls
- **GDPR**: Request correlation and data processing tracking
- **HIPAA**: Cryptographic authentication and audit trails
- **PCI DSS**: Strong authentication and monitoring

All authentication events include correlation IDs for audit trail tracking.

For additional information, see:
- [Authentication Setup Guide](../guides/cli-authentication.md)
- [Troubleshooting Guide](../guides/troubleshooting.md)
- [Security Best Practices](../security/recipes/authentication-flow.md)
- [Production Deployment Guide](../deployment-guide.md)