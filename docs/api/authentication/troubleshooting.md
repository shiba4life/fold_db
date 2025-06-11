# Authentication Troubleshooting Guide

This guide helps you diagnose and resolve common issues with DataFold's signature authentication system. Use this as your first resource when encountering authentication problems.

## 🔍 Quick Diagnosis

### Authentication Status Check

Before diving into specific issues, check your authentication status:

```bash
# CLI: Check authentication status
datafold auth status --verbose

# Test basic connectivity
datafold auth test --show-request

# Verify configuration
datafold config list
```

```javascript
// JavaScript: Test authentication
import { DataFoldClient } from '@datafold/sdk';

const client = new DataFoldClient(config);

// Test with simple request
try {
  const response = await client.get('/api/status');
  console.log('Authentication working:', response.status === 200);
} catch (error) {
  console.error('Authentication failed:', error.message);
}
```

```python
# Python: Test authentication
from datafold_sdk import DataFoldClient

client = DataFoldClient(config)

try:
    response = client.get('/api/status')
    print(f'Authentication working: {response.status_code == 200}')
except Exception as error:
    print(f'Authentication failed: {error}')
```

## ❌ Common Error Messages

### `SIGNATURE_VERIFICATION_FAILED`

**Error Message:**
```json
{
  "success": false,
  "error": {
    "code": "SIGNATURE_VERIFICATION_FAILED",
    "message": "Digital signature verification failed"
  }
}
```

**Causes & Solutions:**

#### 1. Wrong Private Key
```bash
# Check if you're using the correct private key
datafold auth status --check-signature

# Verify key file exists and is readable
ls -la ~/.datafold/keys/private.pem
file ~/.datafold/keys/private.pem
```

**Solution:** Ensure you're using the private key that corresponds to the registered public key.

```javascript
// Verify key pair matches
import { generateKeyPair, verifyKeyPair } from '@datafold/sdk';

const isValid = await verifyKeyPair(privateKey, publicKey);
if (!isValid) {
  throw new Error('Private and public keys do not match');
}
```

#### 2. Incorrect Message Canonicalization
```javascript
// Debug: Show canonical message being signed
import { RFC9421Signer } from '@datafold/sdk';

const signer = new RFC9421Signer(config);
const result = await signer.signRequest(request);

console.log('Canonical message:', result.canonicalMessage);
console.log('Signature input:', result.signatureInput);
```

**Solution:** Verify that your request components match the server's expectations.

#### 3. Clock Synchronization Issues
```bash
# Check system time
date
ntpdate -q pool.ntp.org

# Sync time (if needed)
sudo ntpdate -s pool.ntp.org  # Linux/macOS
w32tm /resync                 # Windows
```

### `CLIENT_NOT_FOUND`

**Error Message:**
```json
{
  "success": false,
  "error": {
    "code": "CLIENT_NOT_FOUND",
    "message": "No public key registered for this client"
  }
}
```

**Solutions:**

#### 1. Register Your Public Key
```bash
# Register public key
datafold auth register \
  --server-url https://api.datafold.com \
  --client-id your-client-id \
  --key-name "Your Key Name"
```

#### 2. Verify Client ID
```bash
# Check what client ID you're using
datafold config get client-id

# List registered keys (if you have admin access)
curl https://api.datafold.com/api/crypto/keys/status/your-client-id
```

### `INVALID_TIMESTAMP`

**Error Message:**
```json
{
  "success": false,
  "error": {
    "code": "INVALID_TIMESTAMP",
    "message": "Request timestamp outside allowed window"
  }
}
```

**Solutions:**

#### 1. Time Synchronization
```bash
# Check current time vs server time
curl -I https://api.datafold.com/api/status | grep Date

# Compare with local time
date -u
```

#### 2. Adjust Security Profile
```javascript
// Use more lenient time validation for development
const client = new DataFoldClient({
  ...config,
  securityProfile: 'lenient'  // Allows larger time windows
});
```

```python
# Adjust timestamp tolerance
from datafold_sdk import DataFoldClient, SecurityProfiles

client = DataFoldClient(
    **config,
    security_profile=SecurityProfiles.LENIENT
)
```

### `NONCE_REUSE`

**Error Message:**
```json
{
  "success": false,
  "error": {
    "code": "NONCE_REUSE", 
    "message": "Nonce has already been used"
  }
}
```

**Solutions:**

#### 1. Ensure Unique Nonces
```javascript
// Generate fresh nonce for each request
import { v4 as uuidv4 } from 'uuid';

const generateNonce = () => uuidv4().replace(/-/g, '');

// Use in signer configuration
const signer = new RFC9421Signer({
  ...config,
  nonceGenerator: generateNonce
});
```

