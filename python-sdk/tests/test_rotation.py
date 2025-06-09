"""
Unit tests for key rotation functionality
"""

import pytest
import secrets
import json
import tempfile
import shutil
from pathlib import Path
from unittest.mock import patch, MagicMock
from datetime import datetime, timezone, timedelta

from datafold_sdk.crypto.rotation import (
    KeyVersion,
    RotationPolicy,
    RotationMetadata,
    KeyRotationManager,
    get_default_rotation_manager,
    DEFAULT_ROTATION_INTERVAL_DAYS,
    MAX_KEY_VERSIONS,
    KEY_VERSION_SEPARATOR,
)
from datafold_sdk.crypto.ed25519 import Ed25519KeyPair, generate_key_pair
from datafold_sdk.crypto.storage import SecureKeyStorage
from datafold_sdk.crypto.derivation import DerivationParameters
from datafold_sdk.exceptions import KeyRotationError, StorageError

# Import cryptography for integration tests
try:
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
    CRYPTOGRAPHY_AVAILABLE_FOR_TESTS = True
except ImportError:
    CRYPTOGRAPHY_AVAILABLE_FOR_TESTS = False


class TestKeyVersion:
    """Test cases for KeyVersion dataclass"""
    
    def test_key_version_creation(self):
        """Test creating KeyVersion with all fields"""
        version = KeyVersion(
            version=1,
            key_id="test_key",
            versioned_key_id="test_key_v1",
            created_at="2025-06-08T15:00:00Z",
            expires_at="2025-09-08T15:00:00Z",
            derivation_params={"algorithm": "HKDF"},
            is_active=True,
            rotation_reason="Initial key"
        )
        
        assert version.version == 1
        assert version.key_id == "test_key"
        assert version.versioned_key_id == "test_key_v1"
        assert version.created_at == "2025-06-08T15:00:00Z"
        assert version.expires_at == "2025-09-08T15:00:00Z"
        assert version.derivation_params == {"algorithm": "HKDF"}
        assert version.is_active is True
        assert version.rotation_reason == "Initial key"
    
    def test_key_version_defaults(self):
        """Test KeyVersion with default values"""
        version = KeyVersion(
            version=1,
            key_id="test_key",
            versioned_key_id="test_key_v1",
            created_at="2025-06-08T15:00:00Z"
        )
        
        assert version.expires_at is None
        assert version.derivation_params is None
        assert version.is_active is True
        assert version.rotation_reason is None
    
    def test_key_version_to_dict(self):
        """Test KeyVersion serialization to dictionary"""
        version = KeyVersion(
            version=1,
            key_id="test_key",
            versioned_key_id="test_key_v1",
            created_at="2025-06-08T15:00:00Z",
            rotation_reason="Test"
        )
        
        version_dict = version.to_dict()
        
        assert isinstance(version_dict, dict)
        assert version_dict['version'] == 1
        assert version_dict['key_id'] == "test_key"
        assert version_dict['versioned_key_id'] == "test_key_v1"
        assert version_dict['created_at'] == "2025-06-08T15:00:00Z"
        assert version_dict['rotation_reason'] == "Test"
    
    def test_key_version_from_dict(self):
        """Test KeyVersion deserialization from dictionary"""
        version_dict = {
            'version': 2,
            'key_id': "test_key",
            'versioned_key_id': "test_key_v2",
            'created_at': "2025-06-08T16:00:00Z",
            'is_active': False,
            'rotation_reason': "Scheduled rotation"
        }
        
        version = KeyVersion.from_dict(version_dict)
        
        assert version.version == 2
        assert version.key_id == "test_key"
        assert version.versioned_key_id == "test_key_v2"
        assert version.created_at == "2025-06-08T16:00:00Z"
        assert version.is_active is False
        assert version.rotation_reason == "Scheduled rotation"


