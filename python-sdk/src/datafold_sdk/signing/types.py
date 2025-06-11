"""
Type definitions for request signing functionality

This module provides type definitions and data classes for RFC 9421 HTTP Message Signatures
implementation with Ed25519 support.
"""

import time
from typing import Dict, List, Optional, Union, Callable, Any
from dataclasses import dataclass, field
from enum import Enum


class HttpMethod(str, Enum):
    """HTTP methods supported for signing"""
    GET = "GET"
    POST = "POST"
    PUT = "PUT"
    DELETE = "DELETE"
    PATCH = "PATCH"
    HEAD = "HEAD"
    OPTIONS = "OPTIONS"


class SignatureAlgorithm(str, Enum):
    """Signature algorithm types"""
    ED25519 = "ed25519"


class DigestAlgorithm(str, Enum):
    """Content digest algorithms"""
    SHA256 = "sha-256"
    SHA512 = "sha-512"


@dataclass
class SignableRequest:
    """
    Request to be signed according to RFC 9421
    
    Attributes:
        method: HTTP method (GET, POST, etc.)
        url: Complete request URL
        headers: Request headers as key-value pairs
        body: Optional request body (string or bytes)
    """
    method: HttpMethod
    url: str
    headers: Dict[str, str]
    body: Optional[Union[str, bytes]] = None
    
    def __post_init__(self):
        """Validate request after initialization"""
        if not self.url:
            raise ValueError("Request URL cannot be empty")
        
        if not isinstance(self.headers, dict):
            raise ValueError("Headers must be a dictionary")
        
        # Normalize headers to lowercase for consistent processing
        self.headers = {k.lower(): v for k, v in self.headers.items()}


@dataclass
class SignatureComponents:
    """
    Signature components that can be included in the signature
    
    Attributes:
        method: Include HTTP method (@method)
        target_uri: Include target URI (@target-uri)
        headers: List of specific headers to include
        content_digest: Include content digest for request body
    """
    method: bool = True
    target_uri: bool = True
    headers: Optional[List[str]] = None
    content_digest: bool = True
    
    def __post_init__(self):
        """Initialize default headers list if None"""
        if self.headers is None:
            self.headers = []
        
        # Normalize header names to lowercase
        self.headers = [h.lower() for h in self.headers]


@dataclass
class SignatureParams:
    """
    Signature parameters for RFC 9421
    
    Attributes:
        created: Unix timestamp when signature was created
        keyid: Key identifier for the signature
        alg: Signature algorithm used
        nonce: Unique nonce for replay protection
    """
    created: int
    keyid: str
    alg: SignatureAlgorithm
    nonce: str
    
    def __post_init__(self):
        """Validate signature parameters"""
        if not self.keyid:
            raise ValueError("Key ID cannot be empty")
        
        if not self.nonce:
            raise ValueError("Nonce cannot be empty")
        
        if self.created <= 0:
            raise ValueError("Created timestamp must be positive")


@dataclass
class ContentDigest:
    """
    Content digest result for request body
    
    Attributes:
        algorithm: Digest algorithm used
        digest: Base64-encoded digest value
        header_value: Complete header value for Content-Digest header
    """
    algorithm: DigestAlgorithm
    digest: str
    header_value: str
    
    def __post_init__(self):
        """Validate content digest"""
        if not self.digest:
            raise ValueError("Digest value cannot be empty")
        
        if not self.header_value:
            raise ValueError("Header value cannot be empty")


@dataclass
class SigningConfig:
    """
    Configuration for request signing
    
    Attributes:
        algorithm: Signature algorithm to use
        key_id: Key identifier for the signature
        private_key: Ed25519 private key bytes (32 bytes)
        components: Components to include in signature
        nonce_generator: Optional custom nonce generator function
        timestamp_generator: Optional custom timestamp generator function
    """
    algorithm: SignatureAlgorithm
    key_id: str
    private_key: bytes
    components: SignatureComponents
    nonce_generator: Optional[Callable[[], str]] = None
    timestamp_generator: Optional[Callable[[], int]] = None
    
    def __post_init__(self):
        """Validate signing configuration"""
        if not self.key_id:
            raise ValueError("Key ID cannot be empty")
        
        if not isinstance(self.private_key, bytes):
            raise ValueError("Private key must be bytes")
        
        if len(self.private_key) != 32:
            raise ValueError("Ed25519 private key must be exactly 32 bytes")
        
        if not isinstance(self.components, SignatureComponents):
            raise ValueError("Components must be SignatureComponents instance")


