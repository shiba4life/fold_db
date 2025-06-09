"""
Cryptographic operations for DataFold Python SDK
"""

from .ed25519 import (
    Ed25519KeyPair,
    generate_key_pair,
    generate_multiple_key_pairs,
    format_key,
    parse_key,
    clear_key_material,
    check_platform_compatibility,
)

from .storage import (
    SecureKeyStorage,
    StorageMetadata,
    get_default_storage,
)

from .derivation import (
    DerivationParameters,
    derive_key_hkdf,
    derive_key_pbkdf2,
    derive_key_scrypt,
    derive_ed25519_key_pair,
    verify_derivation,
    export_derivation_parameters,
    import_derivation_parameters,
    check_derivation_support,
)

from .rotation import (
    KeyVersion,
    RotationPolicy,
    RotationMetadata,
    KeyRotationManager,
    get_default_rotation_manager,
)

from .backup import (
    BackupMetadata,
    KeyBackupManager,
    get_default_backup_manager,
    export_key_to_file,
    import_key_from_file,
)

__all__ = [
    # Ed25519 key operations
    'Ed25519KeyPair',
    'generate_key_pair',
    'generate_multiple_key_pairs',
    'format_key',
    'parse_key',
    'clear_key_material',
    'check_platform_compatibility',
    
    # Key storage
    'SecureKeyStorage',
    'StorageMetadata',
    'get_default_storage',
    
    # Key derivation
    'DerivationParameters',
    'derive_key_hkdf',
    'derive_key_pbkdf2',
    'derive_key_scrypt',
    'derive_ed25519_key_pair',
    'verify_derivation',
    'export_derivation_parameters',
    'import_derivation_parameters',
    'check_derivation_support',
    
    # Key rotation
    'KeyVersion',
    'RotationPolicy',
    'RotationMetadata',
    'KeyRotationManager',
    'get_default_rotation_manager',
    
    # Key backup and restore
    'BackupMetadata',
    'KeyBackupManager',
    'get_default_backup_manager',
    'export_key_to_file',
    'import_key_from_file',
]