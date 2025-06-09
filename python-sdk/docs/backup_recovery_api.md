# Key Backup and Recovery API

This document describes the encrypted key backup and recovery functionality in the DataFold Python SDK.

## Overview

The backup and recovery system provides secure export and import of Ed25519 key pairs using user-provided passphrases and industry-standard encryption. The system follows the backup format guidelines from task 10-1-3 research and supports multiple export formats for cross-platform compatibility.

## Key Features

- **Strong Encryption**: Uses XChaCha20-Poly1305 or AES-GCM for authenticated encryption
- **Robust Key Derivation**: Supports Argon2id, Scrypt, and PBKDF2 for passphrase-based key derivation
- **Multiple Formats**: JSON and binary backup formats
- **Cross-Platform**: Compatible backups that work across different platforms
- **Integrity Verification**: Built-in tampering detection and key integrity checks
- **Secure Memory Handling**: Best-effort sensitive data clearing

## Basic Usage

### Exporting Keys

```python
from datafold_sdk.crypto import generate_key_pair, export_key_to_file

# Generate a key pair
key_pair = generate_key_pair()

# Export to encrypted backup file
passphrase = "YourSecurePassphrase123!"
key_id = "my-signing-key"
backup_file = "my_key_backup.json"

metadata = export_key_to_file(
    key_pair=key_pair,
    passphrase=passphrase,
    key_id=key_id,
    file_path=backup_file,
    export_format='json'
)

print(f"Key exported to {backup_file}")
print(f"Backup created: {metadata.created}")
```

### Importing Keys

```python
from datafold_sdk.crypto import import_key_from_file

# Import from encrypted backup file
passphrase = "YourSecurePassphrase123!"
backup_file = "my_key_backup.json"

key_pair, metadata = import_key_from_file(
    file_path=backup_file,
    passphrase=passphrase,
    verify_integrity=True
)

print(f"Key imported: {metadata.key_id}")
print(f"Created: {metadata.created}")
print(f"Algorithm: {metadata.algorithm}")
```

## Advanced Usage

### Using KeyBackupManager Directly

For more control over the backup process, you can use the `KeyBackupManager` class directly:

```python
from datafold_sdk.crypto import KeyBackupManager, generate_key_pair

# Create a backup manager with specific preferences
manager = KeyBackupManager(
    preferred_kdf='argon2id',
    preferred_encryption='xchacha20-poly1305'
)

# Generate a key pair
key_pair = generate_key_pair()

# Export as JSON string
backup_json = manager.export_key(
    key_pair=key_pair,
    passphrase="StrongPassphrase123!",
    key_id="advanced-key",
    export_format='json',
    kdf_algorithm='scrypt',
    encryption_algorithm='aes-gcm'
)

# Export as binary data
backup_binary = manager.export_key(
    key_pair=key_pair,
    passphrase="StrongPassphrase123!",
    key_id="advanced-key",
    export_format='binary'
)

# Import from string or bytes
imported_pair, metadata = manager.import_key(
    backup_data=backup_json,
    passphrase="StrongPassphrase123!",
    verify_integrity=True
)
```

### Custom Backup Processing

```python
import json
from datafold_sdk.crypto import KeyBackupManager, generate_key_pair

manager = KeyBackupManager()
key_pair = generate_key_pair()

# Export to string
backup_data = manager.export_key(
    key_pair=key_pair,
    passphrase="MyPassphrase123!",
    key_id="custom-key",
    export_format='json'
)

# Parse backup metadata without importing
backup_dict = json.loads(backup_data)
print(f"Backup version: {backup_dict['version']}")
print(f"KDF algorithm: {backup_dict['kdf']}")
print(f"Encryption: {backup_dict['encryption']}")
print(f"Created: {backup_dict['created']}")

# Store backup_data in your preferred storage system
# (database, cloud storage, etc.)

# Later, retrieve and import
imported_pair, metadata = manager.import_key(
    backup_data=backup_data,
    passphrase="MyPassphrase123!"
)
```

## Security Best Practices

### Passphrase Guidelines

Use strong passphrases for backup encryption:

```python
# Good passphrases
strong_passphrases = [
    "MyVerySecure-Passphrase-2025!",
    "correct horse battery staple 123",
    "Tr0ub4dor&3.extended.version",
]

# Avoid weak passphrases
weak_passphrases = [
    "password",
    "123456",
    "qwerty",
    "mypassword"
]
```

### Secure File Handling

