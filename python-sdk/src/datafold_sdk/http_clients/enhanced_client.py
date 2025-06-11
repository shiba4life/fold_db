"""
Enhanced HTTP client with automatic signature injection for DataFold server communication

This module provides advanced HTTP client functionality with seamless automatic signature injection,
configurable signing behavior, request/response interceptors, signature caching, and comprehensive
monitoring capabilities.
"""

import json
import time
import asyncio
import hashlib
import logging
import threading
from typing import (
    Dict, Optional, Any, List, Union, Callable, Tuple, Awaitable, TYPE_CHECKING,
    TypeVar, Generic, Protocol, runtime_checkable
)
from urllib.parse import urljoin, urlparse
from dataclasses import dataclass, field
from collections import defaultdict
from datetime import datetime, timedelta
from enum import Enum
from functools import wraps

if TYPE_CHECKING:
    from ..signing import SigningConfig, SigningOptions, RFC9421SignatureResult

# HTTP client imports with fallback
try:
    import requests
    from requests.adapters import HTTPAdapter
    from requests.packages.urllib3.util.retry import Retry
    from requests.models import Response
    REQUESTS_AVAILABLE = True
except ImportError:
    REQUESTS_AVAILABLE = False
    requests = None
    HTTPAdapter = None
    Retry = None
    Response = None

# Optional async HTTP client support
try:
    import httpx
    HTTPX_AVAILABLE = True
except ImportError:
    HTTPX_AVAILABLE = False
    httpx = None

try:
    import aiohttp
    AIOHTTP_AVAILABLE = True
except ImportError:
    AIOHTTP_AVAILABLE = False
    aiohttp = None

from ..exceptions import ServerCommunicationError, ValidationError
from ..crypto.ed25519 import Ed25519KeyPair
from ..signing import (
    SigningConfig, SigningSession, SigningError, RFC9421Signer,
    SignableRequest, HttpMethod, SigningOptions
)

logger = logging.getLogger(__name__)

# Type definitions
T = TypeVar('T')
RequestType = TypeVar('RequestType')
ResponseType = TypeVar('ResponseType')


class SigningMode(Enum):
    """Signing behavior modes"""
    AUTO = "auto"           # Automatically sign all requests unless disabled per endpoint
    MANUAL = "manual"       # Only sign requests with explicit configuration
    DISABLED = "disabled"   # Never sign requests


@dataclass
class EndpointSigningConfig:
    """Per-endpoint signing configuration"""
    enabled: bool = True
    required: bool = False  # Fail if signing fails vs. continue without signature
    options: Optional['SigningOptions'] = None
    custom_components: Optional[Dict[str, Any]] = None


@dataclass
class RequestContext:
    """Context information for request processing"""
    attempt: int = 0
    max_retries: int = 3
    start_time: float = field(default_factory=time.time)
    endpoint: str = ""
    will_be_signed: bool = False
    correlation_id: Optional[str] = None
    metadata: Dict[str, Any] = field(default_factory=dict)


@dataclass
class SigningMetrics:
    """Metrics for signature injection performance and reliability"""
    total_requests: int = 0
    signed_requests: int = 0
    signing_failures: int = 0
    cache_hits: int = 0
    cache_misses: int = 0
    total_signing_time_ms: float = 0.0
    max_signing_time_ms: float = 0.0
    min_signing_time_ms: float = float('inf')
    
    @property
    def average_signing_time_ms(self) -> float:
        """Calculate average signing time"""
        if self.signed_requests == 0:
            return 0.0
        return self.total_signing_time_ms / self.signed_requests
    
    @property
    def cache_hit_rate(self) -> float:
        """Calculate cache hit rate"""
        total_cache_ops = self.cache_hits + self.cache_misses
        if total_cache_ops == 0:
            return 0.0
        return self.cache_hits / total_cache_ops
    
    @property
    def signing_success_rate(self) -> float:
        """Calculate signing success rate"""
        if self.signed_requests == 0:
            return 1.0
        return (self.signed_requests - self.signing_failures) / self.signed_requests


@dataclass
class SignatureCacheEntry:
    """Cache entry for signed requests"""
    signature_headers: Dict[str, str]
    created_at: datetime
    ttl_seconds: int
    request_hash: str
    
    @property
    def is_expired(self) -> bool:
        """Check if cache entry has expired"""
        return datetime.now() > self.created_at + timedelta(seconds=self.ttl_seconds)


# Protocol definitions for interceptors
@runtime_checkable
class RequestInterceptor(Protocol):
    """Protocol for request interceptors"""
    
    def __call__(
        self,
        request: 'SignableRequest',
        context: RequestContext
    ) -> Union['SignableRequest', Awaitable['SignableRequest']]:
        """Process request before sending"""
        ...


@runtime_checkable
class ResponseInterceptor(Protocol):
    """Protocol for response interceptors"""
    
    def __call__(
        self,
        response: Any,  # requests.Response or httpx.Response
        context: RequestContext
    ) -> Union[Any, Awaitable[Any]]:
        """Process response after receiving"""
        ...


