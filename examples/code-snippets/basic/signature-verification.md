# Basic Signature Verification - Code Snippets

Complete, working examples for verifying RFC 9421 HTTP Message Signatures across multiple programming languages and frameworks.

## 🎯 Overview

These snippets demonstrate how to verify incoming HTTP requests with DataFold signature authentication. Each example includes comprehensive security validation, proper error handling, and production-ready patterns.

## 📚 Language Examples

### JavaScript/TypeScript

#### Basic Signature Verification
```typescript
import { SignatureVerifier, VerificationPolicy } from '@datafold/signature-auth';

async function createBasicVerifier() {
  // Configure verification policies
  const verifier = new SignatureVerifier({
    // Public keys for verification (typically loaded from secure storage)
    publicKeys: {
      'client-key-v1': publicKeyBuffer, // Uint8Array of Ed25519 public key
      'service-key-v1': servicePublicKey
    },
    
    // Default verification policy
    defaultPolicy: 'production',
    
    // Available verification policies
    policies: {
      production: {
        name: 'production',
        description: 'Production security policy',
        verifyTimestamp: true,
        maxTimestampAge: 300, // 5 minutes
        verifyNonce: true,
        verifyContentDigest: true,
        requiredComponents: ['@method', '@target-uri', 'content-digest'],
        allowedAlgorithms: ['ed25519'],
        requireAllHeaders: true
      },
      development: {
        name: 'development',
        description: 'Relaxed policy for development',
        verifyTimestamp: true,
        maxTimestampAge: 600, // 10 minutes
        verifyNonce: false, // Disabled for easier testing
        verifyContentDigest: true,
        requiredComponents: ['@method', '@target-uri'],
        allowedAlgorithms: ['ed25519'],
        requireAllHeaders: false
      }
    },
    
    // Performance monitoring
    performanceMonitoring: {
      enabled: true,
      maxVerificationTime: 100 // 100ms maximum
    }
  });
  
  return verifier;
}

async function verifyIncomingRequest(request) {
  const verifier = await createBasicVerifier();
  
  try {
    // Convert incoming request to verifiable format
    const verifiableRequest = {
      method: request.method,
      url: request.url,
      headers: request.headers,
      body: request.body ? JSON.stringify(request.body) : undefined
    };
    
    // Verify the signature
    const result = await verifier.verifyRequest(verifiableRequest);
    
    if (result.signatureValid) {
      console.log('✅ Signature verification successful');
      console.log('Key ID:', result.diagnostics.signatureAnalysis.keyId);
      console.log('Verification time:', result.performance.totalTime, 'ms');
      
      return {
        authenticated: true,
        keyId: result.diagnostics.signatureAnalysis.keyId,
        verificationResult: result
      };
    } else {
      console.log('❌ Signature verification failed');
      console.log('Error:', result.error);
      
      return {
        authenticated: false,
        error: result.error,
        reason: result.error?.code || 'unknown'
      };
    }
    
  } catch (error) {
    console.error('❌ Verification error:', error.message);
    
    return {
      authenticated: false,
      error: { message: error.message, code: 'verification_exception' }
    };
  }
}

// Usage example
const incomingRequest = {
  method: 'POST',
  url: 'https://api.mycompany.com/v1/data',
  headers: {
    'content-type': 'application/json',
    'signature': 'sig1=:base64-signature-data:',
    'signature-input': 'sig1=("@method" "@target-uri" "content-digest");alg="ed25519";created=1640995200;nonce="random-nonce"',
    'content-digest': 'sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:'
  },
  body: { query: 'SELECT * FROM users', format: 'json' }
};

verifyIncomingRequest(incomingRequest)
  .then(result => {
    if (result.authenticated) {
      console.log('Request authenticated, processing...');
    } else {
      console.log('Authentication failed:', result.error);
    }
  })
  .catch(console.error);
```

