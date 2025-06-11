# Basic Signature Creation - Code Snippets

Complete, working examples for creating RFC 9421 HTTP Message Signatures across multiple programming languages and frameworks.

## 🎯 Overview

These snippets demonstrate the fundamental process of creating cryptographic signatures for HTTP requests using DataFold's signature authentication system. Each example is production-ready and includes proper error handling.

## 📚 Language Examples

### JavaScript/TypeScript

#### Basic Signature Creation
```typescript
import { RFC9421Signer, generateKeyPair } from '@datafold/signature-auth';

async function createBasicSignature() {
  // Generate keypair (do this once, store securely)
  const keypair = await generateKeyPair();
  const keyId = 'my-app-key-v1';
  
  // Create signer instance
  const signer = new RFC9421Signer({
    keyId,
    privateKey: keypair.privateKey,
    // Required components for security
    requiredComponents: ['@method', '@target-uri', 'content-digest'],
    includeTimestamp: true,
    includeNonce: true
  });
  
  // Define request to sign
  const request = {
    method: 'POST',
    url: 'https://api.datafold.com/v1/data',
    headers: {
      'content-type': 'application/json',
      'user-agent': 'MyApp/1.0'
    },
    body: JSON.stringify({
      query: 'SELECT * FROM users WHERE active = true',
      format: 'json'
    })
  };
  
  try {
    // Generate signature
    const signature = await signer.sign(request);
    
    // Apply signature headers to request
    const signedHeaders = {
      ...request.headers,
      'signature': signature.signature,
      'signature-input': signature['signature-input'],
      'content-digest': signature['content-digest']
    };
    
    console.log('✅ Signature created successfully');
    console.log('Signature headers:', signedHeaders);
    
    return { ...request, headers: signedHeaders };
    
  } catch (error) {
    console.error('❌ Signature creation failed:', error.message);
    throw error;
  }
}

// Usage
createBasicSignature()
  .then(signedRequest => {
    console.log('Ready to send:', signedRequest);
  })
  .catch(console.error);
```

#### Express.js Middleware Integration
```typescript
import express from 'express';
import { RFC9421Signer } from '@datafold/signature-auth';

// Middleware to automatically sign outgoing requests
function signatureMiddleware(signerConfig: any) {
  const signer = new RFC9421Signer(signerConfig);
  
  return async (req: express.Request, res: express.Response, next: express.NextFunction) => {
    // Add signing capability to request object
    req.signRequest = async (requestData: any) => {
      try {
        const signature = await signer.sign(requestData);
        return {
          ...requestData,
          headers: {
            ...requestData.headers,
            ...signature
          }
        };
      } catch (error) {
        console.error('Request signing failed:', error);
        throw new Error('Unable to sign request');
      }
    };
    
    next();
  };
}

// Usage in Express app
const app = express();

app.use(signatureMiddleware({
  keyId: process.env.DATAFOLD_KEY_ID,
  privateKey: Buffer.from(process.env.DATAFOLD_PRIVATE_KEY, 'base64')
}));

app.post('/proxy-datafold', async (req, res) => {
  try {
    const requestToSign = {
      method: 'POST',
      url: 'https://api.datafold.com/v1/query',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(req.body)
    };
    
    const signedRequest = await req.signRequest(requestToSign);
    
    // Send signed request to DataFold
    const response = await fetch(signedRequest.url, {
      method: signedRequest.method,
      headers: signedRequest.headers,
      body: signedRequest.body
    });
    
    const data = await response.json();
    res.json(data);
    
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});
```

### Python

