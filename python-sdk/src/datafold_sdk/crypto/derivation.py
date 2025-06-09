"""
Key derivation functionality for DataFold Python SDK

This module provides key derivation functions for creating derived keys from master keys
using cryptographically secure methods like HKDF and PBKDF2.
"""

import os
import secrets
from typing import Dict, List, Optional, Tuple, Union, Any
from dataclasses import dataclass
import base64

# Import cryptography components
try:
    from cryptography.hazmat.primitives.kdf.hkdf import HKDF
    from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
    from cryptography.hazmat.primitives.kdf.scrypt import Scrypt
    from cryptography.hazmat.primitives import hashes
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
    CRYPTOGRAPHY_AVAILABLE = True
except ImportError:
    CRYPTOGRAPHY_AVAILABLE = False
    HKDF = None
    PBKDF2HMAC = None
    Scrypt = None
    hashes = None
    Ed25519PrivateKey = None

from ..exceptions import KeyDerivationError, ValidationError, UnsupportedPlatformError
from .ed25519 import Ed25519KeyPair, ED25519_PRIVATE_KEY_LENGTH, ED25519_PUBLIC_KEY_LENGTH

# Constants for key derivation
DEFAULT_SALT_LENGTH = 32
DEFAULT_INFO_LENGTH = 32
PBKDF2_ITERATIONS = 100000  # OWASP recommended minimum
SCRYPT_N = 32768           # CPU cost factor
SCRYPT_R = 8               # Memory cost factor  
SCRYPT_P = 1               # Parallelization factor


@dataclass
class DerivationParameters:
    """
    Parameters for key derivation operations
    
    Attributes:
        algorithm: Derivation algorithm ('HKDF', 'PBKDF2', 'Scrypt')
        salt: Salt value for derivation
        info: Context information for HKDF
        iterations: Number of iterations for PBKDF2
        length: Output key length in bytes
        hash_algorithm: Hash algorithm to use
    """
    algorithm: str
    salt: bytes
    info: Optional[bytes] = None
    iterations: Optional[int] = None
    length: int = ED25519_PRIVATE_KEY_LENGTH
    hash_algorithm: str = 'SHA256'


def check_derivation_support() -> Dict[str, Any]:
    """
    Check platform support for key derivation operations.
    
    Returns:
        dict: Support information for different derivation methods
    """
    support = {
        'cryptography_available': CRYPTOGRAPHY_AVAILABLE,
        'hkdf_supported': False,
        'pbkdf2_supported': False,
        'scrypt_supported': False,
        'supported_hashes': [],
    }
    
    if CRYPTOGRAPHY_AVAILABLE:
        try:
            # Test HKDF availability
            HKDF(hashes.SHA256(), 32, b'salt', b'info')
            support['hkdf_supported'] = True
        except Exception:
            pass
            
        try:
            # Test PBKDF2 availability
            PBKDF2HMAC(hashes.SHA256(), 32, b'salt', 100000)
            support['pbkdf2_supported'] = True
        except Exception:
            pass
            
        try:
            # Test Scrypt availability
            Scrypt(32, b'salt', SCRYPT_N, SCRYPT_R, SCRYPT_P)
            support['scrypt_supported'] = True
        except Exception:
            pass
            
        # List supported hash algorithms
        supported_hashes = ['SHA256', 'SHA384', 'SHA512', 'SHA3_256', 'SHA3_384', 'SHA3_512']
        for hash_name in supported_hashes:
            try:
                hash_obj = getattr(hashes, hash_name)()
                support['supported_hashes'].append(hash_name)
            except Exception:
                pass
    
    return support


