import { Ed25519KeyPair, StorageError, StoredKeyMetadata, StorageOptions, KeyStorageInterface } from '../types.js';
import { formatKey, parseKey } from '../crypto/ed25519.js';

/**
 * Constants for IndexedDB storage
 */
const DB_NAME = 'DataFoldKeyStorage';
const DB_VERSION = 1;
const STORE_NAME = 'keys';
const ENCRYPTION_ALGORITHM = 'AES-GCM';
const KEY_DERIVATION_ALGORITHM = 'PBKDF2';
const IV_LENGTH = 12; // AES-GCM recommended IV length
const SALT_LENGTH = 16;
const PBKDF2_ITERATIONS = 100000; // OWASP recommended minimum

/**
 * Encrypted key data structure stored in IndexedDB
 */
interface EncryptedKeyData {
  id: string;
  encryptedPrivateKey: ArrayBuffer;
  publicKey: ArrayBuffer;
  iv: ArrayBuffer;
  salt: ArrayBuffer;
  metadata: StoredKeyMetadata;
  version: number;
}

/**
 * IndexedDB-based secure key storage implementation
 * Stores encrypted private keys in browser's IndexedDB with proper isolation
 */
export class IndexedDBKeyStorage implements KeyStorageInterface {
  private db: IDBDatabase | null = null;
  private dbReady: Promise<void> | null = null;

  constructor(private options: StorageOptions = {}) {
    this.dbReady = this.initializeDatabase();
  }

  /**
   * Initialize IndexedDB database and object store
   */
  private async initializeDatabase(): Promise<void> {
    return new Promise((resolve, reject) => {
      if (!window.indexedDB) {
        reject(new StorageError('IndexedDB not supported in this browser', 'INDEXEDDB_NOT_SUPPORTED'));
        return;
      }

      const request = window.indexedDB.open(DB_NAME, DB_VERSION);

      request.onerror = () => {
        reject(new StorageError(
          `Failed to open IndexedDB: ${request.error?.message}`,
          'INDEXEDDB_OPEN_FAILED'
        ));
      };

      request.onsuccess = () => {
        this.db = request.result;
        
        // Add error handler for database
        this.db.onerror = (event) => {
          console.error('IndexedDB error:', event);
        };

        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;
        
        // Create object store for keys
        if (!db.objectStoreNames.contains(STORE_NAME)) {
          const store = db.createObjectStore(STORE_NAME, { keyPath: 'id' });
          
          // Create indices for efficient querying
          store.createIndex('publicKey', 'publicKey', { unique: false });
          store.createIndex('created', 'metadata.created', { unique: false });
        }
      };
    });
  }

  /**
   * Derive encryption key from passphrase using PBKDF2
   */
  private async deriveEncryptionKey(passphrase: string, salt: Uint8Array): Promise<CryptoKey> {
    if (!crypto.subtle) {
      throw new StorageError('WebCrypto not available', 'WEBCRYPTO_NOT_AVAILABLE');
    }

    // Import passphrase as key material
    const keyMaterial = await crypto.subtle.importKey(
      'raw',
      new TextEncoder().encode(passphrase),
      { name: KEY_DERIVATION_ALGORITHM },
      false,
      ['deriveKey']
    );

    // Derive encryption key using PBKDF2
    return await crypto.subtle.deriveKey(
      {
        name: KEY_DERIVATION_ALGORITHM,
        salt: salt,
        iterations: PBKDF2_ITERATIONS,
        hash: 'SHA-256'
      },
      keyMaterial,
      { name: ENCRYPTION_ALGORITHM, length: 256 },
      false,
      ['encrypt', 'decrypt']
    );
  }

  /**
   * Encrypt private key data
   */
  private async encryptPrivateKey(
    privateKey: Uint8Array, 
    passphrase: string
  ): Promise<{ encryptedData: ArrayBuffer; iv: ArrayBuffer; salt: ArrayBuffer }> {
    // Generate random salt and IV
    const salt = crypto.getRandomValues(new Uint8Array(SALT_LENGTH));
    const iv = crypto.getRandomValues(new Uint8Array(IV_LENGTH));

    // Derive encryption key
    const encryptionKey = await this.deriveEncryptionKey(passphrase, salt);

    // Encrypt private key
    const encryptedData = await crypto.subtle.encrypt(
      { name: ENCRYPTION_ALGORITHM, iv: iv },
      encryptionKey,
      privateKey
    );

    return { encryptedData, iv: iv.buffer, salt: salt.buffer };
  }

