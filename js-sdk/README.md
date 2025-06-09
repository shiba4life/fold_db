# DataFold JavaScript SDK

A client-side JavaScript SDK for Ed25519 key generation, management, and server integration in browser environments.

## Features

- **Ed25519 Key Generation**: Generate cryptographically secure Ed25519 key pairs
- **Secure Key Storage**: IndexedDB-based encrypted storage with passphrase protection
- **Key Derivation**: HKDF and PBKDF2 support for deriving keys from master keys
- **Key Rotation**: Secure key rotation with versioning and backward compatibility
- **Server Integration**: Public key registration and signature verification with DataFold server
- **Client-Side Security**: Private keys never leave the client environment
- **Browser Compatibility**: Works in modern browsers with WebCrypto API support
- **Multiple Formats**: Support for hex, base64, and Uint8Array key formats
- **Comprehensive Validation**: Built-in validation for all cryptographic operations
- **Key Export/Import**: Encrypted backup and recovery with multiple formats
- **Cross-Platform Compatibility**: Standard backup formats for data portability
- **TypeScript Support**: Full TypeScript definitions included

## Installation

```bash
npm install @datafold/js-sdk
```

## Quick Start

```javascript
import {
  generateKeyPair,
  formatKey,
  initializeSDK,
  createStorage,
  deriveKey,
  KeyRotationManager,
  KeyDerivationContexts,
  exportKey,
  importKey,
  createServerIntegration,
  quickIntegrationTest
} from '@datafold/js-sdk';

// Initialize the SDK and check compatibility
const { compatible, warnings } = await initializeSDK();

if (!compatible) {
  console.warn('Browser not fully compatible:', warnings);
}

// Generate an Ed25519 key pair
const keyPair = await generateKeyPair();

// Create secure storage
const storage = await createStorage();

// Store the key pair securely
await storage.storeKeyPair('my-key', keyPair, 'secure-passphrase', {
  name: 'My Application Key',
  description: 'Primary key for application operations'
});

// Derive keys for different purposes
const encryptionKey = await deriveKey(keyPair.privateKey, {
  algorithm: 'HKDF',
  info: KeyDerivationContexts.DATA_ENCRYPTION
});

// Set up key rotation
const rotationManager = new KeyRotationManager(storage);
const versionedKey = await rotationManager.createVersionedKeyPair(
  'my-key',
  keyPair,
  'secure-passphrase'
);

console.log('Key stored and ready for rotation');

// Server integration example
const integration = createServerIntegration({
  baseUrl: 'http://localhost:9001'
});

// Register key with server and verify signature
const workflow = await integration.registerAndVerifyWorkflow(
  keyPair,
  'Hello, DataFold!',
  {
    clientId: 'my_app_client',
    keyName: 'Application Key'
  }
);

console.log('Server integration successful:', workflow.verification.verified);
```

## Key Management Workflow

### 1. Key Generation and Storage

```javascript
// Generate and store a key
const keyPair = await generateKeyPair();
const storage = await createStorage();

await storage.storeKeyPair('primary-key', keyPair, 'passphrase', {
  name: 'Primary Application Key',
  tags: ['primary', 'production']
});
```

### 2. Key Derivation

```javascript
// Derive specialized keys
const signingKey = await deriveKey(keyPair.privateKey, {
  algorithm: 'HKDF',
  info: KeyDerivationContexts.SIGNING
});

const backupKey = await deriveKey(keyPair.privateKey, {
  algorithm: 'PBKDF2',
  iterations: 100000
});
```

### 3. Key Rotation

```javascript
// Set up rotation management
const rotationManager = new KeyRotationManager(storage);

// Rotate key with versioning
const result = await rotationManager.rotateKey('primary-key', 'passphrase', {
  reason: 'scheduled_rotation',
  keepOldVersion: true
});

console.log(`Rotated to version ${result.newVersion}`);
```

### 4. Key Export/Import

```javascript
// Export key to encrypted backup
const exportResult = await exportKey(storage, 'primary-key', 'passphrase', {
  format: 'json',
  includeMetadata: true,
  kdfIterations: 100000
});

// Save backup file
const filename = generateBackupFilename('primary-key', 'json');
console.log('Backup created:', filename);

// Import key from backup
const importResult = await importKey(storage, exportResult.data, 'passphrase', {
  overwriteExisting: false,
  validateIntegrity: true
});

console.log(`Key restored: ${importResult.keyId}`);
```

## API Reference

### Key Generation

#### `generateKeyPair(options?): Promise<Ed25519KeyPair>`

Generates a new Ed25519 key pair.

**Parameters:**
- `options` (optional): Key generation options
  - `validate?: boolean` - Whether to validate the generated keys (default: true)
  - `entropy?: Uint8Array` - Custom entropy source (32 bytes, for testing only)

**Returns:** Promise resolving to an Ed25519KeyPair object