#### 2. Check for Request Duplication
```python
# Avoid duplicate requests
import time

class RequestDeduplicator:
    def __init__(self):
        self.recent_requests = set()
    
    def is_duplicate(self, request_signature):
        if request_signature in self.recent_requests:
            return True
        
        self.recent_requests.add(request_signature)
        # Clean old entries periodically
        return False
```

### `RATE_LIMIT_EXCEEDED`

**Error Message:**
```json
{
  "success": false,
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many requests",
    "details": {
      "retry_after": 60
    }
  }
}
```

**Solutions:**

#### 1. Implement Client-Side Rate Limiting
```javascript
// Add rate limiting to prevent server errors
class RateLimitedClient {
  constructor(client, requestsPerSecond = 10) {
    this.client = client;
    this.requestsPerSecond = requestsPerSecond;
    this.lastRequest = 0;
  }
  
  async request(method, url, options) {
    const now = Date.now();
    const timeSinceLastRequest = now - this.lastRequest;
    const minInterval = 1000 / this.requestsPerSecond;
    
    if (timeSinceLastRequest < minInterval) {
      await new Promise(resolve => 
        setTimeout(resolve, minInterval - timeSinceLastRequest)
      );
    }
    
    this.lastRequest = Date.now();
    return this.client.request(method, url, options);
  }
}
```

#### 2. Implement Exponential Backoff
```python
import time
import random

def exponential_backoff_retry(func, max_retries=5):
    """Retry function with exponential backoff"""
    for attempt in range(max_retries):
        try:
            return func()
        except RateLimitError as e:
            if attempt == max_retries - 1:
                raise
            
            # Exponential backoff with jitter
            delay = (2 ** attempt) + random.uniform(0, 1)
            time.sleep(delay)
```

## 🐛 Network Issues

### Connection Timeouts

**Symptoms:**
- Requests hang or timeout
- Intermittent connectivity issues
- Slow response times

**Diagnosis:**
```bash
# Test basic connectivity
curl -v https://api.datafold.com/api/status

# Test with timeout
curl --max-time 10 https://api.datafold.com/api/status

# Check DNS resolution
nslookup api.datafold.com
dig api.datafold.com
```

**Solutions:**

#### 1. Adjust Timeout Settings
```javascript
const client = new DataFoldClient({
  ...config,
  timeout: 30000,  // 30 seconds
  retries: 3
});
```

```python
client = DataFoldClient(
    **config,
    timeout=30.0,
    retries=3
)
```

#### 2. Configure HTTP Client
```javascript
// Custom HTTP agent with connection pooling
import https from 'https';

const httpsAgent = new https.Agent({
  keepAlive: true,
  keepAliveMsecs: 1000,
  maxSockets: 50,
  timeout: 60000
});

const client = new DataFoldClient({
  ...config,
  httpAgent: httpsAgent
});
```

### SSL/TLS Issues

**Symptoms:**
- SSL certificate errors
- TLS handshake failures
- Certificate validation errors

**Diagnosis:**
```bash
# Test SSL connection
openssl s_client -connect api.datafold.com:443 -servername api.datafold.com

# Check certificate details
curl -vI https://api.datafold.com 2>&1 | grep -A 10 -B 10 "certificate"
```

**Solutions:**

#### 1. Update CA Certificates
```bash
# Ubuntu/Debian
sudo apt-get update && sudo apt-get install ca-certificates

# CentOS/RHEL
sudo yum update ca-certificates

# macOS
brew install ca-certificates
```

#### 2. Configure TLS Settings
```python
import ssl
import requests
from requests.adapters import HTTPAdapter

class SecureHTTPAdapter(HTTPAdapter):
    def __init__(self):
        self.ssl_context = ssl.create_default_context()
        self.ssl_context.minimum_version = ssl.TLSVersion.TLSv1_2
        super().__init__()
    
    def init_poolmanager(self, *args, **kwargs):
        kwargs['ssl_context'] = self.ssl_context
        return super().init_poolmanager(*args, **kwargs)

session = requests.Session()
session.mount('https://', SecureHTTPAdapter())
```

## 🔧 Configuration Issues

### Missing Configuration

**Symptoms:**
- "Configuration not found" errors
- Default values being used unexpectedly
- Environment variables not loading

**Diagnosis:**
```bash
# Check configuration file location
datafold config path

# List all configuration
datafold config list --verbose

# Check environment variables
env | grep DATAFOLD
```

**Solutions:**

