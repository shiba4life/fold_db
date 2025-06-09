"""
Unit tests for key derivation functionality
"""

import pytest
import secrets
import json
from unittest.mock import patch, MagicMock

from datafold_sdk.crypto.derivation import (
    DerivationParameters,
    derive_key_hkdf,
    derive_key_pbkdf2,
    derive_key_scrypt,
    derive_ed25519_key_pair,
    verify_derivation,
    export_derivation_parameters,
    import_derivation_parameters,
    check_derivation_support,
    _get_hash_algorithm,
    _generate_salt,
    _generate_info,
    DEFAULT_SALT_LENGTH,
    DEFAULT_INFO_LENGTH,
    PBKDF2_ITERATIONS,
    SCRYPT_N,
    SCRYPT_R,
    SCRYPT_P,
)
from datafold_sdk.crypto.ed25519 import Ed25519KeyPair, ED25519_PRIVATE_KEY_LENGTH
from datafold_sdk.exceptions import (
    KeyDerivationError,
    UnsupportedPlatformError,
)

# Import cryptography for integration tests
try:
    from cryptography.hazmat.primitives import hashes
    from cryptography.hazmat.primitives.kdf.hkdf import HKDF
    from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
    from cryptography.hazmat.primitives.kdf.scrypt import Scrypt
    CRYPTOGRAPHY_AVAILABLE_FOR_TESTS = True
except ImportError:
    CRYPTOGRAPHY_AVAILABLE_FOR_TESTS = False


class TestDerivationParameters:
    """Test cases for DerivationParameters dataclass"""
    
    def test_derivation_parameters_creation(self):
        """Test creating DerivationParameters with all fields"""
        salt = b'test_salt'
        info = b'test_info'
        
        params = DerivationParameters(
            algorithm='HKDF',
            salt=salt,
            info=info,
            iterations=100000,
            length=32,
            hash_algorithm='SHA256'
        )
        
        assert params.algorithm == 'HKDF'
        assert params.salt == salt
        assert params.info == info
        assert params.iterations == 100000
        assert params.length == 32
        assert params.hash_algorithm == 'SHA256'
    
    def test_derivation_parameters_defaults(self):
        """Test DerivationParameters with default values"""
        salt = b'test_salt'
        
        params = DerivationParameters(
            algorithm='PBKDF2',
            salt=salt
        )
        
        assert params.algorithm == 'PBKDF2'
        assert params.salt == salt
        assert params.info is None
        assert params.iterations is None
        assert params.length == ED25519_PRIVATE_KEY_LENGTH
        assert params.hash_algorithm == 'SHA256'


class TestDerivationSupport:
    """Test cases for derivation support checking"""
    
    def test_check_derivation_support_success(self):
        """Test derivation support check when cryptography is available"""
        support = check_derivation_support()
        
        assert isinstance(support, dict)
        assert 'cryptography_available' in support
        assert 'hkdf_supported' in support
        assert 'pbkdf2_supported' in support
        assert 'scrypt_supported' in support
        assert 'supported_hashes' in support
        
        if CRYPTOGRAPHY_AVAILABLE_FOR_TESTS:
            assert support['cryptography_available'] is True
            assert support['hkdf_supported'] is True
            assert support['pbkdf2_supported'] is True
            assert support['scrypt_supported'] is True
            assert 'SHA256' in support['supported_hashes']
    
    @patch('datafold_sdk.crypto.derivation.CRYPTOGRAPHY_AVAILABLE', False)
    def test_check_derivation_support_no_cryptography(self):
        """Test derivation support when cryptography is not available"""
        support = check_derivation_support()
        
        assert support['cryptography_available'] is False
        assert support['hkdf_supported'] is False
        assert support['pbkdf2_supported'] is False
        assert support['scrypt_supported'] is False
        assert support['supported_hashes'] == []


