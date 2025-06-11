import { describe, it, expect, beforeEach, afterEach, jest } from '@jest/globals';
import {
  generateKeyPair,
  clearKeyMaterial
} from '../crypto/ed25519';
import {
  IndexedDBKeyStorage,
  createStorage,
  isStorageSupported,
  validatePassphrase,
  generateKeyId
} from '../storage/index';
import { Ed25519KeyPair, StorageError } from '../types';

// Create a persistent mock storage
const mockStorage = new Map();

// Create a store for encrypted data to ensure consistency
const encryptedDataStore = new Map<string, ArrayBuffer>();

// Mock environment setup (similar to other tests)
const mockSubtle: any = {
  importKey: jest.fn().mockImplementation(() => Promise.resolve({})),
  deriveKey: jest.fn().mockImplementation(() => Promise.resolve({})),
  encrypt: jest.fn().mockImplementation(async (algorithm: any, key: any, data: any) => {
    // Create a consistent encrypted result based on input
    const dataArray = data instanceof Uint8Array ? data : new Uint8Array(data);
    const encrypted = new ArrayBuffer(dataArray.length + 16);
    const view = new Uint8Array(encrypted);
    
    // Simple encryption simulation - XOR with a fixed pattern
    for (let i = 0; i < dataArray.length; i++) {
      view[i] = dataArray[i] ^ 0xAA;
    }
    
    // Store the mapping for consistent decryption
    const keyStr = Array.from(dataArray).join(',');
    encryptedDataStore.set(keyStr, encrypted);
    
    return encrypted;
  }),
  decrypt: jest.fn().mockImplementation(async (algorithm: any, key: any, encryptedData: any) => {
    const encrypted = encryptedData instanceof ArrayBuffer ? encryptedData : new ArrayBuffer(32);
    
    // Find the original data by checking our store
    for (const [originalKey, storedEncrypted] of encryptedDataStore.entries()) {
      if (storedEncrypted.byteLength === encrypted.byteLength) {
        const storedView = new Uint8Array(storedEncrypted);
        const encryptedView = new Uint8Array(encrypted);
        
        // Check if this is the same encrypted data (compare first few bytes)
        let matches = true;
        for (let i = 0; i < Math.min(16, storedView.length, encryptedView.length); i++) {
          if (storedView[i] !== encryptedView[i]) {
            matches = false;
            break;
          }
        }
        
        if (matches) {
          // Decrypt by reversing the XOR
          const originalData = originalKey.split(',').map(s => parseInt(s));
          const decrypted = new ArrayBuffer(originalData.length);
          const decryptedView = new Uint8Array(decrypted);
          
          for (let i = 0; i < originalData.length; i++) {
            decryptedView[i] = originalData[i];
          }
          
          return decrypted;
        }
      }
    }
    
    // Fallback - return something consistent
    const decrypted = new ArrayBuffer(32);
    const view = new Uint8Array(decrypted);
    const encryptedView = new Uint8Array(encrypted);
    
    // Reverse the encryption (XOR with same pattern)
    for (let i = 0; i < Math.min(32, encryptedView.length); i++) {
      view[i] = encryptedView[i] ^ 0xAA;
    }
    
    return decrypted;
  })
};

const mockCrypto = {
  subtle: mockSubtle,
  getRandomValues: jest.fn((array: Uint8Array) => {
    for (let i = 0; i < array.length; i++) {
      array[i] = (i * 7 + 42) % 256;
    }
    return array;
  })
};

