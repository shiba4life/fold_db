# Digital Signature Verification API

This document describes the REST API endpoint for verifying Ed25519 digital signatures from registered clients in the DataFold HTTP server.

## Overview

The signature verification API allows clients to verify digital signatures using their previously registered Ed25519 public keys. This enables secure authentication and message integrity verification workflows.

**Base URL**: `http://localhost:9001/api/crypto`

## Authentication

No authentication is required to access these endpoints, but clients must have previously registered their public keys using the [Public Key Registration API](./public-key-registration.md).

## Endpoints

### Verify Digital Signature

Verifies a digital signature against a registered client's public key.

**Endpoint**: `POST /signatures/verify`

#### Request Body

```json
{
  "client_id": "string",
  "message": "string",
  "signature": "string",
  "message_encoding": "string (optional)",
  "metadata": {
    "key": "value (optional)"
  }
}
```

#### Request Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `client_id` | string | Yes | The unique identifier for the client whose public key should be used for verification |
| `message` | string | Yes | The message data to verify. Encoding depends on `message_encoding` parameter |
| `signature` | string | Yes | Hex-encoded Ed25519 signature (128 hex characters = 64 bytes) |
| `message_encoding` | string | No | Message encoding format: `"utf8"` (default), `"hex"`, or `"base64"` |
| `metadata` | object | No | Optional metadata for audit/logging purposes |

#### Message Encoding Formats

- **`utf8`** (default): Message is treated as UTF-8 text
- **`hex`**: Message is hex-encoded binary data
- **`base64`**: Message is base64-encoded binary data

#### Response

**Success Response** (200 OK):
```json
{
  "success": true,
  "data": {
    "verified": true,
    "client_id": "test_client_001",
    "public_key": "a1b2c3d4e5f6...",
    "verified_at": "2025-06-08T22:13:45.123456Z",
    "message_hash": "sha256_hash_of_message_in_hex"
  }
}
```

**Error Response** (Various status codes):
```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable error message",
    "details": {}
  }
}
```

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `verified` | boolean | Always `true` for successful verification |
| `client_id` | string | The client ID that was verified |
| `public_key` | string | Hex-encoded public key used for verification |
| `verified_at` | string | ISO 8601 timestamp of verification |
| `message_hash` | string | SHA-256 hash of the message (for audit trail) |

#### Status Codes

| Code | Description | Error Codes |
|------|-------------|-------------|
| 200 | Signature verification successful | N/A |
| 400 | Bad Request | `INVALID_CLIENT_ID`, `INVALID_MESSAGE`, `INVALID_SIGNATURE`, `INVALID_ENCODING` |
| 401 | Unauthorized | `SIGNATURE_VERIFICATION_FAILED` |
| 403 | Forbidden | `KEY_NOT_ACTIVE` |
| 404 | Not Found | `CLIENT_NOT_FOUND`, `REGISTRATION_NOT_FOUND` |
| 500 | Internal Server Error | `DATABASE_ERROR`, `INTERNAL_ERROR` |

#### Error Codes

- **`INVALID_CLIENT_ID`**: Client ID is empty or invalid
- **`INVALID_MESSAGE`**: Message is empty or invalid for the specified encoding
- **`INVALID_SIGNATURE`**: Signature is not valid hex or wrong length (must be 64 bytes)
- **`INVALID_ENCODING`**: Message encoding must be 'utf8', 'hex', or 'base64'
- **`SIGNATURE_VERIFICATION_FAILED`**: Digital signature verification failed
- **`KEY_NOT_ACTIVE`**: Public key status is not 'active'
- **`CLIENT_NOT_FOUND`**: No public key registered for this client
- **`REGISTRATION_NOT_FOUND`**: Registration record not found
- **`DATABASE_ERROR`**: Database operation failed
- **`INTERNAL_ERROR`**: Internal server error

## Examples

### Example 1: Verify UTF-8 Text Message

**Request**:
```bash
curl -X POST http://localhost:9001/api/crypto/signatures/verify \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "my_app_client_001",
    "message": "Hello, DataFold!",
    "signature": "a1b2c3d4e5f67890abcdef1234567890fedcba0987654321abcdef1234567890a1b2c3d4e5f67890abcdef1234567890fedcba0987654321abcdef1234567890",
    "message_encoding": "utf8"
  }'
```

**Response**:
```json
{
  "success": true,
  "data": {
    "verified": true,
    "client_id": "my_app_client_001",
    "public_key": "1234567890abcdef1234567890abcdef12345678",
    "verified_at": "2025-06-08T22:13:45.123456Z",
    "message_hash": "185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969"
  }
}
```

