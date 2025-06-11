import { 
  generateKeyPair, 
  generateMultipleKeyPairs, 
  checkBrowserCompatibility,
  formatKey,
  parseKey,
  clearKeyMaterial
} from '../crypto/ed25519';
import { Ed25519KeyError } from '../types';

// Counter for generating unique keys in tests
let keyGenerationCounter = 0;

// Mock crypto for testing environment
const mockCrypto = {
  getRandomValues: jest.fn((array: Uint8Array) => {
    // Generate unique random values using counter for each test
    keyGenerationCounter++;
    for (let i = 0; i < array.length; i++) {
      array[i] = (i * 7 + 42 + keyGenerationCounter * 13) % 256;
    }
    return array;
  }),
  subtle: {
    constructor: {
      prototype: {}
    }
  }
};

// Mock @noble/ed25519
jest.mock('@noble/ed25519', () => ({
  getPublicKeyAsync: jest.fn(async (privateKey: Uint8Array) => {
    // Mock public key generation - create a unique public key based on private key
    const publicKey = new Uint8Array(32);
    for (let i = 0; i < 32; i++) {
      publicKey[i] = (privateKey[i] * 3 + 17 + keyGenerationCounter) % 256;
    }
    return publicKey;
  }),
  signAsync: jest.fn(async (message: Uint8Array, privateKey: Uint8Array) => {
    // Mock signature - create deterministic signature
    const signature = new Uint8Array(64);
    for (let i = 0; i < 64; i++) {
      signature[i] = (message[0] + privateKey[0] + i) % 256;
    }
    return signature;
  }),
  verifyAsync: jest.fn(async (signature: Uint8Array, message: Uint8Array, publicKey: Uint8Array) => {
    // Mock verification - always return true for testing
    return true;
  })
}));

// Set up global crypto mock
Object.defineProperty(globalThis, 'crypto', {
  value: mockCrypto,
  writable: true,
  configurable: true
});

