"""
Secure key storage for DataFold Python SDK

This module provides secure storage for Ed25519 private keys using OS keychain
services with encrypted file fallback, following security best practices.
"""

import os
import stat
import platform
import json
import secrets
from typing import Dict, List, Optional, Tuple, Union, Any
from dataclasses import dataclass
from pathlib import Path
import base64

# Cryptography imports
try:
    from cryptography.fernet import Fernet
    from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
    from cryptography.hazmat.primitives import hashes
    from cryptography.hazmat.primitives.kdf.scrypt import Scrypt
    CRYPTOGRAPHY_AVAILABLE = True
except ImportError:
    CRYPTOGRAPHY_AVAILABLE = False
    Fernet = None
    PBKDF2HMAC = None
    hashes = None
    Scrypt = None

# Keyring import for OS keychain
try:
    import keyring
    from keyring.errors import KeyringError, KeyringLocked
    KEYRING_AVAILABLE = True
except ImportError:
    KEYRING_AVAILABLE = False
    keyring = None
    KeyringError = Exception
    KeyringLocked = Exception

from ..exceptions import StorageError, UnsupportedPlatformError
from .ed25519 import Ed25519KeyPair

# Constants
STORAGE_SERVICE_NAME = "DataFold SDK"
DEFAULT_STORAGE_DIR = ".datafold"
ENCRYPTED_FILE_EXTENSION = ".key"
KEY_FILE_PERMISSIONS = 0o600  # Owner read/write only
SALT_LENGTH = 32
IV_LENGTH = 16
SCRYPT_N = 32768  # CPU cost factor
SCRYPT_R = 8      # Memory cost factor
SCRYPT_P = 1      # Parallelization factor


@dataclass
class StorageMetadata:
    """
    Metadata for stored keys
    
    Attributes:
        key_id: Unique identifier for the key
        storage_type: Type of storage used ('keyring' or 'file')
        created_at: Timestamp when key was stored
        last_accessed: Timestamp when key was last accessed
        algorithm: Key algorithm (e.g., 'Ed25519')
    """
    key_id: str
    storage_type: str
    created_at: str
    last_accessed: str
    algorithm: str


