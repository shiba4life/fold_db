# Key Derivation and Rotation API

This document describes the key derivation and rotation functionality provided by the DataFold Python SDK.

## Overview

The DataFold Python SDK provides comprehensive key derivation and rotation capabilities for secure key management:

- **Key Derivation**: Generate derived keys from master keys using cryptographically secure methods (HKDF, PBKDF2, Scrypt)
- **Key Rotation**: Manage key lifecycles with versioning, automatic rotation, and secure key replacement
- **Integration**: Seamless integration with existing Ed25519 key generation and secure storage systems

## Key Derivation

### Supported Algorithms

The SDK supports three industry-standard key derivation functions:

1. **HKDF (HMAC-based Key Derivation Function)** - RFC 5869
   - Best for deriving multiple keys from a single master key
   - Fast and suitable for high-frequency operations
   - Recommended for most use cases

2. **PBKDF2 (Password-Based Key Derivation Function 2)** - RFC 2898
   - Designed for password-based key derivation
   - Configurable iteration count for adjustable security
   - Good for user-password derived keys

3. **Scrypt** - RFC 7914
   - Memory-hard function resistant to hardware attacks
   - Higher security but more resource intensive
   - Recommended for high-security applications

### Basic Key Derivation

```python
from datafold_sdk.crypto import (
    derive_key_hkdf,
    derive_key_pbkdf2,
    derive_key_scrypt,
    derive_ed25519_key_pair
)

# Derive a raw key using HKDF
master_key = b"your_master_key_material"
derived_key, params = derive_key_hkdf(master_key)

# Derive an Ed25519 key pair directly
key_pair, derivation_params = derive_ed25519_key_pair(
    master_key=master_key,
    context="user_signing_key",
    derivation_method='HKDF'
)
```

### Advanced Derivation with Custom Parameters

```python
# HKDF with custom parameters
derived_key, params = derive_key_hkdf(
    master_key=master_key,
    salt=b"custom_salt",
    info=b"application_context",
    length=64,  # 64-byte output
    hash_algorithm='SHA384'
)

# PBKDF2 with custom iterations
derived_key, params = derive_key_pbkdf2(
    password="user_password",
    salt=b"unique_salt",
    iterations=150000,  # Higher security
    hash_algorithm='SHA512'
)

# Scrypt with custom parameters
derived_key, params = derive_key_scrypt(
    password="user_password",
    n=65536,  # CPU cost factor
    r=8,      # Memory cost factor
    p=1       # Parallelization factor
)
```

### Verifying Derivation

```python
from datafold_sdk.crypto import verify_derivation

# Verify that a key was correctly derived
is_valid = verify_derivation(
    master_key=original_master_key,
    derived_key=alleged_derived_key,
    params=derivation_parameters
)
```

### Parameter Serialization

```python
from datafold_sdk.crypto import (
    export_derivation_parameters,
    import_derivation_parameters
)

# Export parameters for storage
exported = export_derivation_parameters(params)
# exported is a JSON-serializable dictionary

# Import parameters from storage
restored_params = import_derivation_parameters(exported)
```

## Key Rotation

### Basic Rotation Setup

```python
from datafold_sdk.crypto import (
    KeyRotationManager,
    RotationPolicy,
    generate_key_pair,
    get_default_storage
)

# Set up rotation manager
storage = get_default_storage()
rotation_manager = KeyRotationManager(storage)

# Create rotation policy
policy = RotationPolicy(
    rotation_interval_days=90,  # Rotate every 90 days
    max_versions=5,             # Keep 5 versions
    auto_cleanup_expired=True,  # Auto-cleanup old versions
    derivation_method='HKDF'    # Use HKDF for new keys
)

# Initialize key rotation
initial_key = generate_key_pair()
metadata = rotation_manager.initialize_key_rotation(
    key_id="user_signing_key",
    initial_key_pair=initial_key,
    policy=policy,
    passphrase="secure_passphrase"
)
```

### Performing Key Rotation

```python
# Manual rotation
new_key, updated_metadata = rotation_manager.rotate_key(
    key_id="user_signing_key",
    passphrase="secure_passphrase",
    rotation_reason="Scheduled maintenance"
)

# Rotation with custom master key
custom_master = b"specific_master_key"
new_key, metadata = rotation_manager.rotate_key(
    key_id="user_signing_key",
    master_key=custom_master,
    passphrase="secure_passphrase",
    derivation_method='PBKDF2'  # Override policy
)
```

### Accessing Keys

```python
# Get current active key
current_key = rotation_manager.get_current_key(
    key_id="user_signing_key",
    passphrase="secure_passphrase"
)

# Get specific version
version_2_key = rotation_manager.get_key_version(
    key_id="user_signing_key",
    version=2,
    passphrase="secure_passphrase"
)

# List all versions
versions = rotation_manager.list_key_versions("user_signing_key")
for version in versions:
    print(f"Version {version.version}: Active={version.is_active}")
```

### Rotation Management

```python
# Check if rotation is due
if rotation_manager.check_rotation_due("user_signing_key"):
    print("Key rotation is due!")

# Get rotation metadata
metadata = rotation_manager.get_rotation_metadata("user_signing_key")
print(f"Current version: {metadata.current_version}")
print(f"Total rotations: {metadata.rotation_count}")
print(f"Next rotation: {metadata.next_rotation}")

# Manually expire a version
rotation_manager.expire_version("user_signing_key", version=1)

# List all managed keys
managed_keys = rotation_manager.list_managed_keys()
```

