# Signature Verification Utilities

This document describes the signature verification utilities provided by the DataFold JavaScript SDK. These utilities complement the existing signing capabilities and allow you to validate RFC 9421 HTTP Message Signatures.

## Overview

The verification module provides comprehensive tools for:

- **Response Signature Verification**: Validate server-signed responses
- **Client-Side Verification Tools**: Signature validation for peer-to-peer scenarios
- **Developer Tools and Utilities**: Debugging and diagnostic tools
- **Integration and Configuration**: Middleware and configuration management

## Quick Start

```typescript
import { 
  createVerifier,
  verifySignature, 
  VERIFICATION_POLICIES 
} from '@datafold/js-sdk';

// Quick verification helper
const isValid = await verifySignature(
  request,
  headers,
  publicKey,
  'standard'
);

// Full verification with detailed results
const verifier = createVerifier({
  policies: VERIFICATION_POLICIES,
  publicKeys: {
    'my-key-id': publicKeyBytes
  },
  defaultPolicy: 'standard'
});

const result = await verifier.verify(request, headers);
console.log(`Signature valid: ${result.signatureValid}`);
console.log(`Overall status: ${result.status}`);
```

## Core Components

### RFC9421Verifier

The main verification engine that validates RFC 9421 signatures:

```typescript
import { createVerifier, VerificationConfig } from '@datafold/js-sdk';

const config: VerificationConfig = {
  policies: {
    strict: {
      name: 'strict',
      description: 'High security verification',
      verifyTimestamp: true,
      maxTimestampAge: 300, // 5 minutes
      verifyNonce: true,
      verifyContentDigest: true,
      requiredComponents: ['@method', '@target-uri', 'content-digest'],
      allowedAlgorithms: ['ed25519'],
      requireAllHeaders: true
    }
  },
  publicKeys: {
    'server-key-1': serverPublicKey,
    'client-key-1': clientPublicKey
  },
  defaultPolicy: 'strict'
};

const verifier = createVerifier(config);

// Verify a signed request
const result = await verifier.verify(request, headers, {
  policy: 'strict'
});

if (result.status === 'valid' && result.signatureValid) {
  console.log('Signature verification passed');
} else {
  console.log('Verification failed:', result.error?.message);
}
```

### Verification Policies

Pre-defined policies for different security levels:

```typescript
import { 
  STRICT_VERIFICATION_POLICY,
  STANDARD_VERIFICATION_POLICY,
  LENIENT_VERIFICATION_POLICY,
  createVerificationPolicy
} from '@datafold/js-sdk';

// Using built-in policies
const verifier = createVerifier({
  policies: {
    strict: STRICT_VERIFICATION_POLICY,
    standard: STANDARD_VERIFICATION_POLICY,
    development: LENIENT_VERIFICATION_POLICY
  },
  publicKeys: { /* ... */ }
});

// Creating custom policy
const customPolicy = createVerificationPolicy(
  'api-gateway',
  'Policy for API gateway requests',
  {
    verifyTimestamp: true,
    maxTimestampAge: 600, // 10 minutes
    verifyNonce: true,
    verifyContentDigest: true,
    requiredComponents: ['@method', '@target-uri', 'authorization'],
    allowedAlgorithms: ['ed25519'],
    requireAllHeaders: false,
    customRules: [
      {
        name: 'api-key-validation',
        description: 'Validate API key in authorization header',
        validate: async (context) => {
          const hasAuth = context.signatureData.coveredComponents.includes('authorization');
          return {
            passed: hasAuth,
            message: hasAuth ? 'Authorization header covered' : 'Missing authorization header'
          };
        }
      }
    ]
  }
);
```

### Signature Inspector

Debugging and diagnostic utilities:

