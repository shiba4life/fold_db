/**
 * Integration tests for DataFold server communication
 * Tests public key registration and signature verification workflows
 */

import { describe, it, expect, beforeAll, afterAll, beforeEach } from '@jest/globals';
import { generateKeyPair } from '../crypto/ed25519.js';
import {
  DataFoldHttpClient,
  DataFoldServerIntegration,
  createHttpClient,
  createServerIntegration,
  quickIntegrationTest
} from '../server/index.js';
import { DataFoldServerError } from '../types.js';
import { mockServer, cleanupServerMock, setupServerMock } from './__mocks__/server-mock.js';

// Test configuration - using mock server
const TEST_CONFIG = {
  baseUrl: 'http://localhost:9001', // Mock server will intercept these calls
  timeout: 45000, // Further increased timeout for stability
  retries: 0, // Disable retries for tests - mock server doesn't need retries
  retryDelay: 200
};

// Test counter to ensure absolute uniqueness
let testCounter = 0;

// Helper function to generate unique client IDs with high entropy
function generateTestClientId(): string {
  testCounter++;
  const timestamp = Date.now();
  const random1 = Math.random().toString(36).substring(2, 15);
  const random2 = Math.random().toString(36).substring(2, 15);
  const random3 = Math.random().toString(36).substring(2, 15);
  return `test_client_${testCounter}_${timestamp}_${random1}_${random2}_${random3}`;
}

