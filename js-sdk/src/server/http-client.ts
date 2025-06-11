/**
 * HTTP client for DataFold server communication
 * Handles public key registration and signature verification with automatic request signing
 */

import { DataFoldServerError, ServerConnectionConfig, RetryConfig } from '../types.js';
import {
  RFC9421Signer,
  SigningConfig,
  SignableRequest,
  HttpMethod,
  applySignatureHeaders,
  SigningOptions,
  RFC9421SignatureResult,
  SigningError
} from '../signing/index';

/**
 * Default configuration for server connections
 */
const DEFAULT_CONFIG: ServerConnectionConfig = {
  baseUrl: 'http://localhost:9001',
  timeout: 30000,
  retries: 3,
  retryDelay: 1000
};

/**
 * Default retry configuration
 */
const DEFAULT_RETRY_CONFIG: RetryConfig = {
  maxRetries: 3,
  baseDelay: 1000,
  maxDelay: 10000,
  backoffFactor: 2
};

/**
 * HTTP response wrapper
 */
interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details: Record<string, any>;
  };
}

/**
 * Signing behavior modes
 */
export type SigningMode = 'auto' | 'manual' | 'disabled';

/**
 * Per-endpoint signing configuration
 */
export interface EndpointSigningConfig {
  /** Whether signing is enabled for this endpoint */
  enabled: boolean;
  /** Custom signing options for this endpoint */
  options?: SigningOptions;
  /** Whether to require signing (fail if signing fails) */
  required?: boolean;
}

/**
 * Request interceptor function type
 */
export type RequestInterceptor = (
  request: SignableRequest,
  context: RequestContext
) => Promise<SignableRequest> | SignableRequest;

/**
 * Response interceptor function type
 */
export type ResponseInterceptor = (
  response: Response,
  context: RequestContext
) => Promise<Response> | Response;

/**
 * Request context for interceptors
 */
export interface RequestContext {
  /** Request attempt number (0-based) */
  attempt: number;
  /** Total retry attempts configured */
  maxRetries: number;
  /** Request start timestamp */
  startTime: number;
  /** Endpoint identifier */
  endpoint: string;
  /** Whether this request will be signed */
  willBeSigned: boolean;
}

/**
 * Signature cache entry
 */
interface SignatureCacheEntry {
  signature: RFC9421SignatureResult;
  timestamp: number;
  ttl: number;
}

/**
 * Signing metrics
 */
export interface SigningMetrics {
  /** Total requests attempted */
  totalRequests: number;
  /** Requests that were signed */
  signedRequests: number;
  /** Signing failures */
  signingFailures: number;
  /** Average signing time in ms */
  averageSigningTime: number;
  /** Cache hits */
  cacheHits: number;
  /** Cache misses */
  cacheMisses: number;
}

/**
 * Enhanced HTTP client configuration
 */
export interface HttpClientConfig extends Partial<ServerConnectionConfig> {
  /** Signing behavior mode */
  signingMode?: SigningMode;
  /** Enable signature caching */
  enableSignatureCache?: boolean;
  /** Signature cache TTL in milliseconds */
  signatureCacheTtl?: number;
  /** Maximum cache size */
  maxCacheSize?: number;
  /** Enable debug logging */
  debugLogging?: boolean;
  /** Per-endpoint signing configuration */
  endpointConfig?: Record<string, EndpointSigningConfig>;
  /** Default signing options */
  defaultSigningOptions?: SigningOptions;
}

/**
 * HTTP client for DataFold server communication
 */
export class DataFoldHttpClient {
  private config: ServerConnectionConfig;
  private retryConfig: RetryConfig;
  private signer?: RFC9421Signer;
  private signingMode: SigningMode = 'auto';
  private enableSignatureCache: boolean = true;
  private signatureCacheTtl: number = 300000; // 5 minutes
  private maxCacheSize: number = 1000;
  private debugLogging: boolean = false;
  private endpointConfig: Record<string, EndpointSigningConfig> = {};
  private defaultSigningOptions?: SigningOptions;
  
