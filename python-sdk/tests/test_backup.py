"""
Tests for key backup and restore functionality
"""

import json
import os
import tempfile
import pytest
from pathlib import Path
from unittest.mock import patch, MagicMock

from datafold_sdk.crypto.backup import (
    KeyBackupManager,
    BackupMetadata,
    get_default_backup_manager,
    export_key_to_file,
    import_key_from_file,
    BACKUP_VERSION,
    SUPPORTED_EXPORT_FORMATS,
    SUPPORTED_KDF_ALGORITHMS,
    SUPPORTED_ENCRYPTION_ALGORITHMS,
)
from datafold_sdk.crypto.ed25519 import generate_key_pair, Ed25519KeyPair
from datafold_sdk.exceptions import BackupError, ValidationError, UnsupportedPlatformError


class TestKeyBackupManager:
    """Test the KeyBackupManager class"""
    
    def setup_method(self):
        """Set up test fixtures"""
        self.manager = KeyBackupManager()
        self.test_key_pair = generate_key_pair()
        self.test_passphrase = "TestPassphrase123!"
        self.test_key_id = "test-key-2025"
    
    def test_manager_initialization(self):
        """Test manager initialization with default preferences"""
        manager = KeyBackupManager()
        assert manager.preferred_kdf in SUPPORTED_KDF_ALGORITHMS
        assert manager.preferred_encryption in SUPPORTED_ENCRYPTION_ALGORITHMS
    
    def test_manager_initialization_with_preferences(self):
        """Test manager initialization with custom preferences"""
        manager = KeyBackupManager(
            preferred_kdf='scrypt',
            preferred_encryption='aes-gcm'
        )
        assert manager.preferred_kdf == 'scrypt'
        assert manager.preferred_encryption == 'aes-gcm'
    
    def test_manager_initialization_invalid_preferences(self):
        """Test manager initialization with invalid preferences falls back to supported ones"""
        manager = KeyBackupManager(
            preferred_kdf='invalid_kdf',
            preferred_encryption='invalid_encryption'
        )
        assert manager.preferred_kdf in SUPPORTED_KDF_ALGORITHMS
        assert manager.preferred_encryption in SUPPORTED_ENCRYPTION_ALGORITHMS
    
    def test_passphrase_validation_valid(self):
        """Test passphrase validation with valid passphrases"""
        valid_passphrases = [
            "StrongPassword123!",
            "VeryLongPassphraseWithMixedCase",
            "Short1A!",
            "12345678Ab"
        ]
        
        for passphrase in valid_passphrases:
            # Should not raise an exception
            self.manager._validate_passphrase(passphrase)
    
    def test_passphrase_validation_invalid(self):
        """Test passphrase validation with invalid passphrases"""
        invalid_passphrases = [
            "",           # Empty
            "short",      # Too short and weak
            "1234567",    # Too short and weak
            "weakpass",   # Weak
            123,          # Not a string
            None,         # None
        ]
        
        for passphrase in invalid_passphrases:
            with pytest.raises(BackupError):
                self.manager._validate_passphrase(passphrase)
    
    def test_key_data_preparation(self):
        """Test key data preparation combines private and public keys"""
        key_data = self.manager._prepare_key_data(self.test_key_pair)
        
        assert len(key_data) == 64  # 32 private + 32 public
        assert key_data[:32] == self.test_key_pair.private_key
        assert key_data[32:] == self.test_key_pair.public_key
    
    def test_key_pair_extraction(self):
        """Test key pair extraction from combined data"""
        # Prepare test data
        key_data = self.manager._prepare_key_data(self.test_key_pair)
        
        # Extract key pair
        extracted_pair = self.manager._extract_key_pair(key_data)
        
        assert extracted_pair.private_key == self.test_key_pair.private_key
        assert extracted_pair.public_key == self.test_key_pair.public_key
    
    def test_key_pair_extraction_invalid_length(self):
        """Test key pair extraction with invalid data length"""
        invalid_data = b"invalid_length_data"
        
        with pytest.raises(BackupError) as exc_info:
            self.manager._extract_key_pair(invalid_data)
        
        assert "Invalid key data length" in str(exc_info.value)
    
    def test_export_key_json_format(self):
        """Test key export in JSON format"""
        backup_data = self.manager.export_key(
            self.test_key_pair,
            self.test_passphrase,
            self.test_key_id,
            export_format='json'
        )
        
        assert isinstance(backup_data, str)
        
        # Parse and validate JSON structure
        backup_dict = json.loads(backup_data)
        
        required_fields = ['version', 'key_id', 'algorithm', 'kdf', 'encryption',
                          'salt', 'nonce', 'ciphertext', 'created']
        
        for field in required_fields:
            assert field in backup_dict
        
        assert backup_dict['version'] == BACKUP_VERSION
        assert backup_dict['key_id'] == self.test_key_id
        assert backup_dict['algorithm'] == 'Ed25519'
        assert backup_dict['kdf'] in SUPPORTED_KDF_ALGORITHMS
        assert backup_dict['encryption'] in SUPPORTED_ENCRYPTION_ALGORITHMS
    
    def test_export_key_binary_format(self):
        """Test key export in binary format"""
        backup_data = self.manager.export_key(
            self.test_key_pair,
            self.test_passphrase,
            self.test_key_id,
            export_format='binary'
        )
        
        assert isinstance(backup_data, bytes)
        
        # Should be able to parse as JSON
        backup_dict = json.loads(backup_data.decode('utf-8'))
        assert backup_dict['key_id'] == self.test_key_id
    
    def test_export_key_invalid_inputs(self):
        """Test key export with invalid inputs"""
        # Invalid key pair type
        with pytest.raises(ValidationError):
            self.manager.export_key("invalid", self.test_passphrase, self.test_key_id)
        
        # Invalid key ID
        with pytest.raises(ValidationError):
            self.manager.export_key(self.test_key_pair, self.test_passphrase, "")
        
        # Invalid export format
        with pytest.raises(ValidationError):
            self.manager.export_key(
                self.test_key_pair, self.test_passphrase, self.test_key_id,
                export_format='invalid'
            )
        
        # Invalid passphrase
        with pytest.raises(BackupError):
            self.manager.export_key(self.test_key_pair, "weak", self.test_key_id)
    
    def test_import_export_roundtrip_json(self):
        """Test export and import roundtrip with JSON format"""
        # Export key
        backup_data = self.manager.export_key(
            self.test_key_pair,
            self.test_passphrase,
            self.test_key_id,
            export_format='json'
        )
        
        # Import key
        imported_pair, metadata = self.manager.import_key(
            backup_data,
            self.test_passphrase,
            verify_integrity=True
        )
        
        # Verify key pair matches
        assert imported_pair.private_key == self.test_key_pair.private_key
        assert imported_pair.public_key == self.test_key_pair.public_key
        
        # Verify metadata
        assert metadata.version == BACKUP_VERSION
        assert metadata.key_id == self.test_key_id
        assert metadata.algorithm == 'Ed25519'
        assert metadata.format == 'json'
    
    def test_import_export_roundtrip_binary(self):
        """Test export and import roundtrip with binary format"""
        # Export key
        backup_data = self.manager.export_key(
            self.test_key_pair,
            self.test_passphrase,
            self.test_key_id,
            export_format='binary'
        )
        
        # Import key
        imported_pair, metadata = self.manager.import_key(
            backup_data,
            self.test_passphrase,
            verify_integrity=True
        )
        
        # Verify key pair matches
        assert imported_pair.private_key == self.test_key_pair.private_key
        assert imported_pair.public_key == self.test_key_pair.public_key
        
        # Verify metadata
        assert metadata.format == 'binary'
    
    def test_import_key_wrong_passphrase(self):
        """Test import with wrong passphrase"""
        # Export key
        backup_data = self.manager.export_key(
            self.test_key_pair,
            self.test_passphrase,
            self.test_key_id
        )
        
        # Try to import with wrong passphrase
        with pytest.raises(BackupError) as exc_info:
            self.manager.import_key(backup_data, "wrong_passphrase")
        
        assert "Decryption failed" in str(exc_info.value)
    
    def test_import_key_corrupted_data(self):
        """Test import with corrupted backup data"""
        # Export key
        backup_data = self.manager.export_key(
            self.test_key_pair,
            self.test_passphrase,
            self.test_key_id
        )
        
        # Corrupt the data
        corrupted_data = backup_data.replace('a', 'z', 1)
        
        # Try to import corrupted data
        with pytest.raises((BackupError, ValidationError)):
            self.manager.import_key(corrupted_data, self.test_passphrase)
    
    def test_import_key_invalid_json(self):
        """Test import with invalid JSON"""
        invalid_json = "{ invalid json"
        
        with pytest.raises(ValidationError) as exc_info:
            self.manager.import_key(invalid_json, self.test_passphrase)
        
        assert "Invalid JSON" in str(exc_info.value)
    
    def test_import_key_missing_fields(self):
        """Test import with missing required fields"""
        incomplete_backup = {
            'version': BACKUP_VERSION,
            'key_id': self.test_key_id,
            # Missing other required fields
        }
        
        backup_json = json.dumps(incomplete_backup)
        
        with pytest.raises(ValidationError) as exc_info:
            self.manager.import_key(backup_json, self.test_passphrase)
        
        assert "Missing required field" in str(exc_info.value)
    
    def test_import_key_unsupported_version(self):
        """Test import with unsupported backup version"""
        backup_data = self.manager.export_key(
            self.test_key_pair,
            self.test_passphrase,
            self.test_key_id
        )
        
        # Modify version
        backup_dict = json.loads(backup_data)
        backup_dict['version'] = 999
        modified_backup = json.dumps(backup_dict)
        
        with pytest.raises(ValidationError) as exc_info:
            self.manager.import_key(modified_backup, self.test_passphrase)
        
        assert "Unsupported backup version" in str(exc_info.value)
    
    def test_import_key_unsupported_algorithm(self):
        """Test import with unsupported key algorithm"""
        backup_data = self.manager.export_key(
            self.test_key_pair,
            self.test_passphrase,
            self.test_key_id
        )
        
        # Modify algorithm
        backup_dict = json.loads(backup_data)
        backup_dict['algorithm'] = 'RSA'
        modified_backup = json.dumps(backup_dict)
        
        with pytest.raises(ValidationError) as exc_info:
            self.manager.import_key(modified_backup, self.test_passphrase)
        
        assert "Unsupported key algorithm" in str(exc_info.value)
    
    def test_export_import_different_algorithms(self):
        """Test export/import with different KDF and encryption algorithms"""
        algorithms_to_test = []
        
        # Test available algorithms
        for kdf in SUPPORTED_KDF_ALGORITHMS:
            for enc in SUPPORTED_ENCRYPTION_ALGORITHMS:
                algorithms_to_test.append((kdf, enc))
        
        for kdf_algo, enc_algo in algorithms_to_test:
            try:
                # Export with specific algorithms
                backup_data = self.manager.export_key(
                    self.test_key_pair,
                    self.test_passphrase,
                    self.test_key_id,
                    kdf_algorithm=kdf_algo,
                    encryption_algorithm=enc_algo
                )
                
                # Import and verify
                imported_pair, metadata = self.manager.import_key(
                    backup_data,
                    self.test_passphrase
                )
                
                assert imported_pair.private_key == self.test_key_pair.private_key
                assert imported_pair.public_key == self.test_key_pair.public_key
                assert metadata.kdf == kdf_algo
                assert metadata.encryption == enc_algo
                
            except (BackupError, UnsupportedPlatformError):
                # Skip if algorithm not available on this platform
                continue
    
    def test_integrity_verification(self):
        """Test key integrity verification"""
        # Export and import with verification enabled
        backup_data = self.manager.export_key(
            self.test_key_pair,
            self.test_passphrase,
            self.test_key_id
        )
        
        # Import with verification (should succeed)
        imported_pair, _ = self.manager.import_key(
            backup_data,
            self.test_passphrase,
            verify_integrity=True
        )
        
        # Import without verification (should also succeed)
        imported_pair2, _ = self.manager.import_key(
            backup_data,
            self.test_passphrase,
            verify_integrity=False
        )
        
        assert imported_pair.private_key == imported_pair2.private_key
        assert imported_pair.public_key == imported_pair2.public_key
    
    def test_check_backup_support(self):
        """Test backup support checking"""
        support = self.manager.check_backup_support()
        
        assert isinstance(support, dict)
        assert 'cryptography_available' in support
        assert 'supported_kdf_algorithms' in support
        assert 'supported_encryption_algorithms' in support
        assert 'supported_export_formats' in support
        assert 'current_preferences' in support
        
        assert isinstance(support['supported_kdf_algorithms'], list)
        assert isinstance(support['supported_encryption_algorithms'], list)
        assert support['supported_export_formats'] == SUPPORTED_EXPORT_FORMATS


