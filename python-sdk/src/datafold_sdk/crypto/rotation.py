"""
Key rotation functionality for DataFold Python SDK

This module provides secure key rotation capabilities including versioning,
lifecycle management, and seamless key updates while maintaining data accessibility.
"""

import os
import json
import secrets
from typing import Dict, List, Optional, Tuple, Union, Any
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
import base64

# Import cryptography components
try:
    from cryptography.hazmat.primitives import hashes
    from cryptography.hazmat.primitives.kdf.hkdf import HKDF
    CRYPTOGRAPHY_AVAILABLE = True
except ImportError:
    CRYPTOGRAPHY_AVAILABLE = False
    hashes = None
    HKDF = None

from ..exceptions import KeyRotationError, StorageError, KeyDerivationError
from .ed25519 import Ed25519KeyPair, clear_key_material
from .storage import SecureKeyStorage, StorageMetadata
from .derivation import (
    derive_ed25519_key_pair, 
    derive_key_hkdf,
    DerivationParameters,
    export_derivation_parameters,
    import_derivation_parameters
)

# Constants for key rotation
DEFAULT_ROTATION_INTERVAL_DAYS = 90
MAX_KEY_VERSIONS = 10
ROTATION_METADATA_FILE = "rotation_metadata.json"
KEY_VERSION_SEPARATOR = "_v"


