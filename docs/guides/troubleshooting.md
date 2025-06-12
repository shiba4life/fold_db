# DataFold Authentication Troubleshooting Guide

## Overview

This guide helps you resolve authentication issues with DataFold's mandatory signature authentication system. All DataFold operations require proper RFC 9421 HTTP Message Signatures, and this guide provides step-by-step solutions for common problems.

## ⚠️ Critical Information

**Authentication is mandatory and cannot be disabled.** If you're experiencing authentication issues, your DataFold system will be completely inaccessible until resolved. This guide provides systematic approaches to diagnose and fix authentication problems.

## Quick Diagnosis Tools

### 1. Authentication Status Check

Start with a comprehensive status check:

```bash
# Check overall authentication status
datafold auth-status --verbose

# Check specific profile
datafold auth-status --profile production --detailed

# Test authentication with server
datafold auth-test --full-verification

# Check key accessibility
datafold list-keys --health-check
```

### 2. System Health Validation

```bash
# Validate environment configuration
datafold auth-validate --environment production

# Check signature verification setup
datafold verify-response --url https://your-server.com/health --method get

# Test end-to-end authentication flow
datafold auth-test --profile production --end-to-end
```

## Common Issues and Solutions

### Issue 1: "Authentication required but not configured"

**Symptoms:**
- Error: `Authentication configuration missing`
- Error: `No authentication profile specified`
- CLI commands fail immediately

**Diagnosis:**
```bash
# Check if any profiles exist
datafold auth-profile list

# Check default profile
datafold auth-configure --show

# Verify configuration file
ls -la ~/.datafold/config.toml
```

**Solution:**
```bash
# Complete authentication setup
datafold auth-setup --interactive

# Or manual setup:
# 1. Generate key pair
datafold auth-keygen --key-id my-key

# 2. Create authentication profile  
datafold auth-profile create default \
  --server-url https://your-server.com \
  --key-id my-key \
  --set-default

# 3. Test setup
datafold auth-test
```

### Issue 2: "Signature verification failed"

**Symptoms:**
- Error: `Signature verification failed for key_id: xxx`
- Error: `SIGNATURE_VERIFICATION_FAILED`
- 401 Unauthorized responses

**Diagnosis:**
```bash
# Enable debug logging
datafold auth-test --auth-debug --verbose

# Check key consistency
datafold retrieve-key --key-id your-key --public-only
datafold verify-key --private-key-file ~/.datafold/keys/your-key.json

# Test signature generation
datafold inspect-signature --test-signing --key-id your-key
```

**Solutions:**

**A. Key Mismatch Issue:**
```bash
# Re-register public key with server
datafold register-public-key \
  --key-id your-key \
  --public-key-file ~/.datafold/keys/your-key.pub \
  --force-update

# Verify registration
datafold auth-test --key-verification
```

**B. Signature Algorithm Issue:**
```bash
# Check algorithm configuration
datafold auth-configure --show | grep algorithm

# Ensure Ed25519 is used
datafold auth-profile update default --algorithm ed25519
```

**C. Canonical Message Construction Issue:**
```bash
# Test canonical message building
datafold debug-canonical-message \
  --method GET \
  --url https://your-server.com/test \
  --debug-components

# Compare with server expectations
datafold debug-signature --compare-server
```

### Issue 3: "Timestamp validation failed"

**Symptoms:**
- Error: `Request timestamp is too old`
- Error: `Request timestamp is too far in the future`
- Error: `TIMESTAMP_VALIDATION_FAILED`

**Diagnosis:**
```bash
# Check system time
date
timedatectl status

# Check server time
datafold auth-test --timestamp-check --verbose

# Check security profile settings
datafold auth-configure --show | grep -E "(time_window|clock_skew)"
```

**Solutions:**

**A. Clock Synchronization:**
```bash
# Install and configure NTP
sudo apt install ntp
sudo systemctl enable ntp
sudo systemctl start ntp

# Force time synchronization
sudo ntpdate -s time.nist.gov

# Verify synchronization
ntpq -p
```

