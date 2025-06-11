# DataFold Signature Authentication Migration Troubleshooting Guide

**Version:** 1.0  
**Date:** June 9, 2025  
**Task ID:** T11.6-3  

## Quick Reference

### Emergency Contacts
- **Migration Support:** migration-support@datafold.com
- **Emergency Hotline:** +1-800-DATAFOLD
- **Slack Channel:** #datafold-migration-support

### Critical Commands
```bash
# Emergency rollback
kubectl patch configmap datafold-config --patch '{"data":{"SIGNATURE_AUTH_REQUIRED":"false"}}'

# Check migration status
datafold migration status --verbose

# Test authentication
datafold auth-test --debug
```

---

## Common Migration Issues

### 1. Authentication Failures

#### Issue: "Signature verification failed"

**Symptoms:**
- 401 Unauthorized responses from DataFold API
- "Invalid signature" error messages
- Authentication working in development but failing in production

**Diagnostic Steps:**
```bash
# Check key registration
datafold crypto list-keys --client-id your-client-id

# Verify signature generation
datafold auth-test --client-id your-client-id --debug

# Check server logs
kubectl logs -l app=datafold | grep "signature.*fail"

# Test with verbose output
datafold api get /api/health --debug --show-headers
```

**Common Causes & Solutions:**

| Root Cause | Error Message | Solution |
|------------|---------------|----------|
| **Unregistered Key** | "Public key not found for client" | Re-register public key with server |
| **Wrong Key ID** | "Key ID mismatch" | Verify key ID matches registration |
| **Clock Skew** | "Timestamp outside tolerance window" | Sync system clocks with NTP |
| **Expired Timestamp** | "Request timestamp too old" | Check timestamp generation logic |
| **Invalid Signature Format** | "Malformed signature header" | Update SDK to compatible version |
| **Missing Headers** | "Required signature headers missing" | Check proxy/gateway configuration |

**Step-by-Step Resolution:**

1. **Verify Key Registration**
   ```bash
   # Check if key is registered
   datafold crypto get-key --client-id your-app --key-id your-key
   
   # If not found, re-register
   datafold crypto register-key \
     --client-id your-app \
     --key-id your-key \
     --public-key-file path/to/public.key
   ```

2. **Test Signature Generation**
   ```python
   # Python test script
   from datafold_sdk import DataFoldClient
   
   client = DataFoldClient(
       base_url='your-server-url',
       client_id='your-app',
       authentication={
           'type': 'signature',
           'private_key': 'your-private-key',
           'key_id': 'your-key'
       }
   )
   
   try:
       result = client.get_health()
       print("✅ Authentication successful")
   except Exception as e:
       print(f"❌ Authentication failed: {e}")
   ```

3. **Check Network Configuration**
   ```bash
   # Test direct connection (bypassing proxies)
   curl -v https://your-datafold-server/api/health
   
   # Check if signature headers are preserved
   curl -v https://your-datafold-server/api/health \
     -H "Signature: test" -H "Signature-Input: test"
   ```

#### Issue: "High authentication failure rate"

**Symptoms:**
- Multiple clients failing authentication simultaneously
- Sudden spike in 401 errors
- Authentication was working, then stopped

**Investigation Steps:**
```bash
# Check server health
kubectl get pods -l app=datafold
kubectl describe pod datafold-xyz

# Check authentication metrics
curl http://prometheus:9090/api/v1/query?query=datafold_auth_failure_rate

# Check for recent changes
kubectl rollout history deployment/datafold
```

**Common Scenarios:**

1. **Server Configuration Change**
   ```bash
   # Check current config
   kubectl get configmap datafold-config -o yaml
   
   # Compare with previous version
   kubectl rollout history deployment/datafold --revision=1
   ```

2. **Clock Synchronization Issues**
   ```bash
   # Check time sync on all servers
   timedatectl status
   ntpq -p
   
   # Fix NTP synchronization
   sudo systemctl restart ntp
   ```

3. **Database Connectivity Issues**
   ```bash
   # Check database connections
   kubectl logs -l app=datafold | grep -i database
   
   # Test database connectivity
   kubectl exec -it datafold-pod -- nc -zv database-host 5432
   ```