const mockIndexedDB = {
  open: jest.fn(() => {
    const request = {
      result: null as any,
      error: null,
      onsuccess: null as any,
      onerror: null as any,
      onupgradeneeded: null as any
    };
    
    setTimeout(() => {
      // Mock database with persistent storage
      const mockDb = {
        objectStoreNames: { contains: jest.fn(() => false) },
        createObjectStore: jest.fn(() => ({
          createIndex: jest.fn(),
          put: jest.fn((data: any) => {
            mockStorage.set(data.id, data);
            const putRequest = { onsuccess: null as any, onerror: null as any };
            setTimeout(() => putRequest.onsuccess && putRequest.onsuccess(), 0);
            return putRequest;
          }),
          get: jest.fn((keyId: string) => {
            const getRequest = {
              result: mockStorage.get(keyId) || undefined,
              onsuccess: null as any,
              onerror: null as any
            };
            setTimeout(() => getRequest.onsuccess && getRequest.onsuccess(), 0);
            return getRequest;
          }),
          delete: jest.fn((keyId: string) => {
            mockStorage.delete(keyId);
            const deleteRequest = { onsuccess: null as any, onerror: null as any };
            setTimeout(() => deleteRequest.onsuccess && deleteRequest.onsuccess(), 0);
            return deleteRequest;
          }),
          getAll: jest.fn(() => {
            const getAllRequest = {
              result: Array.from(mockStorage.values()),
              onsuccess: null as any,
              onerror: null as any
            };
            setTimeout(() => getAllRequest.onsuccess && getAllRequest.onsuccess(), 0);
            return getAllRequest;
          }),
          count: jest.fn(() => {
            const countRequest = {
              result: mockStorage.size,
              onsuccess: null as any,
              onerror: null as any
            };
            setTimeout(() => countRequest.onsuccess && countRequest.onsuccess(), 0);
            return countRequest;
          }),
          clear: jest.fn(() => {
            mockStorage.clear();
            const clearRequest = { onsuccess: null as any, onerror: null as any };
            setTimeout(() => clearRequest.onsuccess && clearRequest.onsuccess(), 0);
            return clearRequest;
          })
        })),
        transaction: jest.fn(() => ({
          objectStore: jest.fn(() => ({
            put: jest.fn((data: any) => {
              mockStorage.set(data.id, data);
              const putRequest = { onsuccess: null as any, onerror: null as any };
              setTimeout(() => putRequest.onsuccess && putRequest.onsuccess(), 0);
              return putRequest;
            }),
            get: jest.fn((keyId: string) => {
              const getRequest = { 
                result: mockStorage.get(keyId) || undefined,
                onsuccess: null as any,
                onerror: null as any
              };
              setTimeout(() => getRequest.onsuccess && getRequest.onsuccess(), 0);
              return getRequest;
            }),
            delete: jest.fn((keyId: string) => {
              mockStorage.delete(keyId);
              const deleteRequest = { onsuccess: null as any, onerror: null as any };
              setTimeout(() => deleteRequest.onsuccess && deleteRequest.onsuccess(), 0);
              return deleteRequest;
            }),
            getAll: jest.fn(() => {
              const getAllRequest = {
                result: Array.from(mockStorage.values()),
                onsuccess: null as any,
                onerror: null as any
              };
              setTimeout(() => getAllRequest.onsuccess && getAllRequest.onsuccess(), 0);
              return getAllRequest;
            }),
            count: jest.fn(() => {
              const countRequest = {
                result: mockStorage.size,
                onsuccess: null as any,
                onerror: null as any
              };
              setTimeout(() => countRequest.onsuccess && countRequest.onsuccess(), 0);
              return countRequest;
            }),
            clear: jest.fn(() => {
              mockStorage.clear();
              const clearRequest = { onsuccess: null as any, onerror: null as any };
              setTimeout(() => clearRequest.onsuccess && clearRequest.onsuccess(), 0);
              return clearRequest;
            }),
            createIndex: jest.fn()
          }))
        })),
        close: jest.fn(),
        version: 1
      };
      
      request.result = mockDb;
      if (request.onupgradeneeded) {
        request.onupgradeneeded({ target: { result: mockDb } } as any);
      }
      if (request.onsuccess) {
        request.onsuccess();
      }
    }, 0);
    
    return request;
  })
};

// Mock globals
Object.defineProperty(global, 'crypto', {
  value: mockCrypto,
  writable: true
});

Object.defineProperty(global, 'indexedDB', {
  value: mockIndexedDB,
  writable: true
});

Object.defineProperty(global, 'window', {
  value: { indexedDB: mockIndexedDB },
  writable: true
});