def _get_hash_algorithm(hash_name: str):
    """Get hash algorithm object from name"""
    if not CRYPTOGRAPHY_AVAILABLE:
        raise UnsupportedPlatformError(
            "Cryptography package required for key derivation",
            "CRYPTOGRAPHY_UNAVAILABLE"
        )
    
    hash_algorithms = {
        'SHA256': hashes.SHA256,
        'SHA384': hashes.SHA384,
        'SHA512': hashes.SHA512,
        'SHA3_256': hashes.SHA3_256,
        'SHA3_384': hashes.SHA3_384,
        'SHA3_512': hashes.SHA3_512,
    }
    
    if hash_name not in hash_algorithms:
        raise KeyDerivationError(
            f"Unsupported hash algorithm: {hash_name}",
            "UNSUPPORTED_HASH_ALGORITHM"
        )
    
    return hash_algorithms[hash_name]()


def _generate_salt(length: int = DEFAULT_SALT_LENGTH) -> bytes:
    """Generate cryptographically secure salt"""
    return secrets.token_bytes(length)


def _generate_info(context: str, length: int = DEFAULT_INFO_LENGTH) -> bytes:
    """Generate info parameter for HKDF from context string"""
    context_bytes = context.encode('utf-8')
    if len(context_bytes) >= length:
        return context_bytes[:length]
    else:
        # Pad with random bytes to reach desired length
        padding = secrets.token_bytes(length - len(context_bytes))
        return context_bytes + padding


def derive_key_hkdf(master_key: bytes, 
                   salt: Optional[bytes] = None,
                   info: Optional[bytes] = None,
                   length: int = ED25519_PRIVATE_KEY_LENGTH,
                   hash_algorithm: str = 'SHA256') -> Tuple[bytes, DerivationParameters]:
    """
    Derive a key using HKDF (HMAC-based Key Derivation Function).
    
    Args:
        master_key: Master key material for derivation
        salt: Salt value (generated if None)
        info: Context information (generated if None)
        length: Output key length in bytes
        hash_algorithm: Hash algorithm to use
        
    Returns:
        Tuple of (derived_key, derivation_parameters)
        
    Raises:
        KeyDerivationError: If derivation fails
        UnsupportedPlatformError: If HKDF is not supported
    """
    if not CRYPTOGRAPHY_AVAILABLE:
        raise UnsupportedPlatformError(
            "Cryptography package required for HKDF",
            "CRYPTOGRAPHY_UNAVAILABLE"
        )
    
    if not isinstance(master_key, bytes) or len(master_key) == 0:
        raise KeyDerivationError(
            "Master key must be non-empty bytes",
            "INVALID_MASTER_KEY"
        )
    
    if length <= 0 or length > 255 * 32:  # HKDF limit for SHA256
        raise KeyDerivationError(
            f"Invalid key length: {length}",
            "INVALID_KEY_LENGTH"
        )
    
    try:
        # Generate salt and info if not provided
        if salt is None:
            salt = _generate_salt()
        if info is None:
            info = _generate_info("DataFold-HKDF-Context")
        
        # Get hash algorithm
        hash_obj = _get_hash_algorithm(hash_algorithm)
        
        # Derive key using HKDF
        hkdf = HKDF(
            algorithm=hash_obj,
            length=length,
            salt=salt,
            info=info,
        )
        
        derived_key = hkdf.derive(master_key)
        
        # Create parameters object
        params = DerivationParameters(
            algorithm='HKDF',
            salt=salt,
            info=info,
            length=length,
            hash_algorithm=hash_algorithm
        )
        
        return derived_key, params
        
    except Exception as e:
        if isinstance(e, (KeyDerivationError, UnsupportedPlatformError)):
            raise
        
        raise KeyDerivationError(
            f"HKDF derivation failed: {str(e)}",
            "HKDF_DERIVATION_FAILED"
        ) from e


