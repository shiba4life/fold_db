# Automatic Signature Injection for Python SDK

This document describes the enhanced automatic signature injection capabilities in the DataFold Python SDK HTTP client.

## Overview

The enhanced HTTP client provides seamless automatic signature injection with RFC 9421 compliance, configurable signing behavior, performance optimizations, and comprehensive middleware support.

## Quick Start

### Basic Automatic Signing

```python
from datafold_sdk import (
    create_signed_http_client, 
    create_signing_config,
    generate_key_pair
)

# Generate key pair
key_pair = generate_key_pair()

# Create signing configuration
signing_config = (create_signing_config()
    .algorithm('ed25519')
    .key_id('my-client-key')
    .private_key(key_pair.private_key)
    .profile('standard')
    .build())

# Create pre-configured HTTP client with automatic signing
client = create_signed_http_client(
    signing_config=signing_config,
    base_url='https://api.datafold.com',
    signing_mode='auto'
)

# All requests will be automatically signed
status = client.test_connection()
```

### Fluent Configuration

```python
from datafold_sdk import create_fluent_http_client, SigningMode

client = (create_fluent_http_client()
    .base_url('https://api.datafold.com')
    .configure_signing(signing_config)
    .signing_mode(SigningMode.AUTO)
    .debug_logging(True)
    .configure_endpoint_signing('/keys/register', enabled=True, required=True)
    .configure_endpoint_signing('/status', enabled=False)
    .build())
```

## Signing Modes

### Auto Mode
```python
from datafold_sdk import create_enhanced_http_client, SigningMode

client = create_enhanced_http_client(
    base_url='https://api.datafold.com',
    signing_mode=SigningMode.AUTO
)
client.configure_signing(signing_config)

# All requests are automatically signed unless endpoint-specific config overrides
```

### Manual Mode
```python
client = create_enhanced_http_client(
    base_url='https://api.datafold.com',
    signing_mode=SigningMode.MANUAL
)
client.configure_signing(signing_config)

# Only explicitly configured endpoints are signed
```

### Disabled Mode
```python
client = create_enhanced_http_client(
    base_url='https://api.datafold.com',
    signing_mode=SigningMode.DISABLED
)

# No requests are signed, even if signer is configured
```

## Endpoint-Specific Configuration

```python
from datafold_sdk import EndpointSigningConfig, SigningOptions

client = create_enhanced_http_client(
    base_url='https://api.datafold.com',
    signing_mode=SigningMode.AUTO
)

# Configure specific endpoints
client.configure_endpoint_signing(
    '/keys/register',
    EndpointSigningConfig(
        enabled=True,
        required=True,  # Fail if signing fails
        options=SigningOptions(
            components={
                'method': True,
                'target_uri': True,
                'headers': ['content-type', 'authorization'],
                'content_digest': True
            }
        )
    )
)

client.configure_endpoint_signing(
    '/status',
    EndpointSigningConfig(enabled=False)  # Never sign status requests
)

client.configure_endpoint_signing(
    '/data/query',
    EndpointSigningConfig(
        enabled=True,
        required=False,  # Continue without signature if signing fails
        options=SigningOptions(digest_algorithm='sha-512')
    )
)
```

## Performance Optimization

### Signature Caching

```python
from datafold_sdk import HttpClientConfig, SigningMode

config = HttpClientConfig(
    base_url='https://api.datafold.com',
    signing_mode=SigningMode.AUTO,
    enable_signature_cache=True,
    signature_cache_ttl=300,  # 5 minutes
    max_cache_size=1000
)

client = create_enhanced_http_client(config=config)
client.configure_signing(signing_config)

# Identical requests will reuse cached signatures
client.test_connection()  # Cache miss - signs request
client.test_connection()  # Cache hit - reuses signature
```

### Cache Management

```python
# Clear cache manually
client.clear_signature_cache()

# Check cache effectiveness
metrics = client.get_signing_metrics()
if metrics:
    cache_hit_rate = metrics.cache_hit_rate
    print(f"Cache hit rate: {cache_hit_rate:.2%}")
```

## Middleware System

### Request Interceptors