class TestHashAlgorithms:
    """Test cases for hash algorithm utilities"""
    
    def test_get_hash_algorithm_valid(self):
        """Test getting valid hash algorithms"""
        if not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS:
            pytest.skip("Cryptography not available")
        
        # Test supported algorithms
        for hash_name in ['SHA256', 'SHA384', 'SHA512', 'SHA3_256']:
            hash_obj = _get_hash_algorithm(hash_name)
            assert hash_obj is not None
    
    def test_get_hash_algorithm_invalid(self):
        """Test getting invalid hash algorithm"""
        if not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS:
            pytest.skip("Cryptography not available")
        
        with pytest.raises(KeyDerivationError, match="Unsupported hash algorithm"):
            _get_hash_algorithm('INVALID_HASH')
    
    @patch('datafold_sdk.crypto.derivation.CRYPTOGRAPHY_AVAILABLE', False)
    def test_get_hash_algorithm_no_cryptography(self):
        """Test hash algorithm access without cryptography"""
        with pytest.raises(UnsupportedPlatformError, match="Cryptography package required"):
            _get_hash_algorithm('SHA256')


class TestUtilityFunctions:
    """Test cases for utility functions"""
    
    def test_generate_salt(self):
        """Test salt generation"""
        salt = _generate_salt()
        
        assert isinstance(salt, bytes)
        assert len(salt) == DEFAULT_SALT_LENGTH
        
        # Generate another salt and ensure they're different
        salt2 = _generate_salt()
        assert salt != salt2
    
    def test_generate_salt_custom_length(self):
        """Test salt generation with custom length"""
        length = 16
        salt = _generate_salt(length)
        
        assert isinstance(salt, bytes)
        assert len(salt) == length
    
    def test_generate_info(self):
        """Test info generation"""
        context = "test_context"
        info = _generate_info(context)
        
        assert isinstance(info, bytes)
        assert len(info) == DEFAULT_INFO_LENGTH
        assert info.startswith(context.encode('utf-8'))
    
    def test_generate_info_long_context(self):
        """Test info generation with long context"""
        context = "a" * (DEFAULT_INFO_LENGTH + 10)
        info = _generate_info(context)
        
        assert isinstance(info, bytes)
        assert len(info) == DEFAULT_INFO_LENGTH
        assert info == context.encode('utf-8')[:DEFAULT_INFO_LENGTH]


class TestHKDFDerivation:
    """Test cases for HKDF key derivation"""
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_derive_key_hkdf_success(self):
        """Test successful HKDF key derivation"""
        master_key = secrets.token_bytes(32)
        
        derived_key, params = derive_key_hkdf(master_key)
        
        assert isinstance(derived_key, bytes)
        assert len(derived_key) == ED25519_PRIVATE_KEY_LENGTH
        assert isinstance(params, DerivationParameters)
        assert params.algorithm == 'HKDF'
        assert len(params.salt) == DEFAULT_SALT_LENGTH
        assert params.info is not None
        assert params.length == ED25519_PRIVATE_KEY_LENGTH
        assert params.hash_algorithm == 'SHA256'
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_derive_key_hkdf_with_custom_params(self):
        """Test HKDF with custom parameters"""
        master_key = secrets.token_bytes(32)
        salt = secrets.token_bytes(16)
        info = b"custom_info"
        length = 64
        
        derived_key, params = derive_key_hkdf(
            master_key=master_key,
            salt=salt,
            info=info,
            length=length,
            hash_algorithm='SHA384'
        )
        
        assert len(derived_key) == length
        assert params.salt == salt
        assert params.info == info
        assert params.length == length
        assert params.hash_algorithm == 'SHA384'
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_derive_key_hkdf_deterministic(self):
        """Test that HKDF is deterministic with same inputs"""
        master_key = secrets.token_bytes(32)
        salt = secrets.token_bytes(16)
        info = b"test_info"
        
        derived_key1, _ = derive_key_hkdf(master_key, salt=salt, info=info)
        derived_key2, _ = derive_key_hkdf(master_key, salt=salt, info=info)
        
        assert derived_key1 == derived_key2
    
    def test_derive_key_hkdf_invalid_master_key(self):
        """Test HKDF with invalid master key"""
        if not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS:
            pytest.skip("Cryptography not available")
        
        with pytest.raises(KeyDerivationError, match="Master key must be non-empty bytes"):
            derive_key_hkdf(b"")
        
        with pytest.raises(KeyDerivationError, match="Master key must be non-empty bytes"):
            derive_key_hkdf("not_bytes")
    
    def test_derive_key_hkdf_invalid_length(self):
        """Test HKDF with invalid length"""
        if not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS:
            pytest.skip("Cryptography not available")
        
        master_key = secrets.token_bytes(32)
        
        with pytest.raises(KeyDerivationError, match="Invalid key length"):
            derive_key_hkdf(master_key, length=0)
        
        with pytest.raises(KeyDerivationError, match="Invalid key length"):
            derive_key_hkdf(master_key, length=256 * 32)  # Too large
    
    @patch('datafold_sdk.crypto.derivation.CRYPTOGRAPHY_AVAILABLE', False)
    def test_derive_key_hkdf_no_cryptography(self):
        """Test HKDF without cryptography package"""
        with pytest.raises(UnsupportedPlatformError, match="Cryptography package required"):
            derive_key_hkdf(b"test_key")


