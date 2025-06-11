/**
 * Mock server for DataFold server integration tests
 * Simulates the server API responses without requiring a real server
 */

export interface MockServerResponse {
  success: boolean;
  data?: any;
  error?: {
    code: string;
    message: string;
    details: Record<string, any>;
  };
}

export class MockDataFoldServer {
  private registeredKeys = new Map<string, any>();
  private simulateFailures: boolean = false;
  private delayMs: number = 0;

  constructor(options: { simulateFailures?: boolean; delayMs?: number } = {}) {
    this.simulateFailures = options.simulateFailures || false;
    this.delayMs = options.delayMs || 0;
  }

  private async delay(): Promise<void> {
    if (this.delayMs > 0) {
      await new Promise(resolve => setTimeout(resolve, this.delayMs));
    }
  }

  private createResponse(success: boolean, data?: any, error?: any): MockServerResponse {
    return {
      success,
      data: success ? data : undefined,
      error: !success ? error : undefined
    };
  }

  async testConnection(): Promise<MockServerResponse> {
    const startTime = Date.now();
    await this.delay();
    const latency = Date.now() - startTime + Math.floor(Math.random() * 10) + 1; // Ensure > 0
    
    if (this.simulateFailures) {
      return this.createResponse(false, null, {
        code: 'CONNECTION_FAILED',
        message: 'Failed to connect to server',
        details: {}
      });
    }

    return this.createResponse(true, {
      status: 'healthy',
      version: '1.0.0',
      timestamp: Date.now(),
      latency
    });
  }

  async registerPublicKey(params: {
    clientId: string;
    publicKey: string;
    keyName?: string;
    metadata?: Record<string, any>;
  }): Promise<MockServerResponse> {
    await this.delay();

    if (this.simulateFailures) {
      return this.createResponse(false, null, {
        code: 'REGISTRATION_FAILED',
        message: 'Failed to register public key',
        details: { clientId: params.clientId }
      });
    }

    // Check if already registered
    if (this.registeredKeys.has(params.clientId)) {
      return this.createResponse(false, null, {
        code: 'DUPLICATE_REGISTRATION',
        message: 'Client already registered',
        details: { clientId: params.clientId }
      });
    }

    // Validate public key format (basic validation)
    if (!params.publicKey || params.publicKey.length !== 64) {
      return this.createResponse(false, null, {
        code: 'INVALID_PUBLIC_KEY',
        message: 'Invalid public key format',
        details: { expectedLength: 64, actualLength: params.publicKey?.length }
      });
    }

    // Store the registration
    this.registeredKeys.set(params.clientId, {
      publicKey: params.publicKey,
      keyName: params.keyName,
      metadata: params.metadata || {},
      registeredAt: Date.now()
    });

    return this.createResponse(true, {
      registrationId: `reg_${Date.now()}_${Math.random().toString(36).substring(7)}`,
      clientId: params.clientId, // Ensure clientId is always included
      publicKey: params.publicKey,
      keyName: params.keyName || 'Test Key',
      registeredAt: new Date().toISOString(),
      status: 'active'
    });
  }

  async getPublicKeyStatus(clientId: string): Promise<MockServerResponse> {
    await this.delay();

    if (this.simulateFailures) {
      return this.createResponse(false, null, {
        code: 'STATUS_CHECK_FAILED',
        message: 'Failed to check registration status',
        details: { clientId }
      });
    }

    const registration = this.registeredKeys.get(clientId);
    
    if (!registration) {
      return this.createResponse(false, null, {
        code: 'CLIENT_NOT_FOUND',
        message: 'Client not registered',
        details: { clientId }
      });
    }

    return this.createResponse(true, {
      clientId: clientId, // Ensure clientId is always included in response
      publicKey: registration.publicKey,
      keyName: registration.keyName || 'Test Key',
      status: 'active',
      registeredAt: new Date(registration.registeredAt).toISOString(),
      lastUsed: new Date().toISOString()
    });
  }

  async verifySignature(params: {
    clientId: string;
    message: string;
    signature: string;
    encoding?: 'hex' | 'base64';
  }): Promise<MockServerResponse> {
    await this.delay();

    if (this.simulateFailures) {
      return this.createResponse(false, null, {
        code: 'VERIFICATION_FAILED',
        message: 'Failed to verify signature',
        details: { clientId: params.clientId }
      });
    }

    const registration = this.registeredKeys.get(params.clientId);
    
    if (!registration) {
      return this.createResponse(false, null, {
        code: 'CLIENT_NOT_FOUND',
        message: 'Client not registered',
        details: { clientId: params.clientId }
      });
    }

    // Validate signature format - very lenient for test purposes
    if (!params.signature || params.signature.length < 60) {
      return this.createResponse(false, null, {
        code: 'INVALID_SIGNATURE_FORMAT',
        message: 'Invalid signature format',
        details: {
          actualLength: params.signature?.length,
          encoding: params.encoding || 'hex',
          note: 'Signature too short'
        }
      });
    }

    // For mocking purposes, invalid signatures (all zeros) should throw errors
    const expectedLength = params.encoding === 'base64' ? 88 : 128;
    const invalidSignature = params.signature === '0'.repeat(expectedLength);
    
    if (invalidSignature) {
      return this.createResponse(false, null, {
        code: 'INVALID_SIGNATURE',
        message: 'Invalid signature format',
        details: {
          clientId: params.clientId,
          signature: params.signature.substring(0, 20) + '...'
        }
      });
    }

    // Valid signatures return success
    return this.createResponse(true, {
      verified: true,
      clientId: params.clientId, // Ensure clientId is always present
      publicKey: registration.publicKey,
      verifiedAt: new Date().toISOString(),
      messageHash: 'mock_hash_' + Date.now()
    });
  }

