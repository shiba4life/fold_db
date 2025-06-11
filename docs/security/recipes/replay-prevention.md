# Replay Attack Prevention Security Recipe

## Overview

This recipe provides comprehensive strategies and implementations for preventing replay attacks in DataFold signature authentication. Replay attacks occur when attackers intercept valid signed requests and replay them to perform unauthorized actions.

## Security Level
- **Complexity**: Intermediate
- **Security Rating**: Critical
- **Implementation Time**: 45-90 minutes

## Prerequisites

### Technical Understanding
- HTTP Message Signatures (RFC 9421)
- Cryptographic nonces and timestamps
- Attack vectors and threat modeling
- Caching and storage mechanisms

### Infrastructure Requirements
- Distributed cache (Redis recommended)
- Time synchronization (NTP)
- Secure random number generation
- High-availability storage for nonce tracking

### Dependencies
- DataFold SDK with replay prevention features
- Distributed cache client library
- Time synchronization service
- Monitoring and alerting system

## Threat Analysis

### Attack Scenarios

#### Basic Replay Attack
```
1. Attacker intercepts valid signed request
2. Attacker replays request immediately or later
3. Server processes replayed request as legitimate
4. Unauthorized action is performed
```

#### Delayed Replay Attack
```
1. Attacker intercepts and stores signed request
2. Attacker waits for optimal timing
3. Attacker replays request during high-traffic period
4. Request blends in with legitimate traffic
```

#### Distributed Replay Attack
```
1. Attacker intercepts signed request
2. Attacker replays request from multiple IP addresses
3. Load balancers distribute to different servers
4. Lack of global nonce tracking allows success
```

### Impact Assessment
- **Financial**: Unauthorized transactions, duplicate payments
- **Data**: Unauthorized data access or modification
- **Operational**: Service disruption, resource exhaustion
- **Compliance**: Audit failures, regulatory violations

## Implementation Strategies

### Strategy 1: Timestamp-Based Windows

#### Basic Timestamp Validation
```javascript
// server/middleware/timestamp-validation.js
const MAX_REQUEST_AGE = 300; // 5 minutes

class TimestampValidator {
  constructor(config = {}) {
    this.maxAge = config.maxAge || MAX_REQUEST_AGE;
    this.clockSkewTolerance = config.clockSkewTolerance || 60; // 1 minute
  }
  
  validateTimestamp(timestamp, receivedAt = null) {
    const now = receivedAt || Math.floor(Date.now() / 1000);
    const requestTime = parseInt(timestamp);
    
    // Check if timestamp is valid number
    if (isNaN(requestTime) || requestTime <= 0) {
      return {
        valid: false,
        reason: 'invalid_timestamp_format',
        message: 'Timestamp must be a valid Unix timestamp'
      };
    }
    
    // Check if timestamp is too far in the future (clock skew)
    const futureLimit = now + this.clockSkewTolerance;
    if (requestTime > futureLimit) {
      return {
        valid: false,
        reason: 'timestamp_too_future',
        message: 'Request timestamp is too far in the future',
        details: {
          requestTime,
          now,
          allowedSkew: this.clockSkewTolerance
        }
      };
    }
    
    // Check if timestamp is too old
    const pastLimit = now - this.maxAge;
    if (requestTime < pastLimit) {
      return {
        valid: false,
        reason: 'timestamp_too_old',
        message: 'Request timestamp is too old',
        details: {
          requestTime,
          now,
          maxAge: this.maxAge,
          age: now - requestTime
        }
      };
    }
    
    return {
      valid: true,
      age: now - requestTime,
      skew: requestTime - now
    };
  }
}

module.exports = { TimestampValidator };
```