class SignatureCache:
    """Thread-safe signature cache with TTL support"""
    
    def __init__(self, max_size: int = 1000, default_ttl: int = 300):
        self.max_size = max_size
        self.default_ttl = default_ttl
        self._cache: Dict[str, SignatureCacheEntry] = {}
        self._access_order: List[str] = []
        self._lock = threading.RLock()
    
    def _generate_cache_key(self, request: 'SignableRequest') -> str:
        """Generate cache key for request"""
        # Create reproducible hash from request components
        components = [
            request.method.value,
            request.url,
            json.dumps(dict(sorted(request.headers.items())), sort_keys=True),
            request.body or ""
        ]
        content = "|".join(str(c) for c in components)
        return hashlib.sha256(content.encode('utf-8')).hexdigest()
    
    def get(self, request: 'SignableRequest') -> Optional[Dict[str, str]]:
        """Get cached signature headers for request"""
        cache_key = self._generate_cache_key(request)
        
        with self._lock:
            entry = self._cache.get(cache_key)
            if entry is None:
                return None
            
            if entry.is_expired:
                self._cache.pop(cache_key, None)
                if cache_key in self._access_order:
                    self._access_order.remove(cache_key)
                return None
            
            # Update access order
            if cache_key in self._access_order:
                self._access_order.remove(cache_key)
            self._access_order.append(cache_key)
            
            return entry.signature_headers.copy()
    
    def put(
        self,
        request: 'SignableRequest',
        signature_headers: Dict[str, str],
        ttl: Optional[int] = None
    ) -> None:
        """Store signature headers for request"""
        cache_key = self._generate_cache_key(request)
        ttl = ttl or self.default_ttl
        
        with self._lock:
            # Remove oldest entries if at capacity
            while len(self._cache) >= self.max_size and self._access_order:
                oldest_key = self._access_order.pop(0)
                self._cache.pop(oldest_key, None)
            
            # Add new entry
            entry = SignatureCacheEntry(
                signature_headers=signature_headers.copy(),
                created_at=datetime.now(),
                ttl_seconds=ttl,
                request_hash=cache_key
            )
            
            self._cache[cache_key] = entry
            if cache_key in self._access_order:
                self._access_order.remove(cache_key)
            self._access_order.append(cache_key)
    
    def clear(self) -> None:
        """Clear all cached entries"""
        with self._lock:
            self._cache.clear()
            self._access_order.clear()
    
    def size(self) -> int:
        """Get current cache size"""
        with self._lock:
            return len(self._cache)
    
    def cleanup_expired(self) -> int:
        """Remove expired entries and return count removed"""
        removed_count = 0
        
        with self._lock:
            expired_keys = [
                key for key, entry in self._cache.items()
                if entry.is_expired
            ]
            
            for key in expired_keys:
                self._cache.pop(key, None)
                if key in self._access_order:
                    self._access_order.remove(key)
                removed_count += 1
        
        return removed_count


@dataclass
class HttpClientConfig:
    """Enhanced HTTP client configuration"""
    base_url: str
    timeout: float = 30.0
    verify_ssl: bool = True
    retry_attempts: int = 3
    retry_backoff_factor: float = 0.3
    max_retry_delay: float = 10.0
    
    # Signing configuration
    signing_mode: SigningMode = SigningMode.AUTO
    endpoint_signing_config: Dict[str, EndpointSigningConfig] = field(default_factory=dict)
    
    # Signature caching
    enable_signature_cache: bool = True
    signature_cache_ttl: int = 300  # 5 minutes
    max_cache_size: int = 1000
    
    # Performance and debugging
    debug_logging: bool = False
    enable_metrics: bool = True
    performance_target_ms: float = 10.0  # Target signing time
    
    # Headers and user agent
    default_headers: Dict[str, str] = field(default_factory=dict)
    user_agent: str = "DataFold-Python-SDK/2.0.0"
    
    def __post_init__(self):
        """Validate configuration"""
        if not self.base_url:
            raise ValidationError("Base URL cannot be empty")
        
        if not self.base_url.endswith('/'):
            self.base_url += '/'
        
        parsed = urlparse(self.base_url)
        if not parsed.scheme or not parsed.netloc:
            raise ValidationError(f"Invalid base URL format: {self.base_url}")
        
        if self.timeout <= 0:
            raise ValidationError("Timeout must be positive")
        
        if self.retry_attempts < 0:
            raise ValidationError("Retry attempts must be non-negative")


