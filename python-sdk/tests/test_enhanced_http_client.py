"""
Unit tests for enhanced HTTP client with automatic signature injection
"""

import json
import time
import asyncio
import threading
from unittest.mock import Mock, patch, MagicMock
from datetime import datetime, timedelta
import pytest

from datafold_sdk.http_clients import (
    EnhancedDataFoldHttpClient,
    HttpClientConfig,
    SigningMode,
    EndpointSigningConfig,
    RequestContext,
    SigningMetrics,
    SignatureCache,
    create_enhanced_http_client,
    create_signed_http_client,
    create_fluent_http_client,
    create_correlation_middleware,
    create_logging_middleware,
    create_performance_middleware,
)
from datafold_sdk.signing import (
    SigningConfig,
    SignableRequest,
    HttpMethod,
    SigningOptions,
    RFC9421SignatureResult,
    create_signing_config,
)
from datafold_sdk.crypto.ed25519 import Ed25519KeyPair, generate_key_pair
from datafold_sdk.exceptions import ServerCommunicationError, ValidationError


class TestHttpClientConfig:
    """Test HTTP client configuration"""
    
    def test_basic_config(self):
        """Test basic configuration creation"""
        config = HttpClientConfig(base_url="https://api.example.com")
        assert config.base_url == "https://api.example.com/"
        assert config.timeout == 30.0
        assert config.signing_mode == SigningMode.AUTO
        assert config.enable_signature_cache is True
    
    def test_config_validation(self):
        """Test configuration validation"""
        # Empty base URL should raise error
        with pytest.raises(ValidationError, match="Base URL cannot be empty"):
            HttpClientConfig(base_url="")
        
        # Invalid URL format should raise error
        with pytest.raises(ValidationError, match="Invalid base URL format"):
            HttpClientConfig(base_url="invalid-url")
        
        # Negative timeout should raise error
        with pytest.raises(ValidationError, match="Timeout must be positive"):
            HttpClientConfig(base_url="https://api.example.com", timeout=-1.0)
        
        # Negative retry attempts should raise error
        with pytest.raises(ValidationError, match="Retry attempts must be non-negative"):
            HttpClientConfig(base_url="https://api.example.com", retry_attempts=-1)
    
    def test_url_normalization(self):
        """Test URL normalization"""
        config = HttpClientConfig(base_url="https://api.example.com")
        assert config.base_url.endswith('/')
        
        config = HttpClientConfig(base_url="https://api.example.com/")
        assert config.base_url == "https://api.example.com/"


class TestSignatureCache:
    """Test signature cache functionality"""
    
    def test_cache_basic_operations(self):
        """Test basic cache operations"""
        cache = SignatureCache(max_size=2, default_ttl=300)
        
        request = SignableRequest(
            method=HttpMethod.POST,
            url="https://api.example.com/test",
            headers={"content-type": "application/json"},
            body='{"test": "data"}'
        )
        
        # Cache miss
        assert cache.get(request) is None
        
        # Store in cache
        headers = {"signature": "test-signature", "signature-input": "test-input"}
        cache.put(request, headers)
        
        # Cache hit
        cached = cache.get(request)
        assert cached == headers
        assert cache.size() == 1
    
    def test_cache_expiration(self):
        """Test cache expiration"""
        cache = SignatureCache(max_size=10, default_ttl=1)  # 1 second TTL
        
        request = SignableRequest(
            method=HttpMethod.GET,
            url="https://api.example.com/test",
            headers={},
            body=None
        )
        
        headers = {"signature": "test-signature"}
        cache.put(request, headers, ttl=1)
        
        # Should be available immediately
        assert cache.get(request) is not None
        
        # Wait for expiration
        time.sleep(1.1)
        
        # Should be expired
        assert cache.get(request) is None
    
    def test_cache_size_limit(self):
        """Test cache size limit enforcement"""
        cache = SignatureCache(max_size=2, default_ttl=300)
        
        # Add first request
        request1 = SignableRequest(
            method=HttpMethod.GET,
            url="https://api.example.com/test1",
            headers={},
            body=None
        )
        cache.put(request1, {"sig": "1"})
        
        # Add second request
        request2 = SignableRequest(
            method=HttpMethod.GET,
            url="https://api.example.com/test2",
            headers={},
            body=None
        )
        cache.put(request2, {"sig": "2"})
        
        assert cache.size() == 2
        
        # Add third request - should evict first
        request3 = SignableRequest(
            method=HttpMethod.GET,
            url="https://api.example.com/test3",
            headers={},
            body=None
        )
        cache.put(request3, {"sig": "3"})
        
        assert cache.size() == 2
        assert cache.get(request1) is None  # Evicted
        assert cache.get(request2) is not None  # Still there
        assert cache.get(request3) is not None  # Newly added


