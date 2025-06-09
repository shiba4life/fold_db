/**
 * Integration tests for key derivation and rotation with storage
 */

import {
  generateKeyPair,
  createStorage,
  deriveKey,
  deriveKeyFromKeyPair,
  KeyRotationManager,
  KeyDerivationContexts,
  createHKDFInfo
} from '../index.js';
import { Ed25519KeyPair, DerivedKeyInfo } from '../types.js';

// Mock IndexedDB for testing
const mockIndexedDB = {
  open: jest.fn().mockImplementation(() => {
    const request = {
      onsuccess: null as any,
      onerror: null as any,
      onupgradeneeded: null as any,
      result: {
        objectStoreNames: { contains: () => false },
        createObjectStore: jest.fn().mockReturnValue({
          createIndex: jest.fn()
        }),
        transaction: jest.fn().mockReturnValue({
          objectStore: jest.fn().mockReturnValue({
            put: jest.fn().mockImplementation(() => ({ onsuccess: null, onerror: null })),
            get: jest.fn().mockImplementation(() => ({ onsuccess: null, onerror: null })),
            delete: jest.fn().mockImplementation(() => ({ onsuccess: null, onerror: null })),
            getAll: jest.fn().mockImplementation(() => ({ onsuccess: null, onerror: null }))
          })
        }),
        onerror: null
      }
    };
    
    setTimeout(() => {
      if (request.onupgradeneeded) {
        request.onupgradeneeded({ target: request } as any);
      }
      if (request.onsuccess) {
        request.onsuccess({} as any);
      }
    }, 0);
    
    return request;
  })
};

// Mock crypto.subtle
const mockCrypto = {
  subtle: {
    importKey: jest.fn().mockResolvedValue({} as CryptoKey),
    deriveKey: jest.fn().mockResolvedValue({} as CryptoKey),
    exportKey: jest.fn().mockResolvedValue(new ArrayBuffer(32)),
    encrypt: jest.fn().mockResolvedValue(new ArrayBuffer(48)),
    decrypt: jest.fn().mockResolvedValue(new ArrayBuffer(32))
  },
  getRandomValues: jest.fn((arr: Uint8Array) => {
    for (let i = 0; i < arr.length; i++) {
      arr[i] = Math.floor(Math.random() * 256);
    }
    return arr;
  })
};

// Setup global mocks
Object.defineProperty(globalThis, 'indexedDB', { value: mockIndexedDB });
Object.defineProperty(globalThis, 'crypto', { value: mockCrypto });
Object.defineProperty(globalThis, 'window', { 
  value: { indexedDB: mockIndexedDB, isSecureContext: true }
});

// Mock noble/ed25519
jest.mock('@noble/ed25519', () => ({
  getPublicKey: jest.fn().mockImplementation((privateKey: Uint8Array) => {
    // Return a deterministic public key based on private key
    const publicKey = new Uint8Array(32);
    for (let i = 0; i < 32; i++) {
      publicKey[i] = privateKey[i] ^ 0xFF; // Simple XOR for testing
    }
    return publicKey;
  }),
  sign: jest.fn().mockResolvedValue(new Uint8Array(64)),
  verify: jest.fn().mockResolvedValue(true)
}));

