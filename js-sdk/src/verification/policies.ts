/**
 * Predefined verification policies for different security levels
 */

import {
  VerificationPolicy,
  VerificationRule,
  VerificationRuleResult,
  VerificationContext
} from './types.js';
import { validateTimestamp, validateNonce } from '../signing/utils.js';

/**
 * Strict verification policy for high-security scenarios
 */
export const STRICT_VERIFICATION_POLICY: VerificationPolicy = {
  name: 'strict',
  description: 'Maximum security verification with comprehensive checks',
  verifyTimestamp: true,
  maxTimestampAge: 300, // 5 minutes
  verifyNonce: true,
  verifyContentDigest: true,
  requiredComponents: ['@method', '@target-uri', 'content-type', 'content-digest'],
  allowedAlgorithms: ['ed25519'],
  requireAllHeaders: true,
  customRules: [
    {
      name: 'replay-protection',
      description: 'Ensure nonce is fresh and not reused',
      validate: async (context: VerificationContext): Promise<VerificationRuleResult> => {
        // In a real implementation, this would check against a nonce cache
        const nonce = context.signatureData.params.nonce;
        
        if (!validateNonce(nonce)) {
          return {
            passed: false,
            message: 'Invalid nonce format',
            details: { nonce }
          };
        }

        // Note: In production, implement nonce cache checking here
        return {
          passed: true,
          message: 'Nonce validation passed'
        };
      }
    },
    {
      name: 'algorithm-strength',
      description: 'Ensure strong cryptographic algorithm',
      validate: (context: VerificationContext): VerificationRuleResult => {
        const algorithm = context.signatureData.params.alg;
        const strongAlgorithms = ['ed25519'];
        
        if (!strongAlgorithms.includes(algorithm)) {
          return {
            passed: false,
            message: `Weak algorithm detected: ${algorithm}`,
            details: { algorithm, strongAlgorithms }
          };
        }

        return {
          passed: true,
          message: 'Algorithm strength validated'
        };
      }
    }
  ]
};

/**
 * Standard verification policy for balanced security
 */
export const STANDARD_VERIFICATION_POLICY: VerificationPolicy = {
  name: 'standard',
  description: 'Balanced verification suitable for most applications',
  verifyTimestamp: true,
  maxTimestampAge: 900, // 15 minutes
  verifyNonce: true,
  verifyContentDigest: true,
  requiredComponents: ['@method', '@target-uri'],
  allowedAlgorithms: ['ed25519'],
  requireAllHeaders: false,
  customRules: [
    {
      name: 'basic-replay-protection',
      description: 'Basic nonce format validation',
      validate: (context: VerificationContext): VerificationRuleResult => {
        const nonce = context.signatureData.params.nonce;
        
        if (!validateNonce(nonce)) {
          return {
            passed: false,
            message: 'Invalid nonce format',
            details: { nonce }
          };
        }

        return {
          passed: true,
          message: 'Nonce format validated'
        };
      }
    }
  ]
};

/**
 * Lenient verification policy for development/testing
 */
export const LENIENT_VERIFICATION_POLICY: VerificationPolicy = {
  name: 'lenient',
  description: 'Relaxed verification for development and testing',
  verifyTimestamp: true,
  maxTimestampAge: 3600, // 1 hour
  verifyNonce: false,
  verifyContentDigest: false,
  requiredComponents: ['@method'],
  allowedAlgorithms: ['ed25519'],
  requireAllHeaders: false,
  customRules: []
};

/**
 * Legacy verification policy for backward compatibility
 */
export const LEGACY_VERIFICATION_POLICY: VerificationPolicy = {
  name: 'legacy',
  description: 'Legacy verification for older signature formats',
  verifyTimestamp: false,
  verifyNonce: false,
  verifyContentDigest: false,
  requiredComponents: [],
  allowedAlgorithms: ['ed25519'],
  requireAllHeaders: false,
  customRules: []
};

/**
 * Predefined verification policies registry
 */
export const VERIFICATION_POLICIES: Record<string, VerificationPolicy> = {
  strict: STRICT_VERIFICATION_POLICY,
  standard: STANDARD_VERIFICATION_POLICY,
  lenient: LENIENT_VERIFICATION_POLICY,
  legacy: LEGACY_VERIFICATION_POLICY
};

/**
 * Custom verification rules library
 */