#### Advanced Timestamp Validation with Drift Detection
```javascript
// server/middleware/advanced-timestamp-validation.js
class AdvancedTimestampValidator extends TimestampValidator {
  constructor(config = {}) {
    super(config);
    this.clockDriftThreshold = config.clockDriftThreshold || 30; // seconds
    this.clientClockSkews = new Map(); // Track per-client clock skew
  }
  
  validateWithDriftDetection(timestamp, clientId, receivedAt = null) {
    const basicValidation = this.validateTimestamp(timestamp, receivedAt);
    
    if (!basicValidation.valid) {
      return basicValidation;
    }
    
    // Track clock skew per client
    const skew = basicValidation.skew;
    this.updateClientClockSkew(clientId, skew);
    
    // Check for significant clock drift
    const avgSkew = this.getAverageClockSkew(clientId);
    if (Math.abs(skew - avgSkew) > this.clockDriftThreshold) {
      console.warn(`Clock drift detected for client ${clientId}`, {
        currentSkew: skew,
        averageSkew: avgSkew,
        drift: Math.abs(skew - avgSkew)
      });
      
      // Optional: Reject requests with excessive drift
      if (Math.abs(skew - avgSkew) > this.clockDriftThreshold * 2) {
        return {
          valid: false,
          reason: 'excessive_clock_drift',
          message: 'Client clock drift exceeds acceptable limits'
        };
      }
    }
    
    return basicValidation;
  }
  
  updateClientClockSkew(clientId, skew) {
    const history = this.clientClockSkews.get(clientId) || [];
    history.push({ skew, timestamp: Date.now() });
    
    // Keep only recent measurements (last 100 or 1 hour)
    const oneHourAgo = Date.now() - 3600000;
    const recentHistory = history
      .filter(entry => entry.timestamp > oneHourAgo)
      .slice(-100);
    
    this.clientClockSkews.set(clientId, recentHistory);
  }
  
  getAverageClockSkew(clientId) {
    const history = this.clientClockSkews.get(clientId) || [];
    if (history.length === 0) return 0;
    
    const sum = history.reduce((acc, entry) => acc + entry.skew, 0);
    return sum / history.length;
  }
}
```

### Strategy 2: Nonce-Based Tracking

#### Basic Nonce Validation
```javascript
// server/middleware/nonce-validation.js
const Redis = require('redis');

class NonceValidator {
  constructor(config = {}) {
    this.redis = Redis.createClient(config.redis);
    this.nonceExpiry = config.nonceExpiry || 600; // 10 minutes
    this.keyPrefix = config.keyPrefix || 'datafold:nonce:';
  }
  
  async validateNonce(nonce, timestamp) {
    // Validate nonce format
    if (!nonce || typeof nonce !== 'string' || nonce.length < 16) {
      return {
        valid: false,
        reason: 'invalid_nonce_format',
        message: 'Nonce must be at least 16 characters'
      };
    }
    
    // Check if nonce uses cryptographically secure random
    if (!this.isSecureNonce(nonce)) {
      return {
        valid: false,
        reason: 'weak_nonce',
        message: 'Nonce does not appear to be cryptographically secure'
      };
    }
    
    const nonceKey = `${this.keyPrefix}${nonce}`;
    
    try {
      // Check if nonce has been used before
      const exists = await this.redis.exists(nonceKey);
      
      if (exists) {
        // Log potential replay attack
        console.warn('Replay attack detected - nonce reuse:', {
          nonce: nonce.substring(0, 8) + '...',
          timestamp,
          key: nonceKey
        });
        
        return {
          valid: false,
          reason: 'nonce_already_used',
          message: 'Nonce has already been used'
        };
      }
      
      // Store nonce with expiration
      await this.redis.setex(nonceKey, this.nonceExpiry, timestamp);
      
      return {
        valid: true,
        stored: true
      };
      
    } catch (error) {
      console.error('Nonce validation error:', error);
      
      // Fail securely - reject request if we can't validate nonce
      return {
        valid: false,
        reason: 'nonce_validation_error',
        message: 'Unable to validate nonce'
      };
    }
  }
  
  isSecureNonce(nonce) {
    // Basic entropy check - should have good character distribution
    const chars = new Set(nonce);
    const entropy = chars.size / nonce.length;
    
    // Should have at least 50% unique characters for good randomness
    return entropy >= 0.5;
  }
  
  async cleanupExpiredNonces() {
    // Optional: Implement cleanup of expired nonces
    // This is usually handled by Redis TTL, but can be done manually
    const pattern = `${this.keyPrefix}*`;
    const keys = await this.redis.keys(pattern);
    
    let cleaned = 0;
    for (const key of keys) {
      const ttl = await this.redis.ttl(key);
      if (ttl <= 0) {
        await this.redis.del(key);
        cleaned++;
      }
    }
    
    console.log(`Cleaned up ${cleaned} expired nonces`);
    return cleaned;
  }
}

module.exports = { NonceValidator };
```

