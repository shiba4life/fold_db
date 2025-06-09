# Secure Storage API Documentation

## Overview

The DataFold JavaScript SDK provides secure client-side storage for Ed25519 private keys using browser IndexedDB with strong encryption. Private keys are encrypted using AES-GCM with PBKDF2 key derivation before storage, ensuring they are never stored in plaintext.

## Features

- **🔒 Strong Encryption**: AES-GCM with 256-bit keys derived using PBKDF2 (100,000 iterations)
- **🏗️ Origin Isolation**: Keys are isolated per browser origin and not accessible to other websites
- **📋 Metadata Management**: Store key names, descriptions, and tags for organization
- **🛡️ Input Validation**: Comprehensive validation of passphrases, key IDs, and metadata
- **⚡ Efficient Operations**: Optimized for performance with proper error handling
- **🧹 Memory Management**: Secure memory clearing utilities to prevent key leaks

## Quick Start

```javascript
import { 
  generateKeyPair, 
  createStorage, 
  isStorageSupported,
  generateKeyId 
} from '@datafold/js-sdk';

// Check if storage is supported
const { supported, reasons } = isStorageSupported();
if (!supported) {
  console.error('Storage not supported:', reasons);
  return;
}

// Create storage instance
const storage = await createStorage();

// Generate a key pair
const keyPair = await generateKeyPair();

// Generate a unique key ID
const keyId = generateKeyId('user-key');

// Store the key with encryption
await storage.storeKeyPair(
  keyId, 
  keyPair, 
  'SecurePassphrase123!',
  {
    name: 'My Key',
    description: 'Primary signing key',
    tags: ['signing', 'primary']
  }
);

// Retrieve the key later
const retrievedKey = await storage.retrieveKeyPair(keyId, 'SecurePassphrase123!');

// List all stored keys
const keyList = await storage.listKeys();

// Clean up
await storage.close();
```

## API Reference

### Storage Interface

#### `createStorage(options?: StorageOptions): Promise<IndexedDBKeyStorage>`

Creates a new storage instance with IndexedDB backend.

**Parameters:**
- `options` (optional): Storage configuration options

**Returns:** Promise resolving to storage instance

#### `isStorageSupported(): { supported: boolean; reasons: string[] }`

Checks if secure storage is supported in the current environment.

**Returns:** Object with support status and any blocking reasons

### IndexedDBKeyStorage Class

#### `storeKeyPair(keyId, keyPair, passphrase, metadata?): Promise<void>`

Stores an encrypted key pair in IndexedDB.

**Parameters:**
- `keyId` (string): Unique identifier for the key (max 100 chars)
- `keyPair` (Ed25519KeyPair): The key pair to store
- `passphrase` (string): Encryption passphrase (min 8 chars)
- `metadata` (optional): Key metadata object

**Throws:** `StorageError` if validation fails or storage operation fails

```javascript
await storage.storeKeyPair('my-key-1', keyPair, 'MySecurePass123!', {
  name: 'Primary Key',
  description: 'Main signing key for user authentication',
  tags: ['primary', 'auth', 'signing']
});
```

#### `retrieveKeyPair(keyId, passphrase): Promise<Ed25519KeyPair>`

Retrieves and decrypts a key pair from storage.

**Parameters:**
- `keyId` (string): The key identifier
- `passphrase` (string): Decryption passphrase

**Returns:** Promise resolving to the decrypted key pair

**Throws:** `StorageError` if key not found or incorrect passphrase

```javascript
const keyPair = await storage.retrieveKeyPair('my-key-1', 'MySecurePass123!');
// Use keyPair for cryptographic operations
// Remember to clear sensitive data when done
clearKeyMaterial(keyPair);
```

#### `deleteKeyPair(keyId): Promise<void>`

Permanently deletes a key pair from storage.

**Parameters:**
- `keyId` (string): The key identifier to delete

```javascript
await storage.deleteKeyPair('my-key-1');
```

#### `listKeys(): Promise<Array<{ id: string; metadata: StoredKeyMetadata }>>`

Lists all stored keys with their metadata (without private keys).

