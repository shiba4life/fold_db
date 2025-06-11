"""
HTTP client integration for request signing

This module provides integration between the request signing functionality
and HTTP client libraries, enabling automatic request signing for outbound
HTTP requests.
"""

import logging
from typing import Dict, Optional, Any, Union
from functools import wraps

# HTTP client imports with fallback
try:
    import requests
    from requests.models import PreparedRequest
    from requests.sessions import Session
    REQUESTS_AVAILABLE = True
except ImportError:
    REQUESTS_AVAILABLE = False
    requests = None
    PreparedRequest = None
    Session = None

from .types import (
    SignableRequest,
    SigningConfig,
    SigningOptions,
    HttpMethod,
    SigningError,
    SigningErrorCodes,
    HeaderDict,
)
from .rfc9421_signer import RFC9421Signer
from ..exceptions import ServerCommunicationError

logger = logging.getLogger(__name__)


class SigningSession:
    """
    HTTP session wrapper with automatic request signing capability.
    
    This class wraps a requests.Session and automatically signs outgoing
    requests according to the configured signing settings.
    """
    
    def __init__(
        self,
        signing_config: Optional[SigningConfig] = None,
        session: Optional[Session] = None,
        auto_sign: bool = True
    ):
        """
        Initialize signing session.
        
        Args:
            signing_config: Optional signing configuration
            session: Optional existing requests session to wrap
            auto_sign: Whether to automatically sign requests
            
        Raises:
            ServerCommunicationError: If requests library is not available
        """
        if not REQUESTS_AVAILABLE:
            raise ServerCommunicationError(
                "Signing session requires 'requests' package. Install with: pip install requests"
            )
        
        self.session = session or requests.Session()
        self.signing_config = signing_config
        self.signer = RFC9421Signer(signing_config) if signing_config else None
        self.auto_sign = auto_sign
        self._original_request = self.session.request
    
    def configure_signing(
        self,
        config: SigningConfig,
        auto_sign: bool = True
    ) -> None:
        """
        Configure request signing for this session.
        
        Args:
            config: Signing configuration
            auto_sign: Whether to automatically sign requests
        """
        self.signing_config = config
        self.signer = RFC9421Signer(config)
        self.auto_sign = auto_sign
        logger.info(f"Configured request signing for key ID: {config.key_id}")
    
    def disable_signing(self) -> None:
        """Disable automatic request signing."""
        self.auto_sign = False
        logger.info("Disabled automatic request signing")
    
    def enable_signing(self) -> None:
        """Enable automatic request signing (if configured)."""
        if self.signer:
            self.auto_sign = True
            logger.info("Enabled automatic request signing")
        else:
            logger.warning("Cannot enable signing - no signing configuration available")
    
    def request(
        self,
        method: str,
        url: str,
        **kwargs
    ) -> requests.Response:
        """
        Make HTTP request with optional automatic signing.
        
        Args:
            method: HTTP method
            url: Request URL
            **kwargs: Additional arguments for requests
            
        Returns:
            requests.Response: HTTP response
        """
        if self.auto_sign and self.signer:
            try:
                # Sign the request before sending
                kwargs = self._sign_request_kwargs(method, url, **kwargs)
            except Exception as e:
                logger.error(f"Request signing failed: {e}")
                # Continue without signing if signing fails (graceful degradation)
                
        return self._original_request(method, url, **kwargs)
    
    def _sign_request_kwargs(
        self,
        method: str,
        url: str,
        **kwargs
    ) -> Dict[str, Any]:
        """
        Sign request and modify kwargs to include signature headers.
        
        Args:
            method: HTTP method
            url: Request URL
            **kwargs: Original request arguments
            
        Returns:
            dict: Modified kwargs with signature headers
        """
        # Extract request components
        headers = kwargs.get('headers', {}).copy()
        body = kwargs.get('data') or kwargs.get('json')
        
        # Convert JSON to string if needed
        if 'json' in kwargs and body is not None:
            import json
            body = json.dumps(body)
            # Ensure content-type is set for JSON
            if 'content-type' not in {k.lower() for k in headers.keys()}:
                headers['content-type'] = 'application/json'
        
        # Create signable request
        try:
            http_method = HttpMethod(method.upper())
        except ValueError:
            # Unsupported method, skip signing
            logger.warning(f"Unsupported HTTP method for signing: {method}")
            return kwargs
        
        signable_request = SignableRequest(
            method=http_method,
            url=url,
            headers=headers,
            body=body
        )
        
        # Sign the request
        result = self.signer.sign_request(signable_request)
        
        # Update headers with signature
        headers.update(result.headers)
        kwargs['headers'] = headers
        
        logger.debug(f"Signed {method} request to {url}")
        return kwargs
    
    def get(self, url: str, **kwargs) -> requests.Response:
        """Make GET request."""
        return self.request('GET', url, **kwargs)
    
    def post(self, url: str, **kwargs) -> requests.Response:
        """Make POST request."""
        return self.request('POST', url, **kwargs)
    
    def put(self, url: str, **kwargs) -> requests.Response:
        """Make PUT request."""
        return self.request('PUT', url, **kwargs)
    
    def delete(self, url: str, **kwargs) -> requests.Response:
        """Make DELETE request."""
        return self.request('DELETE', url, **kwargs)
    
    def patch(self, url: str, **kwargs) -> requests.Response:
        """Make PATCH request."""
        return self.request('PATCH', url, **kwargs)
    
    def head(self, url: str, **kwargs) -> requests.Response:
        """Make HEAD request."""
        return self.request('HEAD', url, **kwargs)
    
    def options(self, url: str, **kwargs) -> requests.Response:
        """Make OPTIONS request."""
        return self.request('OPTIONS', url, **kwargs)
    
    def close(self) -> None:
        """Close the underlying session."""
        self.session.close()
    
    def __enter__(self):
        """Context manager entry."""
        return self
    
    def __exit__(self, *args):
        """Context manager exit."""
        self.close()