def derive_key_pbkdf2(password: Union[str, bytes],
                     salt: Optional[bytes] = None,
                     iterations: int = PBKDF2_ITERATIONS,
                     length: int = ED25519_PRIVATE_KEY_LENGTH,
                     hash_algorithm: str = 'SHA256') -> Tuple[bytes, DerivationParameters]:
    """
    Derive a key using PBKDF2 (Password-Based Key Derivation Function 2).
    
    Args:
        password: Password/passphrase for derivation
        salt: Salt value (generated if None)
        iterations: Number of iterations
        length: Output key length in bytes
        hash_algorithm: Hash algorithm to use
        
    Returns:
        Tuple of (derived_key, derivation_parameters)
        
    Raises:
        KeyDerivationError: If derivation fails
        UnsupportedPlatformError: If PBKDF2 is not supported
    """
    if not CRYPTOGRAPHY_AVAILABLE:
        raise UnsupportedPlatformError(
            "Cryptography package required for PBKDF2",
            "CRYPTOGRAPHY_UNAVAILABLE"
        )
    
    # Convert password to bytes if string
    if isinstance(password, str):
        password_bytes = password.encode('utf-8')
    elif isinstance(password, bytes):
        password_bytes = password
    else:
        raise KeyDerivationError(
            "Password must be string or bytes",
            "INVALID_PASSWORD_TYPE"
        )
    
    if len(password_bytes) == 0:
        raise KeyDerivationError(
            "Password cannot be empty",
            "EMPTY_PASSWORD"
        )
    
    if iterations < 1000:
        raise KeyDerivationError(
            "Iterations must be at least 1000",
            "INSUFFICIENT_ITERATIONS"
        )
    
    if length <= 0:
        raise KeyDerivationError(
            f"Invalid key length: {length}",
            "INVALID_KEY_LENGTH"
        )
    
    try:
        # Generate salt if not provided
        if salt is None:
            salt = _generate_salt()
        
        # Get hash algorithm
        hash_obj = _get_hash_algorithm(hash_algorithm)
        
        # Derive key using PBKDF2
        pbkdf2 = PBKDF2HMAC(
            algorithm=hash_obj,
            length=length,
            salt=salt,
            iterations=iterations,
        )
        
        derived_key = pbkdf2.derive(password_bytes)
        
        # Create parameters object
        params = DerivationParameters(
            algorithm='PBKDF2',
            salt=salt,
            iterations=iterations,
            length=length,
            hash_algorithm=hash_algorithm
        )
        
        return derived_key, params
        
    except Exception as e:
        if isinstance(e, (KeyDerivationError, UnsupportedPlatformError)):
            raise
        
        raise KeyDerivationError(
            f"PBKDF2 derivation failed: {str(e)}",
            "PBKDF2_DERIVATION_FAILED"
        ) from e


