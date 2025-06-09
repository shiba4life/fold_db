"""
Unit tests for secure key storage functionality
"""

import pytest
import os
import tempfile
import shutil
import platform
import json
import secrets
from pathlib import Path
from unittest.mock import patch, MagicMock, mock_open

from datafold_sdk.crypto.storage import (
    SecureKeyStorage,
    StorageMetadata,
    get_default_storage,
    STORAGE_SERVICE_NAME,
    KEY_FILE_PERMISSIONS,
    SALT_LENGTH,
)
from datafold_sdk.crypto.ed25519 import Ed25519KeyPair, generate_key_pair
from datafold_sdk.exceptions import StorageError, UnsupportedPlatformError

# Import optional dependencies for testing
try:
    import keyring
    from keyring.errors import KeyringError, KeyringLocked
    KEYRING_AVAILABLE_FOR_TESTS = True
except ImportError:
    KEYRING_AVAILABLE_FOR_TESTS = False
    keyring = None
    KeyringError = Exception
    KeyringLocked = Exception


class TestStorageMetadata:
    """Test cases for StorageMetadata dataclass"""
    
    def test_storage_metadata_creation(self):
        """Test creating StorageMetadata"""
        metadata = StorageMetadata(
            key_id="test_key",
            storage_type="keyring",
            created_at="2025-06-08T15:00:00Z",
            last_accessed="2025-06-08T15:00:00Z",
            algorithm="Ed25519"
        )
        
        assert metadata.key_id == "test_key"
        assert metadata.storage_type == "keyring"
        assert metadata.algorithm == "Ed25519"


class TestSecureKeyStorageInit:
    """Test cases for SecureKeyStorage initialization"""
    
    def test_init_default(self):
        """Test default initialization"""
        storage = SecureKeyStorage()
        
        assert storage.storage_dir is not None
        assert storage.storage_dir.exists()
    
    def test_init_custom_directory(self):
        """Test initialization with custom directory"""
        with tempfile.TemporaryDirectory() as temp_dir:
            custom_dir = Path(temp_dir) / "custom_storage"
            storage = SecureKeyStorage(storage_dir=str(custom_dir))
            
            assert storage.storage_dir == custom_dir
            assert storage.storage_dir.exists()
    
    def test_init_disable_keyring(self):
        """Test initialization with keyring disabled"""
        storage = SecureKeyStorage(use_keyring=False)
        
        assert storage.use_keyring is False
    
    def test_storage_directory_permissions(self):
        """Test that storage directory has proper permissions"""
        if platform.system() == "Windows":
            pytest.skip("Permission tests not applicable on Windows")
        
        with tempfile.TemporaryDirectory() as temp_dir:
            custom_dir = Path(temp_dir) / "perm_test"
            storage = SecureKeyStorage(storage_dir=str(custom_dir))
            
            # Check directory permissions (should be 0o700 - owner only)
            dir_stat = custom_dir.stat()
            permissions = dir_stat.st_mode & 0o777
            assert permissions == 0o700