def create_signing_session(
    signing_config: Optional[SigningConfig] = None,
    auto_sign: bool = True,
    **session_kwargs
) -> SigningSession:
    """
    Create a new signing session.
    
    Args:
        signing_config: Optional signing configuration
        auto_sign: Whether to automatically sign requests
        **session_kwargs: Additional arguments for requests.Session
        
    Returns:
        SigningSession: Configured signing session
    """
    if not REQUESTS_AVAILABLE:
        raise ServerCommunicationError(
            "Signing session requires 'requests' package. Install with: pip install requests"
        )
    
    session = requests.Session()
    
    # Apply session configuration
    for key, value in session_kwargs.items():
        if hasattr(session, key):
            setattr(session, key, value)
    
    return SigningSession(
        signing_config=signing_config,
        session=session,
        auto_sign=auto_sign
    )


def enable_request_signing(
    session: Union[Session, SigningSession],
    config: SigningConfig
) -> None:
    """
    Enable request signing for an existing session.
    
    Args:
        session: Session to enable signing for
        config: Signing configuration
        
    Raises:
        SigningError: If session type is not supported
    """
    if isinstance(session, SigningSession):
        session.configure_signing(config, auto_sign=True)
    elif hasattr(session, 'request'):
        # Monkey patch the session's request method
        _monkey_patch_session(session, config)
    else:
        raise SigningError(
            f"Unsupported session type: {type(session)}",
            SigningErrorCodes.INVALID_CONFIG,
            {"session_type": str(type(session))}
        )


def disable_request_signing(session: Union[Session, SigningSession]) -> None:
    """
    Disable request signing for a session.
    
    Args:
        session: Session to disable signing for
    """
    if isinstance(session, SigningSession):
        session.disable_signing()
    elif hasattr(session, '_original_request'):
        # Restore original request method
        session.request = session._original_request
        delattr(session, '_original_request')
        delattr(session, '_signing_config')
        delattr(session, '_signer')


