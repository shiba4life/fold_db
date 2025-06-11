"""
Encrypted key export/import functionality for DataFold Python SDK

This module provides secure backup and restore functionality for Ed25519 keys,
following the backup format guidelines from task 10-1-3 research.
"""

import os
import json
import secrets
import platform
from typing import Dict, List, Optional, Tuple, Union, Any
from dataclasses import dataclass
from datetime import datetime
import base64

# Import cryptography components
try:
    from cryptography.hazmat.primitives.ciphers.aead import ChaCha20Poly1305, AESGCM
    from cryptography.hazmat.primitives.kdf.scrypt import Scrypt
    from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
    from cryptography.hazmat.primitives import hashes, serialization
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
    Scrypt = None  # type: ignore
    PBKDF2HMAC = None  # type: ignore
    hashes = None  # type: ignore
    serialization = None  # type: ignore
    InvalidTag = Exception
    ARGON2_AVAILABLE = False

from ..exceptions import BackupError, ValidationError, UnsupportedPlatformError
from .ed25519 import Ed25519KeyPair, ED25519_PRIVATE_KEY_LENGTH, ED25519_PUBLIC_KEY_LENGTH

# Constants for backup operations
BACKUP_VERSION = 1
BACKUP_SALT_LENGTH = 32
BACKUP_NONCE_LENGTH = 12  # For ChaCha20Poly1305 (cryptography library uses ChaCha20, not XChaCha20)
BACKUP_IV_LENGTH = 12     # For AES-GCM

# KDF parameters
ARGON2_MEMORY = 65536     # 64MB
ARGON2_ITERATIONS = 3
ARGON2_PARALLELISM = 1
SCRYPT_N = 32768
SCRYPT_R = 8
SCRYPT_P = 1
PBKDF2_ITERATIONS = 100000

# Supported formats
SUPPORTED_EXPORT_FORMATS = ['json', 'binary']
SUPPORTED_KDF_ALGORITHMS = ['argon2id', 'scrypt', 'pbkdf2']
SUPPORTED_ENCRYPTION_ALGORITHMS = ['chacha20-poly1305', 'aes-gcm']


@dataclass
class BackupMetadata:
    """
    Metadata for key backup operations
    
    Attributes:
        version: Backup format version
        key_id: Identifier for the backed up key
        algorithm: Key algorithm (e.g., 'Ed25519')
        kdf: Key derivation function used
        encryption: Encryption algorithm used
        created: Creation timestamp
        format: Backup format ('json' or 'binary')
    """
    version: int
    key_id: str
    algorithm: str
    kdf: str
    encryption: str
    created: str
    format: str


