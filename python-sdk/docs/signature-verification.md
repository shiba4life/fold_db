# Signature Verification for DataFold Python SDK

This document provides comprehensive guidance on using the signature verification utilities in the DataFold Python SDK, which implement RFC 9421 HTTP Message Signatures verification.

## Overview

The DataFold Python SDK includes robust signature verification capabilities that allow you to:

- Verify signed HTTP requests and responses
- Validate signature format and integrity
- Apply configurable verification policies
- Debug signature issues with inspection tools
- Integrate verification into web frameworks
- Batch verify multiple signatures efficiently

## Quick Start

### Basic Verification

```python
import asyncio
from datafold_sdk.verification import (
    VerificationConfig,
    create_verifier,
    VerifiableResponse
)
from datafold_sdk.crypto.ed25519 import generate_key_pair

# Generate or load public keys
key_pair = generate_key_pair()
public_keys = {'server-key': key_pair.public_key}

# Create verification configuration
config = VerificationConfig(
    default_policy='standard',
    public_keys=public_keys
)

# Create verifier
verifier = create_verifier(config)

# Create response to verify
response = VerifiableResponse(
    status=200,
    headers={
        'signature-input': 'sig1=("@method" "@target-uri");created=1640995200;keyid="server-key";alg="ed25519";nonce="uuid"',
        'signature': 'sig1=:deadbeef12345678...:',
        'content-type': 'application/json'
    },
    body='{"result": "success"}',
    url='https://api.example.com/data',
    method='GET'
)

# Verify signature
async def verify_example():
    result = await verifier.verify(response, response.headers)
    
    print(f"Status: {result.status.value}")
    print(f"Signature Valid: {result.signature_valid}")
    print(f"All Checks Passed: {all(result.checks.values())}")

asyncio.run(verify_example())
```

## Verification Configuration

### VerificationConfig

The `VerificationConfig` class controls verification behavior:

```python
from datafold_sdk.verification import VerificationConfig

config = VerificationConfig(
    default_policy='standard',           # Default verification policy
    policies={},                         # Custom policies
    public_keys={                        # Known public keys
        'key-1': key1_bytes,
        'key-2': key2_bytes
    },
    trusted_key_sources=[],              # External key sources
    performance_monitoring={             # Performance settings
        'enabled': True,
        'max_verification_time': 50      # milliseconds
    }
)
```

### Public Key Management

```python
# Add public key
verifier.add_public_key('new-key', public_key_bytes)

# Remove public key
verifier.remove_public_key('old-key')

# Update configuration
new_config = VerificationConfig(
    public_keys={'updated-key': new_key}
)
verifier.update_config(new_config)
```

## Verification Policies

### Built-in Policies

The SDK provides four built-in verification policies:

#### Strict Policy
- Maximum security with comprehensive checks
- Timestamp age limit: 5 minutes
- Requires: `@method`, `@target-uri`, `content-type`, `content-digest`
- All security features enabled

#### Standard Policy (Default)
- Balanced security for most applications
- Timestamp age limit: 15 minutes
- Requires: `@method`, `@target-uri`
- Good balance of security and usability

#### Lenient Policy
- Relaxed verification for development/testing
- Timestamp age limit: 1 hour
- Requires: `@method` only
- Minimal security requirements

#### Legacy Policy
- Backward compatibility for older signatures
- No timestamp or nonce verification
- No required components
- Use only when necessary

### Using Policies

```python
# Use built-in policy
result = await verifier.verify(message, headers, policy='strict')

# Set default policy
config = VerificationConfig(default_policy='strict')
```

### Custom Policies

```python
from datafold_sdk.verification.policies import (
    create_verification_policy,
    VerificationRules
)

# Create custom policy
custom_policy = create_verification_policy(
    name='api-gateway',
    description='Policy for API Gateway',
    verify_timestamp=True,
    max_timestamp_age=300,
    verify_nonce=True,
    verify_content_digest=True,
    required_components=['@method', '@target-uri', 'content-type'],
    custom_rules=[
        VerificationRules.timestamp_freshness(300),
        VerificationRules.required_headers(['content-type']),
        VerificationRules.content_type_consistency(),
    ]
)

# Register custom policy
from datafold_sdk.verification.policies import register_verification_policy
register_verification_policy(custom_policy)

# Use custom policy
config = VerificationConfig(default_policy='api-gateway')
```

