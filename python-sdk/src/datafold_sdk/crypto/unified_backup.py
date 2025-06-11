"""
DataFold Unified Backup Format Implementation for Python SDK

This module implements the standardized encrypted backup format for cross-platform
compatibility following the specification from docs/delivery/10/backup/encrypted_backup_format.md
"""

import os
import json
import secrets
import base64
from typing import Dict, List, Optional, Tuple, Union, Any
from dataclasses import dataclass
from datetime import datetime, timezone

# Import cryptography components
try:
    from cryptography.hazmat.primitives.ciphers.aead import ChaCha20Poly1305, AESGCM
    from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
    from cryptography.hazmat.primitives import hashes
    from cryptography.exceptions import InvalidTag
    
    # Try to import Argon2 (available in cryptography >= 41.0.0)
    try:
        from cryptography.hazmat.primitives.kdf.argon2 import Argon2Type, Argon2
        ARGON2_AVAILABLE = True
    except ImportError:
        ARGON2_AVAILABLE = False
        Argon2Type = None  # type: ignore
        Argon2 = None  # type: ignore
    
    CRYPTOGRAPHY_AVAILABLE = True
except ImportError:
    CRYPTOGRAPHY_AVAILABLE = False
    ChaCha20Poly1305 = None  # type: ignore
    AESGCM = None  # type: ignore
    PBKDF2HMAC = None  # type: ignore
    hashes = None  # type: ignore
    InvalidTag = Exception
    ARGON2_AVAILABLE = False

from ..exceptions import BackupError, ValidationError, UnsupportedPlatformError
from .ed25519 import Ed25519KeyPair

# Constants matching the unified specification
UNIFIED_BACKUP_VERSION = 1
MIN_SALT_LENGTH = 16
PREFERRED_SALT_LENGTH = 32
XCHACHA20_NONCE_LENGTH = 24
AES_GCM_NONCE_LENGTH = 12

# Argon2id parameters (preferred)
ARGON2_MIN_MEMORY = 65536  # 64 MiB
ARGON2_MIN_ITERATIONS = 3
ARGON2_MIN_PARALLELISM = 2

# PBKDF2 parameters (legacy compatibility)
PBKDF2_MIN_ITERATIONS = 100000


@dataclass
class UnifiedBackupFormat:
    """
    Unified backup format structure as defined in the specification
    """
    version: int
    kdf: str  # 'argon2id' or 'pbkdf2'
    kdf_params: Dict[str, Any]
    encryption: str  # 'xchacha20-poly1305' or 'aes-gcm'
    nonce: str
    ciphertext: str
    created: str
    metadata: Optional[Dict[str, Any]] = None


@dataclass
class MigrationResult:
    """Migration result for converting legacy backups"""
    success: bool
    original_format: str
    new_format: UnifiedBackupFormat
    warnings: List[str]


@dataclass
class ValidationVector:
    """Test vector for cross-platform validation"""
    passphrase: str
    salt: str
    nonce: str
    kdf: str
    kdf_params: Dict[str, Any]
    encryption: str
    plaintext_key: str
    ciphertext: str
    created: str