  async getConnectionStats(): Promise<MockServerResponse> {
    await this.delay();

    if (this.simulateFailures) {
      return this.createResponse(false, null, {
        code: 'STATS_UNAVAILABLE',
        message: 'Failed to retrieve connection stats',
        details: {}
      });
    }

    return this.createResponse(true, {
      status: 'healthy',
      uptime: Math.floor(Math.random() * 86400000), // Random uptime
      activeConnections: Math.floor(Math.random() * 100),
      totalRegistrations: this.registeredKeys.size,
      serverLoad: Math.random(),
      version: '1.0.0'
    });
  }

  // Helper methods for test setup
  clearRegistrations(): void {
    this.registeredKeys.clear();
  }

  setSimulateFailures(simulate: boolean): void {
    this.simulateFailures = simulate;
  }

  setDelay(delayMs: number): void {
    this.delayMs = delayMs;
  }

  simulateTimeout(): void {
    this.delayMs = 60000; // Very long delay to simulate timeout
  }

  getRegistrationCount(): number {
    return this.registeredKeys.size;
  }

  isClientRegistered(clientId: string): boolean {
    return this.registeredKeys.has(clientId);
  }
}

// Global mock server instance
export const mockServer = new MockDataFoldServer();

// Export for jest mock setup
export const setupServerMock = () => {
  // Clear mock server state completely before setting up
  mockServer.clearRegistrations();
  mockServer.setSimulateFailures(false);
  mockServer.setDelay(0);
  
  // Create a new mock function for each setup to avoid state pollution
  const mockFetch = jest.fn(async (url: any, options?: RequestInit) => {
    const urlStr = typeof url === 'string' ? url : (url as Request).url;
    const method = options?.method || 'GET';
    
    // Add small delay to simulate network latency
    await new Promise(resolve => setTimeout(resolve, Math.floor(Math.random() * 10) + 5));
    
    // Parse URL and route to appropriate mock method
    // Check for specific endpoints first, then general ones
    if (urlStr.includes('/keys/status/')) {
      const parts = urlStr.split('/keys/status/');
      let clientId = '';
      if (parts[1]) {
        // Extract clientId more robustly
        clientId = decodeURIComponent(parts[1].split('?')[0].split('/')[0]);
      }
      
      // Ensure clientId is not empty
      if (!clientId) {
        return Promise.resolve({
          ok: false,
          status: 400,
          json: () => Promise.resolve({
            success: false,
            error: {
              code: 'MISSING_CLIENT_ID',
              message: 'Client ID is required',
              details: { url: urlStr }
            }
          })
        } as Response);
      }
      
      const mockResponse = await mockServer.getPublicKeyStatus(clientId);
      return Promise.resolve({
        ok: mockResponse.success,
        status: mockResponse.success ? 200 : 404,
        json: () => Promise.resolve(mockResponse)
      } as Response);
    }
    
    if (urlStr.includes('/status')) {
      return {
        ok: true,
        status: 200,
        json: () => mockServer.testConnection()
      } as Response;
    }
    
    if (urlStr.includes('/keys/register') && method === 'POST') {
      const body = options?.body ? JSON.parse(options.body.toString()) : {};
      
      // Transform body from snake_case to camelCase to match our mock
      const transformedBody = {
        clientId: body.client_id || body.clientId,
        publicKey: body.public_key || body.publicKey,
        keyName: body.key_name || body.keyName,
        metadata: body.metadata
      };
      
      const mockResponse = await mockServer.registerPublicKey(transformedBody);
      return Promise.resolve({
        ok: mockResponse.success,
        status: mockResponse.success ? 200 : 400,
        json: () => Promise.resolve(mockResponse)
      } as Response);
    }
    
    
    if (urlStr.includes('/signatures/verify') && method === 'POST') {
      const body = options?.body ? JSON.parse(options.body.toString()) : {};
      // Transform body from snake_case to camelCase
      const transformedBody = {
        clientId: body.client_id,
        message: body.message,
        signature: body.signature,
        encoding: body.message_encoding || 'hex'
      };
      
      const mockResponse = await mockServer.verifySignature(transformedBody);
      return Promise.resolve({
        ok: mockResponse.success,
        status: mockResponse.success ? 200 : 400,
        json: () => Promise.resolve(mockResponse)
      } as Response);
    }
    
    if (urlStr.includes('/connection-stats')) {
      const mockResponse = await mockServer.getConnectionStats();
      return Promise.resolve({
        ok: mockResponse.success,
        status: mockResponse.success ? 200 : 500,
        json: () => Promise.resolve(mockResponse)
      } as Response);
    }
    
    // Default response for unmatched routes
    return Promise.resolve({
      ok: false,
      status: 404,
      json: () => Promise.resolve({
        success: false,
        error: {
          code: 'NOT_FOUND',
          message: 'Endpoint not found',
          details: { url: urlStr, method }
        }
      })
    } as Response);
  });
  
  // Override global fetch with our fresh mock
  (global as any).fetch = mockFetch;
};

// Clean up function
export const cleanupServerMock = () => {
  mockServer.clearRegistrations();
  mockServer.setSimulateFailures(false);
  mockServer.setDelay(0);
  
  // Reset fetch mock calls but keep the mock in place
  if (jest.isMockFunction(global.fetch)) {
    (global.fetch as jest.Mock).mockClear();
  }
};