## Verification Results

### VerificationResult Structure

```python
@dataclass
class VerificationResult:
    status: VerificationStatus          # 'valid', 'invalid', 'unknown', 'error'
    signature_valid: bool               # Cryptographic validity
    checks: Dict[str, bool]             # Individual check results
    diagnostics: VerificationDiagnostics  # Detailed analysis
    performance: Dict[str, Any]         # Performance metrics
    error: Optional[Dict[str, Any]]     # Error details if failed
```

### Individual Checks

```python
result = await verifier.verify(message, headers)

# Check individual validation results
print(f"Format Valid: {result.checks['format_valid']}")
print(f"Crypto Valid: {result.checks['cryptographic_valid']}")
print(f"Timestamp Valid: {result.checks['timestamp_valid']}")
print(f"Nonce Valid: {result.checks['nonce_valid']}")
print(f"Content Digest Valid: {result.checks['content_digest_valid']}")
print(f"Component Coverage Valid: {result.checks['component_coverage_valid']}")
print(f"Custom Rules Valid: {result.checks['custom_rules_valid']}")
```

### Diagnostics

```python
# Access detailed diagnostics
diagnostics = result.diagnostics

# Signature analysis
sig_info = diagnostics.signature_analysis
print(f"Algorithm: {sig_info['algorithm']}")
print(f"Key ID: {sig_info['key_id']}")
print(f"Age: {sig_info['age']} seconds")

# Security analysis
security = diagnostics.security_analysis
print(f"Security Level: {security['security_level']}")
print(f"Concerns: {security['concerns']}")
print(f"Recommendations: {security['recommendations']}")
```

## Inspection and Debugging

### Signature Inspector

```python
from datafold_sdk.verification import RFC9421Inspector

inspector = RFC9421Inspector()

# Inspect signature format
format_analysis = inspector.inspect_format(headers)
print(f"RFC 9421 Compliant: {format_analysis.is_valid_rfc9421}")

for issue in format_analysis.issues:
    print(f"{issue.severity}: {issue.message}")

# Analyze components
signature_data = extract_signature_data(headers)
component_analysis = inspector.analyze_components(signature_data)
print(f"Security Level: {component_analysis.security_assessment.level}")

# Validate parameters
param_validation = inspector.validate_parameters(signature_data.params)
print(f"All Parameters Valid: {param_validation.all_valid}")
```

### Quick Diagnostic Tools

```python
from datafold_sdk.verification.utils import (
    validate_signature_format,
    quick_diagnostic
)

# Quick format check
is_valid, issues = validate_signature_format(headers)
print(f"Format Valid: {is_valid}")

# Quick diagnostic report
report = quick_diagnostic(headers)
print(report)
```

### Diagnostic Reports

```python
# Generate comprehensive diagnostic report
result = await verifier.verify(message, headers)
report = inspector.generate_diagnostic_report(result)
print(report)
```

## Framework Integration

### Django Middleware

```python
from datafold_sdk.verification.middleware import create_django_verification_middleware

# Create middleware
config = RequestVerificationConfig(
    verification_config=verification_config,
    default_policy='standard',
    reject_invalid=True
)

DjangoMiddleware = create_django_verification_middleware(config)

# Add to Django settings
MIDDLEWARE = [
    'path.to.DjangoMiddleware',
    # ... other middleware
]

# Access verification result in views
def my_view(request):
    verification = request.signature_verification
    if verification['valid']:
        # Process verified request
        pass
```

### Flask Integration

```python
from datafold_sdk.verification.middleware import create_flask_verification_middleware
from flask import Flask

app = Flask(__name__)

# Create and register middleware
flask_middleware = create_flask_verification_middleware(config)
app.before_request(flask_middleware)

@app.route('/api/data')
def api_endpoint():
    from flask import g
    verification = g.signature_verification
    if verification['valid']:
        return {'result': 'success'}
    else:
        return {'error': 'Invalid signature'}, 401
```