```python
from datafold_sdk import (
    create_correlation_middleware,
    create_logging_middleware,
    create_performance_middleware
)

client = create_enhanced_http_client(base_url='https://api.datafold.com')

# Add correlation ID to all requests
correlation_middleware = create_correlation_middleware(header_name='x-request-id')
client.add_request_interceptor(correlation_middleware)

# Add request/response logging
req_interceptor, resp_interceptor = create_logging_middleware(
    log_level='debug',
    include_headers=True,
    include_body=False
)
client.add_request_interceptor(req_interceptor)
client.add_response_interceptor(resp_interceptor)
```

### Custom Middleware

```python
from datafold_sdk import RequestInterceptor, ResponseInterceptor

def auth_interceptor(request, context):
    """Add authentication header to requests"""
    if 'authorization' not in request.headers:
        request.headers['authorization'] = f'Bearer {get_auth_token()}'
    return request

def response_validator(response, context):
    """Validate response format"""
    if response.status_code == 200:
        try:
            data = response.json()
            if not data.get('success'):
                context.metadata['validation_warning'] = True
        except Exception:
            pass
    return response

client.add_request_interceptor(auth_interceptor)
client.add_response_interceptor(response_validator)
```

### Performance Monitoring

```python
req_interceptor, resp_interceptor = create_performance_middleware()

client.add_request_interceptor(req_interceptor)
client.add_response_interceptor(resp_interceptor)

# Get performance metrics
import time
time.sleep(30)  # After some requests

metrics = req_interceptor.get_metrics()
print('Performance:', {
    'total_requests': metrics['total_requests'],
    'average_latency': metrics['average_latency_ms'],
    'success_rate': metrics['success_rate']
})
```

## Monitoring and Debugging

### Signing Metrics

```python
client = create_signed_http_client(
    signing_config=signing_config,
    base_url='https://api.datafold.com',
    signing_mode=SigningMode.AUTO,
    debug_logging=True
)

# Make some requests...
client.test_connection()
client.register_public_key(key_pair=key_pair)

# Check signing performance
metrics = client.get_signing_metrics()
if metrics:
    print('Signing metrics:', {
        'total_requests': metrics.total_requests,
        'signed_requests': metrics.signed_requests,
        'signing_failures': metrics.signing_failures,
        'average_signing_time': metrics.average_signing_time_ms,
        'cache_hit_rate': metrics.cache_hit_rate
    })

# Reset metrics for fresh measurement
client.reset_signing_metrics()
```

### Debug Logging

```python
import logging

# Enable debug logging
logging.basicConfig(level=logging.DEBUG)

client = create_enhanced_http_client(
    base_url='https://api.datafold.com',
    signing_mode=SigningMode.AUTO,
    debug_logging=True
)

client.configure_signing(signing_config)

# Will log detailed signing information
client.test_connection()
```

## Error Handling

### Graceful Degradation

```python
from datafold_sdk import EndpointSigningConfig, ServerCommunicationError

client = create_enhanced_http_client(
    base_url='https://api.datafold.com',
    signing_mode=SigningMode.AUTO
)

# Configure endpoint requirements
client.configure_endpoint_signing(
    '/critical',
    EndpointSigningConfig(enabled=True, required=True)  # Fail if signing fails
)

client.configure_endpoint_signing(
    '/optional',
    EndpointSigningConfig(enabled=True, required=False)  # Continue without signing
)

try:
    # This will fail if signing fails
    client.make_request('POST', '/critical', json={'data': 'test'})
except ServerCommunicationError as e:
    if e.details and e.details.get('error_code') == 'SIGNING_REQUIRED_FAILED':
        print('Critical endpoint requires signing, but signing failed')

# This will continue even if signing fails
response = client.make_request('GET', '/optional')
```

### Signing Failure Recovery

```python
import time

client = create_enhanced_http_client(
    base_url='https://api.datafold.com',
    signing_mode=SigningMode.AUTO
)

# Configure with potentially faulty signer
client.configure_signing(faulty_signing_config)

# Monitor signing failures
def monitor_signing_health():
    while True:
        time.sleep(60)  # Check every minute
        metrics = client.get_signing_metrics()
        if metrics and metrics.signing_failures > 0:
            print(f"{metrics.signing_failures} signing failures detected")
            
            # Reconfigure with backup signer
            client.configure_signing(backup_signing_config)
            client.reset_signing_metrics()

# Run monitoring in background thread
import threading
monitor_thread = threading.Thread(target=monitor_signing_health, daemon=True)
monitor_thread.start()
```

## Advanced Use Cases

### Dynamic Signing Configuration