### 2. Performance Issues

#### Issue: "High authentication latency"

**Symptoms:**
- API requests taking much longer than before
- Timeout errors during authentication
- Performance degradation after migration

**Performance Analysis:**
```python
# Performance measurement script
import time
import statistics
from datafold_sdk import DataFoldClient

def measure_auth_performance():
    client = DataFoldClient(
        base_url='your-server',
        client_id='perf-test',
        authentication={
            'type': 'signature',
            'private_key': 'your-key',
            'key_id': 'test-key'
        }
    )
    
    latencies = []
    for i in range(50):
        start = time.time()
        try:
            client.get_health()
            latencies.append(time.time() - start)
        except Exception as e:
            print(f"Request {i} failed: {e}")
    
    if latencies:
        print(f"Mean: {statistics.mean(latencies)*1000:.1f}ms")
        print(f"95th percentile: {statistics.quantiles(latencies, n=20)[18]*1000:.1f}ms")
        print(f"Max: {max(latencies)*1000:.1f}ms")

measure_auth_performance()
```

**Performance Optimization:**
```typescript
// Optimized client configuration
const client = new DataFoldClient({
    baseUrl: 'https://api.datafold.com',
    authentication: {
        type: 'signature',
        privateKey: process.env.PRIVATE_KEY,
        keyId: 'optimized-key',
        // Performance optimizations
        cacheSignatures: true,
        signatureCacheTTL: 300,
        includeHeaders: ['content-type'], // Minimal headers
    },
    httpClient: {
        connectionPoolSize: 20,
        keepAliveTimeout: 30000,
        requestTimeout: 15000,
    }
});
```

#### Issue: "Memory usage increase after migration"

**Investigation:**
```bash
# Monitor memory usage
kubectl top pods -l app=datafold

# Check for memory leaks
kubectl exec -it datafold-pod -- ps aux | grep datafold
kubectl exec -it datafold-pod -- cat /proc/meminfo

# Review signature verification cache
kubectl logs -l app=datafold | grep -i cache
```

**Memory Optimization:**
```rust
// Server-side optimization
let signature_config = SignatureAuthConfig {
    nonce_cache_size: 10000,        // Reduce if memory constrained
    nonce_cleanup_interval: 300,    // More frequent cleanup
    key_cache_size: 1000,          // Limit key cache
    verification_cache_ttl: 300,    // Shorter cache TTL
};
```

### 3. Integration Issues

#### Issue: "Third-party systems cannot authenticate"

**Symptoms:**
- Partner integrations failing
- Webhook deliveries failing
- API gateway returning auth errors

**Partner Integration Support:**
```python
# Partner diagnostic script
def diagnose_partner_integration(partner_id):
    print(f"🔍 Diagnosing partner integration: {partner_id}")
    
    # Check if partner supports signature auth
    partner_config = get_partner_config(partner_id)
    
    if not partner_config.get('signature_auth_supported'):
        print("⚠️  Partner does not support signature authentication")
        print("📧 Sending migration notification to partner...")
        send_migration_notification(partner_id)
        return False
    
    # Test partner's implementation
    try:
        test_partner_auth(partner_id)
        print("✅ Partner integration working")
        return True
    except Exception as e:
        print(f"❌ Partner integration failing: {e}")
        return False

# Usage
diagnose_partner_integration('partner-xyz')
```

**Integration Fallback Strategy:**
```python
# Hybrid client for partner migration
class PartnerHybridClient:
    def __init__(self, partner_config):
        self.partner_id = partner_config['partner_id']
        self.migration_deadline = partner_config.get('migration_deadline')
        
        # Create both signature and legacy clients
        self.signature_client = self._create_signature_client(partner_config)
        self.legacy_client = self._create_legacy_client(partner_config)
    
    def make_request(self, *args, **kwargs):
        # Try signature auth first
        try:
            return self.signature_client.request(*args, **kwargs)
        except AuthenticationError:
            # Fallback to legacy auth if signature fails
            if self._legacy_auth_allowed():
                return self.legacy_client.request(*args, **kwargs)
            raise
    
    def _legacy_auth_allowed(self):
        # Check if legacy auth is still allowed
        if self.migration_deadline:
            return datetime.now() < self.migration_deadline
        return False
```