class TestKeyOperations:
    """Test cases for key storage operations"""
    
    @pytest.fixture
    def temp_storage(self):
        """Fixture providing temporary storage"""
        with tempfile.TemporaryDirectory() as temp_dir:
            storage = SecureKeyStorage(
                storage_dir=temp_dir,
                use_keyring=False  # Use file storage for testing
            )
            yield storage
    
    @pytest.fixture
    def test_key_pair(self):
        """Fixture providing test key pair"""
        return generate_key_pair()
    
    def test_store_and_retrieve_key(self, temp_storage, test_key_pair):
        """Test basic key storage and retrieval"""
        key_id = "test_key_001"
        passphrase = "test_passphrase_123"
        
        # Store the key
        metadata = temp_storage.store_key(key_id, test_key_pair, passphrase)
        
        assert metadata.key_id == key_id
        assert metadata.storage_type == "file"
        assert metadata.algorithm == "Ed25519"
        
        # Retrieve the key
        retrieved_key = temp_storage.retrieve_key(key_id, passphrase)
        
        assert retrieved_key is not None
        assert retrieved_key.private_key == test_key_pair.private_key
        assert retrieved_key.public_key == test_key_pair.public_key
    
    def test_store_invalid_key_id(self, temp_storage, test_key_pair):
        """Test storing with invalid key ID"""
        with pytest.raises(StorageError, match="Key ID must be a non-empty string"):
            temp_storage.store_key("", test_key_pair, "passphrase")
        
        with pytest.raises(StorageError, match="Key ID must be a non-empty string"):
            temp_storage.store_key(None, test_key_pair, "passphrase")
        
        with pytest.raises(StorageError, match="Key ID must be a non-empty string"):
            temp_storage.store_key(123, test_key_pair, "passphrase")
    
    def test_store_without_passphrase(self, temp_storage, test_key_pair):
        """Test storing without passphrase (should fail for file storage)"""
        with pytest.raises(StorageError, match="Passphrase required for encrypted file storage"):
            temp_storage.store_key("test_key", test_key_pair)
    
    def test_retrieve_nonexistent_key(self, temp_storage):
        """Test retrieving nonexistent key"""
        result = temp_storage.retrieve_key("nonexistent_key", "passphrase")
        assert result is None
    
    def test_retrieve_invalid_key_id(self, temp_storage):
        """Test retrieving with invalid key ID"""
        with pytest.raises(StorageError, match="Key ID must be a non-empty string"):
            temp_storage.retrieve_key("", "passphrase")
        
        with pytest.raises(StorageError, match="Key ID must be a non-empty string"):
            temp_storage.retrieve_key(None, "passphrase")
    
    def test_retrieve_without_passphrase(self, temp_storage, test_key_pair):
        """Test retrieving without passphrase when file exists"""
        key_id = "test_key_no_pass"
        passphrase = "test_passphrase"
        
        # Store the key first
        temp_storage.store_key(key_id, test_key_pair, passphrase)
        
        # Try to retrieve without passphrase
        with pytest.raises(StorageError, match="Passphrase required for encrypted file storage"):
            temp_storage.retrieve_key(key_id)
    
    def test_retrieve_wrong_passphrase(self, temp_storage, test_key_pair):
        """Test retrieving with wrong passphrase"""
        key_id = "test_key_wrong_pass"
        correct_passphrase = "correct_passphrase"
        wrong_passphrase = "wrong_passphrase"
        
        # Store the key
        temp_storage.store_key(key_id, test_key_pair, correct_passphrase)
        
        # Try to retrieve with wrong passphrase
        with pytest.raises(StorageError, match="Key decryption failed"):
            temp_storage.retrieve_key(key_id, wrong_passphrase)
    
    def test_delete_key(self, temp_storage, test_key_pair):
        """Test key deletion"""
        key_id = "test_key_delete"
        passphrase = "test_passphrase"
        
        # Store the key
        temp_storage.store_key(key_id, test_key_pair, passphrase)
        
        # Verify it exists
        assert temp_storage.retrieve_key(key_id, passphrase) is not None
        
        # Delete the key
        result = temp_storage.delete_key(key_id)
        assert result is True
        
        # Verify it's gone
        assert temp_storage.retrieve_key(key_id, passphrase) is None
    
    def test_delete_nonexistent_key(self, temp_storage):
        """Test deleting nonexistent key"""
        result = temp_storage.delete_key("nonexistent_key")
        assert result is False
    
    def test_delete_invalid_key_id(self, temp_storage):
        """Test deleting with invalid key ID"""
        with pytest.raises(StorageError, match="Key ID must be a non-empty string"):
            temp_storage.delete_key("")
    
    def test_list_keys(self, temp_storage, test_key_pair):
        """Test listing stored keys"""
        # Initially empty
        keys = temp_storage.list_keys()
        assert keys == []
        
        # Store some keys
        passphrase = "test_passphrase"
        key_ids = ["key_001", "key_002", "key_003"]
        
        for key_id in key_ids:
            temp_storage.store_key(key_id, test_key_pair, passphrase)
        
        # List keys
        stored_keys = temp_storage.list_keys()
        assert sorted(stored_keys) == sorted(key_ids)
        
        # Delete one key
        temp_storage.delete_key("key_002")
        
        # List again
        remaining_keys = temp_storage.list_keys()
        assert sorted(remaining_keys) == ["key_001", "key_003"]