```typescript
import { createInspector, validateSignatureFormat } from '@datafold/js-sdk';

const inspector = createInspector();

// Quick format validation
const isValidFormat = validateSignatureFormat(headers);

// Detailed format inspection
const formatAnalysis = inspector.inspectFormat(headers);
console.log('RFC 9421 compliant:', formatAnalysis.isValidRFC9421);
console.log('Issues found:', formatAnalysis.issues);

// Component analysis
const componentAnalysis = inspector.analyzeComponents(extractedSignatureData);
console.log('Security level:', componentAnalysis.securityAssessment.level);
console.log('Valid components:', componentAnalysis.validComponents);

// Parameter validation
const paramValidation = inspector.validateParameters(signatureParams);
console.log('All parameters valid:', paramValidation.allValid);

// Generate diagnostic report
const report = inspector.generateDiagnosticReport(verificationResult);
console.log(report);
```

## Middleware Integration

### HTTP Response Verification

```typescript
import { createResponseVerificationMiddleware } from '@datafold/js-sdk';

const middleware = createResponseVerificationMiddleware({
  verificationConfig: {
    policies: VERIFICATION_POLICIES,
    publicKeys: { 'server-key': serverPublicKey },
    defaultPolicy: 'standard'
  },
  throwOnFailure: false,
  onVerificationFailure: (result, response) => {
    console.warn('Response verification failed:', result.error?.message);
  },
  skipPatterns: [/\/health/, /\/metrics/] // Skip verification for these endpoints
});

// Use with fetch
const response = await fetch('/api/data');
const verifiedResponse = await middleware(response);
```

### Express.js Request Verification

```typescript
import { createExpressVerificationMiddleware } from '@datafold/js-sdk';

const verificationMiddleware = createExpressVerificationMiddleware({
  verificationConfig: {
    policies: VERIFICATION_POLICIES,
    publicKeys: { 'client-key': clientPublicKey }
  },
  rejectInvalid: true
});

app.use('/api/secure', verificationMiddleware);

app.post('/api/secure/data', (req, res) => {
  // Access verification result
  const verification = req.signatureVerification;
  if (verification.valid) {
    console.log('Request signature verified');
  }
  
  res.json({ success: true });
});
```

### Fetch Interceptor

```typescript
import { createFetchInterceptor } from '@datafold/js-sdk';

const securedFetch = createFetchInterceptor({
  verificationConfig: {
    policies: VERIFICATION_POLICIES,
    publicKeys: { 'server-key': serverPublicKey }
  },
  throwOnFailure: true
});

// Use instead of regular fetch
const response = await securedFetch('/api/data');
// Response signature is automatically verified
```

## Batch Verification

For high-throughput scenarios:

```typescript
import { createBatchVerifier } from '@datafold/js-sdk';

const batchVerifier = createBatchVerifier(verificationConfig);

const items = [
  {
    message: request1,
    headers: headers1,
    policy: 'standard'
  },
  {
    message: request2,
    headers: headers2,
    policy: 'strict'
  }
];

const results = await batchVerifier.verifyBatch(items);
const stats = batchVerifier.getBatchStats(results);

console.log(`Verified ${stats.valid}/${stats.total} signatures`);
console.log(`Average time: ${stats.averageTime.toFixed(2)}ms`);
```

## Performance Monitoring

The verification utilities include built-in performance monitoring:

```typescript
const result = await verifier.verify(request, headers);

console.log('Performance metrics:');
console.log(`Total time: ${result.performance.totalTime}ms`);
console.log('Step timings:', result.performance.stepTimings);

// Performance is tracked for:
// - Signature extraction
// - Policy retrieval
// - Public key retrieval
// - Cryptographic verification
// - Policy rule validation
```

## Error Handling

```typescript
import { VerificationError } from '@datafold/js-sdk';

try {
  const result = await verifier.verify(request, headers);
  
  if (result.status === 'error') {
    console.error('Verification error:', result.error);
  } else if (!result.signatureValid) {
    console.warn('Invalid signature:', result.diagnostics);
  }
  
} catch (error) {
  if (error instanceof VerificationError) {
    console.error('Verification failed:', error.message);
    console.error('Error code:', error.code);
    console.error('Details:', error.details);
  }
}
```

## Advanced Configuration

### Custom Key Sources

