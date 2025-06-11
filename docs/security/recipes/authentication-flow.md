# Basic Authentication Flow Security Recipe

## Overview

This recipe provides a comprehensive, secure implementation of the basic DataFold signature authentication flow using RFC 9421 HTTP Message Signatures. It covers the complete end-to-end authentication process from key generation to request verification.

## Security Level
- **Complexity**: Basic to Intermediate
- **Security Rating**: High
- **Implementation Time**: 30-60 minutes

## Prerequisites

### Technical Requirements
- Understanding of HTTP request/response cycle
- Basic cryptography concepts (public/private keys)
- Familiarity with your chosen programming language

### Infrastructure Requirements
- DataFold server with signature authentication enabled
- Secure key storage mechanism
- Network connectivity with proper TLS

### Dependencies
- DataFold SDK (JavaScript, Python, or Rust CLI)
- Cryptographically secure random number generator
- HTTP client library

## Implementation

### Step 1: Key Generation and Registration

#### JavaScript/TypeScript
```typescript
import { generateKeyPair, RFC9421Signer } from '@datafold/signature-auth';
import { DataFoldHttpClient } from '@datafold/client';

// Generate Ed25519 keypair
const keypair = await generateKeyPair();
const keyId = 'client-key-' + Date.now();

// Store private key securely (see Key Management recipe)
await storePrivateKeySecurely(keyId, keypair.privateKey);

// Register public key with DataFold server
const client = new DataFoldHttpClient({
  baseUrl: 'https://api.yourcompany.com',
  signingMode: 'manual' // Manual for registration
});

await client.registerPublicKey({
  keyId: keyId,
  publicKey: keypair.publicKey,
  clientId: 'your-client-id',
  description: 'Production client key'
});
```

#### Python
```python
from datafold_sdk.crypto import generate_ed25519_keypair
from datafold_sdk.http_client import DataFoldHttpClient
from datafold_sdk.signing import RFC9421Signer
import time

# Generate Ed25519 keypair
keypair = generate_ed25519_keypair()
key_id = f'client-key-{int(time.time())}'

# Store private key securely
await store_private_key_securely(key_id, keypair.private_key)

# Register public key with DataFold server
client = DataFoldHttpClient(
    base_url='https://api.yourcompany.com',
    signing_mode='manual'  # Manual for registration
)

await client.register_public_key(
    key_id=key_id,
    public_key=keypair.public_key,
    client_id='your-client-id',
    description='Production client key'
)
```

#### CLI/Rust
```bash
# Generate keypair
datafold auth-keygen --key-id production-key --description "Production client key"

# Register public key with server
datafold auth-register --key-id production-key --server-url https://api.yourcompany.com
```

### Step 2: Configure Automatic Signing

#### JavaScript/TypeScript
```typescript
import { DataFoldHttpClient } from '@datafold/client';

const client = new DataFoldHttpClient({
  baseUrl: 'https://api.yourcompany.com',
  signingMode: 'auto',
  signingConfig: {
    keyId: keyId,
    privateKey: await loadPrivateKeySecurely(keyId),
    requiredComponents: ['@method', '@target-uri', 'content-digest'],
    includeTimestamp: true,
    includeNonce: true
  },
  // Security settings
  enableSignatureCache: true,
  signatureCacheTtl: 60000, // 1 minute
  debugLogging: false // Set to true for debugging
});
```

#### Python
```python
from datafold_sdk.http_client import EnhancedHttpClient
from datafold_sdk.signing import SigningConfig

# Configure automatic signing
client = EnhancedHttpClient(
    base_url='https://api.yourcompany.com',
    signing_mode='auto',
    signing_config=SigningConfig(
        key_id=key_id,
        private_key=await load_private_key_securely(key_id),
        required_components=['@method', '@target-uri', 'content-digest'],
        include_timestamp=True,
        include_nonce=True
    ),
    # Security settings
    enable_signature_cache=True,
    signature_cache_ttl=60,  # 60 seconds
    debug_logging=False
)
```

### Step 3: Making Authenticated Requests