class TestPBKDF2Derivation:
    """Test cases for PBKDF2 key derivation"""
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_derive_key_pbkdf2_success(self):
        """Test successful PBKDF2 key derivation"""
        password = "test_password"
        
        derived_key, params = derive_key_pbkdf2(password)
        
        assert isinstance(derived_key, bytes)
        assert len(derived_key) == ED25519_PRIVATE_KEY_LENGTH
        assert isinstance(params, DerivationParameters)
        assert params.algorithm == 'PBKDF2'
        assert len(params.salt) == DEFAULT_SALT_LENGTH
        assert params.iterations == PBKDF2_ITERATIONS
        assert params.length == ED25519_PRIVATE_KEY_LENGTH
        assert params.hash_algorithm == 'SHA256'
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_derive_key_pbkdf2_bytes_password(self):
        """Test PBKDF2 with bytes password"""
        password = b"test_password"
        
        derived_key, params = derive_key_pbkdf2(password)
        
        assert isinstance(derived_key, bytes)
        assert len(derived_key) == ED25519_PRIVATE_KEY_LENGTH
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_derive_key_pbkdf2_custom_params(self):
        """Test PBKDF2 with custom parameters"""
        password = "test_password"
        salt = secrets.token_bytes(16)
        iterations = 50000
        length = 64
        
        derived_key, params = derive_key_pbkdf2(
            password=password,
            salt=salt,
            iterations=iterations,
            length=length,
            hash_algorithm='SHA512'
        )
        
        assert len(derived_key) == length
        assert params.salt == salt
        assert params.iterations == iterations
        assert params.length == length
        assert params.hash_algorithm == 'SHA512'
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_derive_key_pbkdf2_deterministic(self):
        """Test that PBKDF2 is deterministic with same inputs"""
        password = "test_password"
        salt = secrets.token_bytes(16)
        iterations = 10000
        
        derived_key1, _ = derive_key_pbkdf2(password, salt=salt, iterations=iterations)
        derived_key2, _ = derive_key_pbkdf2(password, salt=salt, iterations=iterations)
        
        assert derived_key1 == derived_key2
    
    def test_derive_key_pbkdf2_invalid_password(self):
        """Test PBKDF2 with invalid password"""
        if not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS:
            pytest.skip("Cryptography not available")
        
        with pytest.raises(KeyDerivationError, match="Password cannot be empty"):
            derive_key_pbkdf2("")
        
        with pytest.raises(KeyDerivationError, match="Password must be string or bytes"):
            derive_key_pbkdf2(123)
    
    def test_derive_key_pbkdf2_invalid_iterations(self):
        """Test PBKDF2 with invalid iterations"""
        if not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS:
            pytest.skip("Cryptography not available")
        
        with pytest.raises(KeyDerivationError, match="Iterations must be at least 1000"):
            derive_key_pbkdf2("password", iterations=500)