#### Express.js Verification Middleware
```typescript
import express from 'express';
import { SignatureVerifier, VerificationError } from '@datafold/signature-auth';

// Create reusable verification middleware
function createSignatureMiddleware(config: any) {
  const verifier = new SignatureVerifier(config);
  
  return async (req: express.Request, res: express.Response, next: express.NextFunction) => {
    try {
      // Convert Express request to verifiable format
      const verifiableRequest = {
        method: req.method,
        url: req.protocol + '://' + req.get('host') + req.originalUrl,
        headers: req.headers,
        body: req.body ? JSON.stringify(req.body) : undefined
      };
      
      // Verify signature
      const result = await verifier.verifyRequest(verifiableRequest);
      
      if (!result.signatureValid) {
        // Log security event
        console.warn('Signature verification failed:', {
          ip: req.ip,
          userAgent: req.get('User-Agent'),
          url: req.originalUrl,
          error: result.error,
          timestamp: new Date().toISOString()
        });
        
        return res.status(401).json({
          error: 'Authentication failed',
          message: 'Invalid or missing signature'
        });
      }
      
      // Store verification result for downstream use
      req.authResult = result;
      req.authenticatedKeyId = result.diagnostics.signatureAnalysis.keyId;
      
      // Log successful authentication
      console.log('Authentication successful:', {
        keyId: req.authenticatedKeyId,
        ip: req.ip,
        url: req.originalUrl,
        verificationTime: result.performance.totalTime
      });
      
      next();
      
    } catch (error) {
      console.error('Authentication middleware error:', error);
      
      res.status(500).json({
        error: 'Authentication validation failed',
        message: 'Unable to verify request signature'
      });
    }
  };
}

// Usage in Express app
const app = express();

// Configure signature verification middleware
const signatureAuth = createSignatureMiddleware({
  publicKeys: {
    'mobile-app-v1': loadPublicKey('mobile-app-v1'),
    'web-app-v1': loadPublicKey('web-app-v1'),
    'service-v1': loadPublicKey('service-v1')
  },
  defaultPolicy: process.env.NODE_ENV === 'production' ? 'production' : 'development',
  policies: {
    production: {
      verifyTimestamp: true,
      maxTimestampAge: 300,
      verifyNonce: true,
      verifyContentDigest: true,
      requiredComponents: ['@method', '@target-uri', 'content-digest'],
      allowedAlgorithms: ['ed25519']
    },
    development: {
      verifyTimestamp: true,
      maxTimestampAge: 600,
      verifyNonce: false,
      verifyContentDigest: true,
      requiredComponents: ['@method', '@target-uri'],
      allowedAlgorithms: ['ed25519']
    }
  }
});

// Apply to all protected routes
app.use('/api', signatureAuth);

// Protected endpoint
app.post('/api/data', (req, res) => {
  res.json({
    message: 'Data received successfully',
    authenticatedAs: req.authenticatedKeyId,
    timestamp: new Date().toISOString()
  });
});

function loadPublicKey(keyId: string): Uint8Array {
  // Load from secure key storage, environment variables, or key management service
  const keyData = process.env[`PUBLIC_KEY_${keyId.toUpperCase().replace('-', '_')}`];
  return Buffer.from(keyData, 'base64');
}
```

### Python

