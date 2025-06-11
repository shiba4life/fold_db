"""
Configuration management for request signing

This module provides configuration management for RFC 9421 HTTP Message Signatures,
including security profiles, configuration builders, and validation.
"""

from typing import Dict, List, Optional, Callable
from dataclasses import dataclass, field

from .types import (
    SigningConfig,
    SignatureComponents,
    SignatureAlgorithm,
    DigestAlgorithm,
    SigningError,
    SigningErrorCodes,
    NonceGenerator,
    TimestampGenerator,
)
from .utils import (
    generate_nonce,
    generate_timestamp,
    validate_signing_private_key,
    validate_nonce,
    validate_timestamp,
)


# Default signature components following server requirements
DEFAULT_SIGNATURE_COMPONENTS = SignatureComponents(
    method=True,
    target_uri=True,
    headers=['content-type'],
    content_digest=True
)

# Strict signature components for high-security scenarios
STRICT_SIGNATURE_COMPONENTS = SignatureComponents(
    method=True,
    target_uri=True,
    headers=['content-type', 'content-length', 'user-agent', 'authorization'],
    content_digest=True
)

# Minimal signature components for basic signing
MINIMAL_SIGNATURE_COMPONENTS = SignatureComponents(
    method=True,
    target_uri=True,
    headers=[],
    content_digest=False
)


@dataclass
class SecurityProfile:
    """
    Security profile for different use cases
    
    Attributes:
        name: Profile name
        description: Profile description
        components: Signature components to include
        digest_algorithm: Content digest algorithm
        validate_nonces: Whether to validate nonce format
        allow_custom_nonces: Whether to allow custom nonces
    """
    name: str
    description: str
    components: SignatureComponents
    digest_algorithm: DigestAlgorithm
    validate_nonces: bool
    allow_custom_nonces: bool


# Predefined security profiles
SECURITY_PROFILES: Dict[str, SecurityProfile] = {
    'strict': SecurityProfile(
        name='Strict',
        description='Maximum security with comprehensive signature coverage',
        components=STRICT_SIGNATURE_COMPONENTS,
        digest_algorithm=DigestAlgorithm.SHA512,
        validate_nonces=True,
        allow_custom_nonces=False
    ),
    
    'standard': SecurityProfile(
        name='Standard',
        description='Balanced security suitable for most applications',
        components=DEFAULT_SIGNATURE_COMPONENTS,
        digest_algorithm=DigestAlgorithm.SHA256,
        validate_nonces=True,
        allow_custom_nonces=True
    ),
    
    'minimal': SecurityProfile(
        name='Minimal',
        description='Basic signing for low-latency scenarios',
        components=MINIMAL_SIGNATURE_COMPONENTS,
        digest_algorithm=DigestAlgorithm.SHA256,
        validate_nonces=False,
        allow_custom_nonces=True
    )
}


