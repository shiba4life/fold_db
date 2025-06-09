/**
 * DataFold Unified Backup Format Implementation
 * 
 * This module implements the standardized encrypted backup format for cross-platform
 * compatibility following the specification from docs/delivery/10/backup/encrypted_backup_format.md
 */

import { Ed25519KeyPair } from '../types.js';

/**
 * Unified backup format structure as defined in the specification
 */
export interface UnifiedBackupFormat {
  version: number;
  kdf: 'argon2id' | 'pbkdf2';
  kdf_params: {
    salt: string;
    iterations: number;
    memory?: number;      // Required for argon2id
    parallelism?: number; // Required for argon2id
  };
  encryption: 'xchacha20-poly1305' | 'aes-gcm';
  nonce: string;
  ciphertext: string;
  created: string;
  metadata?: {
    key_type: string;
    label?: string;
  };
}

/**
 * Migration result for converting legacy backups
 */
export interface MigrationResult {
  success: boolean;
  originalFormat: string;
  newFormat: UnifiedBackupFormat;
  warnings: string[];
}

/**
 * Test vector for cross-platform validation
 */
export interface TestVector {
  passphrase: string;
  salt: string;
  nonce: string;
  kdf: string;
  kdf_params: Record<string, any>;
  encryption: string;
  plaintext_key: string;
  ciphertext: string;
  created: string;
}

/**
 * Constants for the unified format
 */
export const UNIFIED_BACKUP_VERSION = 1;
export const MIN_SALT_LENGTH = 16;
export const PREFERRED_SALT_LENGTH = 32;
export const XCHACHA20_NONCE_LENGTH = 24;
export const AES_GCM_NONCE_LENGTH = 12;

// Argon2id parameters (preferred)
export const ARGON2_MIN_MEMORY = 65536; // 64 MiB
export const ARGON2_MIN_ITERATIONS = 3;
export const ARGON2_MIN_PARALLELISM = 2;

// PBKDF2 parameters (legacy compatibility)
export const PBKDF2_MIN_ITERATIONS = 100000;

/**
 * Unified Backup Manager for cross-platform compatibility
 */
export class UnifiedBackupManager {
  /**
   * Export key using unified backup format
   */
  async exportKey(
    keyPair: Ed25519KeyPair,
    passphrase: string,
    options: {
      keyId?: string;
      label?: string;
      kdf?: 'argon2id' | 'pbkdf2';
      encryption?: 'xchacha20-poly1305' | 'aes-gcm';
      kdfParams?: {
        memory?: number;
        iterations?: number;
        parallelism?: number;
      };
    } = {}
  ): Promise<string> {
    this.validatePassphrase(passphrase);
    this.validateKeyPair(keyPair);

    const kdf = options.kdf || 'argon2id';
    const encryption = options.encryption || 'xchacha20-poly1305';
    
    // Check algorithm support
    this.validateAlgorithmSupport(kdf, encryption);

    // Generate salt and nonce
    const salt = crypto.getRandomValues(new Uint8Array(PREFERRED_SALT_LENGTH));
    const nonceLength = encryption === 'xchacha20-poly1305' ? XCHACHA20_NONCE_LENGTH : AES_GCM_NONCE_LENGTH;
    const nonce = crypto.getRandomValues(new Uint8Array(nonceLength));

    // Prepare KDF parameters
    const kdfParams = this.prepareKdfParams(kdf, options.kdfParams);

    // Derive encryption key
    const encryptionKey = await this.deriveKey(passphrase, salt, kdf, kdfParams);

    // Prepare plaintext (Ed25519 PKCS#8 DER format)
    const plaintext = this.prepareKeyPlaintext(keyPair);

    // Encrypt the key data
    const ciphertext = await this.encryptData(plaintext, encryptionKey, nonce, encryption);

    // Create unified backup format
    const backup: UnifiedBackupFormat = {
      version: UNIFIED_BACKUP_VERSION,
      kdf,
      kdf_params: {
        salt: this.uint8ArrayToBase64(salt),
        iterations: kdfParams.iterations,
        ...(kdf === 'argon2id' && {
          memory: kdfParams.memory,
          parallelism: kdfParams.parallelism
        })
      },
      encryption,
      nonce: this.uint8ArrayToBase64(nonce),
      ciphertext: this.uint8ArrayToBase64(ciphertext),
      created: new Date().toISOString(),
      metadata: {
        key_type: 'ed25519',
        ...(options.label && { label: options.label })
      }
    };

    return JSON.stringify(backup, null, 2);
  }

