"""
Unit tests for Ed25519 key generation functionality
"""

import pytest
import secrets
from unittest.mock import patch, MagicMock

from datafold_sdk.crypto.ed25519 import (
    Ed25519KeyPair,
    generate_key_pair,
    generate_multiple_key_pairs,
    format_key,
    parse_key,
    clear_key_material,
    check_platform_compatibility,
    _validate_private_key,
    _validate_public_key,
    ED25519_PRIVATE_KEY_LENGTH,
    ED25519_PUBLIC_KEY_LENGTH,
)
from datafold_sdk.exceptions import (
    Ed25519KeyError,
    ValidationError,
    UnsupportedPlatformError,
)

# Import cryptography for integration tests
try:
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
    from cryptography.hazmat.primitives import serialization
    CRYPTOGRAPHY_AVAILABLE_FOR_TESTS = True
except ImportError:
    CRYPTOGRAPHY_AVAILABLE_FOR_TESTS = False


class TestEd25519KeyPair:
    """Test cases for Ed25519KeyPair dataclass"""
    
    def test_valid_key_pair_creation(self):
        """Test creating a valid Ed25519KeyPair"""
        private_key = secrets.token_bytes(ED25519_PRIVATE_KEY_LENGTH)
        public_key = secrets.token_bytes(ED25519_PUBLIC_KEY_LENGTH)
        
        key_pair = Ed25519KeyPair(private_key=private_key, public_key=public_key)
        
        assert key_pair.private_key == private_key
        assert key_pair.public_key == public_key
    
    def test_invalid_private_key_type(self):
        """Test that non-bytes private key raises error"""
        with pytest.raises(Ed25519KeyError, match="Private key must be bytes"):
            Ed25519KeyPair(private_key="not_bytes", public_key=secrets.token_bytes(32))
    
    def test_invalid_public_key_type(self):
        """Test that non-bytes public key raises error"""
        with pytest.raises(Ed25519KeyError, match="Public key must be bytes"):
            Ed25519KeyPair(private_key=secrets.token_bytes(32), public_key="not_bytes")
    
    def test_invalid_private_key_length(self):
        """Test that wrong length private key raises error"""
        with pytest.raises(Ed25519KeyError, match="Private key must be exactly 32 bytes"):
            Ed25519KeyPair(private_key=b"too_short", public_key=secrets.token_bytes(32))
    
    def test_invalid_public_key_length(self):
        """Test that wrong length public key raises error"""
        with pytest.raises(Ed25519KeyError, match="Public key must be exactly 32 bytes"):
            Ed25519KeyPair(private_key=secrets.token_bytes(32), public_key=b"too_short")


class TestPlatformCompatibility:
    """Test cases for platform compatibility checking"""
    
    def test_check_platform_compatibility_success(self):
        """Test platform compatibility check when everything is available"""
        result = check_platform_compatibility()
        
        assert isinstance(result, dict)
        assert 'cryptography_available' in result
        assert 'ed25519_supported' in result
        assert 'secure_random_available' in result
        assert 'platform_info' in result
        
        # Should be True in a proper test environment
        assert result['cryptography_available'] is True
        assert result['secure_random_available'] is True
    
    @patch('datafold_sdk.crypto.ed25519.CRYPTOGRAPHY_AVAILABLE', False)
    def test_check_platform_compatibility_no_cryptography(self):
        """Test platform compatibility when cryptography is not available"""
        result = check_platform_compatibility()
        
        assert result['cryptography_available'] is False
        assert result['ed25519_supported'] is False


