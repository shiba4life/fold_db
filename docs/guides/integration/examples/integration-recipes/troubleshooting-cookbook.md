# Troubleshooting Cookbook

Complete troubleshooting guide for DataFold signature authentication integration issues. This cookbook provides step-by-step solutions for common problems encountered during development and production deployment.

## 🎯 Quick Diagnosis

### Authentication Flow Checker

```bash
#!/bin/bash
# auth-diagnostic.sh - Quick authentication diagnostic script

echo "🔍 DataFold Authentication Diagnostic"
echo "======================================"

# Check 1: Server connectivity
echo "1. Testing server connectivity..."
if curl -s --max-time 10 "${DATAFOLD_SERVER_URL:-https://api.datafold.com}/health" > /dev/null; then
    echo "✅ Server is reachable"
else
    echo "❌ Server is not reachable"
    echo "   Check DATAFOLD_SERVER_URL: ${DATAFOLD_SERVER_URL:-https://api.datafold.com}"
fi

# Check 2: Environment variables
echo -e "\n2. Checking environment variables..."
if [ -n "$DATAFOLD_CLIENT_ID" ]; then
    echo "✅ DATAFOLD_CLIENT_ID is set"
else
    echo "❌ DATAFOLD_CLIENT_ID is not set"
fi

if [ -n "$DATAFOLD_PRIVATE_KEY" ]; then
    echo "✅ DATAFOLD_PRIVATE_KEY is set"
else
    echo "❌ DATAFOLD_PRIVATE_KEY is not set"
fi

# Check 3: System time
echo -e "\n3. Checking system time..."
SYSTEM_TIME=$(date -u +%s)
SERVER_TIME=$(curl -s "${DATAFOLD_SERVER_URL:-https://api.datafold.com}/api/system/time" | grep -o '"timestamp":[0-9]*' | cut -d: -f2)
if [ -n "$SERVER_TIME" ]; then
    TIME_DIFF=$((SYSTEM_TIME - SERVER_TIME))
    if [ ${TIME_DIFF#-} -lt 300 ]; then
        echo "✅ System time is synchronized (diff: ${TIME_DIFF}s)"
    else
        echo "❌ System time is out of sync (diff: ${TIME_DIFF}s)"
        echo "   Run: sudo ntpdate -s time.nist.gov"
    fi
else
    echo "⚠️  Could not check server time"
fi

# Check 4: Network configuration
echo -e "\n4. Testing network configuration..."
if curl -s -I "${DATAFOLD_SERVER_URL:-https://api.datafold.com}" | grep -q "signature"; then
    echo "✅ Signature headers are preserved"
else
    echo "⚠️  Check if proxy strips signature headers"
fi

echo -e "\n5. Certificate validation..."
if curl -s --cert-status "${DATAFOLD_SERVER_URL:-https://api.datafold.com}" > /dev/null 2>&1; then
    echo "✅ SSL certificates are valid"
else
    echo "⚠️  SSL certificate issues detected"
fi

echo -e "\nDiagnostic complete!"
```

## 🔧 Common Issues & Solutions

### Issue 1: "Signature verification failed"

**Symptoms:**
- HTTP 401 responses with "Invalid signature" error
- Authentication works locally but fails in production
- Intermittent signature failures

**Root Causes & Solutions:**

#### A. System Time Synchronization

```bash
# Problem: System clock is out of sync
# Signatures include timestamps and have a validity window (usually 5 minutes)

# Check time difference
curl -v https://api.datafold.com/api/system/time
date -u

# Solution: Synchronize system time
sudo ntpdate -s time.nist.gov

# For Docker containers
RUN apt-get update && apt-get install -y ntp
CMD ["ntpd", "-gq"]

# For production servers
echo "0 */6 * * * root ntpdate -s time.nist.gov" >> /etc/crontab
```

#### B. Header Manipulation by Proxies

