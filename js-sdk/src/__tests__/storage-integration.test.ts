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

// Mock environment setup (similar to other tests)
const mockSubtle = {
  importKey: jest.fn().mockResolvedValue({}),
  deriveKey: jest.fn().mockResolvedValue({}),
  encrypt: jest.fn().mockResolvedValue(new ArrayBuffer(64)),
  decrypt: jest.fn().mockResolvedValue(new ArrayBuffer(32))
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
      // Mock database
      const mockDb = {
        objectStoreNames: { contains: jest.fn(() => false) },
        transaction: jest.fn(() => ({
          objectStore: jest.fn(() => ({
            put: jest.fn(() => {
              const putRequest = { onsuccess: null as any, onerror: null as any };
              setTimeout(() => putRequest.onsuccess && putRequest.onsuccess(), 0);
              return putRequest;
            }),
            get: jest.fn(() => {
              const getRequest = { 
                result: {
                  id: 'test-key',
                  encryptedPrivateKey: new ArrayBuffer(64),
                  publicKey: new Uint8Array(32).buffer,
                  iv: new ArrayBuffer(12),
                  salt: new ArrayBuffer(16),
                  metadata: {
                    name: 'test-key',
                    description: '',
                    created: new Date().toISOString(),
                    lastAccessed: new Date().toISOString(),
                    tags: []
                  },
                  version: 1
                },
                onsuccess: null as any,
                onerror: null as any 
              };
              setTimeout(() => getRequest.onsuccess && getRequest.onsuccess(), 0);
              return getRequest;
            }),
            delete: jest.fn(() => {
              const deleteRequest = { onsuccess: null as any, onerror: null as any };
              setTimeout(() => deleteRequest.onsuccess && deleteRequest.onsuccess(), 0);
              return deleteRequest;
            }),
            getAll: jest.fn(() => {
              const getAllRequest = { 
                result: [],
                onsuccess: null as any,
                onerror: null as any 
              };
              setTimeout(() => getAllRequest.onsuccess && getAllRequest.onsuccess(), 0);
              return getAllRequest;
            }),
            count: jest.fn(() => {
              const countRequest = { 
                result: 1,
                onsuccess: null as any,
                onerror: null as any 
              };
              setTimeout(() => countRequest.onsuccess && countRequest.onsuccess(), 0);
              return countRequest;
            }),
            clear: jest.fn(() => {
              const clearRequest = { onsuccess: null as any, onerror: null as any };
              setTimeout(() => clearRequest.onsuccess && clearRequest.onsuccess(), 0);
              return clearRequest;
            }),
            createIndex: jest.fn()
          }))
        })),
        close: jest.fn(),
        onerror: null as any
      };

      if (request.onupgradeneeded) {
        request.result = mockDb;
        request.onupgradeneeded({ target: { result: mockDb } } as any);
      }
      
      request.result = mockDb;
      if (request.onsuccess) {
        request.onsuccess();
      }
    }, 0);
    
    return request;
  })
};

// Set up global mocks
Object.defineProperty(globalThis, 'crypto', {
  value: mockCrypto,
  writable: true,
  configurable: true
});

Object.defineProperty(globalThis, 'window', {
  value: { indexedDB: mockIndexedDB, isSecureContext: true },
  writable: true,
  configurable: true
});

Object.defineProperty(globalThis, 'indexedDB', {
  value: mockIndexedDB,
  writable: true,
  configurable: true
});