class TestFileStorage:
    """Test cases for file-based storage"""
    
    @pytest.fixture
    def temp_storage(self):
        """Fixture providing temporary storage"""
        with tempfile.TemporaryDirectory() as temp_dir:
            storage = SecureKeyStorage(
                storage_dir=temp_dir,
                use_keyring=False
            )
            yield storage
    
    def test_file_permissions(self, temp_storage):
        """Test that key files have proper permissions"""
        if platform.system() == "Windows":
            pytest.skip("Permission tests not applicable on Windows")
        
        test_key = generate_key_pair()
        key_id = "perm_test_key"
        passphrase = "test_passphrase"
        
        # Store the key
        temp_storage.store_key(key_id, test_key, passphrase)
        
        # Check file permissions
        file_path = temp_storage._get_file_path(key_id)
        file_stat = file_path.stat()
        permissions = file_stat.st_mode & 0o777
        assert permissions == KEY_FILE_PERMISSIONS
    
    def test_file_content_structure(self, temp_storage):
        """Test the structure of encrypted key files"""
        test_key = generate_key_pair()
        key_id = "structure_test_key"
        passphrase = "test_passphrase"
        
        # Store the key
        temp_storage.store_key(key_id, test_key, passphrase)
        
        # Read and verify file structure
        file_path = temp_storage._get_file_path(key_id)
        with open(file_path, 'r') as f:
            file_data = json.load(f)
        
        # Check required fields
        assert 'key_id' in file_data
        assert 'storage_type' in file_data
        assert 'algorithm' in file_data
        assert 'created_at' in file_data
        assert 'encrypted_key' in file_data
        
        # Check encrypted key structure
        encrypted_key = file_data['encrypted_key']
        assert 'encrypted_data' in encrypted_key
        assert 'salt' in encrypted_key
        assert 'algorithm' in encrypted_key
        assert 'version' in encrypted_key
        
        assert file_data['key_id'] == key_id
        assert file_data['algorithm'] == 'Ed25519'
        assert encrypted_key['algorithm'] == 'Scrypt-Fernet'
    
    def test_corrupted_file_handling(self, temp_storage):
        """Test handling of corrupted key files"""
        key_id = "corrupted_test_key"
        file_path = temp_storage._get_file_path(key_id)
        
        # Create corrupted file
        with open(file_path, 'w') as f:
            f.write("corrupted data")
        
        # Try to retrieve
        with pytest.raises(StorageError, match="File retrieval failed"):
            temp_storage.retrieve_key(key_id, "passphrase")
    
    def test_file_storage_directory_creation_failure(self):
        """Test handling of storage directory creation failure"""
        # Try to create storage in a location that should fail
        if platform.system() != "Windows":
            invalid_path = "/root/invalid_permission_test"
        else:
            invalid_path = "C:\\Windows\\System32\\invalid_test"
        
        with pytest.raises(StorageError, match="Failed to create storage directory"):
            SecureKeyStorage(storage_dir=invalid_path)


class TestKeyringStorage:
    """Test cases for keyring-based storage"""
    
    @pytest.fixture
    def keyring_storage(self):
        """Fixture providing keyring storage"""
        with tempfile.TemporaryDirectory() as temp_dir:
            storage = SecureKeyStorage(
                storage_dir=temp_dir,
                use_keyring=True
            )
            yield storage
    
    @pytest.mark.skipif(not KEYRING_AVAILABLE_FOR_TESTS, reason="Keyring not available")
    def test_keyring_store_and_retrieve(self, keyring_storage):
        """Test keyring storage and retrieval"""
        test_key = generate_key_pair()
        key_id = "keyring_test_key"
        
        # Mock keyring operations
        with patch('datafold_sdk.crypto.storage.keyring') as mock_keyring:
            mock_keyring.set_password = MagicMock()
            mock_keyring.get_password = MagicMock(return_value=None)
            
            # Store key (should try keyring first, then fall back to file)
            metadata = keyring_storage.store_key(key_id, test_key, "fallback_passphrase")
            
            # Should have tried keyring
            mock_keyring.set_password.assert_called_once()
    
    def test_keyring_unavailable_fallback(self):
        """Test fallback to file storage when keyring unavailable"""
        with tempfile.TemporaryDirectory() as temp_dir:
            # Force keyring unavailable
            with patch('datafold_sdk.crypto.storage.KEYRING_AVAILABLE', False):
                storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=True)
                
                # Should fall back to file storage
                assert storage.use_keyring is False
    
    def test_keyring_error_fallback(self, keyring_storage):
        """Test fallback to file storage when keyring fails"""
        test_key = generate_key_pair()
        key_id = "keyring_error_test"
        passphrase = "fallback_passphrase"
        
        # Mock keyring to raise error
        with patch('datafold_sdk.crypto.storage.keyring') as mock_keyring:
            mock_keyring.set_password.side_effect = KeyringError("Keyring error")
            
            # Should fall back to file storage
            metadata = keyring_storage.store_key(key_id, test_key, passphrase)
            assert metadata.storage_type == "file"


