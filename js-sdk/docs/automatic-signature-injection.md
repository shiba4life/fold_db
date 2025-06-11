# Automatic Signature Injection for JavaScript SDK

This document describes the enhanced automatic signature injection capabilities in the DataFold JavaScript SDK HTTP client.

## Overview

The enhanced HTTP client provides seamless automatic signature injection with RFC 9421 compliance, configurable signing behavior, performance optimizations, and comprehensive middleware support.

## Quick Start

### Basic Automatic Signing

```typescript
import { createSignedHttpClient, createSigningConfig } from '@datafold/js-sdk';

// Create signing configuration
const signingConfig = createSigningConfig()
  .algorithm('ed25519')
  .keyId('my-client-key')
  .privateKey(privateKeyBytes)
  .profile('standard')
  .build();

// Create pre-configured HTTP client with automatic signing
const client = createSignedHttpClient(signingConfig, {
  baseUrl: 'https://api.datafold.com',
  signingMode: 'auto'
});

// All requests will be automatically signed
const status = await client.testConnection();
```

### Fluent Configuration

```typescript
import { createFluentHttpClient, RFC9421Signer } from '@datafold/js-sdk';

const signer = new RFC9421Signer(signingConfig);

const client = createFluentHttpClient()
  .configureSigning(signingConfig)
  .setSigningMode('auto')
  .enableDebugLogging(true)
  .configureEndpointSigning('/keys/register', { 
    enabled: true, 
    required: true 
  })
  .configureEndpointSigning('/status', { 
    enabled: false 
  });
```

## Signing Modes

### Auto Mode
```typescript
const client = createHttpClient({ signingMode: 'auto' });
client.configureSigning(signingConfig);

// All requests are automatically signed unless endpoint-specific config overrides
```

### Manual Mode
```typescript
const client = createHttpClient({ signingMode: 'manual' });
client.configureSigning(signingConfig);

// Only explicitly configured endpoints are signed
```

### Disabled Mode
```typescript
const client = createHttpClient({ signingMode: 'disabled' });

// No requests are signed, even if signer is configured
```

## Endpoint-Specific Configuration

```typescript
const client = createHttpClient({
  signingMode: 'auto',
  endpointConfig: {
    '/keys/register': {
      enabled: true,
      required: true, // Fail if signing fails
      options: {
        components: { 
          method: true,
          targetUri: true,
          headers: ['content-type', 'authorization'],
          contentDigest: true
        }
      }
    },
    '/status': {
      enabled: false // Never sign status requests
    },
    '/data/query': {
      enabled: true,
      required: false, // Continue without signature if signing fails
      options: {
        digestAlgorithm: 'sha-512'
      }
    }
  }
});
```

## Performance Optimization

### Signature Caching

```typescript
const client = createHttpClient({
  signingMode: 'auto',
  enableSignatureCache: true,
  signatureCacheTtl: 300000, // 5 minutes
  maxCacheSize: 1000
});

client.configureSigning(signingConfig);

// Identical requests will reuse cached signatures
await client.testConnection(); // Cache miss - signs request
await client.testConnection(); // Cache hit - reuses signature
```

### Cache Management

```typescript
// Clear cache manually
client.clearSignatureCache();

// Check cache effectiveness
const metrics = client.getSigningMetrics();
console.log(`Cache hit rate: ${metrics.cacheHits / (metrics.cacheHits + metrics.cacheMisses)}`);
```

## Middleware System

### Request Interceptors

```typescript
import { createCorrelationMiddleware, createLoggingMiddleware } from '@datafold/js-sdk';

const client = createHttpClient();

// Add correlation ID to all requests
const correlationMiddleware = createCorrelationMiddleware({
  headerName: 'x-request-id'
});
client.addRequestInterceptor(correlationMiddleware);

// Add request/response logging
const { requestInterceptor, responseInterceptor } = createLoggingMiddleware({
  logLevel: 'debug'
});
client.addRequestInterceptor(requestInterceptor);
client.addResponseInterceptor(responseInterceptor);
```

### Signing Middleware

```typescript
import { createSigningMiddleware } from '@datafold/js-sdk';

const signer = new RFC9421Signer(signingConfig);

const signingMiddleware = createSigningMiddleware(signer, {
  enabled: true,
  required: false, // Continue without signature on failure
  signingOptions: {
    digestAlgorithm: 'sha-256'
  }
});

client.addRequestInterceptor(signingMiddleware);
```

### Performance Monitoring

```typescript
import { createPerformanceMiddleware } from '@datafold/js-sdk';

const perfMiddleware = createPerformanceMiddleware();

client.addRequestInterceptor(perfMiddleware.requestInterceptor);
client.addResponseInterceptor(perfMiddleware.responseInterceptor);

// Get performance metrics
setInterval(() => {
  const metrics = perfMiddleware.getMetrics();
  console.log('Performance:', {
    totalRequests: metrics.totalRequests,
    averageLatency: metrics.averageLatency,
    successRate: metrics.successRate
  });
}, 30000);
```

## Monitoring and Debugging

### Signing Metrics

