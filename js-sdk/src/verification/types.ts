/**
 * Type definitions for signature verification functionality
 */

import { SignatureParams, SignableRequest, DigestAlgorithm } from '../signing/types.js';

/**
 * Verification result status
 */
export type VerificationStatus = 'valid' | 'invalid' | 'unknown' | 'error';

/**
 * Verification policy for different security levels
 */
export interface VerificationPolicy {
  /** Policy name */
  name: string;
  /** Policy description */
  description: string;
  /** Whether to verify timestamp validity */
  verifyTimestamp: boolean;
  /** Maximum allowed timestamp age in seconds */
  maxTimestampAge?: number;
  /** Whether to verify nonce format */
  verifyNonce: boolean;
  /** Whether to verify content digest */
  verifyContentDigest: boolean;
  /** Required signature components */
  requiredComponents?: string[];
  /** Allowed signature algorithms */
  allowedAlgorithms: string[];
  /** Whether to require all specified headers */
  requireAllHeaders: boolean;
  /** Custom verification rules */
  customRules?: VerificationRule[];
}

/**
 * Custom verification rule
 */
export interface VerificationRule {
  /** Rule name */
  name: string;
  /** Rule description */
  description: string;
  /** Rule validation function */
  validate: (context: VerificationContext) => Promise<VerificationRuleResult> | VerificationRuleResult;
}

/**
 * Verification rule result
 */
export interface VerificationRuleResult {
  /** Whether the rule passed */
  passed: boolean;
  /** Rule-specific message */
  message?: string;
  /** Additional details */
  details?: Record<string, any>;
}

/**
 * Verification configuration
 */
export interface VerificationConfig {
  /** Default verification policy */
  defaultPolicy?: string;
  /** Available verification policies */
  policies: Record<string, VerificationPolicy>;
  /** Public keys for verification (keyId -> publicKey) */
  publicKeys: Record<string, Uint8Array>;
  /** Trusted key sources */
  trustedKeySources?: KeySource[];
  /** Performance monitoring settings */
  performanceMonitoring?: {
    enabled: boolean;
    maxVerificationTime: number;
  };
}

/**
 * Key source for dynamic key retrieval
 */
export interface KeySource {
  /** Source name */
  name: string;
  /** Source type */
  type: 'url' | 'function' | 'cache';
  /** Key retrieval function or URL */
  source: string | ((keyId: string) => Promise<Uint8Array | null>);
  /** Cache TTL for retrieved keys */
  cacheTtl?: number;
}

/**
 * Signature data extracted from headers
 */
export interface ExtractedSignatureData {
  /** Signature identifier (e.g., 'sig1') */
  signatureId: string;
  /** Raw signature value */
  signature: string;
  /** Covered components */
  coveredComponents: string[];
  /** Signature parameters */
  params: SignatureParams;
  /** Content digest if present */
  contentDigest?: {
    algorithm: DigestAlgorithm;
    value: string;
  };
}

/**
 * Verification context for signature validation
 */
export interface VerificationContext {
  /** Original request/response being verified */
  message: SignableRequest | VerifiableResponse;
  /** Extracted signature data */
  signatureData: ExtractedSignatureData;
  /** Verification policy being applied */
  policy: VerificationPolicy;
  /** Public key for verification */
  publicKey: Uint8Array;
  /** Additional context data */
  metadata?: Record<string, any>;
}

/**
 * Response that can be verified
 */
export interface VerifiableResponse {
  /** HTTP status code */
  status: number;
  /** Response headers */
  headers: Record<string, string>;
  /** Response body */
  body?: string | Uint8Array;
  /** Original request URL */
  url: string;
  /** HTTP method of original request */
  method: string;
}

/**
 * Comprehensive verification result
 */
export interface VerificationResult {
  /** Overall verification status */
  status: VerificationStatus;
  /** Whether signature is valid */
  signatureValid: boolean;
  /** Individual check results */
  checks: {
    /** Signature format validation */
    formatValid: boolean;
    /** Cryptographic signature verification */
    cryptographicValid: boolean;
    /** Timestamp validation */
    timestampValid: boolean;
    /** Nonce format validation */
    nonceValid: boolean;
    /** Content digest validation */
    contentDigestValid: boolean;
    /** Component coverage validation */
    componentCoverageValid: boolean;
    /** Custom rules validation */
    customRulesValid: boolean;
  };
  /** Detailed diagnostic information */
  diagnostics: VerificationDiagnostics;
  /** Performance metrics */
  performance: {
    /** Total verification time in milliseconds */
    totalTime: number;
    /** Individual step timings */
    stepTimings: Record<string, number>;
  };
  /** Error information if verification failed */
  error?: {
    code: string;
    message: string;
    details?: Record<string, any>;
  };
}