#### JavaScript/TypeScript
```typescript
// Automatic signing - no additional code needed
try {
  // GET request
  const response = await client.get('/api/data');
  console.log('Data retrieved:', response.data);

  // POST request with body
  const createResponse = await client.post('/api/items', {
    name: 'New Item',
    description: 'Item description'
  });
  console.log('Item created:', createResponse.data);

} catch (error) {
  if (error.code === 'SIGNATURE_FAILED') {
    console.error('Authentication failed:', error.message);
    // Handle authentication failure
  } else {
    console.error('Request failed:', error.message);
  }
}
```

#### Python
```python
# Automatic signing - no additional code needed
try:
    # GET request
    response = await client.get('/api/data')
    print('Data retrieved:', response.data)

    # POST request with body
    create_response = await client.post('/api/items', json={
        'name': 'New Item',
        'description': 'Item description'
    })
    print('Item created:', create_response.data)

except SignatureAuthenticationError as e:
    print(f'Authentication failed: {e.message}')
    # Handle authentication failure
except Exception as e:
    print(f'Request failed: {e}')
```

### Step 4: Server-Side Verification

#### JavaScript/TypeScript (Express.js)
```typescript
import express from 'express';
import { SignatureVerifier } from '@datafold/signature-auth';

const app = express();
const verifier = new SignatureVerifier({
  publicKeys: {
    // Load from secure storage or key management service
    [keyId]: publicKey
  },
  defaultPolicy: 'strict',
  policies: {
    strict: {
      name: 'strict',
      description: 'Strict security policy',
      verifyTimestamp: true,
      maxTimestampAge: 300, // 5 minutes
      verifyNonce: true,
      verifyContentDigest: true,
      requiredComponents: ['@method', '@target-uri', 'content-digest'],
      allowedAlgorithms: ['ed25519'],
      requireAllHeaders: true
    }
  }
});

// Signature verification middleware
app.use(async (req, res, next) => {
  try {
    const result = await verifier.verifyRequest(req);
    
    if (!result.signatureValid) {
      return res.status(401).json({
        error: 'Invalid signature',
        details: result.error
      });
    }
    
    // Store verification result for audit logging
    req.authenticationResult = result;
    next();
  } catch (error) {
    res.status(401).json({
      error: 'Authentication failed',
      message: error.message
    });
  }
});
```

#### Python (FastAPI)
```python
from fastapi import FastAPI, Request, HTTPException, Depends
from datafold_sdk.verification import SignatureVerifier, VerificationPolicy

app = FastAPI()

# Configure signature verifier
verifier = SignatureVerifier(
    public_keys={
        # Load from secure storage or key management service
        key_id: public_key
    },
    default_policy='strict',
    policies={
        'strict': VerificationPolicy(
            name='strict',
            description='Strict security policy',
            verify_timestamp=True,
            max_timestamp_age=300,  # 5 minutes
            verify_nonce=True,
            verify_content_digest=True,
            required_components=['@method', '@target-uri', 'content-digest'],
            allowed_algorithms=['ed25519'],
            require_all_headers=True
        )
    }
)

# Signature verification dependency
async def verify_signature(request: Request):
    try:
        result = await verifier.verify_request(request)
        
        if not result.signature_valid:
            raise HTTPException(
                status_code=401,
                detail={
                    'error': 'Invalid signature',
                    'details': result.error
                }
            )
        
        return result
    except Exception as e:
        raise HTTPException(
            status_code=401,
            detail={
                'error': 'Authentication failed',
                'message': str(e)
            }
        )

# Protected endpoint
@app.get('/api/data')
async def get_data(auth_result = Depends(verify_signature)):
    # Access granted - process request
    return {'data': 'sensitive information', 'authenticated': True}
```

## Security Considerations

### Threat Analysis

#### Replay Attacks
- **Threat**: Attacker intercepts and replays valid requests
- **Mitigation**: Use nonces and timestamp validation
- **Implementation**: Include `nonce` and timestamp in all signatures

#### Man-in-the-Middle (MITM)
- **Threat**: Attacker intercepts and modifies requests
- **Mitigation**: Use HTTPS and signature components that include URL
- **Implementation**: Always include `@target-uri` in signature components

