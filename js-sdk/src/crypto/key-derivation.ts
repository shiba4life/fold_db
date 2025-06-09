/**
 * Key derivation utilities for the DataFold JavaScript SDK
 * Supports HKDF and PBKDF2 algorithms for deriving keys from master keys
 */

import { 
  KeyDerivationOptions, 
  DerivedKeyInfo, 
  KeyDerivationError, 
  Ed25519KeyPair 
} from '../types.js';

/**
 * Default configuration for key derivation
 */
const DEFAULT_DERIVATION_CONFIG = {
  algorithm: 'HKDF' as const,
  hash: 'SHA-256' as const,
  length: 32,
  iterations: 100000 // For PBKDF2
};

/**
 * Derive a key from master key material using HKDF or PBKDF2
 */
export async function deriveKey(
  masterKey: Uint8Array,
  options: KeyDerivationOptions
): Promise<DerivedKeyInfo> {
  if (!crypto.subtle) {
    throw new KeyDerivationError(
      'WebCrypto API not available',
      'WEBCRYPTO_NOT_AVAILABLE'
    );
  }

  // Validate master key
  if (!masterKey || masterKey.length === 0) {
    throw new KeyDerivationError(
      'Master key cannot be empty',
      'INVALID_MASTER_KEY'
    );
  }

  // Apply defaults
  const config = { ...DEFAULT_DERIVATION_CONFIG, ...options };

  // Validate algorithm-specific requirements
  if (config.algorithm === 'HKDF' && !config.info) {
    throw new KeyDerivationError(
      'HKDF requires info parameter',
      'HKDF_INFO_REQUIRED'
    );
  }

  // Generate salt if not provided
  const salt = config.salt || crypto.getRandomValues(new Uint8Array(32));

  let derivedKey: Uint8Array;
  let iterations: number | undefined;

  try {
    if (config.algorithm === 'HKDF') {
      derivedKey = await deriveKeyHKDF(masterKey, salt, config.info!, config.hash, config.length);
    } else if (config.algorithm === 'PBKDF2') {
      iterations = config.iterations || DEFAULT_DERIVATION_CONFIG.iterations;
      derivedKey = await deriveKeyPBKDF2(masterKey, salt, iterations, config.hash, config.length);
    } else {
      throw new KeyDerivationError(
        `Unsupported derivation algorithm: ${config.algorithm}`,
        'UNSUPPORTED_ALGORITHM'
      );
    }
  } catch (error) {
    if (error instanceof KeyDerivationError) {
      throw error;
    }
    throw new KeyDerivationError(
      `Key derivation failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
      'DERIVATION_FAILED'
    );
  }

  return {
    key: derivedKey,
    algorithm: config.algorithm,
    salt,
    info: config.info,
    iterations,
    hash: config.hash,
    derived: new Date().toISOString()
  };
}

/**
 * Derive key using HKDF (HMAC-based Key Derivation Function)
 */
async function deriveKeyHKDF(
  masterKey: Uint8Array,
  salt: Uint8Array,
  info: Uint8Array,
  hash: string,
  length: number
): Promise<Uint8Array> {
  // Import master key as key material
  const keyMaterial = await crypto.subtle.importKey(
    'raw',
    masterKey,
    { name: 'HKDF' },
    false,
    ['deriveKey']
  );

  // Derive key using HKDF
  const derivedCryptoKey = await crypto.subtle.deriveKey(
    {
      name: 'HKDF',
      hash,
      salt,
      info
    },
    keyMaterial,
    { name: 'AES-GCM', length: length * 8 }, // Convert bytes to bits
    true,
    ['encrypt', 'decrypt']
  );

  // Export derived key as raw bytes
  const derivedKeyBuffer = await crypto.subtle.exportKey('raw', derivedCryptoKey);
  return new Uint8Array(derivedKeyBuffer);
}

/**
 * Derive key using PBKDF2 (Password-Based Key Derivation Function 2)
 */
async function deriveKeyPBKDF2(
  masterKey: Uint8Array,
  salt: Uint8Array,
  iterations: number,
  hash: string,
  length: number
): Promise<Uint8Array> {
  // Import master key as key material
  const keyMaterial = await crypto.subtle.importKey(
    'raw',
    masterKey,
    { name: 'PBKDF2' },
    false,
    ['deriveKey']
  );

  // Derive key using PBKDF2
  const derivedCryptoKey = await crypto.subtle.deriveKey(
    {
      name: 'PBKDF2',
      salt,
      iterations,
      hash
    },
    keyMaterial,
    { name: 'AES-GCM', length: length * 8 }, // Convert bytes to bits
    true,
    ['encrypt', 'decrypt']
  );

  // Export derived key as raw bytes
  const derivedKeyBuffer = await crypto.subtle.exportKey('raw', derivedCryptoKey);
  return new Uint8Array(derivedKeyBuffer);
}

/**
 * Derive multiple keys from a master key with different info/context parameters
 */
export async function deriveMultipleKeys(
  masterKey: Uint8Array,
  contexts: Array<{ name: string; info: Uint8Array; options?: Partial<KeyDerivationOptions> }>
): Promise<Record<string, DerivedKeyInfo>> {
  const derivedKeys: Record<string, DerivedKeyInfo> = {};

  for (const context of contexts) {
    const options: KeyDerivationOptions = {
      algorithm: 'HKDF',
      hash: 'SHA-256',
      length: 32,
      info: context.info,
      ...context.options
    };

    derivedKeys[context.name] = await deriveKey(masterKey, options);
  }

  return derivedKeys;
}

/**
 * Derive a key from an Ed25519 key pair's private key
 */
export async function deriveKeyFromKeyPair(
  keyPair: Ed25519KeyPair,
  options: KeyDerivationOptions
): Promise<DerivedKeyInfo> {
  return deriveKey(keyPair.privateKey, options);
}

/**
 * Create standardized info parameter for HKDF from a string context
 */
export function createHKDFInfo(context: string, additionalData?: Uint8Array): Uint8Array {
  const contextBytes = new TextEncoder().encode(context);
  
  if (!additionalData) {
    return contextBytes;
  }

  // Combine context and additional data
  const combined = new Uint8Array(contextBytes.length + additionalData.length);
  combined.set(contextBytes, 0);
  combined.set(additionalData, contextBytes.length);
  
  return combined;
}

/**
 * Validate derived key integrity by re-deriving and comparing
 */
export async function validateDerivedKey(
  masterKey: Uint8Array,
  derivedKeyInfo: DerivedKeyInfo
): Promise<boolean> {
  try {
    const options: KeyDerivationOptions = {
      algorithm: derivedKeyInfo.algorithm,
      hash: derivedKeyInfo.hash as 'SHA-256' | 'SHA-384' | 'SHA-512',
      salt: derivedKeyInfo.salt,
      info: derivedKeyInfo.info,
      iterations: derivedKeyInfo.iterations,
      length: derivedKeyInfo.key.length
    };

    const reDerived = await deriveKey(masterKey, options);
    
    // Compare derived keys
    if (reDerived.key.length !== derivedKeyInfo.key.length) {
      return false;
    }

    for (let i = 0; i < reDerived.key.length; i++) {
      if (reDerived.key[i] !== derivedKeyInfo.key[i]) {
        return false;
      }
    }

    return true;
  } catch {
    return false;
  }
}

/**
 * Clear derived key material from memory (best effort)
 */
export function clearDerivedKey(derivedKeyInfo: DerivedKeyInfo): void {
  // Zero out the key material
  if (derivedKeyInfo.key) {
    derivedKeyInfo.key.fill(0);
  }

  // Zero out sensitive parameters
  if (derivedKeyInfo.salt) {
    derivedKeyInfo.salt.fill(0);
  }

  if (derivedKeyInfo.info) {
    derivedKeyInfo.info.fill(0);
  }
}

/**
 * Create a key derivation context for common use cases
 */
export const KeyDerivationContexts = {
  /** For encrypting stored data */
  DATA_ENCRYPTION: createHKDFInfo('datafold.data.encryption.v1'),
  
  /** For signing operations */
  SIGNING: createHKDFInfo('datafold.signing.v1'),
  
  /** For authentication tokens */
  AUTHENTICATION: createHKDFInfo('datafold.auth.v1'),
  
  /** For key wrapping operations */
  KEY_WRAPPING: createHKDFInfo('datafold.key.wrapping.v1'),
  
  /** For backup encryption */
  BACKUP_ENCRYPTION: createHKDFInfo('datafold.backup.encryption.v1')
} as const;