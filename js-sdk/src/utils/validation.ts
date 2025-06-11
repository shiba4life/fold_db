import { Ed25519KeyError } from '../types.js';

/**
 * Constants for validation
 */
const ED25519_PRIVATE_KEY_LENGTH = 32;
const ED25519_PUBLIC_KEY_LENGTH = 32;
const ED25519_SIGNATURE_LENGTH = 64;

/**
 * Validate environment security
 */
export function validateEnvironment(): { secure: boolean; issues: string[] } {
  const issues: string[] = [];
  let secure = true;

  // Check for HTTPS in browser environment
  if (typeof window !== 'undefined' && window.location) {
    if (window.location.protocol !== 'https:' && window.location.hostname !== 'localhost') {
      issues.push('HTTPS required for secure key operations in production');
      secure = false;
    }
  }

  // Check for secure context
  if (typeof window !== 'undefined' && typeof window.isSecureContext !== 'undefined') {
    if (!window.isSecureContext) {
      issues.push('Secure context required for cryptographic operations');
      secure = false;
    }
  }

  // Check WebCrypto availability
  if (typeof crypto === 'undefined' || typeof crypto.subtle === 'undefined') {
    issues.push('WebCrypto API not available');
    secure = false;
  }

  // Check secure random
  if (typeof crypto === 'undefined' || typeof crypto.getRandomValues === 'undefined') {
    issues.push('Secure random number generation not available');
    secure = false;
  }

  return { secure, issues };
}

/**
 * Validate key length and format
 */
export function validateKeyLength(key: Uint8Array, expectedLength: number, keyType: string): void {
  if (!(key instanceof Uint8Array)) {
    throw new Ed25519KeyError(`${keyType} must be a Uint8Array`, 'INVALID_KEY_TYPE');
  }

  if (key.length !== expectedLength) {
    throw new Ed25519KeyError(
      `${keyType} must be exactly ${expectedLength} bytes, got ${key.length}`,
      'INVALID_KEY_LENGTH'
    );
  }
}

/**
 * Validate Ed25519 private key
 */
export function validatePrivateKey(privateKey: Uint8Array): void {
  validateKeyLength(privateKey, ED25519_PRIVATE_KEY_LENGTH, 'Private key');

  // Check for all-zero key (cryptographically invalid)
  const isAllZero = privateKey.every(byte => byte === 0);
  if (isAllZero) {
    throw new Ed25519KeyError('Private key cannot be all zeros', 'INVALID_PRIVATE_KEY_VALUE');
  }

  // Check for all-one key (also invalid)
  const isAllOne = privateKey.every(byte => byte === 255);
  if (isAllOne) {
    throw new Ed25519KeyError('Private key cannot be all ones', 'INVALID_PRIVATE_KEY_VALUE');
  }
}

/**
 * Validate Ed25519 public key
 */
export function validatePublicKey(publicKey: Uint8Array): void {
  validateKeyLength(publicKey, ED25519_PUBLIC_KEY_LENGTH, 'Public key');

  // Additional Ed25519 public key validation could be added here
  // For now, we rely on the library's validation during operations
}

/**
 * Validate Ed25519 signature
 */
export function validateSignature(signature: Uint8Array): void {
  validateKeyLength(signature, ED25519_SIGNATURE_LENGTH, 'Signature');
}

/**
 * Validate input parameters for key generation
 */
export function validateKeyGenerationParams(options: {
  entropy?: Uint8Array;
  validate?: boolean;
}): void {
  if (options.entropy !== undefined) {
    validateKeyLength(options.entropy, ED25519_PRIVATE_KEY_LENGTH, 'Entropy');
  }

  if (options.validate !== undefined && typeof options.validate !== 'boolean') {
    throw new Ed25519KeyError('Validate option must be a boolean', 'INVALID_VALIDATE_OPTION');
  }
}

/**
 * Validate hex string format
 */
export function validateHexString(hex: string): void {
  if (typeof hex !== 'string') {
    throw new Ed25519KeyError('Hex input must be a string', 'INVALID_HEX_TYPE');
  }

  if (hex.length % 2 !== 0) {
    throw new Ed25519KeyError('Hex string must have even length', 'INVALID_HEX_LENGTH');
  }

  if (!/^[0-9a-fA-F]*$/.test(hex)) {
    throw new Ed25519KeyError('Hex string contains invalid characters', 'INVALID_HEX_CHARACTERS');
  }
}

/**
 * Validate base64 string format
 */
export function validateBase64String(base64: string): void {
  if (typeof base64 !== 'string') {
    throw new Ed25519KeyError('Base64 input must be a string', 'INVALID_BASE64_TYPE');
  }

  // Basic base64 pattern check
  if (!/^[A-Za-z0-9+/]*={0,2}$/.test(base64)) {
    throw new Ed25519KeyError('Invalid base64 string format', 'INVALID_BASE64_FORMAT');
  }

  // Check length is valid for base64 (allow missing padding)
  const paddedLength = base64.length + (4 - (base64.length % 4)) % 4;
  if (paddedLength !== base64.length && base64.length % 4 !== 0) {
    // Only throw if it's genuinely invalid, not just missing padding
    const testString = base64 + '='.repeat((4 - (base64.length % 4)) % 4);
    try {
      atob(testString);
    } catch {
      throw new Ed25519KeyError('Invalid base64 string format', 'INVALID_BASE64_FORMAT');
    }
  }
}

/**
 * Validate message for signing
 */
export function validateMessage(message: Uint8Array): void {
  if (!(message instanceof Uint8Array)) {
    throw new Ed25519KeyError('Message must be a Uint8Array', 'INVALID_MESSAGE_TYPE');
  }

  if (message.length === 0) {
    throw new Ed25519KeyError('Message cannot be empty', 'EMPTY_MESSAGE');
  }

  // Ed25519 can handle messages of any length, so no upper limit check needed
}

/**
 * Sanitize and validate count parameter
 */
export function validateCount(count: unknown): number {
  if (typeof count !== 'number') {
    throw new Ed25519KeyError('Count must be a number', 'INVALID_COUNT_TYPE');
  }

  if (!Number.isInteger(count)) {
    throw new Ed25519KeyError('Count must be an integer', 'INVALID_COUNT_INTEGER');
  }

  if (count <= 0) {
    throw new Ed25519KeyError('Count must be positive', 'INVALID_COUNT_NEGATIVE');
  }

  if (count > 1000) {
    throw new Ed25519KeyError('Count cannot exceed 1000', 'COUNT_TOO_LARGE');
  }

  return count;
}

/**
 * Check if running in a secure environment for key operations
 */
export function requireSecureEnvironment(): void {
  const { secure, issues } = validateEnvironment();
  
  if (!secure) {
    throw new Ed25519KeyError(
      `Insecure environment detected: ${issues.join(', ')}`,
      'INSECURE_ENVIRONMENT'
    );
  }
}