  // Internal state
  private signatureCache = new Map<string, SignatureCacheEntry>();
  private requestInterceptors: RequestInterceptor[] = [];
  private responseInterceptors: ResponseInterceptor[] = [];
  private metrics: SigningMetrics = {
    totalRequests: 0,
    signedRequests: 0,
    signingFailures: 0,
    averageSigningTime: 0,
    cacheHits: 0,
    cacheMisses: 0
  };

  constructor(config: HttpClientConfig = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.retryConfig = config.retryConfig || DEFAULT_RETRY_CONFIG;
    
    // Apply enhanced configuration
    if (config.signingMode !== undefined) {
      this.signingMode = config.signingMode;
    }
    if (config.enableSignatureCache !== undefined) {
      this.enableSignatureCache = config.enableSignatureCache;
    }
    if (config.signatureCacheTtl !== undefined) {
      this.signatureCacheTtl = config.signatureCacheTtl;
    }
    if (config.maxCacheSize !== undefined) {
      this.maxCacheSize = config.maxCacheSize;
    }
    if (config.debugLogging !== undefined) {
      this.debugLogging = config.debugLogging;
    }
    if (config.endpointConfig) {
      this.endpointConfig = { ...config.endpointConfig };
    }
    if (config.defaultSigningOptions) {
      this.defaultSigningOptions = { ...config.defaultSigningOptions };
    }
  }

  /**
   * Update client configuration
   */
  updateConfig(config: HttpClientConfig): void {
    this.config = { ...this.config, ...config };
    if (config.retryConfig) {
      this.retryConfig = config.retryConfig;
    }
    
    // Update enhanced configuration
    if (config.signingMode !== undefined) {
      this.signingMode = config.signingMode;
    }
    if (config.enableSignatureCache !== undefined) {
      this.enableSignatureCache = config.enableSignatureCache;
    }
    if (config.signatureCacheTtl !== undefined) {
      this.signatureCacheTtl = config.signatureCacheTtl;
    }
    if (config.maxCacheSize !== undefined) {
      this.maxCacheSize = config.maxCacheSize;
    }
    if (config.debugLogging !== undefined) {
      this.debugLogging = config.debugLogging;
    }
    if (config.endpointConfig) {
      this.endpointConfig = { ...this.endpointConfig, ...config.endpointConfig };
    }
    if (config.defaultSigningOptions) {
      this.defaultSigningOptions = { ...config.defaultSigningOptions };
    }
  }

  /**
   * Configure request signing
   */
  configureSigning(signingConfig: SigningConfig): this {
    this.signer = new RFC9421Signer(signingConfig);
    return this;
  }

  /**
   * Enable automatic request signing
   */
  enableSigning(signer: RFC9421Signer): this {
    this.signer = signer;
    return this;
  }

  /**
   * Disable automatic request signing
   */
  disableSigning(): this {
    this.signer = undefined;
    return this;
  }

  /**
   * Set signing mode
   */
  setSigningMode(mode: SigningMode): this {
    this.signingMode = mode;
    return this;
  }

  /**
   * Configure endpoint-specific signing
   */
  configureEndpointSigning(endpoint: string, config: EndpointSigningConfig): this {
    this.endpointConfig[endpoint] = config;
    return this;
  }

  /**
   * Add request interceptor
   */
  addRequestInterceptor(interceptor: RequestInterceptor): this {
    this.requestInterceptors.push(interceptor);
    return this;
  }

  /**
   * Add response interceptor
   */
  addResponseInterceptor(interceptor: ResponseInterceptor): this {
    this.responseInterceptors.push(interceptor);
    return this;
  }

  /**
   * Check if signing is enabled
   */
  isSigningEnabled(): boolean {
    return this.signer !== undefined && this.signingMode !== 'disabled';
  }

  /**
   * Check if endpoint should be signed
   */
  shouldSignEndpoint(endpoint: string): boolean {
    if (!this.isSigningEnabled()) {
      return false;
    }

    const endpointConfig = this.endpointConfig[endpoint];
    if (endpointConfig) {
      return endpointConfig.enabled;
    }

    // Default behavior based on signing mode
    return this.signingMode === 'auto';
  }