class TestScryptDerivation:
    """Test cases for Scrypt key derivation"""
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_derive_key_scrypt_success(self):
        """Test successful Scrypt key derivation"""
        password = "test_password"
        
        derived_key, params = derive_key_scrypt(password)
        
        assert isinstance(derived_key, bytes)
        assert len(derived_key) == ED25519_PRIVATE_KEY_LENGTH
        assert isinstance(params, DerivationParameters)
        assert params.algorithm == 'Scrypt'
        assert len(params.salt) == DEFAULT_SALT_LENGTH
        assert params.length == ED25519_PRIVATE_KEY_LENGTH
        
        # Check Scrypt parameters are stored in info field
        assert params.info is not None
        scrypt_params = json.loads(params.info.decode('utf-8'))
        assert scrypt_params['n'] == SCRYPT_N
        assert scrypt_params['r'] == SCRYPT_R
        assert scrypt_params['p'] == SCRYPT_P
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_derive_key_scrypt_custom_params(self):
        """Test Scrypt with custom parameters"""
        password = "test_password"
        salt = secrets.token_bytes(16)
        n = 16384
        r = 8
        p = 1
        length = 64
        
        derived_key, params = derive_key_scrypt(
            password=password,
            salt=salt,
            n=n,
            r=r,
            p=p,
            length=length
        )
        
        assert len(derived_key) == length
        assert params.salt == salt
        assert params.length == length
        
        scrypt_params = json.loads(params.info.decode('utf-8'))
        assert scrypt_params['n'] == n
        assert scrypt_params['r'] == r
        assert scrypt_params['p'] == p
    
    def test_derive_key_scrypt_invalid_n(self):
        """Test Scrypt with invalid N parameter"""
        if not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS:
            pytest.skip("Cryptography not available")
        
        with pytest.raises(KeyDerivationError, match="N parameter must be a positive power of 2"):
            derive_key_scrypt("password", n=0)
        
        with pytest.raises(KeyDerivationError, match="N parameter must be a positive power of 2"):
            derive_key_scrypt("password", n=3)  # Not power of 2


class TestEd25519KeyPairDerivation:
    """Test cases for Ed25519 key pair derivation"""
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_derive_ed25519_key_pair_hkdf(self):
        """Test deriving Ed25519 key pair using HKDF"""
        master_key = secrets.token_bytes(32)
        
        key_pair, params = derive_ed25519_key_pair(master_key, derivation_method='HKDF')
        
        assert isinstance(key_pair, Ed25519KeyPair)
        assert len(key_pair.private_key) == ED25519_PRIVATE_KEY_LENGTH
        assert len(key_pair.public_key) == 32
        assert params.algorithm == 'HKDF'
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_derive_ed25519_key_pair_pbkdf2(self):
        """Test deriving Ed25519 key pair using PBKDF2"""
        master_key = b"test_password"
        
        key_pair, params = derive_ed25519_key_pair(master_key, derivation_method='PBKDF2')
        
        assert isinstance(key_pair, Ed25519KeyPair)
        assert len(key_pair.private_key) == ED25519_PRIVATE_KEY_LENGTH
        assert len(key_pair.public_key) == 32
        assert params.algorithm == 'PBKDF2'
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_derive_ed25519_key_pair_scrypt(self):
        """Test deriving Ed25519 key pair using Scrypt"""
        master_key = b"test_password"
        
        key_pair, params = derive_ed25519_key_pair(master_key, derivation_method='Scrypt')
        
        assert isinstance(key_pair, Ed25519KeyPair)
        assert len(key_pair.private_key) == ED25519_PRIVATE_KEY_LENGTH
        assert len(key_pair.public_key) == 32
        assert params.algorithm == 'Scrypt'
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_derive_ed25519_key_pair_custom_context(self):
        """Test deriving Ed25519 key pair with custom context"""
        master_key = secrets.token_bytes(32)
        context = "custom_context"
        
        key_pair, params = derive_ed25519_key_pair(
            master_key, 
            context=context,
            derivation_method='HKDF'
        )
        
        assert isinstance(key_pair, Ed25519KeyPair)
        # Info should contain the context
        assert context.encode('utf-8') in params.info
    
    def test_derive_ed25519_key_pair_unsupported_method(self):
        """Test deriving Ed25519 key pair with unsupported method"""
        if not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS:
            pytest.skip("Cryptography not available")
        
        master_key = secrets.token_bytes(32)
        
        with pytest.raises(KeyDerivationError, match="Unsupported derivation method"):
            derive_ed25519_key_pair(master_key, derivation_method='INVALID')