#### Advanced Nonce Validation with Pattern Detection
```javascript
// server/middleware/advanced-nonce-validation.js
class AdvancedNonceValidator extends NonceValidator {
  constructor(config = {}) {
    super(config);
    this.patternDetection = config.patternDetection || true;
    this.noncePatterns = new Map(); // Track nonce patterns per client
    this.suspiciousPatterns = config.suspiciousPatterns || [
      /^[0-9]+$/, // All numbers
      /^[a-f0-9]+$/, // Hex only
      /(.)\1{4,}/, // Repeated characters
      /^(..)\1+$/, // Repeated pairs
    ];
  }
  
  async validateNonceWithPatternDetection(nonce, clientId, timestamp) {
    const basicValidation = await this.validateNonce(nonce, timestamp);
    
    if (!basicValidation.valid) {
      return basicValidation;
    }
    
    if (this.patternDetection) {
      const patternCheck = this.checkNoncePatterns(nonce, clientId);
      if (!patternCheck.valid) {
        return patternCheck;
      }
    }
    
    return basicValidation;
  }
  
  checkNoncePatterns(nonce, clientId) {
    // Check for suspicious patterns
    for (const pattern of this.suspiciousPatterns) {
      if (pattern.test(nonce)) {
        console.warn(`Suspicious nonce pattern detected for client ${clientId}`, {
          pattern: pattern.toString(),
          nonce: nonce.substring(0, 8) + '...'
        });
        
        return {
          valid: false,
          reason: 'suspicious_nonce_pattern',
          message: 'Nonce contains suspicious patterns'
        };
      }
    }
    
    // Track nonce patterns per client
    this.trackNoncePattern(nonce, clientId);
    
    return { valid: true };
  }
  
  trackNoncePattern(nonce, clientId) {
    // Simple pattern tracking - first and last 4 characters
    const pattern = nonce.substring(0, 4) + '...' + nonce.substring(nonce.length - 4);
    
    const clientPatterns = this.noncePatterns.get(clientId) || new Map();
    const count = clientPatterns.get(pattern) || 0;
    clientPatterns.set(pattern, count + 1);
    
    // Alert if same pattern used too frequently
    if (count > 5) {
      console.warn(`Repeated nonce pattern for client ${clientId}`, {
        pattern,
        count: count + 1
      });
    }
    
    this.noncePatterns.set(clientId, clientPatterns);
  }
}
```

### Strategy 3: Combined Validation Middleware