**Returns:** Promise resolving to array of key metadata

```javascript
const keys = await storage.listKeys();
keys.forEach(key => {
  console.log(`Key: ${key.id}, Name: ${key.metadata.name}`);
});
```

#### `keyExists(keyId): Promise<boolean>`

Checks if a key exists in storage.

**Parameters:**
- `keyId` (string): The key identifier to check

**Returns:** Promise resolving to boolean

```javascript
const exists = await storage.keyExists('my-key-1');
if (exists) {
  console.log('Key found in storage');
}
```

#### `getStorageInfo(): Promise<{ used: number; available: number | null }>`

Gets storage usage information.

**Returns:** Promise resolving to storage usage data

```javascript
const { used, available } = await storage.getStorageInfo();
console.log(`Storage used: ${used} bytes of ${available} bytes available`);
```

#### `clearAllKeys(): Promise<void>`

⚠️ **Dangerous operation** - Permanently deletes all stored keys.

```javascript
if (confirm('Delete all keys? This cannot be undone!')) {
  await storage.clearAllKeys();
}
```

#### `close(): Promise<void>`

Closes the storage connection and cleans up resources.

```javascript
await storage.close();
```

## Utility Functions

### Key ID Management

#### `generateKeyId(prefix?: string): string`

Generates a unique key identifier with optional prefix.

```javascript
const keyId = generateKeyId('user'); // Returns: user_abc123_def456
```

#### `validateKeyId(keyId): { valid: boolean; reason?: string }`

Validates a key ID format and safety.

```javascript
const { valid, reason } = validateKeyId('my-key-1');
if (!valid) {
  console.error('Invalid key ID:', reason);
}
```

#### `sanitizeKeyId(keyId): string`

Sanitizes a key ID by removing unsafe characters.

```javascript
const safe = sanitizeKeyId('user<>key'); // Returns: user__key
```

### Passphrase Validation

#### `validatePassphrase(passphrase): { valid: boolean; issues: string[] }`

Validates passphrase strength and provides improvement suggestions.

```javascript
const { valid, issues } = validatePassphrase('mypass');
if (!valid) {
  console.log('Passphrase issues:', issues);
  // Issues: ['Must be at least 8 characters long', 'Should contain uppercase letters', ...]
}
```

### Metadata Validation

#### `validateMetadata(metadata): { valid: boolean; issues: string[] }`

Validates metadata object structure and constraints.

```javascript
const { valid, issues } = validateMetadata({
  name: 'My Key',
  description: 'A test key',
  tags: ['test', 'example']
});
```

## Data Types

### StoredKeyMetadata

```typescript
interface StoredKeyMetadata {
  /** Human-readable name for the key */
  name: string;
  /** Optional description */
  description: string;
  /** ISO timestamp when key was created */
  created: string;
  /** ISO timestamp when key was last accessed */
  lastAccessed: string;
  /** Optional tags for key organization */
  tags: string[];
}
```

### StorageOptions

```typescript
interface StorageOptions {
  /** Database name (optional, defaults to 'DataFoldKeyStorage') */
  dbName?: string;
  /** Enable debugging logs */
  debug?: boolean;
}
```

### StorageError

```typescript
class StorageError extends Error {
  constructor(message: string, public readonly code: string);
}
```

Common error codes:
- `INDEXEDDB_NOT_SUPPORTED` - IndexedDB not available
- `WEBCRYPTO_NOT_AVAILABLE` - WebCrypto API not available
- `INVALID_KEY_ID` - Key ID validation failed
- `WEAK_PASSPHRASE` - Passphrase doesn't meet requirements
- `KEY_NOT_FOUND` - Requested key doesn't exist
- `DECRYPTION_FAILED` - Incorrect passphrase or corrupted data
- `STORAGE_FAILED` - Database operation failed

## Security Considerations

### Encryption Details

- **Algorithm**: AES-GCM with 256-bit keys
- **Key Derivation**: PBKDF2 with 100,000 iterations using SHA-256
- **IV**: Random 12-byte initialization vector per encryption
- **Salt**: Random 16-byte salt per key derivation

### Best Practices

