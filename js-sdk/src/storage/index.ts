/**
 * DataFold Storage Module
 * Secure client-side key storage with encryption
 */

// Export storage implementations
export { IndexedDBKeyStorage } from './indexeddb-storage.js';

// Export storage utilities
export * from './storage-utils.js';

// Re-export storage-related types
export type { 
  StorageError, 
  StoredKeyMetadata, 
  StorageOptions, 
  KeyStorageInterface 
} from '../types.js';

/**
 * Create a new IndexedDB storage instance with default options
 */
export async function createStorage(options?: import('../types.js').StorageOptions): Promise<import('./indexeddb-storage.js').IndexedDBKeyStorage> {
  const { IndexedDBKeyStorage } = await import('./indexeddb-storage.js');
  return new IndexedDBKeyStorage(options);
}

/**
 * Check if secure storage is supported in the current environment
 */
export function isStorageSupported(): { supported: boolean; reasons: string[] } {
  const reasons: string[] = [];
  let supported = true;

  // Check IndexedDB support
  if (typeof window === 'undefined') {
    reasons.push('Not running in browser environment');
    supported = false;
  } else if (!window.indexedDB) {
    reasons.push('IndexedDB not supported');
    supported = false;
  }

  // Check WebCrypto support
  if (typeof crypto === 'undefined' || !crypto.subtle) {
    reasons.push('WebCrypto API not supported');
    supported = false;
  }

  // Check required crypto algorithms
  if (supported) {
    try {
      // Test if required algorithms are available
      if (!crypto.subtle.importKey || !crypto.subtle.deriveKey || !crypto.subtle.encrypt) {
        reasons.push('Required cryptographic functions not available');
        supported = false;
      }
    } catch {
      reasons.push('WebCrypto API partially supported');
      supported = false;
    }
  }

  return { supported, reasons };
}

/**
 * Get storage quota information for the current origin
 */
export async function getStorageQuota(): Promise<{
  used: number;
  available: number | null;
  percentage: number | null;
}> {
  if (!navigator.storage?.estimate) {
    return { used: 0, available: null, percentage: null };
  }

  try {
    const estimate = await navigator.storage.estimate();
    const used = estimate.usage || 0;
    const available = estimate.quota || null;
    const percentage = available ? Math.round((used / available) * 100) : null;

    return { used, available, percentage };
  } catch {
    return { used: 0, available: null, percentage: null };
  }
}

/**
 * Validate passphrase strength
 */
export function validatePassphrase(passphrase: string): { valid: boolean; issues: string[] } {
  const issues: string[] = [];
  let valid = true;

  if (passphrase.length < 8) {
    issues.push('Must be at least 8 characters long');
    valid = false;
  }

  if (passphrase.length < 12) {
    issues.push('Recommended minimum 12 characters for better security');
  }

  if (!/[a-z]/.test(passphrase)) {
    issues.push('Should contain lowercase letters');
  }

  if (!/[A-Z]/.test(passphrase)) {
    issues.push('Should contain uppercase letters');
  }

  if (!/[0-9]/.test(passphrase)) {
    issues.push('Should contain numbers');
  }

  if (!/[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>\/?]/.test(passphrase)) {
    issues.push('Should contain special characters');
  }

  // Check for common weak patterns
  const commonWeak = ['password', '123456', 'qwerty', 'abc123', 'password123'];
  if (commonWeak.some(weak => passphrase.toLowerCase().includes(weak))) {
    issues.push('Contains common weak patterns');
    valid = false;
  }

  return { valid, issues };
}