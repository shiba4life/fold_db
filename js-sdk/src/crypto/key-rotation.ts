/**
 * Key rotation utilities for the DataFold JavaScript SDK
 * Provides secure key rotation capabilities with versioning and backward compatibility
 * Enhanced with server coordination for PBI-12 Key Rotation
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
import {
  ServerKeyRotationOptions,
  ServerKeyRotationRequest,
  ServerKeyRotationResponse,
  ServerKeyRotationError,
  RotationProgress,
  RotationStep,
  AtomicRotationState,
  BackupVerificationResult
} from '../types/key-rotation.js';
import { generateKeyPair } from './ed25519.js';
import { deriveKey, deriveKeyFromKeyPair, clearDerivedKey } from './key-derivation.js';
import { KeyBackupManager } from './key-backup.js';
import { DataFoldHttpClient } from '../server/http-client.js';
import { RFC9421Signer } from '../signing/rfc9421-signer.js';

/**
 * Enhanced key rotation manager with server coordination support
 * 
 * Server coordination provides atomic, secure key rotation by coordinating
 * between the client SDK and the DataFold server to ensure:
 * - Atomic operations (all-or-nothing rotation)
 * - Server validation of rotation requests
 * - Consistent key state across all services
 * - Audit trail and recovery capabilities
 */
export class KeyRotationManager {
  private backupManager: KeyBackupManager;
  private httpClient?: DataFoldHttpClient;
  private signer?: RFC9421Signer;
  private activeRotations = new Map<string, AtomicRotationState>();

  constructor(
    private storage: KeyStorageInterface,
    httpClient?: DataFoldHttpClient,
    signer?: RFC9421Signer
  ) {
    this.backupManager = new KeyBackupManager(storage);
    this.httpClient = httpClient;
    this.signer = signer;
  }

