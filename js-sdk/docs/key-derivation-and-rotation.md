# Key Derivation and Rotation

This document describes the key derivation and rotation capabilities of the DataFold JavaScript SDK.

## Overview

The SDK provides comprehensive key derivation and rotation functionality that allows you to:

- Derive keys from master keys using HKDF or PBKDF2
- Rotate keys securely while maintaining backward compatibility
- Manage versioned key pairs with audit trails
- Perform emergency key rotations
- Batch rotate multiple keys

## Key Derivation

### Basic Key Derivation

```javascript
import { deriveKey, generateKeyPair, KeyDerivationContexts } from '@datafold/js-sdk';

// Generate a master key
const masterKeyPair = await generateKeyPair();

// Derive a key using HKDF
const derivedKey = await deriveKey(masterKeyPair.privateKey, {
  algorithm: 'HKDF',
  info: KeyDerivationContexts.DATA_ENCRYPTION,
  hash: 'SHA-256',
  length: 32
});

console.log('Derived key:', derivedKey.key);
console.log('Algorithm used:', derivedKey.algorithm);
console.log('Salt:', derivedKey.salt);
```

### PBKDF2 Key Derivation

```javascript
// Derive a key using PBKDF2
const pbkdf2Key = await deriveKey(masterKeyPair.privateKey, {
  algorithm: 'PBKDF2',
  iterations: 100000,
  hash: 'SHA-256',
  length: 32
});

console.log('PBKDF2 derived key:', pbkdf2Key.key);
console.log('Iterations:', pbkdf2Key.iterations);
```

### Multiple Key Derivation

```javascript
import { deriveMultipleKeys, createHKDFInfo } from '@datafold/js-sdk';

// Derive multiple keys for different purposes
const derivedKeys = await deriveMultipleKeys(masterKeyPair.privateKey, [
  { 
    name: 'encryption', 
    info: KeyDerivationContexts.DATA_ENCRYPTION 
  },
  { 
    name: 'signing', 
    info: KeyDerivationContexts.SIGNING 
  },
  { 
    name: 'authentication', 
    info: createHKDFInfo('auth.tokens'),
    options: { hash: 'SHA-512' }
  }
]);

console.log('Encryption key:', derivedKeys.encryption.key);
console.log('Signing key:', derivedKeys.signing.key);
console.log('Auth key:', derivedKeys.authentication.key);
```

### Predefined Contexts

The SDK provides predefined contexts for common use cases:

```javascript
import { KeyDerivationContexts } from '@datafold/js-sdk';

// Available predefined contexts:
// - KeyDerivationContexts.DATA_ENCRYPTION
// - KeyDerivationContexts.SIGNING
// - KeyDerivationContexts.AUTHENTICATION
// - KeyDerivationContexts.KEY_WRAPPING
// - KeyDerivationContexts.BACKUP_ENCRYPTION
```

## Key Rotation

### Basic Key Rotation

```javascript
import { KeyRotationManager, createStorage } from '@datafold/js-sdk';

// Create storage and rotation manager
const storage = await createStorage();
const rotationManager = new KeyRotationManager(storage);

// Create a versioned key pair
const keyPair = await generateKeyPair();
const versionedKeyPair = await rotationManager.createVersionedKeyPair(
  'my-key',
  keyPair,
  'secure-passphrase',
  {
    name: 'My Application Key',
    description: 'Primary key for application operations',
    tags: ['primary', 'application']
  }
);

// Rotate the key
const rotationResult = await rotationManager.rotateKey('my-key', 'secure-passphrase', {
  keepOldVersion: true,
  reason: 'scheduled_rotation',
  metadata: {
    description: 'Rotated on schedule'
  }
});

console.log('New version:', rotationResult.newVersion);
console.log('Previous version:', rotationResult.previousVersion);
console.log('Old version preserved:', rotationResult.oldVersionPreserved);
```

### Key Rotation with Derived Keys

```javascript
// Rotate key and update all derived keys
const rotationResult = await rotationManager.rotateKey('my-key', 'passphrase', {
  rotateDerivedKeys: true,
  reason: 'security_update',
  keepOldVersion: false
});

console.log('Rotated derived keys:', rotationResult.rotatedDerivedKeys);
```

### Emergency Key Rotation

```javascript
import { emergencyKeyRotation } from '@datafold/js-sdk';

// Emergency rotation (e.g., key compromise)
const emergencyResult = await emergencyKeyRotation(
  'compromised-key',
  'old-passphrase',
  'new-secure-passphrase',
  storage,
  'security_breach_detected'
);

console.log('Emergency rotation completed');
console.log('New version:', emergencyResult.newVersion);
```

### Batch Key Rotation

```javascript
import { batchRotateKeys } from '@datafold/js-sdk';

// Rotate multiple keys at once
const keyIds = ['key1', 'key2', 'key3'];
const batchResults = await batchRotateKeys(keyIds, 'passphrase', storage, {
  reason: 'policy_compliance',
  keepOldVersion: true
});

// Check results
for (const [keyId, result] of Object.entries(batchResults)) {
  if (result instanceof Error) {
    console.error(`Failed to rotate ${keyId}:`, result.message);
  } else {
    console.log(`Successfully rotated ${keyId} to version ${result.newVersion}`);
  }
}
```

## Key Version Management

### List Key Versions

```javascript
// List all versions of a key
const versions = await rotationManager.listKeyVersions('my-key', 'passphrase');

versions.forEach(version => {
  console.log(`Version ${version.version}:`);
  console.log(`  Created: ${version.created}`);
  console.log(`  Active: ${version.active}`);
  console.log(`  Reason: ${version.reason}`);
});
```

### Get Specific Version

