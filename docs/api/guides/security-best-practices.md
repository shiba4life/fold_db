# Security Best Practices

This guide covers essential security practices for implementing DataFold's signature authentication in production environments. Following these guidelines will help ensure your implementation is secure, compliant, and resilient.

## 🔐 Cryptographic Security

### Key Generation

#### ✅ Best Practices

```javascript
// Generate keys with proper entropy
import { generateKeyPair } from '@datafold/sdk';

// Use cryptographically secure random number generator
const keyPair = await generateKeyPair();

// Verify key strength
if (keyPair.privateKey.length !== 32) {
  throw new Error('Invalid private key length');
}
```

```python
# Use system entropy for key generation
from datafold_sdk.crypto import generate_keypair
import os

# Ensure sufficient entropy on Linux
if os.path.exists('/proc/sys/kernel/random/entropy_avail'):
    with open('/proc/sys/kernel/random/entropy_avail', 'r') as f:
        entropy = int(f.read().strip())
        if entropy < 1000:
            print("Warning: Low system entropy")

private_key, public_key = generate_keypair()
```

```bash
# Generate keys with CLI tool (uses system entropy)
datafold auth keygen --algorithm ed25519

# Verify key quality
openssl pkey -in ~/.datafold/keys/private.pem -text -noout
```

#### ❌ Avoid These Mistakes

- **Never use weak randomness**: Don't use `Math.random()`, `random.random()`, or other weak PRNGs
- **Don't reuse keys**: Generate unique keypairs for each client/environment
- **Avoid test keys in production**: Don't use hardcoded or example keys
- **Don't share private keys**: Each client should have its own unique keypair

### Key Storage

#### ✅ Secure Storage Practices

```python
# Secure key storage with proper permissions
import os
import stat
from pathlib import Path

def store_private_key(key_bytes: bytes, key_path: str):
    """Store private key with secure permissions"""
    key_file = Path(key_path)
    
    # Create directory with restricted permissions
    key_file.parent.mkdir(mode=0o700, exist_ok=True)
    
    # Write key file
    key_file.write_bytes(key_bytes)
    
    # Set restrictive permissions (owner read/write only)
    key_file.chmod(0o600)
    
    print(f"Private key stored securely at {key_path}")

# Environment variable storage (for containers)
import base64

# Store as base64 in environment
private_key_b64 = base64.b64encode(private_key_bytes).decode('ascii')
os.environ['DATAFOLD_PRIVATE_KEY_B64'] = private_key_b64

# Load from environment
private_key_bytes = base64.b64decode(os.environ['DATAFOLD_PRIVATE_KEY_B64'])
```

```javascript
// Browser: Use IndexedDB for secure storage
class SecureKeyStorage {
  async storePrivateKey(keyBytes) {
    const db = await this.openDB();
    const tx = db.transaction(['keys'], 'readwrite');
    const store = tx.objectStore('keys');
    
    // Store with encryption if available
    const encryptedKey = await this.encryptKey(keyBytes);
    await store.put({ id: 'private_key', data: encryptedKey });
  }
  
  async encryptKey(keyBytes) {
    // Use Web Crypto API for additional encryption
    const key = await crypto.subtle.generateKey(
      { name: 'AES-GCM', length: 256 },
      false,
      ['encrypt', 'decrypt']
    );
    
    const iv = crypto.getRandomValues(new Uint8Array(12));
    const encrypted = await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv },
      key,
      keyBytes
    );
    
    return { encrypted, iv, key };
  }
}
```

```bash
# File system security (Linux/macOS)
# Set proper permissions on key files
chmod 600 ~/.datafold/keys/private.pem
chmod 644 ~/.datafold/keys/public.pem

# Set proper directory permissions
chmod 700 ~/.datafold/keys/

# Use encrypted file systems for sensitive data
# LUKS (Linux)
sudo cryptsetup luksFormat /dev/sdb1
sudo cryptsetup luksOpen /dev/sdb1 encrypted_keys

# FileVault (macOS)
sudo fdesetup enable

# BitLocker (Windows)
manage-bde -on C:
```

#### Cloud and Container Security