#### Issue: "API Gateway signature validation failures"

**Common API Gateway Issues:**

1. **Header Modification**
   ```nginx
   # Nginx configuration to preserve signature headers
   location /api/ {
       proxy_pass http://datafold-backend;
       
       # Preserve all signature-related headers
       proxy_set_header Signature $http_signature;
       proxy_set_header Signature-Input $http_signature_input;
       proxy_set_header Authorization $http_authorization;
       
       # Don't modify headers that are part of signature
       proxy_set_header Host $host;
   }
   ```

2. **Load Balancer Configuration**
   ```yaml
   # HAProxy configuration
   backend datafold-api
       balance roundrobin
       option httpchk GET /health
       
       # Preserve signature headers
       http-request set-header X-Forwarded-For %[src]
       http-request preserve Signature
       http-request preserve Signature-Input
   ```

### 4. Configuration Issues

#### Issue: "Environment-specific configuration problems"

**Configuration Validation Script:**
```bash
#!/bin/bash
# validate-config.sh

echo "🔧 Validating DataFold configuration..."

# Check environment variables
required_vars=("DATAFOLD_PRIVATE_KEY" "DATAFOLD_KEY_ID" "DATAFOLD_CLIENT_ID")
for var in "${required_vars[@]}"; do
    if [ -z "${!var}" ]; then
        echo "❌ Missing required environment variable: $var"
        exit 1
    else
        echo "✅ $var is set"
    fi
done

# Validate key format
if ! echo "$DATAFOLD_PRIVATE_KEY" | base64 -d > /dev/null 2>&1; then
    echo "❌ DATAFOLD_PRIVATE_KEY is not valid base64"
    exit 1
fi

# Test connectivity
if ! curl -f "$DATAFOLD_SERVER_URL/health" > /dev/null 2>&1; then
    echo "❌ Cannot connect to DataFold server: $DATAFOLD_SERVER_URL"
    exit 1
fi

echo "✅ Configuration validation passed"
```

**Environment-Specific Issues:**

| Environment | Common Issues | Solutions |
|-------------|---------------|-----------|
| **Development** | • Wrong server URL<br/>• Test keys not registered | • Verify server URL<br/>• Register development keys |
| **Staging** | • Production keys in staging<br/>• Clock sync issues | • Use staging-specific keys<br/>• Sync staging clocks |
| **Production** | • Debug mode enabled<br/>• Relaxed security settings | • Disable debug mode<br/>• Tighten security settings |

### 5. Migration Process Issues

#### Issue: "Migration stalled or incomplete"

**Migration Progress Check:**
```python
# Migration status checker
def check_migration_progress():
    total_clients = get_total_client_count()
    signature_clients = get_signature_auth_client_count()
    legacy_clients = get_legacy_auth_client_count()
    
    progress_percentage = (signature_clients / total_clients) * 100
    
    print(f"Migration Progress: {progress_percentage:.1f}%")
    print(f"Signature Auth Clients: {signature_clients}")
    print(f"Legacy Auth Clients: {legacy_clients}")
    
    if progress_percentage < 90 and days_since_migration_start() > 30:
        print("⚠️  Migration progress is behind schedule")
        identify_stalled_clients()
    
    return progress_percentage

def identify_stalled_clients():
    stalled_clients = get_clients_without_signature_auth()
    for client in stalled_clients:
        print(f"📋 Stalled client: {client['client_id']}")
        print(f"   Last seen: {client['last_activity']}")
        print(f"   Contact: {client['contact_email']}")
        send_migration_reminder(client)
```

#### Issue: "Rollback required during migration"