#### Basic Signature Verification
```python
import asyncio
import json
import logging
from typing import Dict, Optional, Any
from datafold_sdk.verification import SignatureVerifier, VerificationPolicy
from datafold_sdk.crypto import load_public_key_from_env

logger = logging.getLogger(__name__)

async def create_basic_verifier():
    """Create a signature verifier with production-ready configuration"""
    
    # Load public keys from secure storage
    public_keys = {
        'client-key-v1': await load_public_key_from_env('CLIENT_PUBLIC_KEY_V1'),
        'service-key-v1': await load_public_key_from_env('SERVICE_PUBLIC_KEY_V1')
    }
    
    # Define verification policies
    policies = {
        'production': VerificationPolicy(
            name='production',
            description='Production security policy',
            verify_timestamp=True,
            max_timestamp_age=300,  # 5 minutes
            verify_nonce=True,
            verify_content_digest=True,
            required_components=['@method', '@target-uri', 'content-digest'],
            allowed_algorithms=['ed25519'],
            require_all_headers=True
        ),
        'development': VerificationPolicy(
            name='development',
            description='Development policy with relaxed settings',
            verify_timestamp=True,
            max_timestamp_age=600,  # 10 minutes
            verify_nonce=False,  # Disabled for easier testing
            verify_content_digest=True,
            required_components=['@method', '@target-uri'],
            allowed_algorithms=['ed25519'],
            require_all_headers=False
        )
    }
    
    # Create verifier instance
    verifier = SignatureVerifier(
        public_keys=public_keys,
        default_policy='production',
        policies=policies,
        performance_monitoring={
            'enabled': True,
            'max_verification_time': 100  # 100ms maximum
        }
    )
    
    return verifier

async def verify_incoming_request(request_data: Dict[str, Any]) -> Dict[str, Any]:
    """Verify signature of incoming HTTP request"""
    
    verifier = await create_basic_verifier()
    
    try:
        # Prepare request for verification
        verifiable_request = {
            'method': request_data['method'],
            'url': request_data['url'],
            'headers': request_data['headers'],
            'body': json.dumps(request_data['body']) if request_data.get('body') else None
        }
        
        # Perform signature verification
        result = await verifier.verify_request(verifiable_request)
        
        if result.signature_valid:
            logger.info('✅ Signature verification successful', extra={
                'key_id': result.diagnostics.signature_analysis.key_id,
                'verification_time': result.performance.total_time,
                'algorithm': result.diagnostics.signature_analysis.algorithm
            })
            
            return {
                'authenticated': True,
                'key_id': result.diagnostics.signature_analysis.key_id,
                'verification_result': result
            }
        else:
            logger.warning('❌ Signature verification failed', extra={
                'error_code': result.error.code if result.error else 'unknown',
                'error_message': result.error.message if result.error else 'Unknown error'
            })
            
            return {
                'authenticated': False,
                'error': result.error.__dict__ if result.error else {'message': 'Unknown verification error'},
                'reason': result.error.code if result.error else 'unknown'
            }
            
    except Exception as error:
        logger.error('❌ Verification exception', extra={
            'error': str(error),
            'error_type': type(error).__name__
        })
        
        return {
            'authenticated': False,
            'error': {'message': str(error), 'code': 'verification_exception'}
        }

# Usage example
async def main():
    incoming_request = {
        'method': 'POST',
        'url': 'https://api.mycompany.com/v1/data',
        'headers': {
            'content-type': 'application/json',
            'signature': 'sig1=:base64-signature-data:',
            'signature-input': 'sig1=("@method" "@target-uri" "content-digest");alg="ed25519";created=1640995200;nonce="random-nonce"',
            'content-digest': 'sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:'
        },
        'body': {'query': 'SELECT * FROM users', 'format': 'json'}
    }
    
    result = await verify_incoming_request(incoming_request)
    
    if result['authenticated']:
        print('Request authenticated, processing...')
        print(f"Authenticated as: {result['key_id']}")
    else:
        print(f"Authentication failed: {result['error']}")

if __name__ == '__main__':
    asyncio.run(main())
```

#### FastAPI Verification Dependency
```python
from fastapi import FastAPI, Request, HTTPException, Depends
from fastapi.security import HTTPBearer
from datafold_sdk.verification import SignatureVerifier
from datafold_sdk.crypto import load_public_keys_from_config
import json
import os

app = FastAPI()
security = HTTPBearer()

# Global verifier instance
_verifier = None

async def get_signature_verifier():
    """Get or create signature verifier instance"""
    global _verifier
    if not _verifier:
        public_keys = await load_public_keys_from_config()
        
        _verifier = SignatureVerifier(
            public_keys=public_keys,
            default_policy='production' if os.getenv('ENV') == 'production' else 'development',
            policies={
                'production': {
                    'verify_timestamp': True,
                    'max_timestamp_age': 300,
                    'verify_nonce': True,
                    'verify_content_digest': True,
                    'required_components': ['@method', '@target-uri', 'content-digest'],
                    'allowed_algorithms': ['ed25519']
                },
                'development': {
                    'verify_timestamp': True,
                    'max_timestamp_age': 600,
                    'verify_nonce': False,
                    'verify_content_digest': True,
                    'required_components': ['@method', '@target-uri'],
                    'allowed_algorithms': ['ed25519']
                }
            }
        )
    return _verifier

async def verify_signature(request: Request) -> dict:
    """FastAPI dependency for signature verification"""
    
    verifier = await get_signature_verifier()
    
    try:
        # Read request body for verification
        body_bytes = await request.body()
        body_str = body_bytes.decode('utf-8') if body_bytes else None
        
        # Prepare request for verification
        verifiable_request = {
            'method': request.method,
            'url': str(request.url),
            'headers': dict(request.headers),
            'body': body_str
        }
        
        # Verify signature
        result = await verifier.verify_request(verifiable_request)
        
        if not result.signature_valid:
            raise HTTPException(
                status_code=401,
                detail={
                    'error': 'Authentication failed',
                    'message': 'Invalid or missing signature',
                    'code': result.error.code if result.error else 'unknown'
                }
            )
        
        return {
            'key_id': result.diagnostics.signature_analysis.key_id,
            'verification_time': result.performance.total_time,
            'algorithm': result.diagnostics.signature_analysis.algorithm,
            'policy': result.diagnostics.policy_compliance.policy_name
        }
        
    except HTTPException:
        raise
    except Exception as error:
        raise HTTPException(
            status_code=500,
            detail={
                'error': 'Authentication validation failed',
                'message': 'Unable to verify request signature'
            }
        )

# Protected endpoint using signature verification
@app.post('/api/data')
async def process_data(
    data: dict,
    auth_result: dict = Depends(verify_signature)
):
    return {
        'message': 'Data processed successfully',
        'authenticated_as': auth_result['key_id'],
        'verification_time': f"{auth_result['verification_time']:.2f}ms",
        'timestamp': datetime.utcnow().isoformat()
    }

# Health check endpoint (no authentication required)
@app.get('/health')
async def health_check():
    verifier = await get_signature_verifier()
    
    return {
        'status': 'healthy',
        'verifier_loaded': verifier is not None,
        'policies': list(verifier.config.policies.keys()) if verifier else [],
        'timestamp': datetime.utcnow().isoformat()
    }
```