class TestSigningMetrics:
    """Test signing metrics functionality"""
    
    def test_metrics_calculation(self):
        """Test metrics calculations"""
        metrics = SigningMetrics()
        
        # Initial state
        assert metrics.average_signing_time_ms == 0.0
        assert metrics.cache_hit_rate == 0.0
        assert metrics.signing_success_rate == 1.0
        
        # Add some data
        metrics.total_requests = 10
        metrics.signed_requests = 8
        metrics.signing_failures = 1
        metrics.cache_hits = 3
        metrics.cache_misses = 5
        metrics.total_signing_time_ms = 80.0
        
        # Test calculations
        assert metrics.average_signing_time_ms == 10.0
        assert metrics.cache_hit_rate == 0.375  # 3/8
        assert metrics.signing_success_rate == 0.875  # 7/8


@patch('datafold_sdk.http_clients.enhanced_client.requests')
class TestEnhancedDataFoldHttpClient:
    """Test enhanced HTTP client functionality"""
    
    def setup_method(self):
        """Set up test fixtures"""
        self.key_pair = generate_key_pair()
        self.signing_config = (create_signing_config()
                             .key_id("test-key")
                             .private_key(self.key_pair.private_key)
                             .build())
        
        self.config = HttpClientConfig(
            base_url="https://api.example.com",
            signing_mode=SigningMode.AUTO,
            enable_signature_cache=True,
            debug_logging=True
        )
    
    def test_client_initialization(self, mock_requests):
        """Test client initialization"""
        client = EnhancedDataFoldHttpClient(self.config, self.signing_config)
        
        assert client.config == self.config
        assert client.signing_config == self.signing_config
        assert client.signer is not None
        assert client.cache is not None
        assert client.metrics is not None
        assert len(client.request_interceptors) == 0
        assert len(client.response_interceptors) == 0
    
    def test_client_without_signing(self, mock_requests):
        """Test client without signing configuration"""
        client = EnhancedDataFoldHttpClient(self.config)
        
        assert client.signing_config is None
        assert client.signer is None
        assert client.cache is not None
        assert client.metrics is not None
    
    def test_signing_mode_configuration(self, mock_requests):
        """Test signing mode configuration"""
        client = EnhancedDataFoldHttpClient(self.config, self.signing_config)
        
        # Test different signing modes
        client.set_signing_mode(SigningMode.MANUAL)
        assert client.config.signing_mode == SigningMode.MANUAL
        
        client.set_signing_mode(SigningMode.DISABLED)
        assert client.config.signing_mode == SigningMode.DISABLED
        
        client.set_signing_mode(SigningMode.AUTO)
        assert client.config.signing_mode == SigningMode.AUTO
    
    def test_endpoint_signing_configuration(self, mock_requests):
        """Test endpoint-specific signing configuration"""
        client = EnhancedDataFoldHttpClient(self.config, self.signing_config)
        
        # Configure endpoint signing
        endpoint_config = EndpointSigningConfig(
            enabled=True,
            required=True,
            options=SigningOptions()
        )
        
        client.configure_endpoint_signing("/test", endpoint_config)
        
        assert "/test" in client.config.endpoint_signing_config
        assert client.config.endpoint_signing_config["/test"] == endpoint_config
    
    def test_should_sign_request_auto_mode(self, mock_requests):
        """Test request signing determination in AUTO mode"""
        client = EnhancedDataFoldHttpClient(self.config, self.signing_config)
        client.set_signing_mode(SigningMode.AUTO)
        
        # Default behavior - should sign
        should_sign, required = client._should_sign_request("/test")
        assert should_sign is True
        assert required is False
        
        # Endpoint-specific override - disabled
        client.configure_endpoint_signing("/no-sign", EndpointSigningConfig(enabled=False))
        should_sign, required = client._should_sign_request("/no-sign")
        assert should_sign is False
        assert required is False
        
        # Endpoint-specific override - required
        client.configure_endpoint_signing("/required", EndpointSigningConfig(enabled=True, required=True))
        should_sign, required = client._should_sign_request("/required")
        assert should_sign is True
        assert required is True
    
    def test_should_sign_request_manual_mode(self, mock_requests):
        """Test request signing determination in MANUAL mode"""
        client = EnhancedDataFoldHttpClient(self.config, self.signing_config)
        client.set_signing_mode(SigningMode.MANUAL)
        
        # Default behavior - should not sign
        should_sign, required = client._should_sign_request("/test")
        assert should_sign is False
        assert required is False
        
        # Endpoint-specific override - enabled
        client.configure_endpoint_signing("/sign", EndpointSigningConfig(enabled=True))
        should_sign, required = client._should_sign_request("/sign")
        assert should_sign is True
        assert required is False
    
    def test_should_sign_request_disabled_mode(self, mock_requests):
        """Test request signing determination in DISABLED mode"""
        client = EnhancedDataFoldHttpClient(self.config, self.signing_config)
        client.set_signing_mode(SigningMode.DISABLED)
        
        # Should never sign, even with endpoint config
        should_sign, required = client._should_sign_request("/test")
        assert should_sign is False
        assert required is False
        
        client.configure_endpoint_signing("/test", EndpointSigningConfig(enabled=True, required=True))
        should_sign, required = client._should_sign_request("/test")
        assert should_sign is False
        assert required is False
    
    @patch('datafold_sdk.http_clients.enhanced_client.RFC9421Signer')
    def test_request_signing_with_cache(self, mock_signer_class, mock_requests):
        """Test request signing with caching"""
        mock_signer = Mock()
        mock_signer_class.return_value = mock_signer
        
        # Mock signing result
        mock_result = RFC9421SignatureResult(
            signature_input="test-input",
            signature="test-signature",
            headers={"signature": "test-sig", "signature-input": "test-input"},
            canonical_message="test-message"
        )
        mock_signer.sign_request.return_value = mock_result
        
        client = EnhancedDataFoldHttpClient(self.config, self.signing_config)
        
        request = SignableRequest(
            method=HttpMethod.POST,
            url="https://api.example.com/test",
            headers={"content-type": "application/json"},
            body='{"test": "data"}'
        )
        
        # First call - should sign and cache
        headers, timing = client._sign_request(request, "/test", RequestContext())
        assert headers == mock_result.headers
        assert timing > 0
        
        # Verify signer was called
        mock_signer.sign_request.assert_called_once()
        
        # Reset mock
        mock_signer.sign_request.reset_mock()
        
        # Second call - should use cache
        headers2, timing2 = client._sign_request(request, "/test", RequestContext())
        assert headers2 == mock_result.headers
        
        # Signer should not be called again
        mock_signer.sign_request.assert_not_called()
    
    def test_interceptor_management(self, mock_requests):
        """Test request/response interceptor management"""
        client = EnhancedDataFoldHttpClient(self.config, self.signing_config)
        
        # Create mock interceptors
        req_interceptor = Mock()
        resp_interceptor = Mock()
        
        # Add interceptors
        client.add_request_interceptor(req_interceptor)
        client.add_response_interceptor(resp_interceptor)
        
        assert req_interceptor in client.request_interceptors
        assert resp_interceptor in client.response_interceptors
        
        # Remove interceptors
        assert client.remove_request_interceptor(req_interceptor) is True
        assert client.remove_response_interceptor(resp_interceptor) is True
        
        assert req_interceptor not in client.request_interceptors
        assert resp_interceptor not in client.response_interceptors
        
        # Remove non-existent interceptors
        assert client.remove_request_interceptor(req_interceptor) is False
        assert client.remove_response_interceptor(resp_interceptor) is False
    
    def test_metrics_management(self, mock_requests):
        """Test metrics management"""
        client = EnhancedDataFoldHttpClient(self.config, self.signing_config)
        
        # Initial metrics
        metrics = client.get_signing_metrics()
        assert metrics is not None
        assert metrics.total_requests == 0
        
        # Update metrics
        client._update_metrics(signed=True, signing_time_ms=15.0)
        
        metrics = client.get_signing_metrics()
        assert metrics.total_requests == 1
        assert metrics.signed_requests == 1
        assert metrics.total_signing_time_ms == 15.0
        
        # Reset metrics
        client.reset_signing_metrics()
        
        metrics = client.get_signing_metrics()
        assert metrics.total_requests == 0
        assert metrics.signed_requests == 0
    
    def test_cache_management(self, mock_requests):
        """Test signature cache management"""
        client = EnhancedDataFoldHttpClient(self.config, self.signing_config)
        
        # Cache should be initialized
        assert client.cache is not None
        assert client.cache.size() == 0
        
        # Clear cache
        client.clear_signature_cache()
        assert client.cache.size() == 0