#### Basic Signature Creation
```python
import json
import asyncio
from datafold_sdk.crypto import generate_ed25519_keypair
from datafold_sdk.signing import RFC9421Signer

async def create_basic_signature():
    # Generate keypair (do this once, store securely)
    keypair = generate_ed25519_keypair()
    key_id = 'my-app-key-v1'
    
    # Create signer instance
    signer = RFC9421Signer(
        key_id=key_id,
        private_key=keypair.private_key,
        # Required components for security
        required_components=['@method', '@target-uri', 'content-digest'],
        include_timestamp=True,
        include_nonce=True
    )
    
    # Define request to sign
    request_data = {
        'method': 'POST',
        'url': 'https://api.datafold.com/v1/data',
        'headers': {
            'content-type': 'application/json',
            'user-agent': 'MyApp/1.0'
        },
        'body': json.dumps({
            'query': 'SELECT * FROM users WHERE active = true',
            'format': 'json'
        })
    }
    
    try:
        # Generate signature
        signature = await signer.sign(request_data)
        
        # Apply signature headers to request
        signed_headers = {
            **request_data['headers'],
            'signature': signature['signature'],
            'signature-input': signature['signature-input'],
            'content-digest': signature['content-digest']
        }
        
        print('✅ Signature created successfully')
        print(f'Signature headers: {signed_headers}')
        
        return {**request_data, 'headers': signed_headers}
        
    except Exception as error:
        print(f'❌ Signature creation failed: {error}')
        raise

# Usage
async def main():
    try:
        signed_request = await create_basic_signature()
        print(f'Ready to send: {signed_request}')
    except Exception as e:
        print(f'Error: {e}')

if __name__ == '__main__':
    asyncio.run(main())
```

#### FastAPI Integration
```python
from fastapi import FastAPI, Request, HTTPException, Depends
from datafold_sdk.signing import RFC9421Signer, SigningConfig
from datafold_sdk.crypto import load_private_key_from_env
import httpx
import json

app = FastAPI()

# Global signer instance
signer = None

async def get_signer():
    global signer
    if not signer:
        private_key = await load_private_key_from_env('DATAFOLD_PRIVATE_KEY')
        signer = RFC9421Signer(
            key_id=os.getenv('DATAFOLD_KEY_ID'),
            private_key=private_key,
            required_components=['@method', '@target-uri', 'content-digest'],
            include_timestamp=True,
            include_nonce=True
        )
    return signer

async def sign_request(request_data: dict, signer: RFC9421Signer = Depends(get_signer)):
    """Helper to sign outgoing requests"""
    try:
        signature = await signer.sign(request_data)
        return {
            **request_data,
            'headers': {
                **request_data.get('headers', {}),
                **signature
            }
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Signing failed: {e}")

@app.post("/proxy-datafold")
async def proxy_to_datafold(data: dict, signer: RFC9421Signer = Depends(get_signer)):
    """Proxy requests to DataFold with automatic signing"""
    
    # Prepare request data
    request_data = {
        'method': 'POST',
        'url': 'https://api.datafold.com/v1/query',
        'headers': {'content-type': 'application/json'},
        'body': json.dumps(data)
    }
    
    # Sign the request
    signed_request = await sign_request(request_data, signer)
    
    # Send signed request
    async with httpx.AsyncClient() as client:
        response = await client.request(
            method=signed_request['method'],
            url=signed_request['url'],
            headers=signed_request['headers'],
            content=signed_request['body']
        )
        
        return response.json()
```

### Rust

#### Basic Signature Creation
```rust
use datafold::crypto::ed25519::{generate_master_keypair, MasterKeyPair};
use datafold::cli::auth::{CliRequestSigner, CliSigningConfig, SignatureComponent};
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate keypair (do this once, store securely)
    let keypair = generate_master_keypair()?;
    let key_id = "my-app-key-v1".to_string();
    
    // Create signing configuration
    let signing_config = CliSigningConfig {
        required_components: vec![
            SignatureComponent::Method,
            SignatureComponent::TargetUri,
            SignatureComponent::Header("content-digest".to_string()),
        ],
        include_content_digest: true,
        include_timestamp: true,
        include_nonce: true,
        timestamp_tolerance_seconds: 300,
        cache_signatures: true,
        debug_mode: false,
    };
    
    // Create signer instance
    let signer = CliRequestSigner::new(keypair, key_id, signing_config)?;
    
    // Define request to sign
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    headers.insert("user-agent".to_string(), "MyApp/1.0".to_string());
    
    let body = json!({
        "query": "SELECT * FROM users WHERE active = true",
        "format": "json"
    }).to_string();
    
    let request = reqwest::Request::new(
        reqwest::Method::POST,
        "https://api.datafold.com/v1/data".parse()?,
    );
    
    // Note: In a real implementation, you'd set headers and body on the request
    
    try {
        // Generate signature
        let signed_request = signer.sign_request(request).await?;
        
        println!("✅ Signature created successfully");
        println!("Request ready to send: {:?}", signed_request);
        
        Ok(())
    } catch (error) {
        eprintln!("❌ Signature creation failed: {}", error);
        Err(error.into())
    }
}
```

