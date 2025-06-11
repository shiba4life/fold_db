/**
 * Key rotation utilities for the DataFold JavaScript SDK
 * Provides secure key rotation capabilities with versioning and backward compatibility
 */

import { 
  Ed25519KeyPair,
  KeyRotationOptions,
  KeyRotationResult,
  KeyVersion,
  VersionedKeyPair,
  KeyRotationError,
  StoredKeyMetadata,
  DerivedKeyInfo,
  KeyStorageInterface
} from '../types.js';
import { generateKeyPair } from './ed25519.js';
import { deriveKey, deriveKeyFromKeyPair, clearDerivedKey } from './key-derivation.js';

/**
 * Key rotation manager for handling secure key updates
 */
export class KeyRotationManager {
  constructor(private storage: KeyStorageInterface) {}

  /**
   * Rotate a key pair, generating a new version while optionally preserving the old one
   */
  async rotateKey(
    keyId: string,
    passphrase: string,
    options: KeyRotationOptions = {}
  ): Promise<KeyRotationResult> {
    try {
      // Retrieve current versioned key pair
      const versionedKeyPair = await this.getVersionedKeyPair(keyId, passphrase);
      
      if (!versionedKeyPair) {
        throw new KeyRotationError(
          `Key with ID ${keyId} not found`,
          'KEY_NOT_FOUND'
        );
      }

      // Generate new key pair
      const newKeyPair = await generateKeyPair();
      const newVersion = versionedKeyPair.currentVersion + 1;
      const previousVersion = versionedKeyPair.currentVersion;

      // Create new key version
      const newKeyVersion: KeyVersion = {
        version: newVersion,
        keyPair: newKeyPair,
        created: new Date().toISOString(),
        active: true,
        reason: options.reason || 'rotation',
        derivedKeys: {}
      };

      // Handle derived key rotation if requested
      const rotatedDerivedKeys: string[] = [];
      if (options.rotateDerivedKeys) {
        const currentVersion = versionedKeyPair.versions[previousVersion];
        if (currentVersion.derivedKeys) {
          for (const [derivedKeyName, derivedKeyInfo] of Object.entries(currentVersion.derivedKeys)) {
            try {
              // Re-derive key with new master key
              const newDerivedKey = await deriveKeyFromKeyPair(newKeyPair, {
                algorithm: derivedKeyInfo.algorithm,
                salt: derivedKeyInfo.salt,
                info: derivedKeyInfo.info,
                iterations: derivedKeyInfo.iterations,
                hash: derivedKeyInfo.hash as 'SHA-256' | 'SHA-384' | 'SHA-512',
                length: derivedKeyInfo.key.length
              });

              newKeyVersion.derivedKeys![derivedKeyName] = newDerivedKey;
              rotatedDerivedKeys.push(derivedKeyName);
            } catch (error) {
              console.warn(`Failed to rotate derived key ${derivedKeyName}:`, error);
            }
          }
        }
      }

      // Prepare updated versioned key pair
      const updatedVersionedKeyPair: VersionedKeyPair = {
        ...versionedKeyPair,
        currentVersion: newVersion,
        versions: {
          ...versionedKeyPair.versions,
          [newVersion]: newKeyVersion
        }
      };

      // Update metadata if provided
      if (options.metadata) {
        updatedVersionedKeyPair.metadata = {
          ...updatedVersionedKeyPair.metadata,
          ...options.metadata,
          lastAccessed: new Date().toISOString()
        };
      }

      // Handle old version preservation
      let oldVersionPreserved = false;
      if (options.keepOldVersion) {
        // Mark old version as inactive but keep it
        const oldVersion = updatedVersionedKeyPair.versions[previousVersion];
        if (oldVersion) {
          oldVersion.active = false;
          oldVersionPreserved = true;
        }
      } else {
        // Remove old version
        delete updatedVersionedKeyPair.versions[previousVersion];
      }

      // Store the updated versioned key pair
      await this.storeVersionedKeyPair(updatedVersionedKeyPair, passphrase);

      return {
        newKeyPair,
        newVersion,
        previousVersion,
        oldVersionPreserved,
        rotatedDerivedKeys
      };

    } catch (error) {
      if (error instanceof KeyRotationError) {
        throw error;
      }
      throw new KeyRotationError(
        `Key rotation failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'ROTATION_FAILED'
      );
    }
  }

  /**
   * Get a specific version of a key pair
   */
  async getKeyVersion(
    keyId: string,
    version: number,
    passphrase: string
  ): Promise<KeyVersion | null> {
    const versionedKeyPair = await this.getVersionedKeyPair(keyId, passphrase);
    
    if (!versionedKeyPair || !versionedKeyPair.versions[version]) {
      return null;
    }

    return versionedKeyPair.versions[version];
  }

  /**
   * List all versions of a key
   */
  async listKeyVersions(
    keyId: string,
    passphrase: string
  ): Promise<Array<{ version: number; created: string; active: boolean; reason: string }>> {
    const versionedKeyPair = await this.getVersionedKeyPair(keyId, passphrase);
    
    if (!versionedKeyPair) {
      return [];
    }

    return Object.values(versionedKeyPair.versions).map(version => ({
      version: version.version,
      created: version.created,
      active: version.active,
      reason: version.reason
    }));
  }

  /**
   * Clean up old inactive key versions (keeping only the most recent N versions)
   */
  async cleanupOldVersions(
    keyId: string,
    passphrase: string,
    keepVersions: number = 2
  ): Promise<number> {
    const versionedKeyPair = await this.getVersionedKeyPair(keyId, passphrase);
    
    if (!versionedKeyPair) {
      throw new KeyRotationError(
        `Key with ID ${keyId} not found`,
        'KEY_NOT_FOUND'
      );
    }

    // Sort versions by version number (descending)
    const sortedVersions = Object.values(versionedKeyPair.versions)
      .sort((a, b) => b.version - a.version);

    // Keep the current active version and the specified number of recent versions
    const versionsToKeep = sortedVersions.slice(0, keepVersions);
    const versionsToRemove = sortedVersions.slice(keepVersions);

    // Remove old versions
    let removedCount = 0;
    for (const version of versionsToRemove) {
      if (!version.active) { // Never remove the active version
        // Clear derived keys
        if (version.derivedKeys) {
          for (const derivedKey of Object.values(version.derivedKeys)) {
            clearDerivedKey(derivedKey);
          }
        }

        // Clear key pair (defensively handle non-Uint8Array types)
        if (version.keyPair.privateKey instanceof Uint8Array) {
          version.keyPair.privateKey.fill(0);
        }
        if (version.keyPair.publicKey instanceof Uint8Array) {
          version.keyPair.publicKey.fill(0);
        }

        // Remove from versions
        delete versionedKeyPair.versions[version.version];
        removedCount++;
      }
    }

    // Store updated versioned key pair
    if (removedCount > 0) {
      await this.storeVersionedKeyPair(versionedKeyPair, passphrase);
    }

    return removedCount;
  }

  /**
   * Add a derived key to a specific version
   */
  async addDerivedKey(
    keyId: string,
    version: number,
    derivedKeyName: string,
    derivedKeyInfo: DerivedKeyInfo,
    passphrase: string
  ): Promise<void> {
    const versionedKeyPair = await this.getVersionedKeyPair(keyId, passphrase);
    
    if (!versionedKeyPair || !versionedKeyPair.versions[version]) {
      throw new KeyRotationError(
        `Key version ${version} for key ${keyId} not found`,
        'KEY_VERSION_NOT_FOUND'
      );
    }

    const keyVersion = versionedKeyPair.versions[version];
    if (!keyVersion.derivedKeys) {
      keyVersion.derivedKeys = {};
    }

    keyVersion.derivedKeys[derivedKeyName] = derivedKeyInfo;

    await this.storeVersionedKeyPair(versionedKeyPair, passphrase);
  }

  /**
   * Remove a derived key from a specific version
   */
  async removeDerivedKey(
    keyId: string,
    version: number,
    derivedKeyName: string,
    passphrase: string
  ): Promise<void> {
    const versionedKeyPair = await this.getVersionedKeyPair(keyId, passphrase);
    
    if (!versionedKeyPair || !versionedKeyPair.versions[version]) {
      throw new KeyRotationError(
        `Key version ${version} for key ${keyId} not found`,
        'KEY_VERSION_NOT_FOUND'
      );
    }

    const keyVersion = versionedKeyPair.versions[version];
    if (keyVersion.derivedKeys && keyVersion.derivedKeys[derivedKeyName]) {
      // Clear the derived key before removing
      clearDerivedKey(keyVersion.derivedKeys[derivedKeyName]);
      delete keyVersion.derivedKeys[derivedKeyName];

      await this.storeVersionedKeyPair(versionedKeyPair, passphrase);
    }
  }

  /**
   * Create a versioned key pair from a regular key pair
   */
  async createVersionedKeyPair(
    keyId: string,
    keyPair: Ed25519KeyPair,
    passphrase: string,
    metadata: Partial<StoredKeyMetadata> = {}
  ): Promise<VersionedKeyPair> {
    const initialVersion: KeyVersion = {
      version: 1,
      keyPair,
      created: new Date().toISOString(),
      active: true,
      reason: 'initial',
      derivedKeys: {}
    };

    const fullMetadata: StoredKeyMetadata = {
      name: metadata.name || keyId,
      description: metadata.description || '',
      created: metadata.created || new Date().toISOString(),
      lastAccessed: new Date().toISOString(),
      tags: metadata.tags || []
    };

    const versionedKeyPair: VersionedKeyPair = {
      keyId,
      currentVersion: 1,
      versions: {
        1: initialVersion
      },
      metadata: fullMetadata
    };

    await this.storeVersionedKeyPair(versionedKeyPair, passphrase);
    return versionedKeyPair;
  }

  /**
   * Retrieve versioned key pair from storage
   */
  private async getVersionedKeyPair(
    keyId: string,
    passphrase: string
  ): Promise<VersionedKeyPair | null> {
    try {
      // For now, we'll store versioned key pairs with a special prefix
      const versionedKeyId = `versioned_${keyId}`;
      const keyPair = await this.storage.retrieveKeyPair(versionedKeyId, passphrase);
      
      // The keyPair.privateKey will contain serialized versioned data
      // This is a simplified approach - in production, you'd want a more sophisticated storage format
      const serializedData = new TextDecoder().decode(keyPair.privateKey);
      const parsed = JSON.parse(serializedData) as any;
      
      // Restore Uint8Array objects from plain objects
      const versionedKeyPair: VersionedKeyPair = {
        ...parsed,
        versions: {}
      };
      
      // Restore each version's key pairs
      for (const [versionNum, version] of Object.entries(parsed.versions)) {
        versionedKeyPair.versions[parseInt(versionNum)] = {
          ...version as any,
          keyPair: {
            privateKey: new Uint8Array(Object.values((version as any).keyPair.privateKey)),
            publicKey: new Uint8Array(Object.values((version as any).keyPair.publicKey))
          }
        };
      }
      
      return versionedKeyPair;
    } catch {
      return null;
    }
  }

  /**
   * Store versioned key pair to storage
   */
  private async storeVersionedKeyPair(
    versionedKeyPair: VersionedKeyPair,
    passphrase: string
  ): Promise<void> {
    // Serialize the versioned key pair
    const serializedData = JSON.stringify(versionedKeyPair);
    const serializedBytes = new TextEncoder().encode(serializedData);

    // Create a dummy key pair for storage (we're hijacking the storage format)
    const dummyKeyPair: Ed25519KeyPair = {
      privateKey: serializedBytes,
      publicKey: new Uint8Array(32) // Dummy public key
    };

    const versionedKeyId = `versioned_${versionedKeyPair.keyId}`;
    await this.storage.storeKeyPair(
      versionedKeyId,
      dummyKeyPair,
      passphrase,
      versionedKeyPair.metadata
    );
  }
}

/**
 * Emergency key rotation for compromised keys
 */
export async function emergencyKeyRotation(
  keyId: string,
  oldPassphrase: string,
  newPassphrase: string,
  storage: KeyStorageInterface,
  reason: string = 'emergency_compromise'
): Promise<KeyRotationResult> {
  const rotationManager = new KeyRotationManager(storage);

  // Rotate the key with emergency settings
  const result = await rotationManager.rotateKey(keyId, oldPassphrase, {
    keepOldVersion: false, // Don't keep compromised version
    reason,
    rotateDerivedKeys: true, // Rotate all derived keys
    metadata: {
      lastAccessed: new Date().toISOString(),
      tags: ['emergency_rotated']
    }
  });

  // If passphrase is changing, re-store with new passphrase
  if (oldPassphrase !== newPassphrase) {
    const versionedKeyPair = await rotationManager['getVersionedKeyPair'](keyId, oldPassphrase);
    if (versionedKeyPair) {
      // Delete old storage entry
      await storage.deleteKeyPair(`versioned_${keyId}`);
      
      // Store with new passphrase
      await rotationManager['storeVersionedKeyPair'](versionedKeyPair, newPassphrase);
    }
  }

  return result;
}

/**
 * Batch rotate multiple keys (useful for organization-wide rotation)
 */
export async function batchRotateKeys(
  keyIds: string[],
  passphrase: string,
  storage: KeyStorageInterface,
  options: KeyRotationOptions = {}
): Promise<Record<string, KeyRotationResult | Error>> {
  const results: Record<string, KeyRotationResult | Error> = {};
  const rotationManager = new KeyRotationManager(storage);

  for (const keyId of keyIds) {
    try {
      results[keyId] = await rotationManager.rotateKey(keyId, passphrase, options);
    } catch (error) {
      results[keyId] = error instanceof Error ? error : new Error('Unknown error');
    }
  }

  return results;
}

/**
 * Validate key rotation history for audit purposes
 */
export async function validateKeyRotationHistory(
  keyId: string,
  passphrase: string,
  storage: KeyStorageInterface
): Promise<{
  valid: boolean;
  issues: string[];
  rotationCount: number;
  lastRotation: string | null;
}> {
  const rotationManager = new KeyRotationManager(storage);
  const issues: string[] = [];
  let valid = true;

  try {
    const versions = await rotationManager.listKeyVersions(keyId, passphrase);
    
    if (versions.length === 0) {
      return {
        valid: false,
        issues: ['No key versions found'],
        rotationCount: 0,
        lastRotation: null
      };
    }

    // Check for gaps in version numbers
    const sortedVersions = versions.sort((a, b) => a.version - b.version);
    for (let i = 0; i < sortedVersions.length - 1; i++) {
      if (sortedVersions[i + 1].version - sortedVersions[i].version > 1) {
        issues.push(`Version gap detected: ${sortedVersions[i].version} to ${sortedVersions[i + 1].version}`);
        valid = false;
      }
    }

    // Check that only one version is active
    const activeVersions = versions.filter(v => v.active);
    if (activeVersions.length !== 1) {
      issues.push(`Expected 1 active version, found ${activeVersions.length}`);
      valid = false;
    }

    // Find last rotation (excluding initial version)
    const rotationVersions = versions.filter(v => v.reason !== 'initial');
    const lastRotation = rotationVersions.length > 0 
      ? rotationVersions.sort((a, b) => new Date(b.created).getTime() - new Date(a.created).getTime())[0].created
      : null;

    return {
      valid,
      issues,
      rotationCount: rotationVersions.length,
      lastRotation
    };
  } catch (error) {
    return {
      valid: false,
      issues: [`Validation failed: ${error instanceof Error ? error.message : 'Unknown error'}`],
      rotationCount: 0,
      lastRotation: null
    };
  }
}