class TestDerivationVerification:
    """Test cases for derivation verification"""
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_verify_derivation_hkdf_success(self):
        """Test successful HKDF derivation verification"""
        master_key = secrets.token_bytes(32)
        
        derived_key, params = derive_key_hkdf(master_key)
        
        # Verification should succeed
        assert verify_derivation(master_key, derived_key, params) is True
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_verify_derivation_pbkdf2_success(self):
        """Test successful PBKDF2 derivation verification"""
        password = "test_password"
        
        derived_key, params = derive_key_pbkdf2(password)
        
        # Verification should succeed
        assert verify_derivation(password.encode('utf-8'), derived_key, params) is True
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_verify_derivation_scrypt_success(self):
        """Test successful Scrypt derivation verification"""
        password = "test_password"
        
        derived_key, params = derive_key_scrypt(password)
        
        # Verification should succeed
        assert verify_derivation(password.encode('utf-8'), derived_key, params) is True
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_verify_derivation_wrong_key(self):
        """Test derivation verification with wrong master key"""
        master_key = secrets.token_bytes(32)
        wrong_key = secrets.token_bytes(32)
        
        derived_key, params = derive_key_hkdf(master_key)
        
        # Verification should fail
        assert verify_derivation(wrong_key, derived_key, params) is False
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_verify_derivation_wrong_derived_key(self):
        """Test derivation verification with wrong derived key"""
        master_key = secrets.token_bytes(32)
        wrong_derived = secrets.token_bytes(32)
        
        _, params = derive_key_hkdf(master_key)
        
        # Verification should fail
        assert verify_derivation(master_key, wrong_derived, params) is False
    
    def test_verify_derivation_unsupported_algorithm(self):
        """Test derivation verification with unsupported algorithm"""
        master_key = secrets.token_bytes(32)
        derived_key = secrets.token_bytes(32)
        
        params = DerivationParameters(
            algorithm='UNSUPPORTED',
            salt=secrets.token_bytes(16)
        )
        
        # Verification should fail
        assert verify_derivation(master_key, derived_key, params) is False