#### 1. Create Configuration File
```bash
# Create default configuration
datafold config set server-url https://api.datafold.com
datafold config set client-id your-client-id
datafold config set key-file ~/.datafold/keys/private.pem
```

#### 2. Set Environment Variables
```bash
# Set required environment variables
export DATAFOLD_SERVER_URL="https://api.datafold.com"
export DATAFOLD_CLIENT_ID="your-client-id"
export DATAFOLD_PRIVATE_KEY="$(cat ~/.datafold/keys/private.pem)"
```

### Invalid Configuration Values

**Symptoms:**
- Configuration validation errors
- Unexpected behavior
- Type conversion errors

**Diagnosis & Solutions:**

```javascript
// Validate configuration before using
function validateConfig(config) {
  const errors = [];
  
  if (!config.serverUrl) {
    errors.push('serverUrl is required');
  } else if (!config.serverUrl.startsWith('https://')) {
    errors.push('serverUrl must use HTTPS in production');
  }
  
  if (!config.clientId) {
    errors.push('clientId is required');
  } else if (config.clientId.length > 64) {
    errors.push('clientId must be 64 characters or less');
  }
  
  if (!config.privateKey) {
    errors.push('privateKey is required');
  } else if (config.privateKey.length !== 32) {
    errors.push('privateKey must be exactly 32 bytes');
  }
  
  if (errors.length > 0) {
    throw new Error(`Configuration errors: ${errors.join(', ')}`);
  }
}

validateConfig(config);
```

## 🗝️ Key Management Issues

### Key File Permissions

**Symptoms:**
- Permission denied errors
- Keys not loading
- Security warnings

**Diagnosis:**
```bash
# Check file permissions
ls -la ~/.datafold/keys/

# Check file ownership
stat ~/.datafold/keys/private.pem
```

**Solutions:**
```bash
# Fix permissions
chmod 600 ~/.datafold/keys/private.pem  # Owner read/write only
chmod 644 ~/.datafold/keys/public.pem   # Owner read/write, others read
chmod 700 ~/.datafold/keys/             # Directory: owner access only

# Fix ownership (if needed)
chown $USER:$USER ~/.datafold/keys/*
```

### Key Format Issues

**Symptoms:**
- "Invalid key format" errors
- Key parsing failures
- Cryptographic errors

**Diagnosis & Solutions:**

```bash
# Verify key format
file ~/.datafold/keys/private.pem
openssl pkey -in ~/.datafold/keys/private.pem -text -noout

# Convert between formats if needed
# PEM to hex
openssl pkey -in private.pem -outform DER | xxd -p -c 32

# Hex to PEM
echo "your-hex-key" | xxd -r -p | openssl pkey -inform DER -outform PEM
```

```javascript
// Validate key format in code
function validatePrivateKey(keyBytes) {
  if (!(keyBytes instanceof Uint8Array)) {
    throw new Error('Private key must be Uint8Array');
  }
  
  if (keyBytes.length !== 32) {
    throw new Error('Ed25519 private key must be exactly 32 bytes');
  }
  
  // Additional validation: ensure key is not all zeros
  if (keyBytes.every(byte => byte === 0)) {
    throw new Error('Invalid private key: all zeros');
  }
}
```

### Key Rotation Issues

**Symptoms:**
- Old keys still being used
- New keys not recognized
- Authentication failures after rotation

**Solutions:**

#### 1. Proper Key Rotation Process
```bash
# 1. Generate new keypair
datafold auth keygen --output ~/.datafold/keys-new/

# 2. Register new public key
datafold auth register \
  --key-file ~/.datafold/keys-new/public.pem \
  --client-id your-client-id-v2

# 3. Test new keys
DATAFOLD_CLIENT_ID=your-client-id-v2 \
DATAFOLD_PRIVATE_KEY="$(cat ~/.datafold/keys-new/private.pem)" \
datafold auth test

# 4. Update configuration
datafold config set client-id your-client-id-v2
datafold config set key-file ~/.datafold/keys-new/private.pem

# 5. Backup old keys
mv ~/.datafold/keys ~/.datafold/keys-backup-$(date +%Y%m%d)
mv ~/.datafold/keys-new ~/.datafold/keys
```