### FastAPI Integration

```python
from datafold_sdk.verification.middleware import create_fastapi_verification_middleware
from fastapi import FastAPI, Request

app = FastAPI()

# Create middleware
fastapi_middleware = create_fastapi_verification_middleware(config)

@app.middleware("http")
async def signature_verification_middleware(request: Request, call_next):
    return await fastapi_middleware(request, call_next)

@app.get("/api/data")
async def api_endpoint(request: Request):
    verification = request.state.signature_verification
    if verification['valid']:
        return {'result': 'success'}
```

### Response Verification

```python
from datafold_sdk.verification.middleware import (
    ResponseVerificationConfig,
    create_response_verification_middleware
)

# Create response middleware
response_config = ResponseVerificationConfig(
    verification_config=verification_config,
    default_policy='standard',
    throw_on_failure=False
)

response_middleware = create_response_verification_middleware(response_config)

# Apply to response
verified_response = await response_middleware(response)

# Check verification result
if hasattr(verified_response, '_verification_result'):
    result = verified_response._verification_result
    print(f"Response signature verified: {result.signature_valid}")
```

## Batch Verification

### Batch Processing

```python
from datafold_sdk.verification.middleware import create_batch_verifier

batch_verifier = create_batch_verifier(verification_config)

# Prepare batch items
items = [
    {
        'message': request1,
        'headers': headers1,
        'policy': 'standard'
    },
    {
        'message': request2,
        'headers': headers2,
        'policy': 'strict'
    }
]

# Verify batch
results = await batch_verifier.verify_batch(items)

# Get statistics
stats = batch_verifier.get_batch_stats(results)
print(f"Success Rate: {stats['success_rate']:.1%}")
print(f"Average Time: {stats['average_time']:.2f}ms")
```

## Advanced Features

### Custom Key Sources

```python
from datafold_sdk.verification.types import SimpleKeySource

# HTTP key source
async def fetch_key_from_server(key_id: str) -> Optional[bytes]:
    # Implement key fetching logic
    response = await http_client.get(f'https://keyserver.com/keys/{key_id}')
    if response.status == 200:
        return base64.b64decode(response.json()['public_key'])
    return None

key_source = SimpleKeySource(
    name='key-server',
    type='function',
    source=fetch_key_from_server,
    cache_ttl=3600
)

config = VerificationConfig(
    trusted_key_sources=[key_source]
)
```

### Performance Monitoring

```python
config = VerificationConfig(
    performance_monitoring={
        'enabled': True,
        'max_verification_time': 50  # milliseconds
    }
)

result = await verifier.verify(message, headers)

# Check performance
print(f"Total Time: {result.performance['total_time']:.2f}ms")
print("Step Timings:")
for step, time_ms in result.performance['step_timings'].items():
    print(f"  {step}: {time_ms:.2f}ms")
```

### Async/Await Support

The verification utilities are fully async-compatible:

```python
# All verification operations support async/await
result = await verifier.verify(message, headers)
results = await batch_verifier.verify_batch(items)
processed_response = await response_middleware(response)

# Custom rules can be async too
async def async_custom_rule(context):
    # Perform async validation
    external_result = await external_validator(context.signature_data)
    return VerificationRuleResult(passed=external_result.valid)
```

## Error Handling

### Verification Errors

```python
from datafold_sdk.verification import VerificationError

try:
    result = await verifier.verify(message, headers)
except VerificationError as e:
    print(f"Verification failed: {e.message}")
    print(f"Error code: {e.code}")
    print(f"Details: {e.details}")
```

### Error Codes

Common verification error codes:

- `MISSING_SIGNATURE_INPUT`: Signature-Input header not found
- `MISSING_SIGNATURE`: Signature header not found
- `INVALID_SIGNATURE_FORMAT`: Malformed signature headers
- `PUBLIC_KEY_NOT_FOUND`: Public key not available
- `CRYPTOGRAPHIC_VERIFICATION_FAILED`: Signature verification failed
- `TIMESTAMP_VALIDATION_FAILED`: Timestamp check failed
- `COMPONENT_COVERAGE_FAILED`: Required components missing

