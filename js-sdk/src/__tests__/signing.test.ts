/**
 * Tests for RFC 9421 HTTP Message Signatures implementation
 */

import { 
  RFC9421Signer,
  createSigner,
  signRequest,
  createSigningConfig,
  createFromProfile,
  SECURITY_PROFILES,
  SignableRequest,
  SigningError,
  buildCanonicalMessage,
  generateNonce,
  generateTimestamp,
  validateSigningPrivateKey,
  calculateContentDigest
} from '../signing';
import { generateKeyPair } from '../crypto/ed25519';

describe('RFC 9421 Signing', () => {
  let keyPair: any;
  let signingConfig: any;

  beforeAll(async () => {
    // Generate test key pair
    keyPair = await generateKeyPair();
    
    signingConfig = createSigningConfig()
      .algorithm('ed25519')
      .keyId('test-key-001')
      .privateKey(keyPair.privateKey)
      .profile('standard')
      .build();
  });

  describe('SigningConfig', () => {
    it('should create valid signing configuration', () => {
      expect(signingConfig.algorithm).toBe('ed25519');
      expect(signingConfig.keyId).toBe('test-key-001');
      expect(signingConfig.privateKey).toEqual(keyPair.privateKey);
      expect(signingConfig.components).toBeDefined();
    });

    it('should create configuration from security profile', () => {
      const config = createFromProfile('strict', 'test-key-002', keyPair.privateKey);
      
      expect(config.keyId).toBe('test-key-002');
      expect(config.components).toEqual(SECURITY_PROFILES.strict.components);
    });

    it('should validate configuration', () => {
      expect(() => {
        createSigningConfig()
          .algorithm('ed25519')
          .keyId('')
          .privateKey(keyPair.privateKey)
          .build();
      }).toThrow(SigningError);
    });
  });

  describe('RFC9421Signer', () => {
    let signer: RFC9421Signer;

    beforeEach(() => {
      signer = createSigner(signingConfig);
    });

    it('should sign GET request', async () => {
      const request: SignableRequest = {
        method: 'GET',
        url: 'https://api.datafold.com/api/crypto/keys/status/test-client',
        headers: {
          'user-agent': 'DataFold-JS-SDK/0.1.0'
        }
      };

      const result = await signer.signRequest(request);

      expect(result.signatureInput).toContain('sig1=');
      expect(result.signature).toContain('sig1=:');
      expect(result.headers).toHaveProperty('signature-input');
      expect(result.headers).toHaveProperty('signature');
      expect(result.canonicalMessage).toContain('"@method": GET');
      expect(result.canonicalMessage).toContain('"@target-uri": /api/crypto/keys/status/test-client');
    });

    it('should sign POST request with body', async () => {
      const requestBody = { clientId: 'test-client', data: 'test' };
      const request: SignableRequest = {
        method: 'POST',
        url: 'https://api.datafold.com/api/crypto/keys/register',
        headers: {
          'content-type': 'application/json'
        },
        body: JSON.stringify(requestBody)
      };

      const result = await signer.signRequest(request);

      expect(result.signatureInput).toContain('sig1=');
      expect(result.signature).toContain('sig1=:');
      expect(result.headers).toHaveProperty('content-digest');
      expect(result.canonicalMessage).toContain('"@method": POST');
      expect(result.canonicalMessage).toContain('"content-type": application/json');
      expect(result.canonicalMessage).toContain('"content-digest":');
    });

    it('should include all required signature components', async () => {
      const request: SignableRequest = {
        method: 'POST',
        url: 'https://api.datafold.com/api/crypto/test',
        headers: {
          'content-type': 'application/json'
        },
        body: '{"test": true}'
      };

      const result = await signer.signRequest(request);

      // Check that canonical message includes required components
      expect(result.canonicalMessage).toContain('"@method": POST');
      expect(result.canonicalMessage).toContain('"@target-uri": /api/crypto/test');
      expect(result.canonicalMessage).toContain('"content-type": application/json');
      expect(result.canonicalMessage).toContain('"content-digest":');
      expect(result.canonicalMessage).toContain('"@signature-params":');
    });

    it('should generate valid signature parameters', async () => {
      const request: SignableRequest = {
        method: 'GET',
        url: 'https://api.datafold.com/api/crypto/status',
        headers: {}
      };

      const result = await signer.signRequest(request);

      // Extract parameters from signature input
      const inputMatch = result.signatureInput.match(/sig1=\(([^)]+)\);(.+)$/);
      expect(inputMatch).toBeTruthy();

      const params = inputMatch![2];
      expect(params).toContain('created=');
      expect(params).toContain('keyid="test-key-001"');
      expect(params).toContain('alg="ed25519"');
      expect(params).toContain('nonce=');
    });

    it('should handle custom signing options', async () => {
      const customNonce = generateNonce();
      const customTimestamp = generateTimestamp();

      const request: SignableRequest = {
        method: 'GET',
        url: 'https://api.datafold.com/api/crypto/test',
        headers: {}
      };

      const result = await signer.signRequest(request, {
        nonce: customNonce,
        timestamp: customTimestamp
      });

      expect(result.signatureInput).toContain(`nonce="${customNonce}"`);
      expect(result.signatureInput).toContain(`created=${customTimestamp}`);
    });
  });

  describe('Utility Functions', () => {
    it('should generate valid nonces', () => {
      const nonce1 = generateNonce();
      const nonce2 = generateNonce();

      expect(typeof nonce1).toBe('string');
      expect(typeof nonce2).toBe('string');
      expect(nonce1).not.toBe(nonce2);
      expect(nonce1).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i);
    });

    it('should generate valid timestamps', () => {
      const timestamp = generateTimestamp();
      const now = Math.floor(Date.now() / 1000);

      expect(typeof timestamp).toBe('number');
      expect(timestamp).toBeCloseTo(now, 0);
    });

    it('should validate private keys', () => {
      expect(validateSigningPrivateKey(keyPair.privateKey)).toBe(true);
      expect(validateSigningPrivateKey(new Uint8Array(31))).toBe(false);
      expect(validateSigningPrivateKey(new Uint8Array(33))).toBe(false);
    });

    it('should calculate content digests', async () => {
      const content = 'Hello, World!';
      const digest = await calculateContentDigest(content, 'sha-256');

      expect(digest.algorithm).toBe('sha-256');
      expect(digest.digest).toBeTruthy();
      expect(digest.headerValue).toMatch(/^sha-256=:[A-Za-z0-9+/]+=*:$/);
    });
  });

  describe('Error Handling', () => {
    it('should throw error for invalid algorithm', () => {
      expect(() => {
        createSigningConfig()
          .algorithm('rsa' as any)
          .keyId('test')
          .privateKey(keyPair.privateKey)
          .build();
      }).toThrow(SigningError);
    });

    it('should throw error for invalid private key', () => {
      expect(() => {
        createSigningConfig()
          .algorithm('ed25519')
          .keyId('test')
          .privateKey(new Uint8Array(31))
          .build();
      }).toThrow(SigningError);
    });

    it('should handle signing failures gracefully', async () => {
      // Create signer with invalid private key (all zeros)
      const badConfig = { ...signingConfig, privateKey: new Uint8Array(32) };
      
      // Mock the signAsync to fail for all-zeros private key
      const originalSignAsync = require('@noble/ed25519').signAsync;
      require('@noble/ed25519').signAsync = jest.fn().mockImplementation(async (message: Uint8Array, privateKey: Uint8Array) => {
        // Check if private key is all zeros (invalid)
        if (privateKey.every(byte => byte === 0)) {
          throw new Error('Invalid private key: all zeros');
        }
        return originalSignAsync(message, privateKey);
      });
      
      const signer = new RFC9421Signer(badConfig);

      const request: SignableRequest = {
        method: 'GET',
        url: 'https://api.datafold.com/test',
        headers: {}
      };

      await expect(signer.signRequest(request)).rejects.toThrow();
      
      // Restore original mock
      require('@noble/ed25519').signAsync = originalSignAsync;
    });
  });

  describe('Performance', () => {
    it('should sign requests quickly', async () => {
      const signer = createSigner(signingConfig);
      const request: SignableRequest = {
        method: 'GET',
        url: 'https://api.datafold.com/api/crypto/test',
        headers: {}
      };

      const start = performance.now();
      await signer.signRequest(request);
      const elapsed = performance.now() - start;

      // Should complete in less than 10ms as per requirements
      expect(elapsed).toBeLessThan(10);
    });

    it('should handle multiple requests efficiently', async () => {
      const signer = createSigner(signingConfig);
      const requests = Array(10).fill(null).map((_, i) => ({
        method: 'GET' as const,
        url: `https://api.datafold.com/api/crypto/test${i}`,
        headers: {}
      }));

      const start = performance.now();
      const results = await signer.signRequests(requests);
      const elapsed = performance.now() - start;

      expect(results).toHaveLength(10);
      expect(elapsed).toBeLessThan(100); // Should complete all in reasonable time
    });
  });
});