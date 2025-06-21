# PKM-1-1 TweetNaCl.js Package Guide

**Date Created**: January 22, 2025  
**Package Version**: 1.0.3  
**Documentation Source**: [GitHub Repository](https://github.com/dchest/tweetnacl-js), [NPM Package](https://www.npmjs.com/package/tweetnacl)

## Overview

TweetNaCl.js is a JavaScript port of the TweetNaCl cryptographic library to JavaScript for modern browsers and Node.js. It provides a complete cryptographic toolkit including Ed25519 signatures, X25519 key agreement, and symmetric encryption.

## Key Features

- ✅ **Complete crypto toolkit** - signatures, encryption, hashing, key agreement
- ✅ **Port of proven library** - based on audited TweetNaCl C implementation
- ✅ **Zero dependencies** - self-contained implementation
- ✅ **Audited** - security audit by Cure53 (2017)
- ✅ **Stable API** - mature, well-established interface
- ✅ **TypeScript support** - includes type definitions

## Installation

```bash
npm install tweetnacl
```

For TypeScript:
```bash
npm install tweetnacl @types/tweetnacl
```

## Basic Usage

### Ed25519 Signatures

```typescript
import nacl from 'tweetnacl';

// Generate keypair
const keyPair = nacl.sign.keyPair();
console.log('Public key length:', keyPair.publicKey.length); // 32 bytes
console.log('Secret key length:', keyPair.secretKey.length); // 64 bytes

// Sign message
const message = new TextEncoder().encode('Hello, world!');
const signedMessage = nacl.sign(message, keyPair.secretKey);

// Verify signature
const verifiedMessage = nacl.sign.open(signedMessage, keyPair.publicKey);
console.log('Verification successful:', verifiedMessage !== null);

// Detached signatures
const signature = nacl.sign.detached(message, keyPair.secretKey);
const isValid = nacl.sign.detached.verify(message, signature, keyPair.publicKey);
console.log('Detached signature valid:', isValid);
```

### Key Generation from Seed

```typescript
// Generate deterministic keypair from seed
const seed = nacl.randomBytes(32);
const keyPairFromSeed = nacl.sign.keyPair.fromSeed(seed);

// Or from existing secret key
const keyPairFromSecret = nacl.sign.keyPair.fromSecretKey(keyPair.secretKey);
```

## React Integration Examples

### React Hook for TweetNaCl Operations

```typescript
import { useState, useCallback } from 'react';
import nacl from 'tweetnacl';

interface NaClKeyPair {
  publicKey: Uint8Array;
  secretKey: Uint8Array;
}

export const useTweetNaCl = () => {
  const [keyPair, setKeyPair] = useState<NaClKeyPair | null>(null);
  const [isGenerating, setIsGenerating] = useState(false);

  const generateKeyPair = useCallback(() => {
    setIsGenerating(true);
    try {
      const newKeyPair = nacl.sign.keyPair();
      setKeyPair(newKeyPair);
      return newKeyPair;
    } finally {
      setIsGenerating(false);
    }
  }, []);

  const generateFromSeed = useCallback((seed: Uint8Array) => {
    setIsGenerating(true);
    try {
      const newKeyPair = nacl.sign.keyPair.fromSeed(seed);
      setKeyPair(newKeyPair);
      return newKeyPair;
    } finally {
      setIsGenerating(false);
    }
  }, []);

  const sign = useCallback((message: string | Uint8Array) => {
    if (!keyPair) throw new Error('No keypair available');
    
    const messageBytes = typeof message === 'string' 
      ? new TextEncoder().encode(message)
      : message;
    
    return nacl.sign(messageBytes, keyPair.secretKey);
  }, [keyPair]);

  const signDetached = useCallback((message: string | Uint8Array) => {
    if (!keyPair) throw new Error('No keypair available');
    
    const messageBytes = typeof message === 'string' 
      ? new TextEncoder().encode(message)
      : message;
    
    return nacl.sign.detached(messageBytes, keyPair.secretKey);
  }, [keyPair]);

  const verify = useCallback((
    signedMessage: Uint8Array, 
    publicKey?: Uint8Array
  ) => {
    const pubKey = publicKey || keyPair?.publicKey;
    if (!pubKey) throw new Error('No public key available');
    
    return nacl.sign.open(signedMessage, pubKey);
  }, [keyPair]);

  const verifyDetached = useCallback((
    signature: Uint8Array,
    message: string | Uint8Array,
    publicKey?: Uint8Array
  ) => {
    const messageBytes = typeof message === 'string' 
      ? new TextEncoder().encode(message)
      : message;
    
    const pubKey = publicKey || keyPair?.publicKey;
    if (!pubKey) throw new Error('No public key available');
    
    return nacl.sign.detached.verify(messageBytes, signature, pubKey);
  }, [keyPair]);

  return {
    keyPair,
    isGenerating,
    generateKeyPair,
    generateFromSeed,
    sign,
    signDetached,
    verify,
    verifyDetached
  };
};
```

### React Component Example

```typescript
import React, { useState } from 'react';
import { useTweetNaCl } from './useTweetNaCl';
import nacl from 'tweetnacl';

export const TweetNaClDemo: React.FC = () => {
  const { 
    keyPair, 
    isGenerating, 
    generateKeyPair, 
    signDetached, 
    verifyDetached 
  } = useTweetNaCl();
  
  const [message, setMessage] = useState('Hello, world!');
  const [signature, setSignature] = useState<Uint8Array | null>(null);
  const [isValid, setIsValid] = useState<boolean | null>(null);
  const [seedInput, setSeedInput] = useState('');

  const handleSign = () => {
    try {
      const sig = signDetached(message);
      setSignature(sig);
      setIsValid(null);
    } catch (error) {
      console.error('Signing failed:', error);
    }
  };

  const handleVerify = () => {
    if (!signature) return;
    try {
      const valid = verifyDetached(signature, message);
      setIsValid(valid);
    } catch (error) {
      console.error('Verification failed:', error);
      setIsValid(false);
    }
  };

  const handleGenerateFromSeed = () => {
    try {
      const seed = new TextEncoder().encode(seedInput).slice(0, 32);
      // Pad to 32 bytes if needed
      const paddedSeed = new Uint8Array(32);
      paddedSeed.set(seed);
      
      const newKeyPair = nacl.sign.keyPair.fromSeed(paddedSeed);
      // This would update the hook's state in a real implementation
      console.log('Generated keypair from seed:', newKeyPair);
    } catch (error) {
      console.error('Seed generation failed:', error);
    }
  };

  return (
    <div style={{ padding: '20px', maxWidth: '600px' }}>
      <h2>TweetNaCl.js Ed25519 Demo</h2>
      
      <div style={{ marginBottom: '20px' }}>
        <button 
          onClick={generateKeyPair} 
          disabled={isGenerating}
          style={{ padding: '10px 20px', marginRight: '10px' }}
        >
          {isGenerating ? 'Generating...' : 'Generate Random Keypair'}
        </button>
      </div>

      <div style={{ marginBottom: '20px' }}>
        <h3>Generate from Seed:</h3>
        <input
          type="text"
          value={seedInput}
          onChange={(e) => setSeedInput(e.target.value)}
          placeholder="Enter seed text"
          style={{ 
            width: '70%', 
            padding: '8px',
            marginRight: '10px'
          }}
        />
        <button 
          onClick={handleGenerateFromSeed}
          style={{ padding: '8px 16px' }}
        >
          Generate from Seed
        </button>
      </div>

      {keyPair && (
        <>
          <div style={{ marginBottom: '20px' }}>
            <h3>Public Key (32 bytes):</h3>
            <code style={{ 
              display: 'block', 
              padding: '10px', 
              backgroundColor: '#f5f5f5',
              wordBreak: 'break-all',
              fontSize: '12px'
            }}>
              {Array.from(keyPair.publicKey)
                .map(b => b.toString(16).padStart(2, '0'))
                .join('')}
            </code>
          </div>

          <div style={{ marginBottom: '20px' }}>
            <h3>Secret Key (64 bytes):</h3>
            <code style={{ 
              display: 'block', 
              padding: '10px', 
              backgroundColor: '#ffe6e6',
              wordBreak: 'break-all',
              fontSize: '10px'
            }}>
              {Array.from(keyPair.secretKey)
                .map(b => b.toString(16).padStart(2, '0'))
                .join('')}
            </code>
            <small style={{ color: '#666' }}>
              ⚠️ Secret key should never be displayed in production
            </small>
          </div>

          <div style={{ marginBottom: '20px' }}>
            <h3>Message to Sign:</h3>
            <input
              type="text"
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              style={{ 
                width: '100%', 
                padding: '10px',
                marginBottom: '10px'
              }}
            />
            <button onClick={handleSign} style={{ padding: '10px 20px' }}>
              Sign Message (Detached)
            </button>
          </div>

          {signature && (
            <div style={{ marginBottom: '20px' }}>
              <h3>Signature (64 bytes):</h3>
              <code style={{ 
                display: 'block', 
                padding: '10px', 
                backgroundColor: '#f5f5f5',
                wordBreak: 'break-all',
                fontSize: '12px'
              }}>
                {Array.from(signature)
                  .map(b => b.toString(16).padStart(2, '0'))
                  .join('')}
              </code>
              <button 
                onClick={handleVerify} 
                style={{ padding: '10px 20px', marginTop: '10px' }}
              >
                Verify Signature
              </button>
            </div>
          )}

          {isValid !== null && (
            <div style={{ 
              padding: '15px', 
              backgroundColor: isValid ? '#d4edda' : '#f8d7da',
              border: `1px solid ${isValid ? '#c3e6cb' : '#f5c6cb'}`,
              borderRadius: '4px'
            }}>
              <strong>
                Verification Result: {isValid ? '✅ Valid' : '❌ Invalid'}
              </strong>
            </div>
          )}
        </>
      )}
    </div>
  );
};
```

## Performance Characteristics

Based on benchmarks:

- **Key Generation**: ~1,808 ops/sec (significantly slower than @noble/ed25519)
- **Signing**: ~651 ops/sec  
- **Verification**: Variable performance
- **Larger memory footprint** due to less optimized implementation

## Security Considerations

### Strengths
- **Audited implementation** - Cure53 security audit (2017)
- **Proven cryptographic design** - port of well-tested TweetNaCl
- **Zero dependencies** - reduces supply chain risk
- **Stable codebase** - mature and well-established

### Limitations
- **Performance** - significantly slower than modern alternatives
- **Bundle size** - larger than newer implementations
- **Maintenance** - less actively maintained than newer libraries
- **JavaScript environment limitations** - same memory/timing concerns as other JS crypto

### Best Practices
- Use for compatibility with existing TweetNaCl systems
- Consider performance implications for high-throughput applications
- Always validate cryptographic inputs
- Use secure random number generation for keys

## API Reference

### Key Generation

```typescript
// Random keypair
const keyPair = nacl.sign.keyPair();

// From 32-byte seed
const keyPair = nacl.sign.keyPair.fromSeed(seed);

// From 64-byte secret key
const keyPair = nacl.sign.keyPair.fromSecretKey(secretKey);
```

### Signing and Verification

```typescript
// Attached signature (includes message)
const signedMessage = nacl.sign(message, secretKey);
const originalMessage = nacl.sign.open(signedMessage, publicKey);

// Detached signature (signature separate from message)  
const signature = nacl.sign.detached(message, secretKey);
const isValid = nacl.sign.detached.verify(message, signature, publicKey);
```

### Utility Functions

```typescript
// Generate random bytes
const randomBytes = nacl.randomBytes(32);

// Constant-time comparison
const isEqual = nacl.verify(array1, array2);

// Available constants
nacl.sign.publicKeyLength;  // 32
nacl.sign.secretKeyLength;  // 64
nacl.sign.seedLength;       // 32
nacl.sign.signatureLength;  // 64
```

## Additional Cryptographic Features

TweetNaCl.js provides more than just Ed25519:

### X25519 Key Agreement

```typescript
// Generate X25519 keypair
const boxKeyPair = nacl.box.keyPair();

// Perform key agreement
const sharedSecret = nacl.box.before(theirPublicKey, mySecretKey);
```

### Authenticated Encryption

```typescript
// Encrypt with public key crypto
const nonce = nacl.randomBytes(24);
const encrypted = nacl.box(message, nonce, theirPublicKey, mySecretKey);
const decrypted = nacl.box.open(encrypted, nonce, theirPublicKey, mySecretKey);

// Encrypt with symmetric crypto
const key = nacl.randomBytes(32);
const encrypted = nacl.secretbox(message, nonce, key);
const decrypted = nacl.secretbox.open(encrypted, nonce, key);
```

### Hashing

```typescript
// SHA-512 hash
const hash = nacl.hash(message);
```

## Browser Compatibility

- ✅ **Chrome** (all modern versions)
- ✅ **Firefox** (all modern versions)
- ✅ **Safari** (all modern versions)
- ✅ **Edge** (all modern versions)
- ✅ **Internet Explorer 11** (with polyfills)

### React Native Support

```typescript
// TweetNaCl.js works in React Native without additional polyfills
// However, you might need crypto polyfills for random number generation
import 'react-native-get-random-values';
import nacl from 'tweetnacl';

// Works directly
const keyPair = nacl.sign.keyPair();
```

## Comparison with Alternatives

| Feature | TweetNaCl.js | @noble/ed25519 | WebCrypto API |
|---------|--------------|----------------|---------------|
| Bundle Size | ~30KB | 4KB | 0KB (native) |
| Performance | Slower | Faster | Fastest |
| Full Crypto Suite | ✅ | ❌ (Ed25519 only) | ✅ |
| TypeScript | .d.ts | Native | Native |
| Audit Status | Full (2017) | Partial | Browser dependent |
| Maintenance | Stable | Active | N/A |

## Migration Guide

### To @noble/ed25519

```typescript
// Old TweetNaCl.js code
import nacl from 'tweetnacl';
const keyPair = nacl.sign.keyPair();
const signature = nacl.sign.detached(message, keyPair.secretKey);
const isValid = nacl.sign.detached.verify(message, signature, keyPair.publicKey);

// New @noble/ed25519 code
import * as ed from '@noble/ed25519';
const privateKey = ed.utils.randomPrivateKey();
const publicKey = await ed.getPublicKeyAsync(privateKey);
const signature = await ed.signAsync(message, privateKey);
const isValid = await ed.verifyAsync(signature, message, publicKey);
```

### Key Format Differences

```typescript
// TweetNaCl.js uses 64-byte secret keys (private + public)
const naclKeyPair = nacl.sign.keyPair();
console.log(naclKeyPair.secretKey.length); // 64 bytes
console.log(naclKeyPair.publicKey.length); // 32 bytes

// Extract 32-byte private key for other libraries
const privateKey32 = naclKeyPair.secretKey.slice(0, 32);

// @noble/ed25519 uses 32-byte private keys
const noblePrivateKey = ed.utils.randomPrivateKey();
console.log(noblePrivateKey.length); // 32 bytes
```

## Error Handling

```typescript
const signWithErrorHandling = (message: Uint8Array, secretKey: Uint8Array) => {
  try {
    if (secretKey.length !== 64) {
      throw new Error('Invalid secret key length');
    }
    return nacl.sign.detached(message, secretKey);
  } catch (error) {
    throw new Error(`Signing failed: ${error.message}`);
  }
};

const verifyWithErrorHandling = (
  message: Uint8Array, 
  signature: Uint8Array, 
  publicKey: Uint8Array
) => {
  try {
    if (publicKey.length !== 32) {
      throw new Error('Invalid public key length');
    }
    if (signature.length !== 64) {
      throw new Error('Invalid signature length');
    }
    return nacl.sign.detached.verify(message, signature, publicKey);
  } catch (error) {
    console.warn('Verification failed:', error);
    return false;
  }
};
```

## When to Choose TweetNaCl.js

### Good For:
- ✅ **Legacy compatibility** - existing systems using TweetNaCl
- ✅ **Full cryptographic suite** - need multiple crypto primitives
- ✅ **Proven stability** - mature, well-tested codebase
- ✅ **Conservative choice** - fully audited implementation

### Consider Alternatives If:
- ❌ **Performance critical** - @noble/ed25519 is significantly faster
- ❌ **Bundle size sensitive** - modern alternatives are much smaller
- ❌ **Ed25519 only** - specialized libraries may be better
- ❌ **Active development needed** - less frequently updated

## Conclusion

TweetNaCl.js remains a solid, conservative choice for applications requiring Ed25519 signatures with a proven track record. While newer libraries like @noble/ed25519 offer better performance and smaller bundle sizes, TweetNaCl.js provides a complete cryptographic toolkit with full audit coverage. Choose TweetNaCl.js when you need stability, compatibility, or multiple cryptographic primitives in a single package.