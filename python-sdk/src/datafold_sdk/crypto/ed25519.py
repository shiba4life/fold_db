"""
Ed25519 key generation and management for DataFold Python SDK

This module provides Ed25519 key pair generation using the cryptography package,
following security best practices for client-side key management.
"""

import os
import sys
import secrets
import platform
from typing import Dict, List, Optional, Tuple, Union, Any
from dataclasses import dataclass
import base64
import binascii

# Import cryptography components
try:
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey, Ed25519PublicKey
    from cryptography.hazmat.primitives import serialization, hashes
    from cryptography.exceptions import InvalidSignature
    CRYPTOGRAPHY_AVAILABLE = True
except ImportError:
    CRYPTOGRAPHY_AVAILABLE = False
    Ed25519PrivateKey = None
    Ed25519PublicKey = None

from ..exceptions import Ed25519KeyError, ValidationError, UnsupportedPlatformError

# Constants for Ed25519 key operations
ED25519_PRIVATE_KEY_LENGTH = 32
ED25519_PUBLIC_KEY_LENGTH = 32
ED25519_SIGNATURE_LENGTH = 64


@dataclass
class Ed25519KeyPair:
    """
    Represents an Ed25519 key pair with private and public keys.
    
    Attributes:
        private_key: The private key as bytes (32 bytes)
        public_key: The public key as bytes (32 bytes)
    """
    private_key: bytes
    public_key: bytes
    
    def __post_init__(self):
        """Validate key pair after initialization"""
        if not isinstance(self.private_key, bytes):
            raise Ed25519KeyError("Private key must be bytes", "INVALID_PRIVATE_KEY_TYPE")
        if not isinstance(self.public_key, bytes):
            raise Ed25519KeyError("Public key must be bytes", "INVALID_PUBLIC_KEY_TYPE")
        
        if len(self.private_key) != ED25519_PRIVATE_KEY_LENGTH:
            raise Ed25519KeyError(
                f"Private key must be exactly {ED25519_PRIVATE_KEY_LENGTH} bytes",
                "INVALID_PRIVATE_KEY_LENGTH"
            )
        
        if len(self.public_key) != ED25519_PUBLIC_KEY_LENGTH:
            raise Ed25519KeyError(
                f"Public key must be exactly {ED25519_PUBLIC_KEY_LENGTH} bytes",
                "INVALID_PUBLIC_KEY_LENGTH"
            )


def check_platform_compatibility() -> Dict[str, Any]:
    """
    Check platform compatibility for Ed25519 operations.
    
    Returns:
        dict: Compatibility information including cryptography availability,
              Ed25519 support, secure random availability, and platform details
    """
    compatibility = {
        'cryptography_available': CRYPTOGRAPHY_AVAILABLE,
        'ed25519_supported': False,
        'secure_random_available': hasattr(secrets, 'token_bytes'),
        'platform_info': {
            'system': platform.system(),
            'python_version': sys.version,
            'architecture': platform.architecture()[0],
        }
    }
    
    if CRYPTOGRAPHY_AVAILABLE:
        try:
            # Test Ed25519 availability by attempting key generation
            Ed25519PrivateKey.generate()
            compatibility['ed25519_supported'] = True
        except Exception:
            compatibility['ed25519_supported'] = False
    
    return compatibility


def _generate_secure_random(length: int) -> bytes:
    """
    Generate cryptographically secure random bytes.
    
    Args:
        length: Number of bytes to generate
        
    Returns:
        bytes: Secure random bytes
        
    Raises:
        UnsupportedPlatformError: If secure random generation is not available
    """
    if not hasattr(secrets, 'token_bytes'):
        raise UnsupportedPlatformError(
            "Secure random number generation not supported on this platform",
            "UNSUPPORTED_RANDOM"
        )
    
    return secrets.token_bytes(length)


def _validate_private_key(private_key: bytes) -> None:
    """
    Validate Ed25519 private key.
    
    Args:
        private_key: Private key bytes to validate
        
    Raises:
        ValidationError: If private key is invalid
    """
    if not isinstance(private_key, bytes):
        raise ValidationError("Private key must be bytes", "INVALID_PRIVATE_KEY_TYPE")
    
    if len(private_key) != ED25519_PRIVATE_KEY_LENGTH:
        raise ValidationError(
            f"Private key must be exactly {ED25519_PRIVATE_KEY_LENGTH} bytes",
            "INVALID_PRIVATE_KEY_LENGTH"
        )
    
    # Check for all-zero key (invalid)
    if private_key == b'\x00' * ED25519_PRIVATE_KEY_LENGTH:
        raise ValidationError("Private key cannot be all zeros", "INVALID_PRIVATE_KEY_VALUE")