class TestRotationPolicy:
    """Test cases for RotationPolicy dataclass"""
    
    def test_rotation_policy_defaults(self):
        """Test RotationPolicy with default values"""
        policy = RotationPolicy()
        
        assert policy.rotation_interval_days == DEFAULT_ROTATION_INTERVAL_DAYS
        assert policy.max_versions == MAX_KEY_VERSIONS
        assert policy.auto_cleanup_expired is True
        assert policy.require_confirmation is False
        assert policy.derivation_method == 'HKDF'
        assert policy.backup_old_keys is True
    
    def test_rotation_policy_custom(self):
        """Test RotationPolicy with custom values"""
        policy = RotationPolicy(
            rotation_interval_days=30,
            max_versions=5,
            auto_cleanup_expired=False,
            require_confirmation=True,
            derivation_method='PBKDF2',
            backup_old_keys=False
        )
        
        assert policy.rotation_interval_days == 30
        assert policy.max_versions == 5
        assert policy.auto_cleanup_expired is False
        assert policy.require_confirmation is True
        assert policy.derivation_method == 'PBKDF2'
        assert policy.backup_old_keys is False


class TestRotationMetadata:
    """Test cases for RotationMetadata dataclass"""
    
    def test_rotation_metadata_creation(self):
        """Test creating RotationMetadata"""
        policy = RotationPolicy(rotation_interval_days=30)
        version = KeyVersion(
            version=1,
            key_id="test_key",
            versioned_key_id="test_key_v1",
            created_at="2025-06-08T15:00:00Z"
        )
        
        metadata = RotationMetadata(
            key_id="test_key",
            current_version=1,
            versions=[version],
            policy=policy,
            last_rotation="2025-06-08T15:00:00Z",
            next_rotation="2025-07-08T15:00:00Z",
            rotation_count=0
        )
        
        assert metadata.key_id == "test_key"
        assert metadata.current_version == 1
        assert len(metadata.versions) == 1
        assert metadata.policy == policy
        assert metadata.last_rotation == "2025-06-08T15:00:00Z"
        assert metadata.next_rotation == "2025-07-08T15:00:00Z"
        assert metadata.rotation_count == 0
    
    def test_rotation_metadata_to_dict(self):
        """Test RotationMetadata serialization to dictionary"""
        policy = RotationPolicy(rotation_interval_days=30)
        version = KeyVersion(
            version=1,
            key_id="test_key",
            versioned_key_id="test_key_v1",
            created_at="2025-06-08T15:00:00Z"
        )
        
        metadata = RotationMetadata(
            key_id="test_key",
            current_version=1,
            versions=[version],
            policy=policy,
            rotation_count=1
        )
        
        metadata_dict = metadata.to_dict()
        
        assert isinstance(metadata_dict, dict)
        assert metadata_dict['key_id'] == "test_key"
        assert metadata_dict['current_version'] == 1
        assert len(metadata_dict['versions']) == 1
        assert metadata_dict['policy']['rotation_interval_days'] == 30
        assert metadata_dict['rotation_count'] == 1
    
    def test_rotation_metadata_from_dict(self):
        """Test RotationMetadata deserialization from dictionary"""
        metadata_dict = {
            'key_id': "test_key",
            'current_version': 2,
            'versions': [
                {
                    'version': 1,
                    'key_id': "test_key",
                    'versioned_key_id': "test_key_v1",
                    'created_at': "2025-06-08T15:00:00Z",
                    'is_active': False,
                    'rotation_reason': "Initial key"
                },
                {
                    'version': 2,
                    'key_id': "test_key",
                    'versioned_key_id': "test_key_v2",
                    'created_at': "2025-06-08T16:00:00Z",
                    'is_active': True,
                    'rotation_reason': "Scheduled rotation"
                }
            ],
            'policy': {
                'rotation_interval_days': 60,
                'max_versions': 5,
                'auto_cleanup_expired': True,
                'require_confirmation': False,
                'derivation_method': 'HKDF',
                'backup_old_keys': True
            },
            'rotation_count': 1
        }
        
        metadata = RotationMetadata.from_dict(metadata_dict)
        
        assert metadata.key_id == "test_key"
        assert metadata.current_version == 2
        assert len(metadata.versions) == 2
        assert metadata.versions[0].version == 1
        assert metadata.versions[1].version == 2
        assert metadata.policy.rotation_interval_days == 60
        assert metadata.rotation_count == 1