**Example:**
```javascript
// Generate with default options
const keyPair = await generateKeyPair();

// Generate without validation (faster)
const keyPair = await generateKeyPair({ validate: false });

// Generate with custom entropy (testing only)
const entropy = new Uint8Array(32);
crypto.getRandomValues(entropy);
const keyPair = await generateKeyPair({ entropy });
```

#### `generateMultipleKeyPairs(count, options?): Promise<Ed25519KeyPair[]>`

Generates multiple key pairs efficiently.

**Parameters:**
- `count: number` - Number of key pairs to generate (1-100)
- `options` (optional): Key generation options

**Returns:** Promise resolving to an array of Ed25519KeyPair objects

### Key Formatting

#### `formatKey(key, format): string | Uint8Array`

Converts a key to the specified format.

**Parameters:**
- `key: Uint8Array` - The key to format
- `format: 'hex' | 'base64' | 'uint8array'` - Output format

**Returns:** Formatted key

**Example:**
```javascript
const keyPair = await generateKeyPair();

const hexKey = formatKey(keyPair.privateKey, 'hex');
const base64Key = formatKey(keyPair.publicKey, 'base64');
const arrayKey = formatKey(keyPair.privateKey, 'uint8array');
```

#### `parseKey(keyData, format): Uint8Array`

Parses a key from the specified format.

**Parameters:**
- `keyData: string | Uint8Array` - The key data to parse
- `format: 'hex' | 'base64' | 'uint8array'` - Input format

**Returns:** Parsed key as Uint8Array

### Utility Functions

#### `checkBrowserCompatibility(): BrowserCompatibility`

Checks browser compatibility for Ed25519 operations.

**Returns:** Compatibility information object

#### `initializeSDK(): Promise<{compatible: boolean, warnings: string[]}>`

Initializes the SDK and performs compatibility checks.

#### `isCompatible(): boolean`

Quick synchronous compatibility check.

#### `clearKeyMaterial(keyPair): void`

Clears sensitive key material from memory (best effort).

## Security Features

### Client-Side Key Generation

All key generation happens entirely in the browser using the WebCrypto API's secure random number generator. Private keys never leave the client environment.

### Secure Random Generation

The SDK uses `crypto.getRandomValues()` for cryptographically secure random number generation, ensuring high-quality entropy for key generation.

### Memory Security

The SDK provides utilities to clear sensitive key material from memory, though JavaScript's garbage collection limits guarantee complete memory clearing.

### Environment Validation

The SDK validates that it's running in a secure context (HTTPS) and that required cryptographic APIs are available.

## Error Handling

All SDK functions throw `Ed25519KeyError` objects with descriptive error messages and error codes for programmatic handling.

```javascript
import { generateKeyPair, Ed25519KeyError } from '@datafold/js-sdk';

try {
  const keyPair = await generateKeyPair();
} catch (error) {
  if (error instanceof Ed25519KeyError) {
    console.error('Key generation failed:', error.message);
    console.error('Error code:', error.code);
  }
}
```

## Browser Compatibility

- **Chrome/Edge**: Full support with WebCrypto API
- **Firefox**: Full support with WebCrypto API  
- **Safari**: Full support with WebCrypto API
- **Internet Explorer**: Not supported

**Requirements:**
- HTTPS (required for WebCrypto API in production)
- Modern browser with WebCrypto API support
- Secure context (automatically available on HTTPS and localhost)

## Examples

### Basic Key Generation and Storage

```javascript
import { generateKeyPair, formatKey, parseKey } from '@datafold/js-sdk';

async function createAndStoreKey() {
  // Generate key pair
  const keyPair = await generateKeyPair();
  
  // Format for storage
  const privateKeyHex = formatKey(keyPair.privateKey, 'hex');
  const publicKeyHex = formatKey(keyPair.publicKey, 'hex');
  
  // Store in browser storage (with encryption in real applications)
  localStorage.setItem('privateKey', privateKeyHex);
  localStorage.setItem('publicKey', publicKeyHex);
  
  return { privateKeyHex, publicKeyHex };
}

async function loadStoredKey() {
  // Retrieve from storage
  const privateKeyHex = localStorage.getItem('privateKey');
  const publicKeyHex = localStorage.getItem('publicKey');
  
  if (!privateKeyHex || !publicKeyHex) {
    throw new Error('No stored keys found');
  }
  
  // Parse back to Uint8Array
  const privateKey = parseKey(privateKeyHex, 'hex');
  const publicKey = parseKey(publicKeyHex, 'hex');
  
  return { privateKey, publicKey };
}
```

### Batch Key Generation

```javascript
import { generateMultipleKeyPairs } from '@datafold/js-sdk';

async function generateUserKeys() {
  // Generate keys for multiple users
  const keyPairs = await generateMultipleKeyPairs(10);
  
  const users = keyPairs.map((keyPair, index) => ({
    userId: `user_${index}`,
    publicKey: formatKey(keyPair.publicKey, 'hex'),
    // Never store private keys on server!
    privateKeyForClient: formatKey(keyPair.privateKey, 'hex')
  }));
  
  return users;
}
```

