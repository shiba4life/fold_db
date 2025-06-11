/**
 * Core signature verification engine for RFC 9421 HTTP Message Signatures
 */

import * as ed25519 from '@noble/ed25519';
import {
  VerificationResult,
  VerificationConfig,
  VerificationPolicy,
  VerificationContext,
  ExtractedSignatureData,
  VerificationError,
  VerifiableResponse,
  VerificationStatus,
  VerificationDiagnostics,
  KeySource
} from './types.js';
import {
  SignableRequest,
  SignatureParams,
  ContentDigest,
  DigestAlgorithm
} from '../signing/types.js';
import {
  parseSignatureInput,
  buildCanonicalMessage
} from '../signing/canonical-message.js';
import {
  calculateContentDigest,
  validateTimestamp,
  validateNonce,
  fromHex,
  PerformanceTimer
} from '../signing/utils.js';
import { VERIFICATION_POLICIES } from './policies.js';

/**
 * RFC 9421 signature verifier
 */
export class RFC9421Verifier {
  private config: VerificationConfig;
  private keyCache = new Map<string, { key: Uint8Array; timestamp: number }>();
  private nonceCache = new Set<string>();

  constructor(config: VerificationConfig) {
    this.config = { ...config };
    this.validateConfig();
  }

  /**
   * Verify a signed request or response
   */
  async verify(
    message: SignableRequest | VerifiableResponse,
    headers: Record<string, string>,
    options: {
      policy?: string;
      publicKey?: Uint8Array;
      keyId?: string;
      skipKeyRetrieval?: boolean;
    } = {}
  ): Promise<VerificationResult> {
    const timer = new PerformanceTimer();
    const stepTimings: Record<string, number> = {};

    try {
      // Step 1: Extract signature data from headers
      timer.reset();
      const signatureData = this.extractSignatureData(headers);
      stepTimings.extraction = timer.elapsed();

      // Step 2: Get verification policy
      timer.reset();
      const policy = this.getPolicy(options.policy);
      stepTimings.policyRetrieval = timer.elapsed();

      // Step 3: Get public key for verification
      timer.reset();
      const publicKey = await this.getPublicKey(
        signatureData.params.keyid,
        options.publicKey,
        options.skipKeyRetrieval
      );
      stepTimings.keyRetrieval = timer.elapsed();

      // Step 4: Create verification context
      const context: VerificationContext = {
        message,
        signatureData,
        policy,
        publicKey,
        metadata: {
          headers,
          options
        }
      };

      // Step 5: Perform comprehensive verification
      timer.reset();
      const result = await this.performVerification(context);
      stepTimings.verification = timer.elapsed();

      // Add performance metrics
      result.performance = {
        totalTime: Object.values(stepTimings).reduce((sum, time) => sum + time, 0),
        stepTimings
      };

      return result;

    } catch (error) {
      const totalTime = timer.elapsed();
      
      if (error instanceof VerificationError) {
        // Critical errors that should be thrown (not returned as error results)
        if (error.code === 'MISSING_SIGNATURE_INPUT' || error.code === 'MISSING_SIGNATURE') {
          throw error;
        }
        
        // Format-related errors should return 'invalid' status
        if (this.isFormatError(error.code)) {
          return this.createInvalidResult(error, totalTime, stepTimings);
        }
        
        return this.createErrorResult(error, totalTime, stepTimings);
      }

      const verificationError = new VerificationError(
        `Verification failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'VERIFICATION_FAILED',
        {
          originalError: error instanceof Error ? error.message : 'Unknown error'
        }
      );

      return this.createErrorResult(verificationError, totalTime, stepTimings);
    }
  }

  /**
   * Verify multiple signatures (batch verification)
   */
  async verifyBatch(
    verifications: Array<{
      message: SignableRequest | VerifiableResponse;
      headers: Record<string, string>;
      options?: {
        policy?: string;
        publicKey?: Uint8Array;
        keyId?: string;
      };
    }>
  ): Promise<VerificationResult[]> {
    const results: VerificationResult[] = [];

    for (const verification of verifications) {
      const result = await this.verify(
        verification.message,
        verification.headers,
        verification.options
      );
      results.push(result);
    }

    return results;
  }

  /**
   * Update verification configuration
   */
  updateConfig(newConfig: Partial<VerificationConfig>): void {
    this.config = { ...this.config, ...newConfig };
    this.validateConfig();
  }

  /**
   * Add public key to configuration
   */
  addPublicKey(keyId: string, publicKey: Uint8Array): void {
    this.config.publicKeys[keyId] = publicKey.slice(); // Create copy
  }

  /**
   * Remove public key from configuration
   */
  removePublicKey(keyId: string): void {
    delete this.config.publicKeys[keyId];
  }

  /**
   * Clear nonce cache (for replay protection)
   */
  clearNonceCache(): void {
    this.nonceCache.clear();
  }

  /**
   * Extract signature data from HTTP headers
   */
  private extractSignatureData(headers: Record<string, string>): ExtractedSignatureData {
    // Find signature-input and signature headers (case-insensitive)
    const signatureInputHeader = this.findHeader(headers, 'signature-input');
    const signatureHeader = this.findHeader(headers, 'signature');

    if (!signatureInputHeader) {
      throw new VerificationError(
        'Signature-Input header not found',
        'MISSING_SIGNATURE_INPUT'
      );
    }

    if (!signatureHeader) {
      throw new VerificationError(
        'Signature header not found',
        'MISSING_SIGNATURE'
      );
    }

    // Parse signature input
    let parsedInput;
    try {
      parsedInput = parseSignatureInput(signatureInputHeader);
    } catch (error) {
      throw new VerificationError(
        'Invalid signature-input header format',
        'INVALID_SIGNATURE_INPUT_FORMAT'
      );
    }
    
    // Extract signature value - be more flexible with format
    const signatureMatch = signatureHeader.match(/([^=]+)=:([^:]+):/);
    if (!signatureMatch) {
      throw new VerificationError(
        'Invalid signature header format',
        'INVALID_SIGNATURE_FORMAT'
      );
    }

    const [, signatureId, signatureValue] = signatureMatch;
    
    if (signatureId !== parsedInput.signatureName) {
      throw new VerificationError(
        `Signature ID mismatch: ${signatureId} != ${parsedInput.signatureName}`,
        'SIGNATURE_ID_MISMATCH'
      );
    }

    // Extract content digest if present
    let contentDigest: ExtractedSignatureData['contentDigest'];
    const contentDigestHeader = this.findHeader(headers, 'content-digest');
    if (contentDigestHeader) {
      const digestMatch = contentDigestHeader.match(/([^=]+)=:([^:]+):/);
      if (digestMatch) {
        const [, algorithm, value] = digestMatch;
        contentDigest = {
          algorithm: algorithm as DigestAlgorithm,
          value
        };
      }
    }

    return {
      signatureId,
      signature: signatureValue,
      coveredComponents: parsedInput.coveredComponents,
      params: parsedInput.params,
      contentDigest
    };
  }

  /**
   * Find header with case-insensitive lookup
   */
  private findHeader(headers: Record<string, string>, name: string): string | undefined {
    const lowerName = name.toLowerCase();
    for (const [key, value] of Object.entries(headers)) {
      if (key.toLowerCase() === lowerName) {
        return value;
      }
    }
    return undefined;
  }

  /**
   * Get verification policy
   */
  private getPolicy(policyName?: string): VerificationPolicy {
    const name = policyName || this.config.defaultPolicy || 'standard';
    
    // Check config policies first
    if (this.config.policies[name]) {
      return this.config.policies[name];
    }

    // Check built-in policies
    if (VERIFICATION_POLICIES[name]) {
      return VERIFICATION_POLICIES[name];
    }

    throw new VerificationError(
      `Unknown verification policy: ${name}`,
      'UNKNOWN_POLICY',
      { availablePolicies: Object.keys(this.config.policies) }
    );
  }

  /**
   * Get public key for verification
   */
  private async getPublicKey(
    keyId: string,
    providedKey?: Uint8Array,
    skipRetrieval: boolean = false
  ): Promise<Uint8Array> {
    // Use provided key if available
    if (providedKey) {
      return providedKey;
    }

    // Check configuration
    if (this.config.publicKeys[keyId]) {
      return this.config.publicKeys[keyId];
    }

    // Skip key retrieval if requested
    if (skipRetrieval) {
      throw new VerificationError(
        `Public key not found for key ID: ${keyId}`,
        'PUBLIC_KEY_NOT_FOUND',
        { keyId }
      );
    }

    // Try key sources
    if (this.config.trustedKeySources) {
      for (const source of this.config.trustedKeySources) {
        try {
          const key = await this.retrieveKeyFromSource(source, keyId);
          if (key) {
            return key;
          }
        } catch (error) {
          // Continue to next source
          console.warn(`Key retrieval failed from source ${source.name}:`, error);
        }
      }
    }

    throw new VerificationError(
      `Public key not found for key ID: ${keyId}`,
      'PUBLIC_KEY_NOT_FOUND',
      { keyId }
    );
  }

  /**
   * Retrieve key from external source
   */
  private async retrieveKeyFromSource(
    source: KeySource,
    keyId: string
  ): Promise<Uint8Array | null> {
    if (typeof source.source === 'function') {
      return await source.source(keyId);
    } else if (typeof source.source === 'string' && source.type === 'url') {
      // HTTP key retrieval (simplified implementation)
      const response = await fetch(`${source.source}/${keyId}`);
      if (response.ok) {
        const keyData = await response.json();
        return new Uint8Array(keyData.publicKey);
      }
    }

    return null;
  }

  /**
   * Perform comprehensive signature verification
   */
  private async performVerification(context: VerificationContext): Promise<VerificationResult> {
    const checks = {
      formatValid: false,
      cryptographicValid: false,
      timestampValid: false,
      nonceValid: false,
      contentDigestValid: false,
      componentCoverageValid: false,
      customRulesValid: false
    };

    const diagnostics: VerificationDiagnostics = {
      signatureAnalysis: {
        algorithm: context.signatureData.params.alg,
        keyId: context.signatureData.params.keyid,
        created: context.signatureData.params.created,
        age: Math.floor(Date.now() / 1000) - context.signatureData.params.created,
        nonce: context.signatureData.params.nonce,
        coveredComponents: context.signatureData.coveredComponents
      },
      contentAnalysis: {
        hasContentDigest: !!context.signatureData.contentDigest,
        digestAlgorithm: context.signatureData.contentDigest?.algorithm,
        contentSize: this.getContentSize(context.message),
        contentType: this.getContentType(context.message)
      },
      policyCompliance: {
        policyName: context.policy.name,
        missingRequiredComponents: [],
        extraComponents: [],
        ruleResults: []
      },
      securityAnalysis: {
        securityLevel: 'medium',
        concerns: [],
        recommendations: []
      }
    };

    // 1. Format validation
    checks.formatValid = this.validateFormat(context);

    // 2. Cryptographic verification
    checks.cryptographicValid = await this.verifyCryptographicSignature(context);

    // 3. Timestamp validation
    if (context.policy.verifyTimestamp) {
      checks.timestampValid = this.validateTimestamp(context, diagnostics);
    } else {
      checks.timestampValid = true;
    }

    // 4. Nonce validation
    if (context.policy.verifyNonce) {
      checks.nonceValid = this.validateNonceFormat(context);
    } else {
      checks.nonceValid = true;
    }

    // 5. Content digest validation
    if (context.policy.verifyContentDigest) {
      checks.contentDigestValid = await this.validateContentDigest(context);
    } else {
      checks.contentDigestValid = true;
    }

    // 6. Component coverage validation
    checks.componentCoverageValid = this.validateComponentCoverage(context, diagnostics);

    // 7. Custom rules validation
    if (context.policy.customRules && context.policy.customRules.length > 0) {
      checks.customRulesValid = await this.validateCustomRules(context, diagnostics);
    } else {
      checks.customRulesValid = true;
    }

    // Determine overall status
    const allChecksValid = Object.values(checks).every(check => check);
    const signatureValid = checks.formatValid && checks.cryptographicValid;
    
    let status: VerificationStatus;
    if (allChecksValid && signatureValid) {
      status = 'valid';
    } else if (signatureValid) {
      status = 'invalid'; // Signature is cryptographically valid but fails policy
    } else {
      status = 'invalid';
    }

    // Generate security analysis
    this.generateSecurityAnalysis(checks, diagnostics);

    return {
      status,
      signatureValid,
      checks,
      diagnostics,
      performance: { totalTime: 0, stepTimings: {} } // Will be filled by caller
    };
  }

  /**
   * Validate signature format according to RFC 9421
   */
  private validateFormat(context: VerificationContext): boolean {
    try {
      // Check signature parameters
      const params = context.signatureData.params;
      
      if (!params.created || !params.keyid || !params.alg || !params.nonce) {
        return false;
      }

      // Check algorithm is allowed
      if (!context.policy.allowedAlgorithms.includes(params.alg)) {
        return false;
      }

      // Check covered components format
      const components = context.signatureData.coveredComponents;
      if (!Array.isArray(components) || components.length === 0) {
        return false;
      }

      return true;
    } catch {
      return false;
    }
  }

  /**
   * Verify cryptographic signature using Ed25519
   */
  private async verifyCryptographicSignature(context: VerificationContext): Promise<boolean> {
    try {
      // Reconstruct canonical message
      const signingContext = {
        request: context.message as SignableRequest,
        config: {
          components: this.reconstructSignatureComponents(context.signatureData.coveredComponents),
          algorithm: context.signatureData.params.alg,
          keyId: context.signatureData.params.keyid,
          privateKey: new Uint8Array(32) // Not used for verification
        },
        options: {},
        params: context.signatureData.params,
        contentDigest: context.signatureData.contentDigest ? {
          algorithm: context.signatureData.contentDigest.algorithm,
          digest: context.signatureData.contentDigest.value,
          headerValue: `${context.signatureData.contentDigest.algorithm}=:${context.signatureData.contentDigest.value}:`
        } : undefined
      };

      const canonicalMessage = await buildCanonicalMessage(signingContext);
      const messageBytes = new TextEncoder().encode(canonicalMessage);
      
      // Convert signature from base64 (RFC 9421 format)
      let signatureBytes: Uint8Array;
      try {
        const base64String = context.signatureData.signature.replace(/[^A-Za-z0-9+/]/g, '');
        const binaryString = atob(base64String);
        signatureBytes = new Uint8Array(binaryString.length);
        for (let i = 0; i < binaryString.length; i++) {
          signatureBytes[i] = binaryString.charCodeAt(i);
        }
      } catch (error) {
        console.warn('Failed to decode base64 signature:', error);
        return false;
      }
      
      // Verify signature
      return await ed25519.verifyAsync(signatureBytes, messageBytes, context.publicKey);
      
    } catch (error) {
      console.warn('Cryptographic verification failed:', error);
      return false;
    }
  }

  /**
   * Reconstruct signature components from covered components list
   */
  private reconstructSignatureComponents(coveredComponents: string[]): any {
    const components: any = {
      method: false,
      targetUri: false,
      headers: [],
      contentDigest: false
    };

    for (const component of coveredComponents) {
      switch (component) {
        case '@method':
          components.method = true;
          break;
        case '@target-uri':
          components.targetUri = true;
          break;
        case 'content-digest':
          components.contentDigest = true;
          break;
        default:
          if (!component.startsWith('@')) {
            components.headers.push(component);
          }
          break;
      }
    }

    return components;
  }

  /**
   * Validate timestamp according to policy
   */
  private validateTimestamp(context: VerificationContext, diagnostics: VerificationDiagnostics): boolean {
    const created = context.signatureData.params.created;
    
    if (!validateTimestamp(created)) {
      diagnostics.securityAnalysis.concerns.push('Invalid timestamp format');
      return false;
    }

    if (context.policy.maxTimestampAge) {
      const now = Math.floor(Date.now() / 1000);
      const age = now - created;
      
      if (age > context.policy.maxTimestampAge) {
        diagnostics.securityAnalysis.concerns.push(`Timestamp too old: ${age}s`);
        return false;
      }

      if (age < -60) { // Allow 1 minute clock skew
        diagnostics.securityAnalysis.concerns.push(`Timestamp from future: ${age}s`);
        return false;
      }
    }

    return true;
  }

  /**
   * Validate nonce format
   */
  private validateNonceFormat(context: VerificationContext): boolean {
    return validateNonce(context.signatureData.params.nonce);
  }

  /**
   * Validate content digest if present
   */
  private async validateContentDigest(context: VerificationContext): Promise<boolean> {
    const signatureDigest = context.signatureData.contentDigest;
    
    if (!signatureDigest) {
      // No digest to validate
      return true;
    }

    const message = context.message;
    if (!('body' in message) || !message.body) {
      // No body to digest
      return true;
    }

    try {
      const calculatedDigest = await calculateContentDigest(
        message.body,
        signatureDigest.algorithm
      );
      
      return calculatedDigest.digest === signatureDigest.value;
    } catch {
      return false;
    }
  }

  /**
   * Validate component coverage according to policy
   */
  private validateComponentCoverage(
    context: VerificationContext,
    diagnostics: VerificationDiagnostics
  ): boolean {
    const covered = context.signatureData.coveredComponents;
    const required = context.policy.requiredComponents || [];
    
    const missing = required.filter(component => !covered.includes(component));
    const extra = covered.filter(component => !required.includes(component));
    
    diagnostics.policyCompliance.missingRequiredComponents = missing;
    diagnostics.policyCompliance.extraComponents = extra;
    
    if (missing.length > 0 && context.policy.requireAllHeaders) {
      return false;
    }

    return true;
  }

  /**
   * Validate custom rules
   */
  private async validateCustomRules(
    context: VerificationContext,
    diagnostics: VerificationDiagnostics
  ): Promise<boolean> {
    let allRulesPassed = true;

    for (const rule of context.policy.customRules || []) {
      try {
        const result = await rule.validate(context);
        diagnostics.policyCompliance.ruleResults.push(result);
        
        if (!result.passed) {
          allRulesPassed = false;
        }
      } catch (error) {
        diagnostics.policyCompliance.ruleResults.push({
          passed: false,
          message: `Rule ${rule.name} failed: ${error instanceof Error ? error.message : 'Unknown error'}`
        });
        allRulesPassed = false;
      }
    }

    return allRulesPassed;
  }

  /**
   * Generate security analysis
   */
  private generateSecurityAnalysis(
    checks: VerificationResult['checks'],
    diagnostics: VerificationDiagnostics
  ): void {
    const analysis = diagnostics.securityAnalysis;
    
    // Calculate security level
    const validChecks = Object.values(checks).filter(check => check).length;
    const totalChecks = Object.values(checks).length;
    const securityScore = (validChecks / totalChecks) * 100;
    
    if (securityScore >= 90) {
      analysis.securityLevel = 'high';
    } else if (securityScore >= 70) {
      analysis.securityLevel = 'medium';
    } else {
      analysis.securityLevel = 'low';
    }

    // Add recommendations
    if (!checks.timestampValid) {
      analysis.recommendations.push('Consider implementing timestamp validation');
    }
    if (!checks.nonceValid) {
      analysis.recommendations.push('Implement proper nonce validation for replay protection');
    }
    if (!checks.contentDigestValid) {
      analysis.recommendations.push('Include content digest for request integrity');
    }
  }

  /**
   * Get content size from message
   */
  private getContentSize(message: SignableRequest | VerifiableResponse): number {
    if ('body' in message && message.body) {
      if (typeof message.body === 'string') {
        return new TextEncoder().encode(message.body).length;
      } else {
        return message.body.length;
      }
    }
    return 0;
  }

  /**
   * Get content type from message
   */
  private getContentType(message: SignableRequest | VerifiableResponse): string | undefined {
    const headers = message.headers;
    return this.findHeader(headers, 'content-type');
  }

  /**
   * Create error result
   */
  private createErrorResult(
    error: VerificationError,
    totalTime: number,
    stepTimings: Record<string, number>
  ): VerificationResult {
    return {
      status: 'error',
      signatureValid: false,
      checks: {
        formatValid: false,
        cryptographicValid: false,
        timestampValid: false,
        nonceValid: false,
        contentDigestValid: false,
        componentCoverageValid: false,
        customRulesValid: false
      },
      diagnostics: {
        signatureAnalysis: {
          algorithm: '',
          keyId: '',
          created: 0,
          age: 0,
          nonce: '',
          coveredComponents: []
        },
        contentAnalysis: {
          hasContentDigest: false,
          contentSize: 0
        },
        policyCompliance: {
          policyName: '',
          missingRequiredComponents: [],
          extraComponents: [],
          ruleResults: []
        },
        securityAnalysis: {
          securityLevel: 'low',
          concerns: [error.message],
          recommendations: []
        }
      },
      performance: {
        totalTime,
        stepTimings
      },
      error: {
        code: error.code,
        message: error.message,
        details: error.details
      }
    };
  }

  /**
   * Check if error code represents a format error
   */
  private isFormatError(code: string): boolean {
    const formatErrorCodes = [
      'INVALID_SIGNATURE_FORMAT',
      'INVALID_SIGNATURE_INPUT_FORMAT',
      'SIGNATURE_ID_MISMATCH',
      'INVALID_COMPONENT_FORMAT',
      'INVALID_PARAMETER_FORMAT'
    ];
    return formatErrorCodes.includes(code);
  }

  /**
   * Create invalid result for format errors
   */
  private createInvalidResult(
    error: VerificationError,
    totalTime: number,
    stepTimings: Record<string, number>
  ): VerificationResult {
    return {
      status: 'invalid',
      signatureValid: false,
      checks: {
        formatValid: false,
        cryptographicValid: false,
        timestampValid: false,
        nonceValid: false,
        contentDigestValid: false,
        componentCoverageValid: false,
        customRulesValid: false
      },
      diagnostics: {
        signatureAnalysis: {
          algorithm: '',
          keyId: '',
          created: 0,
          age: 0,
          nonce: '',
          coveredComponents: []
        },
        contentAnalysis: {
          hasContentDigest: false,
          contentSize: 0
        },
        policyCompliance: {
          policyName: '',
          missingRequiredComponents: [],
          extraComponents: [],
          ruleResults: []
        },
        securityAnalysis: {
          securityLevel: 'low',
          concerns: [error.message],
          recommendations: []
        }
      },
      performance: {
        totalTime,
        stepTimings
      },
      error: {
        code: error.code,
        message: error.message,
        details: error.details
      }
    };
  }

  /**
   * Validate configuration
   */
  private validateConfig(): void {
    if (!this.config.policies || Object.keys(this.config.policies).length === 0) {
      throw new VerificationError(
        'At least one verification policy must be configured',
        'NO_POLICIES_CONFIGURED'
      );
    }

    if (!this.config.publicKeys) {
      this.config.publicKeys = {};
    }
  }
}

/**
 * Create a new RFC 9421 verifier instance
 */
export function createVerifier(config: VerificationConfig): RFC9421Verifier {
  return new RFC9421Verifier(config);
}

/**
 * Quick signature verification helper
 */
export async function verifySignature(
  message: SignableRequest | VerifiableResponse,
  headers: Record<string, string>,
  publicKey: Uint8Array,
  policy: string = 'standard'
): Promise<boolean> {
  const config: VerificationConfig = {
    policies: VERIFICATION_POLICIES,
    publicKeys: {},
    defaultPolicy: policy
  };

  const verifier = createVerifier(config);
  const result = await verifier.verify(message, headers, { publicKey });
  
  return result.signatureValid && result.status === 'valid';
}