# Public Key Registration API

This document describes the REST API endpoints for client public key registration and management in the DataFold http_server.

## Base URL

All endpoints are prefixed with `/api/crypto/keys`

## Endpoints

### Register Public Key

Register a client's Ed25519 public key for authentication and signature verification.

**Endpoint:** `POST /api/crypto/keys/register`

**Request Body:**
```json
{
  "client_id": "optional-client-identifier",
  "user_id": "optional-user-identifier", 
  "public_key": "hex-encoded-ed25519-public-key",
  "key_name": "optional-human-readable-key-name",
  "metadata": {
    "optional": "metadata",
    "environment": "production"
  }
}
```

**Request Fields:**
- `client_id` (optional): Unique identifier for the client. If not provided, one will be auto-generated.
- `user_id` (optional): Associated user identifier
- `public_key` (required): 64-character hex-encoded Ed25519 public key (32 bytes)
- `key_name` (optional): Human-readable name for the key
- `metadata` (optional): Additional key-value metadata

**Response (201 Created):**
```json
{
  "success": true,
  "data": {
    "registration_id": "uuid-v4",
    "client_id": "client-identifier",
    "public_key": "hex-encoded-public-key",
    "key_name": "optional-key-name",
    "registered_at": "2025-06-08T15:00:00Z",
    "status": "active"
  }
}
```

**Error Responses:**

- **400 Bad Request**: Invalid public key format
```json
{
  "success": false,
  "error": {
    "code": "INVALID_PUBLIC_KEY",
    "message": "Ed25519 public key must be exactly 32 bytes",
    "details": {}
  }
}
```

- **409 Conflict**: Client already registered or duplicate public key
```json
{
  "success": false,
  "error": {
    "code": "CLIENT_ALREADY_REGISTERED",
    "message": "Client already has a registered public key. Use update endpoint to change keys.",
    "details": {}
  }
}
```

### Get Public Key Status

Retrieve the registration status and details for a client's public key.

**Endpoint:** `GET /api/crypto/keys/status/{client_id}`

**Path Parameters:**
- `client_id`: The client identifier to look up

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "registration_id": "uuid-v4",
    "client_id": "client-identifier",
    "public_key": "hex-encoded-public-key",
    "key_name": "optional-key-name",
    "registered_at": "2025-06-08T15:00:00Z",
    "status": "active",
    "last_used": null
  }
}
```

**Error Responses:**

- **404 Not Found**: Client not registered
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

## Security Considerations

### Public Key Validation

- All public keys must be valid Ed25519 public keys (32 bytes)
- Keys are validated using the `ed25519-dalek` library
- Invalid key formats are rejected with detailed error messages

### Duplicate Prevention

- Each client can only register one public key at a time
- Each public key can only be registered once across all clients
- Attempts to register duplicate keys return `409 Conflict`

### Storage Security

- Public keys are stored with SHA-256 hashes for efficient duplicate detection
- All registration data includes integrity timestamps
- Database operations are atomic to prevent partial registrations

## Rate Limiting

Currently no rate limiting is implemented, but it should be added for production use to prevent abuse of the registration endpoints.

## Examples

### Register a new public key

```bash
curl -X POST http://localhost:9001/api/crypto/keys/register \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "my-app-client",
    "public_key": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    "key_name": "Production Key",
    "metadata": {
      "environment": "production",
      "version": "1.0.0"
    }
  }'
```

### Check registration status

```bash
curl http://localhost:9001/api/crypto/keys/status/my-app-client
```

## Integration Notes

### Client Libraries

This API is designed to integrate with:
- JavaScript SDK (task 10-2-5)
- Python SDK (task 10-3-5) 
- CLI tools (task 10-4-5)

### Future Enhancements

Planned features for future releases:
- Key rotation/update endpoints
- Key revocation
- Signature verification endpoints (task 10-6-2)
- Key expiration and lifecycle management
- Enhanced metadata and querying capabilities