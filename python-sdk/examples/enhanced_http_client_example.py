"""
Example usage of the enhanced HTTP client with automatic signature injection

This example demonstrates the key features of the enhanced DataFold Python SDK
HTTP client, including automatic signature injection, middleware, and monitoring.
"""

import time
import logging
from typing import Dict, Any

from datafold_sdk import (
    # Enhanced HTTP client
    create_signed_http_client,
    create_fluent_http_client,
    create_enhanced_http_client,
    
    # Configuration
    HttpClientConfig,
    SigningMode,
    EndpointSigningConfig,
    
    # Signing
    create_signing_config,
    generate_key_pair,
    
    # Middleware
    create_correlation_middleware,
    create_logging_middleware,
    create_performance_middleware,
)

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)

logger = logging.getLogger(__name__)


def basic_automatic_signing_example():
    """Example 1: Basic automatic signing setup"""
    print("\n=== Basic Automatic Signing Example ===")
    
    # Generate key pair for this example
    key_pair = generate_key_pair()
    
    # Create signing configuration
    signing_config = (create_signing_config()
        .key_id('example-client-001')
        .private_key(key_pair.private_key)
        .profile('standard')  # Use standard security profile
        .build())
    
    # Create HTTP client with automatic signing
    client = create_signed_http_client(
        signing_config=signing_config,
        base_url='https://api.example.com',
        signing_mode=SigningMode.AUTO,
        enable_signature_cache=True,
        debug_logging=True
    )
    
    print(f"Client created with signing enabled: {client.is_signing_enabled()}")
    
    # All requests will be automatically signed
    try:
        # This would make a signed request
        response = client.test_connection()
        print("Connection test successful (would be signed)")
    except Exception as e:
        print(f"Request failed (expected for demo): {e}")
    
    # Get signing metrics
    metrics = client.get_signing_metrics()
    if metrics:
        print(f"Signing metrics: {metrics.total_requests} requests, "
              f"{metrics.signed_requests} signed")
    
    client.close()


def fluent_configuration_example():
    """Example 2: Fluent API configuration"""
    print("\n=== Fluent Configuration Example ===")
    
    key_pair = generate_key_pair()
    signing_config = (create_signing_config()
        .key_id('fluent-client-001')
        .private_key(key_pair.private_key)
        .profile('standard')
        .build())
    
    # Use fluent builder pattern
    client = (create_fluent_http_client()
        .base_url('https://api.example.com')
        .timeout(60.0)
        .retries(5)
        .configure_signing(signing_config)
        .signing_mode(SigningMode.AUTO)
        .enable_signature_cache(True, 600)  # 10 minutes TTL
        .debug_logging(True)
        .configure_endpoint_signing(
            '/public/status', 
            enabled=False  # Don't sign status endpoint
        )
        .configure_endpoint_signing(
            '/secure/data',
            enabled=True,
            required=True  # Require signing for secure endpoints
        )
        .add_correlation_middleware()
        .add_logging_middleware('info', include_headers=True)
        .add_performance_middleware()
        .build())
    
    print(f"Fluent client created with {len(client.request_interceptors)} "
          f"request interceptors and {len(client.response_interceptors)} response interceptors")
    
    client.close()


def endpoint_specific_configuration_example():
    """Example 3: Endpoint-specific signing configuration"""
    print("\n=== Endpoint-Specific Configuration Example ===")
    
    key_pair = generate_key_pair()
    signing_config = (create_signing_config()
        .key_id('endpoint-client-001')
        .private_key(key_pair.private_key)
        .profile('standard')
        .build())
    
    client = create_enhanced_http_client(
        base_url='https://api.example.com',
        signing_mode=SigningMode.AUTO
    )
    client.configure_signing(signing_config)
    
    # Configure different signing behavior per endpoint
    endpoints_config = {
        '/public/health': EndpointSigningConfig(enabled=False),
        '/auth/login': EndpointSigningConfig(
            enabled=True,
            required=True  # Critical - must be signed
        ),
        '/data/query': EndpointSigningConfig(
            enabled=True,
            required=False  # Best effort - continue if signing fails
        ),
        '/admin/users': EndpointSigningConfig(
            enabled=True,
            required=True,
            # Custom signing options for admin endpoints
            # options=SigningOptions(digest_algorithm='sha-512')
        )
    }
    
    for endpoint, config in endpoints_config.items():
        client.configure_endpoint_signing(endpoint, config)
        print(f"Configured {endpoint}: enabled={config.enabled}, required={config.required}")
    
    client.close()


def middleware_and_monitoring_example():
    """Example 4: Middleware and performance monitoring"""
    print("\n=== Middleware and Monitoring Example ===")
    
    client = create_enhanced_http_client(
        base_url='https://api.example.com',
        signing_mode=SigningMode.AUTO,
        enable_signature_cache=True,
        enable_metrics=True
    )
    
    # Add correlation middleware
    correlation_middleware = create_correlation_middleware('x-request-id')
    client.add_request_interceptor(correlation_middleware)
    
    # Add logging middleware
    log_req, log_resp = create_logging_middleware(
        log_level='info',
        include_headers=False,
        include_body=False
    )
    client.add_request_interceptor(log_req)
    client.add_response_interceptor(log_resp)
    
    # Add performance monitoring
    perf_req, perf_resp = create_performance_middleware()
    client.add_request_interceptor(perf_req)
    client.add_response_interceptor(perf_resp)
    
    print(f"Client configured with {len(client.request_interceptors)} middleware components")
    
    # Simulate some requests (would fail in real usage without server)
    for i in range(3):
        try:
            client.test_connection()
        except Exception:
            pass  # Expected to fail in demo
    
    # Check performance metrics
    perf_metrics = perf_req.get_metrics()
    print(f"Performance metrics: {perf_metrics['total_requests']} requests")
    
    # Check signing metrics
    signing_metrics = client.get_signing_metrics()
    if signing_metrics:
        print(f"Signing metrics:")
        print(f"  Total requests: {signing_metrics.total_requests}")
        print(f"  Signed requests: {signing_metrics.signed_requests}")
        print(f"  Average signing time: {signing_metrics.average_signing_time_ms:.2f}ms")
        print(f"  Cache hit rate: {signing_metrics.cache_hit_rate:.2%}")
    
    client.close()