/**
 * Detailed diagnostic information
 */
export interface VerificationDiagnostics {
  /** Signature metadata analysis */
  signatureAnalysis: {
    /** Signature algorithm used */
    algorithm: string;
    /** Key ID used */
    keyId: string;
    /** Signature creation timestamp */
    created: number;
    /** Signature age in seconds */
    age: number;
    /** Nonce value */
    nonce: string;
    /** Covered components */
    coveredComponents: string[];
  };
  /** Content analysis */
  contentAnalysis: {
    /** Whether content digest was present */
    hasContentDigest: boolean;
    /** Content digest algorithm if present */
    digestAlgorithm?: DigestAlgorithm;
    /** Content size */
    contentSize: number;
    /** Content type */
    contentType?: string;
  };
  /** Policy compliance */
  policyCompliance: {
    /** Policy name applied */
    policyName: string;
    /** Required components that were missing */
    missingRequiredComponents: string[];
    /** Extra components that were found */
    extraComponents: string[];
    /** Policy rule results */
    ruleResults: VerificationRuleResult[];
  };
  /** Security analysis */
  securityAnalysis: {
    /** Security level assessment */
    securityLevel: 'low' | 'medium' | 'high';
    /** Potential security concerns */
    concerns: string[];
    /** Recommendations */
    recommendations: string[];
  };
}

/**
 * Verification error types
 */
export class VerificationError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly details?: Record<string, any>
  ) {
    super(message);
    this.name = 'VerificationError';
  }
}

/**
 * Signature inspector interface for debugging
 */
export interface SignatureInspector {
  /** Inspect signature format and structure */
  inspectFormat(headers: Record<string, string>): SignatureFormatAnalysis;
  /** Analyze signature components */
  analyzeComponents(signatureData: ExtractedSignatureData): ComponentAnalysis;
  /** Validate signature parameters */
  validateParameters(params: SignatureParams): ParameterValidation;
  /** Generate diagnostic report */
  generateDiagnosticReport(result: VerificationResult): string;
}

/**
 * Signature format analysis result
 */
export interface SignatureFormatAnalysis {
  /** Whether format is valid RFC 9421 */
  isValidRFC9421: boolean;
  /** Format issues found */
  issues: FormatIssue[];
  /** Signature headers found */
  signatureHeaders: string[];
  /** Detected signature identifiers */
  signatureIds: string[];
}

/**
 * Format issue description
 */
export interface FormatIssue {
  /** Issue severity */
  severity: 'error' | 'warning' | 'info';
  /** Issue code */
  code: string;
  /** Human-readable message */
  message: string;
  /** Affected header or component */
  component?: string;
}

/**
 * Component analysis result
 */
export interface ComponentAnalysis {
  /** Components that are correctly formatted */
  validComponents: string[];
  /** Components with issues */
  invalidComponents: ComponentIssue[];
  /** Missing required components */
  missingComponents: string[];
  /** Security assessment of components */
  securityAssessment: ComponentSecurityAssessment;
}

/**
 * Component issue description
 */
export interface ComponentIssue {
  /** Component name */
  component: string;
  /** Issue type */
  type: 'format' | 'missing' | 'unexpected' | 'security';
  /** Issue message */
  message: string;
  /** Suggested fix */
  suggestion?: string;
}

/**
 * Component security assessment
 */
export interface ComponentSecurityAssessment {
  /** Overall security level */
  level: 'low' | 'medium' | 'high';
  /** Security score (0-100) */
  score: number;
  /** Security strengths */
  strengths: string[];
  /** Security weaknesses */
  weaknesses: string[];
}

/**
 * Parameter validation result
 */
export interface ParameterValidation {
  /** Whether all parameters are valid */
  allValid: boolean;
  /** Individual parameter validations */
  parameters: {
    created: { valid: boolean; message?: string };
    keyid: { valid: boolean; message?: string };
    alg: { valid: boolean; message?: string };
    nonce: { valid: boolean; message?: string };
  };
  /** Additional parameter insights */
  insights: string[];
}

/**
 * Test vector for verification testing
 */
export interface VerificationTestVector {
  /** Test name */
  name: string;
  /** Test description */
  description: string;
  /** Test category */
  category: 'positive' | 'negative' | 'edge-case';
  /** Input request/response */
  input: {
    message: SignableRequest | VerifiableResponse;
    headers: Record<string, string>;
    publicKey: Uint8Array;
  };
  /** Expected verification result */
  expected: {
    status: VerificationStatus;
    signatureValid: boolean;
    specificChecks?: Partial<VerificationResult['checks']>;
  };
  /** Test metadata */
  metadata?: {
    rfc9421Compliant: boolean;
    securityLevel: string;
    tags: string[];
  };
}