### Rust

#### Basic Signature Verification
```rust
use datafold::crypto::ed25519::{PublicKey, verify_signature};
use datafold::cli::verification::{
    CliSignatureVerifier, CliVerificationConfig, VerificationPolicy
};
use serde_json::Value;
use std::collections::HashMap;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create basic signature verifier
    let verifier = create_basic_verifier().await?;
    
    // Example incoming request
    let request = create_test_request();
    
    // Verify the request
    match verify_incoming_request(&verifier, &request).await {
        Ok(result) => {
            if result.authenticated {
                println!("✅ Request authenticated successfully");
                println!("Key ID: {}", result.key_id);
                println!("Verification time: {}ms", result.verification_time);
            } else {
                println!("❌ Authentication failed: {}", result.error);
            }
        }
        Err(e) => {
            eprintln!("❌ Verification error: {}", e);
        }
    }
    
    Ok(())
}

async fn create_basic_verifier() -> Result<CliSignatureVerifier, Box<dyn std::error::Error>> {
    // Load public keys (in production, load from secure storage)
    let mut public_keys = HashMap::new();
    
    // Example: Load from environment variables
    if let Ok(key_data) = std::env::var("CLIENT_PUBLIC_KEY_V1") {
        let key_bytes = base64::decode(key_data)?;
        public_keys.insert("client-key-v1".to_string(), key_bytes);
    }
    
    // Define verification policies
    let mut policies = HashMap::new();
    
    // Production policy
    let production_policy = VerificationPolicy {
        name: "production".to_string(),
        description: "Production security policy".to_string(),
        verify_timestamp: true,
        max_timestamp_age: Some(300), // 5 minutes
        verify_nonce: true,
        verify_content_digest: true,
        required_components: vec![
            "@method".to_string(),
            "@target-uri".to_string(),
            "content-digest".to_string()
        ],
        allowed_algorithms: vec!["ed25519".to_string()],
        require_all_headers: true,
        custom_rules: None,
    };
    
    policies.insert("production".to_string(), production_policy);
    
    // Development policy
    let development_policy = VerificationPolicy {
        name: "development".to_string(),
        description: "Development policy with relaxed settings".to_string(),
        verify_timestamp: true,
        max_timestamp_age: Some(600), // 10 minutes
        verify_nonce: false, // Disabled for easier testing
        verify_content_digest: true,
        required_components: vec![
            "@method".to_string(),
            "@target-uri".to_string()
        ],
        allowed_algorithms: vec!["ed25519".to_string()],
        require_all_headers: false,
        custom_rules: None,
    };
    
    policies.insert("development".to_string(), development_policy);
    
    // Create verification configuration
    let config = CliVerificationConfig {
        default_policy: Some("production".to_string()),
        policies,
        public_keys,
        trusted_key_sources: None,
        performance_monitoring: Some(PerformanceMonitoring {
            enabled: true,
            max_verification_time: 100, // 100ms
        }),
    };
    
    // Create verifier
    let verifier = CliSignatureVerifier::new(config);
    
    Ok(verifier)
}

#[derive(Debug)]
struct VerificationResult {
    authenticated: bool,
    key_id: String,
    verification_time: f64,
    error: String,
}

async fn verify_incoming_request(
    verifier: &CliSignatureVerifier,
    request: &IncomingRequest,
) -> Result<VerificationResult, Box<dyn std::error::Error>> {
    
    let start_time = std::time::Instant::now();
    
    // Convert request to verifiable format
    let verifiable_request = convert_to_verifiable_request(request)?;
    
    // Perform verification
    match verifier.verify_message_signature(
        &verifiable_request.body.unwrap_or_default().as_bytes(),
        &request.signature,
        &request.key_id,
        Some("production")
    ).await {
        Ok(result) => {
            let verification_time = start_time.elapsed().as_millis() as f64;
            
            if result.signature_valid {
                println!("Authentication successful: key_id={}, time={}ms", 
                        request.key_id, verification_time);
                
                Ok(VerificationResult {
                    authenticated: true,
                    key_id: request.key_id.clone(),
                    verification_time,
                    error: String::new(),
                })
            } else {
                println!("Authentication failed: {:?}", result.error);
                
                Ok(VerificationResult {
                    authenticated: false,
                    key_id: request.key_id.clone(),
                    verification_time,
                    error: result.error.map(|e| e.message).unwrap_or_else(|| "Unknown error".to_string()),
                })
            }
        }
        Err(e) => {
            eprintln!("Verification error: {}", e);
            
            Ok(VerificationResult {
                authenticated: false,
                key_id: request.key_id.clone(),
                verification_time: start_time.elapsed().as_millis() as f64,
                error: e.to_string(),
            })
        }
    }
}

#[derive(Debug)]
struct IncomingRequest {
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: Option<String>,
    signature: String,
    signature_input: String,
    key_id: String,
}

fn create_test_request() -> IncomingRequest {
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    headers.insert("signature".to_string(), "sig1=:base64-signature-data:".to_string());
    headers.insert("signature-input".to_string(), 
                  "sig1=(\"@method\" \"@target-uri\" \"content-digest\");alg=\"ed25519\";created=1640995200;nonce=\"random-nonce\"".to_string());
    headers.insert("content-digest".to_string(), 
                  "sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:".to_string());
    
    IncomingRequest {
        method: "POST".to_string(),
        url: "https://api.mycompany.com/v1/data".to_string(),
        headers,
        body: Some(r#"{"query": "SELECT * FROM users", "format": "json"}"#.to_string()),
        signature: "sig1=:base64-signature-data:".to_string(),
        signature_input: "sig1=(\"@method\" \"@target-uri\" \"content-digest\");alg=\"ed25519\";created=1640995200;nonce=\"random-nonce\"".to_string(),
        key_id: "client-key-v1".to_string(),
    }
}

struct VerifiableRequest {
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

fn convert_to_verifiable_request(request: &IncomingRequest) -> Result<VerifiableRequest, Box<dyn std::error::Error>> {
    Ok(VerifiableRequest {
        method: request.method.clone(),
        url: request.url.clone(),
        headers: request.headers.clone(),
        body: request.body.clone(),
    })
}

#[derive(Debug)]
struct PerformanceMonitoring {
    enabled: bool,
    max_verification_time: u64,
}
```