```yaml
# Kubernetes Secrets
apiVersion: v1
kind: Secret
metadata:
  name: datafold-keys
type: Opaque
data:
  private-key: <base64-encoded-private-key>
stringData:
  client-id: "prod-client-123"
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: app
spec:
  template:
    spec:
      containers:
      - name: app
        env:
        - name: DATAFOLD_CLIENT_ID
          valueFrom:
            secretKeyRef:
              name: datafold-keys
              key: client-id
        - name: DATAFOLD_PRIVATE_KEY
          valueFrom:
            secretKeyRef:
              name: datafold-keys
              key: private-key
        # Mount as file for additional security
        volumeMounts:
        - name: datafold-keys
          mountPath: /etc/datafold/keys
          readOnly: true
      volumes:
      - name: datafold-keys
        secret:
          secretName: datafold-keys
          defaultMode: 0400  # Read-only for owner
```

```dockerfile
# Docker security best practices
FROM node:18-alpine

# Create non-root user
RUN addgroup -g 1001 -S datafold && \
    adduser -S datafold -u 1001 -G datafold

# Create secure key directory
RUN mkdir -p /app/keys && \
    chown datafold:datafold /app/keys && \
    chmod 700 /app/keys

USER datafold

# Copy application (no keys in image)
COPY --chown=datafold:datafold package*.json ./
RUN npm ci --only=production

COPY --chown=datafold:datafold . .

# Use runtime secrets, not build-time
ENTRYPOINT ["node", "app.js"]
```

#### ❌ Insecure Storage Patterns

```javascript
// ❌ DON'T: Store keys in code
const PRIVATE_KEY = "1234567890abcdef..."; // Never do this!

// ❌ DON'T: Store keys in client-side code
localStorage.setItem('private_key', privateKey); // Exposed to XSS

// ❌ DON'T: Log private keys
console.log('Private key:', privateKey); // Logs are often visible

// ❌ DON'T: Send keys over insecure channels
fetch('http://api.example.com/register', {
  body: JSON.stringify({ private_key: privateKey }) // HTTP = insecure
});
```

## 🛡️ Request Security

### Signature Validation

#### Server-Side Implementation

```rust
// Comprehensive signature validation
use crate::crypto::ed25519::verify_signature;
use chrono::{DateTime, Utc, Duration};

#[derive(Debug)]
pub struct SignatureValidationConfig {
    pub max_timestamp_skew: Duration,
    pub nonce_ttl: Duration,
    pub required_components: Vec<String>,
    pub strict_mode: bool,
}

impl Default for SignatureValidationConfig {
    fn default() -> Self {
        Self {
            max_timestamp_skew: Duration::minutes(5),
            nonce_ttl: Duration::minutes(6),
            required_components: vec![
                "@method".to_string(),
                "@target-uri".to_string(),
                "content-type".to_string(),
                "content-digest".to_string(),
            ],
            strict_mode: true,
        }
    }
}

pub fn validate_signature_comprehensive(
    request: &HttpRequest,
    public_key: &[u8; 32],
    config: &SignatureValidationConfig,
) -> Result<(), ValidationError> {
    // 1. Extract signature components
    let signature_input = extract_signature_input(request)?;
    let signature = extract_signature(request)?;
    
    // 2. Validate timestamp
    let created = signature_input.created
        .ok_or(ValidationError::MissingTimestamp)?;
    
    let now = Utc::now();
    let created_time = DateTime::from_timestamp(created, 0)
        .ok_or(ValidationError::InvalidTimestamp)?;
    
    if now.signed_duration_since(created_time) > config.max_timestamp_skew {
        return Err(ValidationError::ExpiredSignature);
    }
    
    // 3. Validate nonce uniqueness
    if let Some(nonce) = &signature_input.nonce {
        if !validate_nonce_uniqueness(nonce, config.nonce_ttl)? {
            return Err(ValidationError::NonceReuse);
        }
    } else if config.strict_mode {
        return Err(ValidationError::MissingNonce);
    }
    
    // 4. Validate required components
    for required_component in &config.required_components {
        if !signature_input.components.contains(required_component) {
            return Err(ValidationError::MissingRequiredComponent(required_component.clone()));
        }
    }
    
    // 5. Build canonical message
    let canonical_message = build_canonical_message(request, &signature_input)?;
    
    // 6. Verify signature
    if !verify_signature(public_key, canonical_message.as_bytes(), &signature)? {
        return Err(ValidationError::InvalidSignature);
    }
    
    // 7. Store nonce to prevent reuse
    if let Some(nonce) = &signature_input.nonce {
        store_nonce(nonce, config.nonce_ttl)?;
    }
    
    Ok(())
}
```

#### Security Levels

