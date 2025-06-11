/**
 * Verification middleware for HTTP responses and requests
 */

import {
  VerificationConfig,
  VerificationResult,
  VerifiableResponse,
  VerificationError,
  VerificationPolicy
} from './types.js';
import { SignableRequest } from '../signing/types.js';
import { RFC9421Verifier, createVerifier } from './verifier.js';
import { VERIFICATION_POLICIES } from './policies.js';

/**
 * Response verification middleware configuration
 */
export interface ResponseVerificationConfig {
  /** Verification configuration */
  verificationConfig: VerificationConfig;
  /** Default policy to use */
  defaultPolicy?: string;
  /** Whether to throw on verification failure */
  throwOnFailure?: boolean;
  /** Custom error handler */
  onVerificationFailure?: (result: VerificationResult, response: Response) => void;
  /** Skip verification for certain URLs */
  skipPatterns?: RegExp[];
  /** Enable performance monitoring */
  enablePerfMonitoring?: boolean;
}

/**
 * Request verification middleware configuration
 */
export interface RequestVerificationConfig {
  /** Verification configuration */
  verificationConfig: VerificationConfig;
  /** Default policy to use */
  defaultPolicy?: string;
  /** Whether to reject invalid requests */
  rejectInvalid?: boolean;
  /** Custom validation handler */
  onValidationResult?: (result: VerificationResult, request: Request) => void;
}

/**
 * Verification middleware for fetch responses
 */