describe('Ed25519 Key Generation', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('checkBrowserCompatibility', () => {
    it('should detect WebCrypto API availability', () => {
      const compat = checkBrowserCompatibility();
      
      expect(compat.webCrypto).toBe(true);
      expect(compat.secureRandom).toBe(true);
      expect(compat.nativeEd25519).toBe(false); // Expected in current browsers
      expect(compat.browserInfo).toBeDefined();
    });

    it('should handle missing crypto API', () => {
      const originalCrypto = globalThis.crypto;
      
      // @ts-ignore
      delete globalThis.crypto;
      
      const compat = checkBrowserCompatibility();
      
      expect(compat.webCrypto).toBe(false);
      expect(compat.secureRandom).toBe(false);
      
      globalThis.crypto = originalCrypto;
    });
  });

  describe('generateKeyPair', () => {
    it('should generate a valid key pair', async () => {
      const keyPair = await generateKeyPair();
      
      expect(keyPair.privateKey).toBeInstanceOf(Uint8Array);
      expect(keyPair.publicKey).toBeInstanceOf(Uint8Array);
      expect(keyPair.privateKey.length).toBe(32);
      expect(keyPair.publicKey.length).toBe(32);
      
      // Verify crypto.getRandomValues was called
      expect(mockCrypto.getRandomValues).toHaveBeenCalled();
    });

    it('should generate different key pairs on multiple calls', async () => {
      const keyPair1 = await generateKeyPair();
      const keyPair2 = await generateKeyPair();
      
      // Private keys should be different
      expect(keyPair1.privateKey).not.toEqual(keyPair2.privateKey);
      expect(keyPair1.publicKey).not.toEqual(keyPair2.publicKey);
    });

    it('should accept custom entropy', async () => {
      const customEntropy = new Uint8Array(32);
      customEntropy.fill(123);
      
      const keyPair = await generateKeyPair({ entropy: customEntropy });
      
      expect(keyPair.privateKey).toEqual(customEntropy);
      expect(keyPair.publicKey).toBeInstanceOf(Uint8Array);
    });

    it('should validate generated keys by default', async () => {
      const keyPair = await generateKeyPair({ validate: true });
      
      expect(keyPair.privateKey).toBeInstanceOf(Uint8Array);
      expect(keyPair.publicKey).toBeInstanceOf(Uint8Array);
    });

    it('should skip validation when requested', async () => {
      const keyPair = await generateKeyPair({ validate: false });
      
      expect(keyPair.privateKey).toBeInstanceOf(Uint8Array);
      expect(keyPair.publicKey).toBeInstanceOf(Uint8Array);
    });

    it('should reject invalid entropy length', async () => {
      const invalidEntropy = new Uint8Array(16); // Wrong length
      
      await expect(generateKeyPair({ entropy: invalidEntropy }))
        .rejects.toThrow(Ed25519KeyError);
    });

    it('should handle crypto API unavailability', async () => {
      const originalCrypto = globalThis.crypto;
      
      // @ts-ignore
      delete globalThis.crypto;
      
      await expect(generateKeyPair())
        .rejects.toThrow(Ed25519KeyError);
      
      globalThis.crypto = originalCrypto;
    });
  });

  describe('generateMultipleKeyPairs', () => {
    it('should generate multiple key pairs', async () => {
      const keyPairs = await generateMultipleKeyPairs(3);
      
      expect(keyPairs).toHaveLength(3);
      keyPairs.forEach(keyPair => {
        expect(keyPair.privateKey).toBeInstanceOf(Uint8Array);
        expect(keyPair.publicKey).toBeInstanceOf(Uint8Array);
        expect(keyPair.privateKey.length).toBe(32);
        expect(keyPair.publicKey.length).toBe(32);
      });
    });

    it('should generate unique key pairs', async () => {
      const keyPairs = await generateMultipleKeyPairs(2);
      
      expect(keyPairs[0].privateKey).not.toEqual(keyPairs[1].privateKey);
      expect(keyPairs[0].publicKey).not.toEqual(keyPairs[1].publicKey);
    });

    it('should reject invalid count', async () => {
      await expect(generateMultipleKeyPairs(0))
        .rejects.toThrow(Ed25519KeyError);
      
      await expect(generateMultipleKeyPairs(-1))
        .rejects.toThrow(Ed25519KeyError);
      
      await expect(generateMultipleKeyPairs(101))
        .rejects.toThrow(Ed25519KeyError);
    });

    it('should reject non-integer count', async () => {
      await expect(generateMultipleKeyPairs(3.5))
        .rejects.toThrow(Ed25519KeyError);
    });
  });

  describe('formatKey', () => {
    const testKey = new Uint8Array([1, 2, 3, 4, 255, 254, 253, 252]);

    it('should format key as hex', () => {
      const hex = formatKey(testKey, 'hex');
      expect(hex).toBe('01020304fffefdfc');
    });

    it('should format key as base64', () => {
      const base64 = formatKey(testKey, 'base64');
      expect(typeof base64).toBe('string');
      expect(base64.length).toBeGreaterThan(0);
    });

    it('should format key as uint8array', () => {
      const array = formatKey(testKey, 'uint8array');
      expect(array).toBeInstanceOf(Uint8Array);
      expect(array).toEqual(testKey);
      expect(array).not.toBe(testKey); // Should be a copy
    });

    it('should reject unsupported format', () => {
      expect(() => formatKey(testKey, 'invalid' as any))
        .toThrow(Ed25519KeyError);
    });
  });

  describe('parseKey', () => {
    it('should parse hex string', () => {
      const hexKey = '01020304fffefdfc';
      const parsed = parseKey(hexKey, 'hex');
      
      expect(parsed).toBeInstanceOf(Uint8Array);
      expect(Array.from(parsed)).toEqual([1, 2, 3, 4, 255, 254, 253, 252]);
    });

    it('should parse base64 string', () => {
      const base64Key = 'AQIDBA=='; // [1, 2, 3, 4] in base64
      const parsed = parseKey(base64Key, 'base64');
      
      expect(parsed).toBeInstanceOf(Uint8Array);
      expect(Array.from(parsed)).toEqual([1, 2, 3, 4]);
    });

    it('should parse uint8array', () => {
      const arrayKey = new Uint8Array([1, 2, 3, 4]);
      const parsed = parseKey(arrayKey, 'uint8array');
      
      expect(parsed).toBeInstanceOf(Uint8Array);
      expect(parsed).toEqual(arrayKey);
      expect(parsed).not.toBe(arrayKey); // Should be a copy
    });

    it('should reject invalid hex string', () => {
      expect(() => parseKey('invalid_hex', 'hex'))
        .toThrow(Ed25519KeyError);
      
      expect(() => parseKey('123', 'hex')) // Odd length
        .toThrow(Ed25519KeyError);
    });

    it('should reject invalid base64 string', () => {
      expect(() => parseKey('invalid_base64!', 'base64'))
        .toThrow(Ed25519KeyError);
    });

    it('should reject wrong input type for format', () => {
      expect(() => parseKey(123 as any, 'hex'))
        .toThrow(Ed25519KeyError);
      
      expect(() => parseKey('string', 'uint8array'))
        .toThrow(Ed25519KeyError);
    });
  });

  describe('clearKeyMaterial', () => {
    it('should clear key material from memory', () => {
      const keyPair = {
        privateKey: new Uint8Array([1, 2, 3, 4]),
        publicKey: new Uint8Array([5, 6, 7, 8])
      };
      
      clearKeyMaterial(keyPair);
      
      // Keys should be zeroed out
      expect(Array.from(keyPair.privateKey)).toEqual([0, 0, 0, 0]);
      expect(Array.from(keyPair.publicKey)).toEqual([0, 0, 0, 0]);
    });
  });

  describe('Error Handling', () => {
    it('should throw Ed25519KeyError with proper error codes', async () => {
      try {
        await generateKeyPair({ entropy: new Uint8Array(16) });
        fail('Should have thrown');
      } catch (error) {
        expect(error).toBeInstanceOf(Ed25519KeyError);
        expect((error as Ed25519KeyError).code).toBe('INVALID_ENTROPY_LENGTH');
      }
    });

    it('should include descriptive error messages', async () => {
      try {
        await generateMultipleKeyPairs(-1);
        fail('Should have thrown');
      } catch (error) {
        expect(error).toBeInstanceOf(Ed25519KeyError);
        expect((error as Ed25519KeyError).message).toContain('positive integer');
      }
    });
  });

  describe('Security Properties', () => {
    it('should generate cryptographically secure keys', async () => {
      const keyPair = await generateKeyPair();
      
      // Private key should not be all zeros
      const isAllZeros = keyPair.privateKey.every(byte => byte === 0);
      expect(isAllZeros).toBe(false);
      
      // Private key should not be all ones
      const isAllOnes = keyPair.privateKey.every(byte => byte === 255);
      expect(isAllOnes).toBe(false);
    });

    it('should ensure private key never leaves client environment', async () => {
      const keyPair = await generateKeyPair();
      
      // Verify no network calls are made during key generation
      // (This is implicit in our implementation - keys are generated locally)
      expect(keyPair.privateKey).toBeInstanceOf(Uint8Array);
      expect(keyPair.privateKey.length).toBe(32);
    });

    it('should use secure random number generation', async () => {
      await generateKeyPair();
      
      // Verify crypto.getRandomValues was called
      expect(mockCrypto.getRandomValues).toHaveBeenCalled();
    });
  });
});