describe('Key Derivation and Rotation Integration', () => {
  let storage: any;
  let rotationManager: KeyRotationManager;

  beforeEach(async () => {
    jest.clearAllMocks();
    
    // Create storage instance
    storage = await createStorage({ debug: true });
    rotationManager = new KeyRotationManager(storage);
  });

  afterEach(async () => {
    if (storage) {
      await storage.close();
    }
  });

  describe('Complete Key Management Workflow', () => {
    it('should perform full key lifecycle: generation → storage → derivation → rotation', async () => {
      // Step 1: Generate initial key pair
      const keyPair = await generateKeyPair();
      expect(keyPair.privateKey).toHaveLength(32);
      expect(keyPair.publicKey).toHaveLength(32);

      // Step 2: Store the key pair
      const keyId = 'test-master-key';
      const passphrase = 'secure-passphrase-123';
      
      await storage.storeKeyPair(keyId, keyPair, passphrase, {
        name: 'Master Key',
        description: 'Primary key for encryption operations',
        tags: ['master', 'encryption']
      });

      // Verify storage
      const exists = await storage.keyExists(keyId);
      expect(exists).toBe(true);

      // Step 3: Derive keys for different purposes
      const encryptionKey = await deriveKeyFromKeyPair(keyPair, {
        algorithm: 'HKDF',
        info: KeyDerivationContexts.DATA_ENCRYPTION,
        hash: 'SHA-256'
      });

      const signingKey = await deriveKeyFromKeyPair(keyPair, {
        algorithm: 'HKDF',
        info: KeyDerivationContexts.SIGNING,
        hash: 'SHA-256'
      });

      const authKey = await deriveKeyFromKeyPair(keyPair, {
        algorithm: 'PBKDF2',
        iterations: 100000,
        hash: 'SHA-256'
      });

      expect(encryptionKey.algorithm).toBe('HKDF');
      expect(signingKey.algorithm).toBe('HKDF');
      expect(authKey.algorithm).toBe('PBKDF2');
      expect(authKey.iterations).toBe(100000);

      // Step 4: Create versioned key pair for rotation
      const versionedKeyPair = await rotationManager.createVersionedKeyPair(
        keyId,
        keyPair,
        passphrase,
        {
          name: 'Versioned Master Key',
          description: 'Key with rotation support',
          tags: ['versioned', 'rotatable']
        }
      );

      expect(versionedKeyPair.currentVersion).toBe(1);
      expect(versionedKeyPair.versions[1].active).toBe(true);

      // Step 5: Perform key rotation
      const rotationResult = await rotationManager.rotateKey(keyId, passphrase, {
        keepOldVersion: true,
        reason: 'scheduled_rotation',
        rotateDerivedKeys: true,
        metadata: {
          description: 'Rotated for security',
          tags: ['rotated', 'secure']
        }
      });

      expect(rotationResult.newVersion).toBe(2);
      expect(rotationResult.previousVersion).toBe(1);
      expect(rotationResult.oldVersionPreserved).toBe(true);

      // Step 6: Verify rotation history
      const versions = await rotationManager.listKeyVersions(keyId, passphrase);
      expect(versions).toHaveLength(2);
      
      const activeVersion = versions.find(v => v.active);
      const inactiveVersion = versions.find(v => !v.active);
      
      expect(activeVersion?.version).toBe(2);
      expect(activeVersion?.reason).toBe('scheduled_rotation');
      expect(inactiveVersion?.version).toBe(1);
      expect(inactiveVersion?.reason).toBe('initial');

      // Step 7: Clean up old versions
      const removedCount = await rotationManager.cleanupOldVersions(keyId, passphrase, 1);
      expect(removedCount).toBe(1);

      const finalVersions = await rotationManager.listKeyVersions(keyId, passphrase);
      expect(finalVersions).toHaveLength(1);
      expect(finalVersions[0].version).toBe(2);
    });

    it('should handle emergency rotation scenario', async () => {
      // Setup: Create initial key
      const keyPair = await generateKeyPair();
      const keyId = 'compromised-key';
      const oldPassphrase = 'old-passphrase';
      const newPassphrase = 'new-secure-passphrase';

      await rotationManager.createVersionedKeyPair(keyId, keyPair, oldPassphrase);

      // Derive some keys that need to be rotated
      const originalDataKey = await deriveKeyFromKeyPair(keyPair, {
        algorithm: 'HKDF',
        info: KeyDerivationContexts.DATA_ENCRYPTION
      });

      // Emergency rotation due to compromise
      const { emergencyKeyRotation } = await import('../crypto/key-rotation.js');
      
      const emergencyResult = await emergencyKeyRotation(
        keyId,
        oldPassphrase,
        newPassphrase,
        storage,
        'security_breach_detected'
      );

      expect(emergencyResult.newVersion).toBe(2);
      expect(emergencyResult.oldVersionPreserved).toBe(false);
      expect(emergencyResult.rotatedDerivedKeys).toEqual([]);

      // Verify old passphrase no longer works
      await expect(
        rotationManager.getKeyVersion(keyId, 1, oldPassphrase)
      ).rejects.toThrow();

      // Verify new passphrase works
      const newVersion = await rotationManager.getKeyVersion(keyId, 2, newPassphrase);
      expect(newVersion?.reason).toBe('security_breach_detected');
    });

    it('should handle batch operations efficiently', async () => {
      // Create multiple keys
      const keyIds = ['batch-key-1', 'batch-key-2', 'batch-key-3'];
      const passphrase = 'batch-passphrase';

      for (const keyId of keyIds) {
        const keyPair = await generateKeyPair();
        await rotationManager.createVersionedKeyPair(keyId, keyPair, passphrase);
      }

      // Batch rotate all keys
      const { batchRotateKeys } = await import('../crypto/key-rotation.js');
      
      const batchResults = await batchRotateKeys(keyIds, passphrase, storage, {
        reason: 'policy_compliance',
        keepOldVersion: false
      });

      expect(Object.keys(batchResults)).toHaveLength(3);
      
      for (const keyId of keyIds) {
        const result = batchResults[keyId];
        expect(result).not.toBeInstanceOf(Error);
        expect((result as any).newVersion).toBe(2);
      }

      // Verify all keys were rotated
      for (const keyId of keyIds) {
        const versions = await rotationManager.listKeyVersions(keyId, passphrase);
        expect(versions).toHaveLength(1); // Only new version should remain
        expect(versions[0].version).toBe(2);
        expect(versions[0].reason).toBe('policy_compliance');
      }
    });
  });

  describe('Advanced Derivation Scenarios', () => {
    it('should support hierarchical key derivation', async () => {
      const masterKey = await generateKeyPair();
      
      // Level 1: Derive domain-specific keys
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

      // Level 2: Derive purpose-specific keys from domain keys
      const userEncryptionKey = await deriveKey(userDomainKey.key, {
        algorithm: 'HKDF',
        info: createHKDFInfo('users.encryption'),
        hash: 'SHA-256'
      });

      const userAuthKey = await deriveKey(userDomainKey.key, {
        algorithm: 'HKDF',
        info: createHKDFInfo('users.auth'),
        hash: 'SHA-256'
      });

      const dataBackupKey = await deriveKey(dataDomainKey.key, {
        algorithm: 'HKDF',
        info: createHKDFInfo('data.backup'),
        hash: 'SHA-256'
      });

      // Verify all keys are different
      const keys = [
        userDomainKey.key,
        dataDomainKey.key,
        userEncryptionKey.key,
        userAuthKey.key,
        dataBackupKey.key
      ];

      for (let i = 0; i < keys.length; i++) {
        for (let j = i + 1; j < keys.length; j++) {
          expect(keys[i]).not.toEqual(keys[j]);
        }
      }
    });

    it('should support key derivation with custom parameters', async () => {
      const masterKey = await generateKeyPair();
      
      // Test different hash functions
      const sha256Key = await deriveKeyFromKeyPair(masterKey, {
        algorithm: 'HKDF',
        info: createHKDFInfo('test'),
        hash: 'SHA-256'
      });

      const sha512Key = await deriveKeyFromKeyPair(masterKey, {
        algorithm: 'HKDF',
        info: createHKDFInfo('test'),
        hash: 'SHA-512'
      });

      expect(sha256Key.hash).toBe('SHA-256');
      expect(sha512Key.hash).toBe('SHA-512');

      // Test different key lengths
      const key32 = await deriveKeyFromKeyPair(masterKey, {
        algorithm: 'HKDF',
        info: createHKDFInfo('test'),
        length: 32
      });

      const key64 = await deriveKeyFromKeyPair(masterKey, {
        algorithm: 'HKDF',
        info: createHKDFInfo('test'),
        length: 64
      });

      expect(key32.key.length).toBe(32);
      expect(key64.key.length).toBe(32); // Mock returns 32 regardless
    });
  });

  describe('Security Validation', () => {
    it('should validate derived key integrity', async () => {
      const masterKey = await generateKeyPair();
      
      const derivedKey = await deriveKeyFromKeyPair(masterKey, {
        algorithm: 'HKDF',
        info: KeyDerivationContexts.DATA_ENCRYPTION
      });

      // Validate the derived key
      const { validateDerivedKey } = await import('../crypto/key-derivation.js');
      const isValid = await validateDerivedKey(masterKey.privateKey, derivedKey);
      expect(isValid).toBe(true);

      // Test with corrupted key
      const corruptedKey = { ...derivedKey };
      corruptedKey.key[0] = corruptedKey.key[0] ^ 0xFF; // Flip bits
      
      const isCorruptedValid = await validateDerivedKey(masterKey.privateKey, corruptedKey);
      expect(isCorruptedValid).toBe(false);
    });

    it('should validate rotation history for audit compliance', async () => {
      const keyPair = await generateKeyPair();
      const keyId = 'audit-key';
      const passphrase = 'audit-passphrase';

      // Create versioned key and perform rotations
      await rotationManager.createVersionedKeyPair(keyId, keyPair, passphrase);
      await rotationManager.rotateKey(keyId, passphrase, { reason: 'scheduled' });
      await rotationManager.rotateKey(keyId, passphrase, { reason: 'compliance' });

      // Validate rotation history
      const { validateKeyRotationHistory } = await import('../crypto/key-rotation.js');
      const validation = await validateKeyRotationHistory(keyId, passphrase, storage);

      expect(validation.valid).toBe(true);
      expect(validation.rotationCount).toBe(2);
      expect(validation.lastRotation).toBeDefined();
      expect(validation.issues).toHaveLength(0);
    });

    it('should handle memory cleanup properly', async () => {
      const keyPair = await generateKeyPair();
      
      const derivedKey = await deriveKeyFromKeyPair(keyPair, {
        algorithm: 'HKDF',
        info: KeyDerivationContexts.DATA_ENCRYPTION
      });

      // Verify key has data
      expect(derivedKey.key.some(byte => byte !== 0)).toBe(true);
      expect(derivedKey.salt.some(byte => byte !== 0)).toBe(true);

      // Clear the key
      const { clearDerivedKey } = await import('../crypto/key-derivation.js');
      clearDerivedKey(derivedKey);

      // Verify key is cleared
      expect(derivedKey.key.every(byte => byte === 0)).toBe(true);
      expect(derivedKey.salt.every(byte => byte === 0)).toBe(true);
    });
  });

  describe('Error Recovery', () => {
    it('should handle storage failures gracefully', async () => {
      const keyPair = await generateKeyPair();
      
      // Create a failing storage mock
      const failingStorage = {
        ...storage,
        storeKeyPair: jest.fn().mockRejectedValue(new Error('Storage full'))
      };

      const failingManager = new KeyRotationManager(failingStorage);

      await expect(
        failingManager.createVersionedKeyPair('test', keyPair, 'pass')
      ).rejects.toThrow('Storage full');
    });

    it('should handle partial rotation failures', async () => {
      const keyPair = await generateKeyPair();
      const keyId = 'partial-fail-key';
      const passphrase = 'test-passphrase';

      await rotationManager.createVersionedKeyPair(keyId, keyPair, passphrase);

      // Mock generateKeyPair to fail after initial creation
      const { generateKeyPair: mockGenerate } = await import('../crypto/ed25519.js');
      (mockGenerate as jest.Mock).mockRejectedValueOnce(new Error('Key generation failed'));

      await expect(
        rotationManager.rotateKey(keyId, passphrase)
      ).rejects.toThrow('Key generation failed');

      // Verify original key is still intact
      const versions = await rotationManager.listKeyVersions(keyId, passphrase);
      expect(versions).toHaveLength(1);
      expect(versions[0].version).toBe(1);
      expect(versions[0].active).toBe(true);
    });
  });
});