**B. Adjust Security Profile:**
```bash
# For development - use lenient profile
datafold auth-configure --security-profile lenient

# For production - adjust time window
datafold auth-profile update production \
  --time-window-secs 300 \
  --clock-skew-tolerance 60
```

**C. Server Configuration Issue:**
```bash
# Check if server has correct time
curl -I https://your-server.com/health | grep Date

# Test with different timestamp
datafold auth-test --custom-timestamp $(date +%s)
```

### Issue 4: "Nonce validation failed / Replay attack detected"

**Symptoms:**
- Error: `Nonce has already been used`
- Error: `NONCE_VALIDATION_FAILED`
- Error: `Potential replay attack detected`

**Diagnosis:**
```bash
# Check nonce generation
datafold debug-nonce --generate-test

# Check for request duplication
datafold auth-test --nonce-debug --verbose

# Review recent requests
datafold auth-status --request-history
```

**Solutions:**

**A. Application Issue (Duplicate Requests):**
```bash
# Check if application is retrying requests
grep "retry" /var/log/your-app.log

# Ensure unique nonce per request in your code:
# JavaScript/TypeScript:
const nonce = uuidv4(); // Generate fresh UUID for each request

# Python:
import uuid
nonce = str(uuid.uuid4())
```

**B. Clock Skew Causing Nonce Conflicts:**
```bash
# Same as timestamp issues - synchronize clocks
sudo ntpdate -s time.nist.gov

# Check nonce TTL settings
datafold auth-configure --show | grep nonce_ttl
```

**C. Server Nonce Store Issues:**
```bash
# Clear nonce cache if safe to do so
datafold auth-test --clear-nonce-cache

# Check server capacity
datafold server-status --nonce-store-stats
```

### Issue 5: "Public key lookup failed"

**Symptoms:**
- Error: `Public key lookup failed for key_id: xxx`
- Error: `Key not found in storage`
- Error: `PUBLIC_KEY_LOOKUP_FAILED`

**Diagnosis:**
```bash
# Check key registration status
datafold list-registered-keys --server-url https://your-server.com

# Verify key ID matches
datafold auth-profile show default | grep key_id

# Check server key storage
datafold server-status --key-management
```

**Solutions:**

**A. Key Not Registered:**
```bash
# Register your public key
datafold register-public-key \
  --key-id your-key \
  --public-key-file ~/.datafold/keys/your-key.pub \
  --client-id your-client-id

# Verify registration
datafold list-registered-keys --key-id your-key
```

**B. Key ID Mismatch:**
```bash
# Check what key IDs are registered
datafold list-registered-keys

# Update profile to use correct key ID
datafold auth-profile update default --key-id correct-key-id
```

**C. Server Database Issues:**
```bash
# Contact server administrator to check:
# - Database connectivity
# - Key storage integrity  
# - Server configuration

# Test with server health endpoint
curl -v https://your-server.com/health
```

### Issue 6: "Rate limit exceeded"

**Symptoms:**
- Error: `Rate limit exceeded. Please reduce request frequency`
- Error: `RATE_LIMIT_EXCEEDED`
- 429 Too Many Requests responses

**Diagnosis:**
```bash
# Check current rate limit status
datafold auth-status --rate-limits

# Review request patterns
datafold auth-history --last 100 --analyze-rate

# Check server rate limit configuration
datafold server-status --rate-limits
```

**Solutions:**

**A. Implement Request Throttling:**
```bash
# Add delays between requests in scripts
sleep 0.1  # 100ms delay between requests

# Use exponential backoff in applications:
# JavaScript/TypeScript:
const delay = Math.min(1000 * Math.pow(2, retryCount), 30000);
await new Promise(resolve => setTimeout(resolve, delay));
```

**B. Batch Requests Where Possible:**
```bash
# Instead of multiple single requests, use batch endpoints
datafold batch-query --requests requests.json

# Combine operations where possible
datafold mutate --batch-mutations mutations.json
```