class TestKeyValidation:
    """Test cases for key validation functions"""
    
    def test_validate_private_key_success(self):
        """Test successful private key validation"""
        valid_key = secrets.token_bytes(ED25519_PRIVATE_KEY_LENGTH)
        # Should not raise an exception
        _validate_private_key(valid_key)
    
    def test_validate_private_key_wrong_type(self):
        """Test private key validation with wrong type"""
        with pytest.raises(ValidationError, match="Private key must be bytes"):
            _validate_private_key("not_bytes")
    
    def test_validate_private_key_wrong_length(self):
        """Test private key validation with wrong length"""
        with pytest.raises(ValidationError, match="Private key must be exactly 32 bytes"):
            _validate_private_key(b"too_short")
    
    def test_validate_private_key_all_zeros(self):
        """Test private key validation with all zeros"""
        zero_key = b'\x00' * ED25519_PRIVATE_KEY_LENGTH
        with pytest.raises(ValidationError, match="Private key cannot be all zeros"):
            _validate_private_key(zero_key)
    
    def test_validate_public_key_success(self):
        """Test successful public key validation"""
        valid_key = secrets.token_bytes(ED25519_PUBLIC_KEY_LENGTH)
        # Should not raise an exception
        _validate_public_key(valid_key)
    
    def test_validate_public_key_wrong_type(self):
        """Test public key validation with wrong type"""
        with pytest.raises(ValidationError, match="Public key must be bytes"):
            _validate_public_key("not_bytes")
    
    def test_validate_public_key_wrong_length(self):
        """Test public key validation with wrong length"""
        with pytest.raises(ValidationError, match="Public key must be exactly 32 bytes"):
            _validate_public_key(b"too_short")


class TestKeyGeneration:
    """Test cases for key generation functions"""
    
    def test_generate_key_pair_success(self):
        """Test successful key pair generation"""
        key_pair = generate_key_pair()
        
        assert isinstance(key_pair, Ed25519KeyPair)
        assert len(key_pair.private_key) == ED25519_PRIVATE_KEY_LENGTH
        assert len(key_pair.public_key) == ED25519_PUBLIC_KEY_LENGTH
        assert key_pair.private_key != b'\x00' * ED25519_PRIVATE_KEY_LENGTH
    
    def test_generate_key_pair_with_entropy(self):
        """Test key pair generation with custom entropy"""
        entropy = secrets.token_bytes(ED25519_PRIVATE_KEY_LENGTH)
        key_pair = generate_key_pair(entropy=entropy)
        
        assert isinstance(key_pair, Ed25519KeyPair)
        assert len(key_pair.private_key) == ED25519_PRIVATE_KEY_LENGTH
        assert len(key_pair.public_key) == ED25519_PUBLIC_KEY_LENGTH
    
    def test_generate_key_pair_invalid_entropy_length(self):
        """Test key pair generation with invalid entropy length"""
        invalid_entropy = b"too_short"
        
        with pytest.raises(Ed25519KeyError, match="Entropy must be exactly 32 bytes"):
            generate_key_pair(entropy=invalid_entropy)
    
    def test_generate_key_pair_no_validation(self):
        """Test key pair generation without validation"""
        key_pair = generate_key_pair(validate=False)
        
        assert isinstance(key_pair, Ed25519KeyPair)
        assert len(key_pair.private_key) == ED25519_PRIVATE_KEY_LENGTH
        assert len(key_pair.public_key) == ED25519_PUBLIC_KEY_LENGTH
    
    @patch('datafold_sdk.crypto.ed25519.CRYPTOGRAPHY_AVAILABLE', False)
    def test_generate_key_pair_no_cryptography(self):
        """Test key pair generation when cryptography is not available"""
        with pytest.raises(UnsupportedPlatformError, match="Cryptography package not available"):
            generate_key_pair()
    
    def test_generate_multiple_key_pairs_success(self):
        """Test generating multiple key pairs"""
        count = 5
        key_pairs = generate_multiple_key_pairs(count)
        
        assert len(key_pairs) == count
        assert all(isinstance(kp, Ed25519KeyPair) for kp in key_pairs)
        
        # All key pairs should be different
        private_keys = [kp.private_key for kp in key_pairs]
        assert len(set(private_keys)) == count  # All unique
    
    def test_generate_multiple_key_pairs_invalid_count(self):
        """Test generating multiple key pairs with invalid count"""
        with pytest.raises(Ed25519KeyError, match="Count must be a positive integer"):
            generate_multiple_key_pairs(0)
        
        with pytest.raises(Ed25519KeyError, match="Count must be a positive integer"):
            generate_multiple_key_pairs(-1)
        
        with pytest.raises(Ed25519KeyError, match="Count must be a positive integer"):
            generate_multiple_key_pairs("not_int")
    
    def test_generate_multiple_key_pairs_too_large(self):
        """Test generating too many key pairs at once"""
        with pytest.raises(Ed25519KeyError, match="Cannot generate more than 100 key pairs"):
            generate_multiple_key_pairs(101)


