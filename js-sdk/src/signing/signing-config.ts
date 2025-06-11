/**
 * Configuration management for request signing
 */

import { 
  SigningConfig, 
  SignatureComponents, 
  SignatureAlgorithm, 
  SigningError,
  DigestAlgorithm 
} from './types.js';
import {
  validateNonce,
  validateTimestamp,
  validateSigningPrivateKey,
  generateNonce,
  generateTimestamp
} from './utils.js';

/**
 * Default signature components following server requirements
 */
export const DEFAULT_SIGNATURE_COMPONENTS: SignatureComponents = {
  method: true,
  targetUri: true,
  headers: ['content-type'],
  contentDigest: true
};

/**
 * Strict signature components for high-security scenarios
 */
export const STRICT_SIGNATURE_COMPONENTS: SignatureComponents = {
  method: true,
  targetUri: true,
  headers: ['content-type', 'content-length', 'user-agent', 'authorization'],
  contentDigest: true
};

/**
 * Minimal signature components for basic signing
 */
export const MINIMAL_SIGNATURE_COMPONENTS: SignatureComponents = {
  method: true,
  targetUri: true,
  headers: [],
  contentDigest: false
};

/**
 * Security profiles for different use cases
 */
export interface SecurityProfile {
  name: string;
  description: string;
  components: SignatureComponents;
  digestAlgorithm: DigestAlgorithm;
  validateNonces: boolean;
  allowCustomNonces: boolean;
}

/**
 * Predefined security profiles
 */
export const SECURITY_PROFILES: Record<string, SecurityProfile> = {
  strict: {
    name: 'Strict',
    description: 'Maximum security with comprehensive signature coverage',
    components: STRICT_SIGNATURE_COMPONENTS,
    digestAlgorithm: 'sha-512',
    validateNonces: true,
    allowCustomNonces: false
  },
  
  standard: {
    name: 'Standard',
    description: 'Balanced security suitable for most applications',
    components: DEFAULT_SIGNATURE_COMPONENTS,
    digestAlgorithm: 'sha-256',
    validateNonces: true,
    allowCustomNonces: true
  },
  
  minimal: {
    name: 'Minimal',
    description: 'Basic signing for low-latency scenarios',
    components: MINIMAL_SIGNATURE_COMPONENTS,
    digestAlgorithm: 'sha-256',
    validateNonces: false,
    allowCustomNonces: true
  }
};

/**
 * Signing configuration builder
 */
export class SigningConfigBuilder {
  private config: Partial<SigningConfig> = {};

  /**
   * Set the signature algorithm
   */
  algorithm(alg: SignatureAlgorithm): this {
    this.config.algorithm = alg;
    return this;
  }

  /**
   * Set the key ID
   */
  keyId(id: string): this {
    if (!id || typeof id !== 'string') {
      throw new SigningError('Key ID must be a non-empty string', 'INVALID_KEY_ID');
    }
    this.config.keyId = id;
    return this;
  }

  /**
   * Set the private key
   */
  privateKey(key: Uint8Array): this {
    if (!validateSigningPrivateKey(key)) {
      throw new SigningError('Invalid private key format', 'INVALID_PRIVATE_KEY');
    }
    this.config.privateKey = key.slice(); // Create a copy
    return this;
  }

  /**
   * Set signature components
   */
  components(comp: SignatureComponents): this {
    this.config.components = { ...comp };
    return this;
  }

  /**
   * Apply a security profile
   */
  profile(profileName: string): this {
    const profile = SECURITY_PROFILES[profileName];
    if (!profile) {
      throw new SigningError(
        `Unknown security profile: ${profileName}`,
        'UNKNOWN_PROFILE',
        { availableProfiles: Object.keys(SECURITY_PROFILES) }
      );
    }
    
    this.config.components = { ...profile.components };
    return this;
  }

  /**
   * Set custom nonce generator
   */
  nonceGenerator(generator: () => string): this {
    this.config.nonceGenerator = generator;
    return this;
  }

  /**
   * Set custom timestamp generator
   */
  timestampGenerator(generator: () => number): this {
    this.config.timestampGenerator = generator;
    return this;
  }

  /**
   * Build the final configuration
   */
  build(): SigningConfig {
    // Validate required fields
    if (!this.config.algorithm) {
      throw new SigningError('Algorithm is required', 'MISSING_ALGORITHM');
    }
    
    if (!this.config.keyId) {
      throw new SigningError('Key ID is required', 'MISSING_KEY_ID');
    }
    
    if (!this.config.privateKey) {
      throw new SigningError('Private key is required', 'MISSING_PRIVATE_KEY');
    }

    // Set defaults
    const config: SigningConfig = {
      algorithm: this.config.algorithm,
      keyId: this.config.keyId,
      privateKey: this.config.privateKey,
      components: this.config.components || DEFAULT_SIGNATURE_COMPONENTS,
      nonceGenerator: this.config.nonceGenerator || generateNonce,
      timestampGenerator: this.config.timestampGenerator || generateTimestamp
    };

    // Validate the configuration
    validateSigningConfig(config);

    return config;
  }
}