#### Express.js Replay Prevention Middleware
```javascript
// server/middleware/replay-prevention.js
const { AdvancedTimestampValidator } = require('./advanced-timestamp-validation');
const { AdvancedNonceValidator } = require('./advanced-nonce-validation');

class ReplayPreventionMiddleware {
  constructor(config = {}) {
    this.timestampValidator = new AdvancedTimestampValidator(config.timestamp);
    this.nonceValidator = new AdvancedNonceValidator(config.nonce);
    this.enabled = config.enabled !== false;
    this.strictMode = config.strictMode || false;
  }
  
  middleware() {
    return async (req, res, next) => {
      if (!this.enabled) {
        return next();
      }
      
      try {
        const replayCheck = await this.validateRequest(req);
        
        if (!replayCheck.valid) {
          // Log security event
          console.warn('Replay prevention blocked request:', {
            ip: req.ip,
            userAgent: req.get('User-Agent'),
            url: req.originalUrl,
            reason: replayCheck.reason,
            details: replayCheck.details
          });
          
          // Return error response
          return res.status(401).json({
            error: 'Request validation failed',
            message: this.strictMode ? replayCheck.message : 'Authentication failed'
          });
        }
        
        // Store validation results for audit
        req.replayValidation = replayCheck;
        
        next();
      } catch (error) {
        console.error('Replay prevention middleware error:', error);
        
        // Fail securely
        res.status(500).json({
          error: 'Authentication validation failed',
          message: 'Unable to validate request authenticity'
        });
      }
    };
  }
  
  async validateRequest(req) {
    // Extract signature components
    const signatureInput = req.get('signature-input');
    const timestamp = this.extractTimestamp(signatureInput);
    const nonce = this.extractNonce(signatureInput);
    const clientId = this.extractClientId(req);
    
    if (!timestamp || !nonce) {
      return {
        valid: false,
        reason: 'missing_replay_prevention_data',
        message: 'Request missing timestamp or nonce'
      };
    }
    
    // Validate timestamp
    const timestampValidation = this.timestampValidator.validateWithDriftDetection(
      timestamp, 
      clientId, 
      Math.floor(Date.now() / 1000)
    );
    
    if (!timestampValidation.valid) {
      return timestampValidation;
    }
    
    // Validate nonce
    const nonceValidation = await this.nonceValidator.validateNonceWithPatternDetection(
      nonce, 
      clientId, 
      timestamp
    );
    
    if (!nonceValidation.valid) {
      return nonceValidation;
    }
    
    return {
      valid: true,
      timestamp: timestampValidation,
      nonce: nonceValidation,
      clientId
    };
  }
  
  extractTimestamp(signatureInput) {
    const match = signatureInput?.match(/created=(\d+)/);
    return match ? match[1] : null;
  }
  
  extractNonce(signatureInput) {
    const match = signatureInput?.match(/nonce="([^"]+)"/);
    return match ? match[1] : null;
  }
  
  extractClientId(req) {
    // Extract from signature or headers
    return req.get('x-client-id') || 
           req.authenticatedKeyId || 
           req.ip; // Fallback to IP
  }
}

// Usage
const app = express();

const replayPrevention = new ReplayPreventionMiddleware({
  enabled: true,
  strictMode: process.env.NODE_ENV === 'production',
  timestamp: {
    maxAge: 300, // 5 minutes
    clockSkewTolerance: 60, // 1 minute
    clockDriftThreshold: 30 // 30 seconds
  },
  nonce: {
    nonceExpiry: 600, // 10 minutes
    patternDetection: true,
    redis: {
      host: process.env.REDIS_HOST || 'localhost',
      port: process.env.REDIS_PORT || 6379
    }
  }
});

app.use('/api', replayPrevention.middleware());
```

### Strategy 4: Client-Side Implementation

