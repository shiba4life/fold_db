import {
  validateEnvironment,
  validatePrivateKey,
  validatePublicKey,
  validateSignature,
  validateKeyGenerationParams,
  validateHexString,
  validateBase64String,
  validateMessage,
  validateCount,
  requireSecureEnvironment
} from '../utils/validation';
import { Ed25519KeyError } from '../types';

describe('Validation Utilities', () => {
  describe('validateEnvironment', () => {
    it('should validate secure environment', () => {
      const result = validateEnvironment();
      
      expect(result.secure).toBe(true);
      expect(result.issues).toHaveLength(0);
    });

    it('should detect insecure HTTP in browser', () => {
      const originalLocation = globalThis.window?.location;
      
      Object.defineProperty(globalThis, 'window', {
        value: {
          location: {
            protocol: 'http:',
            hostname: 'example.com'
          },
          isSecureContext: false
        },
        writable: true,
        configurable: true
      });

      const result = validateEnvironment();
      
      expect(result.secure).toBe(false);
      expect(result.issues).toContain('HTTPS required for secure key operations in production');
      
      // Restore original
      if (originalLocation) {
        Object.defineProperty(globalThis, 'window', {
          value: { location: originalLocation },
          writable: true,
          configurable: true
        });
      }
    });

    it('should allow localhost over HTTP', () => {
      Object.defineProperty(globalThis, 'window', {
        value: {
          location: {
            protocol: 'http:',
            hostname: 'localhost'
          },
          isSecureContext: true
        },
        writable: true,
        configurable: true
      });

      const result = validateEnvironment();
      
      expect(result.secure).toBe(true);
    });
  });

  describe('validatePrivateKey', () => {
    it('should validate correct private key', () => {
      const validKey = new Uint8Array(32);
      validKey.fill(123); // Non-zero, non-all-ones
      
      expect(() => validatePrivateKey(validKey)).not.toThrow();
    });

    it('should reject wrong type', () => {
      expect(() => validatePrivateKey('invalid' as any))
        .toThrow(Ed25519KeyError);
    });

    it('should reject wrong length', () => {
      const invalidKey = new Uint8Array(16);
      
      expect(() => validatePrivateKey(invalidKey))
        .toThrow(Ed25519KeyError);
    });

    it('should reject all-zero key', () => {
      const zeroKey = new Uint8Array(32);
      zeroKey.fill(0);
      
      expect(() => validatePrivateKey(zeroKey))
        .toThrow(Ed25519KeyError);
    });

    it('should reject all-ones key', () => {
      const onesKey = new Uint8Array(32);
      onesKey.fill(255);
      
      expect(() => validatePrivateKey(onesKey))
        .toThrow(Ed25519KeyError);
    });
  });

  describe('validatePublicKey', () => {
    it('should validate correct public key', () => {
      const validKey = new Uint8Array(32);
      validKey.fill(42);
      
      expect(() => validatePublicKey(validKey)).not.toThrow();
    });

    it('should reject wrong type', () => {
      expect(() => validatePublicKey(123 as any))
        .toThrow(Ed25519KeyError);
    });

    it('should reject wrong length', () => {
      const invalidKey = new Uint8Array(31);
      
      expect(() => validatePublicKey(invalidKey))
        .toThrow(Ed25519KeyError);
    });
  });

  describe('validateSignature', () => {
    it('should validate correct signature', () => {
      const validSignature = new Uint8Array(64);
      validSignature.fill(17);
      
      expect(() => validateSignature(validSignature)).not.toThrow();
    });

    it('should reject wrong length', () => {
      const invalidSignature = new Uint8Array(63);
      
      expect(() => validateSignature(invalidSignature))
        .toThrow(Ed25519KeyError);
    });
  });

  describe('validateKeyGenerationParams', () => {
    it('should validate correct params', () => {
      const validEntropy = new Uint8Array(32);
      validEntropy.fill(55);
      
      expect(() => validateKeyGenerationParams({
        entropy: validEntropy,
        validate: true
      })).not.toThrow();
    });

    it('should reject invalid entropy length', () => {
      const invalidEntropy = new Uint8Array(16);
      
      expect(() => validateKeyGenerationParams({ entropy: invalidEntropy }))
        .toThrow(Ed25519KeyError);
    });

    it('should reject non-boolean validate option', () => {
      expect(() => validateKeyGenerationParams({ validate: 'true' as any }))
        .toThrow(Ed25519KeyError);
    });
  });

  describe('validateHexString', () => {
    it('should validate correct hex string', () => {
      expect(() => validateHexString('deadbeef')).not.toThrow();
      expect(() => validateHexString('DEADBEEF')).not.toThrow();
      expect(() => validateHexString('0123456789abcdef')).not.toThrow();
    });

    it('should reject non-string input', () => {
      expect(() => validateHexString(123 as any))
        .toThrow(Ed25519KeyError);
    });

    it('should reject odd-length string', () => {
      expect(() => validateHexString('abc'))
        .toThrow(Ed25519KeyError);
    });

    it('should reject invalid hex characters', () => {
      expect(() => validateHexString('abcg'))
        .toThrow(Ed25519KeyError);
      
      expect(() => validateHexString('abc!'))
        .toThrow(Ed25519KeyError);
    });
  });

  describe('validateBase64String', () => {
    it('should validate correct base64 string', () => {
      expect(() => validateBase64String('SGVsbG8=')).not.toThrow();
      expect(() => validateBase64String('SGVsbG8')).not.toThrow();
    });

    it('should reject non-string input', () => {
      expect(() => validateBase64String(123 as any))
        .toThrow(Ed25519KeyError);
    });

    it('should reject invalid base64 characters', () => {
      expect(() => validateBase64String('Hello!@#'))
        .toThrow(Ed25519KeyError);
    });

    it('should reject invalid length', () => {
      expect(() => validateBase64String('abc'))
        .toThrow(Ed25519KeyError);
    });
  });

  describe('validateMessage', () => {
    it('should validate correct message', () => {
      const validMessage = new Uint8Array([1, 2, 3, 4]);
      
      expect(() => validateMessage(validMessage)).not.toThrow();
    });

    it('should reject non-Uint8Array input', () => {
      expect(() => validateMessage('message' as any))
        .toThrow(Ed25519KeyError);
    });

    it('should reject empty message', () => {
      const emptyMessage = new Uint8Array(0);
      
      expect(() => validateMessage(emptyMessage))
        .toThrow(Ed25519KeyError);
    });
  });

  describe('validateCount', () => {
    it('should validate correct count', () => {
      expect(validateCount(5)).toBe(5);
      expect(validateCount(1)).toBe(1);
      expect(validateCount(100)).toBe(100);
    });

    it('should reject non-number input', () => {
      expect(() => validateCount('5'))
        .toThrow(Ed25519KeyError);
    });

    it('should reject non-integer', () => {
      expect(() => validateCount(5.5))
        .toThrow(Ed25519KeyError);
    });

    it('should reject negative count', () => {
      expect(() => validateCount(-1))
        .toThrow(Ed25519KeyError);
    });

    it('should reject zero count', () => {
      expect(() => validateCount(0))
        .toThrow(Ed25519KeyError);
    });

    it('should reject too large count', () => {
      expect(() => validateCount(1001))
        .toThrow(Ed25519KeyError);
    });
  });

  describe('requireSecureEnvironment', () => {
    it('should not throw in secure environment', () => {
      expect(() => requireSecureEnvironment()).not.toThrow();
    });

    it('should throw in insecure environment', () => {
      const originalCrypto = globalThis.crypto;
      
      // @ts-ignore
      delete globalThis.crypto;
      
      expect(() => requireSecureEnvironment())
        .toThrow(Ed25519KeyError);
      
      globalThis.crypto = originalCrypto;
    });
  });

  describe('Error Messages', () => {
    it('should provide descriptive error messages', () => {
      try {
        validateCount(-1);
        fail('Should have thrown');
      } catch (error) {
        expect(error).toBeInstanceOf(Ed25519KeyError);
        expect((error as Ed25519KeyError).message).toContain('positive');
        expect((error as Ed25519KeyError).code).toBe('INVALID_COUNT_NEGATIVE');
      }
    });

    it('should include error codes for programmatic handling', () => {
      const errors = [
        () => validatePrivateKey(new Uint8Array(0)),
        () => validateHexString('xyz'),
        () => validateBase64String('invalid!'),
        () => validateMessage(new Uint8Array(0))
      ];

      errors.forEach(errorFunc => {
        try {
          errorFunc();
          fail('Should have thrown');
        } catch (error) {
          expect(error).toBeInstanceOf(Ed25519KeyError);
          expect((error as Ed25519KeyError).code).toBeDefined();
          expect((error as Ed25519KeyError).code.length).toBeGreaterThan(0);
        }
      });
    });
  });
});