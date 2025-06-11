import {
  IndexedDBKeyStorage,
  createStorage,
  isStorageSupported,
  getStorageQuota,
  validatePassphrase,
  generateKeyId,
  validateKeyId,
  sanitizeKeyId,
  estimateKeyStorageSize,
  validateMetadata,
  withTimeout
} from '../storage/index';
import { generateKeyPair } from '../crypto/ed25519';
import { StorageError, Ed25519KeyPair, StoredKeyMetadata } from '../types';

// Mock IndexedDB for testing
class MockIDBDatabase {
  objectStoreNames = { contains: jest.fn() };
  transaction = jest.fn(() => new MockIDBTransaction());
  close = jest.fn();
  onerror = null;
}

const createMockRequest = (result: any = null) => ({
  result,
  error: null,
  onsuccess: null as any,
  onerror: null as any
});

class MockIDBObjectStore {
  put = jest.fn().mockImplementation(() => {
    const request = createMockRequest();
    setTimeout(() => request.onsuccess && request.onsuccess(), 0);
    return request;
  });
  get = jest.fn().mockImplementation(() => {
    const request = createMockRequest();
    setTimeout(() => request.onsuccess && request.onsuccess(), 0);
    return request;
  });
  delete = jest.fn().mockImplementation(() => {
    const request = createMockRequest();
    setTimeout(() => request.onsuccess && request.onsuccess(), 0);
    return request;
  });
  getAll = jest.fn().mockImplementation(() => {
    const request = createMockRequest([]);
    setTimeout(() => request.onsuccess && request.onsuccess(), 0);
    return request;
  });
  count = jest.fn().mockImplementation(() => {
    const request = createMockRequest(1);
    setTimeout(() => request.onsuccess && request.onsuccess(), 0);
    return request;
  });
  clear = jest.fn().mockImplementation(() => {
    const request = createMockRequest();
    setTimeout(() => request.onsuccess && request.onsuccess(), 0);
    return request;
  });
  createIndex = jest.fn();
}

class MockIDBTransaction {
  objectStore = jest.fn(() => new MockIDBObjectStore());
}

const mockIDBRequest = {
  result: null,
  error: null,
  onsuccess: null as any,
  onerror: null as any
};

const mockIndexedDB = {
  open: jest.fn(() => {
    const request = { ...mockIDBRequest };
    setTimeout(() => {
      if (request.onsuccess) {
        (request as any).result = new MockIDBDatabase();
        request.onsuccess();
      }
    }, 0);
    return request;
  })
};

// Mock WebCrypto
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
      array[i] = Math.floor(Math.random() * 256);
    }
    return array;
  })
};

// Set up global mocks
Object.defineProperty(globalThis, 'indexedDB', {
  value: mockIndexedDB,
  writable: true,
  configurable: true
});

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