1. **Strong Passphrases**: Use passphrases with at least 12 characters, mixed case, numbers, and symbols
2. **Memory Management**: Always call `clearKeyMaterial()` when done with keys
3. **Error Handling**: Implement proper error handling for all storage operations
4. **Regular Backups**: Consider implementing backup/export functionality for key recovery
5. **Access Control**: Store keys only when necessary and minimize access duration

### Limitations

- **Browser Storage**: Keys are tied to browser origin and may be lost if browser data is cleared
- **No Synchronization**: Keys are not synchronized across devices or browsers
- **Quota Limits**: Subject to browser storage quotas (typically several MB minimum)
- **Platform Dependent**: Requires modern browser with IndexedDB and WebCrypto support

## Browser Compatibility

| Feature | Chrome | Firefox | Safari | Edge |
|---------|--------|---------|---------|------|
| IndexedDB | ✅ 23+ | ✅ 10+ | ✅ 8+ | ✅ 12+ |
| WebCrypto | ✅ 37+ | ✅ 34+ | ✅ 7+ | ✅ 12+ |
| AES-GCM | ✅ 37+ | ✅ 34+ | ✅ 7+ | ✅ 79+ |
| PBKDF2 | ✅ 37+ | ✅ 34+ | ✅ 8+ | ✅ 79+ |

## Examples

### Basic Usage

```javascript
import { generateKeyPair, createStorage } from '@datafold/js-sdk';

async function basicExample() {
  // Create storage
  const storage = await createStorage();
  
  // Generate and store key
  const keyPair = await generateKeyPair();
  await storage.storeKeyPair('user-key', keyPair, 'MyPassword123!');
  
  // Retrieve key later
  const retrieved = await storage.retrieveKeyPair('user-key', 'MyPassword123!');
  
  // Clean up
  await storage.close();
}
```

### Key Management

```javascript
async function keyManagementExample() {
  const storage = await createStorage();
  
  // Store multiple keys with metadata
  for (let i = 0; i < 3; i++) {
    const keyPair = await generateKeyPair();
    await storage.storeKeyPair(`key-${i}`, keyPair, 'Password123!', {
      name: `Key ${i + 1}`,
      description: `Test key number ${i + 1}`,
      tags: ['test', `key-${i}`]
    });
  }
  
  // List and manage keys
  const keys = await storage.listKeys();
  console.log(`Stored ${keys.length} keys`);
  
  // Filter keys by tag
  const testKeys = keys.filter(k => k.metadata.tags.includes('test'));
  
  // Delete old keys
  for (const key of testKeys) {
    const created = new Date(key.metadata.created);
    const dayAgo = new Date(Date.now() - 24 * 60 * 60 * 1000);
    
    if (created < dayAgo) {
      await storage.deleteKeyPair(key.id);
      console.log(`Deleted old key: ${key.id}`);
    }
  }
  
  await storage.close();
}
```

### Error Handling

```javascript
async function errorHandlingExample() {
  const storage = await createStorage();
  
  try {
    // Attempt to retrieve non-existent key
    await storage.retrieveKeyPair('non-existent', 'password');
  } catch (error) {
    if (error instanceof StorageError) {
      switch (error.code) {
        case 'KEY_NOT_FOUND':
          console.log('Key does not exist');
          break;
        case 'DECRYPTION_FAILED':
          console.log('Incorrect passphrase');
          break;
        default:
          console.error('Storage error:', error.message);
      }
    }
  }
  
  await storage.close();
}
```

## Migration and Upgrades

The storage format includes version information to support future migrations. Current version is 1.

For breaking changes, the SDK will provide migration utilities to convert stored keys to new formats while preserving data integrity.

## Performance Notes

- **Initialization**: Storage initialization is asynchronous and creates database if needed
- **Encryption**: Key derivation (PBKDF2) is computationally intensive by design for security
- **Storage**: IndexedDB operations are asynchronous and handle large datasets efficiently
- **Memory**: Keep retrieved keys in memory only as long as necessary

## Support

For issues, questions, or feature requests related to the storage API, please refer to the main DataFold SDK documentation or repository.