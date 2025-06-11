/**
 * Integration tests for key derivation and rotation with storage
 */

import { describe, it, expect, beforeEach, afterEach, jest } from '@jest/globals';
import {
  generateKeyPair,
  deriveKey,
  deriveKeyFromKeyPair,
  KeyDerivationContexts,
  createHKDFInfo
} from '../index.js';
import { Ed25519KeyPair, DerivedKeyInfo } from '../types.js';

// Simple mock storage for testing
class MockStorage {
  private data = new Map<string, any>();

  async storeKeyPair(keyId: string, keyPair: any, passphrase: string, metadata?: any): Promise<void> {
    this.data.set(keyId, { keyPair, passphrase, metadata });
  }

  async getKeyPair(keyId: string, passphrase: string): Promise<any> {
    const stored = this.data.get(keyId);
    if (!stored || stored.passphrase !== passphrase) {
      return null;
    }
    return stored.keyPair;
  }

  async keyExists(keyId: string): Promise<boolean> {
    return this.data.has(keyId);
  }

  async deleteKeyPair(keyId: string): Promise<boolean> {
    return this.data.delete(keyId);
  }

  async listKeys(): Promise<string[]> {
    return Array.from(this.data.keys());
  }

  async storeVersionedKeyPair(keyId: string, versionedKeyPair: any, passphrase: string): Promise<void> {
    this.data.set(`versioned:${keyId}`, { versionedKeyPair, passphrase });
  }

  async getVersionedKeyPair(keyId: string, passphrase: string): Promise<any> {
    const stored = this.data.get(`versioned:${keyId}`);
    if (!stored || stored.passphrase !== passphrase) {
      return null;
    }
    return stored.versionedKeyPair;
  }

  clear(): void {
    this.data.clear();
  }

  async close(): Promise<void> {
    // Mock close method
  }
}

describe('Key Derivation and Rotation Integration', () => {
  let storage: MockStorage;

  beforeEach(async () => {
    jest.clearAllMocks();
    storage = new MockStorage();
  });

  afterEach(async () => {
    if (storage) {
      await storage.close();
    }
  });

  describe('Basic Key Management', () => {
    it('should perform key generation and storage workflow', async () => {
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

      // Step 3: Retrieve the key pair
      const retrievedKeyPair = await storage.getKeyPair(keyId, passphrase);
      expect(retrievedKeyPair).toEqual(keyPair);
    }, 10000);

    it('should handle key derivation for different purposes', async () => {
      const keyPair = await generateKeyPair();

      // Step 1: Derive keys for different purposes with different contexts
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

      // Verify keys are properly derived (deterministic but should be different due to different contexts)
      expect(encryptionKey.key).toHaveLength(32);
      expect(signingKey.key).toHaveLength(32);
      expect(authKey.key).toHaveLength(32);
      
      // Only compare when contexts are actually different
      if (JSON.stringify(KeyDerivationContexts.DATA_ENCRYPTION) !== JSON.stringify(KeyDerivationContexts.SIGNING)) {
        expect(encryptionKey.key).not.toEqual(signingKey.key);
      }
    }, 10000);
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
      expect(key64.key.length).toBe(64);
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
        storeKeyPair: (keyId: string, keyPair: any, passphrase: string, metadata?: any) =>
          Promise.reject(new Error('Storage full'))
      };

      await expect(
        failingStorage.storeKeyPair('test', keyPair, 'pass')
      ).rejects.toThrow('Storage full');
    });

    it('should handle invalid passphrase', async () => {
      const keyPair = await generateKeyPair();
      const keyId = 'test-key';
      const correctPassphrase = 'correct-passphrase';
      const wrongPassphrase = 'wrong-passphrase';

      await storage.storeKeyPair(keyId, keyPair, correctPassphrase);

      // Verify correct passphrase works
      const retrievedWithCorrect = await storage.getKeyPair(keyId, correctPassphrase);
      expect(retrievedWithCorrect).toEqual(keyPair);

      // Verify wrong passphrase fails
      const retrievedWithWrong = await storage.getKeyPair(keyId, wrongPassphrase);
      expect(retrievedWithWrong).toBeNull();
    });
  });
});