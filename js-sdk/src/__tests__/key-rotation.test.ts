/**
 * Unit tests for key rotation functionality
 */

import {
  KeyRotationManager,
  emergencyKeyRotation,
  batchRotateKeys,
  validateKeyRotationHistory
} from '../crypto/key-rotation.js';
import { generateKeyPair } from '../crypto/ed25519.js';
import { 
  KeyRotationError, 
  KeyStorageInterface, 
  Ed25519KeyPair,
  StoredKeyMetadata 
} from '../types.js';

// Mock storage implementation for testing
class MockKeyStorage implements KeyStorageInterface {
  private storage = new Map<string, { keyPair: Ed25519KeyPair; metadata: StoredKeyMetadata }>();

  async storeKeyPair(
    keyId: string, 
    keyPair: Ed25519KeyPair, 
    passphrase: string,
    metadata: Partial<StoredKeyMetadata> = {}
  ): Promise<void> {
    const fullMetadata: StoredKeyMetadata = {
      name: metadata.name || keyId,
      description: metadata.description || '',
      created: metadata.created || new Date().toISOString(),
      lastAccessed: new Date().toISOString(),
      tags: metadata.tags || []
    };
    
    this.storage.set(keyId, { keyPair, metadata: fullMetadata });
  }

  async retrieveKeyPair(keyId: string, passphrase: string): Promise<Ed25519KeyPair> {
    const stored = this.storage.get(keyId);
    if (!stored) {
      throw new Error(`Key ${keyId} not found`);
    }
    return stored.keyPair;
  }

  async deleteKeyPair(keyId: string): Promise<void> {
    this.storage.delete(keyId);
  }

  async listKeys(): Promise<Array<{ id: string; metadata: StoredKeyMetadata }>> {
    return Array.from(this.storage.entries()).map(([id, { metadata }]) => ({ id, metadata }));
  }

  async keyExists(keyId: string): Promise<boolean> {
    return this.storage.has(keyId);
  }

  async getStorageInfo(): Promise<{ used: number; available: number | null }> {
    return { used: this.storage.size * 100, available: 1000 };
  }

  async clearAllKeys(): Promise<void> {
    this.storage.clear();
  }

  async close(): Promise<void> {
    // No-op for mock
  }
}

// Mock generateKeyPair
jest.mock('../crypto/ed25519.js', () => ({
  generateKeyPair: jest.fn()
}));

// Mock key derivation
jest.mock('../crypto/key-derivation.js', () => ({
  deriveKeyFromKeyPair: jest.fn(),
  clearDerivedKey: jest.fn()
}));

const mockGenerateKeyPair = generateKeyPair as jest.MockedFunction<typeof generateKeyPair>;