#### JavaScript Client with Replay Prevention
```javascript
// client/replay-prevention-client.js
const crypto = require('crypto');

class ReplayPreventionClient {
  constructor(config = {}) {
    this.nonceLength = config.nonceLength || 32;
    this.timestampPrecision = config.timestampPrecision || 'seconds'; // 'seconds' or 'milliseconds'
    this.clockSyncEnabled = config.clockSyncEnabled || true;
    this.clockOffset = 0; // Will be calculated if clock sync enabled
  }
  
  async initialize() {
    if (this.clockSyncEnabled) {
      await this.synchronizeClock();
    }
  }
  
  generateSecureNonce() {
    // Generate cryptographically secure random nonce
    const randomBytes = crypto.randomBytes(this.nonceLength);
    
    // Use base64url encoding for URL-safe nonce
    return randomBytes
      .toString('base64')
      .replace(/\+/g, '-')
      .replace(/\//g, '_')
      .replace(/=/g, '');
  }
  
  getCurrentTimestamp() {
    const now = Date.now();
    const adjustedTime = now + this.clockOffset;
    
    return this.timestampPrecision === 'seconds' 
      ? Math.floor(adjustedTime / 1000)
      : adjustedTime;
  }
  
  async synchronizeClock() {
    try {
      // Simple clock synchronization with server
      const start = Date.now();
      const response = await fetch('/api/time');
      const end = Date.now();
      
      const serverTime = (await response.json()).timestamp * 1000; // Convert to ms
      const roundTripTime = end - start;
      const networkDelay = roundTripTime / 2;
      
      const serverTimeAdjusted = serverTime + networkDelay;
      this.clockOffset = serverTimeAdjusted - end;
      
      console.log('Clock synchronized:', {
        offset: this.clockOffset,
        roundTripTime,
        serverTime
      });
      
    } catch (error) {
      console.warn('Clock synchronization failed:', error.message);
      this.clockOffset = 0;
    }
  }
  
  createReplayPreventionData() {
    return {
      timestamp: this.getCurrentTimestamp(),
      nonce: this.generateSecureNonce()
    };
  }
  
  // Validate our own nonces to prevent issues
  validateNonce(nonce) {
    if (!nonce || typeof nonce !== 'string') {
      return { valid: false, reason: 'Invalid nonce format' };
    }
    
    if (nonce.length < 16) {
      return { valid: false, reason: 'Nonce too short' };
    }
    
    // Check for obvious patterns
    if (/^(.)\1+$/.test(nonce)) {
      return { valid: false, reason: 'Nonce contains repeated characters' };
    }
    
    return { valid: true };
  }
}

module.exports = { ReplayPreventionClient };
```

#### Python Client Implementation
```python
import time
import secrets
import base64
import asyncio
import aiohttp
from typing import Dict, Optional

class ReplayPreventionClient:
    def __init__(self, config: Dict = None):
        config = config or {}
        self.nonce_length = config.get('nonce_length', 32)
        self.timestamp_precision = config.get('timestamp_precision', 'seconds')
        self.clock_sync_enabled = config.get('clock_sync_enabled', True)
        self.clock_offset = 0.0
        
    async def initialize(self):
        if self.clock_sync_enabled:
            await self.synchronize_clock()
    
    def generate_secure_nonce(self) -> str:
        """Generate cryptographically secure random nonce"""
        random_bytes = secrets.token_bytes(self.nonce_length)
        
        # Use base64url encoding for URL-safe nonce
        return base64.urlsafe_b64encode(random_bytes).decode('ascii').rstrip('=')
    
    def get_current_timestamp(self) -> int:
        """Get current timestamp adjusted for clock offset"""
        now = time.time()
        adjusted_time = now + self.clock_offset
        
        return int(adjusted_time) if self.timestamp_precision == 'seconds' else int(adjusted_time * 1000)
    
    async def synchronize_clock(self):
        """Synchronize clock with server"""
        try:
            start = time.time()
            
            async with aiohttp.ClientSession() as session:
                async with session.get('/api/time') as response:
                    end = time.time()
                    data = await response.json()
                    
            server_time = data['timestamp']
            round_trip_time = end - start
            network_delay = round_trip_time / 2
            
            server_time_adjusted = server_time + network_delay
            self.clock_offset = server_time_adjusted - end
            
            print(f'Clock synchronized: offset={self.clock_offset:.3f}s, rtt={round_trip_time:.3f}s')
            
        except Exception as error:
            print(f'Clock synchronization failed: {error}')
            self.clock_offset = 0.0
    
    def create_replay_prevention_data(self) -> Dict[str, str]:
        """Create timestamp and nonce for replay prevention"""
        return {
            'timestamp': str(self.get_current_timestamp()),
            'nonce': self.generate_secure_nonce()
        }
    
    def validate_nonce(self, nonce: str) -> Dict[str, any]:
        """Validate nonce format and security"""
        if not nonce or not isinstance(nonce, str):
            return {'valid': False, 'reason': 'Invalid nonce format'}
        
        if len(nonce) < 16:
            return {'valid': False, 'reason': 'Nonce too short'}
        
        # Check for obvious patterns
        if len(set(nonce)) < len(nonce) * 0.5:
            return {'valid': False, 'reason': 'Nonce has insufficient entropy'}
        
        return {'valid': True}
```