/**
 * Create a new signing configuration builder
 */
export function createSigningConfig(): SigningConfigBuilder {
  return new SigningConfigBuilder();
}

/**
 * Create signing configuration from security profile
 */
export function createFromProfile(
  profileName: string,
  keyId: string,
  privateKey: Uint8Array
): SigningConfig {
  return createSigningConfig()
    .profile(profileName)
    .keyId(keyId)
    .privateKey(privateKey)
    .algorithm('ed25519')
    .build();
}

/**
 * Validate signing configuration
 */
export function validateSigningConfig(config: SigningConfig): void {
  // Validate algorithm
  if (config.algorithm !== 'ed25519') {
    throw new SigningError(
      `Unsupported algorithm: ${config.algorithm}`,
      'UNSUPPORTED_ALGORITHM'
    );
  }

  // Validate key ID
  if (!config.keyId || typeof config.keyId !== 'string') {
    throw new SigningError('Key ID must be a non-empty string', 'INVALID_KEY_ID');
  }

  // Validate private key
  if (!validateSigningPrivateKey(config.privateKey)) {
    throw new SigningError('Invalid private key format', 'INVALID_PRIVATE_KEY');
  }

  // Validate components
  validateSignatureComponents(config.components);

  // Validate generators
  if (config.nonceGenerator) {
    try {
      const testNonce = config.nonceGenerator();
      if (typeof testNonce !== 'string') {
        throw new SigningError('Nonce generator must return a string', 'INVALID_NONCE_GENERATOR');
      }
    } catch (error) {
      if (error instanceof SigningError) throw error;
      throw new SigningError(
        'Nonce generator failed during validation',
        'NONCE_GENERATOR_FAILED',
        { originalError: error instanceof Error ? error.message : 'Unknown error' }
      );
    }
  }

  if (config.timestampGenerator) {
    try {
      const testTimestamp = config.timestampGenerator();
      if (!validateTimestamp(testTimestamp)) {
        throw new SigningError('Timestamp generator must return a valid Unix timestamp', 'INVALID_TIMESTAMP_GENERATOR');
      }
    } catch (error) {
      if (error instanceof SigningError) throw error;
      throw new SigningError(
        'Timestamp generator failed during validation',
        'TIMESTAMP_GENERATOR_FAILED',
        { originalError: error instanceof Error ? error.message : 'Unknown error' }
      );
    }
  }
}

/**
 * Validate signature components configuration
 */
export function validateSignatureComponents(components: SignatureComponents): void {
  if (!components || typeof components !== 'object') {
    throw new SigningError('Signature components must be an object', 'INVALID_COMPONENTS');
  }

  // Validate headers array if present
  if (components.headers) {
    if (!Array.isArray(components.headers)) {
      throw new SigningError('Headers must be an array', 'INVALID_HEADERS_TYPE');
    }

    for (const header of components.headers) {
      if (typeof header !== 'string') {
        throw new SigningError('Header names must be strings', 'INVALID_HEADER_NAME');
      }

      if (!header.trim()) {
        throw new SigningError('Header names cannot be empty', 'EMPTY_HEADER_NAME');
      }

      // Validate header name format (basic check)
      if (!/^[a-zA-Z0-9\-_]+$/.test(header)) {
        throw new SigningError(
          `Invalid header name format: ${header}`,
          'INVALID_HEADER_FORMAT'
        );
      }
    }
  }

  // Ensure at least one component is selected
  const hasAnyComponent = components.method || 
                         components.targetUri || 
                         (components.headers && components.headers.length > 0) ||
                         components.contentDigest;

  if (!hasAnyComponent) {
    throw new SigningError(
      'At least one signature component must be enabled',
      'NO_COMPONENTS_SELECTED'
    );
  }
}

/**
 * Get available security profiles
 */
export function getAvailableProfiles(): string[] {
  return Object.keys(SECURITY_PROFILES);
}

/**
 * Get security profile details
 */
export function getProfile(name: string): SecurityProfile | undefined {
  return SECURITY_PROFILES[name];
}

/**
 * Clone signing configuration (creates deep copy with new private key array)
 */
export function cloneSigningConfig(config: SigningConfig): SigningConfig {
  return {
    ...config,
    privateKey: config.privateKey.slice(), // Create new array copy
    components: { ...config.components },
    nonceGenerator: config.nonceGenerator,
    timestampGenerator: config.timestampGenerator
  };
}