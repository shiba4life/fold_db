"""
HTTP client integration for DataFold server communication

This module provides HTTP client functionality for connecting the Python SDK
to the DataFold server, enabling public key registration and signature verification.
"""

import json
import time
import hashlib
import uuid
from typing import Dict, Optional, Any, List, Union
from urllib.parse import urljoin, urlparse
from dataclasses import dataclass, field
import logging

# HTTP client imports with fallback
try:
    import requests
    from requests.adapters import HTTPAdapter
    from requests.packages.urllib3.util.retry import Retry
    REQUESTS_AVAILABLE = True
except ImportError:
    REQUESTS_AVAILABLE = False
    requests = None
    HTTPAdapter = None
    Retry = None

from .exceptions import ServerCommunicationError, ValidationError
from .crypto.ed25519 import Ed25519KeyPair

logger = logging.getLogger(__name__)


@dataclass
class ServerConfig:
    """Configuration for DataFold server connection."""
    base_url: str
    timeout: float = 30.0
    verify_ssl: bool = True
    retry_attempts: int = 3
    retry_backoff_factor: float = 0.3
    max_retry_delay: float = 10.0
    
    def __post_init__(self):
        """Validate server configuration."""
        if not self.base_url:
            raise ValidationError("Server base_url cannot be empty")
        
        # Ensure base_url ends with /
        if not self.base_url.endswith('/'):
            self.base_url += '/'
        
        # Validate URL format
        parsed = urlparse(self.base_url)
        if not parsed.scheme or not parsed.netloc:
            raise ValidationError(f"Invalid server URL format: {self.base_url}")
        
        if self.timeout <= 0:
            raise ValidationError("Timeout must be positive")
        
        if self.retry_attempts < 0:
            raise ValidationError("Retry attempts must be non-negative")


@dataclass
class PublicKeyRegistration:
    """Public key registration response from server."""
    registration_id: str
    client_id: str
    public_key: str
    key_name: Optional[str] = None
    registered_at: Optional[str] = None
    status: str = "active"


@dataclass
class SignatureVerificationResult:
    """Signature verification result from server."""
    verified: bool
    client_id: str
    public_key: str
    verified_at: Optional[str] = None
    message_hash: Optional[str] = None


