# Production Deployment Guide for Mandatory Authentication

## Overview

This guide covers deploying DataFold with mandatory signature authentication in production environments. All DataFold deployments require proper authentication configuration as a security prerequisite.

## ⚠️ Critical Security Requirements

**Mandatory authentication cannot be disabled.** All production deployments must:

- Use **strict security profile** for production environments
- Implement proper key management with Hardware Security Modules (HSMs)
- Configure comprehensive monitoring and alerting
- Follow security best practices for cryptographic operations

## Pre-Deployment Checklist

### Security Infrastructure
- [ ] Hardware Security Module (HSM) or secure key vault configured
- [ ] Network Time Protocol (NTP) synchronization enabled
- [ ] TLS 1.3+ encryption for all network communications
- [ ] Firewall rules configured for required ports only
- [ ] Security monitoring and SIEM integration ready

### Authentication Setup
- [ ] Ed25519 key pairs generated using cryptographically secure methods
- [ ] Public keys registered with all DataFold servers
- [ ] Authentication profiles configured for production environment
- [ ] Key rotation procedures documented and tested
- [ ] Backup and recovery procedures for authentication materials

### Monitoring and Logging
- [ ] Security event logging configured
- [ ] Authentication failure alerting implemented
- [ ] Performance monitoring for signature operations
- [ ] Compliance logging for audit requirements
- [ ] Incident response procedures documented

## Server Configuration

### 1. DataFold Node Configuration

Create a production configuration file:

```json
{
  "storage_path": "/var/lib/datafold/data",
  "signature_auth": {
    "security_profile": "strict",
    "allowed_time_window_secs": 60,
    "clock_skew_tolerance_secs": 5,
    "nonce_ttl_secs": 300,
    "max_nonce_store_size": 1000000,
    "enforce_rfc3339_timestamps": true,
    "require_uuid4_nonces": true,
    "max_future_timestamp_secs": 5,
    "required_signature_components": [
      "@method",
      "@target-uri", 
      "@authority",
      "content-digest",
      "content-type"
    ],
    "log_replay_attempts": true,
    "security_logging": {
      "enabled": true,
      "include_correlation_ids": true,
      "include_client_info": true,
      "include_performance_metrics": true,
      "log_successful_auth": true,
      "min_severity": "info",
      "max_log_entry_size": 65536
    },
    "rate_limiting": {
      "enabled": true,
      "max_requests_per_window": 1000,
      "window_size_secs": 60,
      "track_failures_separately": true,
      "max_failures_per_window": 10
    },
    "attack_detection": {
      "enabled": true,
      "brute_force_threshold": 5,
      "brute_force_window_secs": 300,
      "replay_threshold": 3,
      "enable_timing_protection": true
    },
    "response_security": {
      "sign_responses": true,
      "include_server_timestamp": true,
      "include_request_correlation": true
    }
  },
  "crypto": {
    "enabled": true,
    "key_derivation_memory_mb": 256,
    "key_derivation_iterations": 5,
    "key_derivation_parallelism": 4
  },
  "network": {
    "port": 9000,
    "bind_address": "0.0.0.0",
    "enable_mdns": false,
    "max_connections": 1000
  },
  "logging": {
    "level": "info",
    "structured": true,
    "file_path": "/var/log/datafold/node.log",
    "max_file_size_mb": 100,
    "max_files": 10
  }
}
```

### 2. Environment Variables

Set required environment variables:

```bash
# Production environment file (/etc/datafold/production.env)
DATAFOLD_CONFIG=/etc/datafold/production.json
DATAFOLD_LOG_LEVEL=info
DATAFOLD_ENVIRONMENT=production

# Key management (use secure key vault in production)
DATAFOLD_HSM_URL=https://your-hsm-service.com
DATAFOLD_KEY_VAULT_TOKEN=your-vault-token

# Security settings
DATAFOLD_SECURITY_PROFILE=strict
DATAFOLD_ENABLE_SECURITY_LOGGING=true
DATAFOLD_RATE_LIMITING_ENABLED=true

# Monitoring
DATAFOLD_METRICS_ENDPOINT=http://prometheus:9090
DATAFOLD_LOGGING_ENDPOINT=http://elasticsearch:9200
```