```yaml
# Security Profile Configurations

# Strict (High Security)
strict:
  max_timestamp_skew: 60s      # 1 minute
  nonce_required: true
  nonce_ttl: 90s              # 1.5 minutes
  required_components:
    - "@method"
    - "@target-uri" 
    - "content-type"
    - "content-digest"
    - "authorization"          # If present
  strict_component_order: true
  reject_future_timestamps: true
  max_future_timestamp: 10s

# Standard (Production)
standard:
  max_timestamp_skew: 300s     # 5 minutes
  nonce_required: true
  nonce_ttl: 360s             # 6 minutes
  required_components:
    - "@method"
    - "@target-uri"
    - "content-type"
    - "content-digest"
  strict_component_order: false
  reject_future_timestamps: true
  max_future_timestamp: 60s

# Lenient (Development)
lenient:
  max_timestamp_skew: 600s     # 10 minutes
  nonce_required: false
  nonce_ttl: 720s             # 12 minutes
  required_components:
    - "@method"
    - "@target-uri"
  strict_component_order: false
  reject_future_timestamps: false
  max_future_timestamp: 300s
```

### Content Integrity

#### Request Body Protection

```javascript
// Ensure request body integrity
import { createHash } from 'crypto';

function calculateContentDigest(body, algorithm = 'sha-256') {
  if (!body) return null;
  
  const hash = createHash('sha256');
  hash.update(typeof body === 'string' ? body : JSON.stringify(body));
  const digest = hash.digest('base64');
  
  return `${algorithm}=:${digest}:`;
}

// Always include content-digest for POST/PUT requests
const signableRequest = {
  method: 'POST',
  url: '/api/schemas',
  headers: {
    'content-type': 'application/json',
    'content-digest': calculateContentDigest(requestBody)
  },
  body: requestBody
};
```

```python
# Python content digest implementation
import hashlib
import base64
import json

def calculate_content_digest(body, algorithm='sha-256'):
    """Calculate RFC 9530 content digest"""
    if body is None:
        return None
    
    if isinstance(body, dict):
        body_bytes = json.dumps(body, sort_keys=True, separators=(',', ':')).encode('utf-8')
    elif isinstance(body, str):
        body_bytes = body.encode('utf-8')
    else:
        body_bytes = body
    
    hash_obj = hashlib.sha256(body_bytes)
    digest = base64.b64encode(hash_obj.digest()).decode('ascii')
    
    return f'{algorithm}=:{digest}:'

# Validate content digest on server
def validate_content_digest(request_body, content_digest_header):
    """Validate that request body matches content digest"""
    if not content_digest_header:
        return False
    
    # Parse header: "sha-256=:digest:"
    if not content_digest_header.startswith('sha-256=:') or not content_digest_header.endswith(':'):
        return False
    
    expected_digest = content_digest_header[9:-1]  # Remove "sha-256=:" and ":"
    calculated_digest = calculate_content_digest(request_body, 'sha-256')[9:-1]
    
    # Use constant-time comparison to prevent timing attacks
    import hmac
    return hmac.compare_digest(expected_digest, calculated_digest)
```

### Anti-Replay Protection

#### Nonce Management

```python
# Redis-based nonce storage for distributed systems
import redis
import uuid
import time

class NonceStore:
    def __init__(self, redis_client, ttl_seconds=360):
        self.redis = redis_client
        self.ttl = ttl_seconds
        self.prefix = "datafold:nonce:"
    
    def generate_nonce(self):
        """Generate cryptographically secure nonce"""
        return str(uuid.uuid4()).replace('-', '')
    
    def validate_and_store(self, nonce, timestamp):
        """Validate nonce uniqueness and store for TTL"""
        if not self.is_valid_nonce_format(nonce):
            raise ValueError("Invalid nonce format")
        
        key = f"{self.prefix}{nonce}"
        
        # Use SET with NX (only if not exists) and EX (expiration)
        # This is atomic and prevents race conditions
        result = self.redis.set(key, timestamp, nx=True, ex=self.ttl)
        
        if not result:
            raise ReplayAttackError("Nonce already used")
        
        return True
    
    def is_valid_nonce_format(self, nonce):
        """Validate nonce format (UUID4 without dashes)"""
        if len(nonce) != 32:
            return False
        
        try:
            int(nonce, 16)  # Must be valid hex
            return True
        except ValueError:
            return False
    
    def cleanup_expired(self):
        """Cleanup expired nonces (Redis handles this automatically with TTL)"""
        # Redis TTL handles cleanup automatically
        pass

# Usage
nonce_store = NonceStore(redis.Redis(host='localhost', port=6379, db=0))

# In request signing
nonce = nonce_store.generate_nonce()

# In request validation
try:
    nonce_store.validate_and_store(nonce, timestamp)
except ReplayAttackError:
    return HTTPResponse(status=401, body="Replay attack detected")
```