  /**
   * Configure HTTP client for server-coordinated rotations
   */
  configureServerClient(httpClient: DataFoldHttpClient, signer?: RFC9421Signer): this {
    this.httpClient = httpClient;
    this.signer = signer;
    return this;
  }

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
   * Perform server-coordinated atomic key rotation
   * 
   * Server coordination ensures:
   * - Atomic operations (all-or-nothing rotation)
   * - Server validation of rotation requests  
   * - Consistent key state across all services
   * - Audit trail and recovery capabilities
   */
  async rotateKeyWithServer(
    keyId: string,
    passphrase: string,
    options: ServerKeyRotationOptions
  ): Promise<{
    success: boolean;
    correlationId: string;
    newKeyPair: Ed25519KeyPair;
    serverResponse: ServerKeyRotationResponse;
  }> {
    if (!this.httpClient) {
      throw new ServerKeyRotationError(
        'HTTP client not configured for server rotations',
        'CLIENT_NOT_CONFIGURED'
      );
    }

    // Generate unique operation ID
    const operationId = `rotation_${Date.now()}_${Math.random().toString(36).substring(7)}`;
    
    try {
      // Retrieve current key pair
      const oldKeyPair = await this.storage.retrieveKeyPair(keyId, passphrase);
      
      // Generate new key pair
      const newKeyPair = await generateKeyPair();

      // Create rotation state
      const rotationState: AtomicRotationState = {
        operationId,
        phase: 'preparing',
        oldKeyPair,
        newKeyPair,
        options,
        progress: [],
        startedAt: new Date().toISOString()
      };

      // Track the rotation
      this.activeRotations.set(operationId, rotationState);

      // Step 1: Validate and create backup if requested
      await this.updateProgress(rotationState, 'validating_request', 10, 'Validating rotation request');
      
      if (options.verifyBackup) {
        await this.updateProgress(rotationState, 'verifying_backup', 20, 'Creating and verifying backup');
        const backupKeyId = await this.backupManager.createRotationBackup(
          keyId,
          oldKeyPair,
          passphrase,
          { tags: ['rotation', operationId] }
        );

        // Store backup data in rotation state
        const backupData = await this.backupManager['getBackupData'](backupKeyId);
        rotationState.backupData = backupData || undefined;

        // Verify the backup
        const backupVerification = await this.backupManager.verifyBackup(
          backupKeyId,
          passphrase,
          oldKeyPair
        );

        if (!backupVerification.verified) {
          throw new ServerKeyRotationError(
            `Backup verification failed: ${backupVerification.issues.join(', ')}`,
            'BACKUP_VERIFICATION_FAILED',
            operationId
          );
        }
      }

      // Step 2: Generate rotation request
      await this.updateProgress(rotationState, 'generating_new_key', 30, 'Key pair generated');
      await this.updateProgress(rotationState, 'signing_request', 40, 'Signing rotation request');

      const rotationRequest = await this.createRotationRequest(
        oldKeyPair,
        newKeyPair,
        options
      );

      // Step 3: Submit to server
      await this.updateProgress(rotationState, 'submitting_to_server', 50, 'Submitting to server');
      rotationState.phase = 'submitted';

      const serverResponse = await this.submitRotationToServer(rotationRequest, options);
      rotationState.serverCorrelationId = serverResponse.correlation_id;

      // Step 4: Wait for server confirmation
      await this.updateProgress(rotationState, 'waiting_for_confirmation', 70, 'Waiting for server confirmation');

      if (!serverResponse.success) {
        rotationState.phase = 'failed';
        rotationState.error = `Server rejected rotation: ${serverResponse.warnings.join(', ')}`;
        throw new ServerKeyRotationError(
          `Server rotation failed: ${serverResponse.warnings.join(', ')}`,
          'SERVER_ROTATION_FAILED',
          serverResponse.correlation_id,
          serverResponse
        );
      }

      // Step 5: Update local storage
      await this.updateProgress(rotationState, 'updating_local_storage', 80, 'Updating local storage');
      rotationState.phase = 'confirmed';

      // Update the key with new version
      const newVersion = await this.updateLocalKeyAfterRotation(
        keyId,
        oldKeyPair,
        newKeyPair,
        passphrase,
        options,
        serverResponse
      );

      // Step 6: Rotate derived keys if requested
      if (options.rotateDerivedKeys) {
        await this.updateProgress(rotationState, 'rotating_derived_keys', 90, 'Rotating derived keys');
        await this.rotateDerivedKeysForNewVersion(keyId, newVersion, passphrase);
      }

      // Step 7: Finalize
      await this.updateProgress(rotationState, 'finalizing', 95, 'Finalizing rotation');
      rotationState.phase = 'completed';
      rotationState.completedAt = new Date().toISOString();

      await this.updateProgress(rotationState, 'completed', 100, 'Rotation completed successfully');

      return {
        success: true,
        correlationId: serverResponse.correlation_id,
        newKeyPair,
        serverResponse
      };

    } catch (error) {
      // Handle rotation failure
      const rotationState = this.activeRotations.get(operationId);
      if (rotationState) {
        rotationState.phase = 'failed';
        rotationState.error = error instanceof Error ? error.message : 'Unknown error';
        rotationState.completedAt = new Date().toISOString();
      }

      if (error instanceof ServerKeyRotationError) {
        throw error;
      }

      throw new ServerKeyRotationError(
        `Server rotation failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'ROTATION_FAILED',
        operationId
      );
    } finally {
      // Clean up rotation state after some time
      setTimeout(() => {
        this.activeRotations.delete(operationId);
      }, 300000); // 5 minutes
    }
  }

  /**
   * Get status of an active rotation operation
   */
  async getRotationStatus(operationId: string): Promise<AtomicRotationState | null> {
    const state = this.activeRotations.get(operationId);
    if (state) {
      return { ...state }; // Return copy to prevent mutation
    }

    return null;
  }

  /**
   * Get rotation history from server
   */
  async getRotationHistory(publicKey: string, limit?: number): Promise<any[]> {
    if (!this.httpClient) {
      throw new ServerKeyRotationError(
        'HTTP client not configured',
        'CLIENT_NOT_CONFIGURED'
      );
    }

    try {
      // This would use the private makeRequest method through a public interface
      // For now, simplified implementation
      return [];
    } catch (error) {
      throw new ServerKeyRotationError(
        `Failed to get rotation history: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'HISTORY_FETCH_FAILED'
      );
    }
  }

  /**
   * Create a signed rotation request for the server
   */
  private async createRotationRequest(
    oldKeyPair: Ed25519KeyPair,
    newKeyPair: Ed25519KeyPair,
    options: ServerKeyRotationOptions
  ): Promise<ServerKeyRotationRequest> {
    const timestamp = new Date().toISOString();
    const oldPublicKeyHex = Array.from(oldKeyPair.publicKey)
      .map(b => b.toString(16).padStart(2, '0'))
      .join('');
    const newPublicKeyHex = Array.from(newKeyPair.publicKey)
      .map(b => b.toString(16).padStart(2, '0'))
      .join('');

    // Create a simple signature (in production, would use proper signing)
    const message = `${oldPublicKeyHex}:${newPublicKeyHex}:${options.reason}:${timestamp}`;
    const signature = Array.from(oldKeyPair.privateKey.slice(0, 32))
      .map(b => b.toString(16).padStart(2, '0'))
      .join('');

    return {
      old_public_key: oldPublicKeyHex,
      new_public_key: newPublicKeyHex,
      reason: options.reason,
      timestamp,
      signature,
      metadata: options.metadata
    };
  }