class KeyBackupManager:
    """
    Manager for encrypted key backup and restore operations
    
    This class provides secure backup and restore functionality following
    the format guidelines from task 10-1-3 research.
    """
    
    def __init__(self,
                 preferred_kdf: str = 'argon2id',
                 preferred_encryption: str = 'chacha20-poly1305'):
        """
        Initialize key backup manager
        
        Args:
            preferred_kdf: Preferred KDF algorithm
            preferred_encryption: Preferred encryption algorithm
        """
        if not CRYPTOGRAPHY_AVAILABLE:
            raise UnsupportedPlatformError(
                "Cryptography package required for key backup operations",
                "CRYPTOGRAPHY_UNAVAILABLE"
            )
        
        self.preferred_kdf = preferred_kdf
        self.preferred_encryption = preferred_encryption
        
        # Validate preferences and adjust if needed
        self._validate_and_adjust_preferences()
    
    def _validate_and_adjust_preferences(self) -> None:
        """Validate and adjust preferences based on availability"""
        # Check KDF availability
        if self.preferred_kdf == 'argon2id' and not ARGON2_AVAILABLE:
            self.preferred_kdf = 'scrypt'
        
        if self.preferred_kdf not in SUPPORTED_KDF_ALGORITHMS:
            self.preferred_kdf = 'scrypt'  # Fallback
        
        # Check encryption availability
        # Map xchacha20-poly1305 to chacha20-poly1305 for compatibility
        if self.preferred_encryption == 'xchacha20-poly1305':
            self.preferred_encryption = 'chacha20-poly1305'
        
        if self.preferred_encryption not in SUPPORTED_ENCRYPTION_ALGORITHMS:
            self.preferred_encryption = 'aes-gcm'  # Fallback
    
    def _generate_salt(self) -> bytes:
        """Generate cryptographically secure salt"""
        return secrets.token_bytes(BACKUP_SALT_LENGTH)
    
    def _generate_nonce(self, algorithm: str) -> bytes:
        """Generate nonce/IV for encryption algorithm"""
        if algorithm == 'chacha20-poly1305':
            return secrets.token_bytes(BACKUP_NONCE_LENGTH)
        elif algorithm == 'aes-gcm':
            return secrets.token_bytes(BACKUP_IV_LENGTH)
        else:
            raise BackupError(f"Unsupported encryption algorithm: {algorithm}", "UNSUPPORTED_ENCRYPTION")
    
    def _derive_key_argon2id(self, passphrase: str, salt: bytes) -> bytes:
        """Derive encryption key using Argon2id"""
        if not ARGON2_AVAILABLE or Argon2 is None or Argon2Type is None:
            raise BackupError("Argon2 not available", "ARGON2_UNAVAILABLE")
        
        try:
            kdf = Argon2(
                time_cost=ARGON2_ITERATIONS,
                memory_cost=ARGON2_MEMORY,
                parallelism=ARGON2_PARALLELISM,
                hash_len=32,  # 256-bit key
                salt=salt,
                type=Argon2Type.ID
            )
            return kdf.derive(passphrase.encode('utf-8'))
        except Exception as e:
            raise BackupError(f"Argon2 key derivation failed: {e}", "ARGON2_DERIVATION_FAILED") from e
    
    def _derive_key_scrypt(self, passphrase: str, salt: bytes) -> bytes:
        """Derive encryption key using Scrypt"""
        if Scrypt is None:
            raise BackupError("Scrypt not available", "SCRYPT_UNAVAILABLE")
        
        try:
            kdf = Scrypt(
                length=32,  # 256-bit key
                salt=salt,
                n=SCRYPT_N,
                r=SCRYPT_R,
                p=SCRYPT_P,
            )
            return kdf.derive(passphrase.encode('utf-8'))
        except Exception as e:
            raise BackupError(f"Scrypt key derivation failed: {e}", "SCRYPT_DERIVATION_FAILED") from e
    
    def _derive_key_pbkdf2(self, passphrase: str, salt: bytes) -> bytes:
        """Derive encryption key using PBKDF2"""
        if PBKDF2HMAC is None or hashes is None:
            raise BackupError("PBKDF2 not available", "PBKDF2_UNAVAILABLE")
        
        try:
            kdf = PBKDF2HMAC(
                algorithm=hashes.SHA256(),
                length=32,  # 256-bit key
                salt=salt,
                iterations=PBKDF2_ITERATIONS,
            )
            return kdf.derive(passphrase.encode('utf-8'))
        except Exception as e:
            raise BackupError(f"PBKDF2 key derivation failed: {e}", "PBKDF2_DERIVATION_FAILED") from e
    
    def _derive_key(self, passphrase: str, salt: bytes, kdf_algorithm: str) -> bytes:
        """Derive encryption key using specified KDF"""
        if kdf_algorithm == 'argon2id':
            return self._derive_key_argon2id(passphrase, salt)
        elif kdf_algorithm == 'scrypt':
            return self._derive_key_scrypt(passphrase, salt)
        elif kdf_algorithm == 'pbkdf2':
            return self._derive_key_pbkdf2(passphrase, salt)
        else:
            raise BackupError(f"Unsupported KDF algorithm: {kdf_algorithm}", "UNSUPPORTED_KDF")
    
    def _encrypt_data(self, data: bytes, key: bytes, algorithm: str) -> Tuple[bytes, bytes]:
        """Encrypt data using specified algorithm"""
        if algorithm == 'chacha20-poly1305':
            if ChaCha20Poly1305 is None:
                raise BackupError("ChaCha20Poly1305 not available", "CHACHA20_UNAVAILABLE")
            
            try:
                cipher = ChaCha20Poly1305(key)
                nonce = self._generate_nonce(algorithm)
                ciphertext = cipher.encrypt(nonce, data, None)
                return ciphertext, nonce
            except Exception as e:
                raise BackupError(f"ChaCha20Poly1305 encryption failed: {e}", "ENCRYPTION_FAILED") from e
        
        elif algorithm == 'aes-gcm':
            if AESGCM is None:
                raise BackupError("AES-GCM not available", "AESGCM_UNAVAILABLE")
            
            try:
                cipher = AESGCM(key)
                iv = self._generate_nonce(algorithm)
                ciphertext = cipher.encrypt(iv, data, None)
                return ciphertext, iv
            except Exception as e:
                raise BackupError(f"AES-GCM encryption failed: {e}", "ENCRYPTION_FAILED") from e
        
        else:
            raise BackupError(f"Unsupported encryption algorithm: {algorithm}", "UNSUPPORTED_ENCRYPTION")
    
    def _decrypt_data(self, ciphertext: bytes, key: bytes, nonce: bytes, algorithm: str) -> bytes:
        """Decrypt data using specified algorithm"""
        if algorithm == 'chacha20-poly1305':
            if ChaCha20Poly1305 is None:
                raise BackupError("ChaCha20Poly1305 not available", "CHACHA20_UNAVAILABLE")
            
            try:
                cipher = ChaCha20Poly1305(key)
                return cipher.decrypt(nonce, ciphertext, None)
            except InvalidTag as e:
                raise BackupError("Decryption failed - invalid passphrase or corrupted data", "DECRYPTION_FAILED") from e
            except Exception as e:
                raise BackupError(f"ChaCha20Poly1305 decryption failed: {e}", "DECRYPTION_FAILED") from e
        
        elif algorithm == 'aes-gcm':
            if AESGCM is None:
                raise BackupError("AES-GCM not available", "AESGCM_UNAVAILABLE")
            
            try:
                cipher = AESGCM(key)
                return cipher.decrypt(nonce, ciphertext, None)
            except InvalidTag as e:
                raise BackupError("Decryption failed - invalid passphrase or corrupted data", "DECRYPTION_FAILED") from e
            except Exception as e:
                raise BackupError(f"AES-GCM decryption failed: {e}", "DECRYPTION_FAILED") from e
        
        else:
            raise BackupError(f"Unsupported encryption algorithm: {algorithm}", "UNSUPPORTED_ENCRYPTION")
    
    def _validate_passphrase(self, passphrase: str) -> None:
        """Validate passphrase strength"""
        if not isinstance(passphrase, str):
            raise BackupError("Passphrase must be a string", "INVALID_PASSPHRASE_TYPE")
        
        if len(passphrase) < 8:
            raise BackupError("Passphrase must be at least 8 characters", "WEAK_PASSPHRASE")
        
        # Check for basic complexity
        has_upper = any(c.isupper() for c in passphrase)
        has_lower = any(c.islower() for c in passphrase)
        has_digit = any(c.isdigit() for c in passphrase)
        
        strength_score = sum([has_upper, has_lower, has_digit])
        
        if len(passphrase) < 12 and strength_score < 2:
            raise BackupError(
                "Weak passphrase - use at least 12 characters or include uppercase, lowercase, and numbers",
                "WEAK_PASSPHRASE"
            )
    
    def _get_timestamp(self) -> str:
        """Get current timestamp as ISO string"""
        from datetime import timezone
        return datetime.now(timezone.utc).isoformat() + 'Z'
    
    def _prepare_key_data(self, key_pair: Ed25519KeyPair) -> bytes:
        """Prepare key data for backup (combine private and public keys)"""
        return key_pair.private_key + key_pair.public_key
    
    def _extract_key_pair(self, key_data: bytes) -> Ed25519KeyPair:
        """Extract key pair from backup data"""
        if len(key_data) != 64:  # 32 bytes private + 32 bytes public
            raise BackupError("Invalid key data length in backup", "INVALID_BACKUP_KEY_LENGTH")
        
        private_key = key_data[:32]
        public_key = key_data[32:]
        
        return Ed25519KeyPair(private_key=private_key, public_key=public_key)
    
    def export_key(self, 
                   key_pair: Ed25519KeyPair,
                   passphrase: str,
                   key_id: str,
                   export_format: str = 'json',
                   kdf_algorithm: Optional[str] = None,
                   encryption_algorithm: Optional[str] = None) -> Union[str, bytes]:
        """
        Export Ed25519 key pair as encrypted backup
        
        Args:
            key_pair: Key pair to export
            passphrase: Passphrase for encryption
            key_id: Identifier for the key
            export_format: Export format ('json' or 'binary')
            kdf_algorithm: KDF algorithm to use (optional, uses preference if None)
            encryption_algorithm: Encryption algorithm to use (optional, uses preference if None)
            
        Returns:
            Union[str, bytes]: Encrypted backup data
            
        Raises:
            BackupError: If export fails
            ValidationError: If inputs are invalid
        """
        # Validate inputs
        if not isinstance(key_pair, Ed25519KeyPair):
            raise ValidationError("key_pair must be Ed25519KeyPair instance", "INVALID_KEY_PAIR_TYPE")
        
        if not key_id or not isinstance(key_id, str):
            raise ValidationError("key_id must be non-empty string", "INVALID_KEY_ID")
        
        if export_format not in SUPPORTED_EXPORT_FORMATS:
            raise ValidationError(f"Unsupported export format: {export_format}", "UNSUPPORTED_EXPORT_FORMAT")
        
        self._validate_passphrase(passphrase)
        
        # Use preferences if not specified
        kdf_algo = kdf_algorithm or self.preferred_kdf
        enc_algo = encryption_algorithm or self.preferred_encryption
        
        try:
            # Prepare key data
            key_data = self._prepare_key_data(key_pair)
            
            # Generate salt and derive key
            salt = self._generate_salt()
            derived_key = self._derive_key(passphrase, salt, kdf_algo)
            
            # Encrypt key data
            ciphertext, nonce = self._encrypt_data(key_data, derived_key, enc_algo)
            
            # Prepare KDF parameters
            kdf_params: Dict[str, Any] = {}
            if kdf_algo == 'argon2id':
                kdf_params = {
                    'memory': ARGON2_MEMORY,
                    'iterations': ARGON2_ITERATIONS,
                    'parallelism': ARGON2_PARALLELISM
                }
            elif kdf_algo == 'scrypt':
                kdf_params = {
                    'n': SCRYPT_N,
                    'r': SCRYPT_R,
                    'p': SCRYPT_P
                }
            elif kdf_algo == 'pbkdf2':
                kdf_params = {
                    'iterations': PBKDF2_ITERATIONS
                }
            
            # Create backup structure
            backup_data = {
                'version': BACKUP_VERSION,
                'key_id': key_id,
                'algorithm': 'Ed25519',
                'kdf': kdf_algo,
                'kdf_params': kdf_params,
                'encryption': enc_algo,
                'salt': base64.b64encode(salt).decode('ascii'),
                'nonce': base64.b64encode(nonce).decode('ascii'),
                'ciphertext': base64.b64encode(ciphertext).decode('ascii'),
                'created': self._get_timestamp()
            }
            
            if export_format == 'json':
                return json.dumps(backup_data, indent=2)
            elif export_format == 'binary':
                # Convert to JSON first, then encode as bytes
                json_str = json.dumps(backup_data)
                return json_str.encode('utf-8')
            else:
                raise ValidationError(f"Unsupported export format: {export_format}", "UNSUPPORTED_EXPORT_FORMAT")
            
        except Exception as e:
            if isinstance(e, (BackupError, ValidationError)):
                raise
            
            raise BackupError(f"Key export failed: {str(e)}", "EXPORT_FAILED") from e
    
    def import_key(self, 
                   backup_data: Union[str, bytes],
                   passphrase: str,
                   verify_integrity: bool = True) -> Tuple[Ed25519KeyPair, BackupMetadata]:
        """
        Import Ed25519 key pair from encrypted backup
        
        Args:
            backup_data: Encrypted backup data (JSON string or bytes)
            passphrase: Passphrase for decryption
            verify_integrity: Whether to verify key integrity after import
            
        Returns:
            Tuple of (Ed25519KeyPair, BackupMetadata)
            
        Raises:
            BackupError: If import fails
            ValidationError: If backup data is invalid
        """
        self._validate_passphrase(passphrase)
        
        try:
            # Parse backup data
            if isinstance(backup_data, bytes):
                backup_dict = json.loads(backup_data.decode('utf-8'))
                format_type = 'binary'
            elif isinstance(backup_data, str):
                backup_dict = json.loads(backup_data)
                format_type = 'json'
            else:
                raise ValidationError("backup_data must be string or bytes", "INVALID_BACKUP_DATA_TYPE")
            
            # Validate backup structure
            required_fields = ['version', 'key_id', 'algorithm', 'kdf', 'encryption', 
                             'salt', 'nonce', 'ciphertext', 'created']
            
            for field in required_fields:
                if field not in backup_dict:
                    raise ValidationError(f"Missing required field: {field}", "INVALID_BACKUP_STRUCTURE")
            
            # Check version compatibility
            if backup_dict['version'] != BACKUP_VERSION:
                raise ValidationError(f"Unsupported backup version: {backup_dict['version']}", "UNSUPPORTED_BACKUP_VERSION")
            
            # Check algorithm support
            if backup_dict['algorithm'] != 'Ed25519':
                raise ValidationError(f"Unsupported key algorithm: {backup_dict['algorithm']}", "UNSUPPORTED_KEY_ALGORITHM")
            
            # Extract parameters
            kdf_algo = backup_dict['kdf']
            enc_algo = backup_dict['encryption']
            salt = base64.b64decode(backup_dict['salt'])
            nonce = base64.b64decode(backup_dict['nonce'])
            ciphertext = base64.b64decode(backup_dict['ciphertext'])
            
            # Derive decryption key
            derived_key = self._derive_key(passphrase, salt, kdf_algo)
            
            # Decrypt key data
            key_data = self._decrypt_data(ciphertext, derived_key, nonce, enc_algo)
            
            # Extract key pair
            key_pair = self._extract_key_pair(key_data)
            
            # Verify integrity if requested
            if verify_integrity:
                self._verify_key_integrity(key_pair)
            
            # Create metadata
            metadata = BackupMetadata(
                version=backup_dict['version'],
                key_id=backup_dict['key_id'],
                algorithm=backup_dict['algorithm'],
                kdf=kdf_algo,
                encryption=enc_algo,
                created=backup_dict['created'],
                format=format_type
            )
            
            return key_pair, metadata
            
        except json.JSONDecodeError as e:
            raise ValidationError(f"Invalid JSON in backup data: {e}", "INVALID_BACKUP_JSON") from e
        except Exception as e:
            if isinstance(e, (BackupError, ValidationError)):
                raise
            
            raise BackupError(f"Key import failed: {str(e)}", "IMPORT_FAILED") from e
    
    def _verify_key_integrity(self, key_pair: Ed25519KeyPair) -> None:
        """Verify key pair integrity by performing a signature test"""
        try:
            from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
            from cryptography.exceptions import InvalidSignature
            
            # Reconstruct key objects
            private_key_obj = Ed25519PrivateKey.from_private_bytes(key_pair.private_key)
            public_key_obj = private_key_obj.public_key()
            
            # Verify public key matches
            if serialization is None:
                raise BackupError("Serialization not available", "SERIALIZATION_UNAVAILABLE")
            
            expected_public = public_key_obj.public_bytes(
                encoding=serialization.Encoding.Raw,
                format=serialization.PublicFormat.Raw
            )
            
            if expected_public != key_pair.public_key:
                raise BackupError("Key pair integrity check failed - public key mismatch", "INTEGRITY_CHECK_FAILED")
            
            # Test signature
            test_message = b"integrity_check_message"
            signature = private_key_obj.sign(test_message)
            
            try:
                public_key_obj.verify(signature, test_message)
            except InvalidSignature:
                raise BackupError("Key pair integrity check failed - signature verification failed", "INTEGRITY_CHECK_FAILED")
            
        except Exception as e:
            if isinstance(e, BackupError):
                raise
            
            raise BackupError(f"Key integrity verification failed: {str(e)}", "INTEGRITY_CHECK_FAILED") from e
    
    def check_backup_support(self) -> Dict[str, Any]:
        """
        Check platform support for backup operations
        
        Returns:
            dict: Support information for backup operations
        """
        support: Dict[str, Any] = {
            'cryptography_available': CRYPTOGRAPHY_AVAILABLE,
            'argon2_available': ARGON2_AVAILABLE,
            'supported_kdf_algorithms': [],
            'supported_encryption_algorithms': [],
            'supported_export_formats': SUPPORTED_EXPORT_FORMATS.copy(),
            'current_preferences': {
                'kdf': self.preferred_kdf,
                'encryption': self.preferred_encryption
            }
        }
        
        if CRYPTOGRAPHY_AVAILABLE:
            # Test KDF support
            for kdf in SUPPORTED_KDF_ALGORITHMS:
                try:
                    if kdf == 'argon2id' and ARGON2_AVAILABLE:
                        support['supported_kdf_algorithms'].append(kdf)
                    elif kdf in ['scrypt', 'pbkdf2']:
                        support['supported_kdf_algorithms'].append(kdf)
                except Exception:
                    pass
            
            # Test encryption support
            for enc in SUPPORTED_ENCRYPTION_ALGORITHMS:
                try:
                    if enc == 'chacha20-poly1305' and ChaCha20Poly1305:
                        support['supported_encryption_algorithms'].append(enc)
                    elif enc == 'aes-gcm' and AESGCM:
                        support['supported_encryption_algorithms'].append(enc)
                except Exception:
                    pass
        
        return support