#### Timestamp Validation

```rust
// Robust timestamp validation
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone)]
pub struct TimestampConfig {
    pub max_clock_skew: Duration,
    pub max_future_time: Duration,
    pub require_monotonic: bool,
}

pub fn validate_timestamp(
    timestamp: i64,
    config: &TimestampConfig,
    last_timestamp: Option<i64>,
) -> Result<(), TimestampError> {
    let now = Utc::now().timestamp();
    let created_time = timestamp;
    
    // Check for reasonable timestamp bounds
    if created_time < 1_600_000_000 {  // Before 2020
        return Err(TimestampError::TooOld);
    }
    
    if created_time > now + 3600 {  // More than 1 hour in future
        return Err(TimestampError::TooFarInFuture);
    }
    
    // Check clock skew
    let age = now - created_time;
    if age > config.max_clock_skew.num_seconds() {
        return Err(TimestampError::Expired {
            age: Duration::seconds(age),
            max_age: config.max_clock_skew,
        });
    }
    
    // Check future timestamps
    if created_time > now + config.max_future_time.num_seconds() {
        return Err(TimestampError::FutureTimestamp {
            skew: Duration::seconds(created_time - now),
            max_skew: config.max_future_time,
        });
    }
    
    // Check monotonic requirement (for stateful clients)
    if config.require_monotonic {
        if let Some(last_ts) = last_timestamp {
            if created_time <= last_ts {
                return Err(TimestampError::NonMonotonic {
                    current: created_time,
                    previous: last_ts,
                });
            }
        }
    }
    
    Ok(())
}
```

## 🌐 Network Security

### TLS Configuration

#### Client-Side TLS

```javascript
// Strict TLS configuration
import https from 'https';

const httpsAgent = new https.Agent({
  // Require valid certificates
  rejectUnauthorized: true,
  
  // Pin certificate or CA
  ca: fs.readFileSync('datafold-ca-cert.pem'),
  
  // Use strong ciphers only
  ciphers: [
    'ECDHE-RSA-AES128-GCM-SHA256',
    'ECDHE-RSA-AES256-GCM-SHA384',
    'ECDHE-RSA-CHACHA20-POLY1305',
    'ECDHE-ECDSA-AES128-GCM-SHA256',
    'ECDHE-ECDSA-AES256-GCM-SHA384',
    'ECDHE-ECDSA-CHACHA20-POLY1305'
  ].join(':'),
  
  // Require TLS 1.2+
  minVersion: 'TLSv1.2',
  maxVersion: 'TLSv1.3'
});

const client = new DataFoldClient({
  serverUrl: 'https://api.datafold.com',
  httpAgent: httpsAgent
});
```

```python
# Python TLS configuration
import ssl
import requests
from requests.adapters import HTTPAdapter

class SecureHTTPAdapter(HTTPAdapter):
    def __init__(self, *args, **kwargs):
        self.ssl_context = ssl.create_default_context()
        
        # Strict certificate verification
        self.ssl_context.check_hostname = True
        self.ssl_context.verify_mode = ssl.CERT_REQUIRED
        
        # Strong cipher suites
        self.ssl_context.set_ciphers('ECDHE+AESGCM:ECDHE+CHACHA20:DHE+AESGCM:DHE+CHACHA20:!aNULL:!MD5:!DSS')
        
        # TLS 1.2+ only
        self.ssl_context.minimum_version = ssl.TLSVersion.TLSv1_2
        
        super().__init__(*args, **kwargs)
    
    def init_poolmanager(self, *args, **kwargs):
        kwargs['ssl_context'] = self.ssl_context
        return super().init_poolmanager(*args, **kwargs)

# Use with requests
session = requests.Session()
session.mount('https://', SecureHTTPAdapter())

client = DataFoldClient(session=session)
```

#### Certificate Pinning

```python
# Certificate pinning for additional security
import ssl
import hashlib
import base64

class CertificatePinner:
    def __init__(self, expected_fingerprints):
        self.expected_fingerprints = expected_fingerprints
    
    def verify_certificate(self, hostname, cert_der):
        """Verify certificate against pinned fingerprints"""
        # Calculate SHA-256 fingerprint
        fingerprint = hashlib.sha256(cert_der).digest()
        fingerprint_b64 = base64.b64encode(fingerprint).decode('ascii')
        
        if fingerprint_b64 not in self.expected_fingerprints:
            raise ssl.SSLError(f"Certificate fingerprint mismatch for {hostname}")
        
        return True

# DataFold production certificate fingerprints
DATAFOLD_CERT_PINS = [
    "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",  # Replace with actual
    "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB="   # Backup certificate
]

pinner = CertificatePinner(DATAFOLD_CERT_PINS)
```