def _validate_public_key(public_key: bytes) -> None:
    """
    Validate Ed25519 public key.
    
    Args:
        public_key: Public key bytes to validate
        
    Raises:
        ValidationError: If public key is invalid
    """
    if not isinstance(public_key, bytes):
        raise ValidationError("Public key must be bytes", "INVALID_PUBLIC_KEY_TYPE")
    
    if len(public_key) != ED25519_PUBLIC_KEY_LENGTH:
        raise ValidationError(
            f"Public key must be exactly {ED25519_PUBLIC_KEY_LENGTH} bytes",
            "INVALID_PUBLIC_KEY_LENGTH"
        )


def generate_key_pair(*, validate: bool = True, entropy: Optional[bytes] = None) -> Ed25519KeyPair:
    """
    Generate an Ed25519 key pair using the cryptography package.
    
    Args:
        validate: Whether to validate the generated keys (default: True)
        entropy: Custom entropy source for testing only (32 bytes)
        
    Returns:
        Ed25519KeyPair: The generated key pair
        
    Raises:
        UnsupportedPlatformError: If cryptography package is not available
        Ed25519KeyError: If key generation fails
        ValidationError: If validation fails
    """
    if not CRYPTOGRAPHY_AVAILABLE:
        raise UnsupportedPlatformError(
            "Cryptography package not available - install with: pip install cryptography",
            "CRYPTOGRAPHY_UNAVAILABLE"
        )
    
    try:
        if entropy is not None:
            # Use provided entropy (mainly for testing)
            if len(entropy) != ED25519_PRIVATE_KEY_LENGTH:
                raise Ed25519KeyError(
                    f"Entropy must be exactly {ED25519_PRIVATE_KEY_LENGTH} bytes",
                    "INVALID_ENTROPY_LENGTH"
                )
            
            # Create private key from entropy
            private_key_obj = Ed25519PrivateKey.from_private_bytes(entropy)
        else:
            # Generate new private key with secure random
            private_key_obj = Ed25519PrivateKey.generate()
        
        # Get public key
        public_key_obj = private_key_obj.public_key()
        
        # Extract raw bytes
        private_key_bytes = private_key_obj.private_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PrivateFormat.Raw,
            encryption_algorithm=serialization.NoEncryption()
        )
        
        public_key_bytes = public_key_obj.public_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PublicFormat.Raw
        )
        
        # Validate keys if requested
        if validate:
            _validate_private_key(private_key_bytes)
            _validate_public_key(public_key_bytes)
            
            # Additional validation: verify key pair consistency
            test_message = b"test_message_for_validation"
            signature = private_key_obj.sign(test_message)
            
            try:
                public_key_obj.verify(signature, test_message)
            except InvalidSignature:
                raise Ed25519KeyError(
                    "Generated key pair failed consistency validation",
                    "KEY_PAIR_INCONSISTENT"
                )
        
        return Ed25519KeyPair(
            private_key=private_key_bytes,
            public_key=public_key_bytes
        )
        
    except Exception as e:
        if isinstance(e, (Ed25519KeyError, ValidationError, UnsupportedPlatformError)):
            raise
        
        # Wrap any other errors
        raise Ed25519KeyError(
            f"Key generation failed: {str(e)}",
            "GENERATION_FAILED"
        ) from e


def generate_multiple_key_pairs(count: int, *, validate: bool = True) -> List[Ed25519KeyPair]:
    """
    Generate multiple Ed25519 key pairs efficiently.
    
    Args:
        count: Number of key pairs to generate (1-100)
        validate: Whether to validate the generated keys
        
    Returns:
        List[Ed25519KeyPair]: List of generated key pairs
        
    Raises:
        Ed25519KeyError: If count is invalid or generation fails
    """
    if not isinstance(count, int) or count <= 0:
        raise Ed25519KeyError("Count must be a positive integer", "INVALID_COUNT")
    
    if count > 100:
        raise Ed25519KeyError(
            "Cannot generate more than 100 key pairs at once",
            "COUNT_TOO_LARGE"
        )
    
    key_pairs = []
    for _ in range(count):
        key_pair = generate_key_pair(validate=validate)
        key_pairs.append(key_pair)
    
    return key_pairs


