/**
 * DataFold Server Integration
 * High-level API for client-server communication with key management
 */

import * as ed25519 from '@noble/ed25519';
import { 
  Ed25519KeyPair, 
  DataFoldServerError, 
  ServerConnectionConfig,
  PublicKeyRegistrationRequest,
  PublicKeyRegistrationResponse,
  PublicKeyStatusResponse,
  SignatureVerificationRequest,
  SignatureVerificationResponse,
  ServerIntegrationInterface
} from '../types.js';
import { DataFoldHttpClient, createHttpClient } from './http-client.js';
import { formatKey, parseKey } from '../crypto/ed25519.js';

/**
 * Configuration for server integration
 */
export interface ServerIntegrationConfig extends Partial<ServerConnectionConfig> {
  /** Default client ID to use for operations */
  defaultClientId?: string;
  /** Default user ID to use for operations */
  defaultUserId?: string;
  /** Whether to automatically retry failed operations */
  autoRetry?: boolean;
}

/**
 * Registration status information
 */
export interface RegistrationStatus {
  /** Whether the client is registered */
  registered: boolean;
  /** Registration details if registered */
  registration?: PublicKeyStatusResponse;
  /** Error information if registration check failed */
  error?: string;
}

/**
 * Signature generation result
 */
export interface SignatureResult {
  /** The generated signature in hex format */
  signature: string;
  /** The message that was signed */
  message: string;
  /** Message encoding used */
  messageEncoding: 'utf8' | 'hex' | 'base64';
  /** Timestamp when signature was generated */
  timestamp: string;
}

/**
 * High-level server integration client
 */
export class DataFoldServerIntegration implements ServerIntegrationInterface {
  private httpClient: DataFoldHttpClient;
  private config: ServerIntegrationConfig;

  constructor(config: ServerIntegrationConfig = {}) {
    this.config = config;
    this.httpClient = createHttpClient(config);
  }

  /**
   * Update integration configuration
   */
  updateConfig(config: Partial<ServerIntegrationConfig>): void {
    this.config = { ...this.config, ...config };
    this.httpClient.updateConfig(config);
  }

  /**
   * Test connection to the server
   */
  async testConnection(): Promise<{ connected: boolean; latency?: number; error?: string }> {
    return this.httpClient.testConnection();
  }

  /**
   * Register a key pair with the server
   */
  async registerKeyPair(
    keyPair: Ed25519KeyPair,
    options: {
      clientId?: string;
      userId?: string;
      keyName?: string;
      metadata?: Record<string, string>;
    } = {}
  ): Promise<PublicKeyRegistrationResponse> {
    // Format public key as hex
    const publicKeyHex = formatKey(keyPair.publicKey, 'hex') as string;

    const request: PublicKeyRegistrationRequest = {
      clientId: options.clientId || this.config.defaultClientId,
      userId: options.userId || this.config.defaultUserId,
      publicKey: publicKeyHex,
      keyName: options.keyName,
      metadata: options.metadata
    };

    return this.httpClient.registerPublicKey(request);
  }

  /**
   * Register a public key with the server
   */
  async registerPublicKey(request: PublicKeyRegistrationRequest): Promise<PublicKeyRegistrationResponse> {
    return this.httpClient.registerPublicKey(request);
  }

  /**
   * Check registration status for a client
   */
  async checkRegistrationStatus(clientId?: string): Promise<RegistrationStatus> {
    const targetClientId = clientId || this.config.defaultClientId;
    
    if (!targetClientId) {
      return {
        registered: false,
        error: 'No client ID provided and no default client ID configured'
      };
    }

    try {
      const registration = await this.httpClient.getPublicKeyStatus(targetClientId);
      return {
        registered: true,
        registration
      };
    } catch (error) {
      if (error instanceof DataFoldServerError && error.errorCode === 'CLIENT_NOT_FOUND') {
        return { registered: false };
      }
      return {
        registered: false,
        error: error instanceof Error ? error.message : 'Unknown error'
      };
    }
  }

  /**
   * Get public key registration status
   */
  async getPublicKeyStatus(clientId: string): Promise<PublicKeyStatusResponse> {
    return this.httpClient.getPublicKeyStatus(clientId);
  }

  /**
   * Generate a signature for a message
   */
  async generateSignature(
    message: string,
    privateKey: Uint8Array,
    options: {
      messageEncoding?: 'utf8' | 'hex' | 'base64';
    } = {}
  ): Promise<SignatureResult> {
    const messageEncoding = options.messageEncoding || 'utf8';
    
    // Convert message to bytes based on encoding
    let messageBytes: Uint8Array;
    switch (messageEncoding) {
      case 'utf8':
        messageBytes = new TextEncoder().encode(message);
        break;
      case 'hex':
        messageBytes = parseKey(message, 'hex');
        break;
      case 'base64':
        messageBytes = parseKey(message, 'base64');
        break;
      default:
        throw new Error(`Unsupported message encoding: ${messageEncoding}`);
    }

    // Generate signature
    const signature = await ed25519.signAsync(messageBytes, privateKey);
    const signatureHex = formatKey(signature, 'hex') as string;

    return {
      signature: signatureHex,
      message,
      messageEncoding,
      timestamp: new Date().toISOString()
    };
  }