class TestEncryption:
    """Test cases for encryption functionality"""
    
    @pytest.fixture
    def temp_storage(self):
        """Fixture providing temporary storage"""
        with tempfile.TemporaryDirectory() as temp_dir:
            storage = SecureKeyStorage(
                storage_dir=temp_dir,
                use_keyring=False
            )
            yield storage
    
    def test_key_derivation(self, temp_storage):
        """Test key derivation from passphrase"""
        passphrase = "test_passphrase"
        salt = secrets.token_bytes(SALT_LENGTH)
        
        # Derive key twice with same inputs
        key1 = temp_storage._derive_key_from_passphrase(passphrase, salt)
        key2 = temp_storage._derive_key_from_passphrase(passphrase, salt)
        
        # Should be identical
        assert key1 == key2
        assert len(key1) == 32  # 256-bit key
    
    def test_key_derivation_different_salt(self, temp_storage):
        """Test that different salts produce different keys"""
        passphrase = "test_passphrase"
        salt1 = secrets.token_bytes(SALT_LENGTH)
        salt2 = secrets.token_bytes(SALT_LENGTH)
        
        # Ensure different salts
        while salt1 == salt2:
            salt2 = secrets.token_bytes(SALT_LENGTH)
        
        key1 = temp_storage._derive_key_from_passphrase(passphrase, salt1)
        key2 = temp_storage._derive_key_from_passphrase(passphrase, salt2)
        
        # Should be different
        assert key1 != key2
    
    def test_key_derivation_different_passphrase(self, temp_storage):
        """Test that different passphrases produce different keys"""
        salt = secrets.token_bytes(SALT_LENGTH)
        passphrase1 = "passphrase_one"
        passphrase2 = "passphrase_two"
        
        key1 = temp_storage._derive_key_from_passphrase(passphrase1, salt)
        key2 = temp_storage._derive_key_from_passphrase(passphrase2, salt)
        
        # Should be different
        assert key1 != key2
    
    def test_encrypt_decrypt_roundtrip(self, temp_storage):
        """Test encryption and decryption roundtrip"""
        test_data = b"sensitive_key_data_12345678901234567890"
        passphrase = "encryption_test_passphrase"
        
        # Encrypt
        encrypted_payload = temp_storage._encrypt_key_data(test_data, passphrase)
        
        # Verify payload structure
        assert 'encrypted_data' in encrypted_payload
        assert 'salt' in encrypted_payload
        assert 'algorithm' in encrypted_payload
        assert 'version' in encrypted_payload
        
        # Decrypt
        decrypted_data = temp_storage._decrypt_key_data(encrypted_payload, passphrase)
        
        # Should match original
        assert decrypted_data == test_data
    
    def test_decrypt_wrong_passphrase(self, temp_storage):
        """Test decryption with wrong passphrase"""
        test_data = b"sensitive_data"
        correct_passphrase = "correct_passphrase"
        wrong_passphrase = "wrong_passphrase"
        
        # Encrypt with correct passphrase
        encrypted_payload = temp_storage._encrypt_key_data(test_data, correct_passphrase)
        
        # Try to decrypt with wrong passphrase
        with pytest.raises(StorageError, match="Key decryption failed"):
            temp_storage._decrypt_key_data(encrypted_payload, wrong_passphrase)


class TestStorageAvailability:
    """Test cases for storage availability checking"""
    
    def test_check_storage_availability(self):
        """Test storage availability check"""
        with tempfile.TemporaryDirectory() as temp_dir:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            
            availability = storage.check_storage_availability()
            
            assert isinstance(availability, dict)
            assert 'keyring_available' in availability
            assert 'file_storage_available' in availability
            assert 'cryptography_available' in availability
            assert 'storage_dir' in availability
            assert 'storage_dir_exists' in availability
            assert 'platform' in availability
            
            # File storage should always be available
            assert availability['file_storage_available'] is True
            assert availability['storage_dir_exists'] is True
    
    @patch('datafold_sdk.crypto.storage.CRYPTOGRAPHY_AVAILABLE', False)
    def test_check_storage_availability_no_crypto(self):
        """Test storage availability when cryptography unavailable"""
        with tempfile.TemporaryDirectory() as temp_dir:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            
            availability = storage.check_storage_availability()
            assert availability['cryptography_available'] is False


class TestDefaultStorage:
    """Test cases for default storage function"""
    
    def test_get_default_storage(self):
        """Test getting default storage instance"""
        storage = get_default_storage()
        
        assert isinstance(storage, SecureKeyStorage)
        assert storage.storage_dir.exists()