class TestMiddleware:
    """Test middleware functionality"""
    
    def test_correlation_middleware(self):
        """Test correlation ID middleware"""
        middleware = create_correlation_middleware('x-correlation-id')
        
        request = SignableRequest(
            method=HttpMethod.GET,
            url="https://api.example.com/test",
            headers={},
            body=None
        )
        context = RequestContext()
        
        # Apply middleware
        result = middleware(request, context)
        
        # Should add correlation ID header
        assert 'x-correlation-id' in result.headers
        assert context.correlation_id is not None
        assert context.metadata['correlation_id'] == result.headers['x-correlation-id']
    
    def test_logging_middleware(self):
        """Test logging middleware"""
        req_interceptor, resp_interceptor = create_logging_middleware(
            log_level='debug',
            include_headers=True,
            include_body=True
        )
        
        request = SignableRequest(
            method=HttpMethod.POST,
            url="https://api.example.com/test",
            headers={'content-type': 'application/json'},
            body='{"test": "data"}'
        )
        context = RequestContext(correlation_id='test-123')
        
        # Test request interceptor
        with patch('datafold_sdk.http_clients.enhanced_client.logging.getLogger') as mock_get_logger:
            mock_logger = mock_get_logger.return_value
            # Recreate the middleware to use the mocked logger
            req_interceptor, resp_interceptor = create_logging_middleware(
                log_level='debug',
                include_headers=True,
                include_body=True
            )
            result = req_interceptor(request, context)
            assert result == request
            mock_logger.debug.assert_called_once()
    
    def test_performance_middleware(self):
        """Test performance monitoring middleware"""
        req_interceptor, resp_interceptor = create_performance_middleware()
        
        request = SignableRequest(
            method=HttpMethod.GET,
            url="https://api.example.com/test",
            headers={},
            body=None
        )
        context = RequestContext()
        
        # Mock response
        mock_response = Mock()
        mock_response.ok = True
        mock_response.status_code = 200
        
        # Apply request interceptor
        req_interceptor(request, context)
        assert 'perf_start_time' in context.metadata
        
        # Apply response interceptor
        resp_interceptor(mock_response, context)
        
        # Check metrics
        metrics = req_interceptor.get_metrics()
        assert metrics['total_requests'] == 1
        assert metrics['success_rate'] == 1.0
        assert metrics['error_rate'] == 0.0