### Key Derivation Example

```javascript
import { deriveKey, deriveMultipleKeys, KeyDerivationContexts } from '@datafold/js-sdk';

// Single key derivation
const masterKey = await generateKeyPair();
const derivedKey = await deriveKey(masterKey.privateKey, {
  algorithm: 'HKDF',
  info: KeyDerivationContexts.DATA_ENCRYPTION
});

// Multiple key derivation
const derivedKeys = await deriveMultipleKeys(masterKey.privateKey, [
  { name: 'encryption', info: KeyDerivationContexts.DATA_ENCRYPTION },
  { name: 'signing', info: KeyDerivationContexts.SIGNING },
  { name: 'backup', info: KeyDerivationContexts.BACKUP_ENCRYPTION }
]);
```

### Key Rotation Example

```javascript
import { KeyRotationManager, emergencyKeyRotation } from '@datafold/js-sdk';

// Regular rotation
const rotationManager = new KeyRotationManager(storage);
const result = await rotationManager.rotateKey('my-key', 'passphrase', {
  reason: 'scheduled_rotation',
  rotateDerivedKeys: true
});

// Emergency rotation
const emergencyResult = await emergencyKeyRotation(
  'compromised-key',
  'old-passphrase',
  'new-passphrase',
  storage,
  'security_breach'
);
```

### Environment Validation

```javascript
import { initializeSDK, isStorageSupported } from '@datafold/js-sdk';

async function setupCrypto() {
  try {
    // Check storage support
    const { supported, reasons } = isStorageSupported();
    if (!supported) {
      throw new Error(`Storage not supported: ${reasons.join(', ')}`);
    }
    
    // Initialize SDK
    const { compatible, warnings } = await initializeSDK();
    
    if (!compatible) {
      throw new Error(`Browser not compatible: ${warnings.join(', ')}`);
    }
    
    return true;
  } catch (error) {
    console.error('Crypto setup failed:', error.message);
    return false;
  }
}
```

## Contributing

See the main DataFold repository for contribution guidelines.

## License

MIT License - see LICENSE file for details.

## Security

This SDK follows security best practices for client-side cryptography:

- All operations are performed client-side
- Private keys never leave the browser
- Secure random number generation
- Input validation and error handling
- Memory clearing utilities

For security issues, please follow responsible disclosure practices.

## Advanced Features

### Hierarchical Key Derivation

Create a tree of derived keys for different purposes:

```javascript
const masterKey = await generateKeyPair();

// Domain-level keys
const userDomainKey = await deriveKey(masterKey.privateKey, {
  algorithm: 'HKDF',
  info: createHKDFInfo('domain.users')
});

// Purpose-specific keys
const userEncKey = await deriveKey(userDomainKey.key, {
  algorithm: 'HKDF',
  info: createHKDFInfo('users.encryption')
});
```

### Batch Operations

Rotate multiple keys efficiently:

```javascript
import { batchRotateKeys } from '@datafold/js-sdk';

const results = await batchRotateKeys(
  ['key1', 'key2', 'key3'],
  'passphrase',
  storage,
  { reason: 'policy_compliance' }
);
```

### Audit and Compliance

Validate key rotation history:

```javascript
import { validateKeyRotationHistory } from '@datafold/js-sdk';

const validation = await validateKeyRotationHistory('my-key', 'passphrase', storage);
console.log('Rotation history valid:', validation.valid);
console.log('Total rotations:', validation.rotationCount);
```

### Key Backup and Recovery

Create secure encrypted backups:

```javascript
import {
  exportKey,
  importKey,
  generateBackupFilename,
  validateBackupData
} from '@datafold/js-sdk';

// Create encrypted backup
const exportOptions = {
  format: 'json', // or 'binary'
  includeMetadata: true,
  kdfIterations: 100000
};

const backup = await exportKey(storage, 'my-key', 'strong-passphrase', exportOptions);
const filename = generateBackupFilename('my-key', 'json');

// Validate backup before saving
const validation = validateBackupData(backup.data);
if (!validation.valid) {
  throw new Error(`Backup validation failed: ${validation.issues.join(', ')}`);
}

// Save to file (in a real app, you'd use File API or similar)
console.log(`Backup size: ${backup.size} bytes`);
console.log(`Checksum: ${backup.checksum}`);

// Later, restore from backup
const importOptions = {
  validateIntegrity: true,
  overwriteExisting: false,
  customMetadata: {
    description: 'Restored from backup',
    tags: ['restored']
  }
};

const restored = await importKey(storage, backup.data, 'strong-passphrase', importOptions);
console.log(`Restored key: ${restored.keyId}`);
```

## Documentation

- [Key Derivation and Rotation Guide](./docs/key-derivation-and-rotation.md)
- [Storage API Documentation](./docs/storage-api.md)