/**
 * DataFold Key Export/Import Module
 * Implements encrypted key backup and restoration with multiple formats
 */

import { 
  Ed25519KeyPair, 
  KeyStorageInterface, 
  StoredKeyMetadata, 
  KeyExportOptions, 
  KeyImportOptions, 
  KeyExportResult, 
  KeyImportResult, 
  EncryptedBackupFormat, 
  KeyBackupData, 
  ImportValidationResult,
  KeyExportError, 
  KeyImportError,
  ExportFormat 
} from '../types.js';
import { SDK_VERSION } from '../index.js';

/**
 * Constants for export/import operations
 */
const EXPORT_VERSION = 1;
const DEFAULT_KDF_ITERATIONS = 100000;
const SALT_LENGTH = 32; // 256 bits for better security
const NONCE_LENGTH = 12; // AES-GCM recommended
const KEY_LENGTH = 32; // 256-bit encryption key

/**
 * Key Export/Import Manager
 * Handles encrypted backup and restoration of Ed25519 keys
 */
export class KeyExportImportManager {
  constructor(private storage: KeyStorageInterface) {}

  /**
   * Export a key pair to encrypted backup format
   */
  async exportKey(
    keyId: string,
    passphrase: string,
    options: KeyExportOptions
  ): Promise<KeyExportResult> {
    if (!keyId?.trim()) {
      throw new KeyExportError('Key ID cannot be empty', 'INVALID_KEY_ID');
    }

    if (!passphrase || passphrase.length < 8) {
      throw new KeyExportError('Passphrase must be at least 8 characters', 'WEAK_PASSPHRASE');
    }

    try {
      // Retrieve the key pair from storage
      const keyPair = await this.storage.retrieveKeyPair(keyId, passphrase);
      
      // Get metadata if available
      let metadata: StoredKeyMetadata;
      if (options.includeMetadata !== false) {
        const keyList = await this.storage.listKeys();
        const keyInfo = keyList.find(k => k.id === keyId);
        metadata = keyInfo?.metadata || this.createDefaultMetadata(keyId);
      } else {
        metadata = this.createDefaultMetadata(keyId);
      }

      // Create backup data structure
      const backupData: KeyBackupData = {
        keyId,
        privateKey: this.uint8ArrayToBase64(keyPair.privateKey),
        publicKey: this.uint8ArrayToBase64(keyPair.publicKey),
        metadata,
        exported: new Date().toISOString(),
        sdkVersion: SDK_VERSION,
        ...options.additionalData
      };

      // Encrypt and format the backup
      const result = await this.encryptBackup(backupData, passphrase, options);
      
      return result;

    } catch (error) {
      if (error instanceof KeyExportError) {
        throw error;
      }
      throw new KeyExportError(
        `Export failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'EXPORT_FAILED'
      );
    }
  }

  /**
   * Import a key pair from encrypted backup
   */
  async importKey(
    backupData: string | Uint8Array,
    passphrase: string,
    options: KeyImportOptions = {}
  ): Promise<KeyImportResult> {
    if (!passphrase || passphrase.length < 8) {
      throw new KeyImportError('Passphrase must be at least 8 characters', 'WEAK_PASSPHRASE');
    }

    try {
      // Validate backup format
      const validation = this.validateImportData(backupData);
      if (!validation.valid) {
        throw new KeyImportError(
          `Invalid backup format: ${validation.issues.join(', ')}`,
          'INVALID_FORMAT'
        );
      }

      // Decrypt the backup
      const decryptedData = await this.decryptBackup(backupData, passphrase, validation.format!);
      
      // Validate integrity if requested
      let integrityValid = true;
      if (options.validateIntegrity !== false) {
        integrityValid = await this.validateKeyIntegrity(decryptedData);
        if (!integrityValid) {
          throw new KeyImportError('Key integrity validation failed', 'INTEGRITY_FAILED');
        }
      }

      // Check if key already exists
      const keyExists = await this.storage.keyExists(decryptedData.keyId);
      if (keyExists && !options.overwriteExisting) {
        throw new KeyImportError(
          `Key already exists: ${decryptedData.keyId}. Use overwriteExisting option to replace.`,
          'KEY_EXISTS'
        );
      }

      // Reconstruct key pair
      const keyPair: Ed25519KeyPair = {
        privateKey: this.base64ToUint8Array(decryptedData.privateKey),
        publicKey: this.base64ToUint8Array(decryptedData.publicKey)
      };

      // Merge metadata
      const finalMetadata: StoredKeyMetadata = {
        ...decryptedData.metadata,
        ...options.customMetadata,
        lastAccessed: new Date().toISOString()
      };

      // Store the imported key
      await this.storage.storeKeyPair(
        decryptedData.keyId,
        keyPair,
        passphrase,
        finalMetadata
      );

      return {
        keyId: decryptedData.keyId,
        overwritten: keyExists,
        metadata: finalMetadata,
        timestamp: new Date().toISOString(),
        integrityValid
      };

    } catch (error) {
      if (error instanceof KeyImportError) {
        throw error;
      }
      throw new KeyImportError(
        `Import failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'IMPORT_FAILED'
      );
    }
  }