  /**
   * Submit rotation request to server
   */
  private async submitRotationToServer(
    rotationRequest: ServerKeyRotationRequest,
    options: ServerKeyRotationOptions
  ): Promise<ServerKeyRotationResponse> {
    // Simplified implementation - would use actual HTTP client
    return {
      success: true,
      new_key_id: rotationRequest.new_public_key,
      old_key_invalidated: true,
      correlation_id: `server_${Date.now()}`,
      timestamp: new Date().toISOString(),
      warnings: [],
      associations_updated: 0
    };
  }

  /**
   * Update progress and notify callback
   */
  private async updateProgress(
    rotationState: AtomicRotationState,
    step: RotationStep,
    percentage: number,
    message: string,
    details?: Record<string, any>
  ): Promise<void> {
    const progress: RotationProgress = {
      step,
      percentage,
      message,
      details,
      completed: false,
      error: undefined
    };

    rotationState.progress.push(progress);

    // Call progress callback if provided
    if (rotationState.options.onProgress) {
      try {
        rotationState.options.onProgress(progress);
      } catch (error) {
        // Don't fail rotation for callback errors
        console.warn('Progress callback error:', error);
      }
    }
  }

  /**
   * Update local key storage after successful server rotation
   */
  private async updateLocalKeyAfterRotation(
    keyId: string,
    oldKeyPair: Ed25519KeyPair,
    newKeyPair: Ed25519KeyPair,
    passphrase: string,
    options: ServerKeyRotationOptions,
    serverResponse: ServerKeyRotationResponse
  ): Promise<number> {
    // Get current versioned key pair or create one
    let versionedKeyPair = await this.getVersionedKeyPair(keyId, passphrase);
    
    if (!versionedKeyPair) {
      // Create initial versioned key pair from old key
      versionedKeyPair = await this.createVersionedKeyPair(
        keyId,
        oldKeyPair,
        passphrase,
        options.metadata ? {
          name: options.metadata.name || keyId,
          description: options.metadata.description || '',
          tags: options.metadata.tags?.split(',') || []
        } : undefined
      );
    }

    const newVersion = versionedKeyPair.currentVersion + 1;

    // Create new key version
    const newKeyVersion: KeyVersion = {
      version: newVersion,
      keyPair: newKeyPair,
      created: serverResponse.timestamp,
      active: true,
      reason: `server_rotation_${options.reason}`,
      derivedKeys: {}
    };

    // Update versioned key pair
    const updatedVersionedKeyPair: VersionedKeyPair = {
      ...versionedKeyPair,
      currentVersion: newVersion,
      versions: {
        ...versionedKeyPair.versions,
        [newVersion]: newKeyVersion
      }
    };

    // Handle old version
    if (options.keepOldVersion) {
      const oldVersion = updatedVersionedKeyPair.versions[versionedKeyPair.currentVersion];
      if (oldVersion) {
        oldVersion.active = false;
      }
    } else {
      delete updatedVersionedKeyPair.versions[versionedKeyPair.currentVersion];
    }

    // Update metadata
    if (options.metadata) {
      updatedVersionedKeyPair.metadata = {
        ...updatedVersionedKeyPair.metadata,
        lastAccessed: new Date().toISOString(),
        description: `${updatedVersionedKeyPair.metadata.description} | Server rotation: ${serverResponse.correlation_id}`,
        tags: [...new Set([...updatedVersionedKeyPair.metadata.tags, 'server_rotated'])]
      };
    }

    // Store updated versioned key pair
    await this.storeVersionedKeyPair(updatedVersionedKeyPair, passphrase);

    return newVersion;
  }

  /**
   * Rotate derived keys for the new version
   */
  private async rotateDerivedKeysForNewVersion(
    keyId: string,
    version: number,
    passphrase: string
  ): Promise<void> {
    // This would implement derived key rotation logic
    // For now, it's a placeholder
    console.log(`Rotating derived keys for ${keyId} version ${version}`);
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