```javascript
// Problem: Reverse proxy or load balancer strips/modifies signature headers

// Test if headers are preserved
const testHeaders = async () => {
  const response = await fetch('https://httpbin.org/headers', {
    headers: {
      'Signature': 'test-signature',
      'Signature-Input': 'test-input'
    }
  });
  
  const result = await response.json();
  console.log('Headers received:', result.headers);
};

// Solution: Configure proxy to preserve headers
// Nginx example:
/*
location /api/ {
    proxy_pass http://backend;
    proxy_set_header Signature $http_signature;
    proxy_set_header Signature-Input $http_signature_input;
    proxy_set_header Host $host;
}
*/

// HAProxy example:
/*
http-request set-header Signature %[req.hdr(signature)]
http-request set-header Signature-Input %[req.hdr(signature-input)]
*/
```

#### C. Incorrect Message Canonicalization

```javascript
// Problem: Request components don't match what's signed

// Debug signature components
const debugSignature = (request) => {
  console.log('Request details for signing:');
  console.log('Method:', request.method);
  console.log('URL:', request.url);
  console.log('Headers:', Object.entries(request.headers)
    .filter(([key]) => key.toLowerCase().startsWith('content-'))
    .sort());
  console.log('Body hash:', calculateBodyHash(request.body));
};

// Solution: Ensure consistent canonicalization
const canonicalizeRequest = (request) => {
  return {
    method: request.method.toUpperCase(),
    url: new URL(request.url).href, // Normalize URL
    headers: normalizeHeaders(request.headers),
    body: request.body || ''
  };
};

const normalizeHeaders = (headers) => {
  const normalized = {};
  Object.entries(headers).forEach(([key, value]) => {
    normalized[key.toLowerCase()] = value.trim();
  });
  return normalized;
};
```

#### D. Key Mismatch

```python
# Problem: Private key doesn't match registered public key

# Verify key pair consistency
from datafold_sdk import verify_key_pair

def verify_credentials(private_key_hex, public_key_hex):
    try:
        private_key = bytes.fromhex(private_key_hex)
        public_key = bytes.fromhex(public_key_hex)
        
        if verify_key_pair(private_key, public_key):
            print("✅ Key pair is valid")
            return True
        else:
            print("❌ Key pair mismatch")
            return False
    except Exception as e:
        print(f"❌ Key validation error: {e}")
        return False

# Solution: Re-register with correct public key
async def re_register_key(client_id, private_key):
    # Generate public key from private key
    public_key = derive_public_key(private_key)
    
    # Re-register
    response = await register_public_key({
        'client_id': client_id,
        'public_key': public_key.hex(),
        'force_update': True
    })
    
    print(f"Re-registered key for client: {client_id}")
    return response
```

### Issue 2: "Authentication required" in development

**Symptoms:**
- Protected endpoints return 401 even with valid configuration
- Authentication works in tests but not in development server

**Solutions:**

#### A. Environment Configuration

```javascript
// Check environment variables
const validateConfig = () => {
  const required = ['DATAFOLD_SERVER_URL'];
  const optional = ['DATAFOLD_CLIENT_ID', 'DATAFOLD_PRIVATE_KEY'];
  
  console.log('🔧 Configuration Check:');
  
  required.forEach(key => {
    const value = process.env[key];
    if (value) {
      console.log(`✅ ${key}: ${value}`);
    } else {
      console.log(`❌ ${key}: NOT SET`);
    }
  });
  
  optional.forEach(key => {
    const value = process.env[key];
    if (value) {
      console.log(`✅ ${key}: ***${value.slice(-8)}`);
    } else {
      console.log(`⚠️  ${key}: NOT SET (will auto-generate)`);
    }
  });
};

// Development environment file
// .env.development
/*
NODE_ENV=development
DATAFOLD_SERVER_URL=https://api.datafold.com
DATAFOLD_ENABLE_AUTH=true
DATAFOLD_REQUIRE_AUTH=false  # Allow fallback in development
DEBUG=datafold:*
*/
```

#### B. Development Mode Setup