## Monitoring and Alerting

### Security Event Detection

```javascript
// monitoring/replay-attack-detection.js
class ReplayAttackMonitor {
  constructor(config = {}) {
    this.alertThresholds = {
      replayAttempts: config.replayAttempts || 5, // per minute
      timestampViolations: config.timestampViolations || 10,
      noncePatternViolations: config.noncePatternViolations || 3
    };
    
    this.metrics = {
      replayAttempts: 0,
      timestampViolations: 0,
      nonceViolations: 0,
      lastReset: Date.now()
    };
  }
  
  recordReplayAttempt(clientId, details) {
    this.metrics.replayAttempts++;
    
    // Log security event
    console.warn('SECURITY: Replay attempt detected', {
      clientId,
      timestamp: new Date().toISOString(),
      details,
      totalAttempts: this.metrics.replayAttempts
    });
    
    // Check alert thresholds
    this.checkAlertThresholds();
  }
  
  recordTimestampViolation(clientId, violation) {
    this.metrics.timestampViolations++;
    
    console.warn('SECURITY: Timestamp violation', {
      clientId,
      violation,
      totalViolations: this.metrics.timestampViolations
    });
    
    this.checkAlertThresholds();
  }
  
  checkAlertThresholds() {
    const now = Date.now();
    const timeSinceReset = now - this.metrics.lastReset;
    
    // Reset metrics every minute
    if (timeSinceReset > 60000) {
      this.resetMetrics();
      return;
    }
    
    // Check thresholds and alert
    if (this.metrics.replayAttempts >= this.alertThresholds.replayAttempts) {
      this.sendSecurityAlert('HIGH_REPLAY_ATTACK_VOLUME', {
        attempts: this.metrics.replayAttempts,
        timeWindow: timeSinceReset
      });
    }
  }
  
  sendSecurityAlert(type, data) {
    // Integrate with your alerting system
    console.error(`SECURITY ALERT: ${type}`, data);
    
    // Example: Send to security team
    // alertingService.notify('security-team', { type, data, severity: 'HIGH' });
  }
  
  resetMetrics() {
    this.metrics = {
      replayAttempts: 0,
      timestampViolations: 0,
      nonceViolations: 0,
      lastReset: Date.now()
    };
  }
}
```

### Performance Monitoring