```python
import os
from datafold_sdk.crypto import export_key_to_file, import_key_from_file

# Export with secure file permissions
backup_file = "secure_backup.json"
metadata = export_key_to_file(
    key_pair=key_pair,
    passphrase=passphrase,
    key_id="secure-key",
    file_path=backup_file
)

# Verify file permissions (Unix-like systems)
if os.name != 'nt':  # Not Windows
    file_stat = os.stat(backup_file)
    permissions = oct(file_stat.st_mode)[-3:]
    assert permissions == '600', "File should have 600 permissions"

# Import and immediately secure the key
imported_pair, metadata = import_key_from_file(backup_file, passphrase)

# Clear sensitive data when done
from datafold_sdk.crypto import clear_key_material
clear_key_material(imported_pair)
```

### Backup Verification

Always verify backup integrity:

```python
from datafold_sdk.crypto import KeyBackupManager

manager = KeyBackupManager()

# Export key
backup_data = manager.export_key(
    key_pair=original_key_pair,
    passphrase=passphrase,
    key_id="verification-test"
)

# Import with verification enabled (default)
imported_pair, metadata = manager.import_key(
    backup_data=backup_data,
    passphrase=passphrase,
    verify_integrity=True  # This is the default
)

# Verify the keys match
assert imported_pair.private_key == original_key_pair.private_key
assert imported_pair.public_key == original_key_pair.public_key
```

## Backup Format

The backup format follows a standardized JSON structure:

```json
{
  "version": 1,
  "key_id": "user-provided-identifier",
  "algorithm": "Ed25519",
  "kdf": "argon2id",
  "kdf_params": {
    "memory": 65536,
    "iterations": 3,
    "parallelism": 1
  },
  "encryption": "xchacha20-poly1305",
  "salt": "base64-encoded-salt",
  "nonce": "base64-encoded-nonce",
  "ciphertext": "base64-encoded-encrypted-key-data",
  "created": "2025-06-08T22:59:35Z"
}
```

### Field Descriptions

- `version`: Backup format version (currently 1)
- `key_id`: User-provided identifier for the key
- `algorithm`: Key algorithm (always "Ed25519")
- `kdf`: Key derivation function used ("argon2id", "scrypt", or "pbkdf2")
- `kdf_params`: Parameters for the KDF algorithm
- `encryption`: Encryption algorithm ("xchacha20-poly1305" or "aes-gcm")
- `salt`: Random salt for key derivation (base64-encoded)
- `nonce`: Random nonce/IV for encryption (base64-encoded)
- `ciphertext`: Encrypted key data (base64-encoded)
- `created`: ISO 8601 timestamp of backup creation

## Error Handling

### Common Errors

```python
from datafold_sdk.exceptions import BackupError, ValidationError

try:
    # Import with wrong passphrase
    manager.import_key(backup_data, "wrong_passphrase")
except BackupError as e:
    if e.error_code == "DECRYPTION_FAILED":
        print("Incorrect passphrase or corrupted data")
    else:
        print(f"Backup error: {e}")

try:
    # Export with weak passphrase
    manager.export_key(key_pair, "weak", "test-key")
except BackupError as e:
    if e.error_code == "WEAK_PASSPHRASE":
        print("Passphrase is too weak")

try:
    # Import corrupted backup
    manager.import_key("invalid json", passphrase)
except ValidationError as e:
    if e.error_code == "INVALID_BACKUP_JSON":
        print("Backup file is corrupted or invalid")
```

### Error Codes

Common error codes you may encounter:

- `WEAK_PASSPHRASE`: Passphrase doesn't meet security requirements
- `DECRYPTION_FAILED`: Wrong passphrase or corrupted data
- `INVALID_BACKUP_JSON`: Malformed backup data
- `UNSUPPORTED_BACKUP_VERSION`: Backup version not supported
- `BACKUP_FILE_NOT_FOUND`: Backup file doesn't exist
- `INTEGRITY_CHECK_FAILED`: Key integrity verification failed

## Platform Support

### Checking Support

```python
from datafold_sdk.crypto import KeyBackupManager

manager = KeyBackupManager()
support = manager.check_backup_support()

print("Platform Support:")
print(f"Cryptography available: {support['cryptography_available']}")
print(f"Argon2 available: {support['argon2_available']}")
print(f"Supported KDF algorithms: {support['supported_kdf_algorithms']}")
print(f"Supported encryption algorithms: {support['supported_encryption_algorithms']}")
print(f"Current preferences: {support['current_preferences']}")
```

### Cross-Platform Compatibility