#### Key Compromise
- **Threat**: Private keys are stolen or compromised
- **Mitigation**: Secure key storage and regular rotation
- **Implementation**: Use hardware security modules (HSMs) or secure key vaults

#### Timing Attacks
- **Threat**: Attackers infer information from response timing
- **Mitigation**: Use constant-time comparison operations
- **Implementation**: Use dedicated cryptographic libraries

### Risk Mitigation

#### Private Key Protection
```typescript
// Use environment variables or secure key management
const privateKey = process.env.DATAFOLD_PRIVATE_KEY || 
  await keyVault.getSecret('datafold-private-key');

// Never log private keys
console.log('Using key ID:', keyId); // ✅ Safe
console.log('Private key:', privateKey); // ❌ Never do this
```

#### Signature Component Selection
```typescript
// Always include these components for security
const requiredComponents = [
  '@method',        // Prevents method tampering
  '@target-uri',    // Prevents URL manipulation
  'content-digest', // Prevents body modification
  'authorization'   // If using additional auth headers
];
```

#### Error Handling
```typescript
// Generic error responses to prevent information leakage
app.use((error, req, res, next) => {
  if (error.code === 'SIGNATURE_VERIFICATION_FAILED') {
    // Log detailed error server-side
    logger.error('Signature verification failed', {
      keyId: error.keyId,
      reason: error.reason,
      clientIP: req.ip
    });
    
    // Return generic error to client
    res.status(401).json({
      error: 'Authentication failed'
      // Don't include detailed error information
    });
  }
});
```

## Validation & Testing

### Unit Tests

#### JavaScript/TypeScript
```typescript
import { describe, it, expect } from '@jest/globals';
import { RFC9421Signer, SignatureVerifier } from '@datafold/signature-auth';

describe('Authentication Flow', () => {
  it('should sign and verify requests correctly', async () => {
    const keypair = await generateKeyPair();
    const signer = new RFC9421Signer({
      keyId: 'test-key',
      privateKey: keypair.privateKey
    });
    
    const request = {
      method: 'POST',
      url: 'https://api.example.com/data',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ test: 'data' })
    };
    
    const signature = await signer.sign(request);
    expect(signature).toBeDefined();
    
    const verifier = new SignatureVerifier({
      publicKeys: { 'test-key': keypair.publicKey }
    });
    
    const result = await verifier.verify(request, signature);
    expect(result.signatureValid).toBe(true);
  });
});
```

#### Python
```python
import pytest
from datafold_sdk.crypto import generate_ed25519_keypair
from datafold_sdk.signing import RFC9421Signer
from datafold_sdk.verification import SignatureVerifier

@pytest.mark.asyncio
async def test_authentication_flow():
    keypair = generate_ed25519_keypair()
    signer = RFC9421Signer(
        key_id='test-key',
        private_key=keypair.private_key
    )
    
    request = {
        'method': 'POST',
        'url': 'https://api.example.com/data',
        'headers': {'content-type': 'application/json'},
        'body': json.dumps({'test': 'data'})
    }
    
    signature = await signer.sign(request)
    assert signature is not None
    
    verifier = SignatureVerifier(
        public_keys={'test-key': keypair.public_key}
    )
    
    result = await verifier.verify(request, signature)
    assert result.signature_valid is True
```

### Integration Tests

#### End-to-End Authentication Test
```typescript
describe('End-to-End Authentication', () => {
  it('should complete full authentication flow', async () => {
    // 1. Generate keypair
    const keypair = await generateKeyPair();
    const keyId = 'e2e-test-key';
    
    // 2. Register public key
    const registrationClient = new DataFoldHttpClient({
      signingMode: 'manual'
    });
    
    await registrationClient.registerPublicKey({
      keyId,
      publicKey: keypair.publicKey,
      clientId: 'e2e-test-client'
    });
    
    // 3. Configure authenticated client
    const authClient = new DataFoldHttpClient({
      signingMode: 'auto',
      signingConfig: {
        keyId,
        privateKey: keypair.privateKey
      }
    });
    
    // 4. Make authenticated request
    const response = await authClient.get('/api/test');
    expect(response.status).toBe(200);
    
    // 5. Cleanup
    await registrationClient.revokePublicKey(keyId);
  });
});
```

