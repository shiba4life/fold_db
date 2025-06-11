/**
 * RFC 9421 HTTP Message Signatures implementation with Ed25519
 */

import * as ed25519 from '@noble/ed25519';
import {
  SignableRequest,
  SigningConfig,
  SigningOptions,
  RFC9421SignatureResult,
  SignatureParams,
  SigningContext,
  SigningError,
  ContentDigest,
  DigestAlgorithm
} from './types.js';
import {
  calculateContentDigest,
  generateNonce,
  generateTimestamp,
  validateNonce,
  validateTimestamp,
  toHex,
  PerformanceTimer
} from './utils.js';
import {
  buildCanonicalMessage,
  buildSignatureInput,
  extractCoveredComponents
} from './canonical-message.js';
import { validateSigningConfig } from './signing-config.js';

/**
 * RFC 9421 HTTP Message Signatures signer with Ed25519
 */
export class RFC9421Signer {
  private config: SigningConfig;

  constructor(config: SigningConfig) {
    validateSigningConfig(config);
    this.config = { ...config }; // Shallow copy
  }

  /**
   * Sign an HTTP request according to RFC 9421
   */
  async signRequest(
    request: SignableRequest,
    options: SigningOptions = {}
  ): Promise<RFC9421SignatureResult> {
    const timer = new PerformanceTimer();
    
    try {
      // Create signing context
      const context = await this.createSigningContext(request, options);
      
      // Build canonical message
      const canonicalMessage = await buildCanonicalMessage(context);
      
      // Sign the canonical message
      const signatureBytes = await this.signMessage(canonicalMessage);
      const signature = toHex(signatureBytes);
      
      // Get covered components for signature input
      const coveredComponents = extractCoveredComponents(canonicalMessage);
      
      // Build signature input header
      const signatureInput = buildSignatureInput('sig1', coveredComponents, context.params);
      
      // Build result headers
      const headers: Record<string, string> = {
        'signature-input': signatureInput,
        'signature': `sig1=:${signature}:`
      };
      
      // Add content-digest header if applicable
      if (context.contentDigest) {
        headers['content-digest'] = context.contentDigest.headerValue;
      }
      
      // Add content-type header if not present and we have a body
      if (this.shouldAddContentType(request, context)) {
        headers['content-type'] = this.getDefaultContentType(request);
      }
      
      const elapsed = timer.elapsed();
      
      // Performance check (should be <10ms as per requirements)
      if (elapsed > 10) {
        console.warn(`Signing operation took ${elapsed.toFixed(2)}ms (target: <10ms)`);
      }
      
      return {
        signatureInput,
        signature: `sig1=:${signature}:`,
        headers,
        canonicalMessage
      };
      
    } catch (error) {
      if (error instanceof SigningError) {
        throw error;
      }
      
      throw new SigningError(
        `Request signing failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'SIGNING_FAILED',
        {
          originalError: error instanceof Error ? error.message : 'Unknown error',
          elapsed: timer.elapsed()
        }
      );
    }
  }

  /**
   * Sign multiple requests efficiently
   */
  async signRequests(
    requests: SignableRequest[],
    options: SigningOptions = {}
  ): Promise<RFC9421SignatureResult[]> {
    if (!Array.isArray(requests)) {
      throw new SigningError('Requests must be an array', 'INVALID_REQUESTS_TYPE');
    }
    
    if (requests.length === 0) {
      return [];
    }
    
    if (requests.length > 100) {
      throw new SigningError('Cannot sign more than 100 requests at once', 'TOO_MANY_REQUESTS');
    }

    const results: RFC9421SignatureResult[] = [];
    
    for (const request of requests) {
      const result = await this.signRequest(request, options);
      results.push(result);
    }

    return results;
  }

  /**
   * Update signing configuration
   */
  updateConfig(newConfig: Partial<SigningConfig>): void {
    const updatedConfig = { ...this.config, ...newConfig };
    validateSigningConfig(updatedConfig);
    this.config = updatedConfig;
  }

  /**
   * Get current configuration (read-only copy)
   */
  getConfig(): Readonly<SigningConfig> {
    return {
      ...this.config,
      privateKey: this.config.privateKey.slice() // New array copy
    };
  }

  /**
   * Create signing context for a request
   */
  private async createSigningContext(
    request: SignableRequest,
    options: SigningOptions
  ): Promise<SigningContext> {
    // Merge components with options
    const components = {
      ...this.config.components,
      ...options.components
    };

    // Generate signature parameters
    const params: SignatureParams = {
      created: options.timestamp ?? this.config.timestampGenerator!(),
      keyid: this.config.keyId,
      alg: this.config.algorithm,
      nonce: options.nonce ?? this.config.nonceGenerator!()
    };

    // Validate parameters
    this.validateSignatureParams(params);

    // Calculate content digest if needed
    let contentDigest: ContentDigest | undefined;
    if (components.contentDigest && this.hasRequestBody(request)) {
      const digestAlgorithm = options.digestAlgorithm || 'sha-256';
      contentDigest = await calculateContentDigest(request.body!, digestAlgorithm);
    }

    return {
      request,
      config: this.config,
      options,
      params,
      contentDigest
    };
  }

  /**
   * Sign a canonical message using Ed25519
   */
  private async signMessage(message: string): Promise<Uint8Array> {
    try {
      const messageBytes = new TextEncoder().encode(message);
      return await ed25519.signAsync(messageBytes, this.config.privateKey);
    } catch (error) {
      throw new SigningError(
        `Ed25519 signing failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'ED25519_SIGNING_FAILED',
        { originalError: error instanceof Error ? error.message : 'Unknown error' }
      );
    }
  }