**C. Request Server Rate Limit Increase:**
```bash
# Contact server administrator with:
# - Client ID
# - Use case justification  
# - Expected request volume
# - Time patterns
```

### Issue 7: "Configuration error" / Server-side issues

**Symptoms:**
- Error: `Internal server error`
- Error: `Server configuration error`
- Error: `CONFIGURATION_ERROR`

**Diagnosis:**
```bash
# Test server health
curl -v https://your-server.com/health

# Check server logs (if accessible)
# Look for configuration validation errors

# Test with minimal request
datafold auth-test --minimal-request
```

**Solutions:**

**A. Contact Server Administrator:**
Provide the following information:
- Correlation ID from error message
- Client ID and key ID
- Timestamp of failed request
- Full error message
- Steps to reproduce

**B. Temporary Workaround:**
```bash
# If server supports multiple security profiles, try standard profile
datafold auth-configure --security-profile standard

# Use alternative server endpoint if available
datafold auth-profile update default \
  --server-url https://backup-server.com
```

## Environment-Specific Troubleshooting

### Development Environment

**Common Issues:**
- Overly strict security settings
- Clock synchronization problems on VMs
- Self-signed certificates

**Solutions:**
```bash
# Use lenient security profile for development
datafold auth-configure --security-profile lenient

# Accept self-signed certificates (development only!)
datafold auth-configure --allow-self-signed-certs true

# Increase time tolerances
datafold auth-profile update dev \
  --time-window-secs 600 \
  --clock-skew-tolerance 120
```

### Staging Environment

**Common Issues:**
- Mixed security profiles
- Inconsistent key management
- Load balancer interference

**Solutions:**
```bash
# Use standard security profile
datafold auth-configure --security-profile standard

# Test end-to-end through load balancer
datafold auth-test --url https://staging-lb.com/health

# Verify load balancer preserves headers
curl -v -H "Signature-Input: ..." https://staging-lb.com/test
```

### Production Environment

**Common Issues:**
- HSM connectivity problems
- Network time synchronization
- High-availability complications

**Solutions:**
```bash
# Test HSM connectivity
hsm-cli health-check

# Monitor NTP synchronization
chrony tracking

# Test all cluster nodes
for node in node1 node2 node3; do
  datafold auth-test --server-url https://$node.com/health
done
```

## Advanced Debugging

### 1. Enable Debug Logging

**Global Debug Mode:**
```bash
# Enable comprehensive debug logging
export DATAFOLD_LOG_LEVEL=debug
export DATAFOLD_AUTH_DEBUG=true

# Run commands with maximum verbosity
datafold --verbose auth-test --auth-debug --trace
```

**Component-Specific Debugging:**
```bash
# Debug signature generation only
datafold auth-test --debug-signing

# Debug timestamp validation only  
datafold auth-test --debug-timestamp

# Debug nonce handling only
datafold auth-test --debug-nonce

# Debug server communication only
datafold auth-test --debug-network
```

### 2. Signature Analysis Tools

**Inspect Signature Components:**
```bash
# Analyze signature format
datafold inspect-signature \
  --signature-input 'sig1=("@method" "@target-uri");created=1640995200;keyid="test";alg="ed25519"' \
  --signature "base64-signature" \
  --detailed

# Compare with expected format
datafold debug-signature --compare-canonical \
  --expected-file canonical.txt \
  --actual-request request.json
```

**Test Signature Verification:**
```bash
# Verify signature locally
datafold verify-signature \
  --message-file message.txt \
  --signature "base64-signature" \
  --public-key-file public.key \
  --debug

# Test against server expectations
datafold verify-response \
  --url https://server.com/test \
  --method POST \
  --body '{"test": true}' \
  --debug-verification
```

### 3. Network-Level Debugging

**Capture Network Traffic:**
```bash
# Monitor HTTP requests/responses
sudo tcpdump -i any -A 'host your-server.com and port 443'

# Use curl with verbose output
curl -v -H "Signature-Input: ..." -H "Signature: ..." \
  https://your-server.com/api/endpoint

# Trace TLS handshake
openssl s_client -connect your-server.com:443 -servername your-server.com
```