@dataclass
class SigningOptions:
    """
    Signing options for individual requests
    
    Attributes:
        components: Override default signature components
        nonce: Custom nonce for this request
        timestamp: Custom timestamp for this request
        digest_algorithm: Content digest algorithm to use
    """
    components: Optional[SignatureComponents] = None
    nonce: Optional[str] = None
    timestamp: Optional[int] = None
    digest_algorithm: DigestAlgorithm = DigestAlgorithm.SHA256


@dataclass
class RFC9421SignatureResult:
    """
    Generated signature result for RFC 9421
    
    Attributes:
        signature_input: Signature-Input header value
        signature: Signature header value
        headers: All headers that should be added to the request
        canonical_message: Canonical message that was signed
    """
    signature_input: str
    signature: str
    headers: Dict[str, str]
    canonical_message: str
    
    def __post_init__(self):
        """Validate signature result"""
        if not self.signature_input:
            raise ValueError("Signature input cannot be empty")
        
        if not self.signature:
            raise ValueError("Signature cannot be empty")
        
        if not isinstance(self.headers, dict):
            raise ValueError("Headers must be a dictionary")


@dataclass
class SigningContext:
    """
    Signing context for tracking signature state
    
    Attributes:
        request: Request being signed
        config: Signing configuration
        options: Signing options for this request
        params: Generated signature parameters
        components: Effective signature components used
        content_digest: Content digest if applicable
    """
    request: SignableRequest
    config: SigningConfig
    options: SigningOptions
    params: SignatureParams
    components: SignatureComponents
    content_digest: Optional[ContentDigest] = None


class SigningError(Exception):
    """
    Error class for signing operations
    
    Attributes:
        message: Error message
        code: Error code for programmatic handling
        details: Optional additional error details
    """
    
    def __init__(
        self, 
        message: str, 
        code: str, 
        details: Optional[Dict[str, Any]] = None
    ):
        super().__init__(message)
        self.message = message
        self.code = code
        self.details = details or {}
        
    def __str__(self) -> str:
        if self.details:
            return f"{self.message} (code: {self.code}, details: {self.details})"
        return f"{self.message} (code: {self.code})"
        
    def __repr__(self) -> str:
        return f"SigningError(message='{self.message}', code='{self.code}', details={self.details})"


# Common signing error codes
class SigningErrorCodes:
    """Standard error codes for signing operations"""
    
    # Configuration errors
    INVALID_CONFIG = "INVALID_CONFIG"
    INVALID_PRIVATE_KEY = "INVALID_PRIVATE_KEY"
    INVALID_KEY_ID = "INVALID_KEY_ID"
    
    # Request errors
    INVALID_REQUEST = "INVALID_REQUEST"
    INVALID_URL = "INVALID_URL"
    INVALID_METHOD = "INVALID_METHOD"
    INVALID_HEADERS = "INVALID_HEADERS"
    MISSING_REQUIRED_HEADER = "MISSING_REQUIRED_HEADER"
    
    # Signing errors
    SIGNING_FAILED = "SIGNING_FAILED"
    CANONICAL_MESSAGE_FAILED = "CANONICAL_MESSAGE_FAILED"
    DIGEST_CALCULATION_FAILED = "DIGEST_CALCULATION_FAILED"
    
    # Validation errors
    INVALID_NONCE = "INVALID_NONCE"
    INVALID_TIMESTAMP = "INVALID_TIMESTAMP"
    INVALID_SIGNATURE_COMPONENTS = "INVALID_SIGNATURE_COMPONENTS"
    
    # Crypto errors
    CRYPTOGRAPHY_UNAVAILABLE = "CRYPTOGRAPHY_UNAVAILABLE"
    ED25519_UNAVAILABLE = "ED25519_UNAVAILABLE"
    CRYPTO_ERROR = "CRYPTO_ERROR"


# Type aliases for convenience
NonceGenerator = Callable[[], str]
TimestampGenerator = Callable[[], int]
HeaderDict = Dict[str, str]
RequestBody = Union[str, bytes, None]