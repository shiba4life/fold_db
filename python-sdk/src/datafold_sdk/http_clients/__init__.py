"""
HTTP client module for DataFold SDK

This module provides HTTP client functionality for communicating with DataFold servers,
including automatic signature injection, request/response interceptors, and comprehensive
monitoring capabilities.
"""

from .enhanced_client import (
    # Core client classes
    EnhancedDataFoldHttpClient,
    HttpClientConfig,
    SigningMode,
    
    # Configuration classes
    EndpointSigningConfig,
    RequestContext,
    SigningMetrics,
    SignatureCacheEntry,
    SignatureCache,
    
    # Protocol definitions
    RequestInterceptor,
    ResponseInterceptor,
    
    # Factory functions
    create_enhanced_http_client,
    create_signed_http_client,
    create_fluent_http_client,
    FluentHttpClientBuilder,
    
    # Middleware factories
    create_correlation_middleware,
    create_logging_middleware,
    create_performance_middleware,
    create_retry_middleware,
)

# Note: Original http_client imports are now handled directly in main __init__.py
# to avoid circular import issues

__all__ = [
    # Enhanced client
    'EnhancedDataFoldHttpClient',
    'HttpClientConfig',
    'SigningMode',
    'EndpointSigningConfig',
    'RequestContext',
    'SigningMetrics',
    'SignatureCacheEntry',
    'SignatureCache',
    'RequestInterceptor',
    'ResponseInterceptor',
    'create_enhanced_http_client',
    'create_signed_http_client',
    'create_fluent_http_client',
    'FluentHttpClientBuilder',
    'create_correlation_middleware',
    'create_logging_middleware',
    'create_performance_middleware',
    'create_retry_middleware',
]