#### CLI Integration
```bash
#!/bin/bash

# Generate keypair (one-time setup)
datafold auth-keygen --key-id "my-app-key-v1" --output-format pem

# Configure automatic signing
datafold auth-configure \
  --enable-auto-sign true \
  --default-mode auto \
  --required-components "@method,@target-uri,content-digest" \
  --include-timestamp true \
  --include-nonce true

# Create authentication profile
datafold auth-profile create production \
  --server-url "https://api.datafold.com" \
  --key-id "my-app-key-v1" \
  --client-id "my-application"

# Use automatic signing with datafold CLI
datafold query \
  --profile production \
  --query "SELECT * FROM users WHERE active = true" \
  --format json
  
# The CLI will automatically sign the request using the configured profile
```

### Go

#### Basic Signature Creation
```go
package main

import (
    "crypto/ed25519"
    "crypto/rand"
    "encoding/json"
    "fmt"
    "net/http"
    "strings"
    
    "github.com/datafold/signature-auth-go"
)

func createBasicSignature() error {
    // Generate keypair (do this once, store securely)
    publicKey, privateKey, err := ed25519.GenerateKey(rand.Reader)
    if err != nil {
        return fmt.Errorf("key generation failed: %w", err)
    }
    
    keyID := "my-app-key-v1"
    
    // Create signer instance
    signer, err := signatureauth.NewSigner(signatureauth.SignerConfig{
        KeyID:      keyID,
        PrivateKey: privateKey,
        RequiredComponents: []string{"@method", "@target-uri", "content-digest"},
        IncludeTimestamp: true,
        IncludeNonce:     true,
    })
    if err != nil {
        return fmt.Errorf("signer creation failed: %w", err)
    }
    
    // Define request to sign
    requestBody := map[string]interface{}{
        "query":  "SELECT * FROM users WHERE active = true",
        "format": "json",
    }
    
    bodyBytes, err := json.Marshal(requestBody)
    if err != nil {
        return fmt.Errorf("body marshaling failed: %w", err)
    }
    
    req, err := http.NewRequest("POST", "https://api.datafold.com/v1/data", strings.NewReader(string(bodyBytes)))
    if err != nil {
        return fmt.Errorf("request creation failed: %w", err)
    }
    
    req.Header.Set("Content-Type", "application/json")
    req.Header.Set("User-Agent", "MyApp/1.0")
    
    // Generate signature
    signedReq, err := signer.SignRequest(req)
    if err != nil {
        return fmt.Errorf("signature creation failed: %w", err)
    }
    
    fmt.Println("✅ Signature created successfully")
    fmt.Printf("Signature headers: %v\n", signedReq.Header)
    
    return nil
}

func main() {
    if err := createBasicSignature(); err != nil {
        fmt.Printf("❌ Error: %v\n", err)
    }
}
```

## 🔒 Security Notes

### Required Security Components

All examples include these essential security components:

1. **`@method`** - Prevents HTTP method tampering
2. **`@target-uri`** - Prevents URL manipulation attacks  
3. **`content-digest`** - Ensures request body integrity
4. **`timestamp`** - Prevents replay attacks beyond time window
5. **`nonce`** - Prevents duplicate request replay

### Key Management Best Practices

```javascript
// ✅ Good: Load from secure environment
const privateKey = process.env.DATAFOLD_PRIVATE_KEY;

// ✅ Good: Use key management service
const privateKey = await keyVault.getSecret('datafold-private-key');

// ❌ Bad: Hardcode in source code
const privateKey = 'hardcoded-key-here'; // Never do this!

// ❌ Bad: Log private keys
console.log('Private key:', privateKey); // Never do this!
```

### Error Handling

