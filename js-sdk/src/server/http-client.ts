/**
 * HTTP client for DataFold server communication
 * Handles public key registration and signature verification
 */

import { DataFoldServerError, ServerConnectionConfig, RetryConfig } from '../types.js';

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
 * HTTP client for DataFold server communication
 */
export class DataFoldHttpClient {
  private config: ServerConnectionConfig;
  private retryConfig: RetryConfig;

  constructor(config: Partial<ServerConnectionConfig> = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.retryConfig = config.retryConfig || DEFAULT_RETRY_CONFIG;
  }

  /**
   * Update client configuration
   */
  updateConfig(config: Partial<ServerConnectionConfig>): void {
    this.config = { ...this.config, ...config };
    if (config.retryConfig) {
      this.retryConfig = config.retryConfig;
    }
  }

  /**
   * Make an HTTP request with retry logic
   */
  private async makeRequest<T>(
    method: 'GET' | 'POST' | 'PUT' | 'DELETE',
    endpoint: string,
    body?: any,
    customTimeout?: number
  ): Promise<T> {
    const url = `${this.config.baseUrl}/api/crypto${endpoint}`;
    const timeout = customTimeout || this.config.timeout;

    let lastError: Error | null = null;

    for (let attempt = 0; attempt <= this.retryConfig.maxRetries; attempt++) {
      try {
        // Create abort controller for timeout
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), timeout);

        const requestInit: RequestInit = {
          method,
          headers: {
            'Content-Type': 'application/json',
            'User-Agent': 'DataFold-JS-SDK/0.1.0'
          },
          signal: controller.signal
        };

        if (body && method !== 'GET') {
          requestInit.body = JSON.stringify(body);
        }

        const response = await fetch(url, requestInit);
        clearTimeout(timeoutId);

        // Parse response
        const responseData: ApiResponse<T> = await response.json();

        // Handle successful response
        if (response.ok && responseData.success) {
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

        // Don't retry on certain errors
        if (error instanceof DataFoldServerError) {
          const nonRetryableCodes = [
            'INVALID_PUBLIC_KEY',
            'INVALID_CLIENT_ID',
            'INVALID_MESSAGE',
            'INVALID_SIGNATURE',
            'INVALID_ENCODING',
            'CLIENT_ALREADY_REGISTERED',
            'DUPLICATE_PUBLIC_KEY',
            'CLIENT_NOT_FOUND',
            'REGISTRATION_NOT_FOUND',
            'KEY_NOT_ACTIVE',
            'SIGNATURE_VERIFICATION_FAILED'
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
   * Test connection to the server
   */
  async testConnection(): Promise<{ connected: boolean; latency?: number; error?: string }> {
    try {
      const startTime = Date.now();
      
      // Try to get crypto status as a connectivity test
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
    return this.makeRequest('POST', '/keys/register', {
      client_id: request.clientId,
      user_id: request.userId,
      public_key: request.publicKey,
      key_name: request.keyName,
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
    return this.makeRequest('GET', `/keys/status/${encodeURIComponent(clientId)}`);
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
    return this.makeRequest('POST', '/signatures/verify', {
      client_id: request.clientId,
      message: request.message,
      signature: request.signature,
      message_encoding: request.messageEncoding || 'utf8',
      metadata: request.metadata
    });
  }
}

/**
 * Create a default HTTP client instance
 */
export function createHttpClient(config?: Partial<ServerConnectionConfig>): DataFoldHttpClient {
  return new DataFoldHttpClient(config);
}