```javascript
// monitoring/performance-monitor.js
class ReplayPreventionPerformanceMonitor {
  constructor() {
    this.metrics = {
      timestampValidationTime: [],
      nonceValidationTime: [],
      redisOperationTime: [],
      totalValidationTime: []
    };
  }
  
  startTimer() {
    return process.hrtime.bigint();
  }
  
  endTimer(startTime) {
    const endTime = process.hrtime.bigint();
    return Number(endTime - startTime) / 1000000; // Convert to milliseconds
  }
  
  recordTimestampValidation(duration) {
    this.metrics.timestampValidationTime.push(duration);
    this.trimMetrics('timestampValidationTime');
  }
  
  recordNonceValidation(duration) {
    this.metrics.nonceValidationTime.push(duration);
    this.trimMetrics('nonceValidationTime');
  }
  
  trimMetrics(metricName, maxSamples = 1000) {
    if (this.metrics[metricName].length > maxSamples) {
      this.metrics[metricName] = this.metrics[metricName].slice(-maxSamples);
    }
  }
  
  getPerformanceStats() {
    return {
      timestampValidation: this.calculateStats('timestampValidationTime'),
      nonceValidation: this.calculateStats('nonceValidationTime'),
      redisOperations: this.calculateStats('redisOperationTime'),
      totalValidation: this.calculateStats('totalValidationTime')
    };
  }
  
  calculateStats(metricName) {
    const values = this.metrics[metricName];
    if (values.length === 0) return null;
    
    const sorted = [...values].sort((a, b) => a - b);
    const sum = values.reduce((a, b) => a + b, 0);
    
    return {
      count: values.length,
      average: sum / values.length,
      median: sorted[Math.floor(sorted.length / 2)],
      p95: sorted[Math.floor(sorted.length * 0.95)],
      p99: sorted[Math.floor(sorted.length * 0.99)],
      min: sorted[0],
      max: sorted[sorted.length - 1]
    };
  }
}
```

## Testing and Validation

### Replay Attack Simulation Tests

```javascript
// tests/security/replay-attack.test.js
const request = require('supertest');
const app = require('../../server/app');

describe('Replay Attack Prevention', () => {
  describe('Timestamp validation', () => {
    it('should reject requests with old timestamps', async () => {
      const oldTimestamp = Math.floor(Date.now() / 1000) - 600; // 10 minutes ago
      
      const response = await request(app)
        .post('/api/data')
        .set('signature-input', `sig1=("@method" "@target-uri");created=${oldTimestamp};nonce="test123456789012345"`)
        .set('signature', 'mock-signature')
        .expect(401);
      
      expect(response.body.error).toContain('validation failed');
    });
    
    it('should reject requests with future timestamps', async () => {
      const futureTimestamp = Math.floor(Date.now() / 1000) + 300; // 5 minutes future
      
      const response = await request(app)
        .post('/api/data')
        .set('signature-input', `sig1=("@method" "@target-uri");created=${futureTimestamp};nonce="test123456789012345"`)
        .set('signature', 'mock-signature')
        .expect(401);
      
      expect(response.body.error).toContain('validation failed');
    });
  });
  
  describe('Nonce validation', () => {
    it('should reject requests with reused nonces', async () => {
      const timestamp = Math.floor(Date.now() / 1000);
      const nonce = 'test-nonce-12345678901234567890';
      
      const signatureInput = `sig1=("@method" "@target-uri");created=${timestamp};nonce="${nonce}"`;
      
      // First request should succeed
      await request(app)
        .post('/api/data')
        .set('signature-input', signatureInput)
        .set('signature', 'mock-signature')
        .expect(200);
      
      // Second request with same nonce should fail
      const response = await request(app)
        .post('/api/data')
        .set('signature-input', signatureInput)
        .set('signature', 'mock-signature')
        .expect(401);
      
      expect(response.body.error).toContain('validation failed');
    });
    
    it('should reject requests with weak nonces', async () => {
      const timestamp = Math.floor(Date.now() / 1000);
      const weakNonce = '1111111111111111'; // Weak nonce
      
      const response = await request(app)
        .post('/api/data')
        .set('signature-input', `sig1=("@method" "@target-uri");created=${timestamp};nonce="${weakNonce}"`)
        .set('signature', 'mock-signature')
        .expect(401);
      
      expect(response.body.error).toContain('validation failed');
    });
  });
  
  describe('Performance', () => {
    it('should validate replay prevention within performance targets', async () => {
      const timestamp = Math.floor(Date.now() / 1000);
      const nonce = crypto.randomBytes(32).toString('base64url');
      
      const start = Date.now();
      
      await request(app)
        .post('/api/data')
        .set('signature-input', `sig1=("@method" "@target-uri");created=${timestamp};nonce="${nonce}"`)
        .set('signature', 'mock-signature')
        .expect(200);
      
      const duration = Date.now() - start;
      
      // Should complete within 100ms
      expect(duration).toBeLessThan(100);
    });
  });
});
```

