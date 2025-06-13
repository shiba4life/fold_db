# Key Rotation Guide

This guide covers DataFold's key rotation and replacement capabilities, focusing on secure cryptographic key lifecycle management.

## Overview

DataFold implements client-initiated key rotation with server-side coordination, providing secure cryptographic key lifecycle management without service interruption. The system currently supports:

- ✅ **Atomic key replacement** - Single transaction updates across all data associations
- ✅ **Cryptographic proof of ownership** - Old private key must sign rotation requests
- ✅ **Comprehensive audit logging** - Full trail of all key lifecycle events
- ✅ **Multiple rotation reasons** - Scheduled, compromise, policy, user-initiated
- ✅ **Security policies** - Rate limiting, IP restrictions, and risk assessment
- ✅ **Emergency procedures** - Admin override with enhanced logging

## Current Implementation Status

### ✅ Production Ready Features

#### Atomic Key Replacement
```rust
use datafold::crypto::key_rotation::{KeyRotationRequest, RotationReason};

// Generate new key pair
let new_keypair = generate_master_keypair()?;

// Create signed rotation request
let request = KeyRotationRequest::new(
    &old_private_key,
    new_keypair.public_key().clone(),
    RotationReason::UserInitiated,
    Some("client-app".to_string()),
    HashMap::new(),
)?;

// Submit to server (requires signature authentication)
let response = datafold_client.rotate_key(request).await?;
```

#### Cryptographic Security
- **Ed25519 signatures** required for all rotation requests
- **Timestamp validation** prevents replay attacks
- **Nonce verification** ensures request uniqueness
- **Key ownership proof** via signature with old private key

#### Audit and Compliance
- Complete audit trail for all operations
- Security metadata tracking (IP, user agent, session info)
- Configurable audit log retention and export
- Compliance reporting for key lifecycle events

#### Security Policies
Comprehensive security controls protect against unauthorized rotation:

- **Rate limiting** - Per-user and per-IP rotation limits
- **IP restrictions** - Allow/block lists and geolocation controls  
- **Time restrictions** - Business hours and maintenance window blocking
- **Risk assessment** - Behavior analysis and anomaly detection
- **Emergency bypass** - Admin override with enhanced logging

## Key Rotation Process

### 1. Client-Side Key Generation

```bash
# Generate new key pair locally
datafold_cli key-gen --output-dir ~/.datafold/keys/rotation

# Verify backup before rotation
datafold_cli key-backup-verify --key-id current-key
```

### 2. Rotation Request Creation

```rust
// Automatic request signing with old key
let rotation_request = KeyRotationRequest::new(
    &old_private_key,           // Signs the request
    new_public_key,             // Replacement key
    RotationReason::Scheduled,  // Why rotating
    Some("webapp".to_string()), // Client identifier
    metadata,                   // Additional context
)?;
```

### 3. Server-Side Processing

1. **Signature verification** - Validates old key ownership
2. **Security policy evaluation** - Checks rate limits, IP restrictions, risk assessment
3. **Atomic database transaction** - Updates all key associations
4. **Network propagation** - Broadcasts change to peer nodes
5. **Audit logging** - Records complete operation trail

### 4. Response and Confirmation

```json
{
  "success": true,
  "new_key_id": "ed25519:abc123...",
  "old_key_invalidated": true,
  "correlation_id": "uuid-for-audit-trail",
  "timestamp": "2025-06-13T09:21:00Z",
  "warnings": [],
  "associations_updated": 42
}
```

## CLI Usage

### Basic Key Rotation

```bash
# Interactive rotation with backup verification
datafold_cli rotate-key --reason scheduled --interactive

# Automated rotation (for scheduled operations)
datafold_cli rotate-key \
  --old-key ~/.datafold/keys/current.key \
  --reason scheduled \
  --client-id scheduler \
  --verify-backup
```

### Emergency Key Replacement

```bash
# Immediate rotation for compromised keys
datafold_cli rotate-key \
  --reason compromise \
  --force \
  --emergency-override \
  --justification "Key leaked in logs"
```

### Rotation Status and History

```bash
# Check rotation status
datafold_cli rotation-status --correlation-id <uuid>

# View rotation history
datafold_cli rotation-history --limit 10 --format json

# Audit specific rotation
datafold_cli rotation-audit --correlation-id <uuid>
```

## API Integration

### HTTP API Endpoints

```bash
# Initiate key rotation
POST /api/crypto/rotate-key
Content-Type: application/json
Authorization: Signature keyId="current-key",signature="..."

{
  "old_public_key": "hex-encoded-old-key",
  "new_public_key": "hex-encoded-new-key", 
  "reason": "scheduled",
  "timestamp": 1718264460000,
  "nonce": "hex-encoded-nonce",
  "signature": "hex-encoded-signature"
}

# Check rotation status
GET /api/crypto/rotation-status/{correlation_id}

# Get rotation history
GET /api/crypto/rotation-history?limit=10&user_id=user123
```

### JavaScript SDK

```typescript
import { KeyRotationManager } from '@datafold/js-sdk';

const manager = new KeyRotationManager(client);

// Rotate with progress tracking
const result = await manager.rotateKey({
  reason: 'scheduled',
  verifyBackup: true,
  onProgress: (progress) => {
    console.log(`${progress.step}: ${progress.percentage}%`);
  }
});

console.log(`New key: ${result.new_key_id}`);
```

### Python SDK