def format_key(key: bytes, format_type: str) -> Union[str, bytes]:
    """
    Convert key to different formats.
    
    Args:
        key: The key bytes to format
        format_type: Output format ('hex', 'base64', 'bytes', 'pem')
        
    Returns:
        Union[str, bytes]: Formatted key
        
    Raises:
        Ed25519KeyError: If format is unsupported or key is invalid
    """
    if not isinstance(key, bytes):
        raise Ed25519KeyError("Key must be bytes", "INVALID_KEY_TYPE")
    
    if format_type == 'hex':
        return key.hex()
    elif format_type == 'base64':
        return base64.b64encode(key).decode('ascii')
    elif format_type == 'bytes':
        return bytes(key)  # Return a copy
    elif format_type == 'pem':
        # For PEM format, we need to reconstruct the key object
        if len(key) == ED25519_PRIVATE_KEY_LENGTH:
            # Private key
            try:
                private_key_obj = Ed25519PrivateKey.from_private_bytes(key)
                return private_key_obj.private_bytes(
                    encoding=serialization.Encoding.PEM,
                    format=serialization.PrivateFormat.PKCS8,
                    encryption_algorithm=serialization.NoEncryption()
                ).decode('ascii')
            except Exception as e:
                raise Ed25519KeyError(f"Failed to format private key as PEM: {e}", "PEM_FORMAT_FAILED")
        elif len(key) == ED25519_PUBLIC_KEY_LENGTH:
            # Public key
            try:
                public_key_obj = Ed25519PublicKey.from_public_bytes(key)
                return public_key_obj.public_bytes(
                    encoding=serialization.Encoding.PEM,
                    format=serialization.PublicFormat.SubjectPublicKeyInfo
                ).decode('ascii')
            except Exception as e:
                raise Ed25519KeyError(f"Failed to format public key as PEM: {e}", "PEM_FORMAT_FAILED")
        else:
            raise Ed25519KeyError("Invalid key length for PEM format", "INVALID_KEY_LENGTH")
    else:
        raise Ed25519KeyError(f"Unsupported format: {format_type}", "UNSUPPORTED_FORMAT")


def parse_key(key_data: Union[str, bytes], format_type: str) -> bytes:
    """
    Parse key from different formats.
    
    Args:
        key_data: The key data to parse
        format_type: Input format ('hex', 'base64', 'bytes', 'pem')
        
    Returns:
        bytes: Parsed key as bytes
        
    Raises:
        Ed25519KeyError: If format is unsupported or parsing fails
    """
    if format_type == 'hex':
        if not isinstance(key_data, str):
            raise Ed25519KeyError("Hex format requires string input", "INVALID_HEX_INPUT")
        
        try:
            return bytes.fromhex(key_data)
        except ValueError as e:
            raise Ed25519KeyError(f"Invalid hex string: {e}", "INVALID_HEX")
            
    elif format_type == 'base64':
        if not isinstance(key_data, str):
            raise Ed25519KeyError("Base64 format requires string input", "INVALID_BASE64_INPUT")
        
        try:
            return base64.b64decode(key_data)
        except Exception as e:
            raise Ed25519KeyError(f"Invalid base64 string: {e}", "INVALID_BASE64")
            
    elif format_type == 'bytes':
        if not isinstance(key_data, bytes):
            raise Ed25519KeyError("Bytes format requires bytes input", "INVALID_BYTES_INPUT")
        
        return bytes(key_data)  # Return a copy
        
    elif format_type == 'pem':
        if not isinstance(key_data, str):
            raise Ed25519KeyError("PEM format requires string input", "INVALID_PEM_INPUT")
        
        try:
            key_data_bytes = key_data.encode('ascii')
            
            # Try to parse as private key first
            try:
                private_key_obj = serialization.load_pem_private_key(
                    key_data_bytes,
                    password=None
                )
                if isinstance(private_key_obj, Ed25519PrivateKey):
                    return private_key_obj.private_bytes(
                        encoding=serialization.Encoding.Raw,
                        format=serialization.PrivateFormat.Raw,
                        encryption_algorithm=serialization.NoEncryption()
                    )
                else:
                    raise Ed25519KeyError("PEM does not contain Ed25519 private key", "INVALID_PEM_KEY_TYPE")
            except Exception:
                # Try to parse as public key
                try:
                    public_key_obj = serialization.load_pem_public_key(key_data_bytes)
                    if isinstance(public_key_obj, Ed25519PublicKey):
                        return public_key_obj.public_bytes(
                            encoding=serialization.Encoding.Raw,
                            format=serialization.PublicFormat.Raw
                        )
                    else:
                        raise Ed25519KeyError("PEM does not contain Ed25519 public key", "INVALID_PEM_KEY_TYPE")
                except Exception as e:
                    raise Ed25519KeyError(f"Failed to parse PEM key: {e}", "INVALID_PEM")
                    
        except Exception as e:
            if isinstance(e, Ed25519KeyError):
                raise
            raise Ed25519KeyError(f"PEM parsing failed: {e}", "PEM_PARSE_FAILED")
    else:
        raise Ed25519KeyError(f"Unsupported format: {format_type}", "UNSUPPORTED_FORMAT")