**Emergency Rollback Procedure:**
```bash
#!/bin/bash
# emergency-rollback.sh

echo "🚨 EXECUTING EMERGENCY ROLLBACK"

# Step 1: Disable signature auth requirement
kubectl patch configmap datafold-config --patch '{
  "data": {
    "SIGNATURE_AUTH_REQUIRED": "false",
    "SIGNATURE_AUTH_MODE": "hybrid"
  }
}'

# Step 2: Restart services
kubectl rollout restart deployment/datafold

# Step 3: Wait for rollout
kubectl rollout status deployment/datafold --timeout=300s

# Step 4: Verify health
for i in {1..30}; do
  if curl -f http://datafold-service/health > /dev/null 2>&1; then
    echo "✅ Rollback successful - system is healthy"
    break
  fi
  sleep 10
done

# Step 5: Notify stakeholders
curl -X POST "$SLACK_WEBHOOK_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "🚨 DataFold Emergency Rollback Executed\nSignature authentication disabled\nSystem operating in hybrid mode"
  }'
```

---

## Advanced Troubleshooting

### Debug Mode Activation

**Server Debug Mode:**
```rust
// Enable debug logging for signature auth
let debug_config = SignatureAuthConfig {
    debug_mode: true,
    log_all_requests: true,
    log_signature_details: true,
    ..Default::default()
};
```

**Client Debug Mode:**
```javascript
// JavaScript client debug mode
const client = new DataFoldClient({
    baseUrl: 'https://api.datafold.com',
    debug: true,
    authentication: {
        type: 'signature',
        privateKey: process.env.PRIVATE_KEY,
        keyId: 'debug-key',
        debug: true  // Log signature generation details
    }
});
```

### Log Analysis Tools

**Signature Auth Log Parser:**
```python
#!/usr/bin/env python3
# log-analyzer.py

import re
import json
from collections import defaultdict

def analyze_auth_logs(log_file):
    """Analyze authentication logs for patterns"""
    
    failures_by_client = defaultdict(int)
    failures_by_reason = defaultdict(int)
    auth_latencies = []
    
    with open(log_file, 'r') as f:
        for line in f:
            if 'auth_failure' in line:
                try:
                    log_entry = json.loads(line)
                    client_id = log_entry.get('client_id', 'unknown')
                    reason = log_entry.get('failure_reason', 'unknown')
                    
                    failures_by_client[client_id] += 1
                    failures_by_reason[reason] += 1
                    
                except json.JSONDecodeError:
                    continue
            
            elif 'auth_success' in line:
                try:
                    log_entry = json.loads(line)
                    latency = log_entry.get('auth_latency_ms', 0)
                    auth_latencies.append(latency)
                except json.JSONDecodeError:
                    continue
    
    # Generate report
    print("🔍 Authentication Log Analysis")
    print("=" * 40)
    
    print("\nTop Failing Clients:")
    for client, count in sorted(failures_by_client.items(), 
                               key=lambda x: x[1], reverse=True)[:10]:
        print(f"  {client}: {count} failures")
    
    print("\nFailure Reasons:")
    for reason, count in sorted(failures_by_reason.items(), 
                               key=lambda x: x[1], reverse=True):
        print(f"  {reason}: {count} occurrences")
    
    if auth_latencies:
        avg_latency = sum(auth_latencies) / len(auth_latencies)
        max_latency = max(auth_latencies)
        print(f"\nAuth Performance:")
        print(f"  Average latency: {avg_latency:.1f}ms")
        print(f"  Max latency: {max_latency:.1f}ms")

# Usage
analyze_auth_logs('/var/log/datafold/auth.log')
```

### Performance Profiling