class TestKeyRotationManager:
    """Test cases for KeyRotationManager"""
    
    def setUp(self):
        """Set up test environment"""
        self.temp_dir = tempfile.mkdtemp()
        self.storage = SecureKeyStorage(storage_dir=self.temp_dir, use_keyring=False)
        self.manager = KeyRotationManager(self.storage, metadata_dir=self.temp_dir)
    
    def tearDown(self):
        """Clean up test environment"""
        shutil.rmtree(self.temp_dir, ignore_errors=True)
    
    def test_init(self):
        """Test KeyRotationManager initialization"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage)
            
            assert manager.storage == storage
            assert manager.metadata_dir == storage.storage_dir
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)
    
    def test_get_versioned_key_id(self):
        """Test versioned key ID generation"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage)
            
            versioned_id = manager._get_versioned_key_id("test_key", 3)
            assert versioned_id == f"test_key{KEY_VERSION_SEPARATOR}3"
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)
    
    def test_parse_versioned_key_id(self):
        """Test parsing versioned key ID"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage)
            
            base_id, version = manager._parse_versioned_key_id(f"test_key{KEY_VERSION_SEPARATOR}5")
            assert base_id == "test_key"
            assert version == 5
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)
    
    def test_parse_versioned_key_id_invalid(self):
        """Test parsing invalid versioned key ID"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage)
            
            with pytest.raises(KeyRotationError, match="Invalid versioned key ID format"):
                manager._parse_versioned_key_id("invalid_key_id")
            
            with pytest.raises(KeyRotationError, match="Failed to parse versioned key ID"):
                manager._parse_versioned_key_id(f"test_key{KEY_VERSION_SEPARATOR}not_number")
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_initialize_key_rotation(self):
        """Test initializing key rotation for a new key"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage, metadata_dir=temp_dir)
            
            key_pair = generate_key_pair()
            policy = RotationPolicy(rotation_interval_days=30)
            
            metadata = manager.initialize_key_rotation("test_key", key_pair, policy, "test_passphrase")
            
            assert metadata.key_id == "test_key"
            assert metadata.current_version == 1
            assert len(metadata.versions) == 1
            assert metadata.versions[0].version == 1
            assert metadata.versions[0].is_active is True
            assert metadata.policy == policy
            assert metadata.rotation_count == 0
            
            # Check that key was stored
            stored_key = storage.retrieve_key("test_key_v1", "test_passphrase")
            assert stored_key is not None
            assert stored_key.private_key == key_pair.private_key
            assert stored_key.public_key == key_pair.public_key
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)
    
    def test_initialize_key_rotation_invalid_key_id(self):
        """Test initializing rotation with invalid key ID"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage, metadata_dir=temp_dir)
            
            key_pair = generate_key_pair() if CRYPTOGRAPHY_AVAILABLE_FOR_TESTS else MagicMock()
            
            with pytest.raises(KeyRotationError, match="Key ID must be a non-empty string"):
                manager.initialize_key_rotation("", key_pair)
            
            with pytest.raises(KeyRotationError, match="Key ID must be a non-empty string"):
                manager.initialize_key_rotation(None, key_pair)
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_initialize_key_rotation_already_exists(self):
        """Test initializing rotation for key that already exists"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage, metadata_dir=temp_dir)
            
            key_pair = generate_key_pair()
            
            # Initialize first time - should succeed
            manager.initialize_key_rotation("test_key", key_pair, passphrase="test_passphrase")
            
            # Initialize second time - should fail
            with pytest.raises(KeyRotationError, match="Key test_key already has rotation metadata"):
                manager.initialize_key_rotation("test_key", key_pair, passphrase="test_passphrase")
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_rotate_key(self):
        """Test rotating a key to a new version"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage, metadata_dir=temp_dir)
            
            # Initialize key rotation
            initial_key = generate_key_pair()
            manager.initialize_key_rotation("test_key", initial_key, passphrase="test_passphrase")
            
            # Rotate key
            new_key, updated_metadata = manager.rotate_key(
                "test_key",
                passphrase="test_passphrase",
                rotation_reason="Test rotation"
            )
            
            assert isinstance(new_key, Ed25519KeyPair)
            assert updated_metadata.current_version == 2
            assert len(updated_metadata.versions) == 2
            assert updated_metadata.rotation_count == 1
            
            # Check that old version is deactivated
            old_version = next(v for v in updated_metadata.versions if v.version == 1)
            assert old_version.is_active is False
            
            # Check that new version is active
            new_version = next(v for v in updated_metadata.versions if v.version == 2)
            assert new_version.is_active is True
            assert new_version.rotation_reason == "Test rotation"
            
            # Check that new key was stored and is different from old key
            stored_new_key = storage.retrieve_key("test_key_v2", "test_passphrase")
            assert stored_new_key is not None
            assert stored_new_key.private_key != initial_key.private_key
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)
    
    def test_rotate_key_not_initialized(self):
        """Test rotating key that wasn't initialized"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage, metadata_dir=temp_dir)
            
            with pytest.raises(KeyRotationError, match="Key test_key not initialized for rotation"):
                manager.rotate_key("test_key", passphrase="test_passphrase")
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_get_current_key(self):
        """Test getting current active key"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage, metadata_dir=temp_dir)
            
            # Initialize and rotate key
            initial_key = generate_key_pair()
            manager.initialize_key_rotation("test_key", initial_key, passphrase="test_passphrase")
            new_key, _ = manager.rotate_key("test_key", passphrase="test_passphrase")
            
            # Get current key should return the rotated key
            current_key = manager.get_current_key("test_key", passphrase="test_passphrase")
            
            assert current_key is not None
            assert current_key.private_key == new_key.private_key
            assert current_key.public_key == new_key.public_key
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)
    
    def test_get_current_key_not_found(self):
        """Test getting current key for non-existent key"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage, metadata_dir=temp_dir)
            
            current_key = manager.get_current_key("nonexistent_key")
            assert current_key is None
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_get_key_version(self):
        """Test getting specific key version"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage, metadata_dir=temp_dir)
            
            # Initialize and rotate key
            initial_key = generate_key_pair()
            manager.initialize_key_rotation("test_key", initial_key, passphrase="test_passphrase")
            manager.rotate_key("test_key", passphrase="test_passphrase")
            
            # Get version 1
            version_1_key = manager.get_key_version("test_key", 1, passphrase="test_passphrase")
            assert version_1_key is not None
            assert version_1_key.private_key == initial_key.private_key
            
            # Get version 2
            version_2_key = manager.get_key_version("test_key", 2, passphrase="test_passphrase")
            assert version_2_key is not None
            assert version_2_key.private_key != initial_key.private_key
            
            # Get non-existent version
            version_3_key = manager.get_key_version("test_key", 3, passphrase="test_passphrase")
            assert version_3_key is None
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_list_key_versions(self):
        """Test listing all key versions"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage, metadata_dir=temp_dir)
            
            # Initialize and rotate key multiple times
            initial_key = generate_key_pair()
            manager.initialize_key_rotation("test_key", initial_key, passphrase="test_passphrase")
            manager.rotate_key("test_key", passphrase="test_passphrase")
            manager.rotate_key("test_key", passphrase="test_passphrase")
            
            versions = manager.list_key_versions("test_key")
            
            assert len(versions) == 3
            assert versions[0].version == 1
            assert versions[1].version == 2
            assert versions[2].version == 3
            
            # Only version 3 should be active
            active_versions = [v for v in versions if v.is_active]
            assert len(active_versions) == 1
            assert active_versions[0].version == 3
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)
    
    def test_list_key_versions_not_found(self):
        """Test listing versions for non-existent key"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage, metadata_dir=temp_dir)
            
            versions = manager.list_key_versions("nonexistent_key")
            assert versions == []
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_expire_version(self):
        """Test expiring a specific key version"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage, metadata_dir=temp_dir)
            
            # Initialize and rotate key
            initial_key = generate_key_pair()
            manager.initialize_key_rotation("test_key", initial_key, passphrase="test_passphrase")
            manager.rotate_key("test_key", passphrase="test_passphrase")
            
            # Expire version 1
            result = manager.expire_version("test_key", 1)
            assert result is True
            
            # Check that version 1 is expired
            metadata = manager.get_rotation_metadata("test_key")
            version_1 = next(v for v in metadata.versions if v.version == 1)
            assert version_1.is_active is False
            assert version_1.expires_at is not None
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_expire_current_version_error(self):
        """Test that expiring the only active version raises error"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage, metadata_dir=temp_dir)
            
            # Initialize key (only one version)
            initial_key = generate_key_pair()
            manager.initialize_key_rotation("test_key", initial_key, passphrase="test_passphrase")
            
            # Try to expire the only version
            with pytest.raises(KeyRotationError, match="Cannot expire current version 1"):
                manager.expire_version("test_key", 1)
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_delete_key_completely(self):
        """Test completely deleting a key and all versions"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage, metadata_dir=temp_dir)
            
            # Initialize and rotate key
            initial_key = generate_key_pair()
            manager.initialize_key_rotation("test_key", initial_key, passphrase="test_passphrase")
            manager.rotate_key("test_key", passphrase="test_passphrase")
            
            # Delete completely
            result = manager.delete_key_completely("test_key")
            assert result is True
            
            # Check that key is gone
            current_key = manager.get_current_key("test_key")
            assert current_key is None
            
            metadata = manager.get_rotation_metadata("test_key")
            assert metadata is None
            
            # Check that storage doesn't contain the keys
            stored_key_v1 = storage.retrieve_key("test_key_v1", "test_passphrase")
            assert stored_key_v1 is None
            
            stored_key_v2 = storage.retrieve_key("test_key_v2", "test_passphrase")
            assert stored_key_v2 is None
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)
    
    def test_delete_key_completely_not_found(self):
        """Test deleting non-existent key"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage, metadata_dir=temp_dir)
            
            result = manager.delete_key_completely("nonexistent_key")
            assert result is False
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_check_rotation_due(self):
        """Test checking if rotation is due"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage, metadata_dir=temp_dir)
            
            # Initialize key with short rotation interval
            initial_key = generate_key_pair()
            policy = RotationPolicy(rotation_interval_days=1)  # 1 day
            manager.initialize_key_rotation("test_key", initial_key, policy, "test_passphrase")
            
            # Should not be due immediately
            assert manager.check_rotation_due("test_key") is False
            
            # Mock the next rotation time to be in the past
            metadata = manager.get_rotation_metadata("test_key")
            past_time = datetime.now(timezone.utc) - timedelta(days=1)
            metadata.next_rotation = past_time.isoformat()
            manager._save_rotation_metadata(metadata)
            
            # Should now be due
            assert manager.check_rotation_due("test_key") is True
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)
    
    def test_check_rotation_due_not_found(self):
        """Test checking rotation due for non-existent key"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage, metadata_dir=temp_dir)
            
            assert manager.check_rotation_due("nonexistent_key") is False
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_list_managed_keys(self):
        """Test listing all managed keys"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage, metadata_dir=temp_dir)
            
            # Initialize multiple keys
            for key_name in ["key1", "key2", "key3"]:
                key_pair = generate_key_pair()
                manager.initialize_key_rotation(key_name, key_pair, passphrase="test_passphrase")
            
            managed_keys = manager.list_managed_keys()
            
            assert len(managed_keys) == 3
            assert "key1" in managed_keys
            assert "key2" in managed_keys
            assert "key3" in managed_keys
            assert managed_keys == sorted(managed_keys)  # Should be sorted
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)
    
    def test_list_managed_keys_empty(self):
        """Test listing managed keys when none exist"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage, metadata_dir=temp_dir)
            
            managed_keys = manager.list_managed_keys()
            assert managed_keys == []
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)