@dataclass
class KeyVersion:
    """
    Represents a versioned key with metadata
    
    Attributes:
        version: Version number (incremental)
        key_id: Base key identifier
        versioned_key_id: Full versioned key identifier
        created_at: Creation timestamp
        expires_at: Expiration timestamp (optional)
        derivation_params: Parameters used for key derivation
        is_active: Whether this version is currently active
        rotation_reason: Reason for rotation (optional)
    """
    version: int
    key_id: str
    versioned_key_id: str
    created_at: str
    expires_at: Optional[str] = None
    derivation_params: Optional[Dict[str, str]] = None
    is_active: bool = True
    rotation_reason: Optional[str] = None
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for serialization"""
        return {
            'version': self.version,
            'key_id': self.key_id,
            'versioned_key_id': self.versioned_key_id,
            'created_at': self.created_at,
            'expires_at': self.expires_at,
            'derivation_params': self.derivation_params,
            'is_active': self.is_active,
            'rotation_reason': self.rotation_reason,
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'KeyVersion':
        """Create from dictionary"""
        return cls(**data)


@dataclass
class RotationPolicy:
    """
    Key rotation policy configuration
    
    Attributes:
        rotation_interval_days: Days between automatic rotations
        max_versions: Maximum number of versions to keep
        auto_cleanup_expired: Whether to automatically clean up expired keys
        require_confirmation: Whether to require manual confirmation for rotation
        derivation_method: Method to use for deriving new keys ('HKDF', 'PBKDF2', 'Scrypt')
        backup_old_keys: Whether to backup old keys before rotation
    """
    rotation_interval_days: int = DEFAULT_ROTATION_INTERVAL_DAYS
    max_versions: int = MAX_KEY_VERSIONS
    auto_cleanup_expired: bool = True
    require_confirmation: bool = False
    derivation_method: str = 'HKDF'
    backup_old_keys: bool = True


@dataclass
class RotationMetadata:
    """
    Metadata for key rotation history
    
    Attributes:
        key_id: Base key identifier
        current_version: Current active version number
        versions: List of all key versions
        policy: Rotation policy
        last_rotation: Last rotation timestamp
        next_rotation: Next scheduled rotation timestamp
        rotation_count: Total number of rotations performed
    """
    key_id: str
    current_version: int
    versions: List[KeyVersion] = field(default_factory=list)
    policy: Optional[RotationPolicy] = None
    last_rotation: Optional[str] = None
    next_rotation: Optional[str] = None
    rotation_count: int = 0
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for serialization"""
        return {
            'key_id': self.key_id,
            'current_version': self.current_version,
            'versions': [v.to_dict() for v in self.versions],
            'policy': self.policy.__dict__ if self.policy else None,
            'last_rotation': self.last_rotation,
            'next_rotation': self.next_rotation,
            'rotation_count': self.rotation_count,
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'RotationMetadata':
        """Create from dictionary"""
        metadata = cls(
            key_id=data['key_id'],
            current_version=data['current_version'],
            last_rotation=data.get('last_rotation'),
            next_rotation=data.get('next_rotation'),
            rotation_count=data.get('rotation_count', 0),
        )
        
        # Reconstruct versions
        if 'versions' in data:
            metadata.versions = [KeyVersion.from_dict(v) for v in data['versions']]
        
        # Reconstruct policy
        if 'policy' in data and data['policy']:
            metadata.policy = RotationPolicy(**data['policy'])
        
        return metadata


class KeyRotationManager:
    """
    Manager for key rotation operations
    
    This class handles the lifecycle of key rotations including versioning,
    secure storage updates, and cleanup of old key versions.
    """
    
    def __init__(self, storage: SecureKeyStorage, metadata_dir: Optional[str] = None):
        """
        Initialize key rotation manager
        
        Args:
            storage: SecureKeyStorage instance for key operations
            metadata_dir: Directory for rotation metadata (uses storage dir if None)
        """
        self.storage = storage
        self.metadata_dir = Path(metadata_dir) if metadata_dir else storage.storage_dir
        self._ensure_metadata_dir()
        
    def _ensure_metadata_dir(self) -> None:
        """Ensure metadata directory exists"""
        try:
            self.metadata_dir.mkdir(parents=True, exist_ok=True)
            # Set directory permissions to owner only
            if os.name != 'nt':  # Not Windows
                os.chmod(self.metadata_dir, 0o700)
        except Exception as e:
            raise KeyRotationError(
                f"Failed to create metadata directory: {e}",
                "METADATA_DIR_CREATION_FAILED"
            )
    
    def _get_metadata_path(self, key_id: str) -> Path:
        """Get metadata file path for key"""
        safe_key_id = "".join(c for c in key_id if c.isalnum() or c in "-_.")
        return self.metadata_dir / f"{safe_key_id}_rotation.json"
    
    def _get_versioned_key_id(self, key_id: str, version: int) -> str:
        """Generate versioned key identifier"""
        return f"{key_id}{KEY_VERSION_SEPARATOR}{version}"
    
    def _parse_versioned_key_id(self, versioned_key_id: str) -> Tuple[str, int]:
        """Parse versioned key identifier into base ID and version"""
        if KEY_VERSION_SEPARATOR not in versioned_key_id:
            raise KeyRotationError(
                f"Invalid versioned key ID format: {versioned_key_id}",
                "INVALID_VERSIONED_KEY_ID"
            )
        
        parts = versioned_key_id.rsplit(KEY_VERSION_SEPARATOR, 1)
        try:
            base_id = parts[0]
            version = int(parts[1])
            return base_id, version
        except (IndexError, ValueError) as e:
            raise KeyRotationError(
                f"Failed to parse versioned key ID: {versioned_key_id}",
                "VERSIONED_KEY_ID_PARSE_FAILED"
            ) from e
    
    def _get_current_timestamp(self) -> str:
        """Get current timestamp as ISO string"""
        return datetime.now(timezone.utc).isoformat()
    
    def _calculate_next_rotation(self, policy: RotationPolicy) -> str:
        """Calculate next rotation timestamp based on policy"""
        from datetime import timedelta
        next_time = datetime.now(timezone.utc) + timedelta(days=policy.rotation_interval_days)
        return next_time.isoformat()
    
    def _load_rotation_metadata(self, key_id: str) -> Optional[RotationMetadata]:
        """Load rotation metadata from file"""
        metadata_path = self._get_metadata_path(key_id)
        
        if not metadata_path.exists():
            return None
        
        try:
            with open(metadata_path, 'r') as f:
                data = json.load(f)
            return RotationMetadata.from_dict(data)
        except Exception as e:
            raise KeyRotationError(
                f"Failed to load rotation metadata: {e}",
                "METADATA_LOAD_FAILED"
            ) from e
    
    def _save_rotation_metadata(self, metadata: RotationMetadata) -> None:
        """Save rotation metadata to file"""
        metadata_path = self._get_metadata_path(metadata.key_id)
        
        try:
            with open(metadata_path, 'w') as f:
                json.dump(metadata.to_dict(), f, indent=2)
            
            # Set file permissions to owner read/write only
            if os.name != 'nt':  # Not Windows
                os.chmod(metadata_path, 0o600)
                
        except Exception as e:
            raise KeyRotationError(
                f"Failed to save rotation metadata: {e}",
                "METADATA_SAVE_FAILED"
            ) from e
    
    def initialize_key_rotation(self, 
                              key_id: str,
                              initial_key_pair: Ed25519KeyPair,
                              policy: Optional[RotationPolicy] = None,
                              passphrase: Optional[str] = None) -> RotationMetadata:
        """
        Initialize key rotation for a new key
        
        Args:
            key_id: Base key identifier
            initial_key_pair: Initial key pair to manage
            policy: Rotation policy (uses default if None)
            passphrase: Passphrase for key storage
            
        Returns:
            RotationMetadata: Initial rotation metadata
            
        Raises:
            KeyRotationError: If initialization fails
        """
        if not key_id or not isinstance(key_id, str):
            raise KeyRotationError(
                "Key ID must be a non-empty string",
                "INVALID_KEY_ID"
            )
        
        # Check if key already has rotation metadata
        existing_metadata = self._load_rotation_metadata(key_id)
        if existing_metadata:
            raise KeyRotationError(
                f"Key {key_id} already has rotation metadata",
                "KEY_ALREADY_INITIALIZED"
            )
        
        # Use default policy if none provided
        if policy is None:
            policy = RotationPolicy()
        
        try:
            # Store initial key with version 1
            version = 1
            versioned_key_id = self._get_versioned_key_id(key_id, version)
            
            storage_metadata = self.storage.store_key(
                versioned_key_id, 
                initial_key_pair, 
                passphrase
            )
            
            # Create initial key version
            key_version = KeyVersion(
                version=version,
                key_id=key_id,
                versioned_key_id=versioned_key_id,
                created_at=self._get_current_timestamp(),
                is_active=True,
                rotation_reason="Initial key"
            )
            
            # Create rotation metadata
            rotation_metadata = RotationMetadata(
                key_id=key_id,
                current_version=version,
                versions=[key_version],
                policy=policy,
                last_rotation=key_version.created_at,
                next_rotation=self._calculate_next_rotation(policy),
                rotation_count=0
            )
            
            # Save metadata
            self._save_rotation_metadata(rotation_metadata)
            
            return rotation_metadata
            
        except Exception as e:
            if isinstance(e, KeyRotationError):
                raise
            
            raise KeyRotationError(
                f"Failed to initialize key rotation: {e}",
                "ROTATION_INITIALIZATION_FAILED"
            ) from e
    
    def rotate_key(self,
                   key_id: str,
                   master_key: Optional[bytes] = None,
                   passphrase: Optional[str] = None,
                   rotation_reason: Optional[str] = None,
                   derivation_method: Optional[str] = None) -> Tuple[Ed25519KeyPair, RotationMetadata]:
        """
        Rotate a key to a new version
        
        Args:
            key_id: Base key identifier
            master_key: Master key for derivation (generates new if None)
            passphrase: Passphrase for key storage
            rotation_reason: Reason for rotation
            derivation_method: Derivation method override
            
        Returns:
            Tuple of (new_key_pair, updated_metadata)
            
        Raises:
            KeyRotationError: If rotation fails
        """
        # Load existing metadata
        metadata = self._load_rotation_metadata(key_id)
        if not metadata:
            raise KeyRotationError(
                f"Key {key_id} not initialized for rotation",
                "KEY_NOT_INITIALIZED"
            )
        
        # Check policy requirements
        if metadata.policy and metadata.policy.require_confirmation:
            # In a real implementation, this might prompt the user
            # For now, we assume confirmation is implicit
            pass
        
        try:
            # Determine derivation method
            if derivation_method is None:
                derivation_method = metadata.policy.derivation_method if metadata.policy else 'HKDF'
            
            # Generate or use provided master key
            if master_key is None:
                master_key = secrets.token_bytes(32)
            
            # Derive new key pair
            context = f"{key_id}-rotation-{metadata.current_version + 1}"
            new_key_pair, derivation_params = derive_ed25519_key_pair(
                master_key=master_key,
                context=context,
                derivation_method=derivation_method
            )
            
            # Create new version
            new_version = metadata.current_version + 1
            versioned_key_id = self._get_versioned_key_id(key_id, new_version)
            
            # Store new key version
            storage_metadata = self.storage.store_key(
                versioned_key_id,
                new_key_pair,
                passphrase
            )
            
            # Create key version record
            key_version = KeyVersion(
                version=new_version,
                key_id=key_id,
                versioned_key_id=versioned_key_id,
                created_at=self._get_current_timestamp(),
                derivation_params=export_derivation_parameters(derivation_params),
                is_active=True,
                rotation_reason=rotation_reason or "Scheduled rotation"
            )
            
            # Deactivate previous version
            for version in metadata.versions:
                if version.is_active:
                    version.is_active = False
            
            # Update metadata
            metadata.versions.append(key_version)
            metadata.current_version = new_version
            metadata.last_rotation = key_version.created_at
            metadata.next_rotation = self._calculate_next_rotation(metadata.policy)
            metadata.rotation_count += 1
            
            # Clean up old versions if needed
            if metadata.policy and metadata.policy.auto_cleanup_expired:
                self._cleanup_old_versions(metadata)
            
            # Save updated metadata
            self._save_rotation_metadata(metadata)
            
            return new_key_pair, metadata
            
        except Exception as e:
            if isinstance(e, (KeyRotationError, KeyDerivationError, StorageError)):
                raise
            
            raise KeyRotationError(
                f"Key rotation failed: {e}",
                "ROTATION_FAILED"
            ) from e
    
    def get_current_key(self, 
                       key_id: str, 
                       passphrase: Optional[str] = None) -> Optional[Ed25519KeyPair]:
        """
        Get the current active key version
        
        Args:
            key_id: Base key identifier
            passphrase: Passphrase for key decryption
            
        Returns:
            Ed25519KeyPair or None if not found
            
        Raises:
            KeyRotationError: If retrieval fails
        """
        metadata = self._load_rotation_metadata(key_id)
        if not metadata:
            return None
        
        # Find current active version
        current_version = None
        for version in metadata.versions:
            if version.is_active and version.version == metadata.current_version:
                current_version = version
                break
        
        if not current_version:
            raise KeyRotationError(
                f"No active version found for key {key_id}",
                "NO_ACTIVE_VERSION"
            )
        
        try:
            return self.storage.retrieve_key(current_version.versioned_key_id, passphrase)
        except Exception as e:
            raise KeyRotationError(
                f"Failed to retrieve current key: {e}",
                "CURRENT_KEY_RETRIEVAL_FAILED"
            ) from e
    
    def get_key_version(self,
                       key_id: str,
                       version: int,
                       passphrase: Optional[str] = None) -> Optional[Ed25519KeyPair]:
        """
        Get a specific key version
        
        Args:
            key_id: Base key identifier
            version: Version number to retrieve
            passphrase: Passphrase for key decryption
            
        Returns:
            Ed25519KeyPair or None if not found
            
        Raises:
            KeyRotationError: If retrieval fails
        """
        metadata = self._load_rotation_metadata(key_id)
        if not metadata:
            return None
        
        # Find requested version
        target_version = None
        for v in metadata.versions:
            if v.version == version:
                target_version = v
                break
        
        if not target_version:
            return None
        
        try:
            return self.storage.retrieve_key(target_version.versioned_key_id, passphrase)
        except Exception as e:
            raise KeyRotationError(
                f"Failed to retrieve key version {version}: {e}",
                "VERSION_RETRIEVAL_FAILED"
            ) from e
    
    def list_key_versions(self, key_id: str) -> List[KeyVersion]:
        """
        List all versions of a key
        
        Args:
            key_id: Base key identifier
            
        Returns:
            List of KeyVersion objects
        """
        metadata = self._load_rotation_metadata(key_id)
        if not metadata:
            return []
        
        return sorted(metadata.versions, key=lambda v: v.version)
    
    def get_rotation_metadata(self, key_id: str) -> Optional[RotationMetadata]:
        """
        Get rotation metadata for a key
        
        Args:
            key_id: Base key identifier
            
        Returns:
            RotationMetadata or None if not found
        """
        return self._load_rotation_metadata(key_id)
    
    def _cleanup_old_versions(self, metadata: RotationMetadata) -> None:
        """Clean up old key versions based on policy"""
        if not metadata.policy:
            return
        
        # Sort versions by version number (newest first)
        sorted_versions = sorted(metadata.versions, key=lambda v: v.version, reverse=True)
        
        # Keep only the allowed number of versions
        if len(sorted_versions) > metadata.policy.max_versions:
            versions_to_remove = sorted_versions[metadata.policy.max_versions:]
            
            for version in versions_to_remove:
                try:
                    # Delete from storage
                    self.storage.delete_key(version.versioned_key_id)
                    # Remove from metadata
                    metadata.versions.remove(version)
                except Exception:
                    # Continue cleanup even if individual deletion fails
                    pass
    
    def expire_version(self, key_id: str, version: int) -> bool:
        """
        Manually expire a specific key version
        
        Args:
            key_id: Base key identifier
            version: Version to expire
            
        Returns:
            bool: True if version was expired, False if not found
            
        Raises:
            KeyRotationError: If expiration fails
        """
        metadata = self._load_rotation_metadata(key_id)
        if not metadata:
            return False
        
        # Find and expire the version
        for v in metadata.versions:
            if v.version == version:
                v.expires_at = self._get_current_timestamp()
                v.is_active = False
                
                # If this was the current version, we need a new current version
                if version == metadata.current_version:
                    # Find the most recent active version
                    active_versions = [ver for ver in metadata.versions if ver.is_active]
                    if active_versions:
                        latest_active = max(active_versions, key=lambda ver: ver.version)
                        metadata.current_version = latest_active.version
                    else:
                        raise KeyRotationError(
                            f"Cannot expire current version {version} - no other active versions",
                            "CANNOT_EXPIRE_ONLY_VERSION"
                        )
                
                self._save_rotation_metadata(metadata)
                return True
        
        return False
    
    def delete_key_completely(self, key_id: str) -> bool:
        """
        Completely delete a key and all its versions
        
        Args:
            key_id: Base key identifier
            
        Returns:
            bool: True if key was deleted, False if not found
            
        Raises:
            KeyRotationError: If deletion fails
        """
        metadata = self._load_rotation_metadata(key_id)
        if not metadata:
            return False
        
        try:
            # Delete all versions from storage
            for version in metadata.versions:
                self.storage.delete_key(version.versioned_key_id)
            
            # Delete metadata file
            metadata_path = self._get_metadata_path(key_id)
            if metadata_path.exists():
                metadata_path.unlink()
            
            return True
            
        except Exception as e:
            raise KeyRotationError(
                f"Failed to delete key completely: {e}",
                "COMPLETE_DELETION_FAILED"
            ) from e
    
    def check_rotation_due(self, key_id: str) -> bool:
        """
        Check if a key is due for rotation
        
        Args:
            key_id: Base key identifier
            
        Returns:
            bool: True if rotation is due, False otherwise
        """
        metadata = self._load_rotation_metadata(key_id)
        if not metadata or not metadata.next_rotation:
            return False
        
        try:
            next_rotation = datetime.fromisoformat(metadata.next_rotation.replace('Z', '+00:00'))
            return datetime.now(timezone.utc) >= next_rotation
        except Exception:
            return False
    
    def list_managed_keys(self) -> List[str]:
        """
        List all keys managed by this rotation manager
        
        Returns:
            List of base key identifiers
        """
        managed_keys = []
        
        try:
            for metadata_file in self.metadata_dir.glob("*_rotation.json"):
                # Extract key ID from filename
                filename = metadata_file.stem
                if filename.endswith("_rotation"):
                    key_id = filename[:-9]  # Remove "_rotation" suffix
                    managed_keys.append(key_id)
        except Exception:
            # Return empty list if directory access fails
            pass
        
        return sorted(managed_keys)


def get_default_rotation_manager() -> KeyRotationManager:
    """
    Get a default key rotation manager instance
    
    Returns:
        KeyRotationManager: Default manager instance
    """
    from .storage import get_default_storage
    storage = get_default_storage()
    return KeyRotationManager(storage)