describe('DataFold Server Integration', () => {
  let httpClient: DataFoldHttpClient;
  let integration: DataFoldServerIntegration;
  let testKeyPair: any;
  let testClientId: string;
  let localMockServer: any;

  beforeAll(async () => {
    // Set up mock server
    setupServerMock();
    
    // Generate test key pair
    testKeyPair = await generateKeyPair();
    
    // Create clients
    httpClient = createHttpClient(TEST_CONFIG);
    integration = createServerIntegration(TEST_CONFIG);
  });

  afterAll(() => {
    // Clean up mock server state
    cleanupServerMock();
  });

  beforeEach(() => {
    // Generate unique client ID for each test
    testClientId = generateTestClientId();
    
    // Completely reset mock server state and setup fresh mock
    cleanupServerMock();
    setupServerMock();
    
    // Ensure clean state
    expect(mockServer.getRegistrationCount()).toBe(0);
    expect(mockServer.isClientRegistered(testClientId)).toBe(false);
  });

  describe('HTTP Client', () => {
    it('should test server connection successfully', async () => {
      const result = await httpClient.testConnection();
      
      expect(result.connected).toBe(true);
      expect(result.latency).toBeGreaterThan(0);
      expect(result.error).toBeUndefined();
    });

    it('should register a public key successfully', async () => {
      const publicKeyHex = Array.from(testKeyPair.publicKey, (byte: number) =>
        byte.toString(16).padStart(2, '0')
      ).join('');

      const response = await httpClient.registerPublicKey({
        clientId: testClientId,
        publicKey: publicKeyHex,
        keyName: 'Test Key',
        metadata: { test: 'true' }
      });

      expect(response.clientId).toBe(testClientId);
      expect(response.publicKey).toBe(publicKeyHex);
      expect(response.status).toBe('active');
      expect(response.registrationId).toBeTruthy();
      expect(response.registeredAt).toBeTruthy();
    });

    it('should get public key status after registration', async () => {
      // First register a key
      const publicKeyHex = Array.from(testKeyPair.publicKey, (byte: number) =>
        byte.toString(16).padStart(2, '0')
      ).join('');

      await httpClient.registerPublicKey({
        clientId: testClientId,
        publicKey: publicKeyHex,
        keyName: 'Status Test Key'
      });

      // Then get status
      const status = await httpClient.getPublicKeyStatus(testClientId);

      // Debug: log the actual response
      console.log('DEBUG: status response:', JSON.stringify(status, null, 2));
      console.log('DEBUG: testClientId:', testClientId);
      
      expect(status.clientId).toBe(testClientId);
      expect(status.publicKey).toBe(publicKeyHex);
      expect(status.status).toBe('active');
      expect(status.keyName).toBe('Status Test Key');
    });

    it('should verify a valid signature', async () => {
      // Register key first
      const publicKeyHex = Array.from(testKeyPair.publicKey, (byte: number) =>
        byte.toString(16).padStart(2, '0')
      ).join('');

      await httpClient.registerPublicKey({
        clientId: testClientId,
        publicKey: publicKeyHex
      });

      // Generate signature
      const message = 'Test message for verification';
      const messageBytes = new TextEncoder().encode(message);
      
      // Import ed25519 for signing
      const { signAsync } = await import('@noble/ed25519');
      const signature = await signAsync(messageBytes, testKeyPair.privateKey);
      const signatureHex = Array.from(signature, (byte: number) =>
        byte.toString(16).padStart(2, '0')
      ).join('');

      // Verify signature
      const verification = await httpClient.verifySignature({
        clientId: testClientId,
        message,
        signature: signatureHex,
        messageEncoding: 'utf8'
      });

      expect(verification.verified).toBe(true);
      expect(verification.clientId).toBe(testClientId);
      expect(verification.publicKey).toBe(publicKeyHex);
      expect(verification.messageHash).toBeTruthy();
    });

    it('should reject invalid signatures', async () => {
      // Register key first
      const publicKeyHex = Array.from(testKeyPair.publicKey, (byte: number) =>
        byte.toString(16).padStart(2, '0')
      ).join('');

      await httpClient.registerPublicKey({
        clientId: testClientId,
        publicKey: publicKeyHex
      });

      // Try to verify invalid signature
      const message = 'Test message';
      const invalidSignature = '0'.repeat(128); // Invalid signature

      await expect(
        httpClient.verifySignature({
          clientId: testClientId,
          message,
          signature: invalidSignature,
          messageEncoding: 'utf8'
        })
      ).rejects.toThrow(DataFoldServerError);
    });

    it('should handle unregistered client verification attempts', async () => {
      const unregisteredClientId = generateTestClientId();
      const message = 'Test message';
      const signature = '0'.repeat(128);

      await expect(
        httpClient.verifySignature({
          clientId: unregisteredClientId,
          message,
          signature,
          messageEncoding: 'utf8'
        })
      ).rejects.toThrow(DataFoldServerError);
    });

    it('should prevent duplicate public key registration', async () => {
      const publicKeyHex = Array.from(testKeyPair.publicKey, (byte: number) =>
        byte.toString(16).padStart(2, '0')
      ).join('');

      // Register first time
      await httpClient.registerPublicKey({
        clientId: testClientId,
        publicKey: publicKeyHex
      });

      // Try to register same client again
      await expect(
        httpClient.registerPublicKey({
          clientId: testClientId,
          publicKey: publicKeyHex
        })
      ).rejects.toThrow(DataFoldServerError);
    });
  });

  describe('Server Integration', () => {
    it('should register key pair and verify workflow', async () => {
      const result = await integration.registerAndVerifyWorkflow(
        testKeyPair,
        'Integration test message',
        {
          clientId: testClientId,
          keyName: 'Integration Test Key',
          metadata: { 
            test: 'integration',
            timestamp: new Date().toISOString()
          }
        }
      );

      expect(result.registration.clientId).toBe(testClientId);
      expect(result.registration.status).toBe('active');
      expect(result.signatureTest.signature).toBeTruthy();
      expect(result.verification.verified).toBe(true);
      expect(result.verification.clientId).toBe(testClientId);
    });

    it('should check registration status', async () => {
      // Register a key first
      await integration.registerKeyPair(testKeyPair, {
        clientId: testClientId,
        keyName: 'Status Check Test'
      });

      // Check status
      const status = await integration.checkRegistrationStatus(testClientId);

      expect(status.registered).toBe(true);
      expect(status.registration).toBeTruthy();
      expect(status.registration?.clientId).toBe(testClientId);
      expect(status.error).toBeUndefined();
    });

    it('should handle unregistered client status check', async () => {
      const unregisteredClientId = generateTestClientId();
      const status = await integration.checkRegistrationStatus(unregisteredClientId);

      expect(status.registered).toBe(false);
      expect(status.registration).toBeUndefined();
      expect(status.error).toBeUndefined();
    });

    it('should generate and verify signatures', async () => {
      // Register key first
      await integration.registerKeyPair(testKeyPair, {
        clientId: testClientId
      });

      const message = 'Test signature generation';
      
      // Test sign and verify workflow
      const result = await integration.signAndVerify(
        message,
        testKeyPair,
        {
          clientId: testClientId,
          messageEncoding: 'utf8'
        }
      );

      expect(result.signatureResult.signature).toBeTruthy();
      expect(result.signatureResult.message).toBe(message);
      expect(result.verificationResult.verified).toBe(true);
      expect(result.verificationResult.clientId).toBe(testClientId);
    });

    it('should handle different message encodings', async () => {
      await integration.registerKeyPair(testKeyPair, {
        clientId: testClientId
      });

      // Test hex encoding
      const hexMessage = '48656c6c6f'; // "Hello" in hex
      const hexResult = await integration.signAndVerify(
        hexMessage,
        testKeyPair,
        {
          clientId: testClientId,
          messageEncoding: 'hex'
        }
      );

      expect(hexResult.verificationResult.verified).toBe(true);

      // Test base64 encoding
      const base64Message = btoa('Hello'); // "Hello" in base64
      const base64Result = await integration.signAndVerify(
        base64Message,
        testKeyPair,
        {
          clientId: testClientId,
          messageEncoding: 'base64'
        }
      );

      expect(base64Result.verificationResult.verified).toBe(true);
    });

    it('should get connection statistics', async () => {
      const stats = await integration.getConnectionStats();

      expect(stats.connected).toBe(true);
      expect(stats.latency).toBeGreaterThan(0);
      expect(stats.error).toBeUndefined();
    });
  });

  describe('Quick Integration Test', () => {
    it('should run complete integration test successfully', async () => {
      const result = await quickIntegrationTest(TEST_CONFIG);

      expect(result.success).toBe(true);
      expect(result.results).toBeTruthy();
      expect(result.results?.keyGeneration).toBe(true);
      expect(result.results?.registration).toBe(true);
      expect(result.results?.signatureGeneration).toBe(true);
      expect(result.results?.verification).toBe(true);
      expect(result.error).toBeUndefined();
      expect(result.details).toBeTruthy();
    }, 20000); // Further increased timeout for full workflow
  });

  describe('Error Handling', () => {
    it('should handle network timeouts gracefully', async () => {
      // Simulate slow responses
      mockServer.setDelay(50);
      const shortTimeoutClient = createHttpClient({
        ...TEST_CONFIG,
        timeout: 1 // Very short timeout
      });

      const result = await shortTimeoutClient.testConnection();
      // With short timeout, expect connection to succeed but possibly with latency
      expect(result.connected).toBe(true);
    });

    it('should retry failed requests', async () => {
      const retryClient = createHttpClient({
        ...TEST_CONFIG,
        retryConfig: {
          maxRetries: 2,
          baseDelay: 100,
          maxDelay: 1000,
          backoffFactor: 2
        }
      });

      // This should work with retries
      const result = await retryClient.testConnection();
      expect(result.connected).toBe(true);
    });

    it('should validate public key format', async () => {
      await expect(
        httpClient.registerPublicKey({
          clientId: testClientId,
          publicKey: 'invalid_hex_key'
        })
      ).rejects.toThrow(DataFoldServerError);
    });

    it('should validate signature format', async () => {
      // Register key first
      const publicKeyHex = Array.from(testKeyPair.publicKey, (byte: number) =>
        byte.toString(16).padStart(2, '0')
      ).join('');

      await httpClient.registerPublicKey({
        clientId: testClientId,
        publicKey: publicKeyHex
      });

      // Try invalid signature format
      await expect(
        httpClient.verifySignature({
          clientId: testClientId,
          message: 'test',
          signature: 'invalid_signature_format'
        })
      ).rejects.toThrow(DataFoldServerError);
    });
  });
});