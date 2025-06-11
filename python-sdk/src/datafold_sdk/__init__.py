"""
DataFold Python SDK
Client-side key management with Ed25519 support
"""

from .version import __version__
from .crypto.ed25519 import (
    Ed25519KeyPair,
    generate_key_pair,
    generate_multiple_key_pairs,
    format_key,
    parse_key,
    clear_key_material,
    check_platform_compatibility,
    sign_message,
    verify_signature,
)
from .crypto.storage import (
    SecureKeyStorage,
    StorageMetadata,
    get_default_storage,
)
from .crypto.backup import (
    KeyBackupManager,
    BackupMetadata,
    get_default_backup_manager,
    export_key_to_file,
    import_key_from_file,
)
from .exceptions import (
    Ed25519KeyError,
    ValidationError,
    UnsupportedPlatformError,
    StorageError,
    KeyDerivationError,
    KeyRotationError,
    KeyExportError,
    KeyImportError,
    BackupError,
    ServerCommunicationError,
)
# Import original HTTP client directly from the module file
from .http_client import (
    DataFoldHttpClient,
    ServerConfig,
    create_client,
    PublicKeyRegistration,
    SignatureVerificationResult,
)
# Import enhanced HTTP client from the new directory
from .http_clients import (
    # Enhanced HTTP client with automatic signature injection
    EnhancedDataFoldHttpClient,
    HttpClientConfig,
    SigningMode,
    EndpointSigningConfig,
    RequestContext,
    SigningMetrics,
    SignatureCache,
    RequestInterceptor,
    ResponseInterceptor,
    create_enhanced_http_client,
    create_signed_http_client,
    create_fluent_http_client,
    FluentHttpClientBuilder,
    create_correlation_middleware,
    create_logging_middleware,
    create_performance_middleware,
    create_retry_middleware,
)
# Re-enable integration module since circular import is resolved
from .integration import (
    DataFoldClient,
    ClientSession,
    quick_setup,
    load_existing_client,
    register_and_verify_workflow,
)
from .signing import (
    # Core signing functionality
    RFC9421Signer,
    create_signer,
    sign_request,
    # Types
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
    # Configuration
    SigningConfigBuilder,
    SecurityProfile,
    SECURITY_PROFILES,
    DEFAULT_SIGNATURE_COMPONENTS,
    STRICT_SIGNATURE_COMPONENTS,
    MINIMAL_SIGNATURE_COMPONENTS,
    create_signing_config,
    create_from_profile,
    # Utilities
    generate_nonce,
    generate_timestamp,
    calculate_content_digest,
    validate_nonce,
    validate_timestamp,
    validate_signing_private_key,
    parse_url,
    normalize_header_name,
    format_rfc3339_timestamp,
    # HTTP Integration
    create_signing_session,
    SigningSession,
    enable_request_signing,
    disable_request_signing,
)

# Initialize the SDK
def initialize_sdk():
    """
    Initialize the DataFold SDK and check platform compatibility.
    
    Returns:
        dict: Compatibility information with 'compatible' (bool) and 'warnings' (list)
    """
    warnings = []
    compatible = True
    
    try:
        compat_info = check_platform_compatibility()
        if not compat_info['cryptography_available']:
            warnings.append('Cryptography package not available - key generation will fail')
            compatible = False
        
        if not compat_info['secure_random_available']:
            warnings.append('Secure random generation not available - key generation may be insecure')
            compatible = False
            
        if not compat_info['ed25519_supported']:
            warnings.append('Ed25519 not supported by cryptography package - check version')
            compatible = False
            
    except Exception as e:
        warnings.append(f'Platform compatibility check failed: {e}')
        compatible = False
    
    return {
        'compatible': compatible,
        'warnings': warnings
    }


def is_compatible():
    """
    Quick synchronous compatibility check.
    
    Returns:
        bool: True if platform is compatible with basic SDK functionality
    """
    try:
        result = initialize_sdk()
        return result['compatible']
    except Exception:
        return False


# Public API exports
__all__ = [
    '__version__',
    'Ed25519KeyPair',
    'generate_key_pair',
    'generate_multiple_key_pairs',
    'format_key',
    'parse_key',
    'clear_key_material',
    'check_platform_compatibility',
    'sign_message',
    'verify_signature',
    'initialize_sdk',
    'is_compatible',
    # Storage
    'SecureKeyStorage',
    'StorageMetadata',
    'get_default_storage',
    # Backup and Recovery
    'KeyBackupManager',
    'BackupMetadata',
    'get_default_backup_manager',
    'export_key_to_file',
    'import_key_from_file',
    # Exceptions
    'Ed25519KeyError',
    'ValidationError',
    'UnsupportedPlatformError',
    'StorageError',
    'KeyDerivationError',
    'KeyRotationError',
    'KeyExportError',
    'KeyImportError',
    'BackupError',
    'ServerCommunicationError',
    # HTTP Client - Original
    'DataFoldHttpClient',
    'ServerConfig',
    'create_client',
    'PublicKeyRegistration',
    'SignatureVerificationResult',
    # HTTP Client - Enhanced with Automatic Signature Injection
    'EnhancedDataFoldHttpClient',
    'HttpClientConfig',
    'SigningMode',
    'EndpointSigningConfig',
    'RequestContext',
    'SigningMetrics',
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
    # Integration
    'DataFoldClient',
    'ClientSession',
    'quick_setup',
    'load_existing_client',
    'register_and_verify_workflow',
    # Request Signing - Core
    'RFC9421Signer',
    'create_signer',
    'sign_request',
    # Request Signing - Types
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
    # Request Signing - Configuration
    'SigningConfigBuilder',
    'SecurityProfile',
    'SECURITY_PROFILES',
    'DEFAULT_SIGNATURE_COMPONENTS',
    'STRICT_SIGNATURE_COMPONENTS',
    'MINIMAL_SIGNATURE_COMPONENTS',
    'create_signing_config',
    'create_from_profile',
    # Request Signing - Utilities
    'generate_nonce',
    'generate_timestamp',
    'calculate_content_digest',
    'validate_nonce',
    'validate_timestamp',
    'validate_signing_private_key',
    'parse_url',
    'normalize_header_name',
    'format_rfc3339_timestamp',
    # Request Signing - HTTP Integration
    'create_signing_session',
    'SigningSession',
    'enable_request_signing',
    'disable_request_signing',
]