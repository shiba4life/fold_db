/**
 * Type definitions for request signing functionality
 */

/**
 * HTTP methods supported for signing
 */
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'HEAD' | 'OPTIONS';

/**
 * Signature algorithm types
 */
export type SignatureAlgorithm = 'ed25519';

/**
 * Request to be signed
 */
export interface SignableRequest {
  method: HttpMethod;
  url: string;
  headers: Record<string, string>;
  body?: string | Uint8Array;
}

/**
 * Signature components that can be included in the signature
 */
export interface SignatureComponents {
  /** HTTP method (@method) */
  method?: boolean;
  /** Target URI (@target-uri) */
  targetUri?: boolean;
  /** Specific headers to include */
  headers?: string[];
  /** Content digest for request body */
  contentDigest?: boolean;
}

/**
 * Signing configuration
 */
export interface SigningConfig {
  /** Signature algorithm to use */
  algorithm: SignatureAlgorithm;
  /** Key ID for the signature */
  keyId: string;
  /** Private key for signing (raw bytes) */
  privateKey: Uint8Array;
  /** Components to include in signature */
  components: SignatureComponents;
  /** Custom nonce generator (optional) */
  nonceGenerator?: () => string;
  /** Custom timestamp generator (optional) */
  timestampGenerator?: () => number;
}

/**
 * Signature parameters for RFC 9421
 */
export interface SignatureParams {
  /** Timestamp when signature was created */
  created: number;
  /** Key identifier */
  keyid: string;
  /** Signature algorithm */
  alg: SignatureAlgorithm;
  /** Unique nonce for replay protection */
  nonce: string;
}

/**
 * Generated signature result for RFC 9421
 */
export interface RFC9421SignatureResult {
  /** Signature-Input header value */
  signatureInput: string;
  /** Signature header value */
  signature: string;
  /** All headers that should be added to the request */
  headers: Record<string, string>;
  /** Canonical message that was signed */
  canonicalMessage: string;
}

/**
 * Content digest algorithms
 */
export type DigestAlgorithm = 'sha-256' | 'sha-512';

/**
 * Content digest result
 */
export interface ContentDigest {
  algorithm: DigestAlgorithm;
  digest: string;
  headerValue: string;
}

/**
 * Signing options for individual requests
 */
export interface SigningOptions {
  /** Override default components */
  components?: Partial<SignatureComponents>;
  /** Custom nonce for this request */
  nonce?: string;
  /** Custom timestamp for this request */
  timestamp?: number;
  /** Content digest algorithm */
  digestAlgorithm?: DigestAlgorithm;
}

/**
 * Error types for signing operations
 */
export class SigningError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly details?: Record<string, any>
  ) {
    super(message);
    this.name = 'SigningError';
  }
}

/**
 * Signing context for tracking signature state
 */
export interface SigningContext {
  /** Request being signed */
  request: SignableRequest;
  /** Signing configuration */
  config: SigningConfig;
  /** Signing options for this request */
  options: SigningOptions;
  /** Generated signature parameters */
  params: SignatureParams;
  /** Content digest if applicable */
  contentDigest?: ContentDigest;
}