## Production Deployment

### Configuration Template
```yaml
# config/replay-prevention.yaml
replay_prevention:
  enabled: true
  strict_mode: true
  
  timestamp:
    max_age_seconds: 300          # 5 minutes
    clock_skew_tolerance: 60      # 1 minute
    clock_drift_threshold: 30     # 30 seconds
    
  nonce:
    expiry_seconds: 600           # 10 minutes
    pattern_detection: true
    min_length: 16
    redis:
      host: "redis.internal"
      port: 6379
      db: 0
      password: "${REDIS_PASSWORD}"
      
  monitoring:
    alert_thresholds:
      replay_attempts_per_minute: 5
      timestamp_violations_per_minute: 10
      nonce_violations_per_minute: 3
    
  performance:
    max_validation_time_ms: 50
    cache_validation_results: true
    cache_ttl_seconds: 60
```

### Health Checks
```javascript
// health/replay-prevention-health.js
app.get('/health/replay-prevention', async (req, res) => {
  const health = {
    status: 'healthy',
    timestamp: new Date().toISOString(),
    checks: {}
  };
  
  try {
    // Check Redis connectivity
    const redisStart = Date.now();
    await redis.ping();
    health.checks.redis = {
      status: 'healthy',
      responseTime: Date.now() - redisStart
    };
  } catch (error) {
    health.status = 'unhealthy';
    health.checks.redis = {
      status: 'unhealthy',
      error: error.message
    };
  }
  
  // Check performance metrics
  const perfStats = performanceMonitor.getPerformanceStats();
  if (perfStats.totalValidation?.p95 > 100) {
    health.status = 'degraded';
    health.checks.performance = {
      status: 'degraded',
      p95ValidationTime: perfStats.totalValidation.p95
    };
  } else {
    health.checks.performance = {
      status: 'healthy',
      p95ValidationTime: perfStats.totalValidation?.p95 || 0
    };
  }
  
  res.status(health.status === 'healthy' ? 200 : 503).json(health);
});
```

## Advanced Patterns

### Distributed Nonce Tracking
For multi-server deployments, ensure nonce tracking works across all instances:

```javascript
// Use Redis Cluster or distributed cache
const redisCluster = new Redis.Cluster([
  { host: 'redis-node-1', port: 6379 },
  { host: 'redis-node-2', port: 6379 },
  { host: 'redis-node-3', port: 6379 }
]);
```

### Adaptive Time Windows
Adjust time windows based on client behavior:

```javascript
class AdaptiveTimestampValidator {
  adjustTimeWindowForClient(clientId, successRate) {
    if (successRate > 0.99) {
      // High success rate - allow longer window
      return this.maxAge * 1.5;
    } else if (successRate < 0.9) {
      // Low success rate - reduce window
      return this.maxAge * 0.5;
    }
    return this.maxAge;
  }
}
```

## Next Steps

After implementing replay prevention:

1. **[Performance Optimization](performance-optimization.md)** - Optimize validation performance
2. **[Monitoring Setup](../quick-start/monitoring-setup.md)** - Set up comprehensive monitoring
3. **[Incident Response](incident-response.md)** - Prepare for security incidents
4. **[Key Rotation](key-rotation.md)** - Implement key rotation strategies

## References

- [RFC 9421 HTTP Message Signatures](https://datatracker.ietf.org/doc/rfc9421/)
- [OWASP Replay Attack Prevention](https://owasp.org/www-community/attacks/Replay_attack)
- [NIST Cryptographic Standards](https://csrc.nist.gov/projects/cryptographic-standards-and-guidelines)
- [Redis Security Best Practices](https://redis.io/topics/security)