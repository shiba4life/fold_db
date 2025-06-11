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
  encryptedPrivateKey: string; // Base64 encoded for reliable storage
  publicKey: ArrayBuffer;
  iv: string; // Base64 encoded for reliable storage
  salt: string; // Base64 encoded for reliable storage
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
  ): Promise<{ encryptedData: ArrayBuffer; iv: Uint8Array; salt: Uint8Array }> {
    // Generate random salt and IV
    const salt = crypto.getRandomValues(new Uint8Array(SALT_LENGTH));
    const iv = crypto.getRandomValues(new Uint8Array(IV_LENGTH));

    // Derive encryption key
    const encryptionKey = await this.deriveEncryptionKey(passphrase, salt);

    // Add integrity check: prepend magic bytes to private key data
    const MAGIC_BYTES = new Uint8Array([0xD4, 0x7A, 0xF0, 0x1D]); // "DataFold" marker
    
    // Create simple passphrase checksum for integrity verification (compatible with test env)
    // Always use XOR-based checksum for consistency across test/production environments
    const passphraseBytes = new TextEncoder().encode(passphrase);
    const checksumBytes = new Uint8Array(8);
    for (let i = 0; i < passphraseBytes.length; i++) {
      checksumBytes[i % 8] ^= passphraseBytes[i];
    }
    
    const dataWithIntegrity = new Uint8Array(MAGIC_BYTES.length + checksumBytes.length + privateKey.length);
    dataWithIntegrity.set(MAGIC_BYTES, 0);
    dataWithIntegrity.set(checksumBytes, MAGIC_BYTES.length);
    dataWithIntegrity.set(privateKey, MAGIC_BYTES.length + checksumBytes.length);

    console.log('DEBUG: Data with integrity length:', dataWithIntegrity.length);
    console.log('DEBUG: Magic bytes:', Array.from(MAGIC_BYTES));
    console.log('DEBUG: Checksum bytes:', Array.from(checksumBytes));
    console.log('DEBUG: Private key length:', privateKey.length);

    // Encrypt private key with integrity check
    const encryptedData = await crypto.subtle.encrypt(
      { name: ENCRYPTION_ALGORITHM, iv: iv },
      encryptionKey,
      dataWithIntegrity
    );

    console.log('DEBUG: Encrypted data length after encryption:', encryptedData.byteLength);

    return { encryptedData, iv, salt };
  }

  /**
   * Decrypt private key data
   */
  private async decryptPrivateKey(
    encryptedData: Uint8Array,
    iv: Uint8Array,
    salt: Uint8Array,
    passphrase: string
  ): Promise<Uint8Array> {
    try {
      console.log('DEBUG: Starting decryption process');
      // Derive encryption key
      const encryptionKey = await this.deriveEncryptionKey(passphrase, salt);
      console.log('DEBUG: Encryption key derived successfully');

      // Decrypt private key
      console.log('DEBUG: About to decrypt with passphrase:', passphrase);
      console.log('DEBUG: IV length:', iv?.length, 'Salt length:', salt?.length, 'Encrypted data length:', encryptedData?.length);
      
      const decryptedData = await crypto.subtle.decrypt(
        { name: ENCRYPTION_ALGORITHM, iv: iv },
        encryptionKey,
        encryptedData
      );
      console.log('DEBUG: Data decrypted successfully, length:', decryptedData.byteLength);

      const decryptedBytes = new Uint8Array(decryptedData);
      console.log('DEBUG: Decrypted bytes length:', decryptedBytes.length);
      
      // Check for new format with magic bytes and passphrase checksum
      const MAGIC_BYTES = new Uint8Array([0xD4, 0x7A, 0xF0, 0x1D]); // "DataFold" marker
      const CHECKSUM_LENGTH = 8;
      
      console.log('DEBUG: Checking for new format. Required length:', MAGIC_BYTES.length + CHECKSUM_LENGTH + 32);
      
      if (decryptedBytes.length >= MAGIC_BYTES.length + CHECKSUM_LENGTH + 32) {
        console.log('DEBUG: Length check passed, checking magic bytes');
        // Check if this is the new format with magic bytes
        let hasValidMagic = true;
        for (let i = 0; i < MAGIC_BYTES.length; i++) {
          if (decryptedBytes[i] !== MAGIC_BYTES[i]) {
            hasValidMagic = false;
            break;
          }
        }
        
        console.log('DEBUG: Magic bytes valid:', hasValidMagic);
        if (hasValidMagic) {
          // Validate passphrase checksum
          const storedChecksum = decryptedBytes.slice(MAGIC_BYTES.length, MAGIC_BYTES.length + CHECKSUM_LENGTH);
          
          // Compute expected checksum using same method as storage
          // Always use XOR-based checksum for consistency across test/production environments
          const passphraseBytes = new TextEncoder().encode(passphrase);
          const expectedChecksum = new Uint8Array(8);
          for (let i = 0; i < passphraseBytes.length; i++) {
            expectedChecksum[i % 8] ^= passphraseBytes[i];
          }
          
          // Debug checksum comparison
          console.log('Stored checksum:', Array.from(storedChecksum));
          console.log('Expected checksum:', Array.from(expectedChecksum));
          console.log('Passphrase used:', passphrase);
          
          // Compare checksums
          let checksumMatch = true;
          for (let i = 0; i < CHECKSUM_LENGTH; i++) {
            if (storedChecksum[i] !== expectedChecksum[i]) {
              checksumMatch = false;
              break;
            }
          }
          
          if (!checksumMatch) {
            console.log('Checksum mismatch detected!');
            throw new Error('Invalid passphrase - checksum mismatch');
          }
          
          // New format - extract private key after magic bytes and checksum
          const privateKeyBytes = decryptedBytes.slice(MAGIC_BYTES.length + CHECKSUM_LENGTH);
          if (privateKeyBytes.length !== 32) {
            throw new Error('Invalid private key length in new format');
          }
          return privateKeyBytes;
        }
      }
      
      // Fall back to old format (direct 32-byte private key) with strict validation
      if (decryptedBytes.length === 32) {
        // Strict validation for old format - check for invalid patterns
        const allZeros = decryptedBytes.every(b => b === 0);
        const allOnes = decryptedBytes.every(b => b === 255);
        const mostlyRepeating = this.checkForRepeatingPatterns(decryptedBytes);
        
        if (allZeros || allOnes || mostlyRepeating) {
          throw new Error('Invalid decrypted data - likely wrong passphrase');
        }
        
        // Additional entropy check - valid private keys should have reasonable entropy
        const entropy = this.calculateEntropy(decryptedBytes);
        if (entropy < 6.0) { // Threshold for reasonable entropy
          throw new Error('Invalid decrypted data - low entropy suggests wrong passphrase');
        }
        
        return decryptedBytes;
      }
      
      // If neither format matches, it's likely wrong passphrase
      throw new Error('Invalid decrypted data format - incorrect passphrase or corrupted data');
      
    } catch (error) {
      // Decrypt operation failed or data validation failed
      if (error instanceof DOMException || (error instanceof Error && error.name === 'OperationError')) {
        throw new Error('Failed to decrypt private key - incorrect passphrase');
      }
      throw new Error('Failed to decrypt private key - incorrect passphrase or corrupted data');
    }
  }

  /**
   * Convert Uint8Array to base64 string
   */
  private uint8ArrayToBase64(bytes: Uint8Array): string {
    let binary = '';
    for (let i = 0; i < bytes.length; i++) {
      binary += String.fromCharCode(bytes[i]);
    }
    return btoa(binary);
  }

  /**
   * Convert base64 string to Uint8Array
   */
  private base64ToUint8Array(base64: string): Uint8Array {
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    return bytes;
  }

  /**
   * Check for repeating patterns that indicate invalid data
   */
  private checkForRepeatingPatterns(data: Uint8Array): boolean {
    // Check for runs of identical bytes (more than 8 in a row suggests invalid data)
    let maxRun = 1;
    let currentRun = 1;
    
    for (let i = 1; i < data.length; i++) {
      if (data[i] === data[i - 1]) {
        currentRun++;
        maxRun = Math.max(maxRun, currentRun);
      } else {
        currentRun = 1;
      }
    }
    
    return maxRun > 8; // More than 8 consecutive identical bytes is suspicious
  }

  /**
   * Calculate Shannon entropy of data
   */
  private calculateEntropy(data: Uint8Array): number {
    const frequency = new Array(256).fill(0);
    
    // Count byte frequencies
    for (const byte of data) {
      frequency[byte]++;
    }
    
    // Calculate entropy
    let entropy = 0;
    const length = data.length;
    
    for (const count of frequency) {
      if (count > 0) {
        const p = count / length;
        entropy -= p * Math.log2(p);
      }
    }
    
    return entropy;
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
      console.log('DEBUG: Storing key with passphrase:', passphrase);
      const { encryptedData, iv, salt } = await this.encryptPrivateKey(keyPair.privateKey, passphrase);
      console.log('DEBUG: Encrypted data length during storage:', encryptedData.byteLength);
      console.log('DEBUG: IV length during storage:', iv.byteLength, 'Salt length:', salt.byteLength);

      // Prepare metadata
      const fullMetadata: StoredKeyMetadata = {
        name: metadata.name || keyId,
        description: metadata.description || '',
        created: metadata.created || new Date().toISOString(),
        lastAccessed: new Date().toISOString(),
        tags: metadata.tags || []
      };

      // Prepare encrypted key data
      // Store as base64 strings for reliable IndexedDB storage
      const encryptedKeyData: EncryptedKeyData = {
        id: keyId,
        encryptedPrivateKey: this.uint8ArrayToBase64(new Uint8Array(encryptedData)),
        publicKey: keyPair.publicKey.buffer,
        iv: this.uint8ArrayToBase64(iv),
        salt: this.uint8ArrayToBase64(salt),
        metadata: fullMetadata,
        version: 1
      };
      
      console.log('DEBUG: Final encrypted data length before IndexedDB storage (base64):', encryptedKeyData.encryptedPrivateKey.length);

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
    
    console.log('DEBUG: retrieveKeyPair called with keyId:', keyId, 'passphrase:', passphrase);
    
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

        console.log('DEBUG: Retrieved data from IndexedDB (base64):', {
          hasEncryptedPrivateKey: !!encryptedKeyData.encryptedPrivateKey,
          hasIv: !!encryptedKeyData.iv,
          hasSalt: !!encryptedKeyData.salt,
          encryptedPrivateKeyType: typeof encryptedKeyData.encryptedPrivateKey,
          ivType: typeof encryptedKeyData.iv,
          saltType: typeof encryptedKeyData.salt,
          encryptedPrivateKeyLength: encryptedKeyData.encryptedPrivateKey?.length,
          ivLength: encryptedKeyData.iv?.length,
          saltLength: encryptedKeyData.salt?.length
        });

        try {
          // Ensure we have proper string values (IndexedDB may serialize differently)
          const encryptedPrivateKeyStr = typeof encryptedKeyData.encryptedPrivateKey === 'string'
            ? encryptedKeyData.encryptedPrivateKey
            : String(encryptedKeyData.encryptedPrivateKey);
          const ivStr = typeof encryptedKeyData.iv === 'string'
            ? encryptedKeyData.iv
            : String(encryptedKeyData.iv);
          const saltStr = typeof encryptedKeyData.salt === 'string'
            ? encryptedKeyData.salt
            : String(encryptedKeyData.salt);

          // Convert base64 strings back to Uint8Arrays with error handling
          let encryptedPrivateKey: Uint8Array;
          let iv: Uint8Array;
          let salt: Uint8Array;

          try {
            encryptedPrivateKey = this.base64ToUint8Array(encryptedPrivateKeyStr);
            iv = this.base64ToUint8Array(ivStr);
            salt = this.base64ToUint8Array(saltStr);
          } catch (error) {
            // If base64 decoding fails, the data might be in object format
            console.warn('Base64 decoding failed, attempting object conversion');
            throw new Error('Failed to decode stored data - incompatible format');
          }

          console.log('DEBUG: After base64 conversion - lengths:', {
            encryptedPrivateKeyLength: encryptedPrivateKey.length,
            ivLength: iv.length,
            saltLength: salt.length
          });

          // Decrypt private key
          const privateKey = await this.decryptPrivateKey(
            encryptedPrivateKey,
            iv,
            salt,
            passphrase
          );

          // Update last accessed timestamp
          this.updateLastAccessed(keyId).catch(console.warn);

          resolve({
            privateKey,
            publicKey: new Uint8Array(encryptedKeyData.publicKey)
          });

        } catch (error) {
          console.error('Decryption error details:', error);
          reject(new StorageError(
            `Failed to decrypt key: ${error instanceof Error ? error.message : 'Unknown decryption error'}`,
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
   * Delete key by ID (alias for deleteKeyPair)
   */
  async deleteKey(keyId: string): Promise<void> {
    return this.deleteKeyPair(keyId);
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
      if (!this.db) {
        resolve(); // Fail silently if no database connection
        return;
      }
      
      const transaction = this.db.transaction([STORE_NAME], 'readwrite');
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
      // Check if close method exists (may not exist in mocked environments)
      if (typeof this.db.close === 'function') {
        this.db.close();
      }
      this.db = null;
      this.dbReady = null;
    }
  }
}