/**
 * Key backup and recovery utilities for PBI-12 Key Rotation
 * Provides backup verification and recovery flows for failed rotations
 */

import { 
  Ed25519KeyPair, 
  KeyStorageInterface, 
  EncryptedBackupFormat,
  KeyBackupData,
  KeyExportResult,
  KeyImportResult,
  StoredKeyMetadata
} from '../types.js';
import {
  BackupVerificationResult,
  BackupVerificationError,
  RotationRecoveryOptions,
  RotationRecoveryResult,
  AtomicRotationState
} from '../types/key-rotation.js';
import { exportKey, importKey } from './key-export-import.js';
import { generateKeyPair } from './ed25519.js';

/**
 * Key backup manager for rotation safety
 */
export class KeyBackupManager {
  constructor(private storage: KeyStorageInterface) {}

  /**
   * Create a backup before key rotation
   */
  async createRotationBackup(
    keyId: string,
    keyPair: Ed25519KeyPair,
    passphrase: string,
    metadata: Partial<StoredKeyMetadata> = {}
  ): Promise<string> {
    try {
      // Export the key pair to encrypted backup format
      const exportResult = await exportKey(
        this.storage,
        keyId,
        passphrase,
        {
          format: 'json',
          includeMetadata: true,
          kdfIterations: 150000, // Higher iterations for rotation backups
          additionalData: {
            backupType: 'rotation',
            created: new Date().toISOString(),
            sdkVersion: '0.1.0'
          }
        }
      );

      // Store backup with special backup key ID
      const backupKeyId = `backup_${keyId}_${Date.now()}`;
      const backupMetadata: StoredKeyMetadata = {
        name: `Backup for ${metadata.name || keyId}`,
        description: `Pre-rotation backup created at ${new Date().toISOString()}`,
        created: new Date().toISOString(),
        lastAccessed: new Date().toISOString(),
        tags: ['backup', 'rotation', ...(metadata.tags || [])]
      };

      // Store the backup data as a special backup entry
      await this.storeBackupData(backupKeyId, exportResult.data as string, backupMetadata);

      return backupKeyId;
    } catch (error) {
      throw new BackupVerificationError(
        `Failed to create rotation backup: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'BACKUP_CREATION_FAILED'
      );
    }
  }

  /**
   * Verify a backup can recover the key
   */
  async verifyBackup(
    backupKeyId: string,
    passphrase: string,
    originalKeyPair?: Ed25519KeyPair
  ): Promise<BackupVerificationResult> {
    const issues: string[] = [];
    let verified = false;
    let recoverable = false;
    let format: string | undefined;
    let created: string | undefined;

    try {
      // Retrieve backup data
      const backupData = await this.getBackupData(backupKeyId);
      if (!backupData) {
        issues.push('Backup not found');
        return { verified: false, issues, recoverable: false };
      }

      // Parse backup format
      try {
        const parsedBackup = JSON.parse(backupData) as EncryptedBackupFormat;
        format = parsedBackup.type;
        created = parsedBackup.created;

        // Validate backup structure
        if (parsedBackup.type !== 'datafold-key-backup') {
          issues.push(`Invalid backup type: ${parsedBackup.type}`);
        }

        if (!parsedBackup.ciphertext || !parsedBackup.nonce || !parsedBackup.kdf_params) {
          issues.push('Backup missing required encryption components');
        }

        // Attempt to decrypt and import the backup
        const tempKeyId = `temp_verify_${Date.now()}`;
        const importResult = await importKey(
          this.storage,
          backupData,
          passphrase,
          {
            validateIntegrity: true,
            overwriteExisting: false
          }
        );

        recoverable = importResult.integrityValid;

        if (recoverable) {
          // If we have the original key pair, verify they match
          if (originalKeyPair) {
            const restoredKeyPair = await this.storage.retrieveKeyPair(tempKeyId, passphrase);
            const privateKeysMatch = this.compareUint8Arrays(
              originalKeyPair.privateKey,
              restoredKeyPair.privateKey
            );
            const publicKeysMatch = this.compareUint8Arrays(
              originalKeyPair.publicKey,
              restoredKeyPair.publicKey
            );

            if (!privateKeysMatch || !publicKeysMatch) {
              issues.push('Restored key pair does not match original');
              recoverable = false;
            }
          }

          // Clean up temp key
          await this.storage.deleteKeyPair(tempKeyId);
        }

        verified = issues.length === 0 && recoverable;

      } catch (parseError) {
        issues.push(`Failed to parse backup: ${parseError instanceof Error ? parseError.message : 'Unknown error'}`);
      }

    } catch (error) {
      issues.push(`Backup verification failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }

    return {
      verified,
      issues,
      format,
      created,
      recoverable
    };
  }

  /**
   * Recover from a failed rotation using backup
   */
  async recoverFromFailedRotation(
    rotationState: AtomicRotationState,
    options: RotationRecoveryOptions
  ): Promise<RotationRecoveryResult> {
    try {
      const { correlationId, rollback = true, retry = false, metadata = {} } = options;

      if (rotationState.operationId !== correlationId) {
        throw new Error(`Correlation ID mismatch: expected ${rotationState.operationId}, got ${correlationId}`);
      }

      // Check if we have backup data
      if (!rotationState.backupData && rollback) {
        return {
          success: false,
          action: 'manual_intervention_required',
          currentKeyState: {
            keyId: 'unknown',
            version: 0,
            active: false
          },
          details: {
            error: 'No backup data available for rollback',
            phase: rotationState.phase
          }
        };
      }

      let recoveryAction: 'rollback' | 'retry' | 'manual_intervention_required';
      let success = false;
      let currentKeyState = {
        keyId: 'unknown',
        version: 0,
        active: false
      };

      if (rollback && rotationState.backupData) {
        // Attempt rollback using backup
        try {
          await importKey(
            this.storage,
            rotationState.backupData,
            'recovery_passphrase', // This should be derived from the original
            {
              validateIntegrity: true,
              overwriteExisting: true,
              customMetadata: {
                ...metadata,
                description: `Recovery operation ${options.correlationId}`,
                tags: [...(metadata.tags || []), 'recovery', options.correlationId]
              }
            }
          );

          recoveryAction = 'rollback';
          success = true;
          currentKeyState = {
            keyId: 'recovered_key',
            version: 1, // Restored as new version
            active: true
          };

        } catch (rollbackError) {
          recoveryAction = 'manual_intervention_required';
          success = false;
        }

      } else if (retry) {
        // For retry, we would need to re-attempt the rotation
        // This is a simplified implementation
        recoveryAction = 'retry';
        success = false; // Would need actual retry logic
        
      } else {
        recoveryAction = 'manual_intervention_required';
        success = false;
      }

      return {
        success,
        action: recoveryAction,
        currentKeyState,
        details: {
          originalPhase: rotationState.phase,
          recoveryTimestamp: new Date().toISOString(),
          metadata
        }
      };

    } catch (error) {
      return {
        success: false,
        action: 'manual_intervention_required',
        currentKeyState: {
          keyId: 'unknown',
          version: 0,
          active: false
        },
        details: {
          error: error instanceof Error ? error.message : 'Unknown recovery error',
          correlationId: options.correlationId
        }
      };
    }
  }

  /**
   * Validate rotation state for recovery
   */
  async validateRotationState(
    rotationState: AtomicRotationState
  ): Promise<{ valid: boolean; issues: string[] }> {
    const issues: string[] = [];

    // Check required fields
    if (!rotationState.operationId) {
      issues.push('Missing operation ID');
    }

    if (!rotationState.oldKeyPair || !rotationState.newKeyPair) {
      issues.push('Missing key pairs in rotation state');
    }

    if (!rotationState.options) {
      issues.push('Missing rotation options');
    }

    // Check phase validity
    const validPhases = ['preparing', 'submitted', 'confirmed', 'completed', 'failed', 'rolled_back'];
    if (!validPhases.includes(rotationState.phase)) {
      issues.push(`Invalid rotation phase: ${rotationState.phase}`);
    }

    // Check backup data if needed
    if (rotationState.options?.verifyBackup && !rotationState.backupData) {
      issues.push('Backup verification requested but no backup data found');
    }

    // Validate key pairs
    try {
      if (rotationState.oldKeyPair) {
        if (rotationState.oldKeyPair.privateKey.length !== 32) {
          issues.push('Invalid old private key length');
        }
        if (rotationState.oldKeyPair.publicKey.length !== 32) {
          issues.push('Invalid old public key length');
        }
      }

      if (rotationState.newKeyPair) {
        if (rotationState.newKeyPair.privateKey.length !== 32) {
          issues.push('Invalid new private key length');
        }
        if (rotationState.newKeyPair.publicKey.length !== 32) {
          issues.push('Invalid new public key length');
        }
      }
    } catch (error) {
      issues.push(`Key pair validation failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }

    return {
      valid: issues.length === 0,
      issues
    };
  }

  /**
   * Clean up old backup data
   */
  async cleanupOldBackups(olderThanDays: number = 30): Promise<number> {
    try {
      const allKeys = await this.storage.listKeys();
      const cutoffDate = new Date();
      cutoffDate.setDate(cutoffDate.getDate() - olderThanDays);

      let deletedCount = 0;

      for (const { id, metadata } of allKeys) {
        // Check if this is a backup entry
        if (id.startsWith('backup_') && metadata.tags.includes('backup')) {
          const createdDate = new Date(metadata.created);
          if (createdDate < cutoffDate) {
            await this.storage.deleteKeyPair(id);
            deletedCount++;
          }
        }
      }

      return deletedCount;
    } catch (error) {
      throw new BackupVerificationError(
        `Failed to clean up old backups: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'CLEANUP_FAILED'
      );
    }
  }

  /**
   * Store backup data (private method)
   */
  private async storeBackupData(
    backupKeyId: string,
    backupData: string,
    metadata: StoredKeyMetadata
  ): Promise<void> {
    // Create a dummy key pair to store the backup data
    // We encode the backup data as the private key
    const encodedBackup = new TextEncoder().encode(backupData);
    const dummyKeyPair: Ed25519KeyPair = {
      privateKey: encodedBackup,
      publicKey: new Uint8Array(32) // Dummy public key
    };

    await this.storage.storeKeyPair(
      backupKeyId,
      dummyKeyPair,
      'backup_storage', // Fixed passphrase for backup storage
      metadata
    );
  }

  /**
   * Get backup data (private method)
   */
  private async getBackupData(backupKeyId: string): Promise<string | null> {
    try {
      const keyPair = await this.storage.retrieveKeyPair(backupKeyId, 'backup_storage');
      return new TextDecoder().decode(keyPair.privateKey);
    } catch {
      return null;
    }
  }

  /**
   * Compare two Uint8Arrays for equality
   */
  private compareUint8Arrays(a: Uint8Array, b: Uint8Array): boolean {
    if (a.length !== b.length) {
      return false;
    }
    for (let i = 0; i < a.length; i++) {
      if (a[i] !== b[i]) {
        return false;
      }
    }
    return true;
  }
}

/**
 * Create a key backup manager instance
 */
export function createKeyBackupManager(storage: KeyStorageInterface): KeyBackupManager {
  return new KeyBackupManager(storage);
}

/**
 * Quick backup verification utility
 */
export async function quickVerifyBackup(
  backupData: string,
  passphrase: string
): Promise<boolean> {
  try {
    // Create a temporary in-memory storage for verification
    const tempStorage = new Map<string, any>();
    const mockStorage: KeyStorageInterface = {
      async storeKeyPair(keyId: string, keyPair: Ed25519KeyPair, passphrase: string, metadata?: Partial<StoredKeyMetadata>): Promise<void> {
        tempStorage.set(keyId, { keyPair, metadata });
      },
      async retrieveKeyPair(keyId: string, passphrase: string): Promise<Ed25519KeyPair> {
        const stored = tempStorage.get(keyId);
        return stored?.keyPair || { privateKey: new Uint8Array(32), publicKey: new Uint8Array(32) };
      },
      async deleteKeyPair(keyId: string): Promise<void> {
        tempStorage.delete(keyId);
      },
      async listKeys(): Promise<Array<{ id: string; metadata: StoredKeyMetadata }>> {
        return [];
      },
      async keyExists(keyId: string): Promise<boolean> {
        return tempStorage.has(keyId);
      },
      async getStorageInfo(): Promise<{ used: number; available: number | null }> {
        return { used: tempStorage.size, available: null };
      },
      async clearAllKeys(): Promise<void> {
        tempStorage.clear();
      },
      async close(): Promise<void> {
        // No-op for mock storage
      }
    };

    await importKey(
      mockStorage,
      backupData,
      passphrase,
      { validateIntegrity: true }
    );
    return true;
  } catch {
    return false;
  }
}

/**
 * Emergency recovery utility
 */
export async function emergencyRecovery(
  backupData: string,
  recoveryPassphrase: string,
  storage: KeyStorageInterface,
  newKeyId?: string
): Promise<{ success: boolean; keyId?: string; error?: string }> {
  try {
    const keyId = newKeyId || `emergency_recovery_${Date.now()}`;
    
    const importResult = await importKey(
      storage,
      backupData,
      recoveryPassphrase,
      {
        validateIntegrity: true,
        overwriteExisting: true,
        customMetadata: {
          name: 'Emergency Recovery Key',
          description: 'Key recovered from emergency backup',
          tags: ['emergency', 'recovered']
        }
      }
    );

    return {
      success: importResult.integrityValid,
      keyId: importResult.keyId
    };
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown recovery error'
    };
  }
}