  /**
   * Import key from unified backup format
   */
  async importKey(backupData: string, passphrase: string): Promise<{
    keyPair: Ed25519KeyPair;
    metadata: UnifiedBackupFormat['metadata'];
  }> {
    this.validatePassphrase(passphrase);

    let backup: UnifiedBackupFormat;
    try {
      backup = JSON.parse(backupData);
    } catch (error) {
      throw new Error(`Invalid JSON backup data: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }

    // Validate backup format
    this.validateBackupFormat(backup);

    // Extract parameters
    const salt = this.base64ToUint8Array(backup.kdf_params.salt);
    const nonce = this.base64ToUint8Array(backup.nonce);
    const ciphertext = this.base64ToUint8Array(backup.ciphertext);

    // Prepare KDF parameters
    const kdfParams = {
      iterations: backup.kdf_params.iterations,
      ...(backup.kdf === 'argon2id' && {
        memory: backup.kdf_params.memory!,
        parallelism: backup.kdf_params.parallelism!
      })
    };

    // Derive decryption key
    const decryptionKey = await this.deriveKey(passphrase, salt, backup.kdf, kdfParams);

    // Decrypt the key data
    const plaintext = await this.decryptData(ciphertext, decryptionKey, nonce, backup.encryption);

    // Extract key pair from plaintext
    const keyPair = this.extractKeyPair(plaintext);

    return {
      keyPair,
      metadata: backup.metadata
    };
  }

  /**
   * Migrate legacy backup to unified format
   */
  async migrateLegacyBackup(legacyData: string, passphrase: string): Promise<MigrationResult> {
    const warnings: string[] = [];
    
    try {
      // Try to detect and parse legacy format
      const legacyBackup = JSON.parse(legacyData);
      
      // Detect format type
      let originalFormat = 'unknown';
      if (legacyBackup.type === 'datafold-key-backup') {
        originalFormat = 'js-sdk-legacy';
      } else if (legacyBackup.algorithm === 'Ed25519' && !legacyBackup.type) {
        originalFormat = 'python-sdk-legacy';
      }

      if (originalFormat === 'unknown') {
        throw new Error('Unable to detect legacy backup format');
      }

      // Import using legacy method (simplified - you'd implement legacy parsers)
      const { keyPair, metadata } = await this.importLegacyFormat(legacyBackup, passphrase, originalFormat);

      // Export using unified format with warnings about parameter changes
      if (originalFormat === 'js-sdk-legacy' && legacyBackup.kdf === 'pbkdf2') {
        warnings.push('Migrated from PBKDF2 to Argon2id for improved security');
      }
      if (originalFormat === 'js-sdk-legacy' && legacyBackup.encryption === 'aes-gcm') {
        warnings.push('Migrated from AES-GCM to XChaCha20-Poly1305 for improved security');
      }

      const newBackupData = await this.exportKey(keyPair, passphrase, {
        label: metadata?.label,
        kdf: 'argon2id',
        encryption: 'xchacha20-poly1305'
      });

      const newFormat = JSON.parse(newBackupData);

      return {
        success: true,
        originalFormat,
        newFormat,
        warnings
      };

    } catch (error) {
      return {
        success: false,
        originalFormat: 'unknown',
        newFormat: {} as UnifiedBackupFormat,
        warnings: [`Migration failed: ${error instanceof Error ? error.message : 'Unknown error'}`]
      };
    }
  }

  /**
   * Generate test vector for cross-platform validation
   */
  async generateTestVector(): Promise<TestVector> {
    // Use fixed test data for reproducible test vectors
    const passphrase = 'correct horse battery staple';
    const salt = this.base64ToUint8Array('w7Z3pQ2v5Q8v1Q2v5Q8v1Q==');
    const nonce = this.base64ToUint8Array('AAAAAAAAAAAAAAAAAAAAAAAAAAA=');
    
    // Create test key pair
    const testPrivateKey = new Uint8Array(32);
    testPrivateKey.fill(0x42); // Deterministic test key
    const testKeyPair: Ed25519KeyPair = {
      privateKey: testPrivateKey,
      publicKey: new Uint8Array(32) // Would be derived from private key in real implementation
    };

    const kdf = 'argon2id';
    const kdfParams = {
      iterations: ARGON2_MIN_ITERATIONS,
      memory: ARGON2_MIN_MEMORY,
      parallelism: ARGON2_MIN_PARALLELISM
    };
    const encryption = 'xchacha20-poly1305';

    // Derive key and encrypt
    const derivedKey = await this.deriveKey(passphrase, salt, kdf, kdfParams);
    const plaintext = this.prepareKeyPlaintext(testKeyPair);
    const ciphertext = await this.encryptData(plaintext, derivedKey, nonce, encryption);

    return {
      passphrase,
      salt: this.uint8ArrayToBase64(salt),
      nonce: this.uint8ArrayToBase64(nonce),
      kdf,
      kdf_params: kdfParams,
      encryption,
      plaintext_key: this.uint8ArrayToBase64(plaintext),
      ciphertext: this.uint8ArrayToBase64(ciphertext),
      created: '2025-06-08T17:00:00Z'
    };
  }

  /**
   * Validate cross-platform compatibility with test vector
   */
  async validateTestVector(testVector: TestVector): Promise<boolean> {
    try {
      const salt = this.base64ToUint8Array(testVector.salt);
      const nonce = this.base64ToUint8Array(testVector.nonce);
      const expectedCiphertext = this.base64ToUint8Array(testVector.ciphertext);
      const expectedPlaintext = this.base64ToUint8Array(testVector.plaintext_key);

      // Derive key using test vector parameters
      const derivedKey = await this.deriveKey(
        testVector.passphrase,
        salt,
        testVector.kdf as 'argon2id' | 'pbkdf2',
        {
          iterations: testVector.kdf_params.iterations,
          memory: testVector.kdf_params.memory,
          parallelism: testVector.kdf_params.parallelism
        }
      );

      // Decrypt test vector ciphertext
      const decryptedPlaintext = await this.decryptData(
        expectedCiphertext,
        derivedKey,
        nonce,
        testVector.encryption as 'xchacha20-poly1305' | 'aes-gcm'
      );

      // Compare with expected plaintext
      return this.uint8ArraysEqual(decryptedPlaintext, expectedPlaintext);

    } catch (error) {
      console.error('Test vector validation failed:', error);
      return false;
    }
  }

  // Private helper methods

  private validatePassphrase(passphrase: string): void {
    if (!passphrase || typeof passphrase !== 'string') {
      throw new Error('Passphrase must be a non-empty string');
    }
    if (passphrase.length < 8) {
      throw new Error('Passphrase must be at least 8 characters long');
    }
  }

  private validateKeyPair(keyPair: Ed25519KeyPair): void {
    if (!keyPair.privateKey || !keyPair.publicKey) {
      throw new Error('Invalid key pair: missing private or public key');
    }
    if (keyPair.privateKey.length !== 32 || keyPair.publicKey.length !== 32) {
      throw new Error('Invalid key pair: incorrect key lengths');
    }
  }

  private validateAlgorithmSupport(kdf: string, encryption: string): void {
    // Note: This is a simplified check. In a full implementation, you would
    // check for actual crypto API support
    if (kdf === 'argon2id') {
      // WebCrypto doesn't natively support Argon2id, would need a polyfill
      console.warn('Argon2id requires polyfill in browser environment');
    }
    if (encryption === 'xchacha20-poly1305') {
      // WebCrypto doesn't natively support XChaCha20-Poly1305, would need a polyfill
      console.warn('XChaCha20-Poly1305 requires polyfill in browser environment');
    }
  }

  private prepareKdfParams(
    kdf: 'argon2id' | 'pbkdf2',
    customParams?: { memory?: number; iterations?: number; parallelism?: number }
  ): { iterations: number; memory?: number; parallelism?: number } {
    if (kdf === 'argon2id') {
      return {
        iterations: customParams?.iterations || ARGON2_MIN_ITERATIONS,
        memory: customParams?.memory || ARGON2_MIN_MEMORY,
        parallelism: customParams?.parallelism || ARGON2_MIN_PARALLELISM
      };
    } else {
      return {
        iterations: customParams?.iterations || PBKDF2_MIN_ITERATIONS
      };
    }
  }

  private async deriveKey(
    passphrase: string,
    salt: Uint8Array,
    kdf: 'argon2id' | 'pbkdf2',
    params: { iterations: number; memory?: number; parallelism?: number }
  ): Promise<Uint8Array> {
    if (kdf === 'argon2id') {
      // This would require an Argon2 polyfill in browser environments
      throw new Error('Argon2id not yet implemented - requires polyfill');
    } else {
      // Use PBKDF2 with WebCrypto
      const keyMaterial = await crypto.subtle.importKey(
        'raw',
        new TextEncoder().encode(passphrase),
        { name: 'PBKDF2' },
        false,
        ['deriveKey']
      );

      const derivedKey = await crypto.subtle.deriveKey(
        {
          name: 'PBKDF2',
          salt,
          iterations: params.iterations,
          hash: 'SHA-256'
        },
        keyMaterial,
        { name: 'AES-GCM', length: 256 },
        true,
        ['encrypt', 'decrypt']
      );

      const keyBytes = await crypto.subtle.exportKey('raw', derivedKey);
      return new Uint8Array(keyBytes);
    }
  }

  private prepareKeyPlaintext(keyPair: Ed25519KeyPair): Uint8Array {
    // For now, concatenate private and public key
    // In a full implementation, this would create proper PKCS#8 DER format
    const plaintext = new Uint8Array(64);
    plaintext.set(keyPair.privateKey, 0);
    plaintext.set(keyPair.publicKey, 32);
    return plaintext;
  }

  private async encryptData(
    plaintext: Uint8Array,
    key: Uint8Array,
    nonce: Uint8Array,
    algorithm: 'xchacha20-poly1305' | 'aes-gcm'
  ): Promise<Uint8Array> {
    if (algorithm === 'xchacha20-poly1305') {
      // This would require a XChaCha20-Poly1305 polyfill
      throw new Error('XChaCha20-Poly1305 not yet implemented - requires polyfill');
    } else {
      // Use AES-GCM with WebCrypto
      const cryptoKey = await crypto.subtle.importKey(
        'raw',
        key,
        { name: 'AES-GCM' },
        false,
        ['encrypt']
      );

      const encrypted = await crypto.subtle.encrypt(
        { name: 'AES-GCM', iv: nonce },
        cryptoKey,
        plaintext
      );

      return new Uint8Array(encrypted);
    }
  }

  private async decryptData(
    ciphertext: Uint8Array,
    key: Uint8Array,
    nonce: Uint8Array,
    algorithm: 'xchacha20-poly1305' | 'aes-gcm'
  ): Promise<Uint8Array> {
    if (algorithm === 'xchacha20-poly1305') {
      // This would require a XChaCha20-Poly1305 polyfill
      throw new Error('XChaCha20-Poly1305 not yet implemented - requires polyfill');
    } else {
      // Use AES-GCM with WebCrypto
      const cryptoKey = await crypto.subtle.importKey(
        'raw',
        key,
        { name: 'AES-GCM' },
        false,
        ['decrypt']
      );

      const decrypted = await crypto.subtle.decrypt(
        { name: 'AES-GCM', iv: nonce },
        cryptoKey,
        ciphertext
      );

      return new Uint8Array(decrypted);
    }
  }

  private extractKeyPair(plaintext: Uint8Array): Ed25519KeyPair {
    if (plaintext.length !== 64) {
      throw new Error('Invalid plaintext length for Ed25519 key pair');
    }
    
    return {
      privateKey: plaintext.slice(0, 32),
      publicKey: plaintext.slice(32, 64)
    };
  }

  private validateBackupFormat(backup: any): asserts backup is UnifiedBackupFormat {
    const required = ['version', 'kdf', 'kdf_params', 'encryption', 'nonce', 'ciphertext', 'created'];
    for (const field of required) {
      if (!(field in backup)) {
        throw new Error(`Missing required field: ${field}`);
      }
    }

    if (backup.version !== UNIFIED_BACKUP_VERSION) {
      throw new Error(`Unsupported backup version: ${backup.version}`);
    }

    if (!['argon2id', 'pbkdf2'].includes(backup.kdf)) {
      throw new Error(`Unsupported KDF: ${backup.kdf}`);
    }

    if (!['xchacha20-poly1305', 'aes-gcm'].includes(backup.encryption)) {
      throw new Error(`Unsupported encryption: ${backup.encryption}`);
    }

    // Validate KDF parameters
    if (!backup.kdf_params.salt || !backup.kdf_params.iterations) {
      throw new Error('Missing required KDF parameters');
    }

    if (backup.kdf === 'argon2id') {
      if (!backup.kdf_params.memory || !backup.kdf_params.parallelism) {
        throw new Error('Missing Argon2id parameters (memory, parallelism)');
      }
    }
  }

  private async importLegacyFormat(
    legacyBackup: any, 
    passphrase: string, 
    format: string
  ): Promise<{ keyPair: Ed25519KeyPair; metadata?: any }> {
    // Simplified legacy import - in a full implementation, you would
    // have specific parsers for each legacy format
    throw new Error(`Legacy format import not yet implemented for: ${format}`);
  }

  private uint8ArrayToBase64(array: Uint8Array): string {
    return btoa(String.fromCharCode(...array));
  }

  private base64ToUint8Array(base64: string): Uint8Array {
    return new Uint8Array(atob(base64).split('').map(char => char.charCodeAt(0)));
  }

  private uint8ArraysEqual(a: Uint8Array, b: Uint8Array): boolean {
    if (a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) {
      if (a[i] !== b[i]) return false;
    }
    return true;
  }
}

// Export convenience functions
export async function exportKeyUnified(
  keyPair: Ed25519KeyPair,
  passphrase: string,
  options?: Parameters<UnifiedBackupManager['exportKey']>[2]
): Promise<string> {
  const manager = new UnifiedBackupManager();
  return manager.exportKey(keyPair, passphrase, options);
}

export async function importKeyUnified(
  backupData: string,
  passphrase: string
): Promise<{ keyPair: Ed25519KeyPair; metadata: UnifiedBackupFormat['metadata'] }> {
  const manager = new UnifiedBackupManager();
  return manager.importKey(backupData, passphrase);
}

export async function migrateBackupUnified(
  legacyData: string,
  passphrase: string
): Promise<MigrationResult> {
  const manager = new UnifiedBackupManager();
  return manager.migrateLegacyBackup(legacyData, passphrase);
}