export function createResponseVerificationMiddleware(
  config: ResponseVerificationConfig
) {
  const verifier = createVerifier(config.verificationConfig);
  
  return async (response: Response): Promise<Response> => {
    // Check if we should skip verification
    if (config.skipPatterns) {
      const url = response.url;
      for (const pattern of config.skipPatterns) {
        if (pattern.test(url)) {
          return response;
        }
      }
    }

    try {
      // Extract headers
      const headers: Record<string, string> = {};
      response.headers.forEach((value, key) => {
        headers[key.toLowerCase()] = value;
      });

      // Check if response has signature headers
      if (!headers['signature-input'] || !headers['signature']) {
        // No signature to verify
        return response;
      }

      // Clone response to read body
      const responseClone = response.clone();
      const body = await responseClone.text();

      // Create verifiable response
      const verifiableResponse: VerifiableResponse = {
        status: response.status,
        headers,
        body,
        url: response.url,
        method: 'GET' // Default, would need to be passed from original request
      };

      // Perform verification
      const result = await verifier.verify(
        verifiableResponse,
        headers,
        { policy: config.defaultPolicy }
      );

      // Handle verification result
      if (result.status !== 'valid' || !result.signatureValid) {
        if (config.onVerificationFailure) {
          config.onVerificationFailure(result, response);
        }

        if (config.throwOnFailure) {
          throw new VerificationError(
            `Response signature verification failed: ${result.error?.message || 'Unknown error'}`,
            'RESPONSE_VERIFICATION_FAILED',
            {
              url: response.url,
              status: response.status,
              verificationResult: result
            }
          );
        }
      }

      // Add verification result as metadata (if possible)
      if ('verificationResult' in response) {
        (response as any).verificationResult = result;
      }

      return response;

    } catch (error) {
      if (error instanceof VerificationError) {
        throw error;
      }

      if (config.throwOnFailure) {
        throw new VerificationError(
          `Response verification middleware failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
          'MIDDLEWARE_ERROR',
          {
            originalError: error instanceof Error ? error.message : 'Unknown error',
            url: response.url
          }
        );
      }

      // Log error but don't fail the request
      console.warn('Response verification middleware error:', error);
      return response;
    }
  };
}

/**
 * Verification middleware for incoming requests (server-side)
 */
export function createRequestVerificationMiddleware(
  config: RequestVerificationConfig
) {
  const verifier = createVerifier(config.verificationConfig);

  return async (request: Request): Promise<{ 
    valid: boolean; 
    result: VerificationResult; 
    shouldReject: boolean 
  }> => {
    try {
      // Extract headers
      const headers: Record<string, string> = {};
      request.headers.forEach((value, key) => {
        headers[key.toLowerCase()] = value;
      });

      // Check if request has signature headers
      if (!headers['signature-input'] || !headers['signature']) {
        return {
          valid: false,
          result: {
            status: 'unknown',
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
              signatureAnalysis: { algorithm: '', keyId: '', created: 0, age: 0, nonce: '', coveredComponents: [] },
              contentAnalysis: { hasContentDigest: false, contentSize: 0 },
              policyCompliance: { policyName: '', missingRequiredComponents: [], extraComponents: [], ruleResults: [] },
              securityAnalysis: { securityLevel: 'low', concerns: ['No signature present'], recommendations: [] }
            },
            performance: { totalTime: 0, stepTimings: {} }
          },
          shouldReject: false
        };
      }

      // Read request body if present
      let body: string | undefined;
      if (request.body) {
        const requestClone = request.clone();
        body = await requestClone.text();
      }

      // Create signable request
      const signableRequest: SignableRequest = {
        method: request.method as any,
        url: request.url,
        headers,
        body
      };

      // Perform verification
      const result = await verifier.verify(
        signableRequest,
        headers,
        { policy: config.defaultPolicy }
      );

      // Handle verification result
      if (config.onValidationResult) {
        config.onValidationResult(result, request);
      }

      const valid = result.status === 'valid' && result.signatureValid;
      const shouldReject = !valid && (config.rejectInvalid ?? false);

      return {
        valid,
        result,
        shouldReject
      };

    } catch (error) {
      const errorResult: VerificationResult = {
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
          signatureAnalysis: { algorithm: '', keyId: '', created: 0, age: 0, nonce: '', coveredComponents: [] },
          contentAnalysis: { hasContentDigest: false, contentSize: 0 },
          policyCompliance: { policyName: '', missingRequiredComponents: [], extraComponents: [], ruleResults: [] },
          securityAnalysis: { securityLevel: 'low', concerns: [`Middleware error: ${error instanceof Error ? error.message : 'Unknown'}`], recommendations: [] }
        },
        performance: { totalTime: 0, stepTimings: {} },
        error: {
          code: 'MIDDLEWARE_ERROR',
          message: error instanceof Error ? error.message : 'Unknown error'
        }
      };

      return {
        valid: false,
        result: errorResult,
        shouldReject: config.rejectInvalid ?? false
      };
    }
  };
}

/**
 * Express.js middleware for request verification
 */
export function createExpressVerificationMiddleware(
  config: RequestVerificationConfig
) {
  const middleware = createRequestVerificationMiddleware(config);

  return async (req: any, res: any, next: any) => {
    try {
      // Convert Express request to standard Request
      const headers: Record<string, string> = {};
      for (const [key, value] of Object.entries(req.headers)) {
        if (typeof value === 'string') {
          headers[key.toLowerCase()] = value;
        } else if (Array.isArray(value)) {
          headers[key.toLowerCase()] = value[0];
        }
      }

      const url = `${req.protocol}://${req.get('host')}${req.originalUrl}`;
      const body = req.body ? JSON.stringify(req.body) : undefined;

      const mockRequest = {
        method: req.method,
        url,
        headers: new Headers(headers),
        body: body ? new ReadableStream({
          start(controller) {
            controller.enqueue(new TextEncoder().encode(body));
            controller.close();
          }
        }) : null,
        clone: () => mockRequest
      } as Request;

      const { valid, result, shouldReject } = await middleware(mockRequest);

      // Add verification result to request
      req.signatureVerification = {
        valid,
        result,
        shouldReject
      };

      if (shouldReject) {
        return res.status(401).json({
          error: 'Signature verification failed',
          code: 'INVALID_SIGNATURE',
          details: result.error
        });
      }

      next();

    } catch (error) {
      console.error('Express verification middleware error:', error);
      if (config.rejectInvalid) {
        return res.status(500).json({
          error: 'Signature verification error',
          code: 'VERIFICATION_ERROR'
        });
      }
      next();
    }
  };
}

/**
 * Fetch interceptor for automatic response verification
 */
export function createFetchInterceptor(
  config: ResponseVerificationConfig
): (input: RequestInfo | URL, init?: RequestInit) => Promise<Response> {
  const originalFetch = globalThis.fetch;
  const middleware = createResponseVerificationMiddleware(config);

  return async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
    const response = await originalFetch(input, init);
    return middleware(response);
  };
}

/**
 * Policy-based verification middleware factory
 */
export function createPolicyBasedMiddleware(
  policies: Record<string, string>, // URL pattern -> policy name mapping
  config: Omit<ResponseVerificationConfig, 'defaultPolicy'>
) {
  return async (response: Response): Promise<Response> => {
    // Find matching policy
    let policyName = 'standard'; // default
    const url = response.url;

    for (const [pattern, policy] of Object.entries(policies)) {
      if (new RegExp(pattern).test(url)) {
        policyName = policy;
        break;
      }
    }

    // Create middleware with specific policy
    const middleware = createResponseVerificationMiddleware({
      ...config,
      defaultPolicy: policyName
    });

    return middleware(response);
  };
}

/**
 * Batch verification utility
 */
export class BatchVerifier {
  private verifier: RFC9421Verifier;

  constructor(config: VerificationConfig) {
    this.verifier = createVerifier(config);
  }

  /**
   * Verify multiple requests/responses
   */
  async verifyBatch(
    items: Array<{
      message: SignableRequest | VerifiableResponse;
      headers: Record<string, string>;
      policy?: string;
      publicKey?: Uint8Array;
    }>
  ): Promise<VerificationResult[]> {
    const results: VerificationResult[] = [];

    // Process in parallel for better performance
    const promises = items.map(item =>
      this.verifier.verify(item.message, item.headers, {
        policy: item.policy,
        publicKey: item.publicKey
      })
    );

    const batchResults = await Promise.allSettled(promises);

    for (const result of batchResults) {
      if (result.status === 'fulfilled') {
        results.push(result.value);
      } else {
        // Create error result
        results.push({
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
            signatureAnalysis: { algorithm: '', keyId: '', created: 0, age: 0, nonce: '', coveredComponents: [] },
            contentAnalysis: { hasContentDigest: false, contentSize: 0 },
            policyCompliance: { policyName: '', missingRequiredComponents: [], extraComponents: [], ruleResults: [] },
            securityAnalysis: { securityLevel: 'low', concerns: [result.reason.message], recommendations: [] }
          },
          performance: { totalTime: 0, stepTimings: {} },
          error: {
            code: 'BATCH_VERIFICATION_ERROR',
            message: result.reason.message
          }
        });
      }
    }

    return results;
  }

  /**
   * Get batch verification statistics
   */
  getBatchStats(results: VerificationResult[]): {
    total: number;
    valid: number;
    invalid: number;
    errors: number;
    averageTime: number;
  } {
    const total = results.length;
    const valid = results.filter(r => r.status === 'valid').length;
    const invalid = results.filter(r => r.status === 'invalid').length;
    const errors = results.filter(r => r.status === 'error').length;
    const averageTime = results.reduce((sum, r) => sum + r.performance.totalTime, 0) / total;

    return {
      total,
      valid,
      invalid,
      errors,
      averageTime
    };
  }
}

/**
 * Create a batch verifier instance
 */
export function createBatchVerifier(config: VerificationConfig): BatchVerifier {
  return new BatchVerifier(config);
}