class SecureKeyStorage:
    """
    Secure key storage with OS keychain and encrypted file fallback
    
    This class provides cross-platform secure storage for Ed25519 private keys,
    using OS keychain services when available and falling back to encrypted
    file storage with strong KDF.
    """
    
    def __init__(self, storage_dir: Optional[str] = None, use_keyring: bool = True):
        """
        Initialize secure key storage
        
        Args:
            storage_dir: Directory for encrypted file storage (optional)
            use_keyring: Whether to use OS keyring when available
        """
        self.use_keyring = use_keyring and KEYRING_AVAILABLE
        self.storage_dir = Path(storage_dir) if storage_dir else self._get_default_storage_dir()
        self._ensure_storage_dir()
        
    def _get_default_storage_dir(self) -> Path:
        """Get default storage directory based on platform"""
        home = Path.home()
        
        if platform.system() == "Windows":
            # Use APPDATA on Windows
            appdata = os.getenv("APPDATA", str(home))
            return Path(appdata) / DEFAULT_STORAGE_DIR
        elif platform.system() == "Darwin":
            # Use Application Support on macOS
            return home / "Library" / "Application Support" / DEFAULT_STORAGE_DIR
        else:
            # Use hidden directory in home on Linux/Unix
            return home / f".{DEFAULT_STORAGE_DIR}"
    
    def _ensure_storage_dir(self) -> None:
        """Ensure storage directory exists with proper permissions"""
        try:
            self.storage_dir.mkdir(parents=True, exist_ok=True)
            # Set directory permissions to owner only
            if platform.system() != "Windows":
                os.chmod(self.storage_dir, 0o700)
        except Exception as e:
            raise StorageError(
                f"Failed to create storage directory: {e}",
                "STORAGE_DIR_CREATION_FAILED"
            )
    
    def _get_key_identifier(self, key_id: str) -> str:
        """Get platform-specific key identifier for keyring"""
        return f"{STORAGE_SERVICE_NAME}:{key_id}"
    
    def _get_file_path(self, key_id: str) -> Path:
        """Get file path for encrypted key storage"""
        # Sanitize key_id for filesystem
        safe_key_id = "".join(c for c in key_id if c.isalnum() or c in "-_.")
        return self.storage_dir / f"{safe_key_id}{ENCRYPTED_FILE_EXTENSION}"
    
    def _derive_key_from_passphrase(self, passphrase: str, salt: bytes) -> bytes:
        """Derive encryption key from passphrase using Scrypt KDF"""
        if not CRYPTOGRAPHY_AVAILABLE:
            raise UnsupportedPlatformError(
                "Cryptography package required for encrypted file storage",
                "CRYPTOGRAPHY_UNAVAILABLE"
            )
        
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
            raise StorageError(
                f"Key derivation failed: {e}",
                "KEY_DERIVATION_FAILED"
            )
    
    def _encrypt_key_data(self, key_data: bytes, passphrase: str) -> Dict[str, str]:
        """Encrypt key data with passphrase"""
        # Generate random salt
        salt = secrets.token_bytes(SALT_LENGTH)
        
        # Derive encryption key
        encryption_key = self._derive_key_from_passphrase(passphrase, salt)
        
        # Encrypt the key data
        fernet = Fernet(base64.urlsafe_b64encode(encryption_key))
        encrypted_data = fernet.encrypt(key_data)
        
        return {
            'encrypted_data': base64.b64encode(encrypted_data).decode('ascii'),
            'salt': base64.b64encode(salt).decode('ascii'),
            'algorithm': 'Scrypt-Fernet',
            'version': '1.0'
        }
    
    def _decrypt_key_data(self, encrypted_payload: Dict[str, str], passphrase: str) -> bytes:
        """Decrypt key data with passphrase"""
        try:
            # Extract components
            encrypted_data = base64.b64decode(encrypted_payload['encrypted_data'])
            salt = base64.b64decode(encrypted_payload['salt'])
            
            # Derive decryption key
            decryption_key = self._derive_key_from_passphrase(passphrase, salt)
            
            # Decrypt the key data
            fernet = Fernet(base64.urlsafe_b64encode(decryption_key))
            key_data = fernet.decrypt(encrypted_data)
            
            return key_data
        except Exception as e:
            raise StorageError(
                f"Key decryption failed: {e}",
                "KEY_DECRYPTION_FAILED"
            )
    
    def _store_to_keyring(self, key_id: str, key_data: bytes) -> None:
        """Store key data to OS keyring"""
        try:
            identifier = self._get_key_identifier(key_id)
            # Encode key data as base64 for storage
            encoded_data = base64.b64encode(key_data).decode('ascii')
            keyring.set_password(STORAGE_SERVICE_NAME, identifier, encoded_data)
        except (KeyringError, KeyringLocked) as e:
            raise StorageError(
                f"Keyring storage failed: {e}",
                "KEYRING_STORAGE_FAILED"
            )
        except Exception as e:
            raise StorageError(
                f"Unexpected keyring error: {e}",
                "KEYRING_UNEXPECTED_ERROR"
            )
    
    def _retrieve_from_keyring(self, key_id: str) -> Optional[bytes]:
        """Retrieve key data from OS keyring"""
        try:
            identifier = self._get_key_identifier(key_id)
            encoded_data = keyring.get_password(STORAGE_SERVICE_NAME, identifier)
            
            if encoded_data is None:
                return None
                
            # Decode from base64
            return base64.b64decode(encoded_data)
        except (KeyringError, KeyringLocked) as e:
            raise StorageError(
                f"Keyring retrieval failed: {e}",
                "KEYRING_RETRIEVAL_FAILED"
            )
        except Exception as e:
            raise StorageError(
                f"Unexpected keyring error: {e}",
                "KEYRING_UNEXPECTED_ERROR"
            )
    
    def _delete_from_keyring(self, key_id: str) -> bool:
        """Delete key data from OS keyring"""
        try:
            identifier = self._get_key_identifier(key_id)
            keyring.delete_password(STORAGE_SERVICE_NAME, identifier)
            return True
        except KeyringError:
            # Key not found or other keyring error
            return False
        except Exception as e:
            raise StorageError(
                f"Unexpected keyring error: {e}",
                "KEYRING_UNEXPECTED_ERROR"
            )
    
    def _store_to_file(self, key_id: str, key_data: bytes, passphrase: str) -> None:
        """Store key data to encrypted file"""
        file_path = self._get_file_path(key_id)
        
        try:
            # Encrypt the key data
            encrypted_payload = self._encrypt_key_data(key_data, passphrase)
            
            # Add metadata
            file_data = {
                'key_id': key_id,
                'storage_type': 'file',
                'algorithm': 'Ed25519',
                'created_at': self._get_timestamp(),
                'encrypted_key': encrypted_payload
            }
            
            # Write to file with proper permissions
            with open(file_path, 'w') as f:
                json.dump(file_data, f, indent=2)
            
            # Set file permissions to owner read/write only
            if platform.system() != "Windows":
                os.chmod(file_path, KEY_FILE_PERMISSIONS)
                
        except Exception as e:
            # Clean up partial file
            if file_path.exists():
                try:
                    file_path.unlink()
                except:
                    pass
            
            raise StorageError(
                f"File storage failed: {e}",
                "FILE_STORAGE_FAILED"
            )
    
    def _retrieve_from_file(self, key_id: str, passphrase: str) -> Optional[bytes]:
        """Retrieve key data from encrypted file"""
        file_path = self._get_file_path(key_id)
        
        if not file_path.exists():
            return None
        
        try:
            # Read file data
            with open(file_path, 'r') as f:
                file_data = json.load(f)
            
            # Decrypt the key data
            encrypted_payload = file_data['encrypted_key']
            key_data = self._decrypt_key_data(encrypted_payload, passphrase)
            
            return key_data
        except Exception as e:
            raise StorageError(
                f"File retrieval failed: {e}",
                "FILE_RETRIEVAL_FAILED"
            )
    
    def _delete_from_file(self, key_id: str) -> bool:
        """Delete key data file"""
        file_path = self._get_file_path(key_id)
        
        try:
            if file_path.exists():
                file_path.unlink()
                return True
            return False
        except Exception as e:
            raise StorageError(
                f"File deletion failed: {e}",
                "FILE_DELETION_FAILED"
            )
    
    def _get_timestamp(self) -> str:
        """Get current timestamp as ISO string"""
        from datetime import datetime
        return datetime.utcnow().isoformat() + 'Z'
    
    def store_key(self, key_id: str, key_pair: Ed25519KeyPair, 
                  passphrase: Optional[str] = None) -> StorageMetadata:
        """
        Store Ed25519 key pair securely
        
        Args:
            key_id: Unique identifier for the key
            key_pair: Ed25519 key pair to store
            passphrase: Passphrase for file encryption (required if keyring unavailable)
            
        Returns:
            StorageMetadata: Metadata about the stored key
            
        Raises:
            StorageError: If storage fails
            UnsupportedPlatformError: If no storage method available
        """
        if not key_id or not isinstance(key_id, str):
            raise StorageError("Key ID must be a non-empty string", "INVALID_KEY_ID")
        
        # Combine private and public key for storage
        key_data = key_pair.private_key + key_pair.public_key
        
        storage_type = None
        
        # Try keyring first if enabled
        if self.use_keyring:
            try:
                self._store_to_keyring(key_id, key_data)
                storage_type = 'keyring'
            except StorageError:
                # Fall through to file storage
                pass
        
        # Fall back to file storage
        if storage_type is None:
            if passphrase is None:
                raise StorageError(
                    "Passphrase required for encrypted file storage",
                    "PASSPHRASE_REQUIRED"
                )
            
            self._store_to_file(key_id, key_data, passphrase)
            storage_type = 'file'
        
        return StorageMetadata(
            key_id=key_id,
            storage_type=storage_type,
            created_at=self._get_timestamp(),
            last_accessed=self._get_timestamp(),
            algorithm='Ed25519'
        )
    
    def retrieve_key(self, key_id: str, passphrase: Optional[str] = None) -> Optional[Ed25519KeyPair]:
        """
        Retrieve Ed25519 key pair from storage
        
        Args:
            key_id: Unique identifier for the key
            passphrase: Passphrase for file decryption (required for file storage)
            
        Returns:
            Ed25519KeyPair or None if key not found
            
        Raises:
            StorageError: If retrieval fails
        """
        if not key_id or not isinstance(key_id, str):
            raise StorageError("Key ID must be a non-empty string", "INVALID_KEY_ID")
        
        key_data = None
        
        # Try keyring first if enabled
        if self.use_keyring:
            try:
                key_data = self._retrieve_from_keyring(key_id)
            except StorageError:
                # Fall through to file storage
                pass
        
        # Try file storage
        if key_data is None:
            if passphrase is None:
                # Check if file exists before requiring passphrase
                file_path = self._get_file_path(key_id)
                if file_path.exists():
                    raise StorageError(
                        "Passphrase required for encrypted file storage",
                        "PASSPHRASE_REQUIRED"
                    )
                return None
            
            key_data = self._retrieve_from_file(key_id, passphrase)
        
        if key_data is None:
            return None
        
        # Split combined key data
        if len(key_data) != 64:  # 32 bytes private + 32 bytes public
            raise StorageError(
                "Stored key data has invalid length",
                "INVALID_STORED_KEY_LENGTH"
            )
        
        private_key = key_data[:32]
        public_key = key_data[32:]
        
        return Ed25519KeyPair(private_key=private_key, public_key=public_key)
    
    def delete_key(self, key_id: str) -> bool:
        """
        Delete key from storage
        
        Args:
            key_id: Unique identifier for the key
            
        Returns:
            bool: True if key was deleted, False if not found
            
        Raises:
            StorageError: If deletion fails
        """
        if not key_id or not isinstance(key_id, str):
            raise StorageError("Key ID must be a non-empty string", "INVALID_KEY_ID")
        
        deleted_from_keyring = False
        deleted_from_file = False
        
        # Try to delete from keyring
        if self.use_keyring:
            try:
                deleted_from_keyring = self._delete_from_keyring(key_id)
            except StorageError:
                # Continue to try file deletion
                pass
        
        # Try to delete from file
        deleted_from_file = self._delete_from_file(key_id)
        
        return deleted_from_keyring or deleted_from_file
    
    def list_keys(self) -> List[str]:
        """
        List all stored key IDs
        
        Returns:
            List[str]: List of key IDs
        """
        key_ids = set()
        
        # List keys from file storage
        try:
            for file_path in self.storage_dir.glob(f"*{ENCRYPTED_FILE_EXTENSION}"):
                # Extract key ID from filename
                filename = file_path.stem
                key_ids.add(filename)
        except Exception:
            # Directory might not exist or be accessible
            pass
        
        # Note: Keyring doesn't provide a reliable way to list all keys
        # so we rely on file storage for listing
        
        return sorted(list(key_ids))
    
    def check_storage_availability(self) -> Dict[str, Any]:
        """
        Check availability of storage methods
        
        Returns:
            dict: Storage availability information
        """
        result = {
            'keyring_available': KEYRING_AVAILABLE and self.use_keyring,
            'file_storage_available': True,  # Always available
            'cryptography_available': CRYPTOGRAPHY_AVAILABLE,
            'storage_dir': str(self.storage_dir),
            'storage_dir_exists': self.storage_dir.exists(),
            'platform': platform.system()
        }
        
        if KEYRING_AVAILABLE and self.use_keyring:
            try:
                # Test keyring accessibility
                test_key = f"test_key_{secrets.token_hex(8)}"
                test_data = "test_data"
                keyring.set_password(STORAGE_SERVICE_NAME, test_key, test_data)
                retrieved = keyring.get_password(STORAGE_SERVICE_NAME, test_key)
                keyring.delete_password(STORAGE_SERVICE_NAME, test_key)
                result['keyring_functional'] = (retrieved == test_data)
            except Exception as e:
                result['keyring_functional'] = False
                result['keyring_error'] = str(e)
        else:
            result['keyring_functional'] = False
        
        return result


def get_default_storage() -> SecureKeyStorage:
    """
    Get default secure key storage instance
    
    Returns:
        SecureKeyStorage: Default storage instance
    """
    return SecureKeyStorage()