import { StorageError } from '../types.js';

/**
 * Storage utility functions for key management
 */

/**
 * Generate a unique key ID
 */
export function generateKeyId(prefix: string = 'key'): string {
  const timestamp = Date.now().toString(36);
  const randomPart = Math.random().toString(36).substring(2, 8);
  return `${prefix}_${timestamp}_${randomPart}`;
}

/**
 * Validate key ID format
 */
export function validateKeyId(keyId: string): { valid: boolean; reason?: string } {
  if (!keyId || typeof keyId !== 'string') {
    return { valid: false, reason: 'Key ID must be a non-empty string' };
  }

  if (keyId.trim().length === 0) {
    return { valid: false, reason: 'Key ID cannot be empty or only whitespace' };
  }

  if (keyId.length > 100) {
    return { valid: false, reason: 'Key ID must be 100 characters or less' };
  }

  // Check for unsafe characters that might cause issues with storage
  const unsafeChars = /[<>:"/\\|?*\x00-\x1f]/;
  if (unsafeChars.test(keyId)) {
    return { valid: false, reason: 'Key ID contains unsafe characters' };
  }

  return { valid: true };
}

/**
 * Sanitize key ID by removing or replacing unsafe characters
 */
export function sanitizeKeyId(keyId: string): string {
  if (!keyId || typeof keyId !== 'string') {
    throw new StorageError('Invalid key ID input', 'INVALID_KEY_ID');
  }

  // Replace unsafe characters with underscores
  let sanitized = keyId.replace(/[<>:"/\\|?*\x00-\x1f]/g, '_');
  
  // Trim whitespace
  sanitized = sanitized.trim();
  
  // Ensure not empty after sanitization
  if (sanitized.length === 0) {
    throw new StorageError('Key ID becomes empty after sanitization', 'EMPTY_KEY_ID');
  }
  
  // Truncate if too long
  if (sanitized.length > 100) {
    sanitized = sanitized.substring(0, 100);
  }
  
  return sanitized;
}

/**
 * Calculate storage key size estimation
 */
export function estimateKeyStorageSize(
  privateKeySize: number = 32,
  publicKeySize: number = 32,
  metadata: { name?: string; description?: string; tags?: string[] } = {}
): number {
  // Base overhead for encryption (IV, salt, algorithm overhead)
  const encryptionOverhead = 64; // 12-byte IV + 16-byte salt + 36 bytes for AES-GCM overhead
  
  // Metadata size estimation
  const metadataSize = JSON.stringify({
    name: metadata.name || '',
    description: metadata.description || '',
    created: new Date().toISOString(),
    lastAccessed: new Date().toISOString(),
    tags: metadata.tags || []
  }).length * 2; // UTF-16 encoding overhead
  
  // IndexedDB object overhead
  const dbOverhead = 100;
  
  return privateKeySize + encryptionOverhead + publicKeySize + metadataSize + dbOverhead;
}

/**
 * Convert bytes to human-readable format
 */
export function formatBytes(bytes: number, decimals: number = 2): string {
  if (bytes === 0) return '0 Bytes';

  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];

  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
}

/**
 * Check if running in secure context (required for WebCrypto)
 */
export function isSecureContext(): boolean {
  // Check if we're in a secure context
  if (typeof window !== 'undefined') {
    return window.isSecureContext || false;
  }
  
  // For non-browser environments, assume secure
  return true;
}

/**
 * Get browser storage persistence information
 */
export async function getStoragePersistence(): Promise<{
  persistent: boolean;
  canRequestPersistent: boolean;
}> {
  if (!navigator.storage) {
    return { persistent: false, canRequestPersistent: false };
  }

  try {
    const persistent = await navigator.storage.persisted();
    return {
      persistent,
      canRequestPersistent: 'persist' in navigator.storage
    };
  } catch {
    return { persistent: false, canRequestPersistent: false };
  }
}

/**
 * Request persistent storage (reduces chance of data eviction)
 */
export async function requestPersistentStorage(): Promise<boolean> {
  if (!navigator.storage?.persist) {
    return false;
  }

  try {
    return await navigator.storage.persist();
  } catch {
    return false;
  }
}

/**
 * Validate metadata object
 */
export function validateMetadata(metadata: any): { valid: boolean; issues: string[] } {
  const issues: string[] = [];
  let valid = true;

  if (metadata === null || metadata === undefined) {
    return { valid: true, issues: [] }; // Metadata is optional
  }

  if (typeof metadata !== 'object' || Array.isArray(metadata)) {
    return { valid: false, issues: ['Metadata must be an object'] };
  }

  // Validate name
  if (metadata.name !== undefined) {
    if (typeof metadata.name !== 'string') {
      issues.push('Name must be a string');
      valid = false;
    } else if (metadata.name.length > 100) {
      issues.push('Name must be 100 characters or less');
      valid = false;
    }
  }

  // Validate description
  if (metadata.description !== undefined) {
    if (typeof metadata.description !== 'string') {
      issues.push('Description must be a string');
      valid = false;
    } else if (metadata.description.length > 500) {
      issues.push('Description must be 500 characters or less');
      valid = false;
    }
  }

  // Validate tags
  if (metadata.tags !== undefined) {
    if (!Array.isArray(metadata.tags)) {
      issues.push('Tags must be an array');
      valid = false;
    } else {
      if (metadata.tags.length > 20) {
        issues.push('Maximum 20 tags allowed');
        valid = false;
      }
      
      for (const tag of metadata.tags) {
        if (typeof tag !== 'string') {
          issues.push('All tags must be strings');
          valid = false;
          break;
        }
        if (tag.length > 50) {
          issues.push('Each tag must be 50 characters or less');
          valid = false;
          break;
        }
      }
    }
  }

  return { valid, issues };
}

/**
 * Create a storage operation timeout wrapper
 */
export function withTimeout<T>(
  promise: Promise<T>, 
  timeoutMs: number = 10000,
  operation: string = 'Storage operation'
): Promise<T> {
  return new Promise((resolve, reject) => {
    const timeoutId = setTimeout(() => {
      reject(new StorageError(`${operation} timed out after ${timeoutMs}ms`, 'OPERATION_TIMEOUT'));
    }, timeoutMs);

    promise
      .then(result => {
        clearTimeout(timeoutId);
        resolve(result);
      })
      .catch(error => {
        clearTimeout(timeoutId);
        reject(error);
      });
  });
}