class SigningConfigBuilder:
    """
    Builder for creating signing configurations with fluent API
    """
    
    def __init__(self):
        self._algorithm: SignatureAlgorithm = SignatureAlgorithm.ED25519
        self._key_id: Optional[str] = None
        self._private_key: Optional[bytes] = None
        # Create a copy to avoid sharing state between instances
        self._components: SignatureComponents = SignatureComponents(
            method=DEFAULT_SIGNATURE_COMPONENTS.method,
            target_uri=DEFAULT_SIGNATURE_COMPONENTS.target_uri,
            headers=DEFAULT_SIGNATURE_COMPONENTS.headers.copy() if DEFAULT_SIGNATURE_COMPONENTS.headers else [],
            content_digest=DEFAULT_SIGNATURE_COMPONENTS.content_digest
        )
        self._nonce_generator: Optional[NonceGenerator] = None
        self._timestamp_generator: Optional[TimestampGenerator] = None
    
    def algorithm(self, algorithm: SignatureAlgorithm) -> 'SigningConfigBuilder':
        """
        Set signature algorithm.
        
        Args:
            algorithm: Signature algorithm to use
            
        Returns:
            SigningConfigBuilder: Self for method chaining
        """
        self._algorithm = algorithm
        return self
    
    def key_id(self, key_id: str) -> 'SigningConfigBuilder':
        """
        Set key identifier.
        
        Args:
            key_id: Key identifier for the signature
            
        Returns:
            SigningConfigBuilder: Self for method chaining
        """
        self._key_id = key_id
        return self
    
    def private_key(self, private_key: bytes) -> 'SigningConfigBuilder':
        """
        Set private key for signing.
        
        Args:
            private_key: Ed25519 private key bytes (32 bytes)
            
        Returns:
            SigningConfigBuilder: Self for method chaining
        """
        self._private_key = private_key
        return self
    
    def components(self, components: SignatureComponents) -> 'SigningConfigBuilder':
        """
        Set signature components.
        
        Args:
            components: Components to include in signature
            
        Returns:
            SigningConfigBuilder: Self for method chaining
        """
        self._components = components
        return self
    
    def method(self, include: bool = True) -> 'SigningConfigBuilder':
        """
        Include/exclude @method component.
        
        Args:
            include: Whether to include @method component
            
        Returns:
            SigningConfigBuilder: Self for method chaining
        """
        self._components.method = include
        return self
    
    def target_uri(self, include: bool = True) -> 'SigningConfigBuilder':
        """
        Include/exclude @target-uri component.
        
        Args:
            include: Whether to include @target-uri component
            
        Returns:
            SigningConfigBuilder: Self for method chaining
        """
        self._components.target_uri = include
        return self
    
    def headers(self, headers: List[str]) -> 'SigningConfigBuilder':
        """
        Set headers to include in signature.
        
        Args:
            headers: List of header names to include
            
        Returns:
            SigningConfigBuilder: Self for method chaining
        """
        self._components.headers = [h.lower() for h in headers]
        return self
    
    def add_header(self, header: str) -> 'SigningConfigBuilder':
        """
        Add header to signature components.
        
        Args:
            header: Header name to add
            
        Returns:
            SigningConfigBuilder: Self for method chaining
        """
        if self._components.headers is None:
            self._components.headers = []
        
        header_lower = header.lower()
        if header_lower not in self._components.headers:
            self._components.headers.append(header_lower)
        
        return self
    
    def content_digest(self, include: bool = True) -> 'SigningConfigBuilder':
        """
        Include/exclude content digest component.
        
        Args:
            include: Whether to include content digest
            
        Returns:
            SigningConfigBuilder: Self for method chaining
        """
        self._components.content_digest = include
        return self
    
    def nonce_generator(self, generator: NonceGenerator) -> 'SigningConfigBuilder':
        """
        Set custom nonce generator.
        
        Args:
            generator: Function that returns nonce strings
            
        Returns:
            SigningConfigBuilder: Self for method chaining
        """
        self._nonce_generator = generator
        return self
    
    def timestamp_generator(self, generator: TimestampGenerator) -> 'SigningConfigBuilder':
        """
        Set custom timestamp generator.
        
        Args:
            generator: Function that returns Unix timestamps
            
        Returns:
            SigningConfigBuilder: Self for method chaining
        """
        self._timestamp_generator = generator
        return self
    
    def profile(self, profile_name: str) -> 'SigningConfigBuilder':
        """
        Apply security profile.
        
        Args:
            profile_name: Name of security profile ('strict', 'standard', 'minimal')
            
        Returns:
            SigningConfigBuilder: Self for method chaining
            
        Raises:
            SigningError: If profile name is invalid
        """
        if profile_name not in SECURITY_PROFILES:
            raise SigningError(
                f"Unknown security profile: {profile_name}",
                SigningErrorCodes.INVALID_CONFIG,
                {"available_profiles": list(SECURITY_PROFILES.keys())}
            )
        
        profile = SECURITY_PROFILES[profile_name]
        self._components = SignatureComponents(
            method=profile.components.method,
            target_uri=profile.components.target_uri,
            headers=profile.components.headers.copy() if profile.components.headers else [],
            content_digest=profile.components.content_digest
        )
        
        return self
    
    def build(self) -> SigningConfig:
        """
        Build the signing configuration.
        
        Returns:
            SigningConfig: Complete signing configuration
            
        Raises:
            SigningError: If configuration is invalid
        """
        if self._key_id is None:
            raise SigningError(
                "Key ID is required",
                SigningErrorCodes.INVALID_CONFIG
            )
        
        if self._private_key is None:
            raise SigningError(
                "Private key is required",
                SigningErrorCodes.INVALID_CONFIG
            )
        
        # Validate private key
        if not validate_signing_private_key(self._private_key):
            raise SigningError(
                "Invalid private key format",
                SigningErrorCodes.INVALID_PRIVATE_KEY
            )
        
        # Use default generators if not provided
        nonce_gen = self._nonce_generator or generate_nonce
        timestamp_gen = self._timestamp_generator or generate_timestamp
        
        return SigningConfig(
            algorithm=self._algorithm,
            key_id=self._key_id,
            private_key=self._private_key,
            components=self._components,
            nonce_generator=nonce_gen,
            timestamp_generator=timestamp_gen
        )