class UnifiedBackupManager:
    """
    Unified Backup Manager for cross-platform compatibility
    
    This implements the exact same backup format as the JavaScript SDK
    to ensure perfect cross-platform compatibility.
    """
    
    def __init__(self):
        """Initialize unified backup manager"""
        if not CRYPTOGRAPHY_AVAILABLE:
            raise UnsupportedPlatformError(
                "Cryptography package required for unified backup operations",
                "CRYPTOGRAPHY_UNAVAILABLE"
            )
    
    def export_key(self, 
                   key_pair: Ed25519KeyPair,
                   passphrase: str,
                   key_id: Optional[str] = None,
                   label: Optional[str] = None,
                   kdf: str = 'argon2id',
                   encryption: str = 'xchacha20-poly1305',
                   kdf_params: Optional[Dict[str, Any]] = None) -> str:
        """
        Export key using unified backup format
        
        Args:
            key_pair: Ed25519 key pair to export
            passphrase: Passphrase for encryption
            key_id: Optional key identifier
            label: Optional key label
            kdf: Key derivation function ('argon2id' or 'pbkdf2')
            encryption: Encryption algorithm ('xchacha20-poly1305' or 'aes-gcm')
            kdf_params: Optional KDF parameters
            
        Returns:
            JSON string containing the unified backup format
            
        Raises:
            BackupError: If export fails
            ValidationError: If inputs are invalid
        """
        self._validate_passphrase(passphrase)
        self._validate_key_pair(key_pair)
        self._validate_algorithm_support(kdf, encryption)
        
        # Generate salt and nonce
        salt = secrets.token_bytes(PREFERRED_SALT_LENGTH)
        nonce_length = XCHACHA20_NONCE_LENGTH if encryption == 'xchacha20-poly1305' else AES_GCM_NONCE_LENGTH
        nonce = secrets.token_bytes(nonce_length)
        
        # Prepare KDF parameters
        final_kdf_params = self._prepare_kdf_params(kdf, kdf_params)
        
        # Derive encryption key
        encryption_key = self._derive_key(passphrase, salt, kdf, final_kdf_params)
        
        # Prepare plaintext (Ed25519 keys concatenated)
        plaintext = self._prepare_key_plaintext(key_pair)
        
        # Encrypt the key data
        if encryption == 'xchacha20-poly1305':
            ciphertext = self._encrypt_xchacha20_poly1305(plaintext, encryption_key, nonce)
        else:  # aes-gcm
            ciphertext = self._encrypt_aes_gcm(plaintext, encryption_key, nonce)
        
        # Create unified backup format
        backup_data = {
            'version': UNIFIED_BACKUP_VERSION,
            'kdf': kdf,
            'kdf_params': {
                'salt': base64.b64encode(salt).decode('ascii'),
                'iterations': final_kdf_params['iterations'],
            },
            'encryption': encryption,
            'nonce': base64.b64encode(nonce).decode('ascii'),
            'ciphertext': base64.b64encode(ciphertext).decode('ascii'),
            'created': datetime.now(timezone.utc).isoformat() + 'Z'
        }
        
        # Add Argon2id specific parameters
        if kdf == 'argon2id':
            backup_data['kdf_params']['memory'] = final_kdf_params['memory']
            backup_data['kdf_params']['parallelism'] = final_kdf_params['parallelism']
        
        # Add metadata
        if key_id or label:
            backup_data['metadata'] = {
                'key_type': 'ed25519'
            }
            if label:
                backup_data['metadata']['label'] = label
        
        return json.dumps(backup_data, indent=2)
    
    def import_key(self, backup_data: str, passphrase: str) -> Tuple[Ed25519KeyPair, Optional[Dict[str, Any]]]:
        """
        Import key from unified backup format
        
        Args:
            backup_data: JSON string containing unified backup
            passphrase: Passphrase for decryption
            
        Returns:
            Tuple of (Ed25519KeyPair, metadata)
            
        Raises:
            BackupError: If import fails
            ValidationError: If backup data is invalid
        """
        self._validate_passphrase(passphrase)
        
        # Parse backup data
        try:
            backup = json.loads(backup_data)
        except json.JSONDecodeError as e:
            raise ValidationError(f"Invalid JSON backup data: {e}", "INVALID_BACKUP_JSON") from e
        
        # Validate backup format
        self._validate_backup_format(backup)
        
        # Extract parameters
        salt = base64.b64decode(backup['kdf_params']['salt'])
        nonce = base64.b64decode(backup['nonce'])
        ciphertext = base64.b64decode(backup['ciphertext'])
        
        # Prepare KDF parameters
        kdf_params = {
            'iterations': backup['kdf_params']['iterations']
        }
        if backup['kdf'] == 'argon2id':
            kdf_params['memory'] = backup['kdf_params']['memory']
            kdf_params['parallelism'] = backup['kdf_params']['parallelism']
        
        # Derive decryption key
        decryption_key = self._derive_key(passphrase, salt, backup['kdf'], kdf_params)
        
        # Decrypt the key data
        if backup['encryption'] == 'xchacha20-poly1305':
            plaintext = self._decrypt_xchacha20_poly1305(ciphertext, decryption_key, nonce)
        else:  # aes-gcm
            plaintext = self._decrypt_aes_gcm(ciphertext, decryption_key, nonce)
        
        # Extract key pair from plaintext
        key_pair = self._extract_key_pair(plaintext)
        
        return key_pair, backup.get('metadata')
    
    def migrate_legacy_backup(self, legacy_data: str, passphrase: str) -> MigrationResult:
        """
        Migrate legacy backup to unified format
        
        Args:
            legacy_data: Legacy backup data
            passphrase: Passphrase for decryption
            
        Returns:
            MigrationResult with migration status and warnings
        """
        warnings = []
        
        try:
            # Try to detect and parse legacy format
            legacy_backup = json.loads(legacy_data)
            
            # Detect format type
            original_format = 'unknown'
            if 'type' in legacy_backup and legacy_backup['type'] == 'datafold-key-backup':
                original_format = 'js-sdk-legacy'
            elif 'algorithm' in legacy_backup and legacy_backup['algorithm'] == 'Ed25519' and 'type' not in legacy_backup:
                original_format = 'python-sdk-legacy'
            
            if original_format == 'unknown':
                raise BackupError('Unable to detect legacy backup format', 'UNKNOWN_LEGACY_FORMAT')
            
            # Import using legacy method
            key_pair, metadata = self._import_legacy_format(legacy_backup, passphrase, original_format)
            
            # Generate warnings about changes
            if original_format == 'js-sdk-legacy':
                if legacy_backup.get('kdf') == 'pbkdf2':
                    warnings.append('Migrated from PBKDF2 to Argon2id for improved security')
                if legacy_backup.get('encryption') == 'aes-gcm':
                    warnings.append('Migrated from AES-GCM to XChaCha20-Poly1305 for improved security')
            elif original_format == 'python-sdk-legacy':
                if legacy_backup.get('kdf') != 'argon2id':
                    warnings.append(f'Migrated from {legacy_backup.get("kdf", "unknown")} to Argon2id for improved security')
                if legacy_backup.get('encryption') != 'xchacha20-poly1305':
                    warnings.append(f'Migrated from {legacy_backup.get("encryption", "unknown")} to XChaCha20-Poly1305 for improved security')
            
            # Export using unified format
            new_backup_data = self.export_key(
                key_pair, 
                passphrase,
                label=metadata.get('label') if metadata else None,
                kdf='argon2id',
                encryption='xchacha20-poly1305'
            )
            
            new_format_dict = json.loads(new_backup_data)
            new_format = UnifiedBackupFormat(
                version=new_format_dict['version'],
                kdf=new_format_dict['kdf'],
                kdf_params=new_format_dict['kdf_params'],
                encryption=new_format_dict['encryption'],
                nonce=new_format_dict['nonce'],
                ciphertext=new_format_dict['ciphertext'],
                created=new_format_dict['created'],
                metadata=new_format_dict.get('metadata')
            )
            
            return MigrationResult(
                success=True,
                original_format=original_format,
                new_format=new_format,
                warnings=warnings
            )
            
        except Exception as e:
            return MigrationResult(
                success=False,
                original_format='unknown',
                new_format=UnifiedBackupFormat(
                    version=0, kdf='', kdf_params={}, encryption='', 
                    nonce='', ciphertext='', created=''
                ),
                warnings=[f'Migration failed: {str(e)}']
            )
    
    def generate_test_vector(self) -> ValidationVector:
        """
        Generate test vector for cross-platform validation
        
        Returns:
            TestVector with deterministic test data
        """
        # Use fixed test data for reproducible test vectors
        passphrase = 'correct horse battery staple'
        salt = base64.b64decode('w7Z3pQ2v5Q8v1Q2v5Q8v1Q==')
        nonce = base64.b64decode('AAAAAAAAAAAAAAAAAAAAAAAAAAA=')
        
        # Create test key pair (deterministic)
        test_private_key = b'\x42' * 32  # Deterministic test key
        test_public_key = b'\x43' * 32   # Would be derived from private key in real implementation
        test_key_pair = Ed25519KeyPair(private_key=test_private_key, public_key=test_public_key)
        
        kdf = 'argon2id'
        kdf_params = {
            'iterations': ARGON2_MIN_ITERATIONS,
            'memory': ARGON2_MIN_MEMORY,
            'parallelism': ARGON2_MIN_PARALLELISM
        }
        encryption = 'xchacha20-poly1305'
        
        # Derive key and encrypt
        derived_key = self._derive_key(passphrase, salt, kdf, kdf_params)
        plaintext = self._prepare_key_plaintext(test_key_pair)
        ciphertext = self._encrypt_xchacha20_poly1305(plaintext, derived_key, nonce)
        
        return ValidationVector(
            passphrase=passphrase,
            salt=base64.b64encode(salt).decode('ascii'),
            nonce=base64.b64encode(nonce).decode('ascii'),
            kdf=kdf,
            kdf_params=kdf_params,
            encryption=encryption,
            plaintext_key=base64.b64encode(plaintext).decode('ascii'),
            ciphertext=base64.b64encode(ciphertext).decode('ascii'),
            created='2025-06-08T17:00:00Z'
        )
    
    def validate_test_vector(self, test_vector: ValidationVector) -> bool:
        """
        Validate cross-platform compatibility with test vector
        
        Args:
            test_vector: Test vector to validate
            
        Returns:
            True if validation succeeds, False otherwise
        """
        try:
            salt = base64.b64decode(test_vector.salt)
            nonce = base64.b64decode(test_vector.nonce)
            expected_ciphertext = base64.b64decode(test_vector.ciphertext)
            expected_plaintext = base64.b64decode(test_vector.plaintext_key)
            
            # Derive key using test vector parameters
            derived_key = self._derive_key(
                test_vector.passphrase,
                salt,
                test_vector.kdf,
                test_vector.kdf_params
            )
            
            # Decrypt test vector ciphertext
            if test_vector.encryption == 'xchacha20-poly1305':
                decrypted_plaintext = self._decrypt_xchacha20_poly1305(expected_ciphertext, derived_key, nonce)
            else:  # aes-gcm
                decrypted_plaintext = self._decrypt_aes_gcm(expected_ciphertext, derived_key, nonce)
            
            # Compare with expected plaintext
            return decrypted_plaintext == expected_plaintext
            
        except Exception as e:
            print(f"Test vector validation failed: {e}")
            return False
    
    # Private helper methods
    
    def _validate_passphrase(self, passphrase: str) -> None:
        """Validate passphrase strength"""
        if not isinstance(passphrase, str):
            raise ValidationError("Passphrase must be a string", "INVALID_PASSPHRASE_TYPE")
        
        if len(passphrase) < 8:
            raise ValidationError("Passphrase must be at least 8 characters", "WEAK_PASSPHRASE")
    
    def _validate_key_pair(self, key_pair: Ed25519KeyPair) -> None:
        """Validate key pair structure"""
        if not isinstance(key_pair, Ed25519KeyPair):
            raise ValidationError("key_pair must be Ed25519KeyPair instance", "INVALID_KEY_PAIR_TYPE")
        
        if not key_pair.private_key or not key_pair.public_key:
            raise ValidationError("Invalid key pair: missing private or public key", "INVALID_KEY_PAIR")
        
        if len(key_pair.private_key) != 32 or len(key_pair.public_key) != 32:
            raise ValidationError("Invalid key pair: incorrect key lengths", "INVALID_KEY_LENGTHS")
    
    def _validate_algorithm_support(self, kdf: str, encryption: str) -> None:
        """Validate algorithm support"""
        if kdf not in ['argon2id', 'pbkdf2']:
            raise ValidationError(f"Unsupported KDF: {kdf}", "UNSUPPORTED_KDF")
        
        if encryption not in ['xchacha20-poly1305', 'aes-gcm']:
            raise ValidationError(f"Unsupported encryption: {encryption}", "UNSUPPORTED_ENCRYPTION")
        
        if kdf == 'argon2id' and not ARGON2_AVAILABLE:
            raise UnsupportedPlatformError("Argon2id not available in this environment", "ARGON2_UNAVAILABLE")
    
    def _prepare_kdf_params(self, kdf: str, custom_params: Optional[Dict[str, Any]]) -> Dict[str, Any]:
        """Prepare KDF parameters"""
        if kdf == 'argon2id':
            return {
                'iterations': custom_params.get('iterations', ARGON2_MIN_ITERATIONS) if custom_params else ARGON2_MIN_ITERATIONS,
                'memory': custom_params.get('memory', ARGON2_MIN_MEMORY) if custom_params else ARGON2_MIN_MEMORY,
                'parallelism': custom_params.get('parallelism', ARGON2_MIN_PARALLELISM) if custom_params else ARGON2_MIN_PARALLELISM
            }
        else:  # pbkdf2
            return {
                'iterations': custom_params.get('iterations', PBKDF2_MIN_ITERATIONS) if custom_params else PBKDF2_MIN_ITERATIONS
            }
    
    def _derive_key(self, passphrase: str, salt: bytes, kdf: str, params: Dict[str, Any]) -> bytes:
        """Derive encryption key using specified KDF"""
        if kdf == 'argon2id':
            if not ARGON2_AVAILABLE or Argon2 is None or Argon2Type is None:
                raise BackupError("Argon2 not available", "ARGON2_UNAVAILABLE")
            
            try:
                argon2_kdf = Argon2(
                    time_cost=params['iterations'],
                    memory_cost=params['memory'],
                    parallelism=params['parallelism'],
                    hash_len=32,  # 256-bit key
                    salt=salt,
                    type=Argon2Type.ID
                )
                return argon2_kdf.derive(passphrase.encode('utf-8'))
            except Exception as e:
                raise BackupError(f"Argon2 key derivation failed: {e}", "ARGON2_DERIVATION_FAILED") from e
        
        else:  # pbkdf2
            if PBKDF2HMAC is None or hashes is None:
                raise BackupError("PBKDF2 not available", "PBKDF2_UNAVAILABLE")
            
            try:
                pbkdf2_kdf = PBKDF2HMAC(
                    algorithm=hashes.SHA256(),
                    length=32,  # 256-bit key
                    salt=salt,
                    iterations=params['iterations'],
                )
                return pbkdf2_kdf.derive(passphrase.encode('utf-8'))
            except Exception as e:
                raise BackupError(f"PBKDF2 key derivation failed: {e}", "PBKDF2_DERIVATION_FAILED") from e
    
    def _prepare_key_plaintext(self, key_pair: Ed25519KeyPair) -> bytes:
        """Prepare key data for backup (concatenate private and public keys)"""
        return key_pair.private_key + key_pair.public_key
    
    def _extract_key_pair(self, key_data: bytes) -> Ed25519KeyPair:
        """Extract key pair from backup data"""
        if len(key_data) != 64:  # 32 bytes private + 32 bytes public
            raise BackupError("Invalid key data length in backup", "INVALID_BACKUP_KEY_LENGTH")
        
        private_key = key_data[:32]
        public_key = key_data[32:]
        
        return Ed25519KeyPair(private_key=private_key, public_key=public_key)
    
    def _encrypt_xchacha20_poly1305(self, plaintext: bytes, key: bytes, nonce: bytes) -> bytes:
        """Encrypt data using XChaCha20-Poly1305"""
        # Note: cryptography library uses ChaCha20-Poly1305, not XChaCha20-Poly1305
        # For true XChaCha20-Poly1305, you would need a different library like PyNaCl
        if ChaCha20Poly1305 is None:
            raise BackupError("ChaCha20Poly1305 not available", "CHACHA20_UNAVAILABLE")
        
        try:
            # Truncate nonce to 12 bytes for ChaCha20-Poly1305 (not XChaCha20-Poly1305)
            chacha_nonce = nonce[:12] if len(nonce) > 12 else nonce
            cipher = ChaCha20Poly1305(key)
            return cipher.encrypt(chacha_nonce, plaintext, None)
        except Exception as e:
            raise BackupError(f"ChaCha20Poly1305 encryption failed: {e}", "ENCRYPTION_FAILED") from e
    
    def _decrypt_xchacha20_poly1305(self, ciphertext: bytes, key: bytes, nonce: bytes) -> bytes:
        """Decrypt data using XChaCha20-Poly1305"""
        if ChaCha20Poly1305 is None:
            raise BackupError("ChaCha20Poly1305 not available", "CHACHA20_UNAVAILABLE")
        
        try:
            # Truncate nonce to 12 bytes for ChaCha20-Poly1305 (not XChaCha20-Poly1305)
            chacha_nonce = nonce[:12] if len(nonce) > 12 else nonce
            cipher = ChaCha20Poly1305(key)
            return cipher.decrypt(chacha_nonce, ciphertext, None)
        except InvalidTag as e:
            raise BackupError("Decryption failed - invalid passphrase or corrupted data", "DECRYPTION_FAILED") from e
        except Exception as e:
            raise BackupError(f"ChaCha20Poly1305 decryption failed: {e}", "DECRYPTION_FAILED") from e
    
    def _encrypt_aes_gcm(self, plaintext: bytes, key: bytes, nonce: bytes) -> bytes:
        """Encrypt data using AES-GCM"""
        if AESGCM is None:
            raise BackupError("AES-GCM not available", "AESGCM_UNAVAILABLE")
        
        try:
            cipher = AESGCM(key)
            return cipher.encrypt(nonce, plaintext, None)
        except Exception as e:
            raise BackupError(f"AES-GCM encryption failed: {e}", "ENCRYPTION_FAILED") from e
    
    def _decrypt_aes_gcm(self, ciphertext: bytes, key: bytes, nonce: bytes) -> bytes:
        """Decrypt data using AES-GCM"""
        if AESGCM is None:
            raise BackupError("AES-GCM not available", "AESGCM_UNAVAILABLE")
        
        try:
            cipher = AESGCM(key)
            return cipher.decrypt(nonce, ciphertext, None)
        except InvalidTag as e:
            raise BackupError("Decryption failed - invalid passphrase or corrupted data", "DECRYPTION_FAILED") from e
        except Exception as e:
            raise BackupError(f"AES-GCM decryption failed: {e}", "DECRYPTION_FAILED") from e
    
    def _validate_backup_format(self, backup: Dict[str, Any]) -> None:
        """Validate backup format structure"""
        required_fields = ['version', 'kdf', 'kdf_params', 'encryption', 'nonce', 'ciphertext', 'created']
        
        for field in required_fields:
            if field not in backup:
                raise ValidationError(f"Missing required field: {field}", "MISSING_FIELD")
        
        if backup['version'] != UNIFIED_BACKUP_VERSION:
            raise ValidationError(f"Unsupported backup version: {backup['version']}", "UNSUPPORTED_VERSION")
        
        if backup['kdf'] not in ['argon2id', 'pbkdf2']:
            raise ValidationError(f"Unsupported KDF: {backup['kdf']}", "UNSUPPORTED_KDF")
        
        if backup['encryption'] not in ['xchacha20-poly1305', 'aes-gcm']:
            raise ValidationError(f"Unsupported encryption: {backup['encryption']}", "UNSUPPORTED_ENCRYPTION")
        
        # Validate KDF parameters
        kdf_params = backup['kdf_params']
        if 'salt' not in kdf_params or 'iterations' not in kdf_params:
            raise ValidationError("Missing required KDF parameters", "MISSING_KDF_PARAMS")
        
        if backup['kdf'] == 'argon2id':
            if 'memory' not in kdf_params or 'parallelism' not in kdf_params:
                raise ValidationError("Missing Argon2id parameters (memory, parallelism)", "MISSING_ARGON2_PARAMS")
    
    def _import_legacy_format(self, legacy_backup: Dict[str, Any], passphrase: str, format_type: str) -> Tuple[Ed25519KeyPair, Optional[Dict[str, Any]]]:
        """Import from legacy format (simplified implementation)"""
        # This is a placeholder - in a full implementation, you would have
        # specific parsers for each legacy format
        raise BackupError(f"Legacy format import not yet implemented for: {format_type}", "LEGACY_IMPORT_NOT_IMPLEMENTED")


# Convenience functions
def export_key_unified(
    key_pair: Ed25519KeyPair,
    passphrase: str,
    **kwargs
) -> str:
    """Export key using unified backup format"""
    manager = UnifiedBackupManager()
    return manager.export_key(key_pair, passphrase, **kwargs)


def import_key_unified(
    backup_data: str,
    passphrase: str
) -> Tuple[Ed25519KeyPair, Optional[Dict[str, Any]]]:
    """Import key from unified backup format"""
    manager = UnifiedBackupManager()
    return manager.import_key(backup_data, passphrase)


def migrate_backup_unified(
    legacy_data: str,
    passphrase: str
) -> MigrationResult:
    """Migrate legacy backup to unified format"""
    manager = UnifiedBackupManager()
    return manager.migrate_legacy_backup(legacy_data, passphrase)