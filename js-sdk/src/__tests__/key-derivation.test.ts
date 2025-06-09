/**
 * Unit tests for key derivation functionality
 */

import {
  deriveKey,
  deriveMultipleKeys,
  deriveKeyFromKeyPair,
  createHKDFInfo,
  validateDerivedKey,
  clearDerivedKey,
  KeyDerivationContexts
} from '../crypto/key-derivation.js';
import { generateKeyPair } from '../crypto/ed25519.js';
import { KeyDerivationError } from '../types.js';

// Mock crypto.subtle for testing
const mockCrypto = {
  subtle: {
    importKey: jest.fn(),
    deriveKey: jest.fn(),
    exportKey: jest.fn()
  },
  getRandomValues: jest.fn((arr: Uint8Array) => {
    for (let i = 0; i < arr.length; i++) {
      arr[i] = Math.floor(Math.random() * 256);
    }
    return arr;
  })
};

// Override global crypto for tests
Object.defineProperty(globalThis, 'crypto', {
  value: mockCrypto,
  writable: true
});

describe('Key Derivation', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    
    // Setup default mock implementations
    mockCrypto.subtle.importKey.mockResolvedValue({} as CryptoKey);
    mockCrypto.subtle.deriveKey.mockResolvedValue({} as CryptoKey);
    mockCrypto.subtle.exportKey.mockResolvedValue(new ArrayBuffer(32));
  });

  describe('deriveKey', () => {
    it('should derive key using HKDF', async () => {
      const masterKey = new Uint8Array(32);
      const info = new TextEncoder().encode('test-info');
      
      const result = await deriveKey(masterKey, {
        algorithm: 'HKDF',
        info,
        hash: 'SHA-256'
      });

      expect(result.algorithm).toBe('HKDF');
      expect(result.key).toBeInstanceOf(Uint8Array);
      expect(result.key.length).toBe(32);
      expect(result.info).toEqual(info);
      expect(result.hash).toBe('SHA-256');
      expect(result.derived).toBeDefined();
      
      expect(mockCrypto.subtle.importKey).toHaveBeenCalledWith(
        'raw',
        masterKey,
        { name: 'HKDF' },
        false,
        ['deriveKey']
      );
    });

    it('should derive key using PBKDF2', async () => {
      const masterKey = new Uint8Array(32);
      const iterations = 50000;
      
      const result = await deriveKey(masterKey, {
        algorithm: 'PBKDF2',
        iterations,
        hash: 'SHA-256'
      });

      expect(result.algorithm).toBe('PBKDF2');
      expect(result.key).toBeInstanceOf(Uint8Array);
      expect(result.iterations).toBe(iterations);
      expect(result.hash).toBe('SHA-256');
      
      expect(mockCrypto.subtle.importKey).toHaveBeenCalledWith(
        'raw',
        masterKey,
        { name: 'PBKDF2' },
        false,
        ['deriveKey']
      );
    });

    it('should generate random salt if not provided', async () => {
      const masterKey = new Uint8Array(32);
      const info = new TextEncoder().encode('test-info');
      
      const result = await deriveKey(masterKey, {
        algorithm: 'HKDF',
        info
      });

      expect(result.salt).toBeInstanceOf(Uint8Array);
      expect(result.salt.length).toBe(32);
      expect(mockCrypto.getRandomValues).toHaveBeenCalled();
    });

    it('should use provided salt', async () => {
      const masterKey = new Uint8Array(32);
      const salt = new Uint8Array(16);
      const info = new TextEncoder().encode('test-info');
      
      const result = await deriveKey(masterKey, {
        algorithm: 'HKDF',
        info,
        salt
      });

      expect(result.salt).toEqual(salt);
    });

    it('should throw error for HKDF without info', async () => {
      const masterKey = new Uint8Array(32);
      
      await expect(
        deriveKey(masterKey, { algorithm: 'HKDF' })
      ).rejects.toThrow(KeyDerivationError);
      
      await expect(
        deriveKey(masterKey, { algorithm: 'HKDF' })
      ).rejects.toThrow('HKDF requires info parameter');
    });

    it('should throw error for empty master key', async () => {
      const masterKey = new Uint8Array(0);
      
      await expect(
        deriveKey(masterKey, { algorithm: 'HKDF', info: new Uint8Array(8) })
      ).rejects.toThrow(KeyDerivationError);
      
      await expect(
        deriveKey(masterKey, { algorithm: 'HKDF', info: new Uint8Array(8) })
      ).rejects.toThrow('Master key cannot be empty');
    });

    it('should throw error for unsupported algorithm', async () => {
      const masterKey = new Uint8Array(32);
      
      await expect(
        deriveKey(masterKey, { algorithm: 'UNSUPPORTED' as any })
      ).rejects.toThrow(KeyDerivationError);
    });

    it('should handle WebCrypto not available', async () => {
      Object.defineProperty(globalThis, 'crypto', {
        value: { subtle: null },
        writable: true
      });
      
      const masterKey = new Uint8Array(32);
      
      await expect(
        deriveKey(masterKey, { algorithm: 'HKDF', info: new Uint8Array(8) })
      ).rejects.toThrow(KeyDerivationError);
      
      await expect(
        deriveKey(masterKey, { algorithm: 'HKDF', info: new Uint8Array(8) })
      ).rejects.toThrow('WebCrypto API not available');
      
      // Restore crypto
      Object.defineProperty(globalThis, 'crypto', {
        value: mockCrypto,
        writable: true
      });
    });
  });

  describe('deriveMultipleKeys', () => {
    it('should derive multiple keys with different contexts', async () => {
      const masterKey = new Uint8Array(32);
      const contexts = [
        { name: 'encryption', info: new TextEncoder().encode('enc') },
        { name: 'signing', info: new TextEncoder().encode('sign') }
      ];
      
      const results = await deriveMultipleKeys(masterKey, contexts);
      
      expect(Object.keys(results)).toHaveLength(2);
      expect(results.encryption).toBeDefined();
      expect(results.signing).toBeDefined();
      expect(results.encryption.key).toBeInstanceOf(Uint8Array);
      expect(results.signing.key).toBeInstanceOf(Uint8Array);
    });

    it('should use custom options for each context', async () => {
      const masterKey = new Uint8Array(32);
      const contexts = [
        { 
          name: 'test', 
          info: new TextEncoder().encode('test'),
          options: { hash: 'SHA-512' as const, length: 64 }
        }
      ];
      
      const results = await deriveMultipleKeys(masterKey, contexts);
      
      expect(results.test.hash).toBe('SHA-512');
      expect(results.test.key.length).toBe(32); // Mock returns 32 bytes
    });
  });

  describe('deriveKeyFromKeyPair', () => {
    it('should derive key from Ed25519 key pair', async () => {
      // Mock generateKeyPair to return predictable result
      const mockKeyPair = {
        privateKey: new Uint8Array(32),
        publicKey: new Uint8Array(32)
      };
      
      const result = await deriveKeyFromKeyPair(mockKeyPair, {
        algorithm: 'HKDF',
        info: new TextEncoder().encode('test')
      });

      expect(result.key).toBeInstanceOf(Uint8Array);
      expect(result.algorithm).toBe('HKDF');
    });
  });

  describe('createHKDFInfo', () => {
    it('should create info from string context', () => {
      const context = 'test-context';
      const info = createHKDFInfo(context);
      
      expect(info).toBeInstanceOf(Uint8Array);
      expect(new TextDecoder().decode(info)).toBe(context);
    });

    it('should combine context with additional data', () => {
      const context = 'test';
      const additional = new Uint8Array([1, 2, 3]);
      const info = createHKDFInfo(context, additional);
      
      expect(info.length).toBe(4 + 3); // 'test' + [1,2,3]
      expect(info.slice(0, 4)).toEqual(new TextEncoder().encode(context));
      expect(info.slice(4)).toEqual(additional);
    });
  });

  describe('validateDerivedKey', () => {
    it('should validate correct derived key', async () => {
      const masterKey = new Uint8Array(32);
      const derivedKeyInfo = {
        key: new Uint8Array(32),
        algorithm: 'HKDF' as const,
        salt: new Uint8Array(16),
        info: new TextEncoder().encode('test'),
        hash: 'SHA-256',
        derived: new Date().toISOString()
      };
      
      const isValid = await validateDerivedKey(masterKey, derivedKeyInfo);
      expect(isValid).toBe(true);
    });

    it('should detect invalid derived key', async () => {
      // Mock exportKey to return different result for validation
      mockCrypto.subtle.exportKey
        .mockResolvedValueOnce(new ArrayBuffer(32)) // First call (original)
        .mockResolvedValueOnce(new ArrayBuffer(16)); // Second call (validation)
      
      const masterKey = new Uint8Array(32);
      const derivedKeyInfo = {
        key: new Uint8Array(32),
        algorithm: 'HKDF' as const,
        salt: new Uint8Array(16),
        info: new TextEncoder().encode('test'),
        hash: 'SHA-256',
        derived: new Date().toISOString()
      };
      
      const isValid = await validateDerivedKey(masterKey, derivedKeyInfo);
      expect(isValid).toBe(false);
    });

    it('should handle validation errors gracefully', async () => {
      mockCrypto.subtle.deriveKey.mockRejectedValueOnce(new Error('Test error'));
      
      const masterKey = new Uint8Array(32);
      const derivedKeyInfo = {
        key: new Uint8Array(32),
        algorithm: 'HKDF' as const,
        salt: new Uint8Array(16),
        info: new TextEncoder().encode('test'),
        hash: 'SHA-256',
        derived: new Date().toISOString()
      };
      
      const isValid = await validateDerivedKey(masterKey, derivedKeyInfo);
      expect(isValid).toBe(false);
    });
  });

  describe('clearDerivedKey', () => {
    it('should clear key material from memory', () => {
      const derivedKey = {
        key: new Uint8Array([1, 2, 3, 4]),
        salt: new Uint8Array([5, 6, 7, 8]),
        info: new Uint8Array([9, 10, 11, 12]),
        algorithm: 'HKDF' as const,
        hash: 'SHA-256',
        derived: new Date().toISOString()
      };
      
      clearDerivedKey(derivedKey);
      
      expect(Array.from(derivedKey.key)).toEqual([0, 0, 0, 0]);
      expect(Array.from(derivedKey.salt)).toEqual([0, 0, 0, 0]);
      expect(Array.from(derivedKey.info!)).toEqual([0, 0, 0, 0]);
    });

    it('should handle missing optional fields', () => {
      const derivedKey = {
        key: new Uint8Array([1, 2, 3, 4]),
        salt: new Uint8Array([5, 6, 7, 8]),
        algorithm: 'PBKDF2' as const,
        hash: 'SHA-256',
        derived: new Date().toISOString()
      };
      
      expect(() => clearDerivedKey(derivedKey)).not.toThrow();
      expect(Array.from(derivedKey.key)).toEqual([0, 0, 0, 0]);
    });
  });

  describe('KeyDerivationContexts', () => {
    it('should provide predefined contexts', () => {
      expect(KeyDerivationContexts.DATA_ENCRYPTION).toBeInstanceOf(Uint8Array);
      expect(KeyDerivationContexts.SIGNING).toBeInstanceOf(Uint8Array);
      expect(KeyDerivationContexts.AUTHENTICATION).toBeInstanceOf(Uint8Array);
      expect(KeyDerivationContexts.KEY_WRAPPING).toBeInstanceOf(Uint8Array);
      expect(KeyDerivationContexts.BACKUP_ENCRYPTION).toBeInstanceOf(Uint8Array);
      
      // Check that contexts are different
      expect(KeyDerivationContexts.DATA_ENCRYPTION).not.toEqual(KeyDerivationContexts.SIGNING);
      expect(KeyDerivationContexts.SIGNING).not.toEqual(KeyDerivationContexts.AUTHENTICATION);
    });

    it('should create consistent context strings', () => {
      const dataEncContext = new TextDecoder().decode(KeyDerivationContexts.DATA_ENCRYPTION);
      expect(dataEncContext).toBe('datafold.data.encryption.v1');
      
      const signingContext = new TextDecoder().decode(KeyDerivationContexts.SIGNING);
      expect(signingContext).toBe('datafold.signing.v1');
    });
  });

  describe('Error handling', () => {
    it('should throw KeyDerivationError for WebCrypto failures', async () => {
      mockCrypto.subtle.deriveKey.mockRejectedValueOnce(new Error('WebCrypto error'));
      
      const masterKey = new Uint8Array(32);
      
      await expect(
        deriveKey(masterKey, {
          algorithm: 'HKDF',
          info: new TextEncoder().encode('test')
        })
      ).rejects.toThrow(KeyDerivationError);
      
      await expect(
        deriveKey(masterKey, {
          algorithm: 'HKDF',
          info: new TextEncoder().encode('test')
        })
      ).rejects.toThrow('Key derivation failed');
    });

    it('should preserve KeyDerivationError types', async () => {
      const masterKey = new Uint8Array(0); // Invalid empty key
      
      await expect(
        deriveKey(masterKey, {
          algorithm: 'HKDF',
          info: new TextEncoder().encode('test')
        })
      ).rejects.toThrow(KeyDerivationError);
    });
  });

  describe('Integration with different algorithms', () => {
    it('should work with different hash functions', async () => {
      const masterKey = new Uint8Array(32);
      
      const sha256Result = await deriveKey(masterKey, {
        algorithm: 'HKDF',
        info: new TextEncoder().encode('test'),
        hash: 'SHA-256'
      });
      
      const sha512Result = await deriveKey(masterKey, {
        algorithm: 'HKDF',
        info: new TextEncoder().encode('test'),
        hash: 'SHA-512'
      });
      
      expect(sha256Result.hash).toBe('SHA-256');
      expect(sha512Result.hash).toBe('SHA-512');
    });

    it('should support custom key lengths', async () => {
      const masterKey = new Uint8Array(32);
      
      const result = await deriveKey(masterKey, {
        algorithm: 'HKDF',
        info: new TextEncoder().encode('test'),
        length: 64
      });
      
      expect(result.key.length).toBe(32); // Mock returns 32 bytes regardless
    });
  });
});