// Mock @noble/ed25519
jest.mock('@noble/ed25519', () => ({
  getPublicKeyAsync: jest.fn(async (privateKey: Uint8Array) => {
    // Return a deterministic public key for testing
    const publicKey = new Uint8Array(32);
    for (let i = 0; i < 32; i++) {
      publicKey[i] = (privateKey[i] + i) % 256;
    }
    return publicKey;
  }),
  signAsync: jest.fn(async (message: Uint8Array, privateKey: Uint8Array) => {
    // Return a deterministic signature for testing
    const signature = new Uint8Array(64);
    for (let i = 0; i < 64; i++) {
      signature[i] = (message[0] + privateKey[0] + i) % 256;
    }
    return signature;
  }),
  verifyAsync: jest.fn(async () => true)
}));

describe('Storage Integration Tests', () => {
  let storage: IndexedDBKeyStorage;

  beforeEach(async () => {
    jest.clearAllMocks();
    mockStorage.clear();
    storage = new IndexedDBKeyStorage();
  });

  afterEach(async () => {
    mockStorage.clear();
  });

  describe('End-to-End Key Management Workflow', () => {
    it('should complete full key generation, storage, and retrieval cycle', async () => {
      // Step 1: Generate a new key pair
      const keyPair: Ed25519KeyPair = await generateKeyPair();
      expect(keyPair.privateKey).toHaveLength(32);
      expect(keyPair.publicKey).toHaveLength(32);

      // Step 2: Generate a unique key ID
      const keyId = generateKeyId('integration-test');
      expect(keyId).toMatch(/^integration-test_[a-z0-9]+_[a-z0-9]+$/);

      // Step 3: Validate passphrase requirements
      const passphrase = 'IntegrationTestPassphrase123!';
      const passphraseValidation = validatePassphrase(passphrase);
      expect(passphraseValidation.valid).toBe(true);

      // Step 4: Store the key pair with metadata
      await storage.storeKeyPair(keyId, keyPair, passphrase, {
        name: 'Integration Test Key',
        description: 'Comprehensive integration test key for storage validation',
        tags: ['test', 'integration', 'automated']
      });

      // Step 5: Verify key exists
      const exists = await storage.keyExists(keyId);
      expect(exists).toBe(true);

      // Step 6: Retrieve and verify the key pair
      const retrievedKeyPair = await storage.retrieveKeyPair(keyId, passphrase);
      expect(retrievedKeyPair.privateKey).toBeInstanceOf(Uint8Array);
      expect(retrievedKeyPair.publicKey).toBeInstanceOf(Uint8Array);
      expect(retrievedKeyPair.privateKey.length).toBe(32);
      expect(retrievedKeyPair.publicKey.length).toBe(32);

      // Step 7: List stored keys and verify metadata
      const keyList = await storage.listKeys();
      expect(keyList.length).toBeGreaterThan(0);
      
      const storedKey = keyList.find(k => k.id === keyId);
      expect(storedKey).toBeDefined();
      expect(storedKey!.metadata.name).toBe('Integration Test Key');
      expect(storedKey!.metadata.tags).toEqual(['test', 'integration', 'automated']);

      // Step 8: Clean up - clear sensitive data and delete key
      clearKeyMaterial(keyPair);
      clearKeyMaterial(retrievedKeyPair);
      
      await storage.deleteKeyPair(keyId);
      const existsAfterDeletion = await storage.keyExists(keyId);
      expect(existsAfterDeletion).toBe(false);
    });

    it('should handle multiple keys with different passphrases', async () => {
      const keys: Array<{ keyId: string; keyPair: Ed25519KeyPair }> = [];
      const passphrases = ['Pass1_Strong!', 'Pass2_Strong!', 'Pass3_Strong!'];
      
      // Store multiple keys
      for (let i = 0; i < 3; i++) {
        const keyPair = await generateKeyPair();
        const keyId = generateKeyId(`multi-test-${i}`);
        await storage.storeKeyPair(keyId, keyPair, passphrases[i], {
          name: `Test Key ${i + 1}`,
          description: `Test key ${i + 1} for multi-key testing`,
          tags: [`test-${i}`, 'multi']
        });
        keys.push({ keyId, keyPair });
      }

      // Verify all keys exist
      const keyList = await storage.listKeys();
      expect(keyList.length).toBeGreaterThanOrEqual(3);

      // Retrieve each key with its correct passphrase
      for (let i = 0; i < keys.length; i++) {
        const retrievedKeyPair = await storage.retrieveKeyPair(keys[i].keyId, passphrases[i]);
        expect(retrievedKeyPair.privateKey).toBeInstanceOf(Uint8Array);
        expect(retrievedKeyPair.publicKey).toBeInstanceOf(Uint8Array);
      }

      // Test that wrong passphrase fails
      await expect(storage.retrieveKeyPair(keys[0].keyId, passphrases[1]))
        .rejects.toThrow(StorageError);

      // Clean up
      for (const key of keys) {
        await storage.deleteKeyPair(key.keyId);
      }
    });

    it('should demonstrate secure storage error handling', async () => {
      const keyPair = await generateKeyPair();
      const keyId = generateKeyId('error-test');
      const passphrase = 'ErrorTestPassphrase123!';

      // Test weak passphrase rejection
      await expect(storage.storeKeyPair(keyId, keyPair, 'weak'))
        .rejects.toThrow(StorageError);

      // Test empty key ID rejection
      await expect(storage.storeKeyPair('', keyPair, passphrase))
        .rejects.toThrow(StorageError);

      // Test retrieving non-existent key
      await expect(storage.retrieveKeyPair('non-existent-key', passphrase))
        .rejects.toThrow(StorageError);

      // Store a key successfully
      await storage.storeKeyPair(keyId, keyPair, passphrase);

      // Test wrong passphrase
      await expect(storage.retrieveKeyPair(keyId, 'WrongPassphrase123!'))
        .rejects.toThrow(StorageError);

      // Clean up
      await storage.deleteKey(keyId);
    });

    it('should maintain data isolation between different key IDs', async () => {
      const keyPair1 = await generateKeyPair();
      const keyPair2 = await generateKeyPair();
      const keyId1 = generateKeyId('isolation-1');
      const keyId2 = generateKeyId('isolation-2');
      const passphrase = 'IsolationTestPassphrase123!';

      await storage.storeKeyPair(keyId1, keyPair1, passphrase, {
        name: 'Key 1',
        description: 'First isolation test key'
      });

      await storage.storeKeyPair(keyId2, keyPair2, passphrase, {
        name: 'Key 2',
        description: 'Second isolation test key'
      });

      const retrieved1 = await storage.retrieveKeyPair(keyId1, passphrase);
      const retrieved2 = await storage.retrieveKeyPair(keyId2, passphrase);

      expect(retrieved1.privateKey).toBeInstanceOf(Uint8Array);
      expect(retrieved2.privateKey).toBeInstanceOf(Uint8Array);

      // Verify metadata isolation
      const keyList = await storage.listKeys();
      const key1Metadata = keyList.find(k => k.id === keyId1);
      const key2Metadata = keyList.find(k => k.id === keyId2);

      expect(key1Metadata!.metadata.name).toBe('Key 1');
      expect(key2Metadata!.metadata.name).toBe('Key 2');

      // Clean up
      await storage.deleteKey(keyId1);
      await storage.deleteKey(keyId2);
    });

    it('should handle storage quota and persistence', async () => {
      // Test storage support detection
      const supportInfo = isStorageSupported();
      expect(supportInfo.supported).toBe(true);
      expect(supportInfo.reasons).toHaveLength(0);

      // Test storage info retrieval
      const storageInfo = await storage.getStorageInfo();
      expect(typeof storageInfo.used).toBe('number');
      expect(storageInfo.used).toBeGreaterThanOrEqual(0);
    });
  });

  describe('Performance and Security Tests', () => {
    it('should handle batch operations efficiently', async () => {
      const batchSize = 5;
      const keyIds: string[] = [];
      const passphrase = 'BatchTestPassphrase123!';

      const startTime = Date.now();

      // Store multiple keys
      for (let i = 0; i < batchSize; i++) {
        const keyPair = await generateKeyPair();
        const keyId = generateKeyId(`batch-${i}`);
        await storage.storeKeyPair(keyId, keyPair, passphrase);
        keyIds.push(keyId);
      }

      const duration = Date.now() - startTime;
      expect(duration).toBeLessThan(5000); // 5 seconds for mocked operations

      // Verify all keys were stored
      const keyList = await storage.listKeys();
      const batchKeys = keyList.filter(k => k.id.startsWith('batch-'));
      expect(batchKeys.length).toBeGreaterThanOrEqual(batchSize);

      // Clean up
      for (const keyId of keyIds) {
        await storage.deleteKey(keyId);
      }
    });

    it('should clear sensitive data from memory after operations', async () => {
      const keyPair = await generateKeyPair();
      const keyId = generateKeyId('memory-test');
      const passphrase = 'MemoryTestPassphrase123!';

      await storage.storeKeyPair(keyId, keyPair, passphrase);
      const retrievedKeyPair = await storage.retrieveKeyPair(keyId, passphrase);

      // Clear sensitive data
      clearKeyMaterial(keyPair);
      clearKeyMaterial(retrievedKeyPair);

      // Verify data has been cleared (all zeros)
      expect(keyPair.privateKey.every(byte => byte === 0)).toBe(true);
      expect(retrievedKeyPair.privateKey.every(byte => byte === 0)).toBe(true);

      await storage.deleteKey(keyId);
    });

    it('should validate encryption parameters for security', async () => {
      const keyPair = await generateKeyPair();
      const keyId = generateKeyId('encryption-test');
      const passphrase = 'EncryptionTestPassphrase123!';

      await storage.storeKeyPair(keyId, keyPair, passphrase);

      // Verify PBKDF2 parameters
      expect(mockSubtle.deriveKey).toHaveBeenCalledWith(
        expect.objectContaining({
          name: 'PBKDF2',
          iterations: 100000
        }),
        expect.any(Object),
        expect.objectContaining({
          name: 'AES-GCM',
          length: 256
        }),
        false,
        ['encrypt', 'decrypt']
      );

      await storage.deleteKey(keyId);
    });
  });

  describe('Compatibility and Edge Cases', () => {
    it('should handle storage with maximum metadata', async () => {
      const keyPair = await generateKeyPair();
      const keyId = generateKeyId('max-metadata');
      const passphrase = 'MaxMetadataTestPassphrase123!';

      await storage.storeKeyPair(keyId, keyPair, passphrase, {
        name: 'A'.repeat(100),
        description: 'B'.repeat(500),
        tags: Array.from({ length: 20 }, (_, i) => `tag-${i}`)
      });

      const keyList = await storage.listKeys();
      const storedKey = keyList.find(k => k.id === keyId);
      
      expect(storedKey!.metadata.name).toBe('A'.repeat(100));
      expect(storedKey!.metadata.description).toBe('B'.repeat(500));
      expect(storedKey!.metadata.tags).toHaveLength(20);

      await storage.deleteKey(keyId);
    });

    it('should handle concurrent operations safely', async () => {
      const operations: Array<Promise<string>> = [];
      const passphrase = 'ConcurrentTestPassphrase123!';

      // Start multiple concurrent operations
      for (let i = 0; i < 3; i++) {
        operations.push((async () => {
          const keyPair = await generateKeyPair();
          const keyId = generateKeyId(`concurrent-${i}`);
          await storage.storeKeyPair(keyId, keyPair, passphrase);
          return keyId;
        })());
      }

      const keyIds = await Promise.all(operations);

      // Verify all operations completed successfully
      for (const keyId of keyIds) {
        const exists = await storage.keyExists(keyId);
        expect(exists).toBe(true);
        await storage.deleteKey(keyId);
      }
    });
  });
});