```typescript
// Development authentication bypass
const createDevelopmentAuth = () => {
  if (process.env.NODE_ENV === 'development') {
    return {
      middleware: (req, res, next) => {
        req.datafold = {
          isAuthenticated: true,
          clientId: 'dev-client',
          signature: {
            valid: true,
            timestamp: new Date(),
            components: ['method', 'target-uri']
          }
        };
        next();
      }
    };
  }
  
  return { middleware: createActualAuthMiddleware() };
};

// Conditional authentication
const authMiddleware = process.env.BYPASS_AUTH === 'true' 
  ? createDevelopmentAuth().middleware
  : createProductionAuth().middleware;
```

### Issue 3: Performance degradation after enabling authentication

**Symptoms:**
- Increased response times after enabling signature authentication
- High CPU usage during signature operations
- Memory leaks in long-running applications

**Solutions:**

#### A. Enable Signature Caching

```javascript
// Configure aggressive caching for performance
const optimizedClient = new DataFoldClient({
  serverUrl: 'https://api.datafold.com',
  clientId: 'my-client',
  privateKey: privateKey,
  
  // Performance optimizations
  signatureCache: {
    enabled: true,
    ttl: 300, // 5 minutes
    maxSize: 10000 // Large cache for high-volume apps
  },
  
  // Connection pooling
  httpClient: {
    keepAlive: true,
    maxSockets: 20,
    timeout: 30000
  }
});

// Monitor cache effectiveness
setInterval(() => {
  const metrics = optimizedClient.getSigningMetrics();
  const hitRate = metrics.cacheHits / (metrics.cacheHits + metrics.cacheMisses);
  
  console.log(`Signature cache hit rate: ${(hitRate * 100).toFixed(2)}%`);
  
  if (hitRate < 0.8) {
    console.warn('Low cache hit rate - consider increasing TTL or cache size');
  }
}, 60000);
```

#### B. Optimize Signature Components

```python
# Minimize signed components for better performance
fast_signing_config = {
    'algorithm': 'ed25519',
    'key_id': 'my-client',
    'private_key': private_key,
    
    # Minimal components for better performance
    'components': [
        'method',
        'target_uri'
        # Skip content-digest for GET requests
    ]
}

# Conditional component signing
def get_signing_components(request):
    components = ['method', 'target_uri']
    
    # Only include content-digest for requests with body
    if request.method in ['POST', 'PUT', 'PATCH'] and request.body:
        components.append('content_digest')
    
    # Include authorization header if present
    if 'authorization' in request.headers:
        components.append('authorization')
    
    return components
```

#### C. Async Processing

```typescript
// Use async processing to avoid blocking
class AsyncSignatureService {
  private signingQueue: Array<{request: any, resolve: Function, reject: Function}> = [];
  private isProcessing = false;

  async signRequest(request: any): Promise<any> {
    return new Promise((resolve, reject) => {
      this.signingQueue.push({ request, resolve, reject });
      this.processQueue();
    });
  }

  private async processQueue() {
    if (this.isProcessing || this.signingQueue.length === 0) {
      return;
    }

    this.isProcessing = true;

    try {
      // Process in batches for better performance
      const batch = this.signingQueue.splice(0, 10);
      
      await Promise.all(batch.map(async ({ request, resolve, reject }) => {
        try {
          const signedRequest = await this.actuallySignRequest(request);
          resolve(signedRequest);
        } catch (error) {
          reject(error);
        }
      }));

    } finally {
      this.isProcessing = false;
      
      // Process remaining queue
      if (this.signingQueue.length > 0) {
        setImmediate(() => this.processQueue());
      }
    }
  }

  private async actuallySignRequest(request: any): Promise<any> {
    // Actual signing logic here
    return signedRequest;
  }
}
```

### Issue 4: Docker container authentication failures

**Symptoms:**
- Authentication works on host but fails in Docker container
- Containers can't connect to DataFold server
- SSL/TLS errors in containerized environments

**Solutions:**

#### A. Network Configuration