```python
client = create_enhanced_http_client(
    base_url='https://api.datafold.com',
    signing_mode=SigningMode.MANUAL
)

# Configure different signing for different endpoints
client.configure_endpoint_signing(
    '/public-api',
    EndpointSigningConfig(enabled=False)
)

client.configure_endpoint_signing(
    '/private-api',
    EndpointSigningConfig(
        enabled=True,
        required=True,
        options=SigningOptions(digest_algorithm='sha-512')
    )
)

# Update configuration at runtime
client.configure_endpoint_signing(
    '/emergency-endpoint',
    EndpointSigningConfig(enabled=True, required=True)
)
```

### Custom Signing Profiles

```python
from datafold_sdk import create_from_profile

# Use predefined security profiles
strict_config = create_from_profile('strict', 'key-001', private_key)
standard_config = create_from_profile('standard', 'key-001', private_key)
minimal_config = create_from_profile('minimal', 'key-001', private_key)

# Switch profiles based on environment
import os
if os.getenv('ENV') == 'production':
    config = strict_config
else:
    config = standard_config

client = create_signed_http_client(signing_config=config)
```

### Context Manager Usage

```python
# Use client as context manager for automatic cleanup
with create_signed_http_client(
    signing_config=signing_config,
    base_url='https://api.datafold.com'
) as client:
    # Make requests
    status = client.test_connection()
    registration = client.register_public_key(key_pair=key_pair)
    
# Client automatically closed and resources cleaned up
```

### Async/Await Integration

```python
import asyncio

# Create async-compatible interceptors
async def async_auth_interceptor(request, context):
    # Simulate async token refresh
    token = await get_auth_token_async()
    request.headers['authorization'] = f'Bearer {token}'
    return request

# Use with async code
client = create_enhanced_http_client(base_url='https://api.datafold.com')
client.add_request_interceptor(async_auth_interceptor)

# The client handles async interceptors automatically
response = client.make_request('GET', '/test')
```

## Best Practices

### 1. Use Appropriate Signing Modes
- **Production**: Use `SigningMode.AUTO` with endpoint-specific overrides
- **Development**: Use `SigningMode.AUTO` with debug logging
- **Testing**: Use `SigningMode.DISABLED` for faster tests

### 2. Configure Caching Appropriately
- Enable caching for high-frequency requests
- Use shorter TTL for sensitive operations
- Monitor cache hit rates

```python
# Good caching configuration
config = HttpClientConfig(
    base_url='https://api.datafold.com',
    enable_signature_cache=True,
    signature_cache_ttl=300,  # 5 minutes for most requests
    max_cache_size=1000
)

# Override for sensitive endpoints
client.configure_endpoint_signing(
    '/auth/login',
    EndpointSigningConfig(enabled=True, required=True)
)
```

### 3. Handle Errors Gracefully
- Use `required=False` for non-critical endpoints
- Implement fallback mechanisms
- Monitor signing failure rates

```python
# Graceful error handling
client.configure_endpoint_signing(
    '/analytics',
    EndpointSigningConfig(enabled=True, required=False)  # Non-critical
)

client.configure_endpoint_signing(
    '/payment',
    EndpointSigningConfig(enabled=True, required=True)   # Critical
)
```

### 4. Optimize Performance
- Use signature caching for repeated requests
- Monitor signing times
- Use minimal security profiles when appropriate

```python
# Performance monitoring
def check_performance():
    metrics = client.get_signing_metrics()
    if metrics and metrics.average_signing_time_ms > 10:
        print(f"Warning: Average signing time is {metrics.average_signing_time_ms:.2f}ms")
```

### 5. Security Considerations
- Protect private keys properly
- Rotate keys regularly
- Use strict profiles for sensitive data
- Monitor for signing failures

```python
# Secure key management
import os
from pathlib import Path

def load_signing_key():
    key_path = Path(os.getenv('DATAFOLD_KEY_PATH', '~/.datafold/private.key')).expanduser()
    if not key_path.exists():
        raise ValueError("Private key not found")
    
    with open(key_path, 'rb') as f:
        return f.read()

private_key = load_signing_key()
signing_config = (create_signing_config()
    .key_id(os.getenv('DATAFOLD_KEY_ID'))
    .private_key(private_key)
    .profile('strict')  # Use strict profile for production
    .build())
```

## Migration from Basic Client

### Before (Basic Signing)