describe('Key Rotation', () => {
  let storage: MockKeyStorage;
  let rotationManager: KeyRotationManager;

  beforeEach(() => {
    storage = new MockKeyStorage();
    rotationManager = new KeyRotationManager(storage);
    jest.clearAllMocks();

    // Setup default mock for generateKeyPair
    mockGenerateKeyPair.mockResolvedValue({
      privateKey: new Uint8Array(32),
      publicKey: new Uint8Array(32)
    });
  });

  describe('KeyRotationManager', () => {
    describe('createVersionedKeyPair', () => {
      it('should create initial versioned key pair', async () => {
        const keyId = 'test-key';
        const keyPair: Ed25519KeyPair = {
          privateKey: new Uint8Array(32),
          publicKey: new Uint8Array(32)
        };
        const passphrase = 'test-passphrase';

        const versionedKeyPair = await rotationManager.createVersionedKeyPair(
          keyId, 
          keyPair, 
          passphrase,
          { name: 'Test Key', description: 'A test key' }
        );

        expect(versionedKeyPair.keyId).toBe(keyId);
        expect(versionedKeyPair.currentVersion).toBe(1);
        expect(versionedKeyPair.versions[1]).toBeDefined();
        expect(versionedKeyPair.versions[1].active).toBe(true);
        expect(versionedKeyPair.versions[1].reason).toBe('initial');
        expect(versionedKeyPair.metadata.name).toBe('Test Key');
      });

      it('should use default metadata if none provided', async () => {
        const keyId = 'test-key';
        const keyPair: Ed25519KeyPair = {
          privateKey: new Uint8Array(32),
          publicKey: new Uint8Array(32)
        };

        const versionedKeyPair = await rotationManager.createVersionedKeyPair(
          keyId, 
          keyPair, 
          'passphrase'
        );

        expect(versionedKeyPair.metadata.name).toBe(keyId);
        expect(versionedKeyPair.metadata.description).toBe('');
        expect(versionedKeyPair.metadata.tags).toEqual([]);
      });
    });

    describe('rotateKey', () => {
      beforeEach(async () => {
        // Create initial versioned key pair
        const keyPair: Ed25519KeyPair = {
          privateKey: new Uint8Array(32),
          publicKey: new Uint8Array(32)
        };
        await rotationManager.createVersionedKeyPair('test-key', keyPair, 'passphrase');
      });

      it('should rotate key and create new version', async () => {
        const result = await rotationManager.rotateKey('test-key', 'passphrase', {
          reason: 'scheduled_rotation'
        });

        expect(result.newVersion).toBe(2);
        expect(result.previousVersion).toBe(1);
        expect(result.newKeyPair).toBeDefined();
        expect(result.oldVersionPreserved).toBe(false);
        expect(mockGenerateKeyPair).toHaveBeenCalled();
      });

      it('should preserve old version when requested', async () => {
        const result = await rotationManager.rotateKey('test-key', 'passphrase', {
          keepOldVersion: true,
          reason: 'security_update'
        });

        expect(result.oldVersionPreserved).toBe(true);
        
        const versions = await rotationManager.listKeyVersions('test-key', 'passphrase');
        expect(versions).toHaveLength(2);
        expect(versions.find(v => v.version === 1)?.active).toBe(false);
        expect(versions.find(v => v.version === 2)?.active).toBe(true);
      });

      it('should handle metadata updates', async () => {
        await rotationManager.rotateKey('test-key', 'passphrase', {
          metadata: {
            description: 'Updated after rotation',
            tags: ['rotated', 'secure']
          }
        });

        const versions = await rotationManager.listKeyVersions('test-key', 'passphrase');
        const currentVersion = versions.find(v => v.active);
        expect(currentVersion?.reason).toBe('rotation');
      });

      it('should throw error for non-existent key', async () => {
        await expect(
          rotationManager.rotateKey('non-existent', 'passphrase')
        ).rejects.toThrow(KeyRotationError);
        
        await expect(
          rotationManager.rotateKey('non-existent', 'passphrase')
        ).rejects.toThrow('Key with ID non-existent not found');
      });
    });

    describe('getKeyVersion', () => {
      beforeEach(async () => {
        const keyPair: Ed25519KeyPair = {
          privateKey: new Uint8Array(32),
          publicKey: new Uint8Array(32)
        };
        await rotationManager.createVersionedKeyPair('test-key', keyPair, 'passphrase');
        await rotationManager.rotateKey('test-key', 'passphrase');
      });

      it('should retrieve specific key version', async () => {
        const version1 = await rotationManager.getKeyVersion('test-key', 1, 'passphrase');
        const version2 = await rotationManager.getKeyVersion('test-key', 2, 'passphrase');

        expect(version1).toBeDefined();
        expect(version2).toBeDefined();
        expect(version1?.version).toBe(1);
        expect(version2?.version).toBe(2);
        expect(version1?.active).toBe(false);
        expect(version2?.active).toBe(true);
      });

      it('should return null for non-existent version', async () => {
        const version = await rotationManager.getKeyVersion('test-key', 99, 'passphrase');
        expect(version).toBeNull();
      });

      it('should return null for non-existent key', async () => {
        const version = await rotationManager.getKeyVersion('non-existent', 1, 'passphrase');
        expect(version).toBeNull();
      });
    });

    describe('listKeyVersions', () => {
      beforeEach(async () => {
        const keyPair: Ed25519KeyPair = {
          privateKey: new Uint8Array(32),
          publicKey: new Uint8Array(32)
        };
        await rotationManager.createVersionedKeyPair('test-key', keyPair, 'passphrase');
        await rotationManager.rotateKey('test-key', 'passphrase', { reason: 'first_rotation' });
        await rotationManager.rotateKey('test-key', 'passphrase', { reason: 'second_rotation' });
      });

      it('should list all versions', async () => {
        const versions = await rotationManager.listKeyVersions('test-key', 'passphrase');
        
        expect(versions).toHaveLength(3);
        expect(versions.map(v => v.version).sort()).toEqual([1, 2, 3]);
        expect(versions.find(v => v.version === 1)?.reason).toBe('initial');
        expect(versions.find(v => v.version === 2)?.reason).toBe('first_rotation');
        expect(versions.find(v => v.version === 3)?.reason).toBe('second_rotation');
        expect(versions.filter(v => v.active)).toHaveLength(1);
      });

      it('should return empty array for non-existent key', async () => {
        const versions = await rotationManager.listKeyVersions('non-existent', 'passphrase');
        expect(versions).toEqual([]);
      });
    });

    describe('cleanupOldVersions', () => {
      beforeEach(async () => {
        const keyPair: Ed25519KeyPair = {
          privateKey: new Uint8Array(32),
          publicKey: new Uint8Array(32)
        };
        await rotationManager.createVersionedKeyPair('test-key', keyPair, 'passphrase');
        await rotationManager.rotateKey('test-key', 'passphrase', { keepOldVersion: true });
        await rotationManager.rotateKey('test-key', 'passphrase', { keepOldVersion: true });
        await rotationManager.rotateKey('test-key', 'passphrase', { keepOldVersion: true });
      });

      it('should remove old inactive versions', async () => {
        const removedCount = await rotationManager.cleanupOldVersions('test-key', 'passphrase', 2);
        
        expect(removedCount).toBe(2); // Should remove versions 1 and 2, keep 3 and 4
        
        const versions = await rotationManager.listKeyVersions('test-key', 'passphrase');
        expect(versions).toHaveLength(2);
        expect(versions.map(v => v.version).sort()).toEqual([3, 4]);
      });

      it('should never remove active version', async () => {
        const removedCount = await rotationManager.cleanupOldVersions('test-key', 'passphrase', 1);
        
        const versions = await rotationManager.listKeyVersions('test-key', 'passphrase');
        const activeVersion = versions.find(v => v.active);
        expect(activeVersion).toBeDefined();
        expect(activeVersion?.version).toBe(4);
      });

      it('should throw error for non-existent key', async () => {
        await expect(
          rotationManager.cleanupOldVersions('non-existent', 'passphrase')
        ).rejects.toThrow(KeyRotationError);
      });
    });
  });

  describe('emergencyKeyRotation', () => {
    beforeEach(async () => {
      const keyPair: Ed25519KeyPair = {
        privateKey: new Uint8Array(32),
        publicKey: new Uint8Array(32)
      };
      await rotationManager.createVersionedKeyPair('test-key', keyPair, 'old-passphrase');
    });

    it('should perform emergency rotation with new passphrase', async () => {
      const result = await emergencyKeyRotation(
        'test-key',
        'old-passphrase',
        'new-passphrase',
        storage,
        'security_breach'
      );

      expect(result.newVersion).toBe(2);
      expect(result.oldVersionPreserved).toBe(false);
      expect(result.rotatedDerivedKeys).toEqual([]);
    });

    it('should work with same passphrase', async () => {
      const result = await emergencyKeyRotation(
        'test-key',
        'old-passphrase',
        'old-passphrase',
        storage
      );

      expect(result.newVersion).toBe(2);
      expect(result.previousVersion).toBe(1);
    });
  });

  describe('batchRotateKeys', () => {
    beforeEach(async () => {
      const keyPair1: Ed25519KeyPair = {
        privateKey: new Uint8Array(32),
        publicKey: new Uint8Array(32)
      };
      const keyPair2: Ed25519KeyPair = {
        privateKey: new Uint8Array(32),
        publicKey: new Uint8Array(32)
      };
      
      await rotationManager.createVersionedKeyPair('key1', keyPair1, 'passphrase');
      await rotationManager.createVersionedKeyPair('key2', keyPair2, 'passphrase');
    });

    it('should rotate multiple keys successfully', async () => {
      const results = await batchRotateKeys(['key1', 'key2'], 'passphrase', storage);
      
      expect(Object.keys(results)).toHaveLength(2);
      expect(results.key1).toBeInstanceOf(Object);
      expect(results.key2).toBeInstanceOf(Object);
      expect((results.key1 as any).newVersion).toBe(2);
      expect((results.key2 as any).newVersion).toBe(2);
    });

    it('should handle partial failures gracefully', async () => {
      const results = await batchRotateKeys(['key1', 'non-existent', 'key2'], 'passphrase', storage);
      
      expect(Object.keys(results)).toHaveLength(3);
      expect(results.key1).toBeInstanceOf(Object);
      expect(results['non-existent']).toBeInstanceOf(Error);
      expect(results.key2).toBeInstanceOf(Object);
    });
  });

  describe('validateKeyRotationHistory', () => {
    beforeEach(async () => {
      const keyPair: Ed25519KeyPair = {
        privateKey: new Uint8Array(32),
        publicKey: new Uint8Array(32)
      };
      await rotationManager.createVersionedKeyPair('test-key', keyPair, 'passphrase');
      await rotationManager.rotateKey('test-key', 'passphrase', { reason: 'scheduled' });
      await rotationManager.rotateKey('test-key', 'passphrase', { reason: 'security' });
    });

    it('should validate correct rotation history', async () => {
      const validation = await validateKeyRotationHistory('test-key', 'passphrase', storage);
      
      expect(validation.valid).toBe(true);
      expect(validation.issues).toHaveLength(0);
      expect(validation.rotationCount).toBe(2);
      expect(validation.lastRotation).toBeDefined();
    });

    it('should detect issues with rotation history', async () => {
      // This test would need more complex setup to create actual issues
      // For now, we'll test the non-existent key case
      const validation = await validateKeyRotationHistory('non-existent', 'passphrase', storage);
      
      expect(validation.valid).toBe(false);
      expect(validation.issues).toContain('No key versions found');
      expect(validation.rotationCount).toBe(0);
      expect(validation.lastRotation).toBeNull();
    });
  });

  describe('Error handling', () => {
    it('should handle storage errors gracefully', async () => {
      const failingStorage: KeyStorageInterface = {
        ...storage,
        retrieveKeyPair: jest.fn().mockRejectedValue(new Error('Storage error'))
      } as any;

      const failingManager = new KeyRotationManager(failingStorage);
      
      await expect(
        failingManager.rotateKey('test-key', 'passphrase')
      ).rejects.toThrow(KeyRotationError);
    });

    it('should handle key generation failures', async () => {
      mockGenerateKeyPair.mockRejectedValueOnce(new Error('Key generation failed'));
      
      const keyPair: Ed25519KeyPair = {
        privateKey: new Uint8Array(32),
        publicKey: new Uint8Array(32)
      };
      await rotationManager.createVersionedKeyPair('test-key', keyPair, 'passphrase');
      
      await expect(
        rotationManager.rotateKey('test-key', 'passphrase')
      ).rejects.toThrow(KeyRotationError);
    });
  });
});