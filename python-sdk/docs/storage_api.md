# Secure Key Storage API

This document describes the secure key storage functionality in the DataFold Python SDK.

## Overview

The secure storage module provides cross-platform secure storage for Ed25519 private keys using:

1. **OS Keychain** (preferred): macOS Keychain, Windows DPAPI, Linux Secret Service
2. **Encrypted File Storage** (fallback): Strong KDF-encrypted files with restricted permissions

## Basic Usage

### Importing

```python
from datafold_sdk import SecureKeyStorage, generate_key_pair, get_default_storage
```

### Quick Start

```python
# Generate a key pair
key_pair = generate_key_pair()

# Get default storage instance
storage = get_default_storage()

# Store the key securely
metadata = storage.store_key(
    key_id="my_signing_key",
    key_pair=key_pair,
    passphrase="secure_passphrase_123"  # Required for file storage
)

# Retrieve the key
retrieved_key = storage.retrieve_key(
    key_id="my_signing_key",
    passphrase="secure_passphrase_123"
)

# List all stored keys
key_ids = storage.list_keys()

# Delete a key
success = storage.delete_key("my_signing_key")
```

## API Reference

### SecureKeyStorage Class

#### Constructor

```python
SecureKeyStorage(storage_dir=None, use_keyring=True)
```

**Parameters:**
- `storage_dir` (str, optional): Custom directory for encrypted file storage
- `use_keyring` (bool): Whether to use OS keyring when available (default: True)

#### Methods

##### store_key()

```python
store_key(key_id: str, key_pair: Ed25519KeyPair, passphrase: Optional[str] = None) -> StorageMetadata
```

Store an Ed25519 key pair securely.

**Parameters:**
- `key_id`: Unique identifier for the key
- `key_pair`: Ed25519KeyPair object to store
- `passphrase`: Required for encrypted file storage (optional if keyring available)

**Returns:** StorageMetadata with storage information

**Raises:**
- `StorageError`: If storage fails
- `UnsupportedPlatformError`: If no storage method available

##### retrieve_key()

```python
retrieve_key(key_id: str, passphrase: Optional[str] = None) -> Optional[Ed25519KeyPair]
```

Retrieve an Ed25519 key pair from storage.

**Parameters:**
- `key_id`: Unique identifier for the key
- `passphrase`: Required for encrypted file storage

**Returns:** Ed25519KeyPair or None if key not found

**Raises:**
- `StorageError`: If retrieval fails

##### delete_key()

```python
delete_key(key_id: str) -> bool
```

Delete a key from storage.

**Parameters:**
- `key_id`: Unique identifier for the key

**Returns:** True if key was deleted, False if not found

**Raises:**
- `StorageError`: If deletion fails

##### list_keys()

```python
list_keys() -> List[str]
```

List all stored key IDs.

**Returns:** List of key ID strings

##### check_storage_availability()

```python
check_storage_availability() -> Dict[str, Any]
```

Check availability of storage methods.

**Returns:** Dictionary with availability information

### StorageMetadata Class

Contains metadata about stored keys:

```python
@dataclass
class StorageMetadata:
    key_id: str          # Unique identifier
    storage_type: str    # 'keyring' or 'file'
    created_at: str      # ISO timestamp
    last_accessed: str   # ISO timestamp
    algorithm: str       # 'Ed25519'
```

## Security Features

### Key Isolation

- Each key is encrypted with its own passphrase-derived key
- Keys cannot be accessed by other applications
- Platform-specific secure storage integration

### Encryption

- **Scrypt KDF**: Strong key derivation with configurable parameters
- **Fernet encryption**: Authenticated encryption with automatic key rotation
- **Random salts**: Each encryption uses a unique random salt

### File Permissions

- Key files have restrictive permissions (0600 - owner read/write only)
- Storage directories have secure permissions (0700 - owner access only)

### Cross-Platform Support

| Platform | Primary Storage | Fallback |
|----------|----------------|----------|
| macOS | Keychain | Encrypted files |
| Windows | DPAPI | Encrypted files |
| Linux | Secret Service | Encrypted files |

## Error Handling

The storage module uses specific exception types:

- `StorageError`: General storage-related errors
- `UnsupportedPlatformError`: Platform features not available
- `Ed25519KeyError`: Key-specific errors
- `ValidationError`: Validation failures

## Examples

### Custom Storage Directory

```python
import tempfile
from pathlib import Path

# Use custom storage directory
custom_dir = Path.home() / "my_app_keys"
storage = SecureKeyStorage(storage_dir=str(custom_dir))

key_pair = generate_key_pair()
storage.store_key("app_key", key_pair, "my_passphrase")
```

### Keyring-Only Storage

```python
# Disable file storage fallback (keyring only)
storage = SecureKeyStorage(use_keyring=True)

try:
    # This will fail if keyring is not available
    storage.store_key("keyring_only", key_pair)
except StorageError as e:
    print(f"Keyring storage failed: {e}")
```

### Checking Storage Availability

```python
storage = SecureKeyStorage()
availability = storage.check_storage_availability()

print(f"Keyring available: {availability['keyring_available']}")
print(f"Keyring functional: {availability.get('keyring_functional', False)}")
print(f"File storage available: {availability['file_storage_available']}")
print(f"Platform: {availability['platform']}")
```

### Key Lifecycle Management

```python
# Generate and store multiple keys
key_pairs = []
for i in range(3):
    key_pair = generate_key_pair()
    key_id = f"signing_key_{i}"
    storage.store_key(key_id, key_pair, f"passphrase_{i}")
    key_pairs.append((key_id, key_pair))

# List all keys
stored_keys = storage.list_keys()
print(f"Stored keys: {stored_keys}")

# Verify all keys can be retrieved
for key_id, original_key in key_pairs:
    retrieved_key = storage.retrieve_key(key_id, f"passphrase_{stored_keys.index(key_id)}")
    assert retrieved_key.private_key == original_key.private_key
    print(f"Verified key: {key_id}")

# Clean up
for key_id, _ in key_pairs:
    storage.delete_key(key_id)
```

## Best Practices

1. **Use Strong Passphrases**: Use long, random passphrases for file storage
2. **Secure Passphrase Storage**: Don't hardcode passphrases in source code
3. **Key Rotation**: Regularly rotate signing keys
4. **Error Handling**: Always handle storage exceptions appropriately
5. **Testing**: Verify storage functionality in your deployment environment

## Integration with Ed25519 Keys

The storage module integrates seamlessly with the Ed25519 key generation:

```python
from datafold_sdk import generate_key_pair, SecureKeyStorage

# Generate key
key_pair = generate_key_pair()

# Store securely
storage = SecureKeyStorage()
metadata = storage.store_key("my_key", key_pair, "secure_passphrase")

# Use for signing
message = b"important data to sign"
# Note: Actual signing implementation would be in a separate module
```

## Security Considerations

1. **Platform Dependencies**: Keyring availability varies by platform and environment
2. **Passphrase Security**: File storage security depends on passphrase strength
3. **Memory Safety**: Private keys are cleared from memory when possible
4. **File System Security**: Ensure underlying file system supports proper permissions
5. **Backup Considerations**: Consider secure backup strategies for encrypted key files