#### 2. Zero-Downtime Rotation
```python
# Gradual migration approach
class KeyRotationManager:
    def __init__(self, old_client, new_client):
        self.old_client = old_client
        self.new_client = new_client
        self.migration_percentage = 0
    
    def make_request(self, method, url, **kwargs):
        # Gradually migrate to new keys
        import random
        
        if random.randint(1, 100) <= self.migration_percentage:
            try:
                return self.new_client.request(method, url, **kwargs)
            except AuthenticationError:
                # Fallback to old client
                return self.old_client.request(method, url, **kwargs)
        else:
            return self.old_client.request(method, url, **kwargs)
    
    def increase_migration(self, percentage):
        self.migration_percentage = min(100, percentage)

# Usage
rotation_manager = KeyRotationManager(old_client, new_client)
rotation_manager.increase_migration(25)  # 25% of requests use new keys
```

## 📱 Platform-Specific Issues

### Browser Issues

**Symptoms:**
- WebCrypto API not available
- Cross-origin request issues
- LocalStorage limitations

**Solutions:**

#### 1. WebCrypto API Compatibility
```javascript
// Check WebCrypto availability
if (!window.crypto || !window.crypto.subtle) {
  throw new Error('WebCrypto API not available. Use HTTPS or modern browser.');
}

// Fallback for older browsers
async function generateKeyPairWithFallback() {
  if (window.crypto && window.crypto.subtle) {
    return await generateKeyPair();
  } else {
    // Use polyfill or server-side generation
    throw new Error('Please upgrade to a modern browser');
  }
}
```

#### 2. CORS Configuration
```javascript
// Handle CORS issues
const client = new DataFoldClient({
  ...config,
  headers: {
    'Accept': 'application/json',
    'Content-Type': 'application/json'
  },
  credentials: 'include'  // If needed for cookies
});
```

### Node.js Issues

**Symptoms:**
- Module resolution errors
- Crypto library issues
- File system permission errors

**Solutions:**

#### 1. Module Compatibility
```javascript
// Handle ESM/CommonJS compatibility
let ed25519;
try {
  ed25519 = await import('@noble/ed25519');
} catch (error) {
  ed25519 = require('@noble/ed25519');
}
```

#### 2. File System Access
```javascript
// Safe file operations
import { promises as fs } from 'fs';
import { constants } from 'fs';

async function loadPrivateKey(keyPath) {
  try {
    // Check if file exists and is readable
    await fs.access(keyPath, constants.R_OK);
    
    const keyData = await fs.readFile(keyPath, 'utf8');
    return parsePrivateKey(keyData);
  } catch (error) {
    throw new Error(`Cannot load private key from ${keyPath}: ${error.message}`);
  }
}
```

### Python Issues

**Symptoms:**
- Import errors
- SSL context issues
- Asyncio compatibility problems

**Solutions:**

#### 1. Dependency Management
```python
# Check required dependencies
try:
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
    print("✓ Cryptography library available")
except ImportError:
    print("✗ Install cryptography: pip install cryptography")

try:
    import requests
    print("✓ Requests library available")
except ImportError:
    print("✗ Install requests: pip install requests")
```

#### 2. Asyncio Compatibility
```python
# Handle asyncio issues
import asyncio
import sys

def ensure_event_loop():
    """Ensure we have an event loop"""
    try:
        loop = asyncio.get_event_loop()
        if loop.is_closed():
            raise RuntimeError("Event loop is closed")
    except RuntimeError:
        # Create new event loop
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)
    
    return loop

# For Jupyter notebooks
if sys.platform == 'win32':
    asyncio.set_event_loop_policy(asyncio.WindowsProactorEventLoopPolicy())
```

## 🔬 Debugging Tools

### Request/Response Debugging

```bash
# CLI: Debug mode
export DATAFOLD_DEBUG=true
datafold api get /api/schemas --show-request --show-headers

# Show signature details
datafold auth test --show-request
```

```javascript
// JavaScript: Debug logging
const client = new DataFoldClient({
  ...config,
  debug: true,
  requestInterceptor: (config) => {
    console.log('Request:', {
      method: config.method,
      url: config.url,
      headers: config.headers
    });
    return config;
  },
  responseInterceptor: (response) => {
    console.log('Response:', {
      status: response.status,
      headers: response.headers,
      data: response.data
    });
    return response;
  }
});
```

```python
# Python: Debug logging
import logging

# Enable debug logging
logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger('datafold_sdk')

# Custom debug client
class DebugDataFoldClient(DataFoldClient):
    def request(self, method, url, **kwargs):
        logger.debug(f"Request: {method} {url}")
        logger.debug(f"Headers: {kwargs.get('headers', {})}")
        
        response = super().request(method, url, **kwargs)
        
        logger.debug(f"Response: {response.status_code}")
        logger.debug(f"Response headers: {dict(response.headers)}")
        
        return response
```

### Signature Validation Testing

