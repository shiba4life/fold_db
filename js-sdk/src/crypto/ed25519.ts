import * as ed25519 from '@noble/ed25519';
import { Ed25519KeyPair, KeyGenerationOptions, Ed25519KeyError, BrowserCompatibility } from '../types.js';

/**
 * Constants for Ed25519 key operations
 */
const ED25519_PRIVATE_KEY_LENGTH = 32;
const ED25519_PUBLIC_KEY_LENGTH = 32;
const ED25519_SIGNATURE_LENGTH = 64;

/**
 * Check browser compatibility for Ed25519 operations
 */
export function checkBrowserCompatibility(): BrowserCompatibility {
  const webCrypto = typeof crypto !== 'undefined' && typeof crypto.subtle !== 'undefined';
  const secureRandom = typeof crypto !== 'undefined' && typeof crypto.getRandomValues !== 'undefined';
  
  // Check for native Ed25519 support (limited browser support as of 2025)
  let nativeEd25519 = false;
  try {
    // This will throw if Ed25519 is not supported
    if (webCrypto) {
      nativeEd25519 = 'Ed25519' in crypto.subtle.constructor.prototype;
    }
  } catch {
    nativeEd25519 = false;
  }

  const browserInfo = typeof navigator !== 'undefined' ? navigator.userAgent : 'Unknown';

  return {
    webCrypto,
    secureRandom,
    nativeEd25519,
    browserInfo
  };
}

/**
 * Generate secure random bytes using the browser's crypto API
 */
function generateSecureRandom(length: number): Uint8Array {
  const compatibility = checkBrowserCompatibility();
  
  if (!compatibility.secureRandom) {
    throw new Ed25519KeyError(
      'Secure random number generation not supported in this environment',
      'UNSUPPORTED_RANDOM'
    );
  }

  const randomBytes = new Uint8Array(length);
  crypto.getRandomValues(randomBytes);
  return randomBytes;
}

/**
 * Validate Ed25519 private key
 */
function validatePrivateKey(privateKey: Uint8Array): void {
  if (!(privateKey instanceof Uint8Array)) {
    throw new Ed25519KeyError('Private key must be a Uint8Array', 'INVALID_PRIVATE_KEY_TYPE');
  }

  if (privateKey.length !== ED25519_PRIVATE_KEY_LENGTH) {
    throw new Ed25519KeyError(
      `Private key must be exactly ${ED25519_PRIVATE_KEY_LENGTH} bytes`,
      'INVALID_PRIVATE_KEY_LENGTH'
    );
  }

  // Check for all-zero key (invalid)
  const isAllZero = privateKey.every(byte => byte === 0);
  if (isAllZero) {
    throw new Ed25519KeyError('Private key cannot be all zeros', 'INVALID_PRIVATE_KEY_VALUE');
  }
}

/**
 * Validate Ed25519 public key
 */
function validatePublicKey(publicKey: Uint8Array): void {
  if (!(publicKey instanceof Uint8Array)) {
    throw new Ed25519KeyError('Public key must be a Uint8Array', 'INVALID_PUBLIC_KEY_TYPE');
  }

  if (publicKey.length !== ED25519_PUBLIC_KEY_LENGTH) {
    throw new Ed25519KeyError(
      `Public key must be exactly ${ED25519_PUBLIC_KEY_LENGTH} bytes`,
      'INVALID_PUBLIC_KEY_LENGTH'
    );
  }
}

/**
 * Generate an Ed25519 key pair
 * 
 * @param options - Key generation options
 * @returns Promise resolving to the generated key pair
 * @throws Ed25519KeyError if generation fails or validation fails
 */