class TestIntegration:
    """Integration tests for key rotation functionality"""
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_complete_rotation_workflow(self):
        """Test complete key rotation workflow"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage, metadata_dir=temp_dir)
            
            # Step 1: Initialize key rotation
            initial_key = generate_key_pair()
            policy = RotationPolicy(
                rotation_interval_days=30,
                max_versions=3,
                derivation_method='HKDF'
            )
            
            metadata = manager.initialize_key_rotation(
                "test_workflow_key", 
                initial_key, 
                policy, 
                "workflow_passphrase"
            )
            
            assert metadata.current_version == 1
            assert len(metadata.versions) == 1
            
            # Step 2: Perform multiple rotations
            for i in range(4):  # Rotate 4 times (will exceed max_versions)
                new_key, updated_metadata = manager.rotate_key(
                    "test_workflow_key",
                    passphrase="workflow_passphrase",
                    rotation_reason=f"Rotation {i+1}"
                )
                
                assert isinstance(new_key, Ed25519KeyPair)
                assert updated_metadata.current_version == i + 2
                assert updated_metadata.rotation_count == i + 1
            
            # Step 3: Check that old versions were cleaned up
            final_metadata = manager.get_rotation_metadata("test_workflow_key")
            assert len(final_metadata.versions) <= policy.max_versions
            
            # Step 4: Verify current key can be retrieved
            current_key = manager.get_current_key("test_workflow_key", "workflow_passphrase")
            assert current_key is not None
            
            # Step 5: Test versioned access to remaining keys
            versions = manager.list_key_versions("test_workflow_key")
            for version in versions:
                if version.is_active or not policy.auto_cleanup_expired:
                    retrieved_key = manager.get_key_version(
                        "test_workflow_key", 
                        version.version, 
                        "workflow_passphrase"
                    )
                    assert retrieved_key is not None
            
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_rotation_with_different_derivation_methods(self):
        """Test rotation using different derivation methods"""
        temp_dir = tempfile.mkdtemp()
        try:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            manager = KeyRotationManager(storage, metadata_dir=temp_dir)
            
            # Initialize with HKDF
            initial_key = generate_key_pair()
            manager.initialize_key_rotation("test_derivation", initial_key, passphrase="test_pass")
            
            # Rotate using different methods
            methods = ['HKDF', 'PBKDF2', 'Scrypt']
            for i, method in enumerate(methods):
                new_key, metadata = manager.rotate_key(
                    "test_derivation",
                    passphrase="test_pass",
                    derivation_method=method,
                    rotation_reason=f"Test {method}"
                )
                
                # Check that the latest version has the correct derivation method
                latest_version = max(metadata.versions, key=lambda v: v.version)
                if latest_version.derivation_params:
                    from datafold_sdk.crypto.derivation import import_derivation_parameters
                    params = import_derivation_parameters(latest_version.derivation_params)
                    assert params.algorithm == method
        
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)


def test_get_default_rotation_manager():
    """Test getting default rotation manager"""
    manager = get_default_rotation_manager()
    
    assert isinstance(manager, KeyRotationManager)
    assert manager.storage is not None


if __name__ == '__main__':
    pytest.main([__file__])