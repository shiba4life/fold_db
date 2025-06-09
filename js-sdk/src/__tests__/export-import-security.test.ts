/**
 * Security and corruption tests for key export/import functionality
 * Tests for corruption detection, tampering scenarios, and error handling
 */

import { 
  KeyExportImportManager, 
  exportKey, 
  importKey 
} from '../crypto/key-export-import.js';
import { generateKeyPair } from '../crypto/ed25519.js';
import { createStorage } from '../storage/index.js';
import { 
  Ed25519KeyPair, 
  KeyStorageInterface, 
  KeyExportOptions,
  KeyImportError,
  EncryptedBackupFormat 
} from '../types.js';

// Test with actual IndexedDB storage if available, otherwise skip
const itSkipIfNoIndexedDB = typeof window !== 'undefined' && window.indexedDB ? it : it.skip;

describe('Export/Import Security and Corruption Tests', () => {
  let storage: KeyStorageInterface;
  let manager: KeyExportImportManager;
  let testKeyPair: Ed25519KeyPair;
  let testKeyId: string;
  let testPassphrase: string;

  beforeEach(async () => {
    // Only run these tests if IndexedDB is available
    if (typeof window === 'undefined' || !window.indexedDB) {
      return;
    }

    storage = await createStorage({ dbName: `test-security-${Date.now()}` });
    manager = new KeyExportImportManager(storage);
    testKeyPair = await generateKeyPair();
    testKeyId = `security-test-${Date.now()}`;
    testPassphrase = 'secure-test-passphrase-123';

    await storage.storeKeyPair(testKeyId, testKeyPair, testPassphrase, {
      name: 'Security Test Key',
      description: 'Key for security testing'
    });
  });

  afterEach(async () => {
    if (storage) {
      await storage.clearAllKeys();
      await storage.close();
    }
  });

  describe('Corruption Detection', () => {
    itSkipIfNoIndexedDB('should detect corrupted ciphertext', async () => {
      const options: KeyExportOptions = { format: 'json' };
      const exportResult = await manager.exportKey(testKeyId, testPassphrase, options);
      
      // Parse and corrupt the ciphertext
      const backup: EncryptedBackupFormat = JSON.parse(exportResult.data as string);
      const originalCiphertext = backup.ciphertext;
      
      // Corrupt by flipping some bits in the middle
      const corruptedCiphertext = originalCiphertext.slice(0, -20) + 'CORRUPTED_DATA_HERE';
      backup.ciphertext = corruptedCiphertext;
      
      const corruptedBackup = JSON.stringify(backup);
      
      await expect(manager.importKey(corruptedBackup, testPassphrase))
        .rejects.toThrow(KeyImportError);
      
      await expect(manager.importKey(corruptedBackup, testPassphrase))
        .rejects.toThrow(/decryption failed/i);
    });

    itSkipIfNoIndexedDB('should detect corrupted salt', async () => {
      const options: KeyExportOptions = { format: 'json' };
      const exportResult = await manager.exportKey(testKeyId, testPassphrase, options);
      
      const backup: EncryptedBackupFormat = JSON.parse(exportResult.data as string);
      
      // Corrupt the salt - this should cause key derivation to fail
      backup.kdf_params.salt = 'Q09SUlVQVEVE'; // "CORRUPTED" in base64
      
      const corruptedBackup = JSON.stringify(backup);
      
      await expect(manager.importKey(corruptedBackup, testPassphrase))
        .rejects.toThrow(KeyImportError);
    });

    itSkipIfNoIndexedDB('should detect corrupted nonce/IV', async () => {
      const options: KeyExportOptions = { format: 'json' };
      const exportResult = await manager.exportKey(testKeyId, testPassphrase, options);
      
      const backup: EncryptedBackupFormat = JSON.parse(exportResult.data as string);
      
      // Corrupt the nonce
      backup.nonce = 'Q09SUlVQVEVE'; // Invalid nonce
      
      const corruptedBackup = JSON.stringify(backup);
      
      await expect(manager.importKey(corruptedBackup, testPassphrase))
        .rejects.toThrow(KeyImportError);
    });

    itSkipIfNoIndexedDB('should detect truncated ciphertext', async () => {
      const options: KeyExportOptions = { format: 'json' };
      const exportResult = await manager.exportKey(testKeyId, testPassphrase, options);
      
      const backup: EncryptedBackupFormat = JSON.parse(exportResult.data as string);
      
      // Truncate the ciphertext
      backup.ciphertext = backup.ciphertext.slice(0, backup.ciphertext.length / 2);
      
      const truncatedBackup = JSON.stringify(backup);
      
      await expect(manager.importKey(truncatedBackup, testPassphrase))
        .rejects.toThrow(KeyImportError);
    });

    itSkipIfNoIndexedDB('should detect invalid base64 encoding', async () => {
      const options: KeyExportOptions = { format: 'json' };
      const exportResult = await manager.exportKey(testKeyId, testPassphrase, options);
      
      const backup: EncryptedBackupFormat = JSON.parse(exportResult.data as string);
      
      // Introduce invalid base64 characters
      backup.ciphertext = backup.ciphertext + '!@#$%';
      
      const invalidBackup = JSON.stringify(backup);
      
      await expect(manager.importKey(invalidBackup, testPassphrase))
        .rejects.toThrow(KeyImportError);
    });
  });

  describe('Tampering Detection', () => {
    itSkipIfNoIndexedDB('should detect tampered KDF iterations', async () => {
      const options: KeyExportOptions = { format: 'json', kdfIterations: 100000 };
      const exportResult = await manager.exportKey(testKeyId, testPassphrase, options);
      
      const backup: EncryptedBackupFormat = JSON.parse(exportResult.data as string);
      
      // Reduce iterations to weaken security
      backup.kdf_params.iterations = 1000;
      
      const tamperedBackup = JSON.stringify(backup);
      
      await expect(manager.importKey(tamperedBackup, testPassphrase))
        .rejects.toThrow(KeyImportError);
    });

    itSkipIfNoIndexedDB('should detect algorithm substitution', async () => {
      const options: KeyExportOptions = { format: 'json' };
      const exportResult = await manager.exportKey(testKeyId, testPassphrase, options);
      
      const backup: EncryptedBackupFormat = JSON.parse(exportResult.data as string);
      
      // Change encryption algorithm
      backup.encryption = 'aes-cbc' as any;
      
      const tamperedBackup = JSON.stringify(backup);
      
      const validation = manager.validateImportData(tamperedBackup);
      expect(validation.valid).toBe(false);
      expect(validation.issues).toContain('Unsupported encryption algorithm');
    });

    itSkipIfNoIndexedDB('should detect KDF algorithm substitution', async () => {
      const options: KeyExportOptions = { format: 'json' };
      const exportResult = await manager.exportKey(testKeyId, testPassphrase, options);
      
      const backup: EncryptedBackupFormat = JSON.parse(exportResult.data as string);
      
      // Change KDF algorithm
      backup.kdf = 'scrypt' as any;
      
      const tamperedBackup = JSON.stringify(backup);
      
      const validation = manager.validateImportData(tamperedBackup);
      expect(validation.valid).toBe(false);
      expect(validation.issues).toContain('Unsupported KDF algorithm');
    });

    itSkipIfNoIndexedDB('should detect version rollback attacks', async () => {
      const options: KeyExportOptions = { format: 'json' };
      const exportResult = await manager.exportKey(testKeyId, testPassphrase, options);
      
      const backup: EncryptedBackupFormat = JSON.parse(exportResult.data as string);
      
      // Rollback version to a potentially vulnerable version
      backup.version = 0;
      
      const rolledBackBackup = JSON.stringify(backup);
      
      const validation = manager.validateImportData(rolledBackBackup);
      expect(validation.valid).toBe(false);
      expect(validation.issues).toContain('Missing or invalid version field');
    });

    itSkipIfNoIndexedDB('should detect type field tampering', async () => {
      const options: KeyExportOptions = { format: 'json' };
      const exportResult = await manager.exportKey(testKeyId, testPassphrase, options);
      
      const backup: EncryptedBackupFormat = JSON.parse(exportResult.data as string);
      
      // Change backup type
      backup.type = 'malicious-backup' as any;
      
      const tamperedBackup = JSON.stringify(backup);
      
      const validation = manager.validateImportData(tamperedBackup);
      expect(validation.valid).toBe(false);
      expect(validation.issues).toContain('Invalid backup type');
    });
  });

  describe('Binary Format Security', () => {
    itSkipIfNoIndexedDB('should detect corrupted binary magic signature', async () => {
      const options: KeyExportOptions = { format: 'binary' };
      const exportResult = await manager.exportKey(testKeyId, testPassphrase, options);
      
      const binaryData = exportResult.data as Uint8Array;
      const corruptedData = new Uint8Array(binaryData);
      
      // Corrupt the magic signature
      corruptedData[0] = 0xFF;
      corruptedData[1] = 0xFF;
      
      const validation = manager.validateImportData(corruptedData);
      expect(validation.valid).toBe(false);
      expect(validation.issues).toContain('Invalid binary format signature');
    });

    itSkipIfNoIndexedDB('should detect truncated binary data', async () => {
      const options: KeyExportOptions = { format: 'binary' };
      const exportResult = await manager.exportKey(testKeyId, testPassphrase, options);
      
      const binaryData = exportResult.data as Uint8Array;
      const truncatedData = binaryData.slice(0, 32); // Too small
      
      const validation = manager.validateImportData(truncatedData);
      expect(validation.valid).toBe(false);
      expect(validation.issues).toContain('Binary data too small to be valid backup');
    });

    itSkipIfNoIndexedDB('should handle corrupted binary version field', async () => {
      const options: KeyExportOptions = { format: 'binary' };
      const exportResult = await manager.exportKey(testKeyId, testPassphrase, options);
      
      const binaryData = exportResult.data as Uint8Array;
      const corruptedData = new Uint8Array(binaryData);
      
      // Corrupt version field (bytes 4-7)
      corruptedData[4] = 0xFF;
      corruptedData[5] = 0xFF;
      corruptedData[6] = 0xFF;
      corruptedData[7] = 0xFF;
      
      await expect(manager.importKey(corruptedData, testPassphrase))
        .rejects.toThrow(KeyImportError);
    });
  });

  describe('Passphrase Security', () => {
    itSkipIfNoIndexedDB('should resist brute force with wrong passphrase', async () => {
      const options: KeyExportOptions = { format: 'json', kdfIterations: 100000 };
      const exportResult = await manager.exportKey(testKeyId, testPassphrase, options);
      
      const wrongPassphrases = [
        'wrong-passphrase',
        'password123',
        'test',
        testPassphrase.slice(0, -1), // Almost correct
        testPassphrase.toUpperCase(),  // Case variation
        testPassphrase + '1'           // Slight modification
      ];
      
      for (const wrongPassphrase of wrongPassphrases) {
        await expect(manager.importKey(exportResult.data, wrongPassphrase))
          .rejects.toThrow(KeyImportError);
      }
    });

    itSkipIfNoIndexedDB('should enforce minimum passphrase strength', async () => {
      const weakPassphrases = [
        'short',
        '1234567',
        '',
        '       '  // Only whitespace
      ];
      
      for (const weakPassphrase of weakPassphrases) {
        const options: KeyExportOptions = { format: 'json' };
        
        await expect(manager.exportKey(testKeyId, weakPassphrase, options))
          .rejects.toThrow(KeyImportError);
      }
    });
  });

  describe('Error Message Security', () => {
    itSkipIfNoIndexedDB('should not leak sensitive information in error messages', async () => {
      const options: KeyExportOptions = { format: 'json' };
      const exportResult = await manager.exportKey(testKeyId, testPassphrase, options);
      
      try {
        await manager.importKey(exportResult.data, 'wrong-passphrase');
        fail('Expected import to fail');
      } catch (error) {
        const errorMessage = error.message.toLowerCase();
        
        // Should not contain sensitive data
        expect(errorMessage).not.toContain(testPassphrase);
        expect(errorMessage).not.toContain(testKeyId);
        expect(errorMessage).not.toContain('private');
        expect(errorMessage).not.toContain('secret');
        
        // Should contain generic error information
        expect(errorMessage).toMatch(/decryption|passphrase|incorrect/);
      }
    });

    itSkipIfNoIndexedDB('should not expose internal cryptographic details', async () => {
      const options: KeyExportOptions = { format: 'json' };
      const exportResult = await manager.exportKey(testKeyId, testPassphrase, options);
      
      // Corrupt the data
      const backup = JSON.parse(exportResult.data as string);
      backup.ciphertext = 'invalid';
      
      try {
        await manager.importKey(JSON.stringify(backup), testPassphrase);
        fail('Expected import to fail');
      } catch (error) {
        const errorMessage = error.message.toLowerCase();
        
        // Should not expose cryptographic implementation details
        expect(errorMessage).not.toContain('aes');
        expect(errorMessage).not.toContain('gcm');
        expect(errorMessage).not.toContain('pbkdf2');
        expect(errorMessage).not.toContain('sha-256');
        expect(errorMessage).not.toContain('salt');
        expect(errorMessage).not.toContain('iv');
        expect(errorMessage).not.toContain('nonce');
      }
    });
  });

  describe('Time-based Security', () => {
    itSkipIfNoIndexedDB('should maintain consistent timing for invalid passphrases', async () => {
      const options: KeyExportOptions = { format: 'json', kdfIterations: 10000 }; // Lower for faster testing
      const exportResult = await manager.exportKey(testKeyId, testPassphrase, options);
      
      const wrongPassphrases = ['wrong1', 'wrong2', 'wrong3'];
      const timings: number[] = [];
      
      for (const wrongPassphrase of wrongPassphrases) {
        const start = performance.now();
        
        try {
          await manager.importKey(exportResult.data, wrongPassphrase);
        } catch {
          // Expected to fail
        }
        
        const end = performance.now();
        timings.push(end - start);
      }
      
      // Check that timings are reasonably consistent (within 50% variance)
      const avgTiming = timings.reduce((a, b) => a + b) / timings.length;
      const maxVariance = avgTiming * 0.5;
      
      for (const timing of timings) {
        expect(Math.abs(timing - avgTiming)).toBeLessThan(maxVariance);
      }
    });
  });

  describe('Memory Security', () => {
    itSkipIfNoIndexedDB('should handle large corrupted files without memory exhaustion', async () => {
      // Create a large corrupted backup that could cause memory issues
      const largeCorruptedData = 'x'.repeat(10 * 1024 * 1024); // 10MB of invalid data
      
      await expect(manager.importKey(largeCorruptedData, testPassphrase))
        .rejects.toThrow(KeyImportError);
    });

    itSkipIfNoIndexedDB('should handle deeply nested JSON without stack overflow', async () => {
      let deeplyNested = '{"a":';
      for (let i = 0; i < 1000; i++) {
        deeplyNested += '{"b":';
      }
      deeplyNested += '"value"';
      for (let i = 0; i < 1001; i++) {
        deeplyNested += '}';
      }
      
      await expect(manager.importKey(deeplyNested, testPassphrase))
        .rejects.toThrow(KeyImportError);
    });
  });
});

describe('Cross-platform Compatibility Tests', () => {
  itSkipIfNoIndexedDB('should handle different line endings', async () => {
    // This test would be more comprehensive in a real cross-platform environment
    // For now, we test that the implementation doesn't break with different formats
    
    const storage = await createStorage({ dbName: `test-compat-${Date.now()}` });
    const manager = new KeyExportImportManager(storage);
    const keyPair = await generateKeyPair();
    const keyId = 'compat-test';
    const passphrase = 'compat-passphrase';
    
    await storage.storeKeyPair(keyId, keyPair, passphrase);
    
    const options: KeyExportOptions = { format: 'json' };
    const result = await manager.exportKey(keyId, passphrase, options);
    
    // Test with different line endings
    const windowsFormat = (result.data as string).replace(/\n/g, '\r\n');
    const macFormat = (result.data as string).replace(/\n/g, '\r');
    
    // Should still be valid JSON and importable
    expect(() => JSON.parse(windowsFormat)).not.toThrow();
    expect(() => JSON.parse(macFormat)).not.toThrow();
    
    await storage.clearAllKeys();
    await storage.close();
  });
});