class TestBackupFileOperations:
    """Test file-based backup operations"""
    
    def setup_method(self):
        """Set up test fixtures"""
        self.test_key_pair = generate_key_pair()
        self.test_passphrase = "FileTestPassphrase123!"
        self.test_key_id = "file-test-key"
        self.temp_dir = tempfile.mkdtemp()
    
    def teardown_method(self):
        """Clean up test fixtures"""
        # Clean up temp directory
        import shutil
        shutil.rmtree(self.temp_dir, ignore_errors=True)
    
    def test_export_import_file_json(self):
        """Test export and import with JSON file"""
        file_path = os.path.join(self.temp_dir, "test_key.json")
        
        # Export to file
        metadata = export_key_to_file(
            self.test_key_pair,
            self.test_passphrase,
            self.test_key_id,
            file_path,
            export_format='json'
        )
        
        assert isinstance(metadata, BackupMetadata)
        assert metadata.key_id == self.test_key_id
        assert metadata.format == 'json'
        assert os.path.exists(file_path)
        
        # Check file permissions on Unix-like systems
        if os.name != 'nt':  # Not Windows
            file_stat = os.stat(file_path)
            permissions = oct(file_stat.st_mode)[-3:]
            assert permissions == '600'  # Owner read/write only
        
        # Import from file
        imported_pair, import_metadata = import_key_from_file(
            file_path,
            self.test_passphrase
        )
        
        assert imported_pair.private_key == self.test_key_pair.private_key
        assert imported_pair.public_key == self.test_key_pair.public_key
        assert import_metadata.key_id == self.test_key_id
    
    def test_export_import_file_binary(self):
        """Test export and import with binary file"""
        file_path = os.path.join(self.temp_dir, "test_key.bin")
        
        # Export to file
        metadata = export_key_to_file(
            self.test_key_pair,
            self.test_passphrase,
            self.test_key_id,
            file_path,
            export_format='binary'
        )
        
        assert metadata.format == 'binary'
        assert os.path.exists(file_path)
        
        # Import from file
        imported_pair, import_metadata = import_key_from_file(
            file_path,
            self.test_passphrase
        )
        
        assert imported_pair.private_key == self.test_key_pair.private_key
        assert imported_pair.public_key == self.test_key_pair.public_key
    
    def test_import_file_not_found(self):
        """Test import from non-existent file"""
        non_existent_file = os.path.join(self.temp_dir, "non_existent.json")
        
        with pytest.raises(BackupError) as exc_info:
            import_key_from_file(non_existent_file, self.test_passphrase)
        
        assert "Backup file not found" in str(exc_info.value)
    
    def test_export_file_cleanup_on_error(self):
        """Test that partial files are cleaned up on export error"""
        file_path = os.path.join(self.temp_dir, "test_cleanup.json")
        
        # Create an invalid key pair that will cause export to fail
        invalid_pair = Ed25519KeyPair(b"x" * 32, b"y" * 32)
        
        try:
            with patch('datafold_sdk.crypto.backup.KeyBackupManager.export_key', 
                      side_effect=BackupError("Test error", "TEST_ERROR")):
                export_key_to_file(
                    invalid_pair,
                    self.test_passphrase,
                    self.test_key_id,
                    file_path
                )
        except BackupError:
            pass
        
        # File should not exist after failed export
        assert not os.path.exists(file_path)