  /**
   * Get signing metrics
   */
  getSigningMetrics(): Readonly<SigningMetrics> {
    return { ...this.metrics };
  }

  /**
   * Reset signing metrics
   */
  resetSigningMetrics(): this {
    this.metrics = {
      totalRequests: 0,
      signedRequests: 0,
      signingFailures: 0,
      averageSigningTime: 0,
      cacheHits: 0,
      cacheMisses: 0
    };
    return this;
  }

  /**
   * Clear signature cache
   */
  clearSignatureCache(): this {
    this.signatureCache.clear();
    return this;
  }

  /**
   * Enable debug logging
   */
  enableDebugLogging(enabled: boolean = true): this {
    this.debugLogging = enabled;
    return this;
  }

  /**
   * Make an HTTP request with retry logic and automatic signing
   */
  private async makeRequest<T>(
    method: 'GET' | 'POST' | 'PUT' | 'DELETE',
    endpoint: string,
    body?: any,
    customTimeout?: number,
    customSigningOptions?: SigningOptions
  ): Promise<T> {
    const url = `${this.config.baseUrl}/api/crypto${endpoint}`;
    const timeout = customTimeout || this.config.timeout;
    const startTime = Date.now();

    // Update metrics
    this.metrics.totalRequests++;

    let lastError: Error | null = null;

    for (let attempt = 0; attempt <= this.retryConfig.maxRetries; attempt++) {
      try {
        // Create request context
        const context: RequestContext = {
          attempt,
          maxRetries: this.retryConfig.maxRetries,
          startTime,
          endpoint,
          willBeSigned: this.shouldSignRequest(endpoint)
        };

        // Create abort controller for timeout
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), timeout);

        // Prepare headers
        const headers: Record<string, string> = {
          'Content-Type': 'application/json',
          'User-Agent': 'DataFold-JS-SDK/0.1.0'
        };

        // Prepare body
        let requestBody: string | undefined;
        if (body && method !== 'GET') {
          requestBody = JSON.stringify(body);
        }

        // Create signable request
        let signableRequest: SignableRequest = {
          method: method as HttpMethod,
          url,
          headers: { ...headers },
          body: requestBody
        };

        // Apply request interceptors
        for (const interceptor of this.requestInterceptors) {
          signableRequest = await interceptor(signableRequest, context);
        }

        // Apply automatic signing if enabled
        if (this.shouldSignRequest(endpoint)) {
          try {
            const signingResult = await this.performSigning(signableRequest, endpoint, customSigningOptions);
            if (signingResult) {
              // Merge signature headers
              Object.assign(signableRequest.headers, signingResult.headers);
              this.metrics.signedRequests++;
              
              if (this.debugLogging) {
                console.debug('Request signed successfully', {
                  endpoint,
                  signatureInput: signingResult.signatureInput,
                  attempt
                });
              }
            }
          } catch (signingError) {
            this.metrics.signingFailures++;
            
            const endpointConfig = this.endpointConfig[endpoint];
            const signingRequired = endpointConfig?.required ?? false;
            
            if (signingRequired) {
              throw new DataFoldServerError(
                `Required signing failed for endpoint ${endpoint}: ${signingError instanceof Error ? signingError.message : 'Unknown error'}`,
                'SIGNING_REQUIRED_FAILED',
                0,
                {
                  originalError: signingError instanceof Error ? signingError.message : 'Unknown error',
                  endpoint,
                  attempt
                }
              );
            }
            
            if (this.debugLogging) {
              console.warn('Request signing failed (continuing without signature):', {
                endpoint,
                error: signingError instanceof Error ? signingError.message : signingError,
                attempt
              });
            }
          }
        }

        const requestInit: RequestInit = {
          method,
          headers: signableRequest.headers,
          signal: controller.signal
        };

        if (signableRequest.body) {
          requestInit.body = signableRequest.body;
        }

        let response = await fetch(signableRequest.url, requestInit);
        clearTimeout(timeoutId);

        // Apply response interceptors
        for (const interceptor of this.responseInterceptors) {
          response = await interceptor(response, context);
        }

        // Parse response
        const responseData: ApiResponse<T> = await response.json();

        // Handle successful response
        if (response.ok && responseData.success) {
          if (this.debugLogging) {
            console.debug('Request completed successfully', {
              endpoint,
              duration: Date.now() - startTime,
              attempt,
              signed: context.willBeSigned
            });
          }
          return responseData.data as T;
        }

        // Handle API error response
        if (responseData.error) {
          throw new DataFoldServerError(
            responseData.error.message,
            responseData.error.code,
            response.status,
            responseData.error.details
          );
        }

        // Handle HTTP error without API error structure
        throw new DataFoldServerError(
          `HTTP ${response.status}: ${response.statusText}`,
          'HTTP_ERROR',
          response.status
        );

      } catch (error) {
        lastError = error as Error;

        if (this.debugLogging) {
          console.debug('Request attempt failed', {
            endpoint,
            attempt,
            error: error instanceof Error ? error.message : error
          });
        }

        // Don't retry on certain errors
        if (error instanceof DataFoldServerError) {
          const nonRetryableCodes = [
            'INVALID_PUBLIC_KEY',
            'INVALID_CLIENT_ID',
            'INVALID_MESSAGE',
            'INVALID_SIGNATURE',
            'INVALID_ENCODING',
            'CLIENT_ALREADY_REGISTERED',
            'DUPLICATE_REGISTRATION', // Add this to prevent retries on registration conflicts
            'DUPLICATE_PUBLIC_KEY',
            'CLIENT_NOT_FOUND',
            'REGISTRATION_NOT_FOUND',
            'KEY_NOT_ACTIVE',
            'SIGNATURE_VERIFICATION_FAILED',
            'SIGNING_REQUIRED_FAILED'
          ];

          if (nonRetryableCodes.includes(error.errorCode)) {
            throw error;
          }

          // Don't retry on 4xx client errors (except rate limiting)
          if (error.httpStatus >= 400 && error.httpStatus < 500 && error.httpStatus !== 429) {
            throw error;
          }
        }

        // Don't retry if we've exhausted attempts
        if (attempt >= this.retryConfig.maxRetries) {
          break;
        }

        // Calculate delay for next attempt
        const delay = Math.min(
          this.retryConfig.baseDelay * Math.pow(this.retryConfig.backoffFactor, attempt),
          this.retryConfig.maxDelay
        );

        // Wait before retry
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }

    // If we get here, all retries failed
    throw new DataFoldServerError(
      `Request failed after ${this.retryConfig.maxRetries + 1} attempts: ${lastError?.message}`,
      'MAX_RETRIES_EXCEEDED',
      0,
      { originalError: lastError?.message }
    );
  }

  /**
   * Check if a request should be signed
   */
  private shouldSignRequest(endpoint: string): boolean {
    if (!this.isSigningEnabled()) {
      return false;
    }

    // Try exact match first
    const exactConfig = this.endpointConfig[endpoint];
    if (exactConfig !== undefined) {
      return exactConfig.enabled;
    }

    // Try pattern matching for endpoints with parameters
    for (const [pattern, config] of Object.entries(this.endpointConfig)) {
      if (this.matchEndpointPattern(endpoint, pattern)) {
        return config.enabled;
      }
    }

    // Default behavior based on signing mode
    return this.signingMode === 'auto';
  }

  /**
   * Match endpoint pattern (supports simple wildcard matching)
   */
  private matchEndpointPattern(endpoint: string, pattern: string): boolean {
    if (pattern === endpoint) {
      return true;
    }
    
    // Convert pattern to regex (simple wildcard support)
    const regexPattern = pattern
      .replace(/\*/g, '.*')
      .replace(/\//g, '\/')
      .replace(/:/g, '\\:');
      
    const regex = new RegExp(`^${regexPattern}$`);
    return regex.test(endpoint);
  }

  /**
   * Perform request signing with caching support
   */
  private async performSigning(
    request: SignableRequest,
    endpoint: string,
    customOptions?: SigningOptions
  ): Promise<RFC9421SignatureResult | null> {
    if (!this.signer) {
      return null;
    }

    const signingStartTime = Date.now();
    
    try {
      // Check cache if enabled
      if (this.enableSignatureCache) {
        const cacheKey = this.generateCacheKey(request, endpoint, customOptions);
        const cached = this.getFromCache(cacheKey);
        if (cached) {
          this.metrics.cacheHits++;
          return cached;
        }
        this.metrics.cacheMisses++;
      }

      // Merge signing options
      const endpointConfig = this.endpointConfig[endpoint];
      const signingOptions: SigningOptions = {
        ...this.defaultSigningOptions,
        ...endpointConfig?.options,
        ...customOptions
      };

      // Perform signing
      const result = await this.signer.signRequest(request, signingOptions);
      
      // Update signing time metrics (ensure minimum 1ms for accurate tracking)
      const signingTime = Math.max(1, Date.now() - signingStartTime);
      this.updateSigningTimeMetrics(signingTime);

      // Cache result if enabled
      if (this.enableSignatureCache) {
        const cacheKey = this.generateCacheKey(request, endpoint, customOptions);
        this.addToCache(cacheKey, result);
      }

      return result;
    } catch (error) {
      if (error instanceof SigningError) {
        throw error;
      }
      throw new SigningError(
        `Signing failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'SIGNING_FAILED',
        { originalError: error instanceof Error ? error.message : 'Unknown error' }
      );
    }
  }

  /**
   * Generate cache key for signature caching
   */
  private generateCacheKey(
    request: SignableRequest,
    endpoint: string,
    options?: SigningOptions
  ): string {
    const keyData = {
      method: request.method,
      url: request.url,
      headers: request.headers,
      body: request.body,
      endpoint,
      options
    };
    
    // Simple hash of the key data
    const str = JSON.stringify(keyData);
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return hash.toString(36);
  }

  /**
   * Get signature from cache
   */
  private getFromCache(key: string): RFC9421SignatureResult | null {
    const entry = this.signatureCache.get(key);
    if (!entry) {
      return null;
    }

    // Check TTL
    if (Date.now() - entry.timestamp > entry.ttl) {
      this.signatureCache.delete(key);
      return null;
    }

    return entry.signature;
  }

  /**
   * Add signature to cache
   */
  private addToCache(key: string, signature: RFC9421SignatureResult): void {
    // Enforce cache size limit
    if (this.signatureCache.size >= this.maxCacheSize) {
      // Remove oldest entry
      const firstKey = this.signatureCache.keys().next().value;
      if (firstKey) {
        this.signatureCache.delete(firstKey);
      }
    }

    this.signatureCache.set(key, {
      signature,
      timestamp: Date.now(),
      ttl: this.signatureCacheTtl
    });
  }

  /**
   * Update signing time metrics
   */
  private updateSigningTimeMetrics(signingTime: number): void {
    const { averageSigningTime, signedRequests } = this.metrics;
    
    // Calculate new average (signedRequests will be incremented after this call)
    const currentCount = signedRequests; // This will be the count before increment
    const newAverage = currentCount === 0
      ? signingTime
      : (averageSigningTime * currentCount + signingTime) / (currentCount + 1);
    
    this.metrics.averageSigningTime = newAverage;
  }

  /**
   * Test connection to the server
   */
  async testConnection(): Promise<{ connected: boolean; latency?: number; error?: string }> {
    try {
      const startTime = Date.now();
      
      // Try to get crypto status as a connectivity test
      // This will trigger signing if configured since it's a regular request
      await this.makeRequest('GET', '/status', undefined, 5000);
      
      const latency = Date.now() - startTime;
      return { connected: true, latency };
    } catch (error) {
      return {
        connected: false,
        error: error instanceof Error ? error.message : 'Unknown connection error'
      };
    }
  }

  /**
   * Register a public key with the server
   */
  async registerPublicKey(request: {
    clientId?: string;
    userId?: string;
    publicKey: string;
    keyName?: string;
    metadata?: Record<string, string>;
  }): Promise<{
    registrationId: string;
    clientId: string;
    publicKey: string;
    keyName?: string;
    registeredAt: string;
    status: string;
  }> {
    // Validate required fields
    if (!request.publicKey || typeof request.publicKey !== 'string') {
      throw new DataFoldServerError(
        'Public key is required',
        'INVALID_PUBLIC_KEY',
        400,
        { publicKey: request.publicKey }
      );
    }

    return this.makeRequest('POST', '/keys/register', {
      client_id: request.clientId?.trim(),
      user_id: request.userId?.trim(),
      public_key: request.publicKey.trim(),
      key_name: request.keyName?.trim(),
      metadata: request.metadata
    });
  }

  /**
   * Get public key registration status
   */
  async getPublicKeyStatus(clientId: string): Promise<{
    registrationId: string;
    clientId: string;
    publicKey: string;
    keyName?: string;
    registeredAt: string;
    status: string;
    lastUsed?: string;
  }> {
    // Validate clientId before proceeding
    if (!clientId || typeof clientId !== 'string' || clientId.trim() === '') {
      throw new DataFoldServerError(
        'Client ID is required and must be a non-empty string',
        'INVALID_CLIENT_ID',
        400,
        { clientId }
      );
    }
    
    return this.makeRequest('GET', `/keys/status/${encodeURIComponent(clientId.trim())}`);
  }

  /**
   * Verify a digital signature
   */
  async verifySignature(request: {
    clientId: string;
    message: string;
    signature: string;
    messageEncoding?: 'utf8' | 'hex' | 'base64';
    metadata?: Record<string, string>;
  }): Promise<{
    verified: boolean;
    clientId: string;
    publicKey: string;
    verifiedAt: string;
    messageHash: string;
  }> {
    // Validate required fields
    if (!request.clientId || typeof request.clientId !== 'string' || request.clientId.trim() === '') {
      throw new DataFoldServerError(
        'Client ID is required',
        'INVALID_CLIENT_ID',
        400,
        { clientId: request.clientId }
      );
    }

    if (!request.message || typeof request.message !== 'string') {
      throw new DataFoldServerError(
        'Message is required',
        'INVALID_MESSAGE',
        400,
        { message: request.message }
      );
    }

    if (!request.signature || typeof request.signature !== 'string') {
      throw new DataFoldServerError(
        'Signature is required',
        'INVALID_SIGNATURE',
        400,
        { signature: request.signature }
      );
    }

    return this.makeRequest('POST', '/signatures/verify', {
      client_id: request.clientId.trim(),
      message: request.message,
      signature: request.signature.trim(),
      message_encoding: request.messageEncoding || 'utf8',
      metadata: request.metadata
    });
  }
}

/**
 * Create a default HTTP client instance
 */
export function createHttpClient(config?: HttpClientConfig): DataFoldHttpClient {
  return new DataFoldHttpClient(config);
}

/**
 * Create HTTP client with fluent configuration
 */
export function createFluentHttpClient(): DataFoldHttpClient {
  return new DataFoldHttpClient();
}

/**
 * Create HTTP client with signing profile
 */
export function createSignedHttpClient(
  signingConfig: SigningConfig,
  httpConfig?: HttpClientConfig
): DataFoldHttpClient {
  const client = new DataFoldHttpClient(httpConfig);
  client.configureSigning(signingConfig);
  return client;
}

/**
 * Signing middleware for manual request signing
 */
export function createSigningMiddleware(
  signer: RFC9421Signer,
  options?: {
    enabled?: boolean;
    required?: boolean;
    signingOptions?: SigningOptions;
  }
): RequestInterceptor {
  const { enabled = true, required = false, signingOptions } = options || {};
  
  return async (request: SignableRequest, context: RequestContext): Promise<SignableRequest> => {
    if (!enabled) {
      return request;
    }

    try {
      const result = await signer.signRequest(request, signingOptions);
      return {
        ...request,
        headers: {
          ...request.headers,
          ...result.headers
        }
      };
    } catch (error) {
      if (required) {
        throw new DataFoldServerError(
          `Required signing middleware failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
          'SIGNING_MIDDLEWARE_FAILED',
          0,
          { originalError: error instanceof Error ? error.message : 'Unknown error' }
        );
      }
      
      // Log warning but continue
      console.warn('Signing middleware failed:', error);
      return request;
    }
  };
}