def create_signing_config() -> SigningConfigBuilder:
    """
    Create a new signing configuration builder.
    
    Returns:
        SigningConfigBuilder: New configuration builder
    """
    return SigningConfigBuilder()


def create_from_profile(
    profile_name: str,
    key_id: str,
    private_key: bytes
) -> SigningConfig:
    """
    Create signing configuration from security profile.
    
    Args:
        profile_name: Security profile name
        key_id: Key identifier
        private_key: Ed25519 private key bytes
        
    Returns:
        SigningConfig: Complete signing configuration
        
    Raises:
        SigningError: If profile or parameters are invalid
    """
    return (create_signing_config()
            .profile(profile_name)
            .key_id(key_id)
            .private_key(private_key)
            .build())


def validate_signing_config(config: SigningConfig) -> None:
    """
    Validate signing configuration.
    
    Args:
        config: Signing configuration to validate
        
    Raises:
        SigningError: If configuration is invalid
    """
    if not isinstance(config, SigningConfig):
        raise SigningError(
            "Configuration must be SigningConfig instance",
            SigningErrorCodes.INVALID_CONFIG
        )
    
    # Validate algorithm
    if config.algorithm != SignatureAlgorithm.ED25519:
        raise SigningError(
            f"Unsupported algorithm: {config.algorithm}",
            SigningErrorCodes.INVALID_CONFIG
        )
    
    # Validate key ID
    if not config.key_id or not isinstance(config.key_id, str):
        raise SigningError(
            "Key ID must be non-empty string",
            SigningErrorCodes.INVALID_KEY_ID
        )
    
    # Validate private key
    if not validate_signing_private_key(config.private_key):
        raise SigningError(
            "Invalid private key format",
            SigningErrorCodes.INVALID_PRIVATE_KEY
        )
    
    # Validate components
    if not isinstance(config.components, SignatureComponents):
        raise SigningError(
            "Components must be SignatureComponents instance",
            SigningErrorCodes.INVALID_SIGNATURE_COMPONENTS
        )
    
    # Ensure at least one component is enabled
    has_components = (
        config.components.method or
        config.components.target_uri or
        (config.components.headers and len(config.components.headers) > 0) or
        config.components.content_digest
    )
    
    if not has_components:
        raise SigningError(
            "At least one signature component must be enabled",
            SigningErrorCodes.INVALID_SIGNATURE_COMPONENTS
        )
    
    # Test generators if provided
    if config.nonce_generator:
        try:
            test_nonce = config.nonce_generator()
            if not isinstance(test_nonce, str) or not test_nonce:
                raise SigningError(
                    "Nonce generator must return non-empty string",
                    SigningErrorCodes.INVALID_CONFIG
                )
        except Exception as e:
            raise SigningError(
                f"Nonce generator failed: {e}",
                SigningErrorCodes.INVALID_CONFIG,
                {"original_error": str(e)}
            )
    
    if config.timestamp_generator:
        try:
            test_timestamp = config.timestamp_generator()
            if not isinstance(test_timestamp, int) or not validate_timestamp(test_timestamp):
                raise SigningError(
                    "Timestamp generator must return valid Unix timestamp",
                    SigningErrorCodes.INVALID_CONFIG
                )
        except Exception as e:
            raise SigningError(
                f"Timestamp generator failed: {e}",
                SigningErrorCodes.INVALID_CONFIG,
                {"original_error": str(e)}
            )


def get_security_profile(name: str) -> SecurityProfile:
    """
    Get security profile by name.
    
    Args:
        name: Profile name
        
    Returns:
        SecurityProfile: Security profile
        
    Raises:
        SigningError: If profile name is invalid
    """
    if name not in SECURITY_PROFILES:
        raise SigningError(
            f"Unknown security profile: {name}",
            SigningErrorCodes.INVALID_CONFIG,
            {"available_profiles": list(SECURITY_PROFILES.keys())}
        )
    
    return SECURITY_PROFILES[name]


def list_security_profiles() -> List[str]:
    """
    List available security profile names.
    
    Returns:
        list: List of available profile names
    """
    return list(SECURITY_PROFILES.keys())