// Mock @noble/ed25519
jest.mock('@noble/ed25519', () => ({
  getPublicKeyAsync: jest.fn(async (privateKey: Uint8Array) => {
    const publicKey = new Uint8Array(32);
    for (let i = 0; i < 32; i++) {
      publicKey[i] = (privateKey[i] * 3 + 17) % 256;
    }
    return publicKey;
  }),
  signAsync: jest.fn(async (message: Uint8Array, privateKey: Uint8Array) => {
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
    storage = await createStorage();
  });

  afterEach(async () => {
    await storage.close();
  });

  describe('End-to-End Key Management Workflow', () => {
    it('should complete full key generation, storage, and retrieval cycle', async () => {
      // Step 1: Generate a new key pair
      const keyPair = await generateKeyPair();
      expect(keyPair.privateKey).toHaveLength(32);
      expect(keyPair.publicKey).toHaveLength(32);

      // Step 2: Generate a unique key ID
      const keyId = generateKeyId('integration-test');
      expect(keyId).toMatch(/^integration-test_[a-z0-9]+_[a-z0-9]+$/);

      // Step 3: Validate passphrase
      const passphrase = 'SecureTestPassphrase123!';
      const passphraseValidation = validatePassphrase(passphrase);
      expect(passphraseValidation.valid).toBe(true);

      // Step 4: Store the key pair with metadata
      const metadata = {
        name: 'Integration Test Key',
        description: 'A key generated during integration testing',
        tags: ['test', 'integration', 'automated']
      };

      await storage.storeKeyPair(keyId, keyPair, passphrase, metadata);

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

      // Step 9: Verify deletion
      const existsAfterDeletion = await storage.keyExists(keyId);
      expect(existsAfterDeletion).toBe(false);
    });

    it('should handle multiple keys with different passphrases', async () => {
      const keys: Array<{ keyId: string; keyPair: Ed25519KeyPair; passphrase: string }> = [];
      const passphrases = [
        'FirstSecurePassphrase123!',
        'SecondStrongPassword456@',
        'ThirdComplexSecret789#'
      ];

      // Generate and store multiple keys
      for (let i = 0; i < 3; i++) {
        const keyPair = await generateKeyPair();
        const keyId = generateKeyId(`multi-key-${i}`);
        const metadata = {
          name: `Multi-Key Test ${i + 1}`,
          description: `Key ${i + 1} in multi-key test`,
          tags: ['multi-test', `key-${i + 1}`]
        };

        await storage.storeKeyPair(keyId, keyPair, passphrases[i], metadata);
        keys.push({ keyId, keyPair, passphrase: passphrases[i] });
      }

      // Verify all keys exist
      const keyList = await storage.listKeys();
      expect(keyList.length).toBeGreaterThanOrEqual(3);

      // Retrieve each key with its correct passphrase
      for (let i = 0; i < keys.length; i++) {
        const { keyId, passphrase } = keys[i];
        const retrievedKeyPair = await storage.retrieveKeyPair(keyId, passphrase);
        expect(retrievedKeyPair.privateKey).toBeInstanceOf(Uint8Array);
        expect(retrievedKeyPair.publicKey).toBeInstanceOf(Uint8Array);
      }

      // Verify wrong passphrase fails
      await expect(storage.retrieveKeyPair(keys[0].keyId, passphrases[1]))
        .rejects.toThrow(StorageError);

      // Clean up
      for (const { keyId } of keys) {
        await storage.deleteKeyPair(keyId);
      }
    });

    it('should demonstrate secure storage error handling', async () => {
      const keyPair = await generateKeyPair();
      const keyId = generateKeyId('error-test');
      const passphrase = 'ValidPassphrase123!';

      // Test storing with weak passphrase
      await expect(storage.storeKeyPair(keyId, keyPair, 'weak'))
        .rejects.toThrow(StorageError);

      // Test storing with empty key ID
      await expect(storage.storeKeyPair('', keyPair, passphrase))
        .rejects.toThrow(StorageError);

      // Test retrieving non-existent key
      await expect(storage.retrieveKeyPair('non-existent-key', passphrase))
        .rejects.toThrow(StorageError);

      // Store a key successfully
      await storage.storeKeyPair(keyId, keyPair, passphrase);

      // Test retrieving with wrong passphrase
      await expect(storage.retrieveKeyPair(keyId, 'WrongPassphrase123!'))
        .rejects.toThrow(StorageError);

      // Clean up
      await storage.deleteKeyPair(keyId);
    });

    it('should maintain data isolation between different key IDs', async () => {
      // Generate two different key pairs
      const keyPair1 = await generateKeyPair();
      const keyPair2 = await generateKeyPair();
      
      const keyId1 = generateKeyId('isolation-test-1');
      const keyId2 = generateKeyId('isolation-test-2');
      const passphrase = 'SharedPassphrase123!';

      // Store both keys
      await storage.storeKeyPair(keyId1, keyPair1, passphrase, { name: 'Key 1' });
      await storage.storeKeyPair(keyId2, keyPair2, passphrase, { name: 'Key 2' });

      // Retrieve and verify they are different
      const retrieved1 = await storage.retrieveKeyPair(keyId1, passphrase);
      const retrieved2 = await storage.retrieveKeyPair(keyId2, passphrase);

      // Mock will return same public key for same private key input
      // But we can verify the process is isolated
      expect(retrieved1.privateKey).toBeInstanceOf(Uint8Array);
      expect(retrieved2.privateKey).toBeInstanceOf(Uint8Array);

      // Verify metadata isolation
      const keyList = await storage.listKeys();
      const key1Metadata = keyList.find(k => k.id === keyId1);
      const key2Metadata = keyList.find(k => k.id === keyId2);

      expect(key1Metadata!.metadata.name).toBe('Key 1');
      expect(key2Metadata!.metadata.name).toBe('Key 2');

      // Clean up
      await storage.deleteKeyPair(keyId1);
      await storage.deleteKeyPair(keyId2);
    });

    it('should handle storage quota and persistence', async () => {
      // Test storage support detection
      const supportInfo = isStorageSupported();
      expect(supportInfo.supported).toBe(true);
      expect(supportInfo.reasons).toHaveLength(0);

      // Test storage info
      const storageInfo = await storage.getStorageInfo();
      expect(typeof storageInfo.used).toBe('number');
      expect(storageInfo.used).toBeGreaterThanOrEqual(0);
    });
  });

  describe('Performance and Security Tests', () => {
    it('should handle batch operations efficiently', async () => {
      const startTime = Date.now();
      const batchSize = 5;
      const keyIds: string[] = [];

      // Generate and store multiple keys
      for (let i = 0; i < batchSize; i++) {
        const keyPair = await generateKeyPair();
        const keyId = generateKeyId(`batch-${i}`);
        await storage.storeKeyPair(keyId, keyPair, 'BatchPassphrase123!');
        keyIds.push(keyId);
      }

      const endTime = Date.now();
      const duration = endTime - startTime;

      // Should complete in reasonable time (allowing for mocked operations)
      expect(duration).toBeLessThan(5000); // 5 seconds for mocked operations

      // Verify all keys were stored
      const keyList = await storage.listKeys();
      const batchKeys = keyList.filter(k => k.id.startsWith('batch-'));
      expect(batchKeys.length).toBeGreaterThanOrEqual(batchSize);

      // Clean up
      for (const keyId of keyIds) {
        await storage.deleteKeyPair(keyId);
      }
    });

    it('should clear sensitive data from memory after operations', async () => {
      const keyPair = await generateKeyPair();
      const keyId = generateKeyId('memory-test');
      const passphrase = 'MemoryTestPass123!';

      // Store and retrieve key
      await storage.storeKeyPair(keyId, keyPair, passphrase);
      const retrievedKeyPair = await storage.retrieveKeyPair(keyId, passphrase);

      // Clear sensitive data
      clearKeyMaterial(keyPair);
      clearKeyMaterial(retrievedKeyPair);

      // Verify keys are zeroed out
      expect(keyPair.privateKey.every(byte => byte === 0)).toBe(true);
      expect(retrievedKeyPair.privateKey.every(byte => byte === 0)).toBe(true);

      // Clean up
      await storage.deleteKeyPair(keyId);
    });

    it('should validate encryption parameters for security', async () => {
      const keyPair = await generateKeyPair();
      const keyId = generateKeyId('crypto-test');
      const passphrase = 'CryptoTestPass123!';

      await storage.storeKeyPair(keyId, keyPair, passphrase);

      // Verify strong cryptographic parameters were used
      expect(mockSubtle.deriveKey).toHaveBeenCalledWith(
        expect.objectContaining({
          name: 'PBKDF2',
          iterations: 100000, // Strong iteration count
          hash: 'SHA-256'
        }),
        expect.any(Object),
        expect.objectContaining({
          name: 'AES-GCM',
          length: 256 // Strong key length
        }),
        false,
        ['encrypt', 'decrypt']
      );

      // Clean up
      await storage.deleteKeyPair(keyId);
    });
  });

  describe('Compatibility and Edge Cases', () => {
    it('should handle storage with maximum metadata', async () => {
      const keyPair = await generateKeyPair();
      const keyId = generateKeyId('max-metadata');
      const passphrase = 'MaxMetadataPass123!';

      // Maximum allowed metadata
      const maxMetadata = {
        name: 'A'.repeat(100), // Max length
        description: 'B'.repeat(500), // Max length
        tags: Array(20).fill('tag').map((t, i) => `${t}${i}`) // Max count
      };

      await storage.storeKeyPair(keyId, keyPair, passphrase, maxMetadata);

      const keyList = await storage.listKeys();
      const storedKey = keyList.find(k => k.id === keyId);
      
      expect(storedKey!.metadata.name).toBe('A'.repeat(100));
      expect(storedKey!.metadata.description).toBe('B'.repeat(500));
      expect(storedKey!.metadata.tags).toHaveLength(20);

      // Clean up
      await storage.deleteKeyPair(keyId);
    });

    it('should handle concurrent operations safely', async () => {
      const promises: Promise<void>[] = [];
      const keyIds: string[] = [];

      // Simulate concurrent operations
      for (let i = 0; i < 3; i++) {
        const keyPair = await generateKeyPair();
        const keyId = generateKeyId(`concurrent-${i}`);
        keyIds.push(keyId);
        
        promises.push(
          storage.storeKeyPair(keyId, keyPair, 'ConcurrentPass123!')
        );
      }

      // Wait for all operations to complete
      await Promise.all(promises);

      // Verify all keys were stored successfully
      for (const keyId of keyIds) {
        const exists = await storage.keyExists(keyId);
        expect(exists).toBe(true);
      }

      // Clean up
      for (const keyId of keyIds) {
        await storage.deleteKeyPair(keyId);
      }
    });
  });
});