```typescript
const client = createSignedHttpClient(signingConfig, {
  signingMode: 'auto',
  debugLogging: true
});

// Make some requests...
await client.testConnection();
await client.registerPublicKey({ publicKey: '...' });

// Check signing performance
const metrics = client.getSigningMetrics();
console.log('Signing metrics:', {
  totalRequests: metrics.totalRequests,
  signedRequests: metrics.signedRequests,
  signingFailures: metrics.signingFailures,
  averageSigningTime: metrics.averageSigningTime,
  cacheHitRate: metrics.cacheHits / (metrics.cacheHits + metrics.cacheMisses)
});

// Reset metrics for fresh measurement
client.resetSigningMetrics();
```

### Debug Logging

```typescript
const client = createHttpClient({
  signingMode: 'auto',
  debugLogging: true
});

client.configureSigning(signingConfig);

// Will log detailed signing information to console.debug
await client.testConnection();
```

## Error Handling

### Graceful Degradation

```typescript
const client = createHttpClient({
  signingMode: 'auto',
  endpointConfig: {
    '/critical': { enabled: true, required: true },  // Fail if signing fails
    '/optional': { enabled: true, required: false }  // Continue without signing
  }
});

try {
  // This will fail if signing fails
  await client.makeRequest('POST', '/critical', data);
} catch (error) {
  if (error.errorCode === 'SIGNING_REQUIRED_FAILED') {
    console.error('Critical endpoint requires signing, but signing failed');
  }
}

// This will continue even if signing fails
await client.makeRequest('GET', '/optional');
```

### Signing Failure Recovery

```typescript
const client = createHttpClient({ signingMode: 'auto' });

// Configure with potentially faulty signer
client.configureSigning(faultySigningConfig);

// Monitor signing failures
setInterval(() => {
  const metrics = client.getSigningMetrics();
  if (metrics.signingFailures > 0) {
    console.warn(`${metrics.signingFailures} signing failures detected`);
    
    // Reconfigure with backup signer
    client.configureSigning(backupSigningConfig);
    client.resetSigningMetrics();
  }
}, 60000);
```

## Advanced Use Cases

### Dynamic Signing Configuration

```typescript
const client = createHttpClient({ signingMode: 'manual' });

// Configure different signing for different endpoints
client.configureEndpointSigning('/public-api', { enabled: false });
client.configureEndpointSigning('/private-api', { 
  enabled: true, 
  required: true,
  options: { digestAlgorithm: 'sha-512' }
});

// Update configuration at runtime
client.updateConfig({
  endpointConfig: {
    '/emergency-endpoint': { enabled: true, required: true }
  }
});
```

### Custom Signing Profiles

```typescript
import { createFromProfile } from '@datafold/js-sdk';

// Use predefined security profiles
const strictConfig = createFromProfile('strict', 'key-001', privateKey);
const standardConfig = createFromProfile('standard', 'key-001', privateKey);
const minimalConfig = createFromProfile('minimal', 'key-001', privateKey);

// Switch profiles based on environment
const config = process.env.NODE_ENV === 'production' ? strictConfig : standardConfig;

const client = createSignedHttpClient(config);
```

## Best Practices

### 1. Use Appropriate Signing Modes
- **Production**: Use `auto` mode with endpoint-specific overrides
- **Development**: Use `auto` mode with debug logging
- **Testing**: Use `disabled` mode for faster tests

### 2. Configure Caching Appropriately
- Enable caching for high-frequency requests
- Use shorter TTL for sensitive operations
- Monitor cache hit rates

### 3. Handle Errors Gracefully
- Use `required: false` for non-critical endpoints
- Implement fallback mechanisms
- Monitor signing failure rates

### 4. Optimize Performance
- Use signature caching for repeated requests
- Monitor signing times
- Use minimal security profiles when appropriate

### 5. Security Considerations
- Protect private keys properly
- Rotate keys regularly
- Use strict profiles for sensitive data
- Monitor for signing failures

## Migration from Basic Client

### Before (Basic Signing)

```typescript
const client = new DataFoldHttpClient();
client.configureSigning(signingConfig);

// Manual signing control
if (needsSigning) {
  client.enableSigning(signer);
} else {
  client.disableSigning();
}
```

### After (Enhanced Automatic Signing)

```typescript
const client = createSignedHttpClient(signingConfig, {
  signingMode: 'auto',
  enableSignatureCache: true,
  endpointConfig: {
    '/public': { enabled: false },
    '/private': { enabled: true, required: true }
  }
});

// Automatic signing based on configuration
// No manual enable/disable needed
```

## TypeScript Support

The enhanced HTTP client includes full TypeScript support:

```typescript
import type {
  HttpClientConfig,
  SigningMode,
  EndpointSigningConfig,
  RequestInterceptor,
  ResponseInterceptor,
  SigningMetrics
} from '@datafold/js-sdk';

const config: HttpClientConfig = {
  signingMode: 'auto',
  enableSignatureCache: true,
  endpointConfig: {
    '/test': { enabled: true, required: false }
  }
};

const interceptor: RequestInterceptor = async (request, context) => {
  // Type-safe request manipulation
  return request;
};
```

This enhanced automatic signature injection provides a powerful, flexible, and performant foundation for authenticated HTTP communication with the DataFold platform.