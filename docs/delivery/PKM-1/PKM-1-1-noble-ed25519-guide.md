# PKM-1-1 @noble/ed25519 Package Guide

**Date Created**: January 22, 2025  
**Package Version**: 2.3.0  
**Documentation Source**: [GitHub Repository](https://github.com/paulmillr/noble-ed25519), [NPM Package](https://www.npmjs.com/package/@noble/ed25519)

## Overview

@noble/ed25519 is the fastest 4KB JavaScript implementation of Ed25519 signatures. It's a modern, zero-dependency library designed for browser environments with excellent TypeScript support.

## Key Features

- ✅ **EdDSA signatures** compliant with RFC8032, FIPS 186-5
- ✅ **Zero dependencies** - pure JavaScript implementation
- ✅ **Small bundle size** - 4KB gzipped, 400 lines of code
- ✅ **TypeScript native** - built-in type definitions
- ✅ **Browser optimized** - works in all modern browsers
- ✅ **Performance** - significantly faster than alternatives
- ✅ **Security focused** - constant-time algorithms, audited codebase

## Installation

```bash
npm install @noble/ed25519
```

## Basic Usage

### Async API (Default)

```typescript
import * as ed from '@noble/ed25519';

// Generate keypair
const privateKey = ed.utils.randomPrivateKey();
const publicKey = await ed.getPublicKeyAsync(privateKey);

// Sign message
const message = new TextEncoder().encode('Hello, world!');
const signature = await ed.signAsync(message, privateKey);

// Verify signature
const isValid = await ed.verifyAsync(signature, message, publicKey);
console.log('Signature valid:', isValid);
```

### Sync API (Requires SHA-512 Setup)

```typescript
import * as ed from '@noble/ed25519';
import { sha512 } from '@noble/hashes/sha512';

// Enable sync methods
ed.etc.sha512Sync = (...m) => sha512(ed.etc.concatBytes(...m));

// Now sync methods are available
const privateKey = ed.utils.randomPrivateKey();
const publicKey = ed.getPublicKey(privateKey);
const signature = ed.sign(message, privateKey);
const isValid = ed.verify(signature, message, publicKey);
```

## React Integration Examples

### React Hook for Ed25519 Operations

```typescript
import { useState, useCallback } from 'react';
import * as ed from '@noble/ed25519';

interface KeyPair {
  privateKey: Uint8Array;
  publicKey: Uint8Array;
}

export const useEd25519 = () => {
  const [keyPair, setKeyPair] = useState<KeyPair | null>(null);
  const [isGenerating, setIsGenerating] = useState(false);

  const generateKeyPair = useCallback(async () => {
    setIsGenerating(true);
    try {
      const privateKey = ed.utils.randomPrivateKey();
      const publicKey = await ed.getPublicKeyAsync(privateKey);
      const newKeyPair = { privateKey, publicKey };
      setKeyPair(newKeyPair);
      return newKeyPair;
    } finally {
      setIsGenerating(false);
    }
  }, []);

  const sign = useCallback(async (message: string | Uint8Array) => {
    if (!keyPair) throw new Error('No keypair available');
    
    const messageBytes = typeof message === 'string' 
      ? new TextEncoder().encode(message)
      : message;
    
    return await ed.signAsync(messageBytes, keyPair.privateKey);
  }, [keyPair]);

  const verify = useCallback(async (
    signature: Uint8Array, 
    message: string | Uint8Array, 
    publicKey?: Uint8Array
  ) => {
    const messageBytes = typeof message === 'string' 
      ? new TextEncoder().encode(message)
      : message;
    
    const pubKey = publicKey || keyPair?.publicKey;
    if (!pubKey) throw new Error('No public key available');
    
    return await ed.verifyAsync(signature, messageBytes, pubKey);
  }, [keyPair]);

  return {
    keyPair,
    isGenerating,
    generateKeyPair,
    sign,
    verify
  };
};
```

### React Component Example

```typescript
import React, { useState } from 'react';
import { useEd25519 } from './useEd25519';

export const SignatureDemo: React.FC = () => {
  const { keyPair, isGenerating, generateKeyPair, sign, verify } = useEd25519();
  const [message, setMessage] = useState('Hello, world!');
  const [signature, setSignature] = useState<Uint8Array | null>(null);
  const [isValid, setIsValid] = useState<boolean | null>(null);

  const handleSign = async () => {
    try {
      const sig = await sign(message);
      setSignature(sig);
      setIsValid(null);
    } catch (error) {
      console.error('Signing failed:', error);
    }
  };

  const handleVerify = async () => {
    if (!signature) return;
    try {
      const valid = await verify(signature, message);
      setIsValid(valid);
    } catch (error) {
      console.error('Verification failed:', error);
    }
  };

  return (
    <div style={{ padding: '20px', maxWidth: '600px' }}>
      <h2>Ed25519 Signature Demo</h2>
      
      <div style={{ marginBottom: '20px' }}>
        <button 
          onClick={generateKeyPair} 
          disabled={isGenerating}
          style={{ padding: '10px 20px' }}
        >
          {isGenerating ? 'Generating...' : 'Generate New Keypair'}
        </button>
      </div>

      {keyPair && (
        <>
          <div style={{ marginBottom: '20px' }}>
            <h3>Public Key:</h3>
            <code style={{ 
              display: 'block', 
              padding: '10px', 
              backgroundColor: '#f5f5f5',
              wordBreak: 'break-all'
            }}>
              {Array.from(keyPair.publicKey)
                .map(b => b.toString(16).padStart(2, '0'))
                .join('')}
            </code>
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
              Sign Message
            </button>
          </div>

          {signature && (
            <div style={{ marginBottom: '20px' }}>
              <h3>Signature:</h3>
              <code style={{ 
                display: 'block', 
                padding: '10px', 
                backgroundColor: '#f5f5f5',
                wordBreak: 'break-all'
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

Based on benchmarks (Apple M2, macOS 13, Node.js 20):

- **Key Generation**: 9,173 ops/sec @ 109μs/op
- **Signing**: 4,567 ops/sec @ 218μs/op  
- **Verification**: 994 ops/sec @ 1ms/op
- **Point Decompression**: 16,164 ops/sec @ 61μs/op

## Security Considerations

### Strengths
- **Algorithmic constant time** operations
- **RFC8032 compliant** EdDSA implementation
- **ZIP215 compatible** for consensus applications
- **Audited codebase** (v1 audited by Cure53, v2 cross-tested)
- **Zero dependencies** - reduces supply chain risk

### Limitations  
- **No hardware security** - JavaScript execution environment
- **Memory safety** - Cannot guarantee secret memory cleanup due to GC
- **Side-channel risks** - JIT compilation may affect timing guarantees

### Best Practices
- Generate keys in secure environments
- Clear sensitive data when possible
- Use HTTPS in production
- Validate all inputs before cryptographic operations
- Consider hardware security modules for high-value keys

## Browser Compatibility

- ✅ **Chrome** 67+ (full support)
- ✅ **Firefox** 60+ (full support) 
- ✅ **Safari** 13+ (full support)
- ✅ **Edge** 18+ (full support)

### React Native Support

```typescript
// Add polyfill for React Native
import 'react-native-get-random-values';
import { sha512 } from '@noble/hashes/sha512';

// Configure for React Native
ed.etc.sha512Sync = (...m) => sha512(ed.etc.concatBytes(...m));
ed.etc.sha512Async = (...m) => Promise.resolve(ed.etc.sha512Sync(...m));
```

## Common Patterns

### Key Serialization

```typescript
// Convert keys to/from hex strings for storage
const privateKeyHex = ed.etc.bytesToHex(privateKey);
const publicKeyHex = ed.etc.bytesToHex(publicKey);

// Restore from hex
const restoredPrivateKey = ed.etc.hexToBytes(privateKeyHex);
const restoredPublicKey = ed.etc.hexToBytes(publicKeyHex);
```

### Message Preparation

```typescript
// Sign arbitrary data
const data = { userId: 123, action: 'transfer', amount: 100 };
const message = new TextEncoder().encode(JSON.stringify(data));
const signature = await ed.signAsync(message, privateKey);

// Sign file content
const fileBuffer = await file.arrayBuffer();
const fileBytes = new Uint8Array(fileBuffer);
const fileSignature = await ed.signAsync(fileBytes, privateKey);
```

## Error Handling

```typescript
import * as ed from '@noble/ed25519';

const signWithErrorHandling = async (message: string, privateKey: Uint8Array) => {
  try {
    const messageBytes = new TextEncoder().encode(message);
    return await ed.signAsync(messageBytes, privateKey);
  } catch (error) {
    if (error instanceof TypeError) {
      throw new Error('Invalid input parameters for signing');
    }
    throw new Error(`Signing failed: ${error.message}`);
  }
};

const verifyWithErrorHandling = async (
  signature: Uint8Array, 
  message: string, 
  publicKey: Uint8Array
) => {
  try {
    const messageBytes = new TextEncoder().encode(message);
    return await ed.verifyAsync(signature, messageBytes, publicKey);
  } catch (error) {
    // Invalid signature/key formats return false rather than throw
    console.warn('Verification failed:', error);
    return false;
  }
};
```

## Bundle Size Optimization

The library is already optimized for size, but you can further reduce bundle size:

```typescript
// Import only what you need (if using a tree-shaking bundler)
import { getPublicKeyAsync, signAsync, verifyAsync } from '@noble/ed25519';
import { randomPrivateKey } from '@noble/ed25519/utils';
```

## Comparison with Alternatives

| Feature | @noble/ed25519 | TweetNaCl.js | WebCrypto API |
|---------|----------------|--------------|---------------|
| Bundle Size | 4KB | ~30KB | 0KB (native) |
| Performance | Fast | Slower | Fastest |
| TypeScript | Native | .d.ts | Native |
| Dependencies | 0 | 0 | N/A |
| Browser Support | Excellent | Excellent | Good (modern only) |
| Audit Status | Partial | Full (2017) | Browser dependent |

## Migration Guide

### From TweetNaCl.js

```typescript
// Old TweetNaCl.js code
import nacl from 'tweetnacl';
const keyPair = nacl.sign.keyPair();
const signature = nacl.sign(message, keyPair.secretKey);
const isValid = nacl.sign.open(signature, keyPair.publicKey) !== null;

// New @noble/ed25519 code  
import * as ed from '@noble/ed25519';
const privateKey = ed.utils.randomPrivateKey();
const publicKey = await ed.getPublicKeyAsync(privateKey);
const signature = await ed.signAsync(message, privateKey);
const isValid = await ed.verifyAsync(signature, message, publicKey);
```

## Conclusion

@noble/ed25519 provides an excellent balance of performance, security, and developer experience for Ed25519 cryptography in JavaScript/TypeScript applications. Its zero-dependency design, small bundle size, and comprehensive TypeScript support make it ideal for modern web applications, particularly React-based projects requiring client-side signing capabilities.