```python
# Test signature generation and validation
from datafold_sdk.signing import RFC9421Signer
from datafold_sdk.crypto import generate_keypair

def test_signature_roundtrip():
    # Generate test keypair
    private_key, public_key = generate_keypair()
    
    # Create signer
    signer = RFC9421Signer({
        'algorithm': 'ed25519',
        'key_id': 'test-client',
        'private_key': private_key
    })
    
    # Create test request
    request = {
        'method': 'POST',
        'url': 'https://api.datafold.com/api/test',
        'headers': {'content-type': 'application/json'},
        'body': '{"test": true}'
    }
    
    # Sign request
    result = signer.sign_request(request)
    
    print("Signature Input:", result.signature_input)
    print("Signature:", result.signature)
    print("Canonical Message:", result.canonical_message)
    
    # Verify signature (client-side verification for testing)
    from datafold_sdk.crypto.ed25519 import verify_signature
    
    message_bytes = result.canonical_message.encode('utf-8')
    signature_bytes = bytes.fromhex(result.signature.split(':')[1])
    
    is_valid = verify_signature(public_key, message_bytes, signature_bytes)
    print(f"Signature valid: {is_valid}")
    
    return is_valid

# Run test
if test_signature_roundtrip():
    print("✓ Signature roundtrip test passed")
else:
    print("✗ Signature roundtrip test failed")
```

### Network Diagnostics

```bash
# Network troubleshooting script
#!/bin/bash

echo "DataFold Network Diagnostics"
echo "============================"

# Basic connectivity
echo "1. Testing basic connectivity..."
if curl -s --max-time 10 https://api.datafold.com/api/status > /dev/null; then
    echo "✓ Basic connectivity: OK"
else
    echo "✗ Basic connectivity: FAILED"
fi

# DNS resolution
echo "2. Testing DNS resolution..."
if nslookup api.datafold.com > /dev/null 2>&1; then
    echo "✓ DNS resolution: OK"
else
    echo "✗ DNS resolution: FAILED"
fi

# SSL certificate
echo "3. Testing SSL certificate..."
if openssl s_client -connect api.datafold.com:443 -servername api.datafold.com < /dev/null 2>/dev/null | grep -q "Verification: OK"; then
    echo "✓ SSL certificate: OK"
else
    echo "✗ SSL certificate: Issues detected"
fi

# Time synchronization
echo "4. Testing time synchronization..."
server_time=$(curl -s -I https://api.datafold.com/api/status | grep -i date | cut -d' ' -f2-)
local_time=$(date -u)
echo "   Server time: $server_time"
echo "   Local time:  $local_time"

# Authentication test
echo "5. Testing authentication..."
if datafold auth test > /dev/null 2>&1; then
    echo "✓ Authentication: OK"
else
    echo "✗ Authentication: FAILED"
fi

echo "Diagnostics complete."
```

## 📞 Getting Help

### Self-Service Resources

1. **Check Status Page**: [status.datafold.com](https://status.datafold.com)
2. **Review Documentation**: [docs.datafold.com](https://docs.datafold.com)
3. **Search Community Forum**: [community.datafold.com](https://community.datafold.com)
4. **GitHub Issues**: [github.com/datafold/sdk/issues](https://github.com/datafold/sdk/issues)

### When to Contact Support

Contact support if you experience:
- Persistent authentication failures after following this guide
- Suspected security issues or vulnerabilities
- Performance problems affecting production
- Issues that may indicate service problems

### Support Information

- **Community Support**: [community.datafold.com](https://community.datafold.com)
- **Enterprise Support**: [support@datafold.com](mailto:support@datafold.com)
- **Security Issues**: [security@datafold.com](mailto:security@datafold.com)
- **Sales Questions**: [sales@datafold.com](mailto:sales@datafold.com)

### Information to Include

When contacting support, include:
- DataFold SDK version
- Platform/language version
- Error messages (full stack traces)
- Steps to reproduce the issue
- Configuration (without private keys!)
- Network environment details

## 🔗 Related Documentation

- **[Authentication Overview](overview.md)** - How signature authentication works
- **[Getting Started](getting-started.md)** - Step-by-step setup guide  
- **[Security Best Practices](../guides/security-best-practices.md)** - Production security guidelines
- **[JavaScript SDK](../sdks/javascript/README.md)** - JavaScript-specific documentation
- **[Python SDK](../sdks/python/README.md)** - Python-specific documentation
- **[CLI Tool](../sdks/cli/README.md)** - CLI-specific documentation

---

**Need immediate help?** Start with the [Quick Diagnosis](#quick-diagnosis) section, then work through the relevant error message sections above.