describe('Secure Storage', () => {
  let storage: IndexedDBKeyStorage;
  let testKeyPair: Ed25519KeyPair;
  const testPassphrase = 'SecurePassphrase123!';
  const testKeyId = 'test-key-1';

  beforeEach(async () => {
    jest.clearAllMocks();
    storage = new IndexedDBKeyStorage();
    
    // Generate a test key pair
    testKeyPair = {
      privateKey: new Uint8Array(32).fill(1),
      publicKey: new Uint8Array(32).fill(2)
    };
  });

  afterEach(async () => {
    await storage.close();
  });

  describe('IndexedDBKeyStorage', () => {
    describe('initialization', () => {
      it('should initialize database successfully', async () => {
        expect(mockIndexedDB.open).toHaveBeenCalledWith('DataFoldKeyStorage', 1);
      });

      it('should throw error if IndexedDB not supported', async () => {
        // @ts-ignore
        delete (globalThis as any).window.indexedDB;
        
        const newStorage = new IndexedDBKeyStorage();
        
        // The constructor sets up dbReady promise that should reject
        await expect(async () => {
          await (newStorage as any).dbReady;
        }).rejects.toThrow(StorageError);
        
        // Restore for other tests
        (globalThis as any).window.indexedDB = mockIndexedDB;
      });
    });

    describe('storeKeyPair', () => {
      it('should store encrypted key pair successfully', async () => {
        const mockStore = new MockIDBObjectStore();
        const mockTransaction = new MockIDBTransaction();
        const mockDb = new MockIDBDatabase();
        
        mockDb.transaction.mockReturnValue(mockTransaction);
        mockTransaction.objectStore.mockReturnValue(mockStore);
        mockStore.put.mockImplementation(() => {
          const request = { ...mockIDBRequest };
          setTimeout(() => request.onsuccess && request.onsuccess(), 0);
          return request;
        });
        
        // Mock successful database initialization
        (storage as any).db = mockDb;
        (storage as any).dbReady = Promise.resolve();

        await storage.storeKeyPair(testKeyId, testKeyPair, testPassphrase);

        expect(mockDb.transaction).toHaveBeenCalledWith(['keys'], 'readwrite');
        expect(mockStore.put).toHaveBeenCalled();
        expect(mockSubtle.encrypt).toHaveBeenCalled();
      });

      it('should reject empty key ID', async () => {
        await expect(storage.storeKeyPair('', testKeyPair, testPassphrase))
          .rejects.toThrow(StorageError);
      });

      it('should reject weak passphrase', async () => {
        await expect(storage.storeKeyPair(testKeyId, testKeyPair, 'weak'))
          .rejects.toThrow(StorageError);
      });

      it('should include metadata when storing', async () => {
        const mockStore = new MockIDBObjectStore();
        const mockTransaction = new MockIDBTransaction();
        const mockDb = new MockIDBDatabase();
        
        mockDb.transaction.mockReturnValue(mockTransaction);
        mockTransaction.objectStore.mockReturnValue(mockStore);
        
        let storedData: any;
        mockStore.put.mockImplementation((data) => {
          storedData = data;
          const request = { ...mockIDBRequest };
          setTimeout(() => request.onsuccess && request.onsuccess(), 0);
          return request;
        });

        (storage as any).db = mockDb;
        (storage as any).dbReady = Promise.resolve();

        const metadata = {
          name: 'Test Key',
          description: 'A test key',
          tags: ['test', 'example']
        };

        await storage.storeKeyPair(testKeyId, testKeyPair, testPassphrase, metadata);

        expect(storedData.metadata.name).toBe('Test Key');
        expect(storedData.metadata.description).toBe('A test key');
        expect(storedData.metadata.tags).toEqual(['test', 'example']);
      });
    });

    describe('retrieveKeyPair', () => {
      it('should retrieve and decrypt key pair successfully', async () => {
        const mockStore = new MockIDBObjectStore();
        const mockTransaction = new MockIDBTransaction();
        const mockDb = new MockIDBDatabase();
        
        mockDb.transaction.mockReturnValue(mockTransaction);
        mockTransaction.objectStore.mockReturnValue(mockStore);
        
        const storedData = {
          id: testKeyId,
          encryptedPrivateKey: new ArrayBuffer(64),
          publicKey: testKeyPair.publicKey.buffer,
          iv: new ArrayBuffer(12),
          salt: new ArrayBuffer(16),
          metadata: {
            name: testKeyId,
            description: '',
            created: new Date().toISOString(),
            lastAccessed: new Date().toISOString(),
            tags: []
          },
          version: 1
        };
        
        mockStore.get.mockImplementation(() => {
          const request = { ...mockIDBRequest, result: storedData };
          setTimeout(() => request.onsuccess && request.onsuccess(), 0);
          return request;
        });

        // Mock successful decryption
        mockSubtle.decrypt.mockResolvedValue(testKeyPair.privateKey.buffer);

        (storage as any).db = mockDb;
        (storage as any).dbReady = Promise.resolve();

        const retrievedKeyPair = await storage.retrieveKeyPair(testKeyId, testPassphrase);

        expect(retrievedKeyPair.publicKey).toEqual(testKeyPair.publicKey);
        expect(mockSubtle.decrypt).toHaveBeenCalled();
      });

      it('should throw error for non-existent key', async () => {
        const mockStore = new MockIDBObjectStore();
        const mockTransaction = new MockIDBTransaction();
        const mockDb = new MockIDBDatabase();
        
        mockDb.transaction.mockReturnValue(mockTransaction);
        mockTransaction.objectStore.mockReturnValue(mockStore);
        mockStore.get.mockImplementation(() => {
          const request = { ...mockIDBRequest, result: undefined };
          setTimeout(() => request.onsuccess && request.onsuccess(), 0);
          return request;
        });

        (storage as any).db = mockDb;
        (storage as any).dbReady = Promise.resolve();

        await expect(storage.retrieveKeyPair('non-existent', testPassphrase))
          .rejects.toThrow(StorageError);
      });

      it('should throw error for incorrect passphrase', async () => {
        const mockStore = new MockIDBObjectStore();
        const mockTransaction = new MockIDBTransaction();
        const mockDb = new MockIDBDatabase();
        
        mockDb.transaction.mockReturnValue(mockTransaction);
        mockTransaction.objectStore.mockReturnValue(mockStore);
        
        const storedData = {
          id: testKeyId,
          encryptedPrivateKey: new ArrayBuffer(64),
          publicKey: testKeyPair.publicKey.buffer,
          iv: new ArrayBuffer(12),
          salt: new ArrayBuffer(16),
          metadata: {
            name: testKeyId,
            description: '',
            created: new Date().toISOString(),
            lastAccessed: new Date().toISOString(),
            tags: []
          },
          version: 1
        };
        
        mockStore.get.mockImplementation(() => {
          const request = { ...mockIDBRequest, result: storedData };
          setTimeout(() => request.onsuccess && request.onsuccess(), 0);
          return request;
        });

        // Mock decryption failure
        mockSubtle.decrypt.mockRejectedValue(new Error('Decryption failed'));

        (storage as any).db = mockDb;
        (storage as any).dbReady = Promise.resolve();

        await expect(storage.retrieveKeyPair(testKeyId, 'wrong-passphrase'))
          .rejects.toThrow(StorageError);
      });
    });

    describe('deleteKeyPair', () => {
      it('should delete key successfully', async () => {
        const mockStore = new MockIDBObjectStore();
        const mockTransaction = new MockIDBTransaction();
        const mockDb = new MockIDBDatabase();
        
        mockDb.transaction.mockReturnValue(mockTransaction);
        mockTransaction.objectStore.mockReturnValue(mockStore);
        mockStore.delete.mockImplementation(() => {
          const request = { ...mockIDBRequest };
          setTimeout(() => request.onsuccess && request.onsuccess(), 0);
          return request;
        });

        (storage as any).db = mockDb;
        (storage as any).dbReady = Promise.resolve();

        await storage.deleteKeyPair(testKeyId);

        expect(mockStore.delete).toHaveBeenCalledWith(testKeyId);
      });
    });

    describe('listKeys', () => {
      it('should list all stored keys', async () => {
        const mockStore = new MockIDBObjectStore();
        const mockTransaction = new MockIDBTransaction();
        const mockDb = new MockIDBDatabase();
        
        mockDb.transaction.mockReturnValue(mockTransaction);
        mockTransaction.objectStore.mockReturnValue(mockStore);
        
        const storedKeys = [
          {
            id: 'key1',
            metadata: { name: 'Key 1', description: '', created: '2025-01-01', lastAccessed: '2025-01-01', tags: [] }
          },
          {
            id: 'key2',
            metadata: { name: 'Key 2', description: '', created: '2025-01-02', lastAccessed: '2025-01-02', tags: [] }
          }
        ];
        
        mockStore.getAll.mockImplementation(() => {
          const request = { ...mockIDBRequest, result: storedKeys };
          setTimeout(() => request.onsuccess && request.onsuccess(), 0);
          return request;
        });

        (storage as any).db = mockDb;
        (storage as any).dbReady = Promise.resolve();

        const keyList = await storage.listKeys();

        expect(keyList).toHaveLength(2);
        expect(keyList[0].id).toBe('key1');
        expect(keyList[1].id).toBe('key2');
      });
    });

    describe('keyExists', () => {
      it('should return true for existing key', async () => {
        const mockStore = new MockIDBObjectStore();
        const mockTransaction = new MockIDBTransaction();
        const mockDb = new MockIDBDatabase();
        
        mockDb.transaction.mockReturnValue(mockTransaction);
        mockTransaction.objectStore.mockReturnValue(mockStore);
        mockStore.count.mockImplementation(() => {
          const request = { ...mockIDBRequest, result: 1 };
          setTimeout(() => request.onsuccess && request.onsuccess(), 0);
          return request;
        });

        (storage as any).db = mockDb;
        (storage as any).dbReady = Promise.resolve();

        const exists = await storage.keyExists(testKeyId);

        expect(exists).toBe(true);
      });

      it('should return false for non-existent key', async () => {
        const mockStore = new MockIDBObjectStore();
        const mockTransaction = new MockIDBTransaction();
        const mockDb = new MockIDBDatabase();
        
        mockDb.transaction.mockReturnValue(mockTransaction);
        mockTransaction.objectStore.mockReturnValue(mockStore);
        mockStore.count.mockImplementation(() => {
          const request = { ...mockIDBRequest, result: 0 };
          setTimeout(() => request.onsuccess && request.onsuccess(), 0);
          return request;
        });

        (storage as any).db = mockDb;
        (storage as any).dbReady = Promise.resolve();

        const exists = await storage.keyExists('non-existent');

        expect(exists).toBe(false);
      });
    });

    describe('clearAllKeys', () => {
      it('should clear all stored keys', async () => {
        const mockStore = new MockIDBObjectStore();
        const mockTransaction = new MockIDBTransaction();
        const mockDb = new MockIDBDatabase();
        
        mockDb.transaction.mockReturnValue(mockTransaction);
        mockTransaction.objectStore.mockReturnValue(mockStore);
        mockStore.clear.mockImplementation(() => {
          const request = { ...mockIDBRequest };
          setTimeout(() => request.onsuccess && request.onsuccess(), 0);
          return request;
        });

        (storage as any).db = mockDb;
        (storage as any).dbReady = Promise.resolve();

        await storage.clearAllKeys();

        expect(mockStore.clear).toHaveBeenCalled();
      });
    });
  });

  describe('Storage Utils', () => {
    describe('generateKeyId', () => {
      it('should generate unique key IDs', () => {
        const id1 = generateKeyId();
        const id2 = generateKeyId();
        
        expect(id1).not.toBe(id2);
        expect(id1).toMatch(/^key_[a-z0-9]+_[a-z0-9]+$/);
      });

      it('should use custom prefix', () => {
        const id = generateKeyId('custom');
        expect(id).toMatch(/^custom_[a-z0-9]+_[a-z0-9]+$/);
      });
    });

    describe('validateKeyId', () => {
      it('should validate correct key IDs', () => {
        const result = validateKeyId('valid-key-id');
        expect(result.valid).toBe(true);
      });

      it('should reject empty key IDs', () => {
        const result = validateKeyId('');
        expect(result.valid).toBe(false);
        expect(result.reason).toContain('empty');
      });

      it('should reject key IDs with unsafe characters', () => {
        const result = validateKeyId('key<>id');
        expect(result.valid).toBe(false);
        expect(result.reason).toContain('unsafe');
      });

      it('should reject overly long key IDs', () => {
        const longId = 'a'.repeat(101);
        const result = validateKeyId(longId);
        expect(result.valid).toBe(false);
        expect(result.reason).toContain('100 characters');
      });
    });

    describe('sanitizeKeyId', () => {
      it('should sanitize unsafe characters', () => {
        const sanitized = sanitizeKeyId('key<>:id');
        expect(sanitized).toBe('key___id');
      });

      it('should trim whitespace', () => {
        const sanitized = sanitizeKeyId('  key-id  ');
        expect(sanitized).toBe('key-id');
      });

      it('should throw for empty input', () => {
        expect(() => sanitizeKeyId('')).toThrow(StorageError);
      });
    });

    describe('validateMetadata', () => {
      it('should validate correct metadata', () => {
        const metadata = {
          name: 'Test Key',
          description: 'A test key',
          tags: ['test', 'example']
        };
        
        const result = validateMetadata(metadata);
        expect(result.valid).toBe(true);
        expect(result.issues).toHaveLength(0);
      });

      it('should allow undefined metadata', () => {
        const result = validateMetadata(undefined);
        expect(result.valid).toBe(true);
      });

      it('should reject non-object metadata', () => {
        const result = validateMetadata('invalid');
        expect(result.valid).toBe(false);
      });

      it('should reject overly long names', () => {
        const metadata = { name: 'a'.repeat(101) };
        const result = validateMetadata(metadata);
        expect(result.valid).toBe(false);
        expect(result.issues.some(issue => issue.includes('Name'))).toBe(true);
      });

      it('should reject too many tags', () => {
        const metadata = { tags: Array(21).fill('tag') };
        const result = validateMetadata(metadata);
        expect(result.valid).toBe(false);
        expect(result.issues.some(issue => issue.includes('20 tags'))).toBe(true);
      });
    });

    describe('withTimeout', () => {
      it('should resolve when promise resolves within timeout', async () => {
        const promise = Promise.resolve('success');
        const result = await withTimeout(promise, 1000);
        expect(result).toBe('success');
      });

      it('should reject when promise times out', async () => {
        const promise = new Promise(resolve => setTimeout(resolve, 2000));
        await expect(withTimeout(promise, 100)).rejects.toThrow(StorageError);
      });
    });
  });

  describe('Storage Support Detection', () => {
    describe('isStorageSupported', () => {
      it('should detect supported environment', () => {
        const result = isStorageSupported();
        expect(result.supported).toBe(true);
        expect(result.reasons).toHaveLength(0);
      });

      it('should detect missing IndexedDB', () => {
        // @ts-ignore
        delete (globalThis as any).window.indexedDB;
        
        const result = isStorageSupported();
        expect(result.supported).toBe(false);
        expect(result.reasons.some(r => r.includes('IndexedDB'))).toBe(true);
        
        // Restore for other tests
        (globalThis as any).window.indexedDB = mockIndexedDB;
      });

      it('should detect missing WebCrypto', () => {
        const originalCrypto = (globalThis as any).crypto;
        // @ts-ignore
        delete (globalThis as any).crypto;
        
        const result = isStorageSupported();
        expect(result.supported).toBe(false);
        expect(result.reasons.some(r => r.includes('WebCrypto'))).toBe(true);
        
        (globalThis as any).crypto = originalCrypto;
      });
    });
  });

  describe('Passphrase Validation', () => {
    describe('validatePassphrase', () => {
      it('should validate strong passphrase', () => {
        const result = validatePassphrase('StrongPassphrase123!');
        expect(result.valid).toBe(true);
      });

      it('should reject short passphrase', () => {
        const result = validatePassphrase('short');
        expect(result.valid).toBe(false);
        expect(result.issues.some(issue => issue.includes('8 characters'))).toBe(true);
      });

      it('should reject common weak patterns', () => {
        const result = validatePassphrase('password123');
        expect(result.valid).toBe(false);
        expect(result.issues.some(issue => issue.includes('weak patterns'))).toBe(true);
      });

      it('should recommend character variety', () => {
        const result = validatePassphrase('alllowercase');
        expect(result.issues.some(issue => issue.includes('uppercase'))).toBe(true);
        expect(result.issues.some(issue => issue.includes('numbers'))).toBe(true);
        expect(result.issues.some(issue => issue.includes('special'))).toBe(true);
      });
    });
  });

  describe('Error Handling', () => {
    it('should throw StorageError with proper error codes', async () => {
      try {
        await storage.storeKeyPair('', testKeyPair, testPassphrase);
        fail('Should have thrown');
      } catch (error) {
        expect(error).toBeInstanceOf(StorageError);
        expect((error as StorageError).code).toBe('INVALID_KEY_ID');
      }
    });

    it('should include descriptive error messages', async () => {
      try {
        await storage.storeKeyPair(testKeyId, testKeyPair, 'weak');
        fail('Should have thrown');
      } catch (error) {
        expect(error).toBeInstanceOf(StorageError);
        expect((error as StorageError).message).toContain('8 characters');
      }
    });
  });

  describe('Security Properties', () => {
    beforeEach(() => {
      jest.clearAllMocks();
    });

    it('should use strong encryption parameters', async () => {
      // Trigger encryption by storing a key
      await storage.storeKeyPair('security-test', testKeyPair, 'test-passphrase');
      
      // Verify PBKDF2 parameters
      expect(mockSubtle.deriveKey).toHaveBeenCalledWith(
        expect.objectContaining({
          name: 'PBKDF2',
          iterations: 100000,
          hash: 'SHA-256'
        }),
        expect.any(Object),
        expect.objectContaining({
          name: 'AES-GCM',
          length: 256
        }),
        false,
        ['encrypt', 'decrypt']
      );
    });

    it('should generate random IV and salt for each encryption', async () => {
      // Trigger multiple encryption operations
      await storage.storeKeyPair('random-test-1', testKeyPair, 'test-passphrase');
      await storage.storeKeyPair('random-test-2', testKeyPair, 'test-passphrase');
      
      const calls = mockCrypto.getRandomValues.mock.calls;
      expect(calls.length).toBeGreaterThan(0);
      
      // Should generate different random values each time
      if (calls.length >= 2) {
        const firstCall = calls[0][0];
        const secondCall = calls[1][0];
        expect(firstCall).not.toEqual(secondCall);
      }
    });

    it('should not store plaintext private keys', async () => {
      // Trigger encryption by storing a key
      await storage.storeKeyPair('encryption-test', testKeyPair, 'test-passphrase');
      
      // Verify that encryption is called before storage
      expect(mockSubtle.encrypt).toHaveBeenCalled();
    });
  });
});