```dockerfile
# Dockerfile with proper network configuration
FROM node:18-alpine

# Install CA certificates for SSL
RUN apk add --no-cache ca-certificates

# Update certificate store
RUN update-ca-certificates

# Set proper DNS
RUN echo "nameserver 8.8.8.8" > /etc/resolv.conf

# Create app user
RUN addgroup -g 1001 -S nodejs && adduser -D -S -G nodejs -u 1001 nextjs

WORKDIR /app

# Copy and install dependencies
COPY package*.json ./
RUN npm ci --only=production

# Copy application
COPY . .

# Set environment
ENV NODE_ENV=production
ENV DATAFOLD_SERVER_URL=https://api.datafold.com

USER nextjs

EXPOSE 3000

CMD ["npm", "start"]
```

#### B. Container-specific Environment

```yaml
# docker-compose.yml
version: '3.8'

services:
  app:
    build: .
    environment:
      - NODE_ENV=production
      - DATAFOLD_SERVER_URL=https://api.datafold.com
      - DATAFOLD_CLIENT_ID=${DATAFOLD_CLIENT_ID}
      - DATAFOLD_PRIVATE_KEY=${DATAFOLD_PRIVATE_KEY}
      
      # Network configuration
      - NODE_TLS_REJECT_UNAUTHORIZED=1
      - HTTPS_PROXY=${HTTPS_PROXY:-}
      - NO_PROXY=localhost,127.0.0.1
      
    # DNS configuration
    dns:
      - 8.8.8.8
      - 8.8.4.4
      
    # Add necessary capabilities
    cap_add:
      - NET_ADMIN
    
    # Health check
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s
```

#### C. Container Debugging

```bash
#!/bin/bash
# debug-container.sh

# Enter container for debugging
docker exec -it container-name /bin/sh

# Check network connectivity
nslookup api.datafold.com
curl -v https://api.datafold.com/health

# Check certificates
curl -v --cert-status https://api.datafold.com/health

# Check environment
env | grep DATAFOLD

# Check application logs
docker logs --tail 100 container-name

# Check DNS resolution
cat /etc/resolv.conf
```

### Issue 5: CI/CD pipeline authentication failures

**Symptoms:**
- Tests pass locally but fail in CI/CD
- Secrets not properly injected
- Environment-specific configuration issues

**Solutions:**

#### A. CI/CD Configuration

```yaml
# .github/workflows/test.yml
name: Test with Authentication

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    env:
      NODE_ENV: test
      DATAFOLD_SERVER_URL: https://test-api.datafold.com
      DATAFOLD_ENABLE_AUTH: true
      DATAFOLD_REQUIRE_AUTH: false
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '18'
        cache: 'npm'
    
    - name: Install dependencies
      run: npm ci
    
    - name: Setup test credentials
      env:
        DATAFOLD_CLIENT_ID: ${{ secrets.TEST_DATAFOLD_CLIENT_ID }}
        DATAFOLD_PRIVATE_KEY: ${{ secrets.TEST_DATAFOLD_PRIVATE_KEY }}
      run: |
        echo "Setting up test authentication..."
        # Validate credentials format
        if [[ ${#DATAFOLD_PRIVATE_KEY} -ne 64 ]]; then
          echo "❌ Invalid private key format"
          exit 1
        fi
    
    - name: Run tests
      run: |
        npm run test:unit
        npm run test:integration
      env:
        DATAFOLD_CLIENT_ID: ${{ secrets.TEST_DATAFOLD_CLIENT_ID }}
        DATAFOLD_PRIVATE_KEY: ${{ secrets.TEST_DATAFOLD_PRIVATE_KEY }}
```

#### B. Secret Management