### Rate Limiting and DDoS Protection

#### Client-Side Rate Limiting

```javascript
// Implement client-side rate limiting
class RateLimitedClient {
  constructor(client, options = {}) {
    this.client = client;
    this.requestsPerSecond = options.requestsPerSecond || 10;
    this.burstSize = options.burstSize || 20;
    this.tokens = this.burstSize;
    this.lastRefill = Date.now();
  }
  
  async request(method, url, options) {
    await this.waitForToken();
    return this.client.request(method, url, options);
  }
  
  async waitForToken() {
    const now = Date.now();
    const elapsed = (now - this.lastRefill) / 1000;
    
    // Refill tokens based on elapsed time
    this.tokens = Math.min(
      this.burstSize,
      this.tokens + elapsed * this.requestsPerSecond
    );
    this.lastRefill = now;
    
    if (this.tokens < 1) {
      const waitTime = (1 - this.tokens) / this.requestsPerSecond * 1000;
      await new Promise(resolve => setTimeout(resolve, waitTime));
      this.tokens = 1;
    }
    
    this.tokens -= 1;
  }
}

const rateLimitedClient = new RateLimitedClient(client, {
  requestsPerSecond: 5,
  burstSize: 10
});
```

#### Exponential Backoff

```python
# Implement exponential backoff for resilience
import time
import random
from typing import Callable, Any

class ExponentialBackoff:
    def __init__(self, 
                 initial_delay: float = 1.0,
                 max_delay: float = 60.0,
                 exponential_base: float = 2.0,
                 jitter: bool = True,
                 max_retries: int = 5):
        self.initial_delay = initial_delay
        self.max_delay = max_delay
        self.exponential_base = exponential_base
        self.jitter = jitter
        self.max_retries = max_retries
    
    def execute(self, func: Callable, *args, **kwargs) -> Any:
        """Execute function with exponential backoff retry"""
        delay = self.initial_delay
        
        for attempt in range(self.max_retries + 1):
            try:
                return func(*args, **kwargs)
            
            except Exception as e:
                if attempt == self.max_retries:
                    raise e
                
                # Calculate delay with optional jitter
                if self.jitter:
                    actual_delay = delay * (0.5 + random.random() * 0.5)
                else:
                    actual_delay = delay
                
                time.sleep(actual_delay)
                delay = min(delay * self.exponential_base, self.max_delay)

# Usage
backoff = ExponentialBackoff(initial_delay=1.0, max_retries=3)

def make_api_call():
    return client.get('/api/schemas')

try:
    result = backoff.execute(make_api_call)
except Exception as e:
    print(f"API call failed after retries: {e}")
```

## 🏢 Deployment Security

### Environment Separation

```yaml
# Environment-specific configurations
environments:
  development:
    server_url: "http://localhost:9001"
    security_profile: "lenient"
    debug_logging: true
    ssl_verify: false
    
  staging:
    server_url: "https://staging-api.datafold.com"
    security_profile: "standard"
    debug_logging: true
    ssl_verify: true
    certificate_pinning: false
    
  production:
    server_url: "https://api.datafold.com"
    security_profile: "strict"
    debug_logging: false
    ssl_verify: true
    certificate_pinning: true
    rate_limiting:
      requests_per_second: 100
      burst_size: 200
    monitoring:
      alert_on_auth_failures: true
      log_all_requests: true
```

### Container Security

```dockerfile
# Secure Docker container
FROM node:18-alpine AS base

# Security: Use non-root user
RUN addgroup -g 1001 -S datafold && \
    adduser -S datafold -u 1001 -G datafold

# Security: Install security updates
RUN apk update && apk upgrade && \
    apk add --no-cache dumb-init && \
    rm -rf /var/cache/apk/*

# Security: Create secure directories
RUN mkdir -p /app /app/keys && \
    chown -R datafold:datafold /app && \
    chmod 755 /app && \
    chmod 700 /app/keys

USER datafold
WORKDIR /app

# Install dependencies (no devDependencies in production)
COPY --chown=datafold:datafold package*.json ./
RUN npm ci --only=production && npm cache clean --force

# Copy application
COPY --chown=datafold:datafold . .

# Security: Don't run as PID 1
ENTRYPOINT ["dumb-init", "--"]
CMD ["node", "app.js"]

# Security: Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD node healthcheck.js

# Security: Read-only filesystem except for necessary directories
# (Configure in docker-compose or k8s)
```