  /**
   * Sign a message and verify it with the server
   */
  async signAndVerify(
    message: string,
    keyPair: Ed25519KeyPair,
    options: {
      clientId?: string;
      messageEncoding?: 'utf8' | 'hex' | 'base64';
      metadata?: Record<string, string>;
    } = {}
  ): Promise<{
    signatureResult: SignatureResult;
    verificationResult: SignatureVerificationResponse;
  }> {
    // Generate signature
    const signatureResult = await this.generateSignature(
      message,
      keyPair.privateKey,
      { messageEncoding: options.messageEncoding }
    );

    // Verify signature with server
    const verificationResult = await this.verifySignature({
      clientId: options.clientId || this.config.defaultClientId || '',
      message,
      signature: signatureResult.signature,
      messageEncoding: options.messageEncoding,
      metadata: options.metadata
    });

    return { signatureResult, verificationResult };
  }

  /**
   * Verify a digital signature with the server
   */
  async verifySignature(request: SignatureVerificationRequest): Promise<SignatureVerificationResponse> {
    return this.httpClient.verifySignature(request);
  }

  /**
   * Complete registration and verification workflow
   */
  async registerAndVerifyWorkflow(
    keyPair: Ed25519KeyPair,
    testMessage: string = 'DataFold JS SDK Test Message',
    options: {
      clientId?: string;
      userId?: string;
      keyName?: string;
      metadata?: Record<string, string>;
      messageEncoding?: 'utf8' | 'hex' | 'base64';
    } = {}
  ): Promise<{
    registration: PublicKeyRegistrationResponse;
    signatureTest: SignatureResult;
    verification: SignatureVerificationResponse;
  }> {
    // Step 1: Register the key pair
    const registration = await this.registerKeyPair(keyPair, options);

    // Step 2: Generate a test signature
    const signatureTest = await this.generateSignature(
      testMessage,
      keyPair.privateKey,
      { messageEncoding: options.messageEncoding }
    );

    // Step 3: Verify the signature with the server
    const verification = await this.verifySignature({
      clientId: registration.clientId,
      message: testMessage,
      signature: signatureTest.signature,
      messageEncoding: options.messageEncoding || 'utf8',
      metadata: options.metadata
    });

    return { registration, signatureTest, verification };
  }

  /**
   * Get connection statistics
   */
  async getConnectionStats(): Promise<{
    connected: boolean;
    latency?: number;
    serverTime?: string;
    error?: string;
  }> {
    const connectionTest = await this.testConnection();
    
    if (!connectionTest.connected) {
      return connectionTest;
    }

    // Try to get server status for more detailed information
    try {
      // Note: This would need a server status endpoint
      return {
        connected: true,
        latency: connectionTest.latency,
        serverTime: new Date().toISOString() // Placeholder
      };
    } catch (error) {
      return {
        connected: true,
        latency: connectionTest.latency,
        error: 'Could not get detailed server information'
      };
    }
  }
}

/**
 * Create a server integration instance
 */
export function createServerIntegration(config?: ServerIntegrationConfig): DataFoldServerIntegration {
  return new DataFoldServerIntegration(config);
}

/**
 * Quick integration test - register a temporary key and verify a signature
 */
export async function quickIntegrationTest(
  config?: ServerIntegrationConfig
): Promise<{
  success: boolean;
  results?: {
    keyGeneration: boolean;
    registration: boolean;
    signatureGeneration: boolean;
    verification: boolean;
  };
  error?: string;
  details?: any;
}> {
  try {
    const { generateKeyPair } = await import('../crypto/ed25519.js');
    const integration = createServerIntegration(config);

    // Test connection
    const connection = await integration.testConnection();
    if (!connection.connected) {
      return {
        success: false,
        error: `Server connection failed: ${connection.error}`
      };
    }

    // Generate a test key pair
    const keyPair = await generateKeyPair();
    
    // Run full workflow
    const testClientId = `test_client_${Date.now()}`;
    const workflow = await integration.registerAndVerifyWorkflow(
      keyPair,
      'Integration test message',
      {
        clientId: testClientId,
        keyName: 'Integration Test Key',
        metadata: {
          test: 'true',
          timestamp: new Date().toISOString()
        }
      }
    );

    return {
      success: true,
      results: {
        keyGeneration: true,
        registration: true,
        signatureGeneration: true,
        verification: workflow.verification.verified
      },
      details: workflow
    };

  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error occurred',
      details: error
    };
  }
}