def custom_middleware_example():
    """Example 5: Custom middleware implementation"""
    print("\n=== Custom Middleware Example ===")
    
    def auth_interceptor(request, context):
        """Add authentication header"""
        if 'authorization' not in request.headers:
            # In real usage, get token from secure storage
            request.headers['authorization'] = 'Bearer demo-token'
        return request
    
    def request_id_interceptor(request, context):
        """Add custom request ID"""
        if 'x-custom-id' not in request.headers:
            request.headers['x-custom-id'] = f"req-{int(time.time())}"
        return request
    
    def response_validator(response, context):
        """Validate response format"""
        try:
            if hasattr(response, 'json') and response.status_code == 200:
                data = response.json()
                if not data.get('success'):
                    context.metadata['validation_warning'] = True
                    print("Warning: Response indicates failure")
        except Exception:
            pass
        return response
    
    client = create_enhanced_http_client(base_url='https://api.example.com')
    
    # Add custom middleware
    client.add_request_interceptor(auth_interceptor)
    client.add_request_interceptor(request_id_interceptor)
    client.add_response_interceptor(response_validator)
    
    print("Custom middleware added for authentication, request IDs, and response validation")
    
    client.close()


def performance_optimization_example():
    """Example 6: Performance optimization with caching"""
    print("\n=== Performance Optimization Example ===")
    
    key_pair = generate_key_pair()
    signing_config = (create_signing_config()
        .key_id('perf-client-001')
        .private_key(key_pair.private_key)
        .profile('minimal')  # Use minimal profile for better performance
        .build())
    
    # Configure for high performance
    config = HttpClientConfig(
        base_url='https://api.example.com',
        signing_mode=SigningMode.AUTO,
        enable_signature_cache=True,
        signature_cache_ttl=300,  # 5 minutes
        max_cache_size=1000,
        performance_target_ms=5.0,  # Stricter performance target
        enable_metrics=True
    )
    
    client = create_enhanced_http_client(config=config)
    client.configure_signing(signing_config)
    
    print("High-performance client configured with:")
    print(f"  Cache enabled: {config.enable_signature_cache}")
    print(f"  Cache TTL: {config.signature_cache_ttl}s")
    print(f"  Performance target: {config.performance_target_ms}ms")
    
    # Simulate cache usage
    if client.cache:
        print(f"Initial cache size: {client.cache.size()}")
        
        # In real usage, repeated identical requests would hit cache
        print("Cache would improve performance for repeated requests")
    
    client.close()


def error_handling_example():
    """Example 7: Error handling and graceful degradation"""
    print("\n=== Error Handling Example ===")
    
    client = create_enhanced_http_client(
        base_url='https://api.example.com',
        signing_mode=SigningMode.AUTO
    )
    
    # Configure endpoints with different error handling
    client.configure_endpoint_signing(
        '/critical/payment',
        EndpointSigningConfig(enabled=True, required=True)  # Must be signed
    )
    
    client.configure_endpoint_signing(
        '/optional/analytics',
        EndpointSigningConfig(enabled=True, required=False)  # Best effort
    )
    
    print("Configured endpoints with different signing requirements:")
    print("  /critical/payment - signing required (fail if signing fails)")
    print("  /optional/analytics - signing optional (continue if signing fails)")
    
    # In real usage, you would handle signing failures appropriately
    try:
        # This would fail if signing is required but fails
        # client.make_request('POST', '/critical/payment', json={'amount': 100})
        print("Critical endpoint would fail if signing fails")
    except Exception as e:
        print(f"Critical request failed: {e}")
    
    try:
        # This would continue even if signing fails
        # client.make_request('GET', '/optional/analytics')
        print("Optional endpoint would continue even if signing fails")
    except Exception as e:
        print(f"Optional request failed: {e}")
    
    client.close()


def main():
    """Run all examples"""
    print("DataFold Python SDK Enhanced HTTP Client Examples")
    print("=" * 50)
    
    try:
        basic_automatic_signing_example()
        fluent_configuration_example()
        endpoint_specific_configuration_example()
        middleware_and_monitoring_example()
        custom_middleware_example()
        performance_optimization_example()
        error_handling_example()
        
        print("\n=== All Examples Completed Successfully ===")
        print("\nKey Features Demonstrated:")
        print("✓ Automatic signature injection")
        print("✓ Configurable signing modes")
        print("✓ Endpoint-specific configuration")
        print("✓ Middleware system")
        print("✓ Performance monitoring")
        print("✓ Signature caching")
        print("✓ Error handling and graceful degradation")
        print("✓ Fluent API configuration")
        
    except Exception as e:
        logger.error(f"Example failed: {e}")
        import traceback
        traceback.print_exc()


if __name__ == '__main__':
    main()