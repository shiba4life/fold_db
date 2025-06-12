# DataFold Migration Guide: Moving to Mandatory Authentication

## Overview

This guide helps you migrate existing DataFold deployments to the new mandatory signature authentication system. **Authentication is now required for all DataFold operations** and cannot be disabled.

## ⚠️ Breaking Changes

**CRITICAL**: This is a breaking change that affects all DataFold deployments:

- **Signature authentication is now mandatory** for all requests
- **No bypass options** - authentication cannot be disabled
- **Security profiles required** - must select appropriate security level
- **Key management mandatory** - all clients must have Ed25519 key pairs
- **Configuration changes required** - existing configs must be updated

## Migration Timeline

### Phase 1: Pre-Migration (1-2 weeks before)
- [ ] Audit existing DataFold deployments
- [ ] Plan authentication infrastructure
- [ ] Set up key management systems
- [ ] Prepare monitoring and logging
- [ ] Test migration in staging environment

### Phase 2: Migration Preparation (1 week before)
- [ ] Generate and register Ed25519 key pairs
- [ ] Create authentication profiles for all environments
- [ ] Update application configurations
- [ ] Set up backup and recovery procedures
- [ ] Schedule maintenance windows

### Phase 3: Migration Execution (Migration day)
- [ ] Stop existing DataFold services
- [ ] Update to mandatory authentication version
- [ ] Apply new configurations
- [ ] Start services with authentication enabled
- [ ] Verify all systems are operational
- [ ] Monitor for authentication issues

### Phase 4: Post-Migration (1-2 weeks after)
- [ ] Monitor authentication metrics
- [ ] Address any compatibility issues
- [ ] Optimize performance settings
- [ ] Update documentation and procedures
- [ ] Train operations team on new authentication system

## Pre-Migration Assessment

### 1. Inventory Current Deployments

Run this assessment script on each system:

```bash
#!/bin/bash
# DataFold Migration Assessment Script

echo "DataFold Migration Assessment Report"
echo "Generated: $(date)"
echo "Host: $(hostname)"
echo "================================="

# Check DataFold installation
echo -e "\n--- DataFold Installation ---"
if command -v datafold_cli &> /dev/null; then
    datafold_cli --version
    echo "Installation path: $(which datafold_cli)"
else
    echo "DataFold CLI not found"
fi

if command -v datafold_node &> /dev/null; then
    datafold_node --version
    echo "Node installation path: $(which datafold_node)"
else
    echo "DataFold Node not found"
fi

# Check current configuration
echo -e "\n--- Current Configuration ---"
if [ -f ~/.datafold/config.toml ]; then
    echo "Config file exists: ~/.datafold/config.toml"
    grep -E "(auth|sign)" ~/.datafold/config.toml || echo "No authentication config found"
else
    echo "No DataFold config file found"
fi

# Check for existing keys
echo -e "\n--- Existing Keys ---"
if [ -d ~/.datafold/keys ]; then
    echo "Keys directory exists"
    ls -la ~/.datafold/keys/ || echo "No keys found"
else
    echo "No keys directory found"
fi

# Check running processes
echo -e "\n--- Running Processes ---"
ps aux | grep -E "datafold" | grep -v grep || echo "No DataFold processes running"

# Check network connectivity
echo -e "\n--- Network Connectivity ---"
if [ -n "$DATAFOLD_SERVER_URL" ]; then
    echo "Testing connectivity to: $DATAFOLD_SERVER_URL"
    curl -I "$DATAFOLD_SERVER_URL/health" 2>/dev/null || echo "Cannot connect to server"
else
    echo "No DATAFOLD_SERVER_URL configured"
fi

echo -e "\n--- Migration Requirements ---"
echo "[ ] Install/upgrade to latest DataFold version with mandatory auth"
echo "[ ] Generate Ed25519 key pairs for all clients"
echo "[ ] Set up authentication profiles"
echo "[ ] Update application configurations"
echo "[ ] Test authentication in staging environment"
echo -e "\nAssessment complete. Save this report for migration planning."
```

### 2. Identify Integration Points

Document all systems that integrate with DataFold:

```bash
# Find DataFold integrations
echo "Scanning for DataFold integrations..."

# Check application code
find /path/to/applications -name "*.js" -o -name "*.py" -o -name "*.rs" | \
  xargs grep -l "datafold" > datafold-integrations.txt

# Check configuration files
find /etc -name "*.conf" -o -name "*.yaml" -o -name "*.json" | \
  xargs grep -l "datafold" >> datafold-integrations.txt

# Check scripts and automation
find /opt /usr/local/bin -name "*.sh" -o -name "*.py" | \
  xargs grep -l "datafold" >> datafold-integrations.txt

echo "Integration points saved to: datafold-integrations.txt"
```