export const VERIFICATION_RULES = {
  /**
   * Timestamp freshness rule
   */
  timestampFreshness: (maxAge: number): VerificationRule => ({
    name: 'timestamp-freshness',
    description: `Ensure timestamp is within ${maxAge} seconds`,
    validate: (context: VerificationContext): VerificationRuleResult => {
      const now = Math.floor(Date.now() / 1000);
      const created = context.signatureData.params.created;
      const age = now - created;

      if (age > maxAge) {
        return {
          passed: false,
          message: `Timestamp too old: ${age}s > ${maxAge}s`,
          details: { age, maxAge, created, now }
        };
      }

      if (age < -60) { // Allow 1 minute clock skew
        return {
          passed: false,
          message: `Timestamp from future: ${age}s`,
          details: { age, created, now }
        };
      }

      return {
        passed: true,
        message: `Timestamp is fresh: ${age}s old`
      };
    }
  }),

  /**
   * Required headers rule
   */
  requiredHeaders: (headers: string[]): VerificationRule => ({
    name: 'required-headers',
    description: `Ensure required headers are present: ${headers.join(', ')}`,
    validate: (context: VerificationContext): VerificationRuleResult => {
      const coveredComponents = context.signatureData.coveredComponents;
      const missing = headers.filter(header => !coveredComponents.includes(header.toLowerCase()));

      if (missing.length > 0) {
        return {
          passed: false,
          message: `Missing required headers: ${missing.join(', ')}`,
          details: { missing, required: headers, covered: coveredComponents }
        };
      }

      return {
        passed: true,
        message: 'All required headers present'
      };
    }
  }),

  /**
   * Key ID validation rule
   */
  keyIdValidation: (validKeyIds: string[]): VerificationRule => ({
    name: 'key-id-validation',
    description: 'Validate key ID against allowed list',
    validate: (context: VerificationContext): VerificationRuleResult => {
      const keyId = context.signatureData.params.keyid;

      if (!validKeyIds.includes(keyId)) {
        return {
          passed: false,
          message: `Invalid key ID: ${keyId}`,
          details: { keyId, validKeyIds }
        };
      }

      return {
        passed: true,
        message: 'Key ID validated'
      };
    }
  }),

  /**
   * Content type consistency rule
   */
  contentTypeConsistency: (): VerificationRule => ({
    name: 'content-type-consistency',
    description: 'Ensure content-type is covered when body is present',
    validate: (context: VerificationContext): VerificationRuleResult => {
      const message = context.message;
      const hasBody = ('body' in message && message.body !== undefined && message.body !== null);
      const coveredComponents = context.signatureData.coveredComponents;
      const hasContentType = coveredComponents.includes('content-type');

      if (hasBody && !hasContentType) {
        return {
          passed: false,
          message: 'Content-type should be covered when body is present',
          details: { hasBody, hasContentType }
        };
      }

      return {
        passed: true,
        message: 'Content-type coverage is appropriate'
      };
    }
  }),

  /**
   * Nonce uniqueness rule (requires external storage)
   */
  nonceUniqueness: (nonceCache: Set<string> | Map<string, number>): VerificationRule => ({
    name: 'nonce-uniqueness',
    description: 'Ensure nonce has not been used before',
    validate: (context: VerificationContext): VerificationRuleResult => {
      const nonce = context.signatureData.params.nonce;

      if (nonceCache instanceof Set) {
        if (nonceCache.has(nonce)) {
          return {
            passed: false,
            message: `Nonce already used: ${nonce}`,
            details: { nonce }
          };
        }
        nonceCache.add(nonce);
      } else if (nonceCache instanceof Map) {
        const lastUsed = nonceCache.get(nonce);
        if (lastUsed) {
          return {
            passed: false,
            message: `Nonce already used at: ${new Date(lastUsed).toISOString()}`,
            details: { nonce, lastUsed }
          };
        }
        nonceCache.set(nonce, Date.now());
      }

      return {
        passed: true,
        message: 'Nonce is unique'
      };
    }
  })
};

/**
 * Create a custom verification policy
 */
export function createVerificationPolicy(
  name: string,
  description: string,
  options: Partial<VerificationPolicy>
): VerificationPolicy {
  return {
    name,
    description,
    verifyTimestamp: true,
    verifyNonce: true,
    verifyContentDigest: true,
    allowedAlgorithms: ['ed25519'],
    requireAllHeaders: false,
    customRules: [],
    ...options
  };
}

/**
 * Merge verification policies
 */
export function mergeVerificationPolicies(
  base: VerificationPolicy,
  override: Partial<VerificationPolicy>
): VerificationPolicy {
  return {
    ...base,
    ...override,
    customRules: [
      ...(base.customRules || []),
      ...(override.customRules || [])
    ]
  };
}

/**
 * Get verification policy by name
 */
export function getVerificationPolicy(name: string): VerificationPolicy | undefined {
  return VERIFICATION_POLICIES[name];
}

/**
 * Get all available verification policies
 */
export function getAvailableVerificationPolicies(): string[] {
  return Object.keys(VERIFICATION_POLICIES);
}

/**
 * Validate verification policy configuration
 */
export function validateVerificationPolicy(policy: VerificationPolicy): void {
  if (!policy.name || typeof policy.name !== 'string') {
    throw new Error('Policy name must be a non-empty string');
  }

  if (!policy.description || typeof policy.description !== 'string') {
    throw new Error('Policy description must be a non-empty string');
  }

  if (!Array.isArray(policy.allowedAlgorithms) || policy.allowedAlgorithms.length === 0) {
    throw new Error('Policy must specify at least one allowed algorithm');
  }

  if (policy.maxTimestampAge !== undefined && policy.maxTimestampAge <= 0) {
    throw new Error('Maximum timestamp age must be positive');
  }

  if (policy.customRules) {
    for (const rule of policy.customRules) {
      if (!rule.name || typeof rule.name !== 'string') {
        throw new Error('Custom rule name must be a non-empty string');
      }
      if (typeof rule.validate !== 'function') {
        throw new Error('Custom rule must have a validate function');
      }
    }
  }
}