class TestEdgeCases:
    """Test cases for edge cases and error conditions"""
    
    @pytest.fixture
    def temp_storage(self):
        """Fixture providing temporary storage"""
        with tempfile.TemporaryDirectory() as temp_dir:
            storage = SecureKeyStorage(
                storage_dir=temp_dir,
                use_keyring=False
            )
            yield storage
    
    def test_invalid_stored_key_length(self, temp_storage):
        """Test handling of stored key with invalid length"""
        key_id = "invalid_length_key"
        passphrase = "test_passphrase"
        
        # Create file with invalid key data length
        file_path = temp_storage._get_file_path(key_id)
        invalid_data = b"invalid_length_data"  # Wrong length
        encrypted_payload = temp_storage._encrypt_key_data(invalid_data, passphrase)
        
        file_data = {
            'key_id': key_id,
            'storage_type': 'file',
            'algorithm': 'Ed25519',
            'created_at': temp_storage._get_timestamp(),
            'encrypted_key': encrypted_payload
        }
        
        with open(file_path, 'w') as f:
            json.dump(file_data, f)
        
        # Try to retrieve
        with pytest.raises(StorageError, match="Stored key data has invalid length"):
            temp_storage.retrieve_key(key_id, passphrase)
    
    def test_key_id_sanitization(self, temp_storage):
        """Test that key IDs are properly sanitized for filesystem"""
        test_key = generate_key_pair()
        unsafe_key_id = "unsafe/key\\id:with*special?chars"
        passphrase = "test_passphrase"
        
        # Should work despite unsafe characters
        metadata = temp_storage.store_key(unsafe_key_id, test_key, passphrase)
        assert metadata.key_id == unsafe_key_id
        
        # Should be able to retrieve
        retrieved_key = temp_storage.retrieve_key(unsafe_key_id, passphrase)
        assert retrieved_key is not None
    
    @patch('datafold_sdk.crypto.storage.CRYPTOGRAPHY_AVAILABLE', False)
    def test_encryption_without_cryptography(self):
        """Test that encryption fails gracefully without cryptography"""
        with tempfile.TemporaryDirectory() as temp_dir:
            storage = SecureKeyStorage(storage_dir=temp_dir, use_keyring=False)
            test_key = Ed25519KeyPair(
                private_key=secrets.token_bytes(32),
                public_key=secrets.token_bytes(32)
            )
            
            with pytest.raises(UnsupportedPlatformError, match="Cryptography package required"):
                storage.store_key("test_key", test_key, "passphrase")


class TestSecurityFeatures:
    """Test cases for security features"""
    
    @pytest.fixture
    def temp_storage(self):
        """Fixture providing temporary storage"""
        with tempfile.TemporaryDirectory() as temp_dir:
            storage = SecureKeyStorage(
                storage_dir=temp_dir,
                use_keyring=False
            )
            yield storage
    
    def test_salt_randomness(self, temp_storage):
        """Test that salts are properly randomized"""
        passphrase = "test_passphrase"
        data = b"test_data"
        
        # Generate multiple encrypted payloads
        payloads = []
        for _ in range(10):
            payload = temp_storage._encrypt_key_data(data, passphrase)
            payloads.append(payload['salt'])
        
        # All salts should be different
        assert len(set(payloads)) == len(payloads)
    
    def test_encrypted_data_randomness(self, temp_storage):
        """Test that encrypted data is properly randomized"""
        passphrase = "test_passphrase"
        data = b"test_data"
        
        # Generate multiple encrypted payloads
        encrypted_data_list = []
        for _ in range(10):
            payload = temp_storage._encrypt_key_data(data, passphrase)
            encrypted_data_list.append(payload['encrypted_data'])
        
        # All encrypted data should be different (due to random salts)
        assert len(set(encrypted_data_list)) == len(encrypted_data_list)
    
    def test_key_isolation(self, temp_storage):
        """Test that keys are properly isolated"""
        test_key1 = generate_key_pair()
        test_key2 = generate_key_pair()
        passphrase1 = "passphrase_one"
        passphrase2 = "passphrase_two"
        
        # Store keys with different IDs and passphrases
        temp_storage.store_key("key_1", test_key1, passphrase1)
        temp_storage.store_key("key_2", test_key2, passphrase2)
        
        # Should not be able to retrieve key_1 with key_2's passphrase
        with pytest.raises(StorageError):
            temp_storage.retrieve_key("key_1", passphrase2)
        
        # Should not be able to retrieve key_2 with key_1's passphrase
        with pytest.raises(StorageError):
            temp_storage.retrieve_key("key_2", passphrase1)
        
        # But should work with correct passphrases
        retrieved_key1 = temp_storage.retrieve_key("key_1", passphrase1)
        retrieved_key2 = temp_storage.retrieve_key("key_2", passphrase2)
        
        assert retrieved_key1.private_key == test_key1.private_key
        assert retrieved_key2.private_key == test_key2.private_key


if __name__ == '__main__':
    pytest.main([__file__])