## 🔒 Security Notes

### Critical Security Validations

All verification examples implement these essential checks:

1. **Signature Format Validation** - Ensure RFC 9421 compliance
2. **Cryptographic Verification** - Verify Ed25519 signature authenticity
3. **Timestamp Validation** - Prevent replay attacks with time windows
4. **Nonce Validation** - Prevent replay attacks with unique identifiers
5. **Content Integrity** - Verify request body hasn't been tampered with
6. **Component Coverage** - Ensure all required signature components are present

### Security Best Practices

```javascript
// ✅ Good: Secure error handling
try {
  const result = await verifier.verifyRequest(request);
  if (!result.signatureValid) {
    // Log detailed error server-side for debugging
    logger.warn('Signature verification failed', { 
      error: result.error, 
      clientIP: request.ip 
    });
    
    // Return generic error to client
    return { authenticated: false, error: 'Authentication failed' };
  }
} catch (error) {
  // Log unexpected errors
  logger.error('Verification exception', { error: error.message });
  
  // Never expose internal errors to clients
  return { authenticated: false, error: 'Authentication validation failed' };
}

// ❌ Bad: Exposing detailed errors
catch (error) {
  return { authenticated: false, error: error.message }; // Don't do this!
}
```

### Public Key Management

```javascript
// ✅ Good: Load from secure storage
const loadPublicKeys = async () => {
  const keyVault = new KeyVault(process.env.KEY_VAULT_URL);
  return {
    'client-key-v1': await keyVault.getPublicKey('client-key-v1'),
    'service-key-v1': await keyVault.getPublicKey('service-key-v1')
  };
};

// ✅ Good: Environment variable loading with validation
const loadPublicKeyFromEnv = (keyName) => {
  const keyData = process.env[keyName];
  if (!keyData) {
    throw new Error(`Missing public key: ${keyName}`);
  }
  
  try {
    return Buffer.from(keyData, 'base64');
  } catch (error) {
    throw new Error(`Invalid public key format: ${keyName}`);
  }
};

// ❌ Bad: Hardcoded keys
const publicKeys = {
  'client-key': 'hardcoded-key-data' // Never do this!
};
```