def clear_key_material(key_pair: Ed25519KeyPair) -> None:
    """
    Clear sensitive key material from memory (best effort).
    
    Note: Python doesn't provide guaranteed memory clearing due to garbage collection
    and string immutability, but this provides a best-effort approach.
    
    Args:
        key_pair: The key pair to clear
    """
    try:
        # Python bytes are immutable, so we can't actually overwrite them
        # But we can replace the references to reduce the window of exposure
        if hasattr(key_pair, 'private_key'):
            # Replace with random data of same length, then with zeros
            random_data = _generate_secure_random(len(key_pair.private_key))
            key_pair.private_key = random_data
            key_pair.private_key = b'\x00' * len(random_data)
        
        if hasattr(key_pair, 'public_key'):
            # Public keys are not as sensitive, but clear for consistency
            key_pair.public_key = b'\x00' * len(key_pair.public_key)
            
    except Exception:
        # Silently ignore errors in cleanup
        pass


def sign_message(private_key: bytes, message: Union[str, bytes]) -> bytes:
    """
    Sign a message using Ed25519 private key.
    
    Args:
        private_key: Ed25519 private key bytes (32 bytes)
        message: Message to sign (string or bytes)
        
    Returns:
        bytes: Ed25519 signature (64 bytes)
        
    Raises:
        Ed25519KeyError: If signing fails
        ValidationError: If inputs are invalid
    """
    if not CRYPTOGRAPHY_AVAILABLE:
        raise UnsupportedPlatformError(
            "Cryptography package not available for signing",
            "CRYPTOGRAPHY_UNAVAILABLE"
        )
    
    # Validate private key
    _validate_private_key(private_key)
    
    # Convert message to bytes if needed
    if isinstance(message, str):
        message_bytes = message.encode('utf-8')
    else:
        message_bytes = message
    
    try:
        # Create private key object
        private_key_obj = Ed25519PrivateKey.from_private_bytes(private_key)
        
        # Sign the message
        signature = private_key_obj.sign(message_bytes)
        
        return signature
        
    except Exception as e:
        raise Ed25519KeyError(f"Message signing failed: {e}", "SIGNING_FAILED") from e


def verify_signature(public_key: bytes, message: Union[str, bytes], signature: bytes) -> bool:
    """
    Verify a signature using Ed25519 public key.
    
    Args:
        public_key: Ed25519 public key bytes (32 bytes)
        message: Original message (string or bytes)
        signature: Signature to verify (64 bytes)
        
    Returns:
        bool: True if signature is valid, False otherwise
        
    Raises:
        Ed25519KeyError: If verification process fails
        ValidationError: If inputs are invalid
    """
    if not CRYPTOGRAPHY_AVAILABLE:
        raise UnsupportedPlatformError(
            "Cryptography package not available for verification",
            "CRYPTOGRAPHY_UNAVAILABLE"
        )
    
    # Validate inputs
    _validate_public_key(public_key)
    
    if not isinstance(signature, bytes) or len(signature) != ED25519_SIGNATURE_LENGTH:
        raise ValidationError(f"Signature must be {ED25519_SIGNATURE_LENGTH} bytes")
    
    # Convert message to bytes if needed
    if isinstance(message, str):
        message_bytes = message.encode('utf-8')
    else:
        message_bytes = message
    
    try:
        # Create public key object
        public_key_obj = Ed25519PublicKey.from_public_bytes(public_key)
        
        # Verify the signature
        public_key_obj.verify(signature, message_bytes)
        return True
        
    except InvalidSignature:
        return False
    except Exception as e:
        raise Ed25519KeyError(f"Signature verification failed: {e}", "VERIFICATION_FAILED") from e