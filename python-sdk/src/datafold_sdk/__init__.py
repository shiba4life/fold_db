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
from .http_client import (
    DataFoldHttpClient,
    ServerConfig,
    create_client,
    PublicKeyRegistration,
    SignatureVerificationResult,
)
from .integration import (
    DataFoldClient,
    ClientSession,
    quick_setup,
    load_existing_client,
    register_and_verify_workflow,
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
    # HTTP Client
    'DataFoldHttpClient',
    'ServerConfig',
    'create_client',
    'PublicKeyRegistration',
    'SignatureVerificationResult',
    # Integration
    'DataFoldClient',
    'ClientSession',
    'quick_setup',
    'load_existing_client',
    'register_and_verify_workflow',
]