## 🧪 Testing

### Unit Test Template

```javascript
describe('Signature Verification', () => {
  let verifier;
  let testKeypair;
  
  beforeEach(async () => {
    testKeypair = await generateKeyPair();
    verifier = new SignatureVerifier({
      publicKeys: {
        'test-key': testKeypair.publicKey
      },
      defaultPolicy: 'test',
      policies: {
        test: {
          verifyTimestamp: true,
          maxTimestampAge: 300,
          verifyNonce: true,
          verifyContentDigest: true,
          requiredComponents: ['@method', '@target-uri', 'content-digest'],
          allowedAlgorithms: ['ed25519']
        }
      }
    });
  });
  
  it('should verify valid signatures', async () => {
    // Create signed request
    const signer = new RFC9421Signer({
      keyId: 'test-key',
      privateKey: testKeypair.privateKey
    });
    
    const request = {
      method: 'POST',
      url: 'https://api.test.com/data',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ test: 'data' })
    };
    
    const signature = await signer.sign(request);
    const signedRequest = {
      ...request,
      headers: { ...request.headers, ...signature }
    };
    
    // Verify signature
    const result = await verifier.verifyRequest(signedRequest);
    
    expect(result.signatureValid).toBe(true);
    expect(result.diagnostics.signatureAnalysis.keyId).toBe('test-key');
  });
  
  it('should reject invalid signatures', async () => {
    const request = {
      method: 'POST',
      url: 'https://api.test.com/data',
      headers: {
        'content-type': 'application/json',
        'signature': 'sig1=:invalid-signature:',
        'signature-input': 'sig1=("@method");alg="ed25519"'
      },
      body: JSON.stringify({ test: 'data' })
    };
    
    const result = await verifier.verifyRequest(request);
    
    expect(result.signatureValid).toBe(false);
    expect(result.error).toBeDefined();
  });
  
  it('should enforce timestamp validation', async () => {
    const oldTimestamp = Math.floor(Date.now() / 1000) - 600; // 10 minutes ago
    
    const request = {
      method: 'GET',
      url: 'https://api.test.com/data',
      headers: {
        'signature': 'sig1=:some-signature:',
        'signature-input': `sig1=("@method");alg="ed25519";created=${oldTimestamp}`
      }
    };
    
    const result = await verifier.verifyRequest(request);
    
    expect(result.signatureValid).toBe(false);
    expect(result.error?.code).toBe('timestamp_too_old');
  });
});
```

### Integration Test Template