```bash
# Set up CI/CD secrets properly

# GitHub Actions
gh secret set DATAFOLD_CLIENT_ID --body "test-client-id"
gh secret set DATAFOLD_PRIVATE_KEY --body "$(cat private-key.hex)"

# GitLab CI
gitlab-ci-yml:
variables:
  DATAFOLD_SERVER_URL: "https://test-api.datafold.com"

test:
  variables:
    DATAFOLD_CLIENT_ID: $TEST_CLIENT_ID
    DATAFOLD_PRIVATE_KEY: $TEST_PRIVATE_KEY
  script:
    - npm test

# Jenkins
// In Jenkins pipeline
withCredentials([
    string(credentialsId: 'datafold-client-id', variable: 'DATAFOLD_CLIENT_ID'),
    string(credentialsId: 'datafold-private-key', variable: 'DATAFOLD_PRIVATE_KEY')
]) {
    sh 'npm test'
}
```

### Issue 6: "Client ID not found" errors

**Symptoms:**
- Server returns 404 for client ID
- Authentication was working previously
- New deployments can't authenticate

**Solutions:**

#### A. Key Registration Status Check

```javascript
// Check if client is still registered
const checkRegistrationStatus = async (clientId) => {
  try {
    const response = await fetch(`${serverUrl}/api/crypto/keys/${clientId}/status`);
    
    if (response.ok) {
      const status = await response.json();
      console.log('Registration status:', status);
      return status;
    } else {
      console.log('Client ID not found or expired');
      return null;
    }
  } catch (error) {
    console.error('Failed to check registration:', error);
    return null;
  }
};

// Re-register if needed
const ensureRegistration = async (clientId, publicKey) => {
  const status = await checkRegistrationStatus(clientId);
  
  if (!status || status.data.status !== 'active') {
    console.log('Re-registering client...');
    
    const response = await fetch(`${serverUrl}/api/crypto/keys/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        client_id: clientId,
        public_key: publicKey,
        force_update: true
      })
    });
    
    const result = await response.json();
    console.log('Re-registration result:', result);
    return result;
  }
  
  return status;
};
```

#### B. Automatic Recovery

```python
# Automatic client registration recovery
class AutoRecoveringDataFoldClient:
    def __init__(self, config):
        self.config = config
        self.client = None
        self.last_registration_check = 0
        
    async def make_request(self, method, path, **kwargs):
        try:
            if not self.client:
                await self._initialize_client()
            
            return await self.client.request(method, path, **kwargs)
            
        except AuthenticationError as e:
            if 'client_id' in str(e).lower():
                print("Client ID error detected, attempting recovery...")
                await self._recover_registration()
                return await self.client.request(method, path, **kwargs)
            raise
    
    async def _recover_registration(self):
        """Attempt to recover from client ID errors"""
        try:
            # Generate new credentials
            private_key, public_key = generate_key_pair()
            new_client_id = f"recovered-{int(time.time())}"
            
            # Register new client
            await self._register_client(new_client_id, public_key)
            
            # Update configuration
            self.config.client_id = new_client_id
            self.config.private_key = private_key
            
            # Reinitialize client
            await self._initialize_client()
            
            print(f"Successfully recovered with new client ID: {new_client_id}")
            
        except Exception as e:
            print(f"Failed to recover registration: {e}")
            raise
```

## 🔍 Advanced Debugging

### Debug Logging Configuration

```javascript
// Enable comprehensive debug logging
process.env.DEBUG = 'datafold:*,http:*,signature:*';

const debug = require('debug');
const debugAuth = debug('datafold:auth');
const debugHttp = debug('datafold:http');
const debugSig = debug('datafold:signature');

// Log all authentication events
const logAuthEvent = (event, details) => {
  const timestamp = new Date().toISOString();
  debugAuth(`[${timestamp}] ${event}:`, details);
};

// Log HTTP requests with full details
const logHttpRequest = (request, response) => {
  debugHttp('Request:', {
    method: request.method,
    url: request.url,
    headers: Object.keys(request.headers),
    bodySize: request.body ? request.body.length : 0
  });
  
  debugHttp('Response:', {
    status: response.status,
    headers: Object.keys(response.headers),
    bodySize: response.data ? JSON.stringify(response.data).length : 0
  });
};

// Log signature details
const logSignatureDetails = (signature, components) => {
  debugSig('Signature generation:', {
    algorithm: signature.algorithm,
    keyId: signature.keyId,
    components: components,
    timestamp: signature.created
  });
};
```

### Network Traffic Analysis

```bash
# Capture network traffic for analysis
# Install tcpdump or wireshark