```python
from datafold_sdk import DataFoldHttpClient, ServerConfig

client = DataFoldHttpClient(ServerConfig(base_url='https://api.datafold.com'))
client.configure_signing(signing_config)

# Manual signing control
if needs_signing:
    client.enable_signing(signer)
else:
    client.disable_signing()
```

### After (Enhanced Automatic Signing)

```python
from datafold_sdk import create_signed_http_client, SigningMode, EndpointSigningConfig

client = create_signed_http_client(
    signing_config=signing_config,
    base_url='https://api.datafold.com',
    signing_mode=SigningMode.AUTO,
    enable_signature_cache=True,
    endpoint_signing_config={
        '/public': EndpointSigningConfig(enabled=False),
        '/private': EndpointSigningConfig(enabled=True, required=True)
    }
)

# Automatic signing based on configuration
# No manual enable/disable needed
```

## Integration Examples

### Django Integration

```python
# settings.py
from datafold_sdk import create_signed_http_client, SigningMode
import os

DATAFOLD_CLIENT = create_signed_http_client(
    signing_config=load_signing_config(),
    base_url=os.getenv('DATAFOLD_API_URL'),
    signing_mode=SigningMode.AUTO,
    enable_signature_cache=True,
    debug_logging=settings.DEBUG
)

# views.py
from django.conf import settings

def analytics_view(request):
    client = settings.DATAFOLD_CLIENT
    data = client.make_request('GET', '/analytics/dashboard')
    return JsonResponse(data)
```

### Flask Integration

```python
from flask import Flask, current_app
from datafold_sdk import create_signed_http_client

def create_app():
    app = Flask(__name__)
    
    # Initialize DataFold client
    app.datafold_client = create_signed_http_client(
        signing_config=load_signing_config(),
        base_url=app.config['DATAFOLD_API_URL'],
        signing_mode='auto'
    )
    
    @app.route('/api/data')
    def get_data():
        client = current_app.datafold_client
        return client.make_request('GET', '/data/latest')
    
    return app
```

### FastAPI Integration

```python
from fastapi import FastAPI, Depends
from datafold_sdk import EnhancedDataFoldHttpClient, create_signed_http_client

app = FastAPI()

# Dependency injection
async def get_datafold_client() -> EnhancedDataFoldHttpClient:
    return create_signed_http_client(
        signing_config=load_signing_config(),
        base_url=settings.DATAFOLD_API_URL,
        signing_mode='auto'
    )

@app.get("/api/status")
async def get_status(client: EnhancedDataFoldHttpClient = Depends(get_datafold_client)):
    return client.test_connection()
```

## Type Hints Support

The enhanced HTTP client includes full type hint support:

```python
from typing import Dict, Any, Optional
from datafold_sdk import (
    HttpClientConfig,
    SigningMode,
    EndpointSigningConfig,
    RequestInterceptor,
    ResponseInterceptor,
    SigningMetrics
)

def create_client_config(
    base_url: str,
    signing_mode: SigningMode = SigningMode.AUTO,
    cache_enabled: bool = True
) -> HttpClientConfig:
    return HttpClientConfig(
        base_url=base_url,
        signing_mode=signing_mode,
        enable_signature_cache=cache_enabled
    )

def custom_interceptor(request, context) -> RequestInterceptor:
    # Type-safe request manipulation
    request.headers['x-custom'] = 'value'
    return request
```

## Troubleshooting

### Common Issues

1. **Signing Failures**
   ```python
   # Check signing configuration
   if not client.is_signing_enabled():
       print("Signing not properly configured")
   
   # Check metrics for failures
   metrics = client.get_signing_metrics()
   if metrics and metrics.signing_failures > 0:
       print(f"Signing failures: {metrics.signing_failures}")
   ```

2. **Performance Issues**
   ```python
   # Check signing times
   metrics = client.get_signing_metrics()
   if metrics and metrics.average_signing_time_ms > 10:
       print("Consider enabling caching or optimizing signing")
   ```

3. **Cache Issues**
   ```python
   # Clear cache if stale
   client.clear_signature_cache()
   
   # Check cache effectiveness
   metrics = client.get_signing_metrics()
   if metrics and metrics.cache_hit_rate < 0.5:
       print("Cache hit rate is low, consider adjusting TTL")
   ```

This enhanced automatic signature injection provides a powerful, flexible, and performant foundation for authenticated HTTP communication with the DataFold platform while maintaining Python idioms and conventions.