  /**
   * Validate signature parameters
   */
  private validateSignatureParams(params: SignatureParams): void {
    if (!validateTimestamp(params.created)) {
      throw new SigningError(
        `Invalid timestamp: ${params.created}`,
        'INVALID_TIMESTAMP'
      );
    }

    if (!validateNonce(params.nonce)) {
      throw new SigningError(
        `Invalid nonce format: ${params.nonce}`,
        'INVALID_NONCE'
      );
    }

    if (!params.keyid || typeof params.keyid !== 'string') {
      throw new SigningError('Key ID must be a non-empty string', 'INVALID_KEY_ID');
    }

    if (params.alg !== 'ed25519') {
      throw new SigningError(
        `Unsupported algorithm: ${params.alg}`,
        'UNSUPPORTED_ALGORITHM'
      );
    }
  }

  /**
   * Check if request has a body
   */
  private hasRequestBody(request: SignableRequest): boolean {
    return request.body !== undefined && request.body !== null;
  }

  /**
   * Check if we should add content-type header
   */
  private shouldAddContentType(request: SignableRequest, context: SigningContext): boolean {
    // Add content-type if:
    // 1. We're including it in signature components
    // 2. Request has a body
    // 3. Content-type header is not already present
    
    const hasContentTypeComponent = context.config.components.headers?.includes('content-type') ?? false;
    const hasBody = this.hasRequestBody(request);
    const hasContentTypeHeader = this.hasHeader(request, 'content-type');

    return hasContentTypeComponent && hasBody && !hasContentTypeHeader;
  }

  /**
   * Check if request has a specific header
   */
  private hasHeader(request: SignableRequest, headerName: string): boolean {
    const lowerName = headerName.toLowerCase();
    return Object.keys(request.headers).some(key => key.toLowerCase() === lowerName);
  }

  /**
   * Get default content type based on request body
   */
  private getDefaultContentType(request: SignableRequest): string {
    if (!request.body) {
      return 'application/octet-stream';
    }

    if (typeof request.body === 'string') {
      // Try to detect JSON
      try {
        JSON.parse(request.body);
        return 'application/json';
      } catch {
        return 'text/plain; charset=utf-8';
      }
    }

    // Binary data
    return 'application/octet-stream';
  }
}

/**
 * Create a new RFC 9421 signer instance
 */
export function createSigner(config: SigningConfig): RFC9421Signer {
  return new RFC9421Signer(config);
}

/**
 * Sign a single request with the provided configuration
 */
export async function signRequest(
  request: SignableRequest,
  config: SigningConfig,
  options: SigningOptions = {}
): Promise<RFC9421SignatureResult> {
  const signer = createSigner(config);
  return signer.signRequest(request, options);
}

/**
 * Verify a signature (for testing purposes)
 * Note: This is mainly for development/testing. Production verification should be done server-side.
 */
export async function verifySignature(
  request: SignableRequest,
  signatureResult: RFC9421SignatureResult,
  publicKey: Uint8Array
): Promise<boolean> {
  try {
    const messageBytes = new TextEncoder().encode(signatureResult.canonicalMessage);
    
    // Extract signature from signature header
    // Format: sig1=:hexsignature:
    const signatureMatch = signatureResult.signature.match(/sig1=:([^:]+):/);
    if (!signatureMatch) {
      return false;
    }
    
    const signatureHex = signatureMatch[1];
    const signatureBytes = new Uint8Array(
      signatureHex.match(/.{1,2}/g)!.map((byte: string) => parseInt(byte, 16))
    );
    
    return await ed25519.verifyAsync(signatureBytes, messageBytes, publicKey);
  } catch {
    return false;
  }
}

/**
 * Extract signature information from signed request headers
 */
export function extractSignatureInfo(headers: Record<string, string>): {
  signatureInput?: string;
  signature?: string;
  contentDigest?: string;
} {
  const result: any = {};
  
  for (const [key, value] of Object.entries(headers)) {
    const lowerKey = key.toLowerCase();
    
    switch (lowerKey) {
      case 'signature-input':
        result.signatureInput = value;
        break;
      case 'signature':
        result.signature = value;
        break;
      case 'content-digest':
        result.contentDigest = value;
        break;
    }
  }
  
  return result;
}

/**
 * Merge signature headers into request headers
 */
export function applySignatureHeaders(
  requestHeaders: Record<string, string>,
  signatureResult: RFC9421SignatureResult
): Record<string, string> {
  return {
    ...requestHeaders,
    ...signatureResult.headers
  };
}

/**
 * Check if request appears to be signed
 */
export function isRequestSigned(request: SignableRequest): boolean {
  const headers = Object.keys(request.headers).map(k => k.toLowerCase());
  return headers.includes('signature-input') && headers.includes('signature');
}