### Performance Validation

#### Signing Performance Test
```typescript
describe('Performance Tests', () => {
  it('should sign requests within performance targets', async () => {
    const signer = new RFC9421Signer({
      keyId: 'perf-test',
      privateKey: keypair.privateKey
    });
    
    const request = {
      method: 'GET',
      url: 'https://api.example.com/data'
    };
    
    const startTime = Date.now();
    const signature = await signer.sign(request);
    const signingTime = Date.now() - startTime;
    
    // Should sign within 10ms for basic requests
    expect(signingTime).toBeLessThan(10);
  });
});
```

## Monitoring & Maintenance

### Key Metrics to Monitor

#### Authentication Metrics
```typescript
// Track these metrics in your monitoring system
const authMetrics = {
  signatureVerificationSuccessRate: 99.9, // Target: >99.5%
  averageSigningTime: 2.5, // Target: <10ms
  averageVerificationTime: 1.8, // Target: <5ms
  failedAuthenticationAttempts: 12, // Monitor for anomalies
  uniqueKeyIdsActive: 150 // Track key usage
};
```

#### Security Event Monitoring
```typescript
// Log security events for monitoring
function logSecurityEvent(event, details) {
  logger.security({
    timestamp: new Date().toISOString(),
    event,
    details,
    severity: getSeverityLevel(event)
  });
  
  // Alert on critical events
  if (event === 'SIGNATURE_VERIFICATION_FAILED') {
    alerting.notify('security-team', {
      event,
      details,
      urgency: 'high'
    });
  }
}
```

### Regular Maintenance Tasks

#### Weekly Tasks
- Review authentication success rates
- Check for unusual authentication patterns
- Verify key rotation schedules
- Update security monitoring rules

#### Monthly Tasks
- Rotate authentication keys
- Review and update security policies
- Performance baseline validation
- Security vulnerability assessment

#### Quarterly Tasks
- Comprehensive security audit
- Key management system review
- Update cryptographic libraries
- Penetration testing

## Troubleshooting

### Common Issues

#### "Invalid Signature" Errors
```typescript
// Debug signature verification failures
const debugVerification = async (request, signature) => {
  const verifier = new SignatureVerifier({
    debugMode: true // Enable detailed logging
  });
  
  const result = await verifier.verify(request, signature);
  
  if (!result.signatureValid) {
    console.log('Verification failed:', {
      error: result.error,
      signatureData: result.diagnostics.signatureAnalysis,
      policyCompliance: result.diagnostics.policyCompliance
    });
  }
};
```

#### Clock Skew Issues
```typescript
// Handle timestamp validation with tolerance
const verificationConfig = {
  policies: {
    'clock-tolerant': {
      verifyTimestamp: true,
      maxTimestampAge: 600, // 10 minutes tolerance
      clockSkewTolerance: 60 // 1 minute skew tolerance
    }
  }
};
```

#### Performance Issues
```typescript
// Enable signature caching for performance
const client = new DataFoldHttpClient({
  enableSignatureCache: true,
  signatureCacheTtl: 300000, // 5 minutes
  maxCacheSize: 10000
});
```

### Debug Mode
```typescript
// Enable comprehensive debugging
const client = new DataFoldHttpClient({
  signingMode: 'auto',
  debugLogging: true,
  performanceMonitoring: true
});

// This will log:
// - Signature generation details
// - Timing information
// - Cache hit/miss rates
// - Verification results
```

## Next Steps

After implementing basic authentication:

1. **[Key Rotation Strategy](key-rotation.md)** - Implement automated key rotation
2. **[Performance Optimization](performance-optimization.md)** - Optimize for production scale
3. **[Monitoring Setup](../quick-start/monitoring-setup.md)** - Set up comprehensive monitoring
4. **[Production Deployment](../quick-start/production-deployment.md)** - Deploy securely to production

## References

- [RFC 9421 HTTP Message Signatures](https://datatracker.ietf.org/doc/rfc9421/)
- [DataFold SDK Documentation](../../sdks/)
- [Integration Guides](../guides/integration/)
- [Performance Benchmarking](../performance/)