```yaml
# Kubernetes security configuration
apiVersion: apps/v1
kind: Deployment
metadata:
  name: datafold-app
spec:
  template:
    spec:
      # Security Context
      securityContext:
        runAsNonRoot: true
        runAsUser: 1001
        runAsGroup: 1001
        fsGroup: 1001
        seccompProfile:
          type: RuntimeDefault
      
      containers:
      - name: app
        image: my-app:latest
        
        # Container Security Context
        securityContext:
          allowPrivilegeEscalation: false
          capabilities:
            drop:
            - ALL
          readOnlyRootFilesystem: true
          runAsNonRoot: true
          runAsUser: 1001
        
        # Resource Limits
        resources:
          limits:
            memory: "512Mi"
            cpu: "500m"
          requests:
            memory: "256Mi"
            cpu: "250m"
        
        # Environment Variables
        env:
        - name: NODE_ENV
          value: "production"
        
        # Secret Management
        envFrom:
        - secretRef:
            name: datafold-secrets
        
        # Volume Mounts
        volumeMounts:
        - name: tmp
          mountPath: /tmp
        - name: keys
          mountPath: /app/keys
          readOnly: true
        
        # Liveness and Readiness Probes
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        
        readinessProbe:
          httpGet:
            path: /ready
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
      
      volumes:
      - name: tmp
        emptyDir: {}
      - name: keys
        secret:
          secretName: datafold-keys
          defaultMode: 0400
```

## 📊 Monitoring and Alerting

### Security Monitoring

```python
# Comprehensive security monitoring
import logging
import time
from collections import defaultdict
from datetime import datetime, timedelta

class SecurityMonitor:
    def __init__(self):
        self.auth_failures = defaultdict(list)
        self.suspicious_patterns = defaultdict(int)
        self.alert_thresholds = {
            'auth_failures_per_hour': 10,
            'auth_failures_per_ip': 5,
            'nonce_reuse_attempts': 1,
            'timestamp_skew_violations': 3
        }
    
    def log_auth_failure(self, client_id, ip_address, reason, request_details):
        """Log authentication failure with context"""
        timestamp = datetime.utcnow()
        
        failure_event = {
            'timestamp': timestamp,
            'client_id': client_id,
            'ip_address': ip_address,
            'reason': reason,
            'user_agent': request_details.get('user_agent'),
            'endpoint': request_details.get('endpoint'),
            'method': request_details.get('method')
        }
        
        # Store for pattern analysis
        self.auth_failures[client_id].append(failure_event)
        
        # Clean old entries (keep last 24 hours)
        cutoff = timestamp - timedelta(hours=24)
        self.auth_failures[client_id] = [
            event for event in self.auth_failures[client_id]
            if event['timestamp'] > cutoff
        ]
        
        # Check for suspicious patterns
        self.analyze_patterns(client_id, ip_address, reason)
        
        # Log structured event
        logging.warning(
            "Authentication failure",
            extra={
                'security_event': 'auth_failure',
                'client_id': client_id,
                'ip_address': ip_address,
                'reason': reason,
                'timestamp': timestamp.isoformat(),
                **request_details
            }
        )
    
    def analyze_patterns(self, client_id, ip_address, reason):
        """Analyze for suspicious patterns and alert"""
        now = datetime.utcnow()
        hour_ago = now - timedelta(hours=1)
        
        # Count recent failures
        recent_failures = [
            event for event in self.auth_failures[client_id]
            if event['timestamp'] > hour_ago
        ]
        
        # Check thresholds
        if len(recent_failures) >= self.alert_thresholds['auth_failures_per_hour']:
            self.send_alert(
                'high_auth_failure_rate',
                f"Client {client_id} has {len(recent_failures)} auth failures in the last hour"
            )
        
        # Check IP-based patterns
        ip_failures = [
            event for event in recent_failures
            if event['ip_address'] == ip_address
        ]
        
        if len(ip_failures) >= self.alert_thresholds['auth_failures_per_ip']:
            self.send_alert(
                'ip_based_attack',
                f"IP {ip_address} has {len(ip_failures)} auth failures"
            )
        
        # Check for specific attack patterns
        if reason == 'nonce_reuse':
            self.send_alert(
                'replay_attack_detected',
                f"Replay attack detected from client {client_id} at IP {ip_address}"
            )
        
        if reason == 'timestamp_skew':
            self.suspicious_patterns['timestamp_skew'] += 1
            if self.suspicious_patterns['timestamp_skew'] >= self.alert_thresholds['timestamp_skew_violations']:
                self.send_alert(
                    'clock_skew_attack',
                    f"Multiple timestamp skew violations detected"
                )
    
    def send_alert(self, alert_type, message):
        """Send security alert to monitoring system"""
        alert = {
            'type': alert_type,
            'message': message,
            'timestamp': datetime.utcnow().isoformat(),
            'severity': 'high' if 'attack' in alert_type else 'medium'
        }
        
        # Log alert
        logging.error(
            f"Security Alert: {alert_type}",
            extra={'security_alert': alert}
        )
        
        # Send to external monitoring (implement based on your system)
        # self.send_to_slack(alert)
        # self.send_to_pagerduty(alert)
        # self.send_to_datadog(alert)

# Usage in request handler
security_monitor = SecurityMonitor()

def handle_auth_failure(client_id, ip_address, reason, request):
    request_details = {
        'user_agent': request.headers.get('User-Agent'),
        'endpoint': request.path,
        'method': request.method,
        'content_length': request.headers.get('Content-Length'),
        'referer': request.headers.get('Referer')
    }
    
    security_monitor.log_auth_failure(client_id, ip_address, reason, request_details)
```

