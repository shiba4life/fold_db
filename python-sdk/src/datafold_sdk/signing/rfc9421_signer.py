"""
RFC 9421 HTTP Message Signatures implementation with Ed25519

This module provides the main signer implementation for RFC 9421 HTTP Message Signatures
using Ed25519 digital signatures. It integrates with the existing DataFold SDK crypto
infrastructure.
"""

import time
from typing import Optional

from ..crypto.ed25519 import sign_message
from ..exceptions import UnsupportedPlatformError
from .types import (
    SignableRequest,
    SigningConfig,
    SigningOptions,
    RFC9421SignatureResult,
    SignatureParams,
    SigningContext,
    SigningError,
    SigningErrorCodes,
    ContentDigest,
    SignatureAlgorithm,
    DigestAlgorithm,
    SignatureComponents,
)
from .utils import (
    calculate_content_digest,
    generate_nonce,
    generate_timestamp,
    normalize_header_name,
    validate_nonce,
    validate_timestamp,
    to_hex,
    PerformanceTimer,
)
from .canonical_message import (
    build_canonical_message,
    build_signature_input,
    extract_covered_components,
)
from .signing_config import validate_signing_config


class RFC9421Signer:
    """
    RFC 9421 HTTP Message Signatures signer with Ed25519
    
    This class provides request signing functionality according to RFC 9421
    HTTP Message Signatures specification using Ed25519 digital signatures.
    """
    
    def __init__(self, config: SigningConfig):
        """
        Initialize the signer with configuration.
        
        Args:
            config: Signing configuration
            
        Raises:
            SigningError: If configuration is invalid
        """
        validate_signing_config(config)
        self.config = self._copy_config(config)
    
    def sign_request(
        self,
        request: SignableRequest,
        options: Optional[SigningOptions] = None
    ) -> RFC9421SignatureResult:
        """
        Sign an HTTP request according to RFC 9421.
        
        Args:
            request: Request to sign
            options: Optional signing options to override defaults
            
        Returns:
            RFC9421SignatureResult: Signing result with headers and metadata
            
        Raises:
            SigningError: If signing fails
        """
        timer = PerformanceTimer()
        
        try:
            # Merge options with defaults from config
            effective_options = self._merge_options_with_defaults(options)
            
            # Create signing context
            context = self._create_signing_context(request, effective_options)
            
            # Build canonical message
            canonical_message = build_canonical_message(context)
            
            # Sign the canonical message
            signature_bytes = self._sign_message(canonical_message)
            signature_hex = to_hex(signature_bytes)
            
            # Get covered components for signature input
            covered_components = extract_covered_components(canonical_message)
            
            # Build signature input header
            signature_input = build_signature_input('sig1', covered_components, context.params)
            
            # Build result headers
            headers = {
                'signature-input': signature_input,
                'signature': f'sig1=:{signature_hex}:'
            }
            
            # Add content-digest header if applicable
            if context.content_digest:
                headers['content-digest'] = context.content_digest.header_value
            
            # Add content-type header if not present and we have a body
            if self._should_add_content_type(request, context):
                headers['content-type'] = self._get_default_content_type(request)
            
            elapsed_ms = timer.elapsed_ms()
            
            # Performance check (should be <10ms as per requirements)
            if elapsed_ms > 10:
                import logging
                logger = logging.getLogger(__name__)
                logger.warning(f"Signing operation took {elapsed_ms:.2f}ms (target: <10ms)")
            
            return RFC9421SignatureResult(
                signature_input=signature_input,
                signature=f'sig1=:{signature_hex}:',
                headers=headers,
                canonical_message=canonical_message
            )
            
        except Exception as e:
            if isinstance(e, SigningError):
                raise
            
            raise SigningError(
                f"Request signing failed: {e}",
                SigningErrorCodes.SIGNING_FAILED,
                {"original_error": str(e)}
            )
    
    def _create_signing_context(
        self,
        request: SignableRequest,
        effective_options: SigningOptions
    ) -> SigningContext:
        """
        Create signing context from request and options.
        
        Args:
            request: Request to sign
            options: Signing options
            
        Returns:
            SigningContext: Complete signing context
            
        Raises:
            SigningError: If context creation fails
        """
        try:
            # Generate signature parameters
            nonce = effective_options.nonce
            if nonce is None:
                nonce_gen = self.config.nonce_generator or generate_nonce
                nonce = nonce_gen()
            
            timestamp = effective_options.timestamp
            if timestamp is None:
                timestamp_gen = self.config.timestamp_generator or generate_timestamp
                timestamp = timestamp_gen()
            
            # Validate generated values
            if not validate_nonce(nonce):
                raise SigningError(
                    f"Invalid nonce format: {nonce}",
                    SigningErrorCodes.INVALID_NONCE,
                    {"nonce": nonce}
                )
            
            if not validate_timestamp(timestamp):
                raise SigningError(
                    f"Invalid timestamp: {timestamp}",
                    SigningErrorCodes.INVALID_TIMESTAMP,
                    {"timestamp": timestamp}
                )
            
            params = SignatureParams(
                created=timestamp,
                keyid=self.config.key_id,
                alg=self.config.algorithm,
                nonce=nonce
            )
            
            # Use components from options or config
            components = effective_options.components or self.config.components
            
            # Validate that all required headers are present
            self._validate_required_headers(request, components)
            
            # Calculate content digest if needed
            content_digest = None
            if components.content_digest:
                digest_algorithm = effective_options.digest_algorithm
                if self._has_request_body(request):
                    content_digest = calculate_content_digest(request.body, digest_algorithm)
                else:
                    # Content digest requested but no body - calculate for empty body
                    content_digest = calculate_content_digest(b'', digest_algorithm)
            
            return SigningContext(
                request=request,
                config=self.config,
                options=effective_options,
                params=params,
                components=components,
                content_digest=content_digest
            )
            
        except Exception as e:
            if isinstance(e, SigningError):
                raise
            
            raise SigningError(
                f"Failed to create signing context: {e}",
                SigningErrorCodes.SIGNING_FAILED,
                {"original_error": str(e)}
            )
    
    def _sign_message(self, message: str) -> bytes:
        """
        Sign a message using Ed25519.
        
        Args:
            message: Message to sign
            
        Returns:
            bytes: Ed25519 signature
            
        Raises:
            SigningError: If signing fails
        """
        try:
            # Use the existing SDK sign_message function
            signature = sign_message(self.config.private_key, message)
            return signature
            
        except UnsupportedPlatformError as e:
            raise SigningError(
                f"Cryptography not available: {e}",
                SigningErrorCodes.CRYPTOGRAPHY_UNAVAILABLE,
                {"original_error": str(e)}
            )
        except Exception as e:
            raise SigningError(
                f"Message signing failed: {e}",
                SigningErrorCodes.SIGNING_FAILED,
                {"original_error": str(e)}
            )
    
    def _should_add_content_type(
        self,
        request: SignableRequest,
        context: SigningContext
    ) -> bool:
        """
        Check if content-type header should be added.
        
        Args:
            request: Request being signed
            context: Signing context
            
        Returns:
            bool: True if content-type should be added
        """
        # Only add if we have a body and no content-type is present
        if not self._has_request_body(request):
            return False
        
        # Check if content-type is already present
        for header_name in request.headers:
            if header_name.lower() == 'content-type':
                return False
        
        # Check if content-type is included in signature components
        if context.config.components.headers:
            return 'content-type' in context.config.components.headers
        
        return False
    
    def _get_default_content_type(self, request: SignableRequest) -> str:
        """
        Get default content-type for request.
        
        Args:
            request: Request to get content-type for
            
        Returns:
            str: Default content-type header value
        """
        if isinstance(request.body, str):
            # Try to detect if it's JSON
            body_stripped = request.body.strip()
            if (body_stripped.startswith('{') and body_stripped.endswith('}')) or \
               (body_stripped.startswith('[') and body_stripped.endswith(']')):
                return 'application/json'
        
        # Default for other content
        return 'application/octet-stream'
    
    def _has_request_body(self, request: SignableRequest) -> bool:
        """
        Check if request has a body.
        
        Args:
            request: Request to check
            
        Returns:
            bool: True if request has body content
        """
        if request.body is None:
            return False
        
        if isinstance(request.body, str):
            return len(request.body) > 0
        
        if isinstance(request.body, bytes):
            return len(request.body) > 0
        
        return False
    
    def _copy_config(self, config: SigningConfig) -> SigningConfig:
        """
        Create a defensive copy of signing configuration.
        
        Args:
            config: Original configuration
            
        Returns:
            SigningConfig: Copied configuration
        """
        # Create new instances to avoid mutation
        from .types import SignatureComponents
        
        components_copy = SignatureComponents(
            method=config.components.method,
            target_uri=config.components.target_uri,
            headers=config.components.headers.copy() if config.components.headers else [],
            content_digest=config.components.content_digest
        )
        
        return SigningConfig(
            algorithm=config.algorithm,
            key_id=config.key_id,
            private_key=config.private_key,  # bytes are immutable
            components=components_copy,
            nonce_generator=config.nonce_generator,
            timestamp_generator=config.timestamp_generator
        )
    
    def _merge_options_with_defaults(self, options: Optional[SigningOptions]) -> SigningOptions:
        """
        Merge signing options with defaults from config.
        
        Args:
            options: User-provided options (may be None)
            
        Returns:
            SigningOptions: Merged options with appropriate defaults
        """
        if options is None:
            # Create default options - use SHA256 as reasonable default
            return SigningOptions(digest_algorithm=DigestAlgorithm.SHA256)
        
        return options
    
    def _validate_required_headers(self, request: SignableRequest, components: SignatureComponents) -> None:
        """
        Validate that all required headers are present in the request.
        
        For standard profiles, headers are included if present but not required.
        Only strict validation is enforced for explicitly required headers.
        
        Args:
            request: Request to validate
            components: Signature components requiring headers
            
        Raises:
            SigningError: If required headers are missing
        """
        if not components.headers:
            return
        
        # For standard profiles, most headers are optional (include if present)
        # Only enforce strict validation for non-standard headers that are truly required
        
        # Normalize request headers for case-insensitive comparison
        request_headers = {h.lower() for h in request.headers.keys()}
        
        # Check for truly required headers (skip standard optional ones)
        missing_headers = []
        for required_header in components.headers:
            normalized_required = required_header.lower()
            
            # For standard profiles, some headers are optional (include if present)
            # But for custom configurations, all specified headers are required
            is_standard_optional = normalized_required in ['content-type', 'content-length', 'user-agent', 'authorization']
            is_custom_config = len(components.headers) <= 2 and 'authorization' in [h.lower() for h in components.headers]
            
            # Skip validation only for standard optional headers in standard profiles
            if is_standard_optional and not is_custom_config:
                continue
                
            # Require all headers to be present for custom configs or non-standard headers
            if normalized_required not in request_headers:
                missing_headers.append(required_header)
        
        if missing_headers:
            raise SigningError(
                f"Required headers missing: {', '.join(missing_headers)}",
                SigningErrorCodes.MISSING_REQUIRED_HEADER,
                {
                    "missing_headers": missing_headers,
                    "request_headers": list(request.headers.keys()),
                    "required_headers": components.headers
                }
            )


def create_signer(config: SigningConfig) -> RFC9421Signer:
    """
    Create a new RFC 9421 signer.
    
    Args:
        config: Signing configuration
        
    Returns:
        RFC9421Signer: Configured signer instance
    """
    return RFC9421Signer(config)


def sign_request(
    request: SignableRequest,
    config: SigningConfig,
    options: Optional[SigningOptions] = None
) -> RFC9421SignatureResult:
    """
    Sign a request with the given configuration.
    
    Args:
        request: Request to sign
        config: Signing configuration
        options: Optional signing options
        
    Returns:
        RFC9421SignatureResult: Signing result
    """
    signer = create_signer(config)
    return signer.sign_request(request, options)


def verify_signer_compatibility() -> bool:
    """
    Verify that the signer can function properly.
    
    Returns:
        bool: True if signer is compatible with current environment
    """
    try:
        # Check if cryptography is available
        from ..crypto.ed25519 import check_platform_compatibility
        
        compat = check_platform_compatibility()
        return (
            compat.get('cryptography_available', False) and
            compat.get('ed25519_supported', False)
        )
    except Exception:
        return False