```python
from datafold import KeyRotationClient

client = KeyRotationClient.from_config()

# Simple rotation
result = await client.rotate_key(
    reason='user_initiated',
    verify_backup=True,
    metadata={'app': 'data-processor'}
)

print(f"Rotation successful: {result.correlation_id}")
```

## Security Considerations

### Security Model

1. **Cryptographic Authentication**: All operations require valid Ed25519 signatures
2. **Signature-based Authorization**: Old private key must sign rotation requests  
3. **Rate Limiting**: Configurable limits prevent rotation abuse
4. **Risk Assessment**: Behavioral analysis detects unusual patterns
5. **Audit Logging**: Complete trail of all security events
6. **Client-side Key Control**: Private keys never leave client environment

### Best Practices

#### For Production Use
1. **Always verify backups** before rotating keys
2. **Use specific rotation reasons** for audit trail clarity
3. **Implement client-side key validation** before sending requests
4. **Monitor rotation audit logs** for suspicious activity
5. **Test rotation procedures** in non-production environments

#### For Development
1. **Use separate key pairs** for each environment
2. **Automate scheduled rotations** for security hygiene
3. **Document rotation procedures** in runbooks
4. **Test emergency rotation** scenarios regularly

## Troubleshooting

### Common Issues

#### Rotation Request Rejected
```
Error: Invalid signature on rotation request
```

**Solution**: Ensure old private key is correct and request is properly signed:
```bash
datafold_cli verify-signature --request rotation_request.json --key old_key.pem
```

#### Rate Limit Exceeded
```
Error: User rate limit exceeded: 10 rotations in the last hour
```

**Solution**: Wait for rate limit window to reset, or use emergency bypass if authorized.

#### Risk Assessment Block
```
Error: Risk score 0.8 exceeds maximum allowed 0.7
```

**Solution**: Review risk factors (unusual IP, time, etc.) or use emergency bypass if necessary.

### Diagnostic Commands

```bash
# Verify key rotation configuration
datafold_cli config validate --section key_rotation

# Check security policy status
datafold_cli security-policy status --verbose

# Validate client key setup
datafold_cli key-validate --key-id current-key

# Test signature authentication
datafold_cli auth-test --key-id current-key
```

## Advanced Configuration

### Custom Security Policies

```rust
// Example: Stricter policy for production
let production_policy = KeyRotationSecurityPolicy {
    name: "production_strict".to_string(),
    enabled: true,
    rate_limits: RateLimitConfig {
        max_rotations_per_user_per_hour: 2,  // Very restrictive
        max_failed_attempts_per_user_per_hour: 3,
        lockout_duration_minutes: 60,
        enable_progressive_delays: true,
    },
    risk_assessment: RiskAssessmentConfig {
        enabled: true,
        max_risk_score: 0.5,  // Lower threshold for stricter security
        risk_factors: RiskFactors {
            unusual_ip_weight: 0.4,
            unusual_location_weight: 0.3,
            unusual_time_weight: 0.2,
            new_device_weight: 0.3,
            vpn_proxy_weight: 0.4,
            recent_failures_weight: 0.5,
        },
        risk_actions: RiskActions {
            low_risk: RiskAction::Allow,
            medium_risk: RiskAction::AllowWithMonitoring,
            high_risk: RiskAction::Block,
        },
    },
    // ... other settings
};
```

### Risk-Based Policies

```rust
// Example: Risk-adaptive security
let adaptive_policy = RiskAssessmentConfig {
    enabled: true,
    max_risk_score: 0.6,  // Moderate threshold
    risk_actions: RiskActions {
        low_risk: RiskAction::Allow,
        medium_risk: RiskAction::AllowWithMonitoring,
        high_risk: RiskAction::Block,
    },
    // Risk factors for different scenarios
    risk_factors: RiskFactors {
        unusual_ip_weight: 0.3,
        unusual_location_weight: 0.2,
        unusual_time_weight: 0.1,
        new_device_weight: 0.2,
        vpn_proxy_weight: 0.3,
        recent_failures_weight: 0.4,
    },
};
```

## Emergency Procedures

### Emergency Key Rotation

When immediate key rotation is required:

1. **Assess the situation** - Determine if key is compromised
2. **Prepare justification** - Document reason for emergency rotation
3. **Use emergency override** - If available and authorized
4. **Monitor closely** - Watch for successful completion
5. **Verify propagation** - Ensure all systems updated
6. **Update documentation** - Record incident and resolution

### Recovery Procedures

If key rotation fails:

1. **Check rotation status** - Use correlation ID to track progress
2. **Review audit logs** - Identify specific failure point
3. **Verify key backup** - Ensure old key is still accessible
4. **Retry rotation** - If failure was transient
5. **Use emergency bypass** - If rotation is critical
6. **Contact support** - For complex recovery scenarios

## Monitoring and Alerting

### Key Metrics

- **Rotation success rate** - Percentage of successful rotations
- **Average rotation time** - Time from request to completion
- **Security violations** - Blocked attempts and policy violations
- **Risk scores** - Distribution of risk assessments
- **Emergency bypass usage** - Frequency and justification

### Recommended Alerts

- **Failed rotations** - Immediate alert for rotation failures
- **High risk rotations** - Alert for rotations with elevated risk
- **Rate limit hits** - Notice when users hit rate limits
- **Emergency bypass** - Alert when bypass procedures used
- **Policy violations** - Notice for security policy violations

For production deployment guidance, see the [Production Deployment Guide](../deployment-guide.md).