**Authentication Performance Profiler:**
```python
# auth-profiler.py
import time
import threading
from concurrent.futures import ThreadPoolExecutor
from datafold_sdk import DataFoldClient

class AuthProfiler:
    def __init__(self, base_url, client_id, private_key, key_id):
        self.client = DataFoldClient(
            base_url=base_url,
            client_id=client_id,
            authentication={
                'type': 'signature',
                'private_key': private_key,
                'key_id': key_id
            }
        )
        self.results = []
    
    def profile_single_request(self):
        """Profile a single authenticated request"""
        start_time = time.time()
        try:
            self.client.get_health()
            latency = time.time() - start_time
            self.results.append(('success', latency))
            return latency
        except Exception as e:
            latency = time.time() - start_time
            self.results.append(('failure', latency, str(e)))
            return latency
    
    def profile_concurrent_requests(self, num_threads=10, requests_per_thread=10):
        """Profile concurrent authenticated requests"""
        print(f"🔄 Running {num_threads} threads with {requests_per_thread} requests each...")
        
        with ThreadPoolExecutor(max_workers=num_threads) as executor:
            futures = []
            for _ in range(num_threads):
                for _ in range(requests_per_thread):
                    futures.append(executor.submit(self.profile_single_request))
            
            # Wait for all requests to complete
            latencies = [future.result() for future in futures]
        
        self.analyze_results()
    
    def analyze_results(self):
        """Analyze profiling results"""
        successes = [r for r in self.results if r[0] == 'success']
        failures = [r for r in self.results if r[0] == 'failure']
        
        total_requests = len(self.results)
        success_rate = len(successes) / total_requests * 100
        
        if successes:
            success_latencies = [r[1] for r in successes]
            avg_latency = sum(success_latencies) / len(success_latencies)
            max_latency = max(success_latencies)
            min_latency = min(success_latencies)
        else:
            avg_latency = max_latency = min_latency = 0
        
        print(f"\n📊 Profiling Results:")
        print(f"Total Requests: {total_requests}")
        print(f"Success Rate: {success_rate:.1f}%")
        print(f"Average Latency: {avg_latency*1000:.1f}ms")
        print(f"Min Latency: {min_latency*1000:.1f}ms")
        print(f"Max Latency: {max_latency*1000:.1f}ms")
        
        if failures:
            print(f"\n❌ Failures ({len(failures)}):")
            failure_reasons = {}
            for failure in failures:
                reason = failure[2] if len(failure) > 2 else 'unknown'
                failure_reasons[reason] = failure_reasons.get(reason, 0) + 1
            
            for reason, count in failure_reasons.items():
                print(f"  {reason}: {count}")

# Usage
profiler = AuthProfiler(
    base_url='https://api.datafold.com',
    client_id='profiler-test',
    private_key='your-private-key',
    key_id='test-key'
)
profiler.profile_concurrent_requests(num_threads=5, requests_per_thread=20)
```

---

## Getting Help

### Self-Service Resources

1. **Documentation**
   - [Migration Guide](migration-guide.md)
   - [Authentication Overview](../api/authentication/overview.md)
   - [Troubleshooting Guide](../api/authentication/troubleshooting.md)

2. **Diagnostic Tools**
   - `datafold auth-test --debug`
   - `datafold migration status`
   - `datafold crypto audit-keys`

3. **Community Resources**
   - [Community Forum](https://community.datafold.com)
   - [Knowledge Base](https://kb.datafold.com)
   - [FAQ](https://docs.datafold.com/faq)

### Support Escalation

#### Level 1: Self-Service (0-2 hours)
- Review this troubleshooting guide
- Check documentation and FAQ
- Run diagnostic tools
- Search community forum

#### Level 2: Community Support (2-24 hours)
- Post in community forum
- Join Slack support channel
- Check knowledge base articles
- Review GitHub issues

#### Level 3: Technical Support (24-48 hours)
- Email: migration-support@datafold.com
- Include diagnostic information
- Provide system configuration
- Attach relevant logs

#### Level 4: Emergency Support (15 minutes)
- Phone: +1-800-DATAFOLD
- For production outages only
- Have incident details ready
- System admin access required

### Support Request Template

```markdown
**Subject:** DataFold Migration Issue - [Brief Description]

**Environment:**
- DataFold Server Version: 
- Client SDK Version: 
- Environment: [Development/Staging/Production]
- Operating System: 

**Issue Description:**
[Detailed description of the problem]

**Steps to Reproduce:**
1. 
2. 
3. 

**Expected Behavior:**
[What you expected to happen]

**Actual Behavior:**
[What actually happened]

**Error Messages:**
```
[Paste error messages here]
```

**Diagnostic Information:**
```bash
# Output of diagnostic commands
datafold auth-test --debug
datafold migration status
```

**Additional Context:**
[Any additional information that might be helpful]
```

---

**Remember:** When in doubt, don't hesitate to reach out for help. The DataFold team is here to ensure your migration is successful! 🚀