# Standardized Server Integration Patterns

This document defines the standardized integration patterns implemented across all DataFold client platforms (JavaScript SDK, Python SDK, and CLI) for consistent server communication.

## API Endpoint Standardization

All platforms use consistent API endpoints and parameter naming:

### Public Key Registration
- **Endpoint**: `POST /api/crypto/keys/register`
- **Request Parameters** (snake_case):
  - `client_id`: Optional client identifier (auto-generated if not provided)
  - `user_id`: Optional user identifier
  - `public_key`: Hex-encoded Ed25519 public key (32 bytes)
  - `key_name`: Optional human-readable key name
  - `metadata`: Optional metadata dictionary

### Signature Verification  
- **Endpoint**: `POST /api/crypto/signatures/verify`
- **Request Parameters** (snake_case):
  - `client_id`: Client identifier
  - `message`: Message content
  - `signature`: Hex-encoded Ed25519 signature (64 bytes)
  - `message_encoding`: Encoding format ('utf8', 'hex', 'base64')
  - `metadata`: Optional context metadata

### Registration Status
- **Endpoint**: `GET /api/crypto/keys/status/{client_id}`
- **Response**: Registration details and current status

## Configuration Standardization

All platforms support consistent configuration options:

```typescript
interface ServerConnectionConfig {
  baseUrl: string;           // Server base URL
  timeout: number;           // Request timeout in milliseconds/seconds
  retries: number;           // Number of retry attempts
  retryDelay: number;        // Base delay between retries
  verifySSL?: boolean;       // SSL certificate verification
}
```

## Error Handling Standardization

### Common Error Codes
All platforms handle these standard error codes consistently:

- `INVALID_PUBLIC_KEY`: Malformed or invalid public key
- `INVALID_CLIENT_ID`: Missing or invalid client identifier
- `INVALID_MESSAGE`: Empty or malformed message
- `INVALID_SIGNATURE`: Malformed or invalid signature
- `INVALID_ENCODING`: Unsupported message encoding
- `CLIENT_ALREADY_REGISTERED`: Client already has registered key
- `DUPLICATE_PUBLIC_KEY`: Public key already registered by another client
- `CLIENT_NOT_FOUND`: No registration found for client
- `REGISTRATION_NOT_FOUND`: Registration record not found
- `KEY_NOT_ACTIVE`: Public key status is not active
- `SIGNATURE_VERIFICATION_FAILED`: Digital signature verification failed

### Error Response Format
```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {}
  }
}
```

### Retry Logic
- Automatic retry on network errors and 5xx server errors
- No retry on 4xx client errors (except 429 rate limiting)
- Exponential backoff with configurable base delay and max delay
- Configurable maximum retry attempts

## Workflow Standardization

### Complete Registration and Verification Workflow

All platforms implement this standardized workflow:

1. **Key Generation**: Generate Ed25519 key pair locally
2. **Server Registration**: Register public key with server
3. **Message Signing**: Sign test message with private key
4. **Server Verification**: Verify signature using server endpoint
5. **Status Validation**: Confirm registration status

### JavaScript SDK Example
```typescript
import { createServerIntegration, generateKeyPair } from '@datafold/js-sdk';

const integration = createServerIntegration({
  baseUrl: 'https://api.datafold.com',
  timeout: 30000
});

const keyPair = await generateKeyPair();
const workflow = await integration.registerAndVerifyWorkflow(
  keyPair,
  'Test message',
  { clientId: 'client_123' }
);
```

### Python SDK Example
```python
from datafold_sdk.integration import register_and_verify_workflow

session, result = register_and_verify_workflow(
    'https://api.datafold.com',
    'Test message',
    'client_123'
)
```

### CLI Example
```bash
datafold-cli server test \
  --server-url https://api.datafold.com \
  --client-id client_123 \
  --test-message "Test message"
```

## Security Standardization

### Key Management
- Private keys never leave the client
- Secure storage using platform-appropriate mechanisms:
  - Browser: IndexedDB with encryption
  - Python: OS keychain/encrypted files
  - CLI: Encrypted file storage with proper permissions

### Message Encoding
All platforms support consistent message encoding:
- `utf8`: UTF-8 string encoding (default)
- `hex`: Hexadecimal byte encoding
- `base64`: Base64 byte encoding

### Signature Format
- Ed25519 signatures always 64 bytes
- Hex-encoded in API communication
- Consistent verification across all platforms

## Testing Standardization

### Cross-Platform Test Coverage
- Connection testing with valid/invalid servers
- Key generation and registration workflows
- Message signing and verification
- Error handling for all error codes
- Configuration validation
- Negative testing scenarios

### Test Scenarios
1. **Happy Path**: Complete workflow success
2. **Network Errors**: Connection failures, timeouts
3. **Server Errors**: 4xx/5xx responses
4. **Invalid Data**: Malformed keys, signatures, messages
5. **Edge Cases**: Empty data, oversized data, special characters

## Implementation Status

### ✅ Completed Integrations
- **JavaScript SDK**: Full server integration with standardized patterns
- **Python SDK**: Complete integration client with workflow support  
- **CLI**: Comprehensive command-line interface with all server operations
- **Server**: HTTP API endpoints with consistent error handling

### ✅ Standardized Components
- API endpoint definitions and parameter naming
- Error code definitions and response formats
- Configuration option consistency
- Retry logic and timeout handling
- Security patterns and key management
- Cross-platform test suite

### ✅ Documentation
- Integration patterns documentation
- API reference consistency
- Platform-specific setup guides
- Troubleshooting guides with common error codes

## Future Enhancements

### Planned Improvements
- WebSocket support for real-time operations
- Batch operations for multiple signatures
- Key rotation workflow standardization
- Advanced authentication patterns
- Performance optimization patterns

### Backward Compatibility
All changes maintain backward compatibility with existing client implementations while adding new standardized patterns.