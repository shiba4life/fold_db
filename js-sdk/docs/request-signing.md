# Request Signing with RFC 9421 HTTP Message Signatures

The DataFold JavaScript SDK provides comprehensive support for signing HTTP requests using RFC 9421 HTTP Message Signatures with Ed25519 cryptography. This ensures request authenticity and integrity when communicating with DataFold servers.

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Configuration](#configuration)
4. [Signing Requests](#signing-requests)
5. [HTTP Client Integration](#http-client-integration)
6. [Security Profiles](#security-profiles)
7. [Advanced Usage](#advanced-usage)
8. [Performance Considerations](#performance-considerations)
9. [Troubleshooting](#troubleshooting)

## Overview

The signing implementation follows RFC 9421 standards and includes:

- **Ed25519 Digital Signatures**: Industry-standard elliptic curve cryptography
- **Canonical Message Construction**: Deterministic message formatting for signature verification
- **Signature Components**: Configurable inclusion of HTTP method, URI, headers, and content digest
- **Timestamp and Nonce Protection**: Replay attack prevention
- **Automatic HTTP Client Integration**: Seamless request signing

## Quick Start

```javascript
import { 
  generateKeyPair,
  createSigningConfig,
  RFC9421Signer 
} from '@datafold/js-sdk';

// 1. Generate a key pair
const keyPair = await generateKeyPair();

// 2. Create signing configuration
const config = createSigningConfig()
  .algorithm('ed25519')
  .keyId('my-client-001')
  .privateKey(keyPair.privateKey)
  .profile('standard')
  .build();

// 3. Create signer
const signer = new RFC9421Signer(config);

// 4. Sign a request
const request = {
  method: 'GET',
  url: 'https://api.datafold.com/api/crypto/keys/status/my-client-001',
  headers: {
    'user-agent': 'MyApp/1.0'
  }
};

const signatureResult = await signer.signRequest(request);

// 5. Use the signed headers in your HTTP request
const response = await fetch(request.url, {
  method: request.method,
  headers: {
    ...request.headers,
    ...signatureResult.headers
  }
});
```

## Configuration

### Security Profiles

The SDK provides three predefined security profiles:

#### Standard (Recommended)
```javascript
const config = createSigningConfig()
  .profile('standard')
  .keyId('my-client')
  .privateKey(privateKey)
  .build();
```

- **Components**: @method, @target-uri, content-type, content-digest
- **Digest Algorithm**: SHA-256
- **Nonce Validation**: Enabled
- **Use Case**: Most production applications

#### Strict (High Security)
```javascript
const config = createSigningConfig()
  .profile('strict')
  .keyId('my-client')
  .privateKey(privateKey)
  .build();
```

- **Components**: @method, @target-uri, content-type, content-length, user-agent, authorization, content-digest
- **Digest Algorithm**: SHA-512
- **Nonce Validation**: Strict UUID4 validation
- **Use Case**: High-security environments

#### Minimal (Low Latency)
```javascript
const config = createSigningConfig()
  .profile('minimal')
  .keyId('my-client')
  .privateKey(privateKey)
  .build();
```

- **Components**: @method, @target-uri
- **Content Digest**: Disabled
- **Use Case**: High-performance scenarios with reduced security

### Custom Configuration

```javascript
const config = createSigningConfig()
  .algorithm('ed25519')
  .keyId('my-client')
  .privateKey(privateKey)
  .components({
    method: true,
    targetUri: true,
    headers: ['content-type', 'authorization', 'x-api-version'],
    contentDigest: true
  })
  .nonceGenerator(() => generateCustomNonce())
  .timestampGenerator(() => Math.floor(Date.now() / 1000))
  .build();
```

## Signing Requests

### Basic Request Signing

```javascript
const request = {
  method: 'POST',
  url: 'https://api.datafold.com/api/crypto/keys/register',
  headers: {
    'content-type': 'application/json'
  },
  body: JSON.stringify({
    clientId: 'my-client',
    publicKey: 'hex-encoded-public-key'
  })
};

const result = await signer.signRequest(request);
```

### Signing Options

```javascript
const result = await signer.signRequest(request, {
  // Custom components for this request
  components: {
    method: true,
    targetUri: true,
    headers: ['content-type', 'x-request-id'],
    contentDigest: false
  },
  // Custom nonce
  nonce: 'custom-nonce-12345',
  // Custom timestamp
  timestamp: Math.floor(Date.now() / 1000),
  // Digest algorithm
  digestAlgorithm: 'sha-512'
});
```

### Batch Signing

```javascript
const requests = [
  { method: 'GET', url: 'https://api.datafold.com/endpoint1', headers: {} },
  { method: 'POST', url: 'https://api.datafold.com/endpoint2', headers: {}, body: '{"data": true}' }
];

const results = await signer.signRequests(requests);
```

## HTTP Client Integration

### Automatic Signing

```javascript
import { DataFoldHttpClient } from '@datafold/js-sdk';

// Create HTTP client
const client = new DataFoldHttpClient({
  baseUrl: 'https://api.datafold.com',
  timeout: 30000
});

// Configure automatic signing
client.configureSigning(signingConfig);

// All subsequent requests will be automatically signed
const registration = await client.registerPublicKey({
  clientId: 'my-client',
  publicKey: publicKeyHex,
  keyName: 'My Application Key'
});
```

### Manual Signing Control

```javascript
// Enable signing
client.enableSigning(signer);

// Make signed requests
const result1 = await client.getPublicKeyStatus('my-client');

// Disable signing temporarily
client.disableSigning();

// Make unsigned request
const result2 = await client.testConnection();

// Re-enable signing
client.enableSigning(signer);
```

## Advanced Usage

### Custom Canonical Message Construction

```javascript
import { buildCanonicalMessage } from '@datafold/js-sdk';

const context = {
  request: signableRequest,
  config: signingConfig,
  options: signingOptions,
  params: {
    created: Math.floor(Date.now() / 1000),
    keyid: 'my-client',
    alg: 'ed25519',
    nonce: generateNonce()
  },
  contentDigest: await calculateContentDigest(requestBody, 'sha-256')
};

const canonicalMessage = await buildCanonicalMessage(context);
```

### Signature Verification (Development/Testing)

```javascript
import { verifySignature } from '@datafold/js-sdk';

const isValid = await verifySignature(
  signableRequest,
  signatureResult,
  publicKey
);
```

### Key Management Integration

```javascript
import { 
  generateKeyPair,
  exportPrivateKey,
  importPrivateKey
} from '@datafold/js-sdk';

// Generate and export for storage
const keyPair = await generateKeyPair();
const exportedKey = await exportPrivateKey(keyPair.privateKey, 'password123');

// Later, import for signing
const importedKey = await importPrivateKey(exportedKey, 'password123');
const config = createSigningConfig()
  .keyId('my-client')
  .privateKey(importedKey)
  .profile('standard')
  .build();
```

## Performance Considerations

### Optimization Tips

1. **Reuse Signers**: Create signer instances once and reuse them
```javascript
const signer = new RFC9421Signer(config);
// Reuse this signer for multiple requests
```

2. **Minimize Signature Components**: Include only necessary headers
```javascript
const config = createSigningConfig()
  .components({
    method: true,
    targetUri: true,
    headers: ['content-type'], // Only essential headers
    contentDigest: true
  })
  .build();
```

3. **Batch Operations**: Use batch signing for multiple requests
```javascript
const results = await signer.signRequests(multipleRequests);
```

### Performance Targets

- **Single Request Signing**: <10ms (target)
- **Throughput**: >100 requests/second
- **Memory Usage**: Minimal impact with proper key management

### Performance Monitoring

```javascript
const start = performance.now();
const result = await signer.signRequest(request);
const elapsed = performance.now() - start;

if (elapsed > 10) {
  console.warn(`Signing took ${elapsed.toFixed(2)}ms (target: <10ms)`);
}
```

## Error Handling

### Common Errors

```javascript
try {
  const result = await signer.signRequest(request);
} catch (error) {
  if (error instanceof SigningError) {
    switch (error.code) {
      case 'INVALID_PRIVATE_KEY':
        console.error('Private key is invalid or corrupted');
        break;
      case 'SIGNING_FAILED':
        console.error('Ed25519 signing operation failed');
        break;
      case 'INVALID_URL':
        console.error('Request URL is malformed');
        break;
      case 'CRYPTO_UNAVAILABLE':
        console.error('Web Crypto API not available');
        break;
      default:
        console.error('Unknown signing error:', error.message);
    }
  }
}
```

### Error Recovery

```javascript
const signRequestWithRetry = async (request, maxRetries = 3) => {
  for (let attempt = 0; attempt < maxRetries; attempt++) {
    try {
      return await signer.signRequest(request);
    } catch (error) {
      if (attempt === maxRetries - 1) throw error;
      
      // Wait before retry
      await new Promise(resolve => setTimeout(resolve, 100 * (attempt + 1)));
    }
  }
};
```

## Browser Compatibility

### Requirements

- **Modern Browsers**: Chrome 60+, Firefox 55+, Safari 11+, Edge 79+
- **Web Crypto API**: Required for signature generation and content digests
- **Secure Context**: HTTPS required in production environments

### Compatibility Check

```javascript
import { initializeSDK } from '@datafold/js-sdk';

const { compatible, warnings } = await initializeSDK();

if (!compatible) {
  console.error('Browser not compatible:', warnings);
}
```

### Polyfills

For older browsers, consider polyfills:

```html
<!-- For older browsers missing crypto.getRandomValues -->
<script src="https://cdnjs.cloudflare.com/ajax/libs/seedrandom/3.0.5/seedrandom.min.js"></script>

<!-- For environments missing TextEncoder/TextDecoder -->
<script src="https://polyfill.io/v3/polyfill.min.js?features=TextEncoder%2CTextDecoder"></script>
```

## Troubleshooting

### Common Issues

1. **"Private key must be exactly 32 bytes"**
   - Ensure you're using a valid Ed25519 private key
   - Check key import/export formats

2. **"Invalid signature format"**
   - Verify request URL is properly formatted
   - Check that all required headers are present

3. **"Content digest required but not provided"**
   - Ensure requests with bodies include content-digest component
   - Verify body encoding matches content-type

4. **Performance issues**
   - Check for unnecessary signature components
   - Verify Web Crypto API availability
   - Consider using batch operations

### Debug Mode

```javascript
// Enable detailed logging for development
const signer = new RFC9421Signer(config);

// Log canonical messages for debugging
const result = await signer.signRequest(request);
console.log('Canonical message:', result.canonicalMessage);
console.log('Signature headers:', result.headers);
```

### Testing with curl

```bash
# Test signed request with curl
curl -X POST https://api.datafold.com/api/crypto/keys/register \
  -H "Content-Type: application/json" \
  -H "Signature-Input: sig1=(...);created=1234567890;keyid=\"test\";alg=\"ed25519\";nonce=\"uuid\"" \
  -H "Signature: sig1=:base64signature:" \
  -H "Content-Digest: sha-256=:hash:" \
  -d '{"clientId":"test","publicKey":"hex"}'
```

## Best Practices

1. **Key Security**: Store private keys securely, never in source code
2. **Key Rotation**: Implement regular key rotation policies
3. **Error Handling**: Always handle signing errors gracefully
4. **Performance**: Monitor signing performance in production
5. **Testing**: Test signature verification with your server implementation
6. **Compatibility**: Check browser compatibility before deployment

## Examples

See the [`examples/`](../examples/) directory for complete working examples:

- [`request-signing-example.html`](../examples/request-signing-example.html) - Interactive browser example
- [`basic-usage.html`](../examples/basic-usage.html) - Basic SDK usage
- [`server-integration-example.html`](../examples/server-integration-example.html) - Full server integration

For more information, see the [main SDK documentation](../README.md) and [API reference](./api-reference.md).