```javascript
describe('End-to-End Signature Verification', () => {
  let app;
  let testClient;
  
  beforeAll(async () => {
    app = await createTestApp();
    testClient = new AuthenticatedTestClient();
  });
  
  it('should authenticate valid requests', async () => {
    const response = await testClient.post('/api/data', {
      query: 'SELECT * FROM test_table'
    });
    
    expect(response.status).toBe(200);
    expect(response.data.authenticated).toBe(true);
  });
  
  it('should reject unauthenticated requests', async () => {
    const response = await request(app)
      .post('/api/data')
      .send({ query: 'SELECT * FROM test_table' })
      .expect(401);
    
    expect(response.body.error).toBe('Authentication failed');
  });
});
```

## ⚡ Performance Optimization

### Verification Caching

```javascript
// Cache verification results for identical requests
const verificationCache = new Map();

const cachedVerify = async (request) => {
  const cacheKey = generateCacheKey(request);
  
  if (verificationCache.has(cacheKey)) {
    const cached = verificationCache.get(cacheKey);
    if (Date.now() - cached.timestamp < 60000) { // 1 minute TTL
      return cached.result;
    }
    verificationCache.delete(cacheKey);
  }
  
  const result = await verifier.verifyRequest(request);
  
  if (result.signatureValid) {
    verificationCache.set(cacheKey, {
      result,
      timestamp: Date.now()
    });
  }
  
  return result;
};
```

### Batch Verification

```javascript
// Verify multiple requests in parallel
const verifyBatch = async (requests) => {
  const verifications = requests.map(request => 
    verifier.verifyRequest(request).catch(error => ({
      signatureValid: false,
      error: { message: error.message }
    }))
  );
  
  return Promise.all(verifications);
};
```

## 🔗 Variations

### Custom Verification Policies

```javascript
// Environment-specific policies
const createPolicyForEnvironment = (env) => {
  const basePolicy = {
    verifyTimestamp: true,
    verifyContentDigest: true,
    allowedAlgorithms: ['ed25519']
  };
  
  switch (env) {
    case 'production':
      return {
        ...basePolicy,
        maxTimestampAge: 300,
        verifyNonce: true,
        requiredComponents: ['@method', '@target-uri', 'content-digest', 'authorization']
      };
    case 'staging':
      return {
        ...basePolicy,
        maxTimestampAge: 600,
        verifyNonce: true,
        requiredComponents: ['@method', '@target-uri', 'content-digest']
      };
    case 'development':
      return {
        ...basePolicy,
        maxTimestampAge: 1800,
        verifyNonce: false,
        requiredComponents: ['@method', '@target-uri']
      };
    default:
      throw new Error(`Unknown environment: ${env}`);
  }
};
```

### Multi-Key Verification

```javascript
// Support multiple key sources
const verifier = new SignatureVerifier({
  publicKeys: {
    // Static keys
    'static-key-v1': staticPublicKey,
    
    // Dynamic key loading
    'dynamic-key-*': async (keyId) => {
      return await keyManagementService.getPublicKey(keyId);
    }
  },
  
  // Key source priorities
  keySourcePriority: ['static', 'key-management-service', 'cache']
});
```

## 📚 Related Snippets

- **[Signature Creation](signature-creation.md)** - Create signatures for requests
- **[Key Generation](key-generation.md)** - Generate Ed25519 keypairs
- **[Error Handling](../error-handling/auth-errors.md)** - Handle verification errors
- **[Performance Caching](../performance/signature-caching.md)** - Optimize verification performance

## 🆘 Troubleshooting

### Common Issues

**"Public key not found"**
```javascript
// Ensure key is properly loaded
const publicKeys = await loadPublicKeys();
console.log('Available keys:', Object.keys(publicKeys));

// Check key format
const keyBuffer = publicKeys['my-key'];
console.log('Key length:', keyBuffer.length); // Should be 32 bytes for Ed25519
```

**"Timestamp validation failed"**
```javascript
// Check server time synchronization
const now = Math.floor(Date.now() / 1000);
const requestTime = parseInt(extractTimestamp(signatureInput));
console.log('Time difference:', Math.abs(now - requestTime), 'seconds');
```

**"Content digest mismatch"**
```javascript
// Verify body encoding matches
const receivedBody = JSON.stringify(request.body);
const expectedDigest = calculateSHA256Digest(receivedBody);
console.log('Expected digest:', expectedDigest);
console.log('Received digest:', request.headers['content-digest']);
```

## 📄 License

These code snippets are provided under the MIT license for maximum reusability.