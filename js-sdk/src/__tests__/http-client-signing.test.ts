/**
 * Tests for enhanced HTTP client automatic signature injection
 */

import { describe, it, expect, beforeAll, beforeEach, afterEach, jest } from '@jest/globals';
import { generateKeyPair } from '../crypto/ed25519.js';
import { 
  DataFoldHttpClient,
  createHttpClient,
  createFluentHttpClient,
  createSignedHttpClient,
  HttpClientConfig,
  SigningMode
} from '../server/http-client.js';
import { 
  RFC9421Signer,
  createSigningConfig,
  SigningConfig
} from '../signing/index.js';

// Mock fetch for testing
const mockFetch = jest.fn() as jest.MockedFunction<typeof fetch>;
(global as any).fetch = mockFetch;

describe('Enhanced HTTP Client Signing', () => {
  let keyPair: any;
  let signingConfig: SigningConfig;
  let signer: RFC9421Signer;

  beforeAll(async () => {
    // Generate test key pair
    keyPair = await generateKeyPair();
    
    signingConfig = createSigningConfig()
      .algorithm('ed25519')
      .keyId('test-key-001')
      .privateKey(keyPair.privateKey)
      .profile('standard')
      .build();
    
    signer = new RFC9421Signer(signingConfig);
  });

  beforeEach(() => {
    mockFetch.mockClear();
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('Client Configuration', () => {
    it('should create client with default configuration', () => {
      const client = createHttpClient();
      
      expect(client.isSigningEnabled()).toBe(false);
      expect(client.getSigningMetrics().totalRequests).toBe(0);
    });

    it('should create client with enhanced configuration', () => {
      const config: HttpClientConfig = {
        signingMode: 'auto',
        enableSignatureCache: true,
        signatureCacheTtl: 60000,
        debugLogging: true,
        endpointConfig: {
          '/keys/register': { enabled: true, required: true },
          '/status': { enabled: false }
        }
      };

      const client = createHttpClient(config);
      client.configureSigning(signingConfig);
      
      expect(client.isSigningEnabled()).toBe(true);
    });

    it('should create fluent client with chained configuration', () => {
      const client = createFluentHttpClient()
        .configureSigning(signingConfig)
        .setSigningMode('auto')
        .enableDebugLogging(true)
        .configureEndpointSigning('/keys/register', { enabled: true, required: true });
      
      expect(client.isSigningEnabled()).toBe(true);
    });

    it('should create pre-configured signed client', () => {
      const client = createSignedHttpClient(signingConfig, {
        signingMode: 'auto',
        debugLogging: true
      });
      
      expect(client.isSigningEnabled()).toBe(true);
    });
  });

  describe('Signing Modes', () => {
    it('should respect disabled signing mode', async () => {
      const mockResponse = new Response(
        JSON.stringify({ success: true, data: { connected: true } }),
        { status: 200, headers: { 'content-type': 'application/json' } }
      );
      mockFetch.mockResolvedValueOnce(mockResponse);

      const client = createHttpClient({ signingMode: 'disabled' });
      client.configureSigning(signingConfig);
      
      await client.testConnection();
      
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/crypto/status'),
        expect.objectContaining({
          headers: expect.not.objectContaining({
            signature: expect.anything(),
            'signature-input': expect.anything()
          })
        })
      );
    });

    it('should auto-sign requests in auto mode', async () => {
      const mockResponse = new Response(
        JSON.stringify({ success: true, data: { connected: true } }),
        { status: 200, headers: { 'content-type': 'application/json' } }
      );
      mockFetch.mockResolvedValueOnce(mockResponse);

      const client = createHttpClient({ signingMode: 'auto' });
      client.configureSigning(signingConfig);
      
      await client.testConnection();
      
      const lastCall = mockFetch.mock.calls[mockFetch.mock.calls.length - 1];
      const requestInit = lastCall[1] as RequestInit;
      const headers = requestInit.headers as Record<string, string>;
      
      expect(headers).toHaveProperty('signature');
      expect(headers).toHaveProperty('signature-input');
    });

    it('should respect endpoint-specific configuration', async () => {
      const mockResponse = new Response(
        JSON.stringify({ success: true, data: { connected: true } }),
        { status: 200, headers: { 'content-type': 'application/json' } }
      );
      mockFetch.mockResolvedValueOnce(mockResponse);

      const client = createHttpClient({
        signingMode: 'auto',
        endpointConfig: {
          '/status': { enabled: false }
        }
      });
      client.configureSigning(signingConfig);
      
      await client.testConnection();
      
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/crypto/status'),
        expect.objectContaining({
          headers: expect.not.objectContaining({
            signature: expect.anything(),
            'signature-input': expect.anything()
          })
        })
      );
    });
  });

  describe('Signature Caching', () => {
    it('should cache signatures when enabled', async () => {
      const client = createHttpClient({
        signingMode: 'auto',
        enableSignatureCache: true,
        signatureCacheTtl: 60000
      });
      client.configureSigning(signingConfig);

      const mockResponse1 = new Response(
        JSON.stringify({ success: true, data: { connected: true } }),
        { status: 200, headers: { 'content-type': 'application/json' } }
      );
      const mockResponse2 = new Response(
        JSON.stringify({ success: true, data: { connected: true } }),
        { status: 200, headers: { 'content-type': 'application/json' } }
      );
      
      mockFetch.mockResolvedValueOnce(mockResponse1);
      mockFetch.mockResolvedValueOnce(mockResponse2);

      await client.testConnection();
      await client.testConnection();

      const metrics = client.getSigningMetrics();
      expect(metrics.totalRequests).toBe(2);
      expect(metrics.cacheHits).toBe(1);
      expect(metrics.cacheMisses).toBe(1);
    });

    it('should clear cache on demand', async () => {
      const client = createHttpClient({
        signingMode: 'auto',
        enableSignatureCache: true
      });
      client.configureSigning(signingConfig);

      const mockResponse1 = new Response(
        JSON.stringify({ success: true, data: { connected: true } }),
        { status: 200, headers: { 'content-type': 'application/json' } }
      );
      const mockResponse2 = new Response(
        JSON.stringify({ success: true, data: { connected: true } }),
        { status: 200, headers: { 'content-type': 'application/json' } }
      );

      mockFetch.mockResolvedValueOnce(mockResponse1);
      await client.testConnection();
      
      client.clearSignatureCache();
      
      mockFetch.mockResolvedValueOnce(mockResponse2);
      await client.testConnection();

      const metrics = client.getSigningMetrics();
      expect(metrics.cacheHits).toBe(0);
      expect(metrics.cacheMisses).toBe(2);
    });
  });

  describe('Metrics and Monitoring', () => {
    it('should track signing metrics', async () => {
      const client = createHttpClient({ signingMode: 'auto' });
      client.configureSigning(signingConfig);

      const mockResponse = new Response(
        JSON.stringify({ success: true, data: { connected: true } }),
        { status: 200, headers: { 'content-type': 'application/json' } }
      );
      mockFetch.mockResolvedValueOnce(mockResponse);

      await client.testConnection();

      const metrics = client.getSigningMetrics();
      expect(metrics.totalRequests).toBe(1);
      expect(metrics.signedRequests).toBe(1);
      expect(metrics.signingFailures).toBe(0);
      expect(metrics.averageSigningTime).toBeGreaterThan(0);
    });

    it('should reset metrics on demand', async () => {
      const client = createHttpClient({ signingMode: 'auto' });
      client.configureSigning(signingConfig);

      const mockResponse = new Response(
        JSON.stringify({ success: true, data: { connected: true } }),
        { status: 200, headers: { 'content-type': 'application/json' } }
      );
      mockFetch.mockResolvedValueOnce(mockResponse);

      await client.testConnection();
      client.resetSigningMetrics();

      const metrics = client.getSigningMetrics();
      expect(metrics.totalRequests).toBe(0);
      expect(metrics.signedRequests).toBe(0);
      expect(metrics.signingFailures).toBe(0);
      expect(metrics.averageSigningTime).toBe(0);
    });
  });

  describe('Fluent API', () => {
    it('should support method chaining', () => {
      const client = createFluentHttpClient();
      
      const result = client
        .configureSigning(signingConfig)
        .setSigningMode('auto')
        .enableDebugLogging(true)
        .configureEndpointSigning('/test', { enabled: true })
        .resetSigningMetrics()
        .clearSignatureCache();
      
      expect(result).toBe(client); // Should return the same instance for chaining
      expect(client.isSigningEnabled()).toBe(true);
    });
  });

  describe('Error Handling', () => {
    it('should handle signing failures gracefully when not required', async () => {
      // Create client with invalid signing config that will fail
      const invalidConfig = {
        ...signingConfig,
        privateKey: new Uint8Array(16).fill(0) // Invalid key length (should be 32)
      };
      
      const client = createHttpClient({
        signingMode: 'auto',
        debugLogging: false // Disable debug logging for cleaner test output
      });
      
      // This should not throw due to the invalid key - it will be caught during signing
      try {
        client.configureSigning(invalidConfig);
      } catch (error) {
        // Expected to throw during configuration with invalid key
      }
      
      // Re-configure with a valid key but make the request signing fail by corrupting the signer
      client.configureSigning(signingConfig);
      
      // Mock the signer to throw an error during signing
      const originalSigner = (client as any).signer;
      if (originalSigner) {
        (client as any).signer = {
          ...originalSigner,
          signRequest: jest.fn().mockImplementation(() => {
            throw new Error('Mocked signing failure');
          })
        };
      }

      const mockResponse = new Response(
        JSON.stringify({ success: true, data: { connected: true } }),
        { status: 200, headers: { 'content-type': 'application/json' } }
      );
      mockFetch.mockResolvedValueOnce(mockResponse);

      // Should not throw, should continue without signing
      await expect(client.testConnection()).resolves.toBeDefined();

      const metrics = client.getSigningMetrics();
      expect(metrics.signingFailures).toBe(1);
      expect(mockFetch).toHaveBeenCalled();
    });
  });

  describe('Configuration Updates', () => {
    it('should allow updating configuration after creation', () => {
      const client = createHttpClient();
      
      expect(client.isSigningEnabled()).toBe(false);
      
      client.updateConfig({
        signingMode: 'auto',
        debugLogging: true
      });
      
      client.configureSigning(signingConfig);
      
      expect(client.isSigningEnabled()).toBe(true);
    });

    it('should allow toggling signing modes', () => {
      const client = createHttpClient({ signingMode: 'auto' });
      client.configureSigning(signingConfig);
      
      expect(client.isSigningEnabled()).toBe(true);
      
      client.setSigningMode('disabled');
      expect(client.isSigningEnabled()).toBe(false);
      
      client.setSigningMode('auto');
      expect(client.isSigningEnabled()).toBe(true);
    });
  });
});