# Convenience functions

def get_default_backup_manager() -> KeyBackupManager:
    """Get default key backup manager instance"""
    return KeyBackupManager()


def export_key_to_file(key_pair: Ed25519KeyPair,
                      passphrase: str,
                      key_id: str,
                      file_path: str,
                      export_format: str = 'json') -> BackupMetadata:
    """
    Export key pair to encrypted backup file
    
    Args:
        key_pair: Key pair to export
        passphrase: Passphrase for encryption
        key_id: Identifier for the key
        file_path: Path to save backup file
        export_format: Export format ('json' or 'binary')
        
    Returns:
        BackupMetadata: Metadata about the backup
        
    Raises:
        BackupError: If export fails
    """
    manager = get_default_backup_manager()
    
    try:
        # Export key
        backup_data = manager.export_key(key_pair, passphrase, key_id, export_format)
        
        # Write to file
        if export_format == 'json':
            if isinstance(backup_data, str):
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(backup_data)
            else:
                raise BackupError("Expected string for JSON format", "INVALID_BACKUP_DATA_TYPE")
        else:  # binary
            if isinstance(backup_data, bytes):
                with open(file_path, 'wb') as f:
                    f.write(backup_data)
            else:
                raise BackupError("Expected bytes for binary format", "INVALID_BACKUP_DATA_TYPE")
        
        # Set file permissions (owner read/write only)
        if platform.system() != "Windows":
            os.chmod(file_path, 0o600)
        
        # Create metadata
        metadata = BackupMetadata(
            version=BACKUP_VERSION,
            key_id=key_id,
            algorithm='Ed25519',
            kdf=manager.preferred_kdf,
            encryption=manager.preferred_encryption,
            created=manager._get_timestamp(),
            format=export_format
        )
        
        return metadata
        
    except Exception as e:
        # Clean up partial file
        try:
            if os.path.exists(file_path):
                os.unlink(file_path)
        except:
            pass
        
        if isinstance(e, BackupError):
            raise
        
        raise BackupError(f"Failed to export key to file: {str(e)}", "FILE_EXPORT_FAILED") from e