  /**
   * Decrypt private key data
   */
  private async decryptPrivateKey(
    encryptedData: ArrayBuffer,
    iv: ArrayBuffer,
    salt: ArrayBuffer,
    passphrase: string
  ): Promise<Uint8Array> {
    // Derive encryption key
    const encryptionKey = await this.deriveEncryptionKey(passphrase, new Uint8Array(salt));

    // Decrypt private key
    const decryptedData = await crypto.subtle.decrypt(
      { name: ENCRYPTION_ALGORITHM, iv: iv },
      encryptionKey,
      encryptedData
    );

    return new Uint8Array(decryptedData);
  }

  /**
   * Store encrypted key pair in IndexedDB
   */
  async storeKeyPair(
    keyId: string, 
    keyPair: Ed25519KeyPair, 
    passphrase: string,
    metadata: Partial<StoredKeyMetadata> = {}
  ): Promise<void> {
    await this.dbReady;
    
    if (!this.db) {
      throw new StorageError('Database not initialized', 'DB_NOT_INITIALIZED');
    }

    if (!keyId || keyId.trim().length === 0) {
      throw new StorageError('Key ID cannot be empty', 'INVALID_KEY_ID');
    }

    if (!passphrase || passphrase.length < 8) {
      throw new StorageError('Passphrase must be at least 8 characters', 'WEAK_PASSPHRASE');
    }

    try {
      // Encrypt private key
      const { encryptedData, iv, salt } = await this.encryptPrivateKey(keyPair.privateKey, passphrase);

      // Prepare metadata
      const fullMetadata: StoredKeyMetadata = {
        name: metadata.name || keyId,
        description: metadata.description || '',
        created: metadata.created || new Date().toISOString(),
        lastAccessed: new Date().toISOString(),
        tags: metadata.tags || []
      };

      // Prepare encrypted key data
      const encryptedKeyData: EncryptedKeyData = {
        id: keyId,
        encryptedPrivateKey: encryptedData,
        publicKey: keyPair.publicKey.buffer,
        iv,
        salt,
        metadata: fullMetadata,
        version: 1
      };

      // Store in IndexedDB
      return new Promise((resolve, reject) => {
        const transaction = this.db!.transaction([STORE_NAME], 'readwrite');
        const store = transaction.objectStore(STORE_NAME);
        
        const request = store.put(encryptedKeyData);
        
        request.onsuccess = () => resolve();
        request.onerror = () => {
          reject(new StorageError(
            `Failed to store key: ${request.error?.message}`,
            'STORAGE_FAILED'
          ));
        };
      });

    } catch (error) {
      if (error instanceof StorageError) {
        throw error;
      }
      throw new StorageError(
        `Encryption failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'ENCRYPTION_FAILED'
      );
    }
  }

  /**
   * Retrieve and decrypt key pair from IndexedDB
   */
  async retrieveKeyPair(keyId: string, passphrase: string): Promise<Ed25519KeyPair> {
    await this.dbReady;
    
    if (!this.db) {
      throw new StorageError('Database not initialized', 'DB_NOT_INITIALIZED');
    }

    if (!keyId || keyId.trim().length === 0) {
      throw new StorageError('Key ID cannot be empty', 'INVALID_KEY_ID');
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readonly');
      const store = transaction.objectStore(STORE_NAME);
      
      const request = store.get(keyId);
      
      request.onsuccess = async () => {
        const encryptedKeyData: EncryptedKeyData | undefined = request.result;
        
        if (!encryptedKeyData) {
          reject(new StorageError(`Key not found: ${keyId}`, 'KEY_NOT_FOUND'));
          return;
        }

        try {
          // Decrypt private key
          const privateKey = await this.decryptPrivateKey(
            encryptedKeyData.encryptedPrivateKey,
            encryptedKeyData.iv,
            encryptedKeyData.salt,
            passphrase
          );

          // Update last accessed timestamp
          this.updateLastAccessed(keyId).catch(console.warn);

          resolve({
            privateKey,
            publicKey: new Uint8Array(encryptedKeyData.publicKey)
          });

        } catch (error) {
          reject(new StorageError(
            'Failed to decrypt key - incorrect passphrase or corrupted data',
            'DECRYPTION_FAILED'
          ));
        }
      };
      
      request.onerror = () => {
        reject(new StorageError(
          `Failed to retrieve key: ${request.error?.message}`,
          'RETRIEVAL_FAILED'
        ));
      };
    });
  }

  /**
   * Delete key pair from storage
   */
  async deleteKeyPair(keyId: string): Promise<void> {
    await this.dbReady;
    
    if (!this.db) {
      throw new StorageError('Database not initialized', 'DB_NOT_INITIALIZED');
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readwrite');
      const store = transaction.objectStore(STORE_NAME);
      
      const request = store.delete(keyId);
      
      request.onsuccess = () => resolve();
      request.onerror = () => {
        reject(new StorageError(
          `Failed to delete key: ${request.error?.message}`,
          'DELETION_FAILED'
        ));
      };
    });
  }

  /**
   * List all stored key metadata (without private keys)
   */
  async listKeys(): Promise<Array<{ id: string; metadata: StoredKeyMetadata }>> {
    await this.dbReady;
    
    if (!this.db) {
      throw new StorageError('Database not initialized', 'DB_NOT_INITIALIZED');
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readonly');
      const store = transaction.objectStore(STORE_NAME);
      
      const request = store.getAll();
      
      request.onsuccess = () => {
        const results: EncryptedKeyData[] = request.result;
        const keyList = results.map(data => ({
          id: data.id,
          metadata: data.metadata
        }));
        resolve(keyList);
      };
      
      request.onerror = () => {
        reject(new StorageError(
          `Failed to list keys: ${request.error?.message}`,
          'LIST_FAILED'
        ));
      };
    });
  }

  /**
   * Check if a key exists in storage
   */
  async keyExists(keyId: string): Promise<boolean> {
    await this.dbReady;
    
    if (!this.db) {
      throw new StorageError('Database not initialized', 'DB_NOT_INITIALIZED');
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readonly');
      const store = transaction.objectStore(STORE_NAME);
      
      const request = store.count(keyId);
      
      request.onsuccess = () => {
        resolve(request.result > 0);
      };
      
      request.onerror = () => {
        reject(new StorageError(
          `Failed to check key existence: ${request.error?.message}`,
          'EXISTENCE_CHECK_FAILED'
        ));
      };
    });
  }

  /**
   * Update last accessed timestamp for a key
   */
  private async updateLastAccessed(keyId: string): Promise<void> {
    if (!this.db) return;

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readwrite');
      const store = transaction.objectStore(STORE_NAME);
      
      const getRequest = store.get(keyId);
      
      getRequest.onsuccess = () => {
        const data: EncryptedKeyData | undefined = getRequest.result;
        if (data) {
          data.metadata.lastAccessed = new Date().toISOString();
          store.put(data);
        }
        resolve();
      };
      
      getRequest.onerror = () => resolve(); // Fail silently for timestamp updates
    });
  }

  /**
   * Get storage usage information
   */
  async getStorageInfo(): Promise<{ used: number; available: number | null }> {
    if (!navigator.storage || !navigator.storage.estimate) {
      return { used: 0, available: null };
    }

    try {
      const estimate = await navigator.storage.estimate();
      return {
        used: estimate.usage || 0,
        available: estimate.quota || null
      };
    } catch {
      return { used: 0, available: null };
    }
  }

  /**
   * Clear all stored keys (use with caution!)
   */
  async clearAllKeys(): Promise<void> {
    await this.dbReady;
    
    if (!this.db) {
      throw new StorageError('Database not initialized', 'DB_NOT_INITIALIZED');
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readwrite');
      const store = transaction.objectStore(STORE_NAME);
      
      const request = store.clear();
      
      request.onsuccess = () => resolve();
      request.onerror = () => {
        reject(new StorageError(
          `Failed to clear storage: ${request.error?.message}`,
          'CLEAR_FAILED'
        ));
      };
    });
  }

  /**
   * Close database connection
   */
  async close(): Promise<void> {
    if (this.db) {
      this.db.close();
      this.db = null;
      this.dbReady = null;
    }
  }
}