```typescript
const config: VerificationConfig = {
  policies: VERIFICATION_POLICIES,
  publicKeys: {},
  trustedKeySources: [
    {
      name: 'key-server',
      type: 'url',
      source: 'https://keys.example.com/api/keys',
      cacheTtl: 3600000 // 1 hour
    },
    {
      name: 'local-cache',
      type: 'function',
      source: async (keyId: string) => {
        return await localKeyCache.get(keyId);
      }
    }
  ]
};
```

### Custom Verification Rules

```typescript
const customRule: VerificationRule = {
  name: 'tenant-validation',
  description: 'Validate tenant ID in request',
  validate: async (context) => {
    const message = context.message;
    const tenantId = extractTenantId(message);
    const keyId = context.signatureData.params.keyid;
    
    const isValidTenant = await validateTenantKey(tenantId, keyId);
    
    return {
      passed: isValidTenant,
      message: isValidTenant ? 'Tenant validated' : 'Invalid tenant for key',
      details: { tenantId, keyId }
    };
  }
};

const policy = createVerificationPolicy('multi-tenant', 'Multi-tenant validation', {
  customRules: [customRule]
});
```

## Security Considerations

### Timestamp Validation

- Always verify timestamps to prevent replay attacks
- Configure appropriate `maxTimestampAge` based on your use case
- Account for clock skew between systems

### Nonce Validation

- Implement nonce caching to prevent replay attacks
- Use secure random nonce generation
- Consider nonce expiration policies

### Component Coverage

- Require critical headers in signature components
- Always include `@method` and `@target-uri`
- Include `content-digest` for requests with bodies
- Cover security-relevant headers like `authorization`

### Key Management

- Regularly rotate signing keys
- Use secure key storage
- Implement key revocation mechanisms
- Validate key ownership and permissions

## Testing

The SDK includes test vectors for development and testing:

```typescript
import { TestVectorRunner } from '@datafold/js-sdk/verification/test-vectors';

const results = await TestVectorRunner.runTestVectors(
  ALL_TEST_VECTORS,
  async (message, headers, publicKey) => {
    return await verifySignature(message, headers, publicKey);
  }
);

console.log(TestVectorRunner.generateTestReport(results));
```

## Browser vs Node.js

The verification utilities work in both browser and Node.js environments:

### Browser
- Uses Web Crypto API for cryptographic operations
- Supports modern browsers with crypto.subtle
- IndexedDB storage for key caching

### Node.js
- Uses Node.js crypto module
- File system storage options
- Better performance for batch operations

## Migration from Basic Verification

If you're upgrading from basic signature verification:

```typescript
// Before: Basic verification
const isValid = await ed25519.verify(signature, message, publicKey);

// After: Comprehensive verification
const result = await verifier.verify(request, headers);
const isValid = result.signatureValid && result.status === 'valid';

// Additional benefits:
console.log('Detailed diagnostics:', result.diagnostics);
console.log('Policy compliance:', result.diagnostics.policyCompliance);
console.log('Security analysis:', result.diagnostics.securityAnalysis);
```

## Best Practices

1. **Choose Appropriate Policies**: Use strict policies for high-security scenarios, standard for general use
2. **Monitor Performance**: Set up alerts if verification times exceed thresholds
3. **Log Verification Results**: Keep audit logs for security analysis
4. **Handle Graceful Degradation**: Have fallback strategies for verification failures
5. **Regular Key Rotation**: Implement automated key rotation procedures
6. **Test Thoroughly**: Use test vectors to validate your verification setup

## Troubleshooting

### Common Issues

1. **Missing Headers**: Ensure signature-input and signature headers are present
2. **Clock Skew**: Allow reasonable timestamp tolerance for distributed systems
3. **Key Mismatches**: Verify public keys match the signing private keys
4. **Component Mismatches**: Ensure covered components match the actual request
5. **Algorithm Support**: Only Ed25519 is currently supported

### Debug Tools

Use the inspector for troubleshooting:

```typescript
const inspector = createInspector();
const analysis = inspector.inspectFormat(headers);

if (!analysis.isValidRFC9421) {
  console.log('Format issues:');
  analysis.issues.forEach(issue => {
    console.log(`- ${issue.severity}: ${issue.message}`);
  });
}
```

For more detailed examples and API documentation, see the TypeScript definitions and test files.