Backups created on one platform can be imported on any other supported platform:

```python
# Create backup on Linux
backup_data = manager.export_key(key_pair, passphrase, "cross-platform-key")

# Import on Windows/macOS/other Linux systems
imported_pair, metadata = manager.import_key(backup_data, passphrase)
```

## Integration with Storage System

The backup system integrates with the existing storage system:

```python
from datafold_sdk.crypto import SecureKeyStorage, KeyBackupManager

# Create key and store securely
storage = SecureKeyStorage()
key_pair = generate_key_pair()
storage_metadata = storage.store_key("main-key", key_pair, passphrase="storage_pass")

# Create backup
backup_manager = KeyBackupManager()
backup_data = backup_manager.export_key(
    key_pair=key_pair,
    passphrase="backup_passphrase",  # Different from storage passphrase
    key_id="main-key"
)

# Save backup separately from main storage
with open("backup.json", "w") as f:
    f.write(backup_data)
```

## Recovery Scenarios

### Complete System Recovery

```python
from datafold_sdk.crypto import import_key_from_file, SecureKeyStorage

# After system loss, restore from backup
restored_pair, metadata = import_key_from_file(
    "backup.json",
    "backup_passphrase"
)

# Re-establish in secure storage
storage = SecureKeyStorage()
storage.store_key(metadata.key_id, restored_pair, "new_storage_passphrase")

print(f"Restored key: {metadata.key_id}")
```

### Partial Recovery with Verification

```python
# Restore and verify against known public key
expected_public_key = bytes.fromhex("known_public_key_hex")

restored_pair, metadata = import_key_from_file("backup.json", passphrase)

if restored_pair.public_key == expected_public_key:
    print("Key successfully verified and restored")
else:
    print("WARNING: Restored key doesn't match expected public key")
```

## Performance Considerations

### KDF Algorithm Performance

Different KDF algorithms have different performance characteristics:

```python
import time
from datafold_sdk.crypto import KeyBackupManager

manager = KeyBackupManager()

# Argon2id: Most secure, moderate speed
start = time.time()
backup_argon2 = manager.export_key(
    key_pair, passphrase, "test", 
    kdf_algorithm='argon2id'
)
argon2_time = time.time() - start

# Scrypt: Good security, fast
start = time.time()
backup_scrypt = manager.export_key(
    key_pair, passphrase, "test",
    kdf_algorithm='scrypt'
)
scrypt_time = time.time() - start

# PBKDF2: Legacy compatibility, fastest
start = time.time()
backup_pbkdf2 = manager.export_key(
    key_pair, passphrase, "test",
    kdf_algorithm='pbkdf2'
)
pbkdf2_time = time.time() - start

print(f"Argon2id: {argon2_time:.3f}s")
print(f"Scrypt: {scrypt_time:.3f}s") 
print(f"PBKDF2: {pbkdf2_time:.3f}s")
```

## API Reference

### Classes

#### `KeyBackupManager`

Main class for backup operations.

```python
class KeyBackupManager:
    def __init__(self, preferred_kdf='argon2id', preferred_encryption='xchacha20-poly1305')
    def export_key(self, key_pair, passphrase, key_id, export_format='json', 
                   kdf_algorithm=None, encryption_algorithm=None) -> Union[str, bytes]
    def import_key(self, backup_data, passphrase, verify_integrity=True) -> Tuple[Ed25519KeyPair, BackupMetadata]
    def check_backup_support(self) -> Dict[str, Any]
```

#### `BackupMetadata`

Contains metadata about backup operations.

```python
@dataclass
class BackupMetadata:
    version: int
    key_id: str
    algorithm: str
    kdf: str
    encryption: str
    created: str
    format: str
```

### Functions

#### `get_default_backup_manager() -> KeyBackupManager`

Returns a default backup manager instance.

#### `export_key_to_file(key_pair, passphrase, key_id, file_path, export_format='json') -> BackupMetadata`

Exports a key pair to an encrypted backup file.

#### `import_key_from_file(file_path, passphrase, verify_integrity=True) -> Tuple[Ed25519KeyPair, BackupMetadata]`

Imports a key pair from an encrypted backup file.

## Constants

```python
BACKUP_VERSION = 1
SUPPORTED_EXPORT_FORMATS = ['json', 'binary']
SUPPORTED_KDF_ALGORITHMS = ['argon2id', 'scrypt', 'pbkdf2']
SUPPORTED_ENCRYPTION_ALGORITHMS = ['xchacha20-poly1305', 'aes-gcm']