def derive_key_scrypt(password: Union[str, bytes],
                     salt: Optional[bytes] = None,
                     n: int = SCRYPT_N,
                     r: int = SCRYPT_R,
                     p: int = SCRYPT_P,
                     length: int = ED25519_PRIVATE_KEY_LENGTH) -> Tuple[bytes, DerivationParameters]:
    """
    Derive a key using Scrypt (memory-hard key derivation function).
    
    Args:
        password: Password/passphrase for derivation
        salt: Salt value (generated if None)
        n: CPU cost factor (must be power of 2)
        r: Memory cost factor
        p: Parallelization factor
        length: Output key length in bytes
        
    Returns:
        Tuple of (derived_key, derivation_parameters)
        
    Raises:
        KeyDerivationError: If derivation fails
        UnsupportedPlatformError: If Scrypt is not supported
    """
    if not CRYPTOGRAPHY_AVAILABLE:
        raise UnsupportedPlatformError(
            "Cryptography package required for Scrypt",
            "CRYPTOGRAPHY_UNAVAILABLE"
        )
    
    # Convert password to bytes if string
    if isinstance(password, str):
        password_bytes = password.encode('utf-8')
    elif isinstance(password, bytes):
        password_bytes = password
    else:
        raise KeyDerivationError(
            "Password must be string or bytes",
            "INVALID_PASSWORD_TYPE"
        )
    
    if len(password_bytes) == 0:
        raise KeyDerivationError(
            "Password cannot be empty",
            "EMPTY_PASSWORD"
        )
    
    # Validate Scrypt parameters
    if n <= 0 or (n & (n - 1)) != 0:  # Check if n is power of 2
        raise KeyDerivationError(
            "N parameter must be a positive power of 2",
            "INVALID_SCRYPT_N"
        )
    
    if r <= 0:
        raise KeyDerivationError(
            "R parameter must be positive",
            "INVALID_SCRYPT_R"
        )
    
    if p <= 0:
        raise KeyDerivationError(
            "P parameter must be positive",
            "INVALID_SCRYPT_P"
        )
    
    if length <= 0:
        raise KeyDerivationError(
            f"Invalid key length: {length}",
            "INVALID_KEY_LENGTH"
        )
    
    try:
        # Generate salt if not provided
        if salt is None:
            salt = _generate_salt()
        
        # Derive key using Scrypt
        scrypt = Scrypt(
            length=length,
            salt=salt,
            n=n,
            r=r,
            p=p,
        )
        
        derived_key = scrypt.derive(password_bytes)
        
        # Create parameters object (store Scrypt params in iterations field as JSON)
        import json
        scrypt_params = json.dumps({'n': n, 'r': r, 'p': p})
        params = DerivationParameters(
            algorithm='Scrypt',
            salt=salt,
            iterations=None,  # Use info field for Scrypt parameters
            info=scrypt_params.encode('utf-8'),
            length=length,
            hash_algorithm='N/A'  # Scrypt doesn't use separate hash
        )
        
        return derived_key, params
        
    except Exception as e:
        if isinstance(e, (KeyDerivationError, UnsupportedPlatformError)):
            raise
        
        raise KeyDerivationError(
            f"Scrypt derivation failed: {str(e)}",
            "SCRYPT_DERIVATION_FAILED"
        ) from e


def derive_ed25519_key_pair(master_key: bytes,
                           context: str = "Ed25519",
                           derivation_method: str = 'HKDF',
                           **kwargs) -> Tuple[Ed25519KeyPair, DerivationParameters]:
    """
    Derive an Ed25519 key pair from master key material.
    
    Args:
        master_key: Master key material for derivation
        context: Context string for key derivation
        derivation_method: Method to use ('HKDF', 'PBKDF2', 'Scrypt')
        **kwargs: Additional parameters for the derivation method
        
    Returns:
        Tuple of (Ed25519KeyPair, derivation_parameters)
        
    Raises:
        KeyDerivationError: If derivation fails
        UnsupportedPlatformError: If required algorithms are not supported
    """
    if derivation_method == 'HKDF':
        # For HKDF, use master_key directly
        info = kwargs.get('info', _generate_info(f"DataFold-{context}"))
        derived_key, params = derive_key_hkdf(
            master_key=master_key,
            salt=kwargs.get('salt'),
            info=info,
            length=ED25519_PRIVATE_KEY_LENGTH,
            hash_algorithm=kwargs.get('hash_algorithm', 'SHA256')
        )
    elif derivation_method == 'PBKDF2':
        # For PBKDF2, treat master_key as password
        derived_key, params = derive_key_pbkdf2(
            password=master_key,
            salt=kwargs.get('salt'),
            iterations=kwargs.get('iterations', PBKDF2_ITERATIONS),
            length=ED25519_PRIVATE_KEY_LENGTH,
            hash_algorithm=kwargs.get('hash_algorithm', 'SHA256')
        )
    elif derivation_method == 'Scrypt':
        # For Scrypt, treat master_key as password
        derived_key, params = derive_key_scrypt(
            password=master_key,
            salt=kwargs.get('salt'),
            n=kwargs.get('n', SCRYPT_N),
            r=kwargs.get('r', SCRYPT_R),
            p=kwargs.get('p', SCRYPT_P),
            length=ED25519_PRIVATE_KEY_LENGTH
        )
    else:
        raise KeyDerivationError(
            f"Unsupported derivation method: {derivation_method}",
            "UNSUPPORTED_DERIVATION_METHOD"
        )
    
    try:
        # Create Ed25519 key pair from derived key material
        private_key_obj = Ed25519PrivateKey.from_private_bytes(derived_key)
        public_key_obj = private_key_obj.public_key()
        
        # Extract raw bytes
        from cryptography.hazmat.primitives import serialization
        private_key_bytes = private_key_obj.private_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PrivateFormat.Raw,
            encryption_algorithm=serialization.NoEncryption()
        )
        
        public_key_bytes = public_key_obj.public_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PublicFormat.Raw
        )
        
        key_pair = Ed25519KeyPair(
            private_key=private_key_bytes,
            public_key=public_key_bytes
        )
        
        return key_pair, params
        
    except Exception as e:
        raise KeyDerivationError(
            f"Failed to create Ed25519 key pair from derived material: {str(e)}",
            "KEY_PAIR_CREATION_FAILED"
        ) from e