```javascript
try {
  const signature = await signer.sign(request);
  return signature;
} catch (error) {
  // ✅ Good: Log for debugging, return generic error
  logger.error('Signature generation failed', { error: error.message });
  throw new Error('Authentication failed');
  
  // ❌ Bad: Expose detailed error to client
  // throw new Error(`Signature failed: ${error.message}`);
}
```

## 🧪 Testing

### Unit Test Template

```javascript
describe('Signature Creation', () => {
  let signer;
  let keypair;
  
  beforeEach(async () => {
    keypair = await generateKeyPair();
    signer = new RFC9421Signer({
      keyId: 'test-key',
      privateKey: keypair.privateKey
    });
  });
  
  it('should create valid signatures', async () => {
    const request = {
      method: 'GET',
      url: 'https://api.test.com/data',
      headers: {}
    };
    
    const signature = await signer.sign(request);
    
    expect(signature).toHaveProperty('signature');
    expect(signature).toHaveProperty('signature-input');
    expect(signature['signature-input']).toContain('ed25519');
  });
  
  it('should include content digest for requests with body', async () => {
    const request = {
      method: 'POST',
      url: 'https://api.test.com/data',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ test: 'data' })
    };
    
    const signature = await signer.sign(request);
    
    expect(signature).toHaveProperty('content-digest');
    expect(signature['content-digest']).toMatch(/sha-256=:/);
  });
});
```

### Integration Test Template

```javascript
it('should work end-to-end with real server', async () => {
  const client = new DataFoldHttpClient({
    signingMode: 'auto',
    signingConfig: {
      keyId: 'test-key',
      privateKey: testKeypair.privateKey
    }
  });
  
  const response = await client.get('/api/test');
  expect(response.status).toBe(200);
});
```

## ⚡ Performance Optimization

### Signature Caching

```javascript
const signer = new RFC9421Signer({
  keyId: 'my-key',
  privateKey: privateKey,
  enableCaching: true,
  cacheTtl: 60000, // 1 minute
  maxCacheSize: 1000
});
```

### Connection Pooling

```javascript
const client = new DataFoldHttpClient({
  signingMode: 'auto',
  connectionPooling: {
    maxConnections: 10,
    keepAlive: true,
    timeout: 30000
  }
});
```

## 🔗 Variations

### Custom Signature Components

```javascript
// Include additional headers for enhanced security
const signer = new RFC9421Signer({
  keyId: 'my-key',
  privateKey: privateKey,
  requiredComponents: [
    '@method',
    '@target-uri', 
    'content-digest',
    'authorization', // If using additional auth
    'x-api-version'  // Version-specific signing
  ]
});
```

### Development vs Production Configuration

```javascript
const isDevelopment = process.env.NODE_ENV === 'development';

const signer = new RFC9421Signer({
  keyId: 'my-key',
  privateKey: privateKey,
  requiredComponents: isDevelopment 
    ? ['@method', '@target-uri'] // Minimal for dev
    : ['@method', '@target-uri', 'content-digest', 'authorization'], // Full for prod
  includeTimestamp: !isDevelopment, // Skip timestamp in dev
  debugLogging: isDevelopment
});
```

## 📚 Related Snippets

- **[Signature Verification](../basic/signature-verification.md)** - Verify incoming signatures
- **[Key Generation](../basic/key-generation.md)** - Generate Ed25519 keypairs
- **[Error Handling](../error-handling/auth-errors.md)** - Handle authentication errors
- **[Performance Optimization](../performance/signature-caching.md)** - Optimize signing performance

## 🆘 Troubleshooting

### Common Issues

**"Invalid signature format"**
```javascript
// Check signature-input format
console.log('Signature input:', signature['signature-input']);
// Should be: sig1=("@method" "@target-uri");alg="ed25519";created=1234567890
```

**"Timestamp too old"**
```javascript
// Check server time synchronization
const now = Math.floor(Date.now() / 1000);
console.log('Current timestamp:', now);
```

**"Content digest mismatch"**
```javascript
// Ensure body encoding is consistent
const body = JSON.stringify(data); // Use exact same encoding
```

## 📄 License

These code snippets are provided under the MIT license for maximum reusability.