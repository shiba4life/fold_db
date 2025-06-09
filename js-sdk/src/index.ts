/**
 * DataFold JavaScript SDK
 * Client-side key management with Ed25519 support
 */

// Export core types
export * from './types.js';

// Export Ed25519 functionality
export * from './crypto/ed25519.js';

// Export utilities
export * from './utils/validation.js';

// Export storage functionality
export * from './storage/index.js';

// Export key derivation functionality
export * from './crypto/key-derivation.js';

// Export key rotation functionality
export * from './crypto/key-rotation.js';

// Export key export/import functionality
export * from './crypto/key-export-import.js';

// Export server integration functionality
export * from './server/index.js';

// Export version information
export const SDK_VERSION = '0.1.0';

/**
 * SDK initialization and compatibility check
 */
export async function initializeSDK(): Promise<{ compatible: boolean; warnings: string[] }> {
  const warnings: string[] = [];
  let compatible = true;

  // Check browser compatibility
  const { checkBrowserCompatibility } = await import('./crypto/ed25519.js');
  const compat = checkBrowserCompatibility();

  if (!compat.webCrypto) {
    warnings.push('WebCrypto API not available - some features may not work');
    compatible = false;
  }

  if (!compat.secureRandom) {
    warnings.push('Secure random generation not available - key generation will fail');
    compatible = false;
  }

  if (!compat.nativeEd25519) {
    warnings.push('Native Ed25519 not supported - using JavaScript implementation');
    // This is not a compatibility issue, just informational
  }

  return { compatible, warnings };
}

/**
 * Quick compatibility check (synchronous)
 */
export function isCompatible(): boolean {
  return typeof crypto !== 'undefined' && 
         typeof crypto.subtle !== 'undefined' && 
         typeof crypto.getRandomValues !== 'undefined';
}