export async function generateKeyPair(options: KeyGenerationOptions = {}): Promise<Ed25519KeyPair> {
  const { validate = true, entropy } = options;

  try {
    let privateKey: Uint8Array;

    if (entropy) {
      // Use provided entropy (mainly for testing)
      if (entropy.length !== ED25519_PRIVATE_KEY_LENGTH) {
        throw new Ed25519KeyError(
          `Entropy must be exactly ${ED25519_PRIVATE_KEY_LENGTH} bytes`,
          'INVALID_ENTROPY_LENGTH'
        );
      }
      privateKey = entropy.slice(); // Create a copy
    } else {
      // Generate secure random private key
      privateKey = generateSecureRandom(ED25519_PRIVATE_KEY_LENGTH);
    }

    // Generate public key from private key using @noble/ed25519
    const publicKey = await ed25519.getPublicKeyAsync(privateKey);

    // Validate keys if requested
    if (validate) {
      validatePrivateKey(privateKey);
      validatePublicKey(publicKey);

      // Additional validation: verify key pair consistency
      const testMessage = new Uint8Array([1, 2, 3, 4, 5]);
      const signature = await ed25519.signAsync(testMessage, privateKey);
      const isValid = await ed25519.verifyAsync(signature, testMessage, publicKey);
      
      if (!isValid) {
        throw new Ed25519KeyError(
          'Generated key pair failed consistency validation',
          'KEY_PAIR_INCONSISTENT'
        );
      }
    }

    return {
      privateKey,
      publicKey
    };

  } catch (error) {
    if (error instanceof Ed25519KeyError) {
      throw error;
    }
    
    // Wrap any other errors
    throw new Ed25519KeyError(
      `Key generation failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
      'GENERATION_FAILED'
    );
  }
}

/**
 * Generate multiple key pairs efficiently
 * 
 * @param count - Number of key pairs to generate
 * @param options - Key generation options
 * @returns Promise resolving to array of generated key pairs
 */
export async function generateMultipleKeyPairs(
  count: number, 
  options: KeyGenerationOptions = {}
): Promise<Ed25519KeyPair[]> {
  if (!Number.isInteger(count) || count <= 0) {
    throw new Ed25519KeyError('Count must be a positive integer', 'INVALID_COUNT');
  }

  if (count > 100) {
    throw new Ed25519KeyError('Cannot generate more than 100 key pairs at once', 'COUNT_TOO_LARGE');
  }

  const keyPairs: Ed25519KeyPair[] = [];
  
  for (let i = 0; i < count; i++) {
    const keyPair = await generateKeyPair(options);
    keyPairs.push(keyPair);
  }

  return keyPairs;
}

/**
 * Convert key to different formats
 */
export function formatKey(key: Uint8Array, format: 'hex' | 'base64' | 'uint8array'): string | Uint8Array {
  switch (format) {
    case 'hex':
      return Array.from(key, byte => byte.toString(16).padStart(2, '0')).join('');
    case 'base64':
      // Convert to base64
      const binary = String.fromCharCode(...key);
      return btoa(binary);
    case 'uint8array':
      return key.slice(); // Return a copy
    default:
      throw new Ed25519KeyError(`Unsupported format: ${format}`, 'UNSUPPORTED_FORMAT');
  }
}

/**
 * Parse key from different formats
 */
export function parseKey(keyData: string | Uint8Array, format: 'hex' | 'base64' | 'uint8array'): Uint8Array {
  switch (format) {
    case 'hex':
      if (typeof keyData !== 'string') {
        throw new Ed25519KeyError('Hex format requires string input', 'INVALID_HEX_INPUT');
      }
      if (keyData.length % 2 !== 0) {
        throw new Ed25519KeyError('Hex string must have even length', 'INVALID_HEX_LENGTH');
      }
      const hexBytes = [];
      for (let i = 0; i < keyData.length; i += 2) {
        const byte = parseInt(keyData.substr(i, 2), 16);
        if (isNaN(byte)) {
          throw new Ed25519KeyError('Invalid hex character', 'INVALID_HEX_CHARACTER');
        }
        hexBytes.push(byte);
      }
      return new Uint8Array(hexBytes);
      
    case 'base64':
      if (typeof keyData !== 'string') {
        throw new Ed25519KeyError('Base64 format requires string input', 'INVALID_BASE64_INPUT');
      }
      try {
        const binary = atob(keyData);
        return new Uint8Array(binary.split('').map(char => char.charCodeAt(0)));
      } catch {
        throw new Ed25519KeyError('Invalid base64 string', 'INVALID_BASE64');
      }
      
    case 'uint8array':
      if (!(keyData instanceof Uint8Array)) {
        throw new Ed25519KeyError('Uint8Array format requires Uint8Array input', 'INVALID_UINT8ARRAY_INPUT');
      }
      return keyData.slice(); // Return a copy
      
    default:
      throw new Ed25519KeyError(`Unsupported format: ${format}`, 'UNSUPPORTED_FORMAT');
  }
}

/**
 * Clear sensitive key material from memory (best effort)
 * Note: JavaScript doesn't provide guaranteed memory clearing, but this helps
 */
export function clearKeyMaterial(keyPair: Ed25519KeyPair): void {
  // Overwrite with random data, then zeros
  if (keyPair.privateKey) {
    crypto.getRandomValues(keyPair.privateKey);
    keyPair.privateKey.fill(0);
  }
  
  // Public keys are not sensitive, but clear for consistency
  if (keyPair.publicKey) {
    keyPair.publicKey.fill(0);
  }
}