class TestConvenienceFunctions:
    """Test convenience functions"""
    
    def test_get_default_backup_manager(self):
        """Test default backup manager creation"""
        manager = get_default_backup_manager()
        
        assert isinstance(manager, KeyBackupManager)
        assert manager.preferred_kdf in SUPPORTED_KDF_ALGORITHMS
        assert manager.preferred_encryption in SUPPORTED_ENCRYPTION_ALGORITHMS


class TestErrorConditions:
    """Test various error conditions and edge cases"""
    
    def test_cryptography_unavailable(self):
        """Test behavior when cryptography is unavailable"""
        with patch('datafold_sdk.crypto.backup.CRYPTOGRAPHY_AVAILABLE', False):
            with pytest.raises(UnsupportedPlatformError) as exc_info:
                KeyBackupManager()
            
            assert "Cryptography package required" in str(exc_info.value)
    
    def test_tampered_backup_detection(self):
        """Test detection of tampered backup data"""
        manager = KeyBackupManager()
        key_pair = generate_key_pair()
        passphrase = "TestPassphrase123!"
        key_id = "tamper-test"
        
        # Export key
        backup_data = manager.export_key(key_pair, passphrase, key_id)
        
        # Parse and tamper with ciphertext
        backup_dict = json.loads(backup_data)
        original_ciphertext = backup_dict['ciphertext']
        
        # Flip a bit in the ciphertext
        tampered_bytes = base64.b64decode(original_ciphertext)
        tampered_bytes = bytes([tampered_bytes[0] ^ 1]) + tampered_bytes[1:]
        backup_dict['ciphertext'] = base64.b64encode(tampered_bytes).decode('ascii')
        
        tampered_backup = json.dumps(backup_dict)
        
        # Import should fail due to authentication tag mismatch
        with pytest.raises(BackupError) as exc_info:
            manager.import_key(tampered_backup, passphrase)
        
        assert "Decryption failed" in str(exc_info.value)


if __name__ == '__main__':
    pytest.main([__file__])