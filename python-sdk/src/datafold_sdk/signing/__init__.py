"""
DataFold Python SDK - Request Signing Module

RFC 9421 HTTP Message Signatures implementation with Ed25519 support.
This module provides request signing functionality for authenticating with 
DataFold's signature-protected API endpoints.
"""

from .types import (
    SignableRequest,
    SigningConfig,
    SignatureComponents,
    RFC9421SignatureResult,
    SignatureParams,
    ContentDigest,
    SigningOptions,
    SigningError,
    DigestAlgorithm,
    HttpMethod,
    SignatureAlgorithm,
)

from .rfc9421_signer import (
    RFC9421Signer,
    create_signer,
    sign_request,
)

from .signing_config import (
    SigningConfigBuilder,
    SecurityProfile,
    SECURITY_PROFILES,
    DEFAULT_SIGNATURE_COMPONENTS,
    STRICT_SIGNATURE_COMPONENTS,
    MINIMAL_SIGNATURE_COMPONENTS,
    create_signing_config,
    create_from_profile,
)

from .utils import (
    generate_nonce,
    generate_timestamp,
    calculate_content_digest,
    validate_nonce,
    validate_timestamp,
    validate_signing_private_key,
    parse_url,
    normalize_header_name,
    format_rfc3339_timestamp,
)

from .integration import (
    create_signing_session,
    SigningSession,
    enable_request_signing,
    disable_request_signing,
)

# Public API exports
__all__ = [
    # Core signing functionality
    'RFC9421Signer',
    'create_signer',
    'sign_request',
    # Types
    'SignableRequest',
    'SigningConfig',
    'SignatureComponents',
    'RFC9421SignatureResult',
    'SignatureParams',
    'ContentDigest',
    'SigningOptions',
    'SigningError',
    'DigestAlgorithm',
    'HttpMethod',
    'SignatureAlgorithm',
    # Configuration
    'SigningConfigBuilder',
    'SecurityProfile',
    'SECURITY_PROFILES',
    'DEFAULT_SIGNATURE_COMPONENTS',
    'STRICT_SIGNATURE_COMPONENTS',
    'MINIMAL_SIGNATURE_COMPONENTS',
    'create_signing_config',
    'create_from_profile',
    # Utilities
    'generate_nonce',
    'generate_timestamp',
    'calculate_content_digest',
    'validate_nonce',
    'validate_timestamp',
    'validate_signing_private_key',
    'parse_url',
    'normalize_header_name',
    'format_rfc3339_timestamp',
    # HTTP Integration
    'create_signing_session',
    'SigningSession',
    'enable_request_signing',
    'disable_request_signing',
]