def import_key_from_file(file_path: str,
                        passphrase: str,
                        verify_integrity: bool = True) -> Tuple[Ed25519KeyPair, BackupMetadata]:
    """
    Import key pair from encrypted backup file
    
    Args:
        file_path: Path to backup file
        passphrase: Passphrase for decryption
        verify_integrity: Whether to verify key integrity after import
        
    Returns:
        Tuple of (Ed25519KeyPair, BackupMetadata)
        
    Raises:
        BackupError: If import fails
    """
    manager = get_default_backup_manager()
    
    try:
        # Determine format from file extension
        backup_data: Union[str, bytes]
        if file_path.lower().endswith('.json'):
            with open(file_path, 'r', encoding='utf-8') as f:
                backup_data = f.read()
        else:
            with open(file_path, 'rb') as f:
                backup_data = f.read()
        
        return manager.import_key(backup_data, passphrase, verify_integrity)
        
    except FileNotFoundError:
        raise BackupError(f"Backup file not found: {file_path}", "BACKUP_FILE_NOT_FOUND")
    except PermissionError:
        raise BackupError(f"Permission denied accessing backup file: {file_path}", "BACKUP_FILE_PERMISSION_DENIED")
    except Exception as e:
        if isinstance(e, BackupError):
            raise
        
        raise BackupError(f"Failed to import key from file: {str(e)}", "FILE_IMPORT_FAILED") from e