class TestKeyFormatting:
    """Test cases for key formatting functions"""
    
    def test_format_key_hex(self):
        """Test formatting key as hex"""
        key = b'\x01\x02\x03\x04'
        result = format_key(key, 'hex')
        
        assert result == "01020304"
        assert isinstance(result, str)
    
    def test_format_key_base64(self):
        """Test formatting key as base64"""
        key = b'\x01\x02\x03\x04'
        result = format_key(key, 'base64')
        
        assert isinstance(result, str)
        # Should be valid base64
        import base64
        decoded = base64.b64decode(result)
        assert decoded == key
    
    def test_format_key_bytes(self):
        """Test formatting key as bytes"""
        key = b'\x01\x02\x03\x04'
        result = format_key(key, 'bytes')
        
        assert result == key
        assert result is not key  # Should be a copy
        assert isinstance(result, bytes)
    
    def test_format_key_pem_private(self):
        """Test formatting private key as PEM"""
        # Generate a real key for PEM testing
        key_pair = generate_key_pair()
        result = format_key(key_pair.private_key, 'pem')
        
        assert isinstance(result, str)
        assert '-----BEGIN PRIVATE KEY-----' in result
        assert '-----END PRIVATE KEY-----' in result
    
    def test_format_key_pem_public(self):
        """Test formatting public key as PEM"""
        # Generate a real key for PEM testing
        key_pair = generate_key_pair()
        result = format_key(key_pair.public_key, 'pem')
        
        assert isinstance(result, str)
        assert '-----BEGIN PUBLIC KEY-----' in result
        assert '-----END PUBLIC KEY-----' in result
    
    def test_format_key_invalid_type(self):
        """Test formatting with invalid key type"""
        with pytest.raises(Ed25519KeyError, match="Key must be bytes"):
            format_key("not_bytes", 'hex')
    
    def test_format_key_unsupported_format(self):
        """Test formatting with unsupported format"""
        key = b'\x01\x02\x03\x04'
        with pytest.raises(Ed25519KeyError, match="Unsupported format"):
            format_key(key, 'unsupported')


class TestKeyParsing:
    """Test cases for key parsing functions"""
    
    def test_parse_key_hex(self):
        """Test parsing key from hex"""
        hex_key = "01020304"
        result = parse_key(hex_key, 'hex')
        
        assert result == b'\x01\x02\x03\x04'
        assert isinstance(result, bytes)
    
    def test_parse_key_base64(self):
        """Test parsing key from base64"""
        import base64
        key = b'\x01\x02\x03\x04'
        base64_key = base64.b64encode(key).decode('ascii')
        result = parse_key(base64_key, 'base64')
        
        assert result == key
        assert isinstance(result, bytes)
    
    def test_parse_key_bytes(self):
        """Test parsing key from bytes"""
        key = b'\x01\x02\x03\x04'
        result = parse_key(key, 'bytes')
        
        assert result == key
        assert result is not key  # Should be a copy
        assert isinstance(result, bytes)
    
    def test_parse_key_pem_private(self):
        """Test parsing private key from PEM"""
        # Generate a real key and format as PEM
        key_pair = generate_key_pair()
        pem_key = format_key(key_pair.private_key, 'pem')
        result = parse_key(pem_key, 'pem')
        
        assert result == key_pair.private_key
        assert isinstance(result, bytes)
    
    def test_parse_key_pem_public(self):
        """Test parsing public key from PEM"""
        # Generate a real key and format as PEM
        key_pair = generate_key_pair()
        pem_key = format_key(key_pair.public_key, 'pem')
        result = parse_key(pem_key, 'pem')
        
        assert result == key_pair.public_key
        assert isinstance(result, bytes)
    
    def test_parse_key_invalid_hex(self):
        """Test parsing invalid hex"""
        with pytest.raises(Ed25519KeyError, match="Invalid hex string"):
            parse_key("invalid_hex", 'hex')
    
    def test_parse_key_invalid_base64(self):
        """Test parsing invalid base64"""
        with pytest.raises(Ed25519KeyError, match="Invalid base64 string"):
            parse_key("invalid_base64!", 'base64')
    
    def test_parse_key_invalid_pem(self):
        """Test parsing invalid PEM"""
        with pytest.raises(Ed25519KeyError, match="Failed to parse PEM key"):
            parse_key("invalid_pem", 'pem')
    
    def test_parse_key_unsupported_format(self):
        """Test parsing with unsupported format"""
        with pytest.raises(Ed25519KeyError, match="Unsupported format"):
            parse_key("data", 'unsupported')