### 3. Systemd Service Configuration

Create `/etc/systemd/system/datafold-node.service`:

```ini
[Unit]
Description=DataFold Node with Mandatory Authentication
After=network.target
Requires=network.target

[Service]
Type=simple
User=datafold
Group=datafold
WorkingDirectory=/opt/datafold
EnvironmentFile=/etc/datafold/production.env
ExecStart=/usr/local/bin/datafold_node --config ${DATAFOLD_CONFIG}
ExecReload=/bin/kill -HUP $MAINPID
Restart=always
RestartSec=10
TimeoutStartSec=300
TimeoutStopSec=30

# Security settings
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
ReadWritePaths=/var/lib/datafold /var/log/datafold
PrivateTmp=true
MemoryDenyWriteExecute=true

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
```

## Client Configuration

### 1. Authentication Profile Setup

Create production authentication profiles:

```bash
# Create production profile with strict security
datafold auth-profile create production \
  --server-url https://datafold.yourcompany.com \
  --key-id production-key-2024 \
  --security-profile strict \
  --user-id production-client \
  --set-default

# Create staging profile for testing
datafold auth-profile create staging \
  --server-url https://staging-datafold.yourcompany.com \
  --key-id staging-key-2024 \
  --security-profile standard \
  --user-id staging-client
```

### 2. Application Configuration

For applications using DataFold:

```typescript
// Production client configuration
import { DataFoldHttpClient, SecurityProfile } from '@datafold/client';

const client = new DataFoldHttpClient({
  baseUrl: process.env.DATAFOLD_SERVER_URL,
  authenticationRequired: true, // Always true
  securityProfile: SecurityProfile.STRICT,
  signingConfig: {
    keyId: process.env.DATAFOLD_KEY_ID,
    privateKey: await loadFromHSM(process.env.DATAFOLD_KEY_ID),
    requiredComponents: [
      '@method', 
      '@target-uri', 
      '@authority',
      'content-digest',
      'content-type'
    ],
    includeTimestamp: true,
    includeNonce: true,
    maxClockSkew: 5 // 5 seconds for strict mode
  },
  // Production security settings
  enableSignatureCache: true,
  signatureCacheTtl: 30000, // 30 seconds max in production
  mandatoryVerification: true,
  debugLogging: false,
  // Error handling
  retryConfig: {
    maxRetries: 3,
    backoffMultiplier: 2,
    maxBackoffMs: 5000
  },
  // Monitoring
  metricsCollector: productionMetricsCollector,
  securityEventLogger: productionSecurityLogger
});
```

## Key Management

### 1. HSM Integration

For production key management:

```bash
# Generate keys in HSM
hsm-cli generate-key --algorithm ed25519 --key-id datafold-prod-2024 --purpose signing

# Export public key for registration
hsm-cli export-public-key --key-id datafold-prod-2024 --format hex > production.pub

# Register public key with DataFold
datafold register-public-key \
  --key-id datafold-prod-2024 \
  --public-key-file production.pub \
  --client-id production-client \
  --environment production
```

### 2. Key Rotation Procedures

Implement automated key rotation:

```bash
#!/bin/bash
# Key rotation script (/opt/datafold/scripts/rotate-keys.sh)

set -euo pipefail

CURRENT_KEY_ID="datafold-prod-2024"  
NEW_KEY_ID="datafold-prod-$(date +%Y%m)"
HSM_URL="${DATAFOLD_HSM_URL}"
SERVER_URL="${DATAFOLD_SERVER_URL}"

echo "Starting key rotation from ${CURRENT_KEY_ID} to ${NEW_KEY_ID}"

# Generate new key in HSM
hsm-cli generate-key --algorithm ed25519 --key-id "${NEW_KEY_ID}" --purpose signing

# Export public key
hsm-cli export-public-key --key-id "${NEW_KEY_ID}" --format hex > "/tmp/${NEW_KEY_ID}.pub"

# Register new key with DataFold
datafold register-public-key \
  --key-id "${NEW_KEY_ID}" \
  --public-key-file "/tmp/${NEW_KEY_ID}.pub" \
  --client-id production-client \
  --environment production

# Test new key
datafold auth-test --key-id "${NEW_KEY_ID}"

# Update configuration to use new key
datafold auth-profile update production --key-id "${NEW_KEY_ID}"

# Verify new configuration
datafold auth-test --profile production

# Schedule old key deactivation (after grace period)
echo "${CURRENT_KEY_ID}" >> /var/lib/datafold/keys-to-deactivate.txt

echo "Key rotation completed successfully"
```