# Capture HTTPS traffic (requires SSL key)
sudo tcpdump -i any -s 0 -w datafold-traffic.pcap host api.datafold.com

# Analyze with curl verbose mode
curl -v -X POST https://api.datafold.com/api/test \
  -H "Signature: keyId=\"test\",algorithm=\"ed25519\",signature=\"abc123\"" \
  -H "Signature-Input: sig=(\"@method\" \"@target-uri\");created=1625097600" \
  -d '{"test": true}' \
  2>&1 | tee debug-output.txt

# Check SSL handshake
openssl s_client -connect api.datafold.com:443 -servername api.datafold.com
```

### Memory and Performance Profiling

```javascript
// Memory leak detection
const memwatch = require('@airbnb/node-memwatch');

memwatch.on('leak', (info) => {
  console.error('Memory leak detected:', info);
});

memwatch.on('stats', (stats) => {
  console.log('Memory stats:', {
    current_base: stats.current_base,
    estimated_base: stats.estimated_base,
    growth: stats.growth,
    reason: stats.reason
  });
});

// Profile signature operations
const profileSignature = async (operation, iterations = 1000) => {
  const startTime = process.hrtime.bigint();
  const startMemory = process.memoryUsage();
  
  for (let i = 0; i < iterations; i++) {
    await operation();
  }
  
  const endTime = process.hrtime.bigint();
  const endMemory = process.memoryUsage();
  
  const duration = Number(endTime - startTime) / 1000000; // Convert to ms
  const memoryDelta = endMemory.heapUsed - startMemory.heapUsed;
  
  console.log(`Performance profile for ${operation.name}:`);
  console.log(`  Total time: ${duration.toFixed(2)}ms`);
  console.log(`  Average time: ${(duration / iterations).toFixed(2)}ms`);
  console.log(`  Memory delta: ${(memoryDelta / 1024 / 1024).toFixed(2)}MB`);
  console.log(`  Operations/sec: ${(iterations / (duration / 1000)).toFixed(2)}`);
};
```

## 🎯 Prevention Strategies

### 1. Automated Health Checks

```yaml
# health-check.yml - Kubernetes health check
apiVersion: v1
kind: ConfigMap
metadata:
  name: health-check-script