```javascript
// Get a specific version of a key
const version2 = await rotationManager.getKeyVersion('my-key', 2, 'passphrase');

if (version2) {
  console.log('Version 2 key pair:', version2.keyPair);
  console.log('Created:', version2.created);
  console.log('Derived keys:', Object.keys(version2.derivedKeys || {}));
}
```

### Clean Up Old Versions

```javascript
// Remove old inactive versions (keep only 2 most recent)
const removedCount = await rotationManager.cleanupOldVersions('my-key', 'passphrase', 2);

console.log(`Removed ${removedCount} old versions`);
```

## Advanced Features

### Hierarchical Key Derivation

```javascript
import { deriveKey, createHKDFInfo } from '@datafold/js-sdk';

// Create a hierarchical key structure
const masterKey = await generateKeyPair();

// Level 1: Domain keys
const userDomainKey = await deriveKey(masterKey.privateKey, {
  algorithm: 'HKDF',
  info: createHKDFInfo('domain.users'),
  hash: 'SHA-256'
});

const dataDomainKey = await deriveKey(masterKey.privateKey, {
  algorithm: 'HKDF',
  info: createHKDFInfo('domain.data'),
  hash: 'SHA-256'
});

// Level 2: Purpose-specific keys
const userEncryptionKey = await deriveKey(userDomainKey.key, {
  algorithm: 'HKDF',
  info: createHKDFInfo('users.encryption'),
  hash: 'SHA-256'
});

const dataBackupKey = await deriveKey(dataDomainKey.key, {
  algorithm: 'HKDF',
  info: createHKDFInfo('data.backup'),
  hash: 'SHA-256'
});
```

### Key Validation

```javascript
import { validateDerivedKey } from '@datafold/js-sdk';

// Validate a derived key
const isValid = await validateDerivedKey(masterKey.privateKey, derivedKey);

if (isValid) {
  console.log('Derived key is valid');
} else {
  console.error('Derived key validation failed');
}
```

### Audit and Compliance

```javascript
import { validateKeyRotationHistory } from '@datafold/js-sdk';

// Validate rotation history for audit purposes
const validation = await validateKeyRotationHistory('my-key', 'passphrase', storage);

console.log('Rotation history valid:', validation.valid);
console.log('Total rotations:', validation.rotationCount);
console.log('Last rotation:', validation.lastRotation);

if (!validation.valid) {
  console.error('Validation issues:', validation.issues);
}
```

### Memory Security

```javascript
import { clearDerivedKey } from '@datafold/js-sdk';

// Clear sensitive key material from memory
clearDerivedKey(derivedKey);

// The key material is now zeroed out
console.log('Key cleared from memory');
```

## Security Best Practices

### 1. Use Strong Passphrases

```javascript
import { validatePassphrase } from '@datafold/js-sdk';

const passphrase = 'my-super-secure-passphrase-123!';
const validation = validatePassphrase(passphrase);

if (!validation.valid) {
  console.error('Weak passphrase:', validation.issues);
}
```

### 2. Regular Key Rotation

```javascript
// Implement automated key rotation
async function scheduleKeyRotation(keyId, storage) {
  const rotationManager = new KeyRotationManager(storage);
  
  // Check last rotation
  const validation = await validateKeyRotationHistory(keyId, passphrase, storage);
  const lastRotation = new Date(validation.lastRotation || 0);
  const daysSinceRotation = (Date.now() - lastRotation.getTime()) / (1000 * 60 * 60 * 24);
  
  // Rotate if more than 90 days
  if (daysSinceRotation > 90) {
    await rotationManager.rotateKey(keyId, passphrase, {
      reason: 'scheduled_rotation',
      keepOldVersion: true
    });
    console.log(`Key ${keyId} rotated after ${daysSinceRotation} days`);
  }
}
```

### 3. Secure Context Requirements

```javascript
import { isSecureContext } from '@datafold/js-sdk';

// Ensure running in secure context
if (!isSecureContext()) {
  throw new Error('Key operations require HTTPS or localhost');
}
```

### 4. Error Handling

```javascript
import { KeyDerivationError, KeyRotationError } from '@datafold/js-sdk';

try {
  const derivedKey = await deriveKey(masterKey, options);
} catch (error) {
  if (error instanceof KeyDerivationError) {
    console.error('Key derivation failed:', error.code, error.message);
  } else {
    console.error('Unexpected error:', error);
  }
}

try {
  const result = await rotationManager.rotateKey(keyId, passphrase);
} catch (error) {
  if (error instanceof KeyRotationError) {
    console.error('Key rotation failed:', error.code, error.message);
  } else {
    console.error('Unexpected error:', error);
  }
}
```

## Configuration Options

### Key Derivation Options

```javascript
const options = {
  algorithm: 'HKDF' | 'PBKDF2',
  hash: 'SHA-256' | 'SHA-384' | 'SHA-512',
  length: 32, // bytes
  salt: new Uint8Array(32), // optional, random if not provided
  info: new Uint8Array(), // required for HKDF
  iterations: 100000 // required for PBKDF2
};
```

### Key Rotation Options

```javascript
const rotationOptions = {
  keepOldVersion: true, // preserve old version
  reason: 'scheduled_rotation', // rotation reason
  rotateDerivedKeys: true, // update derived keys
  metadata: {
    description: 'Updated description',
    tags: ['rotated', 'secure']
  }
};
```

## Performance Considerations

- Key derivation operations are CPU-intensive
- Use appropriate iteration counts for PBKDF2 (minimum 100,000)
- Consider using HKDF for better performance when deriving multiple keys
- Clean up old key versions regularly to manage storage usage
- Use batch operations for rotating multiple keys

## Browser Compatibility

- Requires WebCrypto API support
- HTTPS required in production (secure context)
- IndexedDB required for key storage
- Modern browsers (Chrome 60+, Firefox 55+, Safari 11+)