### Example 2: Verify Hex-Encoded Binary Data

**Request**:
```bash
curl -X POST http://localhost:9001/api/crypto/signatures/verify \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "binary_client_002",
    "message": "48656c6c6f2c20446174614f6c6421",
    "signature": "b2c3d4e5f67890a1bcdef1234567890fedcba0987654321abcdef1234567890a1b2c3d4e5f67890abcdef1234567890fedcba0987654321abcdef1234567890",
    "message_encoding": "hex"
  }'
```

### Example 3: Verify Base64-Encoded Data

**Request**:
```bash
curl -X POST http://localhost:9001/api/crypto/signatures/verify \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "base64_client_003",
    "message": "SGVsbG8sIERhdGFGb2xkIQ==",
    "signature": "c3d4e5f67890a1b2def1234567890fedcba0987654321abcdef1234567890a1b2c3d4e5f67890abcdef1234567890fedcba0987654321abcdef1234567890",
    "message_encoding": "base64"
  }'
```

### Example 4: Verification Failure

**Request**:
```bash
curl -X POST http://localhost:9001/api/crypto/signatures/verify \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "my_app_client_001",
    "message": "Hello, DataFold!",
    "signature": "invalid_signature_that_will_fail_verification_000000000000000000000000000000000000000000000000000000000000",
    "message_encoding": "utf8"
  }'
```

**Response** (401 Unauthorized):
```json
{
  "success": false,
  "error": {
    "code": "SIGNATURE_VERIFICATION_FAILED",
    "message": "Digital signature verification failed",
    "details": {}
  }
}
```

### Example 5: Unregistered Client

**Request**:
```bash
curl -X POST http://localhost:9001/api/crypto/signatures/verify \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "unregistered_client",
    "message": "Hello, DataFold!",
    "signature": "a1b2c3d4e5f67890abcdef1234567890fedcba0987654321abcdef1234567890a1b2c3d4e5f67890abcdef1234567890fedcba0987654321abcdef1234567890",
    "message_encoding": "utf8"
  }'
```

**Response** (404 Not Found):
```json
{
  "success": false,
  "error": {
    "code": "CLIENT_NOT_FOUND",
    "message": "No public key registered for this client",
    "details": {}
  }
}
```

## Integration Workflow

### 1. Client Registration
Before using signature verification, clients must register their public keys:

```bash
# Register public key
curl -X POST http://localhost:9001/api/crypto/keys/register \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "my_app_client_001",
    "public_key": "1234567890abcdef1234567890abcdef12345678",
    "key_name": "Production Key"
  }'
```

### 2. Generate Signature (Client-Side)
Using an Ed25519 library, sign your message:

```javascript
// Example with Node.js and @noble/ed25519
import { sign } from '@noble/ed25519';

const privateKey = '...'; // Your private key
const message = 'Hello, DataFold!';
const messageBytes = new TextEncoder().encode(message);
const signature = await sign(messageBytes, privateKey);
const signatureHex = Buffer.from(signature).toString('hex');
```

### 3. Verify Signature
Send the signature verification request:

```bash
curl -X POST http://localhost:9001/api/crypto/signatures/verify \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "my_app_client_001",
    "message": "Hello, DataFold!",
    "signature": "'$signatureHex'",
    "message_encoding": "utf8"
  }'
```

## Security Considerations

### Signature Format
- Only Ed25519 signatures are supported
- Signatures must be exactly 64 bytes (128 hex characters)
- Invalid signature formats are rejected with detailed error messages

### Message Encoding
- UTF-8 encoding is the default and most common
- Hex encoding is useful for binary data
- Base64 encoding provides standard binary data encoding
- Invalid encoding specifications are rejected

### Public Key Status
- Only 'active' public keys can be used for verification
- Revoked or suspended keys will return `KEY_NOT_ACTIVE` error
- Key status is checked on every verification request

### Audit Trail
- All verification attempts are logged
- Message hashes are included in responses for audit purposes
- Last used timestamps are updated on successful verifications
- Failed verification attempts are logged with client details

### Rate Limiting
Consider implementing rate limiting in production environments to prevent abuse of the verification endpoint.

## Dependencies

This endpoint depends on:
- [Public Key Registration API](./public-key-registration.md) for client key management
- Ed25519 signature verification using the `ed25519-dalek` library
- SHA-256 hashing for message integrity audit trail

## Related APIs

- [Public Key Registration](./public-key-registration.md) - Register and manage client public keys
- [Crypto Initialization](./crypto-initialization.md) - Initialize database crypto system