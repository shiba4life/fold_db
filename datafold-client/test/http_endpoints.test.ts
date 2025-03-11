import { DataFoldClient } from '../src';
import { ChildProcess } from 'child_process';
import { describe, test, expect, beforeAll, afterAll, beforeEach } from '@jest/globals';
import { 
  getNodeServerUrl, 
  startTestServer, 
  stopTestServer, 
  cleanupTestData 
} from './test_data/test-server-helper';
import axios from 'axios';

describe('DataFold Node HTTP Endpoints Tests', () => {
  let nodeProcess: ChildProcess;
  let client: DataFoldClient;
  let baseUrl: string;

  beforeAll(async () => {
    // Start the test server
    nodeProcess = await startTestServer();
    baseUrl = getNodeServerUrl();

    // Create client
    client = new DataFoldClient({
      baseUrl
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

  describe('Schema Endpoints', () => {
    test('GET /api/schemas should return empty array when no schemas exist', async () => {
      const response = await axios.get(`${baseUrl}/api/schemas`);
      expect(response.status).toBe(200);
      expect(response.data).toEqual({ schemas: [] });
    });

    test('POST /api/schema should create a new schema', async () => {
      const schema = {
        name: 'TestSchema',
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

      const response = await axios.post(`${baseUrl}/api/schema`, schema);
      expect(response.status).toBe(200);
      expect(response.data).toEqual({ success: true });

      // Verify schema was created
      const schemasResponse = await axios.get(`${baseUrl}/api/schemas`);
      expect(schemasResponse.data.schemas).toContain('TestSchema');
    });

    test('POST /api/schema should return 400 for invalid schema', async () => {
      const invalidSchema = {
        // Missing required fields
        name: 'InvalidSchema'
      };

      try {
        await axios.post(`${baseUrl}/api/schema`, invalidSchema);
        expect(true).toBe(false); // This line should not be reached
      } catch (error: any) {
        expect(error.response.status).toBe(400);
        expect(error.response.data.success).toBe(false);
        expect(error.response.data.error).toContain('Invalid schema format');
      }
    });

    test('DELETE /api/schema/:name should delete a schema', async () => {
      // Create a schema first
      const schema = {
        name: 'SchemaToDelete',
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

      await axios.post(`${baseUrl}/api/schema`, schema);

      // Delete the schema
      const response = await axios.delete(`${baseUrl}/api/schema/SchemaToDelete`);
      expect(response.status).toBe(200);
      expect(response.data).toEqual({ success: true });

      // Verify schema was deleted
      const schemasResponse = await axios.get(`${baseUrl}/api/schemas`);
      expect(schemasResponse.data.schemas).not.toContain('SchemaToDelete');
    });

    test('DELETE /api/schema/:name should return 404 for non-existent schema', async () => {
      try {
        await axios.delete(`${baseUrl}/api/schema/NonExistentSchema`);
        expect(true).toBe(false); // This line should not be reached
      } catch (error: any) {
        expect(error.response.status).toBe(404);
        expect(error.response.data.success).toBe(false);
        expect(error.response.data.error).toContain('Schema not found');
      }
    });
  });

  describe('Schema Loading Endpoints', () => {
    test('POST /api/schema/load/file should load a schema from file', async () => {
      const request = {
        file_path: '/path/to/schema.json'
      };

      const response = await axios.post(`${baseUrl}/api/schema/load/file`, request);
      expect(response.status).toBe(200);
      expect(response.data.data.schema_name).toBe('schema');
      expect(response.data.data.message).toContain('Schema loaded successfully');

      // Verify schema was created
      const schemasResponse = await axios.get(`${baseUrl}/api/schemas`);
      expect(schemasResponse.data.schemas).toContain('schema');
    });

    test('POST /api/schema/load/file should return 400 for missing file path', async () => {
      try {
        await axios.post(`${baseUrl}/api/schema/load/file`, {});
        expect(true).toBe(false); // This line should not be reached
      } catch (error: any) {
        expect(error.response.status).toBe(400);
        expect(error.response.data.success).toBe(false);
        expect(error.response.data.error).toContain('File path is required');
      }
    });

    test('POST /api/schema/load/json should load a schema from JSON', async () => {
      const request = {
        schema_json: {
          name: 'JsonSchema',
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
        }
      };

      const response = await axios.post(`${baseUrl}/api/schema/load/json`, request);
      expect(response.status).toBe(200);
      expect(response.data.data.schema_name).toBe('JsonSchema');
      expect(response.data.data.message).toContain('Schema loaded successfully');

      // Verify schema was created
      const schemasResponse = await axios.get(`${baseUrl}/api/schemas`);
      expect(schemasResponse.data.schemas).toContain('JsonSchema');
    });

    test('POST /api/schema/load/json should return 400 for invalid schema JSON', async () => {
      const request = {
        schema_json: {
          // Missing required fields
          name: 'InvalidSchema'
        }
      };

      try {
        await axios.post(`${baseUrl}/api/schema/load/json`, request);
        expect(true).toBe(false); // This line should not be reached
      } catch (error: any) {
        expect(error.response.status).toBe(400);
        expect(error.response.data.success).toBe(false);
        expect(error.response.data.error).toContain('Invalid schema format');
      }
    });
  });

  describe('Execute Operation Endpoint', () => {
    beforeEach(async () => {
      // Create a test schema for operation tests
      const schema = {
        name: 'OperationTestSchema',
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
          },
          name: {
            permission_policy: {
              read_policy: { NoRequirement: null },
              write_policy: { NoRequirement: null }
            },
            payment_config: {
              base_multiplier: 1.0
            },
            field_mappers: {}
          },
          age: {
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

      await client.createSchema(schema);
    });

    test('POST /api/execute should execute a query operation', async () => {
      // Create a test record first
      await client.create('OperationTestSchema', { id: '1', name: 'Test User', age: 30 });

      // Execute a query operation
      const operation = {
        type: 'query',
        schema: 'OperationTestSchema',
        fields: ['id', 'name', 'age'],
        filter: { id: '1' }
      };

      const response = await axios.post(`${baseUrl}/api/execute`, {
        operation: JSON.stringify(operation)
      });

      expect(response.status).toBe(200);
      expect(response.data.results).toHaveLength(1);
      expect(response.data.results[0]).toEqual({
        id: '1',
        name: 'Test User',
        age: 30
      });
      expect(response.data.count).toBe(1);
    });

    test('POST /api/execute should execute a create mutation operation', async () => {
      const operation = {
        type: 'mutation',
        schema: 'OperationTestSchema',
        operation: 'create',
        data: { id: '2', name: 'New User', age: 25 }
      };

      const response = await axios.post(`${baseUrl}/api/execute`, {
        operation: JSON.stringify(operation)
      });

      expect(response.status).toBe(200);
      expect(response.data.success).toBe(true);
      expect(response.data.affected_count).toBe(1);

      // Verify the record was created
      const queryResult = await client.findOne('OperationTestSchema', ['id', 'name', 'age'], { id: '2' });
      expect(queryResult).toEqual({
        id: '2',
        name: 'New User',
        age: 25
      });
    });

    test('POST /api/execute should execute an update mutation operation', async () => {
      // Create a test record first
      await client.create('OperationTestSchema', { id: '3', name: 'Update User', age: 40 });

      const operation = {
        type: 'mutation',
        schema: 'OperationTestSchema',
        operation: 'update',
        filter: { id: '3' },
        data: { name: 'Updated User', age: 41 }
      };

      const response = await axios.post(`${baseUrl}/api/execute`, {
        operation: JSON.stringify(operation)
      });

      expect(response.status).toBe(200);
      expect(response.data.success).toBe(true);
      expect(response.data.affected_count).toBe(1);

      // Verify the record was updated
      const queryResult = await client.findOne('OperationTestSchema', ['id', 'name', 'age'], { id: '3' });
      expect(queryResult).toEqual({
        id: '3',
        name: 'Updated User',
        age: 41
      });
    });

    test('POST /api/execute should execute a delete mutation operation', async () => {
      // Create a test record first
      await client.create('OperationTestSchema', { id: '4', name: 'Delete User', age: 50 });

      const operation = {
        type: 'mutation',
        schema: 'OperationTestSchema',
        operation: 'delete',
        filter: { id: '4' }
      };

      const response = await axios.post(`${baseUrl}/api/execute`, {
        operation: JSON.stringify(operation)
      });

      expect(response.status).toBe(200);
      expect(response.data.success).toBe(true);
      expect(response.data.affected_count).toBe(1);

      // Verify the record was deleted
      const queryResult = await client.findOne('OperationTestSchema', ['id', 'name', 'age'], { id: '4' });
      expect(queryResult).toBeNull();
    });

    test('POST /api/execute should return 400 for missing operation', async () => {
      try {
        await axios.post(`${baseUrl}/api/execute`, {});
        expect(true).toBe(false); // This line should not be reached
      } catch (error: any) {
        expect(error.response.status).toBe(400);
        expect(error.response.data.success).toBe(false);
        expect(error.response.data.error).toContain('Operation is required');
      }
    });

    test('POST /api/execute should return 400 for invalid operation format', async () => {
      try {
        await axios.post(`${baseUrl}/api/execute`, {
          operation: '{invalid json'
        });
        expect(true).toBe(false); // This line should not be reached
      } catch (error: any) {
        expect(error.response.status).toBe(400);
        expect(error.response.data.success).toBe(false);
        expect(error.response.data.error).toContain('Invalid operation format');
      }
    });

    test('POST /api/execute should return 404 for non-existent schema', async () => {
      const operation = {
        type: 'query',
        schema: 'NonExistentSchema',
        fields: ['id'],
        filter: null
      };

      try {
        await axios.post(`${baseUrl}/api/execute`, {
          operation: JSON.stringify(operation)
        });
        expect(true).toBe(false); // This line should not be reached
      } catch (error: any) {
        expect(error.response.status).toBe(404);
        expect(error.response.data.success).toBe(false);
        expect(error.response.data.error).toContain('Schema not found');
      }
    });

    test('POST /api/execute should return 400 for unsupported operation type', async () => {
      const operation = {
        type: 'unsupported',
        schema: 'OperationTestSchema',
        fields: ['id'],
        filter: null
      };

      try {
        await axios.post(`${baseUrl}/api/execute`, {
          operation: JSON.stringify(operation)
        });
        expect(true).toBe(false); // This line should not be reached
      } catch (error: any) {
        expect(error.response.status).toBe(400);
        expect(error.response.data.success).toBe(false);
        expect(error.response.data.error).toContain('Unsupported operation type');
      }
    });
  });

  describe('Network Status Endpoint', () => {
    test('GET /api/network/status should return network status', async () => {
      const response = await axios.get(`${baseUrl}/api/network/status`);
      expect(response.status).toBe(200);
      expect(response.data).toEqual({
        running: true,
        node_count: 1,
        connection_count: 0,
        local_node_id: 'test-node'
      });
    });
  });
});