def verify_derivation(master_key: bytes, 
                     derived_key: bytes,
                     params: DerivationParameters) -> bool:
    """
    Verify that a derived key was correctly derived from master key with given parameters.
    
    Args:
        master_key: Original master key
        derived_key: Allegedly derived key
        params: Derivation parameters used
        
    Returns:
        bool: True if derivation is correct, False otherwise
    """
    try:
        if params.algorithm == 'HKDF':
            test_key, _ = derive_key_hkdf(
                master_key=master_key,
                salt=params.salt,
                info=params.info,
                length=params.length,
                hash_algorithm=params.hash_algorithm
            )
        elif params.algorithm == 'PBKDF2':
            test_key, _ = derive_key_pbkdf2(
                password=master_key,
                salt=params.salt,
                iterations=params.iterations,
                length=params.length,
                hash_algorithm=params.hash_algorithm
            )
        elif params.algorithm == 'Scrypt':
            # Parse Scrypt parameters from info field
            import json
            scrypt_params = json.loads(params.info.decode('utf-8'))
            test_key, _ = derive_key_scrypt(
                password=master_key,
                salt=params.salt,
                n=scrypt_params['n'],
                r=scrypt_params['r'],
                p=scrypt_params['p'],
                length=params.length
            )
        else:
            return False
        
        # Constant-time comparison
        return secrets.compare_digest(derived_key, test_key)
        
    except Exception:
        return False


def export_derivation_parameters(params: DerivationParameters) -> Dict[str, str]:
    """
    Export derivation parameters for storage/transmission.
    
    Args:
        params: Derivation parameters to export
        
    Returns:
        dict: Serializable parameters
    """
    exported = {
        'algorithm': params.algorithm,
        'salt': base64.b64encode(params.salt).decode('ascii'),
        'length': str(params.length),
        'hash_algorithm': params.hash_algorithm,
    }
    
    if params.info is not None:
        exported['info'] = base64.b64encode(params.info).decode('ascii')
    
    if params.iterations is not None:
        exported['iterations'] = str(params.iterations)
    
    return exported


def import_derivation_parameters(exported: Dict[str, str]) -> DerivationParameters:
    """
    Import derivation parameters from storage.
    
    Args:
        exported: Exported parameters dictionary
        
    Returns:
        DerivationParameters: Reconstructed parameters
        
    Raises:
        KeyDerivationError: If import fails
    """
    try:
        params = DerivationParameters(
            algorithm=exported['algorithm'],
            salt=base64.b64decode(exported['salt']),
            length=int(exported['length']),
            hash_algorithm=exported['hash_algorithm'],
        )
        
        if 'info' in exported:
            params.info = base64.b64decode(exported['info'])
        
        if 'iterations' in exported:
            params.iterations = int(exported['iterations'])
        
        return params
        
    except Exception as e:
        raise KeyDerivationError(
            f"Failed to import derivation parameters: {str(e)}",
            "PARAMETER_IMPORT_FAILED"
        ) from e