## Migration Steps

### Step 1: Install Authentication-Enabled Version

#### Rust/CLI Installation
```bash
# Backup current installation
cp $(which datafold_cli) /backup/datafold_cli.backup 2>/dev/null || true
cp -r ~/.datafold /backup/datafold-config.backup 2>/dev/null || true

# Install latest version with mandatory authentication
cargo install datafold --force

# Verify installation includes authentication support
datafold_cli --version
datafold_cli auth-setup --help
```

#### Server Installation
```bash
# Stop existing DataFold services
sudo systemctl stop datafold-node
sudo systemctl stop datafold-http-server

# Backup existing configuration
sudo cp /etc/datafold/node.conf /backup/node.conf.backup
sudo cp -r /var/lib/datafold /backup/datafold-data.backup

# Install new version
sudo cargo install datafold --root /usr/local
sudo systemctl daemon-reload
```

### Step 2: Key Generation and Management

#### Generate Ed25519 Key Pairs

For each client system:

```bash
# Generate client key pair
datafold auth-keygen --key-id "client-$(hostname)-$(date +%Y%m)" \
  --description "Client key for $(hostname) - $(date)"

# Generate server key pair (if running DataFold server)
datafold auth-keygen --key-id "server-$(hostname)-$(date +%Y%m)" \
  --description "Server key for $(hostname) - $(date)" \
  --server-key

# Backup keys securely
cp ~/.datafold/keys/*.json /secure/backup/location/
chmod 600 /secure/backup/location/*.json
```

#### Register Public Keys

Register client public keys with all DataFold servers:

```bash
# List of servers to register with
SERVERS=(
  "https://datafold-prod.company.com"
  "https://datafold-staging.company.com"
  "https://datafold-dev.company.com"
)

CLIENT_KEY_ID="client-$(hostname)-$(date +%Y%m)"

for server in "${SERVERS[@]}"; do
  echo "Registering key with $server"
  
  datafold register-public-key \
    --server-url "$server" \
    --key-id "$CLIENT_KEY_ID" \
    --client-id "$(hostname)" \
    --description "Migration key for $(hostname)" \
    --public-key-file ~/.datafold/keys/${CLIENT_KEY_ID}.pub
done
```

### Step 3: Create Authentication Profiles

Create environment-specific authentication profiles:

```bash
# Production profile with strict security
datafold auth-profile create production \
  --server-url https://datafold-prod.company.com \
  --key-id "$CLIENT_KEY_ID" \
  --security-profile strict \
  --user-id "prod-$(whoami)" \
  --description "Production environment profile"

# Staging profile with standard security
datafold auth-profile create staging \
  --server-url https://datafold-staging.company.com \
  --key-id "$CLIENT_KEY_ID" \
  --security-profile standard \
  --user-id "staging-$(whoami)" \
  --description "Staging environment profile"

# Development profile with lenient security
datafold auth-profile create development \
  --server-url https://datafold-dev.company.com \
  --key-id "$CLIENT_KEY_ID" \
  --security-profile lenient \
  --user-id "dev-$(whoami)" \
  --description "Development environment profile"

# Set default profile based on environment
case "${ENVIRONMENT:-production}" in
  "production")
    datafold auth-profile set-default production
    ;;
  "staging") 
    datafold auth-profile set-default staging
    ;;
  "development")
    datafold auth-profile set-default development
    ;;
esac
```

### Step 4: Update Application Configurations

#### Environment Variables

Update environment variable configurations:

```bash
# /etc/environment or application-specific env files

# Required authentication variables
DATAFOLD_KEY_ID=client-hostname-202412
DATAFOLD_SECURITY_PROFILE=strict
DATAFOLD_DEFAULT_PROFILE=production

# Optional configuration variables
DATAFOLD_AUTH_DEBUG=false
DATAFOLD_SIGNATURE_CACHE_TTL=60
DATAFOLD_MAX_CLOCK_SKEW=30

# Legacy variables (remove these)
# DATAFOLD_DISABLE_AUTH=true  # <- Remove, auth cannot be disabled
# DATAFOLD_SKIP_VERIFICATION=true  # <- Remove, verification is mandatory
```

#### Application Code Updates