class TestFactoryFunctions:
    """Test factory functions"""
    
    def setup_method(self):
        """Set up test fixtures"""
        self.key_pair = generate_key_pair()
        self.signing_config = (create_signing_config()
                             .key_id("test-key")
                             .private_key(self.key_pair.private_key)
                             .build())
    
    @patch('datafold_sdk.http_clients.enhanced_client.requests')
    def test_create_enhanced_http_client(self, mock_requests):
        """Test enhanced HTTP client factory"""
        client = create_enhanced_http_client(
            base_url="https://api.example.com",
            signing_config=self.signing_config,
            signing_mode=SigningMode.AUTO,
            enable_signature_cache=True
        )
        
        assert isinstance(client, EnhancedDataFoldHttpClient)
        assert client.config.base_url == "https://api.example.com/"
        assert client.config.signing_mode == SigningMode.AUTO
        assert client.signing_config == self.signing_config
    
    @patch('datafold_sdk.http_clients.enhanced_client.requests')
    def test_create_signed_http_client(self, mock_requests):
        """Test signed HTTP client factory"""
        client = create_signed_http_client(
            signing_config=self.signing_config,
            base_url="https://api.example.com",
            signing_mode=SigningMode.AUTO
        )
        
        assert isinstance(client, EnhancedDataFoldHttpClient)
        assert client.config.signing_mode == SigningMode.AUTO
        assert client.signing_config == self.signing_config
        assert client.is_signing_enabled() is True
    
    @patch('datafold_sdk.http_clients.enhanced_client.requests')
    def test_fluent_http_client_builder(self, mock_requests):
        """Test fluent HTTP client builder"""
        client = (create_fluent_http_client()
                 .base_url("https://api.example.com")
                 .timeout(60.0)
                 .retries(5)
                 .configure_signing(self.signing_config)
                 .signing_mode(SigningMode.AUTO)
                 .enable_signature_cache(True, 600)
                 .debug_logging(True)
                 .configure_endpoint_signing("/test", enabled=True, required=True)
                 .add_correlation_middleware()
                 .add_logging_middleware('info', True, False)
                 .add_performance_middleware()
                 .build())
        
        assert isinstance(client, EnhancedDataFoldHttpClient)
        assert client.config.base_url == "https://api.example.com/"
        assert client.config.timeout == 60.0
        assert client.config.retry_attempts == 5
        assert client.config.signing_mode == SigningMode.AUTO
        assert client.config.enable_signature_cache is True
        assert client.config.signature_cache_ttl == 600
        assert client.config.debug_logging is True
        assert "/test" in client.config.endpoint_signing_config
        assert len(client.request_interceptors) == 3  # correlation, logging, performance
        assert len(client.response_interceptors) == 2  # logging, performance