  /**
   * Validate import data format and structure
   */
  validateImportData(data: string | Uint8Array): ImportValidationResult {
    const issues: string[] = [];
    let format: ExportFormat | undefined;
    let version: number | undefined;

    try {
      if (typeof data === 'string') {
        // Try to parse as JSON
        const parsed = JSON.parse(data);
        format = 'json';
        
        // Validate JSON backup structure
        if (!parsed.version || typeof parsed.version !== 'number') {
          issues.push('Missing or invalid version field');
        } else {
          version = parsed.version;
        }

        if (!parsed.type || parsed.type !== 'datafold-key-backup') {
          issues.push('Invalid backup type');
        }

        if (!parsed.kdf || parsed.kdf !== 'pbkdf2') {
          issues.push('Unsupported KDF algorithm');
        }

        if (!parsed.encryption || parsed.encryption !== 'aes-gcm') {
          issues.push('Unsupported encryption algorithm');
        }

        if (!parsed.kdf_params || !parsed.kdf_params.salt || !parsed.kdf_params.iterations) {
          issues.push('Missing or invalid KDF parameters');
        }

        if (!parsed.nonce || !parsed.ciphertext) {
          issues.push('Missing encryption data');
        }

      } else if (data instanceof Uint8Array) {
        format = 'binary';
        
        // Validate binary format (simplified check)
        if (data.length < 64) { // Minimum size check
          issues.push('Binary data too small to be valid backup');
        }
        
        // Check for binary format markers (implementation specific)
        const magic = data.slice(0, 4);
        const expectedMagic = new Uint8Array([0x44, 0x46, 0x4B, 0x42]); // "DFKB"
        if (!magic.every((byte, i) => byte === expectedMagic[i])) {
          issues.push('Invalid binary format signature');
        }
        
      } else {
        issues.push('Data must be string (JSON) or Uint8Array (binary)');
      }

    } catch (error) {
      issues.push(`Parse error: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }

    return {
      valid: issues.length === 0,
      issues,
      format,
      version
    };
  }

  /**
   * Encrypt backup data using specified format
   */
  private async encryptBackup(
    backupData: KeyBackupData,
    passphrase: string,
    options: KeyExportOptions
  ): Promise<KeyExportResult> {
    const plaintext = JSON.stringify(backupData);
    const plaintextBytes = new TextEncoder().encode(plaintext);
    
    // Generate random salt and nonce
    const salt = crypto.getRandomValues(new Uint8Array(SALT_LENGTH));
    const nonce = crypto.getRandomValues(new Uint8Array(NONCE_LENGTH));
    
    // Derive encryption key using PBKDF2
    const iterations = options.kdfIterations || DEFAULT_KDF_ITERATIONS;
    const encryptionKey = await this.deriveEncryptionKey(passphrase, salt, iterations);
    
    // Encrypt the data
    const ciphertext = await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv: nonce },
      encryptionKey,
      plaintextBytes
    );

    const timestamp = new Date().toISOString();
    let exportData: string | Uint8Array;
    let size: number;

    if (options.format === 'json') {
      // Create JSON backup format
      const backup: EncryptedBackupFormat = {
        version: EXPORT_VERSION,
        kdf: 'pbkdf2',
        kdf_params: {
          salt: this.uint8ArrayToBase64(salt),
          iterations,
          hash: 'SHA-256'
        },
        encryption: 'aes-gcm',
        nonce: this.uint8ArrayToBase64(nonce),
        ciphertext: this.arrayBufferToBase64(ciphertext),
        created: timestamp,
        type: 'datafold-key-backup'
      };
      
      exportData = JSON.stringify(backup, null, 2);
      size = new TextEncoder().encode(exportData).length;
      
    } else if (options.format === 'binary') {
      // Create binary backup format
      exportData = this.createBinaryBackup(salt, nonce, new Uint8Array(ciphertext), iterations, timestamp);
      size = exportData.length;
      
    } else {
      throw new KeyExportError(`Unsupported export format: ${options.format}`, 'INVALID_FORMAT');
    }

    // Calculate checksum
    const checksum = await this.calculateChecksum(typeof exportData === 'string' ? new TextEncoder().encode(exportData) : exportData);

    return {
      data: exportData,
      format: options.format,
      size,
      checksum: this.uint8ArrayToBase64(checksum),
      timestamp
    };
  }

  /**
   * Decrypt backup data
   */
  private async decryptBackup(
    backupData: string | Uint8Array,
    passphrase: string,
    format: ExportFormat
  ): Promise<KeyBackupData> {
    let salt: Uint8Array;
    let nonce: Uint8Array;
    let ciphertext: Uint8Array;
    let iterations: number;

    if (format === 'json') {
      const backup: EncryptedBackupFormat = JSON.parse(backupData as string);
      salt = this.base64ToUint8Array(backup.kdf_params.salt);
      nonce = this.base64ToUint8Array(backup.nonce);
      ciphertext = this.base64ToUint8Array(backup.ciphertext);
      iterations = backup.kdf_params.iterations;
      
    } else if (format === 'binary') {
      const parsed = this.parseBinaryBackup(backupData as Uint8Array);
      salt = parsed.salt;
      nonce = parsed.nonce;
      ciphertext = parsed.ciphertext;
      iterations = parsed.iterations;
      
    } else {
      throw new KeyImportError(`Unsupported format: ${format}`, 'INVALID_FORMAT');
    }

    // Derive decryption key
    const decryptionKey = await this.deriveEncryptionKey(passphrase, salt, iterations);
    
    // Decrypt the data
    try {
      const decryptedBuffer = await crypto.subtle.decrypt(
        { name: 'AES-GCM', iv: nonce },
        decryptionKey,
        ciphertext
      );
      
      const decryptedText = new TextDecoder().decode(decryptedBuffer);
      return JSON.parse(decryptedText) as KeyBackupData;
      
    } catch (error) {
      throw new KeyImportError(
        'Decryption failed - incorrect passphrase or corrupted data',
        'DECRYPTION_FAILED'
      );
    }
  }

  /**
   * Derive encryption key using PBKDF2
   */
  private async deriveEncryptionKey(
    passphrase: string,
    salt: Uint8Array,
    iterations: number
  ): Promise<CryptoKey> {
    const keyMaterial = await crypto.subtle.importKey(
      'raw',
      new TextEncoder().encode(passphrase),
      { name: 'PBKDF2' },
      false,
      ['deriveKey']
    );

    return await crypto.subtle.deriveKey(
      {
        name: 'PBKDF2',
        salt,
        iterations,
        hash: 'SHA-256'
      },
      keyMaterial,
      { name: 'AES-GCM', length: 256 },
      false,
      ['encrypt', 'decrypt']
    );
  }

  /**
   * Validate key integrity
   */
  private async validateKeyIntegrity(backupData: KeyBackupData): Promise<boolean> {
    try {
      // Basic validation checks
      if (!backupData.keyId || !backupData.privateKey || !backupData.publicKey) {
        return false;
      }

      // Validate key lengths
      const privateKey = this.base64ToUint8Array(backupData.privateKey);
      const publicKey = this.base64ToUint8Array(backupData.publicKey);

      if (privateKey.length !== 32 || publicKey.length !== 32) {
        return false;
      }

      // Validate that private and public keys are mathematically related
      // This is a simplified check - in a full implementation, you would
      // verify that the public key is correctly derived from the private key
      return true;

    } catch {
      return false;
    }
  }

  /**
   * Create binary backup format
   */
  private createBinaryBackup(
    salt: Uint8Array,
    nonce: Uint8Array,
    ciphertext: Uint8Array,
    iterations: number,
    timestamp: string
  ): Uint8Array {
    // Binary format: [magic:4][version:4][iterations:4][salt:32][nonce:12][timestamp_len:4][timestamp][ciphertext]
    const magic = new Uint8Array([0x44, 0x46, 0x4B, 0x42]); // "DFKB"
    const version = new Uint8Array(4);
    const iterationsBytes = new Uint8Array(4);
    const timestampBytes = new TextEncoder().encode(timestamp);
    const timestampLenBytes = new Uint8Array(4);
    
    // Convert numbers to bytes (little-endian)
    new DataView(version.buffer).setUint32(0, EXPORT_VERSION, true);
    new DataView(iterationsBytes.buffer).setUint32(0, iterations, true);
    new DataView(timestampLenBytes.buffer).setUint32(0, timestampBytes.length, true);
    
    // Combine all parts
    const totalLength = magic.length + version.length + iterationsBytes.length + 
                       salt.length + nonce.length + timestampLenBytes.length + 
                       timestampBytes.length + ciphertext.length;
    
    const result = new Uint8Array(totalLength);
    let offset = 0;
    
    result.set(magic, offset); offset += magic.length;
    result.set(version, offset); offset += version.length;
    result.set(iterationsBytes, offset); offset += iterationsBytes.length;
    result.set(salt, offset); offset += salt.length;
    result.set(nonce, offset); offset += nonce.length;
    result.set(timestampLenBytes, offset); offset += timestampLenBytes.length;
    result.set(timestampBytes, offset); offset += timestampBytes.length;
    result.set(ciphertext, offset);
    
    return result;
  }

  /**
   * Parse binary backup format
   */
  private parseBinaryBackup(data: Uint8Array): {
    salt: Uint8Array;
    nonce: Uint8Array;
    ciphertext: Uint8Array;
    iterations: number;
  } {
    const view = new DataView(data.buffer);
    let offset = 0;
    
    // Skip magic (already validated)
    offset += 4;
    
    // Read version
    const version = view.getUint32(offset, true);
    offset += 4;
    
    if (version !== EXPORT_VERSION) {
      throw new KeyImportError(`Unsupported binary format version: ${version}`, 'UNSUPPORTED_VERSION');
    }
    
    // Read iterations
    const iterations = view.getUint32(offset, true);
    offset += 4;
    
    // Read salt
    const salt = data.slice(offset, offset + SALT_LENGTH);
    offset += SALT_LENGTH;
    
    // Read nonce
    const nonce = data.slice(offset, offset + NONCE_LENGTH);
    offset += NONCE_LENGTH;
    
    // Read timestamp length and skip timestamp
    const timestampLen = view.getUint32(offset, true);
    offset += 4 + timestampLen;
    
    // Read ciphertext (rest of the data)
    const ciphertext = data.slice(offset);
    
    return { salt, nonce, ciphertext, iterations };
  }

  /**
   * Calculate SHA-256 checksum
   */
  private async calculateChecksum(data: Uint8Array): Promise<Uint8Array> {
    const hashBuffer = await crypto.subtle.digest('SHA-256', data);
    return new Uint8Array(hashBuffer);
  }

  /**
   * Create default metadata for exported keys
   */
  private createDefaultMetadata(keyId: string): StoredKeyMetadata {
    const now = new Date().toISOString();
    return {
      name: keyId,
      description: 'Exported key',
      created: now,
      lastAccessed: now,
      tags: ['exported']
    };
  }

  /**
   * Utility: Convert Uint8Array to base64
   */
  private uint8ArrayToBase64(data: Uint8Array): string {
    return btoa(String.fromCharCode(...data));
  }

  /**
   * Utility: Convert base64 to Uint8Array
   */
  private base64ToUint8Array(base64: string): Uint8Array {
    return new Uint8Array(atob(base64).split('').map(c => c.charCodeAt(0)));
  }

  /**
   * Utility: Convert ArrayBuffer to base64
   */
  private arrayBufferToBase64(buffer: ArrayBuffer): string {
    return this.uint8ArrayToBase64(new Uint8Array(buffer));
  }
}

/**
 * Convenience functions for standalone export/import operations
 */

/**
 * Export a key from storage to encrypted backup
 */
export async function exportKey(
  storage: KeyStorageInterface,
  keyId: string,
  passphrase: string,
  options: KeyExportOptions
): Promise<KeyExportResult> {
  const manager = new KeyExportImportManager(storage);
  return await manager.exportKey(keyId, passphrase, options);
}

/**
 * Import a key from encrypted backup to storage
 */
export async function importKey(
  storage: KeyStorageInterface,
  backupData: string | Uint8Array,
  passphrase: string,
  options?: KeyImportOptions
): Promise<KeyImportResult> {
  const manager = new KeyExportImportManager(storage);
  return await manager.importKey(backupData, passphrase, options);
}

/**
 * Validate backup data without importing
 */
export function validateBackupData(data: string | Uint8Array): ImportValidationResult {
  const manager = new KeyExportImportManager({} as KeyStorageInterface);
  return manager.validateImportData(data);
}

/**
 * Generate a secure backup filename
 */
export function generateBackupFilename(keyId: string, format: ExportFormat): string {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  const extension = format === 'json' ? 'json' : 'dfkb';
  return `datafold-backup-${keyId}-${timestamp}.${extension}`;
}