/**
 * Correlation ID middleware for request tracking
 */
export function createCorrelationMiddleware(options?: {
  headerName?: string;
  generateId?: () => string;
}): RequestInterceptor {
  const { headerName = 'x-correlation-id', generateId = () => `req_${Date.now()}_${Math.random().toString(36).substring(7)}` } = options || {};
  
  return async (request: SignableRequest): Promise<SignableRequest> => {
    if (!request.headers[headerName]) {
      request.headers[headerName] = generateId();
    }
    return request;
  };
}

/**
 * Request logging middleware
 */
export function createLoggingMiddleware(options?: {
  logRequests?: boolean;
  logResponses?: boolean;
  logLevel?: 'debug' | 'info' | 'warn' | 'error';
}): {
  requestInterceptor: RequestInterceptor;
  responseInterceptor: ResponseInterceptor;
} {
  const { logRequests = true, logResponses = true, logLevel = 'debug' } = options || {};
  
  const log = (level: string, message: string, data?: any) => {
    if (typeof console[level as keyof Console] === 'function') {
      (console[level as keyof Console] as any)(message, data);
    }
  };

  return {
    requestInterceptor: async (request: SignableRequest, context: RequestContext): Promise<SignableRequest> => {
      if (logRequests) {
        log(logLevel, 'HTTP Request', {
          method: request.method,
          url: request.url,
          headers: request.headers,
          attempt: context.attempt,
          endpoint: context.endpoint
        });
      }
      return request;
    },

    responseInterceptor: async (response: Response, context: RequestContext): Promise<Response> => {
      if (logResponses) {
        log(logLevel, 'HTTP Response', {
          status: response.status,
          statusText: response.statusText,
          headers: (() => {
            const headerObj: Record<string, string> = {};
            response.headers.forEach((value, key) => {
              headerObj[key] = value;
            });
            return headerObj;
          })(),
          attempt: context.attempt,
          endpoint: context.endpoint,
          duration: Date.now() - context.startTime
        });
      }
      return response;
    }
  };
}