### Complete Cleanup

```python
# Delete key and all versions completely
rotation_manager.delete_key_completely("user_signing_key")
```

## Integration Examples

### Web Application User Keys

```python
from datafold_sdk.crypto import (
    KeyRotationManager,
    RotationPolicy,
    derive_ed25519_key_pair,
    get_default_storage
)

class UserKeyManager:
    def __init__(self):
        self.storage = get_default_storage()
        self.rotation_manager = KeyRotationManager(self.storage)
        
        # Policy for user signing keys
        self.user_policy = RotationPolicy(
            rotation_interval_days=365,  # Annual rotation
            max_versions=3,              # Keep 3 versions
            derivation_method='HKDF'
        )
    
    def create_user_key(self, user_id: str, password: str):
        """Create initial key for user"""
        # Derive key from user password
        master_key = password.encode('utf-8')
        key_pair, _ = derive_ed25519_key_pair(
            master_key=master_key,
            context=f"user_{user_id}_signing",
            derivation_method='PBKDF2'
        )
        
        # Initialize rotation
        metadata = self.rotation_manager.initialize_key_rotation(
            key_id=f"user_{user_id}",
            initial_key_pair=key_pair,
            policy=self.user_policy,
            passphrase=password
        )
        
        return key_pair, metadata
    
    def get_user_key(self, user_id: str, password: str):
        """Get current key for user"""
        return self.rotation_manager.get_current_key(
            key_id=f"user_{user_id}",
            passphrase=password
        )
    
    def rotate_user_key(self, user_id: str, password: str):
        """Rotate user's key"""
        return self.rotation_manager.rotate_key(
            key_id=f"user_{user_id}",
            passphrase=password,
            rotation_reason="User-requested rotation"
        )
```

### Automated Rotation Service

```python
import asyncio
from datetime import datetime, timezone

class AutoRotationService:
    def __init__(self, rotation_manager: KeyRotationManager):
        self.rotation_manager = rotation_manager
    
    async def check_and_rotate_keys(self):
        """Check all managed keys and rotate if due"""
        managed_keys = self.rotation_manager.list_managed_keys()
        
        for key_id in managed_keys:
            if self.rotation_manager.check_rotation_due(key_id):
                try:
                    # In real implementation, you'd need to get the passphrase
                    # securely (e.g., from a key management service)
                    passphrase = self.get_key_passphrase(key_id)
                    
                    new_key, metadata = self.rotation_manager.rotate_key(
                        key_id=key_id,
                        passphrase=passphrase,
                        rotation_reason="Automated rotation"
                    )
                    
                    print(f"Rotated key {key_id} to version {metadata.current_version}")
                    
                    # Notify applications of key rotation
                    await self.notify_key_rotation(key_id, metadata.current_version)
                    
                except Exception as e:
                    print(f"Failed to rotate key {key_id}: {e}")
    
    def get_key_passphrase(self, key_id: str) -> str:
        # Implementation would retrieve passphrase from secure storage
        pass
    
    async def notify_key_rotation(self, key_id: str, new_version: int):
        # Implementation would notify applications
        pass
```

## Security Best Practices

### 1. Master Key Management
- Use high-entropy master keys (>256 bits)
- Derive master keys from user passwords using PBKDF2/Scrypt
- Never store master keys in plaintext

### 2. Derivation Method Selection
- **HKDF**: For high-performance applications, system-generated keys
- **PBKDF2**: For user password-derived keys (minimum 100,000 iterations)
- **Scrypt**: For high-security scenarios where resistance to hardware attacks is critical

### 3. Rotation Policies
- Set appropriate rotation intervals based on key usage and threat model
- Keep sufficient versions for backward compatibility
- Use automatic cleanup to prevent key accumulation

### 4. Storage Security
- Always use strong passphrases for key encryption
- Leverage OS keychain when available
- Set proper file permissions for encrypted storage

### 5. Context Separation
- Use different contexts/info parameters for different key purposes
- Avoid key reuse across different applications or users

## Error Handling

```python
from datafold_sdk.exceptions import (
    KeyDerivationError,
    KeyRotationError,
    UnsupportedPlatformError
)

try:
    derived_key, params = derive_key_hkdf(master_key)
except UnsupportedPlatformError:
    print("Cryptography package not available")
except KeyDerivationError as e:
    print(f"Derivation failed: {e.error_code} - {e}")

try:
    new_key, metadata = rotation_manager.rotate_key("key_id")
except KeyRotationError as e:
    print(f"Rotation failed: {e.error_code} - {e}")
```

## Platform Compatibility

Check platform support before using specific features:

```python
from datafold_sdk.crypto import check_derivation_support

support = check_derivation_support()
if support['hkdf_supported']:
    # Use HKDF
    pass
elif support['pbkdf2_supported']:
    # Fall back to PBKDF2
    pass
else:
    # Handle unsupported platform
    pass
```

## Performance Considerations

### Key Derivation Performance
- **HKDF**: Fastest, suitable for real-time operations
- **PBKDF2**: Moderate, scalable with iteration count
- **Scrypt**: Slowest, highest security

### Rotation Overhead
- Store frequently accessed keys in memory cache
- Use async rotation for multiple keys
- Monitor storage space for version accumulation

### Memory Management
- Clear sensitive key material after use
- Use context managers for automatic cleanup
- Monitor memory usage in long-running applications

For more information, see the individual module documentation and example code in the `examples/` directory.