class TestKeyClearance:
    """Test cases for key clearance function"""
    
    def test_clear_key_material(self):
        """Test clearing key material"""
        key_pair = generate_key_pair()
        original_private = key_pair.private_key
        original_public = key_pair.public_key
        
        clear_key_material(key_pair)
        
        # Keys should be different (cleared)
        assert key_pair.private_key != original_private
        assert key_pair.public_key != original_public
        
        # Should be zeros
        assert key_pair.private_key == b'\x00' * len(original_private)
        assert key_pair.public_key == b'\x00' * len(original_public)
    
    def test_clear_key_material_exception_handling(self):
        """Test that clear_key_material handles exceptions gracefully"""
        # Create a mock key pair that will raise an exception
        mock_key_pair = MagicMock()
        mock_key_pair.private_key = MagicMock()
        mock_key_pair.private_key.__len__.side_effect = Exception("Test exception")
        
        # Should not raise an exception
        clear_key_material(mock_key_pair)


class TestIntegration:
    """Integration tests for Ed25519 functionality"""
    
    def test_key_generation_and_formatting_roundtrip(self):
        """Test that generated keys can be formatted and parsed back"""
        key_pair = generate_key_pair()
        
        for format_type in ['hex', 'base64', 'bytes', 'pem']:
            # Format private key
            formatted_private = format_key(key_pair.private_key, format_type)
            parsed_private = parse_key(formatted_private, format_type)
            assert parsed_private == key_pair.private_key
            
            # Format public key
            formatted_public = format_key(key_pair.public_key, format_type)
            parsed_public = parse_key(formatted_public, format_type)
            assert parsed_public == key_pair.public_key
    
    def test_key_pair_consistency(self):
        """Test that generated key pairs are cryptographically consistent"""
        from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
        
        key_pair = generate_key_pair()
        
        # Reconstruct cryptography objects
        private_key_obj = Ed25519PrivateKey.from_private_bytes(key_pair.private_key)
        public_key_obj = private_key_obj.public_key()
        
        # Verify public key matches
        reconstructed_public = public_key_obj.public_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PublicFormat.Raw
        )
        assert reconstructed_public == key_pair.public_key
        
        # Test signing and verification
        message = b"test message for signature verification"
        signature = private_key_obj.sign(message)
        
        # Verification should succeed
        public_key_obj.verify(signature, message)  # Should not raise
    
    def test_entropy_determinism(self):
        """Test that same entropy produces same key pair"""
        entropy = secrets.token_bytes(32)
        
        key_pair1 = generate_key_pair(entropy=entropy)
        key_pair2 = generate_key_pair(entropy=entropy)
        
        assert key_pair1.private_key == key_pair2.private_key
        assert key_pair1.public_key == key_pair2.public_key
    
    def test_different_entropy_produces_different_keys(self):
        """Test that different entropy produces different key pairs"""
        entropy1 = secrets.token_bytes(32)
        entropy2 = secrets.token_bytes(32)
        
        # Ensure different entropy
        while entropy1 == entropy2:
            entropy2 = secrets.token_bytes(32)
        
        key_pair1 = generate_key_pair(entropy=entropy1)
        key_pair2 = generate_key_pair(entropy=entropy2)
        
        assert key_pair1.private_key != key_pair2.private_key
        assert key_pair1.public_key != key_pair2.public_key


if __name__ == '__main__':
    pytest.main([__file__])