**Proxy Debugging:**
```bash
# Use mitmproxy to inspect requests
mitmproxy --listen-port 8080

# Configure datafold to use proxy
export https_proxy=http://localhost:8080
datafold auth-test --proxy-debug
```

## Performance Troubleshooting

### Slow Authentication Performance

**Diagnosis:**
```bash
# Measure authentication latency
datafold performance-test --auth-benchmark --iterations 100

# Profile signature operations
datafold auth-test --performance-profile

# Check system resources
top -p $(pgrep datafold)
iostat -x 1 10
```

**Solutions:**
```bash
# Enable signature caching
datafold auth-configure --enable-signature-cache true --cache-ttl 60

# Optimize key operations
datafold optimize-keys --cache-private-key

# Use hardware acceleration if available
datafold auth-configure --use-hardware-crypto true
```

### Memory Usage Issues

**Diagnosis:**
```bash
# Check memory usage
datafold auth-status --memory-stats

# Monitor nonce store size
datafold auth-status --nonce-store-stats

# Profile memory usage
valgrind --tool=massif datafold auth-test
```

**Solutions:**
```bash
# Reduce nonce store size
datafold auth-configure --max-nonce-store-size 10000

# Enable nonce cleanup
datafold auth-configure --enable-nonce-cleanup true

# Adjust cache sizes
datafold auth-configure --max-signature-cache-size 1000
```

## Getting Help

### 1. Collect Diagnostic Information

Before contacting support, collect:

```bash
#!/bin/bash
# Diagnostic information collector
REPORT_FILE="datafold-diagnostic-$(date +%Y%m%d-%H%M%S).txt"

echo "DataFold Diagnostic Report" > $REPORT_FILE
echo "Generated: $(date)" >> $REPORT_FILE
echo "================================" >> $REPORT_FILE

# System information
echo -e "\n--- System Information ---" >> $REPORT_FILE
uname -a >> $REPORT_FILE
date >> $REPORT_FILE

# DataFold configuration
echo -e "\n--- DataFold Configuration ---" >> $REPORT_FILE
datafold --version >> $REPORT_FILE 2>&1
datafold auth-configure --show >> $REPORT_FILE 2>&1

# Authentication status
echo -e "\n--- Authentication Status ---" >> $REPORT_FILE
datafold auth-status --verbose >> $REPORT_FILE 2>&1

# Profile information
echo -e "\n--- Profiles ---" >> $REPORT_FILE
datafold auth-profile list --verbose >> $REPORT_FILE 2>&1

# Key information (public info only)
echo -e "\n--- Keys ---" >> $REPORT_FILE
datafold list-keys --public-only >> $REPORT_FILE 2>&1

# Test results
echo -e "\n--- Test Results ---" >> $REPORT_FILE
datafold auth-test --debug >> $REPORT_FILE 2>&1

echo "Diagnostic report saved to: $REPORT_FILE"
echo "Send this file when requesting support"
```

### 2. Support Channels

- **GitHub Issues**: For bugs and feature requests
- **Documentation**: Check latest docs at https://docs.datafold.dev
- **Community Forum**: Community support and discussions
- **Enterprise Support**: For enterprise customers with SLA

### 3. Emergency Procedures

**For Production Outages:**

1. **Immediate Assessment:**
   ```bash
   # Check if issue is widespread
   datafold auth-test --all-profiles
   
   # Test alternative endpoints
   datafold auth-test --backup-servers
   ```

2. **Temporary Workarounds:**
   ```bash
   # Switch to backup authentication profile if available
   datafold auth-profile set-default backup
   
   # Use emergency key if configured
   datafold auth-test --key-id emergency-key
   ```

3. **Escalation:**
   - Contact technical lead immediately
   - Engage vendor support if applicable
   - Document incident timeline
   - Prepare rollback plan if needed

Remember: Authentication is mandatory in DataFold. There are no workarounds to bypass security - issues must be resolved through proper authentication configuration.