def _monkey_patch_session(session: Session, config: SigningConfig) -> None:
    """
    Monkey patch a requests session to add signing capability.
    
    Args:
        session: Session to patch
        config: Signing configuration
    """
    # Store original request method and signing config
    session._original_request = session.request
    session._signing_config = config
    session._signer = RFC9421Signer(config)
    
    @wraps(session._original_request)
    def signing_request(method: str, url: str, **kwargs):
        """Patched request method with signing."""
        try:
            # Create signing session temporarily to leverage signing logic
            temp_signing_session = SigningSession(config, session, auto_sign=True)
            kwargs = temp_signing_session._sign_request_kwargs(method, url, **kwargs)
        except Exception as e:
            logger.error(f"Request signing failed: {e}")
            # Continue without signing on failure
        
        return session._original_request(method, url, **kwargs)
    
    # Replace the request method
    session.request = signing_request


def sign_prepared_request(
    prepared_request: PreparedRequest,
    config: SigningConfig,
    options: Optional[SigningOptions] = None
) -> PreparedRequest:
    """
    Sign a prepared request.
    
    Args:
        prepared_request: Prepared request to sign
        config: Signing configuration
        options: Optional signing options
        
    Returns:
        PreparedRequest: Request with signature headers added
        
    Raises:
        SigningError: If signing fails
    """
    if not REQUESTS_AVAILABLE:
        raise SigningError(
            "Prepared request signing requires 'requests' package",
            SigningErrorCodes.CRYPTOGRAPHY_UNAVAILABLE
        )
    
    # Extract components from prepared request
    method = HttpMethod(prepared_request.method.upper())
    url = prepared_request.url
    headers = dict(prepared_request.headers) if prepared_request.headers else {}
    body = prepared_request.body
    
    # Convert body to appropriate format
    if body and isinstance(body, bytes):
        try:
            body = body.decode('utf-8')
        except UnicodeDecodeError:
            # Keep as bytes for binary content
            pass
    
    # Create signable request
    signable_request = SignableRequest(
        method=method,
        url=url,
        headers=headers,
        body=body
    )
    
    # Sign the request
    signer = RFC9421Signer(config)
    result = signer.sign_request(signable_request, options)
    
    # Update prepared request headers
    if prepared_request.headers is None:
        prepared_request.headers = {}
    
    prepared_request.headers.update(result.headers)
    
    return prepared_request


def create_signing_adapter(config: SigningConfig) -> 'SigningHTTPAdapter':
    """
    Create a requests HTTPAdapter that automatically signs requests.
    
    Args:
        config: Signing configuration
        
    Returns:
        SigningHTTPAdapter: HTTP adapter with signing capability
    """
    return SigningHTTPAdapter(config)


class SigningHTTPAdapter:
    """
    HTTPAdapter that automatically signs requests.
    
    This can be used with requests.Session.mount() to add signing
    to specific URL patterns.
    """
    
    def __init__(self, config: SigningConfig):
        """
        Initialize signing adapter.
        
        Args:
            config: Signing configuration
        """
        if not REQUESTS_AVAILABLE:
            raise SigningError(
                "Signing adapter requires 'requests' package",
                SigningErrorCodes.CRYPTOGRAPHY_UNAVAILABLE
            )
        
        from requests.adapters import HTTPAdapter
        
        self.config = config
        self.signer = RFC9421Signer(config)
        self.adapter = HTTPAdapter()
    
    def send(self, request, **kwargs):
        """
        Send request with automatic signing.
        
        Args:
            request: Prepared request
            **kwargs: Additional arguments
            
        Returns:
            Response object
        """
        try:
            # Sign the prepared request
            signed_request = sign_prepared_request(request, self.config)
            return self.adapter.send(signed_request, **kwargs)
        except Exception as e:
            logger.error(f"Request signing failed in adapter: {e}")
            # Fall back to unsigned request
            return self.adapter.send(request, **kwargs)
    
    def close(self):
        """Close the adapter."""
        self.adapter.close()