class TestAsyncIntegration:
    """Test async/await integration"""
    
    def setup_method(self):
        """Set up test fixtures"""
        self.key_pair = generate_key_pair()
        self.signing_config = (create_signing_config()
                             .key_id("test-key")
                             .private_key(self.key_pair.private_key)
                             .build())
        
        self.config = HttpClientConfig(
            base_url="https://api.example.com",
            signing_mode=SigningMode.AUTO
        )
    
    @pytest.mark.asyncio
    @patch('datafold_sdk.http_clients.enhanced_client.requests')
    async def test_async_request_interceptors(self, mock_requests):
        """Test async request interceptors"""
        client = EnhancedDataFoldHttpClient(self.config, self.signing_config)
        
        # Create async interceptor
        async def async_interceptor(request, context):
            await asyncio.sleep(0.01)  # Simulate async work
            request.headers['x-async'] = 'true'
            return request
        
        client.add_request_interceptor(async_interceptor)
        
        request = SignableRequest(
            method=HttpMethod.GET,
            url="https://api.example.com/test",
            headers={},
            body=None
        )
        context = RequestContext()
        
        # Apply interceptors
        result = await client._apply_request_interceptors(request, context)
        
        assert result.headers['x-async'] == 'true'
    
    @pytest.mark.asyncio
    @patch('datafold_sdk.http_clients.enhanced_client.requests')
    async def test_async_response_interceptors(self, mock_requests):
        """Test async response interceptors"""
        client = EnhancedDataFoldHttpClient(self.config, self.signing_config)
        
        # Create async interceptor
        async def async_interceptor(response, context):
            await asyncio.sleep(0.01)  # Simulate async work
            response.processed = True
            return response
        
        client.add_response_interceptor(async_interceptor)
        
        mock_response = Mock()
        context = RequestContext()
        
        # Apply interceptors
        result = await client._apply_response_interceptors(mock_response, context)
        
        assert result.processed is True


if __name__ == '__main__':
    pytest.main([__file__])