### Metrics Collection

```python
# Prometheus metrics for security monitoring
from prometheus_client import Counter, Histogram, Gauge, start_http_server

# Security Metrics
auth_requests_total = Counter(
    'datafold_auth_requests_total',
    'Total authentication requests',
    ['client_id', 'status', 'reason']
)

auth_request_duration = Histogram(
    'datafold_auth_request_duration_seconds',
    'Authentication request duration',
    ['client_id']
)

signature_verification_duration = Histogram(
    'datafold_signature_verification_duration_seconds',
    'Signature verification duration'
)

active_nonces = Gauge(
    'datafold_active_nonces',
    'Number of active nonces in storage'
)

auth_failures_by_reason = Counter(
    'datafold_auth_failures_total',
    'Authentication failures by reason',
    ['reason', 'client_id']
)

# Usage in middleware
def auth_middleware(request):
    start_time = time.time()
    client_id = extract_client_id(request)
    
    try:
        # Perform authentication
        result = authenticate_request(request)
        
        # Record success metrics
        auth_requests_total.labels(
            client_id=client_id,
            status='success',
            reason='valid_signature'
        ).inc()
        
        return result
        
    except AuthenticationError as e:
        # Record failure metrics
        auth_requests_total.labels(
            client_id=client_id,
            status='failure',
            reason=e.reason
        ).inc()
        
        auth_failures_by_reason.labels(
            reason=e.reason,
            client_id=client_id
        ).inc()
        
        raise
        
    finally:
        # Record timing
        duration = time.time() - start_time
        auth_request_duration.labels(client_id=client_id).observe(duration)
```

## 🚨 Incident Response

### Security Incident Playbook

```yaml
# Security Incident Response Plan
incident_types:
  key_compromise:
    severity: critical
    response_time: 15_minutes
    actions:
      - immediate_key_revocation
      - audit_recent_requests
      - notify_security_team
      - generate_new_keys
      - update_monitoring_rules
  
  replay_attack:
    severity: high
    response_time: 30_minutes
    actions:
      - block_suspicious_ips
      - increase_nonce_monitoring
      - audit_affected_requests
      - notify_operations_team
  
  mass_auth_failures:
    severity: medium
    response_time: 1_hour
    actions:
      - analyze_failure_patterns
      - check_system_health
      - verify_time_synchronization
      - escalate_if_persistent

automated_responses:
  - trigger: auth_failure_rate > 100/hour
    action: temporary_rate_limiting
    duration: 30_minutes
  
  - trigger: nonce_reuse_detected
    action: immediate_client_blocking
    duration: 1_hour
  
  - trigger: timestamp_skew > 10_minutes
    action: alert_operations_team
```

### Key Revocation Process