## Best Practices

### Security Recommendations

1. **Use Strict Policies in Production**
   ```python
   # Production configuration
   config = VerificationConfig(default_policy='strict')
   ```

2. **Implement Proper Key Management**
   ```python
   # Secure key storage and rotation
   config = VerificationConfig(
       public_keys=load_keys_from_secure_storage(),
       trusted_key_sources=[secure_key_source]
   )
   ```

3. **Monitor Performance**
   ```python
   # Enable performance monitoring
   config = VerificationConfig(
       performance_monitoring={
           'enabled': True,
           'max_verification_time': 50
       }
   )
   ```

4. **Use Content Digest Verification**
   ```python
   # Always verify content integrity for sensitive data
   policy = create_verification_policy(
       name='secure-api',
       verify_content_digest=True,
       required_components=['@method', '@target-uri', 'content-digest']
   )
   ```

### Development and Testing

1. **Use Lenient Policies for Development**
   ```python
   # Development configuration
   config = VerificationConfig(default_policy='lenient')
   ```

2. **Leverage Inspection Tools**
   ```python
   # Debug signature issues
   inspector = RFC9421Inspector()
   analysis = inspector.inspect_format(headers)
   ```

3. **Test with Various Scenarios**
   ```python
   # Test different verification scenarios
   test_policies = ['strict', 'standard', 'lenient']
   for policy in test_policies:
       result = await verifier.verify(message, headers, policy=policy)
       assert result.status == expected_status[policy]
   ```

## Troubleshooting

### Common Issues

**Cryptographic Verification Fails**
- Verify public key matches the signing key
- Check signature format (must be hex-encoded)
- Ensure canonical message reconstruction is correct

**Timestamp Validation Fails**
- Check system clock synchronization
- Adjust `max_timestamp_age` in policy
- Verify timestamp format (Unix seconds)

**Component Coverage Fails**
- Check required components in policy
- Verify all required headers are present
- Ensure component names match exactly

**Format Validation Fails**
- Use signature inspector to identify issues
- Check RFC 9421 compliance
- Verify header syntax and encoding

### Debug Steps

1. **Inspect Signature Format**
   ```python
   analysis = inspector.inspect_format(headers)
   for issue in analysis.issues:
       print(f"{issue.severity}: {issue.message}")
   ```

2. **Check Individual Verification Steps**
   ```python
   result = await verifier.verify(message, headers)
   for check, passed in result.checks.items():
       if not passed:
           print(f"Failed check: {check}")
   ```

3. **Review Diagnostics**
   ```python
   report = inspector.generate_diagnostic_report(result)
   print(report)
   ```

## API Reference

For complete API documentation, see the individual module documentation:

- [`datafold_sdk.verification`](../src/datafold_sdk/verification/) - Main verification module
- [`datafold_sdk.verification.verifier`](../src/datafold_sdk/verification/verifier.py) - Core verifier implementation
- [`datafold_sdk.verification.policies`](../src/datafold_sdk/verification/policies.py) - Verification policies
- [`datafold_sdk.verification.inspector`](../src/datafold_sdk/verification/inspector.py) - Inspection utilities
- [`datafold_sdk.verification.middleware`](../src/datafold_sdk/verification/middleware.py) - Framework middleware
- [`datafold_sdk.verification.utils`](../src/datafold_sdk/verification/utils.py) - Utility functions

## Examples

Complete examples are available in the [`examples/`](../examples/) directory:

- [`verification_example.py`](../examples/verification_example.py) - Comprehensive verification examples
- [`enhanced_http_client_example.py`](../examples/enhanced_http_client_example.py) - HTTP client integration

## Performance Considerations

The verification utilities are designed for high performance:

- **Target Performance**: <50ms per verification
- **Async Operations**: Full async/await support
- **Batch Processing**: Efficient batch verification
- **Caching**: Key and result caching capabilities
- **Monitoring**: Built-in performance monitoring

Monitor verification performance in production and adjust policies and configurations as needed to maintain optimal performance while ensuring security requirements are met.