class EnhancedDataFoldHttpClient:
    """
    Enhanced HTTP client with automatic signature injection capabilities
    
    Features:
    - Configurable signing modes (auto/manual/disabled)
    - Per-endpoint signing configuration
    - Request/response interceptors
    - Signature caching for performance
    - Comprehensive metrics and monitoring
    - Support for multiple HTTP libraries
    """
    
    def __init__(
        self,
        config: HttpClientConfig,
        signing_config: Optional['SigningConfig'] = None
    ):
        """
        Initialize enhanced HTTP client
        
        Args:
            config: HTTP client configuration
            signing_config: Optional signing configuration
        """
        if not REQUESTS_AVAILABLE:
            raise ServerCommunicationError(
                "Enhanced HTTP client requires 'requests' package. Install with: pip install requests"
            )
        
        self.config = config
        self.signing_config = signing_config
        self.signer = RFC9421Signer(signing_config) if signing_config else None
        
        # Initialize components
        self.session = self._create_session()
        self.cache = SignatureCache(config.max_cache_size, config.signature_cache_ttl) if config.enable_signature_cache else None
        self.metrics = SigningMetrics() if config.enable_metrics else None
        
        # Interceptors
        self.request_interceptors: List[RequestInterceptor] = []
        self.response_interceptors: List[ResponseInterceptor] = []
        
        # Thread safety
        self._metrics_lock = threading.RLock()
        
        # API endpoints
        self.endpoints = {
            'register_key': 'api/crypto/keys/register',
            'key_status': 'api/crypto/keys/status',
            'verify_signature': 'api/crypto/signatures/verify',
            'test_connection': 'api/health/status',
        }
        
        logger.info(f"Enhanced HTTP client initialized for: {config.base_url}")
        if signing_config:
            logger.info(f"Automatic signing enabled with key ID: {signing_config.key_id}")
    
    def _create_session(self) -> requests.Session:
        """Create HTTP session with retry logic"""
        session = requests.Session()
        
        # Configure retry strategy
        retry_strategy = Retry(
            total=self.config.retry_attempts,
            status_forcelist=[429, 500, 502, 503, 504],
            allowed_methods=["HEAD", "GET", "PUT", "DELETE", "OPTIONS", "TRACE", "POST"],
            backoff_factor=self.config.retry_backoff_factor,
        )
        
        adapter = HTTPAdapter(max_retries=retry_strategy)
        session.mount("http://", adapter)
        session.mount("https://", adapter)
        
        # Set default headers
        default_headers = {
            'Content-Type': 'application/json',
            'Accept': 'application/json',
            'User-Agent': self.config.user_agent
        }
        default_headers.update(self.config.default_headers)
        session.headers.update(default_headers)
        
        return session
    
    def configure_signing(
        self,
        signing_config: 'SigningConfig',
        mode: Optional[SigningMode] = None
    ) -> None:
        """
        Configure request signing for this client
        
        Args:
            signing_config: Signing configuration to use
            mode: Optional signing mode override
        """
        self.signing_config = signing_config
        self.signer = RFC9421Signer(signing_config)
        
        if mode:
            self.config.signing_mode = mode
        
        # Clear cache when signing config changes
        if self.cache:
            self.cache.clear()
        
        logger.info(f"Signing configured with key ID: {signing_config.key_id}, mode: {self.config.signing_mode.value}")
    
    def set_signing_mode(self, mode: SigningMode) -> None:
        """Set signing mode"""
        self.config.signing_mode = mode
        logger.info(f"Signing mode set to: {mode.value}")
    
    def configure_endpoint_signing(
        self,
        endpoint: str,
        config: EndpointSigningConfig
    ) -> None:
        """
        Configure signing for specific endpoint
        
        Args:
            endpoint: Endpoint path or pattern
            config: Endpoint-specific signing configuration
        """
        self.config.endpoint_signing_config[endpoint] = config
        logger.debug(f"Endpoint signing configured for {endpoint}: enabled={config.enabled}, required={config.required}")
    
    def add_request_interceptor(self, interceptor: RequestInterceptor) -> None:
        """Add request interceptor"""
        self.request_interceptors.append(interceptor)
        logger.debug(f"Added request interceptor: {interceptor}")
    
    def add_response_interceptor(self, interceptor: ResponseInterceptor) -> None:
        """Add response interceptor"""
        self.response_interceptors.append(interceptor)
        logger.debug(f"Added response interceptor: {interceptor}")
    
    def remove_request_interceptor(self, interceptor: RequestInterceptor) -> bool:
        """Remove request interceptor"""
        try:
            self.request_interceptors.remove(interceptor)
            logger.debug(f"Removed request interceptor: {interceptor}")
            return True
        except ValueError:
            return False
    
    def remove_response_interceptor(self, interceptor: ResponseInterceptor) -> bool:
        """Remove response interceptor"""
        try:
            self.response_interceptors.remove(interceptor)
            logger.debug(f"Removed response interceptor: {interceptor}")
            return True
        except ValueError:
            return False
    
    def clear_signature_cache(self) -> None:
        """Clear signature cache"""
        if self.cache:
            self.cache.clear()
            logger.debug("Signature cache cleared")
    
    def get_signing_metrics(self) -> Optional[SigningMetrics]:
        """Get current signing metrics"""
        return self.metrics
    
    def reset_signing_metrics(self) -> None:
        """Reset signing metrics"""
        if self.metrics:
            with self._metrics_lock:
                self.metrics = SigningMetrics()
            logger.debug("Signing metrics reset")
    
    def _update_metrics(
        self,
        signed: bool = False,
        cache_hit: bool = False,
        signing_time_ms: float = 0.0,
        signing_failed: bool = False
    ) -> None:
        """Update signing metrics"""
        if not self.metrics:
            return
        
        with self._metrics_lock:
            self.metrics.total_requests += 1
            
            if signed:
                self.metrics.signed_requests += 1
                self.metrics.total_signing_time_ms += signing_time_ms
                
                if signing_time_ms > self.metrics.max_signing_time_ms:
                    self.metrics.max_signing_time_ms = signing_time_ms
                
                if signing_time_ms < self.metrics.min_signing_time_ms:
                    self.metrics.min_signing_time_ms = signing_time_ms
            
            if signing_failed:
                self.metrics.signing_failures += 1
            
            if cache_hit:
                self.metrics.cache_hits += 1
            elif signed:
                self.metrics.cache_misses += 1
    
    def _should_sign_request(self, endpoint: str) -> Tuple[bool, bool]:
        """
        Determine if request should be signed
        
        Args:
            endpoint: Request endpoint
            
        Returns:
            Tuple of (should_sign, required)
        """
        if self.config.signing_mode == SigningMode.DISABLED or not self.signer:
            return False, False
        
        # Check endpoint-specific configuration
        endpoint_config = self.config.endpoint_signing_config.get(endpoint)
        if endpoint_config:
            return endpoint_config.enabled, endpoint_config.required
        
        # Default behavior based on signing mode
        if self.config.signing_mode == SigningMode.AUTO:
            return True, False
        elif self.config.signing_mode == SigningMode.MANUAL:
            return False, False
        
        return False, False
    
    def _get_endpoint_signing_options(self, endpoint: str) -> Optional['SigningOptions']:
        """Get signing options for specific endpoint"""
        endpoint_config = self.config.endpoint_signing_config.get(endpoint)
        return endpoint_config.options if endpoint_config else None
    
    async def _apply_request_interceptors(
        self,
        request: 'SignableRequest',
        context: RequestContext
    ) -> 'SignableRequest':
        """Apply request interceptors"""
        current_request = request
        
        for interceptor in self.request_interceptors:
            try:
                result = interceptor(current_request, context)
                if asyncio.iscoroutine(result):
                    current_request = await result
                else:
                    current_request = result
            except Exception as e:
                logger.error(f"Request interceptor failed: {e}")
                if self.config.debug_logging:
                    logger.exception("Request interceptor error details")
        
        return current_request
    
    async def _apply_response_interceptors(
        self,
        response: Response,
        context: RequestContext
    ) -> Response:
        """Apply response interceptors"""
        current_response = response
        
        for interceptor in self.response_interceptors:
            try:
                result = interceptor(current_response, context)
                if asyncio.iscoroutine(result):
                    current_response = await result
                else:
                    current_response = result
            except Exception as e:
                logger.error(f"Response interceptor failed: {e}")
                if self.config.debug_logging:
                    logger.exception("Response interceptor error details")
        
        return current_response
    
    def _sign_request(
        self,
        request: 'SignableRequest',
        endpoint: str,
        context: RequestContext
    ) -> Tuple[Dict[str, str], float]:
        """
        Sign request and return signature headers and timing
        
        Args:
            request: Request to sign
            endpoint: Request endpoint
            context: Request context
            
        Returns:
            Tuple of (signature_headers, signing_time_ms)
        """
        start_time = time.time()
        
        try:
            # Check cache first
            if self.cache:
                cached_headers = self.cache.get(request)
                if cached_headers:
                    signing_time_ms = (time.time() - start_time) * 1000
                    self._update_metrics(signed=True, cache_hit=True, signing_time_ms=signing_time_ms)
                    
                    if self.config.debug_logging:
                        logger.debug(f"Using cached signature for {request.method.value} {request.url}")
                    
                    return cached_headers, signing_time_ms
            
            # Get endpoint-specific options
            signing_options = self._get_endpoint_signing_options(endpoint)
            
            # Sign the request
            result = self.signer.sign_request(request, signing_options)
            signature_headers = result.headers
            
            # Cache the result
            if self.cache:
                self.cache.put(request, signature_headers, self.config.signature_cache_ttl)
            
            signing_time_ms = (time.time() - start_time) * 1000
            
            # Performance warning
            if signing_time_ms > self.config.performance_target_ms:
                logger.warning(
                    f"Signing took {signing_time_ms:.2f}ms (target: {self.config.performance_target_ms}ms)"
                )
            
            self._update_metrics(signed=True, cache_hit=False, signing_time_ms=signing_time_ms)
            
            if self.config.debug_logging:
                logger.debug(
                    f"Signed {request.method.value} {request.url} in {signing_time_ms:.2f}ms"
                )
            
            return signature_headers, signing_time_ms
            
        except Exception as e:
            signing_time_ms = (time.time() - start_time) * 1000
            self._update_metrics(signed=False, signing_failed=True, signing_time_ms=signing_time_ms)
            
            logger.error(f"Request signing failed: {e}")
            if self.config.debug_logging:
                logger.exception("Signing error details")
            
            raise SigningError(f"Request signing failed: {e}")
    
    async def _make_request_async(
        self,
        method: str,
        endpoint: str,
        **kwargs
    ) -> Dict[str, Any]:
        """
        Make HTTP request with automatic signing and interceptors (async version)
        
        Args:
            method: HTTP method
            endpoint: API endpoint path
            **kwargs: Additional arguments for requests
            
        Returns:
            dict: Response JSON data
        """
        url = urljoin(self.config.base_url, endpoint)
        context = RequestContext(
            endpoint=endpoint,
            start_time=time.time(),
            max_retries=self.config.retry_attempts
        )
        
        # Determine if signing is needed
        should_sign, signing_required = self._should_sign_request(endpoint)
        context.will_be_signed = should_sign
        
        try:
            # Create signable request for interceptors and signing
            headers = kwargs.get('headers', {}).copy()
            body = kwargs.get('data') or kwargs.get('json')
            
            # Convert JSON to string if needed
            if 'json' in kwargs and body is not None:
                body = json.dumps(body)
                if 'content-type' not in {k.lower() for k in headers.keys()}:
                    headers['content-type'] = 'application/json'
            
            try:
                http_method = HttpMethod(method.upper())
            except ValueError:
                logger.warning(f"Unsupported HTTP method for signing: {method}")
                should_sign = False
            
            signable_request = SignableRequest(
                method=http_method if should_sign else HttpMethod.GET,  # fallback
                url=url,
                headers=headers,
                body=body
            )
            
            # Apply request interceptors
            signable_request = await self._apply_request_interceptors(signable_request, context)
            
            # Apply signing if needed
            if should_sign and self.signer:
                try:
                    signature_headers, signing_time = self._sign_request(signable_request, endpoint, context)
                    headers.update(signature_headers)
                    
                    if self.config.debug_logging:
                        logger.debug(f"Request signed successfully in {signing_time:.2f}ms")
                        
                except Exception as e:
                    if signing_required:
                        raise ServerCommunicationError(
                            f"Required signing failed for {endpoint}: {e}",
                            details={'error_code': 'SIGNING_REQUIRED_FAILED'}
                        )
                    else:
                        logger.warning(f"Optional signing failed for {endpoint}: {e}")
            
            # Update kwargs with modified headers
            kwargs['headers'] = headers
            kwargs.setdefault('timeout', self.config.timeout)
            kwargs.setdefault('verify', self.config.verify_ssl)
            
            # Make the actual request
            if self.config.debug_logging:
                logger.debug(f"Making {method} request to {url}")
            
            response = self.session.request(method, url, **kwargs)
            
            # Apply response interceptors
            response = await self._apply_response_interceptors(response, context)
            
            # Handle HTTP errors
            if not response.ok:
                try:
                    error_data = response.json()
                    if 'error' in error_data:
                        error_info = error_data['error']
                        message = error_info.get('message', f'HTTP {response.status_code}')
                        code = error_info.get('code', 'HTTP_ERROR')
                    else:
                        message = f'HTTP {response.status_code}: {response.reason}'
                        code = 'HTTP_ERROR'
                except json.JSONDecodeError:
                    message = f'HTTP {response.status_code}: {response.reason}'
                    code = 'HTTP_ERROR'
                
                raise ServerCommunicationError(
                    f"Server request failed: {message}",
                    details={'status_code': response.status_code, 'code': code}
                )
            
            # Parse JSON response
            try:
                return response.json()
            except json.JSONDecodeError as e:
                raise ServerCommunicationError(f"Invalid JSON response: {e}")
                
        except requests.exceptions.Timeout:
            raise ServerCommunicationError(f"Request timeout after {self.config.timeout} seconds")
        except requests.exceptions.ConnectionError as e:
            raise ServerCommunicationError(f"Connection error: {e}")
        except requests.exceptions.RequestException as e:
            raise ServerCommunicationError(f"Request failed: {e}")
    
    def _make_request_sync(
        self,
        method: str,
        endpoint: str,
        **kwargs
    ) -> Dict[str, Any]:
        """
        Make HTTP request with automatic signing and interceptors (sync version)
        
        Args:
            method: HTTP method
            endpoint: API endpoint path
            **kwargs: Additional arguments for requests
            
        Returns:
            dict: Response JSON data
        """
        # Run async version in sync context
        try:
            loop = asyncio.get_event_loop()
        except RuntimeError:
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)
        
        if loop.is_running():
            # If we're already in an async context, create a new thread
            import concurrent.futures
            with concurrent.futures.ThreadPoolExecutor() as executor:
                future = executor.submit(
                    lambda: asyncio.run(self._make_request_async(method, endpoint, **kwargs))
                )
                return future.result()
        else:
            return loop.run_until_complete(self._make_request_async(method, endpoint, **kwargs))
    
    def make_request(
        self,
        method: str,
        endpoint: str,
        **kwargs
    ) -> Dict[str, Any]:
        """
        Make HTTP request with automatic signing and interceptors
        
        Args:
            method: HTTP method
            endpoint: API endpoint path
            **kwargs: Additional arguments for requests
            
        Returns:
            dict: Response JSON data
        """
        return self._make_request_sync(method, endpoint, **kwargs)
    
    # Public API methods
    def register_public_key(
        self,
        key_pair: Ed25519KeyPair,
        client_id: Optional[str] = None,
        user_id: Optional[str] = None,
        key_name: Optional[str] = None,
        metadata: Optional[Dict[str, str]] = None
    ) -> 'PublicKeyRegistration':
        """
        Register public key with DataFold server
        
        Args:
            key_pair: Ed25519 key pair to register
            client_id: Optional client identifier
            user_id: Optional user identifier
            key_name: Optional human-readable key name
            metadata: Optional metadata dictionary
            
        Returns:
            PublicKeyRegistration: Registration confirmation
        """
        if not isinstance(key_pair, Ed25519KeyPair):
            raise ValidationError("key_pair must be an Ed25519KeyPair instance")
        
        # Generate client ID if not provided
        if client_id is None:
            import uuid
            client_id = f"python_sdk_{uuid.uuid4().hex[:8]}"
        
        # Prepare registration request
        request_data = {
            'client_id': client_id,
            'public_key': key_pair.public_key.hex(),
        }
        
        if user_id:
            request_data['user_id'] = user_id
        if key_name:
            request_data['key_name'] = key_name
        if metadata:
            request_data['metadata'] = metadata
        
        logger.info(f"Registering public key for client: {client_id}")
        
        # Make registration request
        response = self.make_request('POST', self.endpoints['register_key'], json=request_data)
        
        # Parse response
        if not response.get('success'):
            error = response.get('error', {})
            raise ServerCommunicationError(
                f"Registration failed: {error.get('message', 'Unknown error')}",
                details={'code': error.get('code', 'REGISTRATION_ERROR')}
            )
        
        from ..http_client import PublicKeyRegistration
        data = response['data']
        registration = PublicKeyRegistration(
            registration_id=data['registration_id'],
            client_id=data['client_id'],
            public_key=data['public_key'],
            key_name=data.get('key_name'),
            registered_at=data.get('registered_at'),
            status=data.get('status', 'active')
        )
        
        logger.info(f"Public key registered successfully: {registration.registration_id}")
        return registration
    
    def get_key_status(self, client_id: str) -> 'PublicKeyRegistration':
        """Get public key registration status"""
        if not client_id:
            raise ValidationError("client_id cannot be empty")
        
        logger.debug(f"Getting key status for client: {client_id}")
        
        response = self.make_request('GET', f"{self.endpoints['key_status']}/{client_id}")
        
        if not response.get('success'):
            error = response.get('error', {})
            raise ServerCommunicationError(
                f"Status check failed: {error.get('message', 'Unknown error')}",
                details={'code': error.get('code', 'STATUS_ERROR')}
            )
        
        from ..http_client import PublicKeyRegistration
        data = response['data']
        return PublicKeyRegistration(
            registration_id=data['registration_id'],
            client_id=data['client_id'],
            public_key=data['public_key'],
            key_name=data.get('key_name'),
            registered_at=data.get('registered_at'),
            status=data.get('status', 'active')
        )
    
    def verify_signature(
        self,
        client_id: str,
        message: Union[str, bytes],
        signature: bytes,
        message_encoding: str = 'utf8'
    ) -> 'SignatureVerificationResult':
        """Verify digital signature with server"""
        if not client_id:
            raise ValidationError("client_id cannot be empty")
        
        if not message:
            raise ValidationError("message cannot be empty")
        
        if not signature or len(signature) != 64:
            raise ValidationError("signature must be exactly 64 bytes")
        
        if message_encoding not in ['utf8', 'hex', 'base64']:
            raise ValidationError("message_encoding must be 'utf8', 'hex', or 'base64'")
        
        # Prepare message based on encoding
        if isinstance(message, bytes):
            if message_encoding == 'utf8':
                message_str = message.decode('utf-8')
            elif message_encoding == 'hex':
                message_str = message.hex()
            else:  # base64
                import base64
                message_str = base64.b64encode(message).decode('ascii')
        else:
            message_str = message
        
        # Prepare verification request
        request_data = {
            'client_id': client_id,
            'message': message_str,
            'signature': signature.hex(),
            'message_encoding': message_encoding
        }
        
        logger.debug(f"Verifying signature for client: {client_id}")
        
        response = self.make_request('POST', self.endpoints['verify_signature'], json=request_data)
        
        if not response.get('success'):
            error = response.get('error', {})
            # Signature verification failure is not necessarily an error
            if error.get('code') == 'SIGNATURE_VERIFICATION_FAILED':
                from ..http_client import SignatureVerificationResult
                return SignatureVerificationResult(
                    verified=False,
                    client_id=client_id,
                    public_key='',
                    verified_at=None,
                    message_hash=None
                )
            else:
                raise ServerCommunicationError(
                    f"Verification failed: {error.get('message', 'Unknown error')}",
                    details={'code': error.get('code', 'VERIFICATION_ERROR')}
                )
        
        from ..http_client import SignatureVerificationResult
        data = response['data']
        return SignatureVerificationResult(
            verified=data['verified'],
            client_id=data['client_id'],
            public_key=data['public_key'],
            verified_at=data.get('verified_at'),
            message_hash=data.get('message_hash')
        )
    
    def test_connection(self) -> Dict[str, Any]:
        """Test connection to DataFold server"""
        return self.make_request('GET', self.endpoints['test_connection'])
    
    def is_signing_enabled(self) -> bool:
        """Check if signing is configured and enabled"""
        return (
            self.signing_config is not None and 
            self.config.signing_mode != SigningMode.DISABLED
        )
    
    def close(self) -> None:
        """Close HTTP session and cleanup resources"""
        if hasattr(self, 'session'):
            self.session.close()
        
        if self.cache:
            self.cache.clear()
        
        logger.debug("Enhanced HTTP client closed")
    
    def __enter__(self):
        """Context manager entry"""
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit"""
        self.close()


# Middleware factories
def create_correlation_middleware(
    header_name: str = 'x-request-id',
    id_generator: Optional[Callable[[], str]] = None
) -> RequestInterceptor:
    """
    Create request correlation middleware
    
    Args:
        header_name: Header name for correlation ID
        id_generator: Optional function to generate correlation IDs
        
    Returns:
        RequestInterceptor: Correlation middleware
    """
    import uuid
    
    def generate_id():
        return str(uuid.uuid4())
    
    generator = id_generator or generate_id
    
    def correlation_interceptor(request: 'SignableRequest', context: RequestContext) -> 'SignableRequest':
        if header_name not in request.headers:
            correlation_id = generator()
            request.headers[header_name] = correlation_id
            context.correlation_id = correlation_id
            context.metadata['correlation_id'] = correlation_id
        
        return request
    
    return correlation_interceptor


def create_logging_middleware(
    log_level: str = 'debug',
    include_headers: bool = False,
    include_body: bool = False
) -> Tuple[RequestInterceptor, ResponseInterceptor]:
    """
    Create request/response logging middleware
    
    Args:
        log_level: Logging level ('debug', 'info', 'warning', 'error')
        include_headers: Whether to log headers
        include_body: Whether to log request/response bodies
        
    Returns:
        Tuple of (request_interceptor, response_interceptor)
    """
    import logging
    middleware_logger = logging.getLogger(__name__)
    log_func = getattr(middleware_logger, log_level, middleware_logger.debug)
    
    def request_interceptor(request: 'SignableRequest', context: RequestContext) -> 'SignableRequest':
        log_data = {
            'method': request.method.value,
            'url': request.url,
            'attempt': context.attempt,
            'correlation_id': context.correlation_id
        }
        
        if include_headers:
            log_data['headers'] = dict(request.headers)
        
        if include_body and request.body:
            log_data['body'] = request.body[:500] + '...' if len(str(request.body)) > 500 else request.body
        
        log_func(f"HTTP Request: {log_data}")
        return request
    
    def response_interceptor(response: Response, context: RequestContext) -> Response:
        elapsed_ms = (time.time() - context.start_time) * 1000
        
        log_data = {
            'status_code': response.status_code,
            'elapsed_ms': f"{elapsed_ms:.2f}",
            'correlation_id': context.correlation_id
        }
        
        if include_headers:
            log_data['headers'] = dict(response.headers)
        
        if include_body:
            try:
                body = response.text[:500] + '...' if len(response.text) > 500 else response.text
                log_data['body'] = body
            except Exception:
                log_data['body'] = '<binary or invalid encoding>'
        
        log_func(f"HTTP Response: {log_data}")
        return response
    
    return request_interceptor, response_interceptor


def create_performance_middleware() -> Tuple[RequestInterceptor, ResponseInterceptor]:
    """
    Create performance monitoring middleware
    
    Returns:
        Tuple of (request_interceptor, response_interceptor) with metrics tracking
    """
    metrics = {
        'total_requests': 0,
        'total_time_ms': 0.0,
        'success_count': 0,
        'error_count': 0,
        'request_times': []
    }
    
    def request_interceptor(request: 'SignableRequest', context: RequestContext) -> 'SignableRequest':
        context.metadata['perf_start_time'] = time.time()
        return request
    
    def response_interceptor(response: Response, context: RequestContext) -> Response:
        start_time = context.metadata.get('perf_start_time', context.start_time)
        elapsed_ms = (time.time() - start_time) * 1000
        
        metrics['total_requests'] += 1
        metrics['total_time_ms'] += elapsed_ms
        metrics['request_times'].append(elapsed_ms)
        
        # Keep only last 100 request times
        if len(metrics['request_times']) > 100:
            metrics['request_times'] = metrics['request_times'][-100:]
        
        if response.ok:
            metrics['success_count'] += 1
        else:
            metrics['error_count'] += 1
        
        return response
    
    # Attach metrics getter to interceptors
    def get_metrics():
        total_requests = metrics['total_requests']
        if total_requests == 0:
            return {
                'total_requests': 0,
                'average_latency_ms': 0.0,
                'success_rate': 1.0,
                'error_rate': 0.0
            }
        
        return {
            'total_requests': total_requests,
            'average_latency_ms': metrics['total_time_ms'] / total_requests,
            'success_rate': metrics['success_count'] / total_requests,
            'error_rate': metrics['error_count'] / total_requests,
            'recent_latencies_ms': metrics['request_times'][-10:]  # Last 10 requests
        }
    
    request_interceptor.get_metrics = get_metrics
    response_interceptor.get_metrics = get_metrics
    
    return request_interceptor, response_interceptor


def create_retry_middleware(
    max_retries: int = 3,
    backoff_factor: float = 2.0,
    retry_status_codes: List[int] = None
) -> RequestInterceptor:
    """
    Create retry middleware for failed requests
    
    Args:
        max_retries: Maximum number of retry attempts
        backoff_factor: Exponential backoff factor
        retry_status_codes: HTTP status codes that should trigger retries
        
    Returns:
        RequestInterceptor: Retry middleware
    """
    if retry_status_codes is None:
        retry_status_codes = [429, 500, 502, 503, 504]
    
    def retry_interceptor(request: 'SignableRequest', context: RequestContext) -> 'SignableRequest':
        context.max_retries = max_retries
        context.metadata['retry_status_codes'] = retry_status_codes
        context.metadata['backoff_factor'] = backoff_factor
        return request
    
    return retry_interceptor


# Factory functions
def create_enhanced_http_client(
    base_url: str,
    signing_config: Optional['SigningConfig'] = None,
    **config_kwargs
) -> EnhancedDataFoldHttpClient:
    """
    Create enhanced HTTP client with default configuration
    
    Args:
        base_url: DataFold server base URL
        signing_config: Optional signing configuration
        **config_kwargs: Additional configuration options
        
    Returns:
        EnhancedDataFoldHttpClient: Configured HTTP client
    """
    config = HttpClientConfig(base_url=base_url, **config_kwargs)
    return EnhancedDataFoldHttpClient(config, signing_config)


def create_signed_http_client(
    signing_config: 'SigningConfig',
    base_url: str,
    signing_mode: SigningMode = SigningMode.AUTO,
    **config_kwargs
) -> EnhancedDataFoldHttpClient:
    """
    Create HTTP client with automatic signing pre-configured
    
    Args:
        signing_config: Signing configuration
        base_url: DataFold server base URL
        signing_mode: Signing mode to use
        **config_kwargs: Additional configuration options
        
    Returns:
        EnhancedDataFoldHttpClient: Pre-configured signed HTTP client
    """
    config_kwargs.setdefault('signing_mode', signing_mode)
    config = HttpClientConfig(base_url=base_url, **config_kwargs)
    return EnhancedDataFoldHttpClient(config, signing_config)


class FluentHttpClientBuilder:
    """
    Fluent builder for enhanced HTTP client configuration
    """
    
    def __init__(self, base_url: Optional[str] = None):
        self._config_kwargs = {}
        self._signing_config: Optional['SigningConfig'] = None
        self._endpoint_configs: Dict[str, EndpointSigningConfig] = {}
        self._interceptors: List[Tuple[str, Any]] = []
        
        if base_url:
            self._config_kwargs['base_url'] = base_url
    
    def base_url(self, url: str) -> 'FluentHttpClientBuilder':
        """Set base URL"""
        self._config_kwargs['base_url'] = url
        return self
    
    def timeout(self, seconds: float) -> 'FluentHttpClientBuilder':
        """Set request timeout"""
        self._config_kwargs['timeout'] = seconds
        return self
    
    def retries(self, attempts: int) -> 'FluentHttpClientBuilder':
        """Set retry attempts"""
        self._config_kwargs['retry_attempts'] = attempts
        return self
    
    def configure_signing(self, config: 'SigningConfig') -> 'FluentHttpClientBuilder':
        """Configure request signing"""
        self._signing_config = config
        return self
    
    def signing_mode(self, mode: SigningMode) -> 'FluentHttpClientBuilder':
        """Set signing mode"""
        self._config_kwargs['signing_mode'] = mode
        return self
    
    def enable_signature_cache(self, enabled: bool = True, ttl: int = 300) -> 'FluentHttpClientBuilder':
        """Enable signature caching"""
        self._config_kwargs['enable_signature_cache'] = enabled
        self._config_kwargs['signature_cache_ttl'] = ttl
        return self
    
    def debug_logging(self, enabled: bool = True) -> 'FluentHttpClientBuilder':
        """Enable debug logging"""
        self._config_kwargs['debug_logging'] = enabled
        return self
    
    def configure_endpoint_signing(
        self,
        endpoint: str,
        enabled: bool = True,
        required: bool = False,
        options: Optional['SigningOptions'] = None
    ) -> 'FluentHttpClientBuilder':
        """Configure endpoint-specific signing"""
        self._endpoint_configs[endpoint] = EndpointSigningConfig(
            enabled=enabled,
            required=required,
            options=options
        )
        return self
    
    def add_correlation_middleware(
        self,
        header_name: str = 'x-request-id'
    ) -> 'FluentHttpClientBuilder':
        """Add correlation ID middleware"""
        self._interceptors.append(('correlation', {'header_name': header_name}))
        return self
    
    def add_logging_middleware(
        self,
        log_level: str = 'debug',
        include_headers: bool = False,
        include_body: bool = False
    ) -> 'FluentHttpClientBuilder':
        """Add logging middleware"""
        self._interceptors.append(('logging', {
            'log_level': log_level,
            'include_headers': include_headers,
            'include_body': include_body
        }))
        return self
    
    def add_performance_middleware(self) -> 'FluentHttpClientBuilder':
        """Add performance monitoring middleware"""
        self._interceptors.append(('performance', {}))
        return self
    
    def build(self) -> EnhancedDataFoldHttpClient:
        """Build the HTTP client"""
        if 'base_url' not in self._config_kwargs:
            raise ValidationError("Base URL is required")
        
        # Set endpoint configurations
        if self._endpoint_configs:
            self._config_kwargs['endpoint_signing_config'] = self._endpoint_configs
        
        # Create client
        config = HttpClientConfig(**self._config_kwargs)
        client = EnhancedDataFoldHttpClient(config, self._signing_config)
        
        # Add middleware
        for middleware_type, kwargs in self._interceptors:
            if middleware_type == 'correlation':
                client.add_request_interceptor(create_correlation_middleware(**kwargs))
            elif middleware_type == 'logging':
                req_interceptor, resp_interceptor = create_logging_middleware(**kwargs)
                client.add_request_interceptor(req_interceptor)
                client.add_response_interceptor(resp_interceptor)
            elif middleware_type == 'performance':
                req_interceptor, resp_interceptor = create_performance_middleware()
                client.add_request_interceptor(req_interceptor)
                client.add_response_interceptor(resp_interceptor)
        
        return client


def create_fluent_http_client(base_url: Optional[str] = None) -> FluentHttpClientBuilder:
    """
    Create fluent HTTP client builder
    
    Args:
        base_url: Optional base URL to start with
        
    Returns:
        FluentHttpClientBuilder: Fluent builder instance
    """
    return FluentHttpClientBuilder(base_url)