```python
# Emergency key revocation
class EmergencyKeyRevocation:
    def __init__(self, client, monitoring_system):
        self.client = client
        self.monitoring = monitoring_system
    
    def revoke_key_immediately(self, client_id, reason):
        """Immediately revoke a key and block all requests"""
        try:
            # 1. Add to revocation list
            self.add_to_revocation_list(client_id)
            
            # 2. Revoke on server
            response = self.client.delete(f'/api/crypto/keys/revoke/{client_id}', json={
                'reason': reason,
                'emergency': True,
                'revoked_by': 'security_system'
            })
            
            # 3. Clear any cached authentication
            self.clear_auth_cache(client_id)
            
            # 4. Alert monitoring systems
            self.monitoring.send_alert('key_revoked', {
                'client_id': client_id,
                'reason': reason,
                'timestamp': datetime.utcnow().isoformat()
            })
            
            # 5. Log security event
            logging.critical(
                f"Emergency key revocation for client {client_id}",
                extra={
                    'security_event': 'key_revocation',
                    'client_id': client_id,
                    'reason': reason,
                    'emergency': True
                }
            )
            
            return True
            
        except Exception as e:
            logging.error(f"Failed to revoke key for {client_id}: {e}")
            return False
    
    def audit_recent_requests(self, client_id, hours=24):
        """Audit recent requests from potentially compromised client"""
        cutoff = datetime.utcnow() - timedelta(hours=hours)
        
        # Query request logs
        suspicious_requests = self.find_requests_since(client_id, cutoff)
        
        # Analyze for indicators of compromise
        iocs = self.analyze_for_compromise(suspicious_requests)
        
        # Generate report
        report = {
            'client_id': client_id,
            'audit_period': f'{hours} hours',
            'total_requests': len(suspicious_requests),
            'suspicious_patterns': iocs,
            'recommended_actions': self.generate_recommendations(iocs)
        }
        
        return report
```

## 📋 Security Checklist

### Pre-Production Checklist

- [ ] **Cryptographic Security**
  - [ ] Ed25519 keys generated with cryptographically secure randomness
  - [ ] Private keys stored with proper permissions (600)
  - [ ] No hardcoded keys in source code
  - [ ] Key rotation plan implemented
  - [ ] Backup and recovery procedures tested

- [ ] **Authentication Security**
  - [ ] Signature verification implementation audited
  - [ ] Nonce storage prevents replay attacks
  - [ ] Timestamp validation configured appropriately
  - [ ] Required signature components enforced
  - [ ] Error messages don't leak sensitive information

- [ ] **Network Security**
  - [ ] TLS 1.2+ enforced for all communications
  - [ ] Certificate validation enabled
  - [ ] Certificate pinning implemented (if applicable)
  - [ ] Network timeouts configured
  - [ ] Rate limiting implemented

- [ ] **Application Security**
  - [ ] Input validation on all parameters
  - [ ] SQL injection protection
  - [ ] XSS protection (if applicable)
  - [ ] CSRF protection (if applicable)
  - [ ] Secure headers configured

- [ ] **Infrastructure Security**
  - [ ] Containers run as non-root user
  - [ ] Secrets managed securely (Kubernetes secrets, etc.)
  - [ ] Monitoring and alerting configured
  - [ ] Log aggregation setup
  - [ ] Incident response plan documented

- [ ] **Compliance and Audit**
  - [ ] Security audit completed
  - [ ] Penetration testing performed
  - [ ] Compliance requirements met (SOC2, PCI DSS, etc.)
  - [ ] Documentation reviewed and approved
  - [ ] Security training completed

### Post-Deployment Monitoring

- [ ] **Operational Security**
  - [ ] Authentication metrics monitored
  - [ ] Failed authentication alerts configured
  - [ ] Performance monitoring setup
  - [ ] Log retention policies implemented
  - [ ] Regular security scans scheduled

- [ ] **Maintenance**
  - [ ] Dependencies updated regularly
  - [ ] Security patches applied promptly
  - [ ] Key rotation performed on schedule
  - [ ] Backup integrity verified
  - [ ] Incident response procedures tested

## 🔗 Related Documentation

- **[Authentication Overview](../authentication/overview.md)** - How signature authentication works
- **[Getting Started](../authentication/getting-started.md)** - Implementation guide
- **[Troubleshooting](../authentication/troubleshooting.md)** - Common issues and solutions
- **[Performance Optimization](performance-optimization.md)** - Performance best practices

## 📞 Security Support

For security-related questions or to report security issues:

- **Security Issues**: [security@datafold.com](mailto:security@datafold.com)
- **Security Documentation**: [Security Center](https://security.datafold.com)
- **Emergency Contact**: [+1-555-DATAFOLD](tel:+15553283653)
- **Bug Bounty**: [HackerOne Program](https://hackerone.com/datafold)

---

**Remember**: Security is a shared responsibility. Follow these guidelines, stay updated with security best practices, and always prioritize security in your implementation decisions.