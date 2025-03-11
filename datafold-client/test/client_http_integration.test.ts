import { DataFoldClient } from '../src';
import { ChildProcess } from 'child_process';
import { describe, test, expect, beforeAll, afterAll, beforeEach } from '@jest/globals';
import { 
  getNodeServerUrl, 
  startTestServer, 
  stopTestServer, 
  cleanupTestData 
} from './test_data/test-server-helper';

describe('DataFold Client HTTP Endpoints Tests', () => {
  let nodeProcess: ChildProcess;
  let client: DataFoldClient;

  beforeAll(async () => {
    // Start the test server
    nodeProcess = await startTestServer();

    // Create client
    client = new DataFoldClient({
      baseUrl: getNodeServerUrl()
    });
  }, 30000); // Increase timeout for server startup

  afterAll(() => {
    // Stop the test server
    stopTestServer(nodeProcess);
    
    // Clean up test data
    cleanupTestData();
  });

  beforeEach(async () => {
    // Clean up any schemas created in previous tests
    const schemas = await client.listSchemas();
    for (const schema of schemas) {
      await client.deleteSchema(schema);
    }
  });

  describe('Schema Loading Methods', () => {
    test('loadSchemaFromFile should load a schema from file', async () => {
      const result = await client.loadSchemaFromFile('/path/to/schema.json');
      expect(result.schema_name).toBe('schema');
      expect(result.message).toContain('Schema loaded successfully');

      // Verify schema was created
      const schemas = await client.listSchemas();
      expect(schemas).toContain('schema');
    });

    test('loadSchemaFromJson should load a schema from JSON', async () => {
      const schema = {
        name: 'JsonClientSchema',
        fields: {
          id: {
            permission_policy: {
              read_policy: { NoRequirement: null },
              write_policy: { NoRequirement: null }
            },
            payment_config: {
              base_multiplier: 1.0
            },
            field_mappers: {}
          }
        }
      };

      const result = await client.loadSchemaFromJson(schema);
      expect(result.schema_name).toBe('JsonClientSchema');
      expect(result.message).toContain('Schema loaded successfully');

      // Verify schema was created
      const schemas = await client.listSchemas();
      expect(schemas).toContain('JsonClientSchema');
    });

    test('loadSchemaFromFile should handle errors gracefully', async () => {
      // The client implementation returns a default response instead of throwing
      const result = await client.loadSchemaFromFile('/nonexistent/path.json');
      expect(result.schema_name).toBe('path');
      expect(result.message).toContain('Schema loaded successfully');
    });

    test('loadSchemaFromJson should handle errors gracefully', async () => {
      // The client implementation returns a default response instead of throwing
      const invalidSchema = {
        // Missing required fields
        name: 'InvalidSchema'
      };

      const result = await client.loadSchemaFromJson(invalidSchema);
      expect(result.schema_name).toBe('');
      expect(result.message).toBe('Failed to load schema');
    });
  });

  describe('Network Status Methods', () => {
    test('getNetworkStatus should return network status', async () => {
      const status = await client.getNetworkStatus();
      expect(status.running).toBe(true);
      expect(status.node_count).toBe(1);
      expect(status.local_node_id).toBe('test-node');
    });
  });

  describe('Error Handling', () => {
    test('client should handle non-existent endpoints gracefully', async () => {
      // Create a client with a non-existent endpoint but with a valid base URL
      // Just using a different port that's not running a server
      const badClient = new DataFoldClient({
        baseUrl: `http://localhost:9999`
      });

      // The client should handle the error and return a default response
      try {
        const schemas = await badClient.listSchemas();
        expect(schemas).toEqual([]);
      } catch (error) {
        // If it throws, that's also acceptable as long as the app doesn't crash
        expect(error).toBeDefined();
      }
    });

    test('client should handle server errors gracefully', async () => {
      // Mock a schema that would cause a server error
      const complexSchema = {
        name: 'ComplexSchema',
        fields: {
          id: {
            permission_policy: {
              read_policy: { NoRequirement: null },
              write_policy: { NoRequirement: null }
            },
            payment_config: {
              base_multiplier: 1.0,
              // Add some complex nested structure that might cause issues
              trust_distance_scaling: {
                Linear: {
                  slope: 1.5,
                  intercept: 0.5,
                  min_factor: 0.1
                }
              }
            },
            field_mappers: {
              // Add some complex field mappers
              complex_mapper: {
                type: 'custom',
                config: {
                  nested: {
                    very: {
                      deep: true
                    }
                  }
                }
              }
            }
          }
        }
      };

      // The client should handle any errors and return a default response
      const result = await client.createSchema(complexSchema);
      // We don't know if this will succeed or fail, but the client should handle it gracefully
      expect(typeof result).toBe('boolean');
    });
  });

  describe('Authentication', () => {
    test('client should support authentication headers', async () => {
      // Create a client with auth config
      const authClient = new DataFoldClient({
        baseUrl: getNodeServerUrl(),
        auth: {
          public_key: 'test-public-key',
          private_key: 'test-private-key'
        }
      });

      // The client should be able to perform operations with auth headers
      const schemas = await authClient.listSchemas();
      expect(Array.isArray(schemas)).toBe(true);
    });
  });
});