data:
  check.sh: |
    #!/bin/bash
    
    # Check authentication health
    RESPONSE=$(curl -s -w "%{http_code}" http://localhost:3000/health)
    HTTP_CODE="${RESPONSE: -3}"
    
    if [ "$HTTP_CODE" -eq 200 ]; then
      echo "Health check passed"
      exit 0
    else
      echo "Health check failed with code: $HTTP_CODE"
      exit 1
    fi
---
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: app
    livenessProbe:
      exec:
        command: ["/bin/bash", "/scripts/check.sh"]
      initialDelaySeconds: 30
      periodSeconds: 10
    readinessProbe:
      exec:
        command: ["/bin/bash", "/scripts/check.sh"]
      initialDelaySeconds: 5
      periodSeconds: 5
```

### 2. Monitoring and Alerting

```javascript
// monitoring.js - Comprehensive monitoring setup
const prometheus = require('prom-client');

// Metrics
const authSuccessCounter = new prometheus.Counter({
  name: 'datafold_auth_success_total',
  help: 'Total successful authentications'
});

const authFailureCounter = new prometheus.Counter({
  name: 'datafold_auth_failure_total',
  help: 'Total failed authentications',
  labelNames: ['error_type']
});

const signatureLatency = new prometheus.Histogram({
  name: 'datafold_signature_duration_seconds',
  help: 'Signature generation latency'
});

// Middleware to collect metrics
const metricsMiddleware = (req, res, next) => {
  const start = Date.now();
  
  res.on('finish', () => {
    const duration = (Date.now() - start) / 1000;
    
    if (req.datafold?.isAuthenticated) {
      authSuccessCounter.inc();
    } else if (res.statusCode === 401) {
      authFailureCounter.inc({ error_type: 'unauthorized' });
    }
    
    if (req.path.includes('/api/')) {
      signatureLatency.observe(duration);
    }
  });
  
  next();
};

// Alert rules (Prometheus/AlertManager)
/*
groups:
- name: datafold_auth
  rules:
  - alert: HighAuthFailureRate
    expr: rate(datafold_auth_failure_total[5m]) > 0.1
    for: 2m
    annotations:
      summary: "High authentication failure rate"
      
  - alert: SlowSignatureGeneration
    expr: histogram_quantile(0.95, datafold_signature_duration_seconds) > 0.5
    for: 1m
    annotations:
      summary: "Slow signature generation"
*/
```

### 3. Configuration Validation

```typescript
// config-validator.ts
export class ConfigValidator {
  static validateAuthConfig(config: any): ValidationResult {
    const errors: string[] = [];
    const warnings: string[] = [];
    
    // Required fields
    if (!config.serverUrl) {
      errors.push('serverUrl is required');
    } else if (!this.isValidUrl(config.serverUrl)) {
      errors.push('serverUrl must be a valid URL');
    }
    
    // Key validation
    if (config.privateKey) {
      if (!this.isValidPrivateKey(config.privateKey)) {
        errors.push('privateKey format is invalid');
      }
    } else if (config.requireAuth) {
      warnings.push('privateKey not provided - will auto-generate');
    }
    
    // Client ID validation
    if (config.clientId && !this.isValidClientId(config.clientId)) {
      errors.push('clientId format is invalid');
    }
    
    // Performance settings
    if (config.signatureCache?.ttl && config.signatureCache.ttl < 60) {
      warnings.push('signatureCache TTL is very low - may impact performance');
    }
    
    return {
      valid: errors.length === 0,
      errors,
      warnings
    };
  }
  
  private static isValidUrl(url: string): boolean {
    try {
      new URL(url);
      return true;
    } catch {
      return false;
    }
  }
  
  private static isValidPrivateKey(key: string): boolean {
    // Ed25519 private key should be 64 hex characters
    return /^[a-fA-F0-9]{64}$/.test(key);
  }
  
  private static isValidClientId(clientId: string): boolean {
    // Client ID should be alphanumeric with hyphens/underscores
    return /^[a-zA-Z0-9_-]+$/.test(clientId);
  }
}

interface ValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
}
```

## 📞 Getting Help

### Support Channels
- **GitHub Issues**: Report bugs and request features
- **Documentation**: Comprehensive guides and API reference
- **Community Forum**: Ask questions and share solutions
- **Discord/Slack**: Real-time community support

### Before Asking for Help
1. ✅ Run the diagnostic script above
2. ✅ Check this troubleshooting guide
3. ✅ Search existing issues and discussions
4. ✅ Prepare minimal reproduction case
5. ✅ Include relevant logs and configuration

### Bug Report Template
```markdown
## Bug Report

**Environment:**
- OS: [e.g., Ubuntu 20.04]
- Node.js/Python version: [e.g., Node 18.x]
- DataFold SDK version: [e.g., 1.2.3]
- Framework: [e.g., Express, FastAPI]

**Expected Behavior:**
[Clear description of expected behavior]

**Actual Behavior:**
[Clear description of what actually happens]

**Steps to Reproduce:**
1. [First step]
2. [Second step]
3. [And so on...]

**Configuration:**
```yaml
# Sanitized configuration (remove secrets!)
datafold:
  serverUrl: https://api.datafold.com
  enableAuth: true
  requireAuth: false
```

**Logs:**
```
[Include relevant error logs]
```

**Additional Context:**
[Any other context about the problem]
```

---

🔧 **Remember**: Most authentication issues are caused by time synchronization, network configuration, or key management problems. Start with the basics and work your way up to more complex debugging!

💡 **Pro Tip**: Keep this troubleshooting guide bookmarked and contribute back when you discover new solutions!