class TestParameterSerialization:
    """Test cases for parameter serialization"""
    
    def test_export_derivation_parameters(self):
        """Test exporting derivation parameters"""
        params = DerivationParameters(
            algorithm='HKDF',
            salt=b'test_salt',
            info=b'test_info',
            iterations=100000,
            length=32,
            hash_algorithm='SHA256'
        )
        
        exported = export_derivation_parameters(params)
        
        assert isinstance(exported, dict)
        assert exported['algorithm'] == 'HKDF'
        assert 'salt' in exported
        assert 'info' in exported
        assert exported['iterations'] == '100000'
        assert exported['length'] == '32'
        assert exported['hash_algorithm'] == 'SHA256'
    
    def test_import_derivation_parameters(self):
        """Test importing derivation parameters"""
        exported = {
            'algorithm': 'PBKDF2',
            'salt': 'dGVzdF9zYWx0',  # base64 encoded 'test_salt'
            'info': 'dGVzdF9pbmZv',   # base64 encoded 'test_info'
            'iterations': '50000',
            'length': '64',
            'hash_algorithm': 'SHA384'
        }
        
        params = import_derivation_parameters(exported)
        
        assert isinstance(params, DerivationParameters)
        assert params.algorithm == 'PBKDF2'
        assert params.salt == b'test_salt'
        assert params.info == b'test_info'
        assert params.iterations == 50000
        assert params.length == 64
        assert params.hash_algorithm == 'SHA384'
    
    def test_export_import_roundtrip(self):
        """Test that export/import is a successful roundtrip"""
        original_params = DerivationParameters(
            algorithm='Scrypt',
            salt=secrets.token_bytes(16),
            info=b'scrypt_params',
            length=32,
            hash_algorithm='N/A'
        )
        
        exported = export_derivation_parameters(original_params)
        imported_params = import_derivation_parameters(exported)
        
        assert imported_params.algorithm == original_params.algorithm
        assert imported_params.salt == original_params.salt
        assert imported_params.info == original_params.info
        assert imported_params.length == original_params.length
        assert imported_params.hash_algorithm == original_params.hash_algorithm
    
    def test_import_derivation_parameters_invalid(self):
        """Test importing invalid derivation parameters"""
        invalid_exported = {
            'algorithm': 'HKDF',
            'salt': 'invalid_base64!',  # Invalid base64
        }
        
        with pytest.raises(KeyDerivationError, match="Failed to import derivation parameters"):
            import_derivation_parameters(invalid_exported)


class TestIntegration:
    """Integration tests for key derivation functionality"""
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_derive_and_verify_hkdf_integration(self):
        """Test complete HKDF derivation and verification workflow"""
        master_key = secrets.token_bytes(32)
        
        # Derive key
        derived_key, params = derive_key_hkdf(master_key)
        
        # Export and import parameters
        exported = export_derivation_parameters(params)
        imported_params = import_derivation_parameters(exported)
        
        # Verify derivation with imported parameters
        assert verify_derivation(master_key, derived_key, imported_params) is True
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_derive_multiple_ed25519_keys(self):
        """Test deriving multiple Ed25519 key pairs from same master key"""
        master_key = secrets.token_bytes(32)
        
        # Derive different keys with different contexts
        key_pair1, _ = derive_ed25519_key_pair(master_key, context="context1")
        key_pair2, _ = derive_ed25519_key_pair(master_key, context="context2")
        
        # Keys should be different
        assert key_pair1.private_key != key_pair2.private_key
        assert key_pair1.public_key != key_pair2.public_key
    
    @pytest.mark.skipif(not CRYPTOGRAPHY_AVAILABLE_FOR_TESTS, reason="Cryptography not available")
    def test_ed25519_key_pair_cryptographic_correctness(self):
        """Test that derived Ed25519 key pairs are cryptographically correct"""
        from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
        
        master_key = secrets.token_bytes(32)
        key_pair, _ = derive_ed25519_key_pair(master_key)
        
        # Reconstruct cryptography objects
        private_key_obj = Ed25519PrivateKey.from_private_bytes(key_pair.private_key)
        public_key_obj = private_key_obj.public_key()
        
        # Verify public key matches
        from cryptography.hazmat.primitives import serialization
        reconstructed_public = public_key_obj.public_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PublicFormat.Raw
        )
        assert reconstructed_public == key_pair.public_key
        
        # Test signing and verification
        message = b"test message for derived key verification"
        signature = private_key_obj.sign(message)
        
        # Verification should succeed
        public_key_obj.verify(signature, message)  # Should not raise


if __name__ == '__main__':
    pytest.main([__file__])