## Monitoring and Alerting

### 1. Security Metrics

Monitor these critical security metrics:

```yaml
# Prometheus monitoring rules
groups:
  - name: datafold-security
    rules:
      - alert: DataFoldAuthenticationFailures
        expr: rate(datafold_auth_failures_total[5m]) > 0.1
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "High authentication failure rate"
          description: "DataFold authentication failures exceed threshold"

      - alert: DataFoldReplayAttackDetected
        expr: increase(datafold_replay_attacks_total[5m]) > 0
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "Replay attack detected"
          description: "Potential replay attack against DataFold"

      - alert: DataFoldSignatureLatencyHigh
        expr: histogram_quantile(0.95, rate(datafold_signature_duration_seconds_bucket[5m])) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High signature verification latency"
          description: "DataFold signature operations are slow"

      - alert: DataFoldRateLimitExceeded
        expr: increase(datafold_rate_limit_exceeded_total[5m]) > 10
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "Rate limiting triggered"
          description: "Multiple clients hitting rate limits"
```

### 2. Log Analysis

Configure log aggregation and analysis:

```yaml
# Elasticsearch/Kibana dashboard configuration
{
  "dashboard": {
    "title": "DataFold Security Dashboard",
    "panels": [
      {
        "title": "Authentication Events",
        "type": "histogram",
        "query": "service:datafold AND event_type:authentication"
      },
      {
        "title": "Failed Authentication by IP",
        "type": "table", 
        "query": "service:datafold AND event_type:authentication_failure",
        "group_by": "client_ip"
      },
      {
        "title": "Signature Verification Performance",
        "type": "line_chart",
        "query": "service:datafold AND metrics.signature_verification_time_ms:*"
      },
      {
        "title": "Security Alerts",
        "type": "table",
        "query": "service:datafold AND severity:(critical OR warning)"
      }
    ]
  }
}
```

## Health Checks and Monitoring

### 1. Authentication Health Check

```bash
#!/bin/bash
# Health check script (/opt/datafold/scripts/health-check.sh)

set -euo pipefail

SERVER_URL="${DATAFOLD_SERVER_URL}"
PROFILE="${DATAFOLD_PROFILE:-production}"

# Test authentication
if ! datafold auth-test --profile "${PROFILE}" >/dev/null 2>&1; then
    echo "CRITICAL: Authentication test failed"
    exit 2
fi

# Test signature verification
if ! datafold verify-response --url "${SERVER_URL}/health" --method get >/dev/null 2>&1; then
    echo "CRITICAL: Server response verification failed"
    exit 2
fi

# Check key accessibility
if ! datafold list-keys --key-id "${DATAFOLD_KEY_ID}" >/dev/null 2>&1; then
    echo "CRITICAL: Cannot access signing key"
    exit 2
fi

echo "OK: All authentication checks passed"
exit 0
```

### 2. Kubernetes Health Checks

For Kubernetes deployments:

```yaml
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: datafold-node
    image: datafold:latest
    env:
    - name: DATAFOLD_SECURITY_PROFILE
      value: "strict"
    livenessProbe:
      exec:
        command:
        - /opt/datafold/scripts/health-check.sh
      initialDelaySeconds: 30
      periodSeconds: 60
      timeoutSeconds: 10
      failureThreshold: 3
    readinessProbe:
      httpGet:
        path: /health
        port: 9000
        httpHeaders:
        - name: Authorization
          value: "Signature ..."
      initialDelaySeconds: 15
      periodSeconds: 30
      timeoutSeconds: 5
      failureThreshold: 2
```

## Security Compliance

### 1. Audit Configuration

Enable comprehensive audit logging:

```json
{
  "audit_logging": {
    "enabled": true,
    "events": [
      "authentication_success",
      "authentication_failure", 
      "key_registration",
      "key_rotation",
      "configuration_change",
      "security_policy_change",
      "rate_limit_exceeded"
    ],
    "destinations": [
      {
        "type": "file",
        "path": "/var/log/audit/datafold-audit.log"
      },
      {
        "type": "syslog",
        "facility": "auth",
        "severity": "info"
      },
      {
        "type": "webhook",
        "url": "https://siem.yourcompany.com/datafold-audit"
      }
    ]
  }
}
```

### 2. Compliance Reporting

Generate compliance reports:

```bash
#!/bin/bash
# Compliance report generator

REPORT_DATE=$(date +%Y-%m-%d)
REPORT_FILE="/var/reports/datafold-compliance-${REPORT_DATE}.json"

cat > "${REPORT_FILE}" <<EOF
{
  "report_date": "${REPORT_DATE}",
  "compliance_framework": "SOC2_TYPE2",
  "authentication_status": {
    "mandatory_auth_enabled": true,
    "security_profile": "strict",
    "key_rotation_enabled": true,
    "audit_logging_enabled": true
  },
  "security_controls": [
    {
      "control_id": "CC6.1",
      "description": "Logical access security measures",
      "status": "implemented",
      "evidence": "Mandatory signature authentication with Ed25519"
    },
    {
      "control_id": "CC6.2", 
      "description": "User authentication",
      "status": "implemented",
      "evidence": "RFC 9421 HTTP Message Signatures required for all requests"
    },
    {
      "control_id": "CC6.7",
      "description": "Data transmission controls",
      "status": "implemented", 
      "evidence": "TLS 1.3 encryption with signature authentication"
    }
  ]
}
EOF

echo "Compliance report generated: ${REPORT_FILE}"
```

## Troubleshooting

### Common Production Issues

1. **Authentication failures after deployment:**
   ```bash
   # Check key accessibility
   datafold auth-status --verbose
   
   # Verify server connectivity
   datafold auth-test --profile production --debug
   
   # Check system time synchronization
   chrony sources -v
   ```

2. **High signature verification latency:**
   ```bash
   # Check system resources
   top -p $(pgrep datafold_node)
   
   # Monitor signature performance
   datafold performance-monitor --auth-metrics
   
   # Check HSM connectivity
   hsm-cli health-check
   ```

3. **Rate limiting issues:**
   ```bash
   # Check rate limit status
   datafold auth-status --rate-limits
   
   # Review recent authentication patterns
   grep "rate_limit" /var/log/datafold/node.log | tail -20
   ```

For additional troubleshooting, see the [Troubleshooting Guide](troubleshooting-guide.md).

## Security Incident Response

### 1. Authentication Compromise Response

If authentication keys are compromised:

```bash
#!/bin/bash
# Emergency key revocation script

COMPROMISED_KEY_ID="$1"
EMERGENCY_CONTACT="security@yourcompany.com"

echo "SECURITY INCIDENT: Revoking compromised key ${COMPROMISED_KEY_ID}"

# Immediately revoke key
datafold revoke-key --key-id "${COMPROMISED_KEY_ID}" --reason "security_incident"

# Generate new emergency key
EMERGENCY_KEY_ID="emergency-$(date +%s)"
hsm-cli generate-key --algorithm ed25519 --key-id "${EMERGENCY_KEY_ID}"

# Update all production profiles
datafold auth-profile update production --key-id "${EMERGENCY_KEY_ID}"

# Send security alert
echo "Key ${COMPROMISED_KEY_ID} revoked. Emergency key ${EMERGENCY_KEY_ID} activated." | \
  mail -s "DataFold Security Incident" "${EMERGENCY_CONTACT}"

echo "Emergency key rotation completed"
```

### 2. Attack Pattern Response

For detected attack patterns:

```bash
# Block suspicious clients
datafold security-block --client-ip "suspicious.ip.address" --duration "24h"

# Increase security monitoring
datafold configure --security-profile strict --enhanced-monitoring

# Generate incident report
datafold security-report --incident-type "attack_pattern" --output "/var/reports/incident-$(date +%s).json"
```

This deployment guide provides comprehensive coverage of production deployment requirements for DataFold with mandatory authentication. Always test configurations in staging environments before production deployment.