class DataFoldHttpClient:
    """
    HTTP client for communicating with DataFold server.
    
    Provides methods for public key registration and signature verification
    with automatic retry logic and error handling.
    """
    
    def __init__(self, config: ServerConfig):
        """
        Initialize the HTTP client.
        
        Args:
            config: Server configuration settings
        """
        if not REQUESTS_AVAILABLE:
            raise ServerCommunicationError(
                "HTTP client requires 'requests' package. Install with: pip install requests"
            )
        
        self.config = config
        self.session = self._create_session()
        
        # API endpoints
        self.endpoints = {
            'register_key': 'api/crypto/keys/register',
            'key_status': 'api/crypto/keys/status',
            'verify_signature': 'api/crypto/signatures/verify',
        }
        
        logger.info(f"Initialized DataFold HTTP client for server: {config.base_url}")
    
    def _create_session(self):
        """Create HTTP session with retry logic."""
        session = requests.Session()
        
        # Configure retry strategy
        retry_strategy = Retry(
            total=self.config.retry_attempts,
            status_forcelist=[429, 500, 502, 503, 504],
            method_whitelist=["HEAD", "GET", "PUT", "DELETE", "OPTIONS", "TRACE", "POST"],
            backoff_factor=self.config.retry_backoff_factor,
        )
        
        adapter = HTTPAdapter(max_retries=retry_strategy)
        session.mount("http://", adapter)
        session.mount("https://", adapter)
        
        # Set default headers
        session.headers.update({
            'Content-Type': 'application/json',
            'Accept': 'application/json',
            'User-Agent': 'DataFold-Python-SDK/1.0.0'
        })
        
        return session
    
    def _make_request(self, method: str, endpoint: str, **kwargs) -> Dict[str, Any]:
        """
        Make HTTP request with error handling.
        
        Args:
            method: HTTP method (GET, POST, etc.)
            endpoint: API endpoint path
            **kwargs: Additional arguments for requests
            
        Returns:
            dict: Response JSON data
            
        Raises:
            ServerCommunicationError: On HTTP or network errors
        """
        url = urljoin(self.config.base_url, endpoint)
        
        # Set timeout and SSL verification
        kwargs.setdefault('timeout', self.config.timeout)
        kwargs.setdefault('verify', self.config.verify_ssl)
        
        try:
            logger.debug(f"Making {method} request to {url}")
            response = self.session.request(method, url, **kwargs)
            
            # Handle HTTP error status codes
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
    
    def register_public_key(
        self,
        key_pair: Ed25519KeyPair,
        client_id: Optional[str] = None,
        user_id: Optional[str] = None,
        key_name: Optional[str] = None,
        metadata: Optional[Dict[str, str]] = None
    ) -> PublicKeyRegistration:
        """
        Register public key with DataFold server.
        
        Args:
            key_pair: Ed25519 key pair to register
            client_id: Optional client identifier (generated if not provided)
            user_id: Optional user identifier
            key_name: Optional human-readable key name
            metadata: Optional metadata dictionary
            
        Returns:
            PublicKeyRegistration: Registration confirmation
            
        Raises:
            ServerCommunicationError: On network or server errors
            ValidationError: On invalid input
        """
        if not isinstance(key_pair, Ed25519KeyPair):
            raise ValidationError("key_pair must be an Ed25519KeyPair instance")
        
        # Generate client ID if not provided
        if client_id is None:
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
        response = self._make_request(
            'POST',
            self.endpoints['register_key'],
            json=request_data
        )
        
        # Parse response
        if not response.get('success'):
            error = response.get('error', {})
            raise ServerCommunicationError(
                f"Registration failed: {error.get('message', 'Unknown error')}",
                details={'code': error.get('code', 'REGISTRATION_ERROR')}
            )
        
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
    
    def get_key_status(self, client_id: str) -> PublicKeyRegistration:
        """
        Get public key registration status.
        
        Args:
            client_id: Client identifier
            
        Returns:
            PublicKeyRegistration: Current registration status
            
        Raises:
            ServerCommunicationError: On network or server errors
        """
        if not client_id:
            raise ValidationError("client_id cannot be empty")
        
        logger.debug(f"Getting key status for client: {client_id}")
        
        # Make status request
        response = self._make_request(
            'GET',
            f"{self.endpoints['key_status']}/{client_id}"
        )
        
        # Parse response
        if not response.get('success'):
            error = response.get('error', {})
            raise ServerCommunicationError(
                f"Status check failed: {error.get('message', 'Unknown error')}",
                details={'code': error.get('code', 'STATUS_ERROR')}
            )
        
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
    ) -> SignatureVerificationResult:
        """
        Verify digital signature with server.
        
        Args:
            client_id: Client identifier
            message: Message that was signed
            signature: Ed25519 signature bytes
            message_encoding: Message encoding ('utf8', 'hex', 'base64')
            
        Returns:
            SignatureVerificationResult: Verification result
            
        Raises:
            ServerCommunicationError: On network or server errors
            ValidationError: On invalid input
        """
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
        
        # Make verification request
        response = self._make_request(
            'POST',
            self.endpoints['verify_signature'],
            json=request_data
        )
        
        # Parse response
        if not response.get('success'):
            error = response.get('error', {})
            # Note: Signature verification failure is not necessarily an error,
            # but other issues (network, invalid input) are
            if error.get('code') == 'SIGNATURE_VERIFICATION_FAILED':
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
        
        data = response['data']
        return SignatureVerificationResult(
            verified=data['verified'],
            client_id=data['client_id'],
            public_key=data['public_key'],
            verified_at=data.get('verified_at'),
            message_hash=data.get('message_hash')
        )
    
    def close(self):
        """Close the HTTP session."""
        if hasattr(self, 'session'):
            self.session.close()
            logger.debug("HTTP session closed")


def create_client(
    base_url: str,
    timeout: float = 30.0,
    verify_ssl: bool = True,
    retry_attempts: int = 3
) -> DataFoldHttpClient:
    """
    Create DataFold HTTP client with default configuration.
    
    Args:
        base_url: DataFold server base URL
        timeout: Request timeout in seconds
        verify_ssl: Whether to verify SSL certificates
        retry_attempts: Number of retry attempts for failed requests
        
    Returns:
        DataFoldHttpClient: Configured HTTP client
    """
    config = ServerConfig(
        base_url=base_url,
        timeout=timeout,
        verify_ssl=verify_ssl,
        retry_attempts=retry_attempts
    )
    return DataFoldHttpClient(config)