/**
 * Performance monitoring middleware
 */
export function createPerformanceMiddleware(): {
  requestInterceptor: RequestInterceptor;
  responseInterceptor: ResponseInterceptor;
  getMetrics: () => {
    totalRequests: number;
    averageLatency: number;
    successRate: number;
    errorsByEndpoint: Record<string, number>;
  };
} {
  let totalRequests = 0;
  let totalLatency = 0;
  let successfulRequests = 0;
  const errorsByEndpoint: Record<string, number> = {};

  return {
    requestInterceptor: async (request: SignableRequest, context: RequestContext): Promise<SignableRequest> => {
      totalRequests++;
      return request;
    },

    responseInterceptor: async (response: Response, context: RequestContext): Promise<Response> => {
      const latency = Date.now() - context.startTime;
      totalLatency += latency;

      if (response.ok) {
        successfulRequests++;
      } else {
        errorsByEndpoint[context.endpoint] = (errorsByEndpoint[context.endpoint] || 0) + 1;
      }

      return response;
    },

    getMetrics: () => ({
      totalRequests,
      averageLatency: totalRequests > 0 ? totalLatency / totalRequests : 0,
      successRate: totalRequests > 0 ? successfulRequests / totalRequests : 0,
      errorsByEndpoint: { ...errorsByEndpoint }
    })
  };
}