**JavaScript/TypeScript Applications:**

```typescript
// Before migration (optional authentication)
const client = new DataFoldHttpClient({
  baseUrl: 'https://api.company.com',
  signingMode: 'manual' // or 'disabled'
});

// After migration (mandatory authentication)
import { DataFoldHttpClient, SecurityProfile } from '@datafold/client';

const client = new DataFoldHttpClient({
  baseUrl: 'https://api.company.com',
  // Authentication is always required - no option to disable
  authenticationRequired: true,
  securityProfile: process.env.NODE_ENV === 'production' 
    ? SecurityProfile.STRICT 
    : SecurityProfile.STANDARD,
  signingConfig: {
    keyId: process.env.DATAFOLD_KEY_ID,
    privateKey: await loadPrivateKey(process.env.DATAFOLD_KEY_ID),
    requiredComponents: ['@method', '@target-uri', 'content-digest'],
    includeTimestamp: true,
    includeNonce: true
  }
});
```

**Python Applications:**

```python
# Before migration (optional authentication)
from datafold_sdk import DataFoldClient

client = DataFoldClient(
    base_url='https://api.company.com',
    signing_enabled=False  # Could be disabled
)

# After migration (mandatory authentication)
from datafold_sdk import DataFoldClient, SecurityProfile
from datafold_sdk.signing import SigningConfig

client = DataFoldClient(
    base_url='https://api.company.com',
    # Authentication is always enabled - no option to disable
    authentication_required=True,
    security_profile=SecurityProfile.STRICT if os.getenv('ENV') == 'production' else SecurityProfile.STANDARD,
    signing_config=SigningConfig(
        key_id=os.getenv('DATAFOLD_KEY_ID'),
        private_key=load_private_key(os.getenv('DATAFOLD_KEY_ID')),
        required_components=['@method', '@target-uri', 'content-digest'],
        include_timestamp=True,
        include_nonce=True
    )
)
```

**Shell Scripts:**

```bash
# Before migration (optional authentication)
datafold query --no-sign --schema users --fields id,name

# After migration (mandatory authentication)
# Authentication is automatic - no --no-sign flag available
datafold query --schema users --fields id,name

# Specify profile if not using default
datafold query --profile production --schema users --fields id,name
```

### Step 5: Server Configuration Updates

Update DataFold server configurations:

```json
{
  "storage_path": "/var/lib/datafold/data",
  "signature_auth": {
    "security_profile": "strict",
    "allowed_time_window_secs": 300,
    "clock_skew_tolerance_secs": 30,
    "nonce_ttl_secs": 600,
    "max_nonce_store_size": 1000000,
    "enforce_rfc3339_timestamps": true,
    "require_uuid4_nonces": true,
    "required_signature_components": [
      "@method",
      "@target-uri", 
      "content-digest"
    ],
    "log_replay_attempts": true,
    "security_logging": {
      "enabled": true,
      "include_correlation_ids": true,
      "include_client_info": true,
      "log_successful_auth": true,
      "min_severity": "info"
    },
    "rate_limiting": {
      "enabled": true,
      "max_requests_per_window": 1000,
      "window_size_secs": 60,
      "max_failures_per_window": 10
    },
    "attack_detection": {
      "enabled": true,
      "brute_force_threshold": 5,
      "brute_force_window_secs": 300,
      "replay_threshold": 3
    }
  }
}
```

### Step 6: Testing and Validation

#### Pre-Migration Testing

```bash
# Test authentication setup
datafold auth-test --all-profiles

# Test key accessibility  
datafold list-keys --health-check

# Test server connectivity
datafold auth-test --profile production --full-verification

# Test signature verification
datafold verify-response --url https://server.com/health --method get
```

#### Migration Smoke Tests

```bash
#!/bin/bash
# Post-migration smoke test script

echo "Running DataFold migration smoke tests..."

# Test authentication is working
echo "Testing authentication..."
if datafold auth-test; then
    echo "✅ Authentication test passed"
else
    echo "❌ Authentication test failed"
    exit 1
fi

# Test basic operations
echo "Testing basic operations..."
if datafold auth-status > /dev/null; then
    echo "✅ Auth status check passed"
else
    echo "❌ Auth status check failed"
    exit 1
fi

# Test server connectivity with authentication
echo "Testing server connectivity..."
if datafold verify-response --url "$DATAFOLD_SERVER_URL/health" --method get > /dev/null; then
    echo "✅ Server connectivity test passed"
else
    echo "❌ Server connectivity test failed"
    exit 1
fi

# Test profile switching
echo "Testing profile switching..."
for profile in production staging development; do
    if datafold auth-profile show "$profile" > /dev/null 2>&1; then
        echo "✅ Profile $profile exists"
    else
        echo "⚠️  Profile $profile not found (may be OK)"
    fi
done

echo "Smoke tests completed successfully!"
```

