# JavaScript SDK Server Integration Guide

This guide covers server integration for the DataFold JavaScript SDK, including public key registration and signature verification workflows.

## Quick Start

```typescript
import { createServerIntegration, generateKeyPair } from '@datafold/js-sdk';

// Create server integration instance
const integration = createServerIntegration({
  baseUrl: 'https://api.datafold.com',
  timeout: 30000,
  retries: 3
});

// Generate key pair
const keyPair = await generateKeyPair();

// Complete registration and verification workflow
const result = await integration.registerAndVerifyWorkflow(
  keyPair,
  'Test message',
  {
    clientId: 'my-client-123',
    keyName: 'My Application Key'
  }
);

console.log('Registration successful:', result.registration);
console.log('Verification successful:', result.verification.verified);
```

## Configuration

### Server Connection Configuration
```typescript
interface ServerConnectionConfig {
  baseUrl: string;           // DataFold server URL
  timeout?: number;          // Request timeout in milliseconds (default: 30000)
  retries?: number;          // Number of retry attempts (default: 3)
  retryDelay?: number;       // Base delay between retries (default: 1000)
  retryConfig?: RetryConfig; // Advanced retry configuration
}
```

### Retry Configuration
```typescript
interface RetryConfig {
  maxRetries: number;     // Maximum retry attempts
  baseDelay: number;      // Base delay in milliseconds
  maxDelay: number;       // Maximum delay in milliseconds
  backoffFactor: number;  // Exponential backoff factor
}
```

## Core Operations

### 1. Test Server Connection
```typescript
const connection = await integration.testConnection();
if (connection.connected) {
  console.log(`Connected with ${connection.latency}ms latency`);
} else {
  console.error('Connection failed:', connection.error);
}
```

### 2. Register Public Key
```typescript
const keyPair = await generateKeyPair();
const registration = await integration.registerKeyPair(keyPair, {
  clientId: 'my-client-123',
  userId: 'user-456',
  keyName: 'My Application Key',
  metadata: {
    application: 'MyApp',
    version: '1.0.0'
  }
});

console.log('Registration ID:', registration.registrationId);
```

### 3. Check Registration Status
```typescript
const status = await integration.checkRegistrationStatus('my-client-123');
if (status.registered) {
  console.log('Client is registered:', status.registration);
} else {
  console.log('Client not registered:', status.error);
}
```

### 4. Generate and Verify Signature
```typescript
// Generate signature
const signature = await integration.generateSignature(
  'Hello, DataFold!',
  keyPair.privateKey,
  { messageEncoding: 'utf8' }
);

// Verify with server
const verification = await integration.verifySignature({
  clientId: 'my-client-123',
  message: 'Hello, DataFold!',
  signature: signature.signature,
  messageEncoding: 'utf8'
});

console.log('Signature verified:', verification.verified);
```

## Error Handling

### Standard Error Codes
The JavaScript SDK handles these standard error codes:

- `INVALID_PUBLIC_KEY`: Malformed or invalid public key
- `CLIENT_ALREADY_REGISTERED`: Client already has registered key
- `CLIENT_NOT_FOUND`: No registration found for client
- `SIGNATURE_VERIFICATION_FAILED`: Digital signature verification failed
- `MAX_RETRIES_EXCEEDED`: Request failed after maximum retries

### Error Handling Example
```typescript
try {
  const registration = await integration.registerKeyPair(keyPair, options);
  console.log('Registration successful');
} catch (error) {
  if (error instanceof DataFoldServerError) {
    switch (error.errorCode) {
      case 'CLIENT_ALREADY_REGISTERED':
        console.log('Client already registered, checking status...');
        break;
      case 'INVALID_PUBLIC_KEY':
        console.error('Invalid public key format');
        break;
      default:
        console.error('Server error:', error.message);
    }
  } else {
    console.error('Unexpected error:', error);
  }
}
```

## Advanced Usage

### Custom Retry Configuration
```typescript
const integration = createServerIntegration({
  baseUrl: 'https://api.datafold.com',
  retryConfig: {
    maxRetries: 5,
    baseDelay: 500,
    maxDelay: 30000,
    backoffFactor: 2.5
  }
});
```

### Message Encoding Options
```typescript
// UTF-8 string (default)
const signature1 = await integration.generateSignature(
  'Hello, world!',
  privateKey,
  { messageEncoding: 'utf8' }
);

// Hex-encoded bytes
const signature2 = await integration.generateSignature(
  '48656c6c6f2c20776f726c6421',
  privateKey,
  { messageEncoding: 'hex' }
);

// Base64-encoded bytes
const signature3 = await integration.generateSignature(
  'SGVsbG8sIHdvcmxkIQ==',
  privateKey,
  { messageEncoding: 'base64' }
);
```

## Integration Testing

### Quick Integration Test
```typescript
import { quickIntegrationTest } from '@datafold/js-sdk/server';

const result = await quickIntegrationTest({
  baseUrl: 'https://api.datafold.com'
});

if (result.success) {
  console.log('Integration test passed:', result.results);
} else {
  console.error('Integration test failed:', result.error);
}
```

## Best Practices

### 1. Key Storage
- Store private keys securely using the storage API
- Never transmit private keys to the server
- Use secure storage for long-term key persistence

### 2. Error Handling
- Always handle network errors gracefully
- Implement appropriate retry logic for transient failures
- Provide meaningful error messages to users

### 3. Configuration
- Use appropriate timeout values for your use case
- Configure retry settings based on network conditions
- Enable SSL verification in production

### 4. Testing
- Test with various message encodings
- Verify error handling for all error scenarios
- Use integration tests for end-to-end validation

## API Reference

For complete API documentation, see the TypeScript definitions and generated API docs.

## Troubleshooting

### Common Issues

1. **Connection Timeout**: Increase timeout value or check network connectivity
2. **SSL Certificate Errors**: Verify server certificate or disable SSL verification for testing
3. **Invalid Key Format**: Ensure keys are properly formatted as hex strings
4. **Client Already Registered**: Check existing registration or use different client ID

### Debug Mode
Enable debug logging to troubleshoot issues:

```typescript
// Browser console will show detailed request/response logs
localStorage.setItem('datafold-debug', 'true');