### Step 7: Monitoring and Alerting Updates

Update monitoring configurations to track authentication metrics:

```yaml
# Prometheus/Grafana monitoring rules
groups:
  - name: datafold-migration-monitoring
    rules:
      - alert: DataFoldAuthenticationFailures
        expr: rate(datafold_auth_failures_total[5m]) > 0.1
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "High authentication failure rate after migration"
          
      - alert: DataFoldMigrationSuccess
        expr: rate(datafold_auth_success_total[5m]) > 0
        for: 1m
        labels:
          severity: info
        annotations:
          summary: "DataFold authentication working after migration"
```

## Common Migration Issues

### Issue 1: "Authentication required but not configured"

**Cause:** Application not updated to use authentication

**Solution:**
```bash
# Check if authentication profile exists
datafold auth-profile list

# Create profile if missing
datafold auth-setup --interactive

# Update application environment variables
export DATAFOLD_DEFAULT_PROFILE=production
```

### Issue 2: "Key not found" errors

**Cause:** Key generation or registration incomplete

**Solution:**
```bash
# Generate missing keys
datafold auth-keygen --key-id missing-key

# Register public key with server
datafold register-public-key --key-id missing-key --server-url https://server.com

# Verify registration
datafold list-registered-keys --server-url https://server.com
```

### Issue 3: "Timestamp validation failed"

**Cause:** Clock synchronization issues

**Solution:**
```bash
# Install and configure NTP
sudo apt install ntp
sudo systemctl enable ntp
sudo systemctl start ntp

# Force time synchronization
sudo ntpdate -s time.nist.gov

# Verify time synchronization
timedatectl status
```

### Issue 4: Performance degradation

**Cause:** Signature operations adding latency

**Solution:**
```bash
# Enable signature caching
datafold auth-configure --enable-signature-cache true

# Optimize security profile for performance
datafold auth-configure --security-profile standard

# Monitor performance metrics
datafold performance-monitor --auth-metrics
```

## Rollback Procedures

If migration issues occur, follow these rollback steps:

### Emergency Rollback

```bash
#!/bin/bash
# Emergency rollback script

echo "Starting DataFold migration rollback..."

# Stop new services
sudo systemctl stop datafold-node
sudo systemctl stop datafold-http-server

# Restore previous version
sudo cp /backup/datafold_cli.backup $(which datafold_cli)
sudo cp /backup/node.conf.backup /etc/datafold/node.conf

# Restore data if needed
sudo rsync -av /backup/datafold-data.backup/ /var/lib/datafold/

# Restore user configurations
cp -r /backup/datafold-config.backup ~/.datafold

# Start previous version services
sudo systemctl start datafold-node
sudo systemctl start datafold-http-server

# Verify rollback
sleep 10
if curl -f http://localhost:9000/health > /dev/null 2>&1; then
    echo "✅ Rollback completed successfully"
else
    echo "❌ Rollback may have issues - check logs"
    exit 1
fi
```

### Partial Rollback

For gradual rollback:

```bash
# Rollback specific clients while keeping servers updated
datafold auth-configure --compatibility-mode legacy

# Allow mixed authentication modes temporarily (if supported)
datafold server-configure --allow-mixed-auth true --temporary

# Schedule proper rollback during next maintenance window
```

## Post-Migration Checklist

After successful migration:

- [ ] Verify all authentication profiles are working
- [ ] Confirm all applications can authenticate successfully
- [ ] Check authentication performance metrics
- [ ] Validate security logging is working
- [ ] Test key rotation procedures
- [ ] Update documentation and runbooks
- [ ] Train operations team on authentication troubleshooting
- [ ] Schedule regular authentication health checks
- [ ] Plan key rotation schedule
- [ ] Review and optimize security profiles based on usage

## Support and Resources

- **Migration Support**: Contact technical support with correlation IDs for any issues
- **Documentation**: Refer to updated authentication guides
- **Community**: Join migration discussion forums
- **Emergency Contacts**: Keep emergency contact information readily available

Remember: **Authentication is mandatory in the new version.** There are no workarounds to bypass authentication requirements. All issues must be resolved through proper authentication configuration.