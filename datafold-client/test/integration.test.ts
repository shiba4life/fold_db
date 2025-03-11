import { DataFoldClient } from '../src';
import { ChildProcess } from 'child_process';
import { describe, test, expect, beforeAll, afterAll } from '@jest/globals';
import { 
  getNodeServerUrl, 
  startTestServer, 
  stopTestServer, 
  cleanupTestData 
} from './test_data/test-server-helper';

describe('DataFold Client Integration Tests', () => {
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

  test('should connect to the node server', async () => {
    const schemas = await client.listSchemas();
    expect(Array.isArray(schemas)).toBe(true);
  });

  test('should create a schema', async () => {
    const testSchema = {
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

    const result = await client.createSchema(testSchema);
    expect(result).toBe(true);

    // Verify schema was created
    const schemas = await client.listSchemas();
    expect(schemas).toContain('TestSchema');
  });

  test('should create a record', async () => {
    const testData = {
      id: '1',
      name: 'Test User',
      age: 30
    };

    const result = await client.create('TestSchema', testData);
    expect(result.success).toBe(true);
    expect(result.affected_count).toBe(1);
  });

  test('should find records', async () => {
    const results = await client.find('TestSchema', ['id', 'name', 'age']);
    expect(results.length).toBe(1);
    expect(results[0]).toEqual({
      id: '1',
      name: 'Test User',
      age: 30
    });
  });

  test('should find records with filter', async () => {
    const results = await client.find('TestSchema', ['id', 'name'], { id: '1' });
    expect(results.length).toBe(1);
    expect(results[0]).toEqual({
      id: '1',
      name: 'Test User'
    });

    // Should return empty array for non-matching filter
    const emptyResults = await client.find('TestSchema', ['id', 'name'], { id: 'non-existent' });
    expect(emptyResults.length).toBe(0);
  });

  test('should find one record', async () => {
    const result = await client.findOne('TestSchema', ['id', 'name', 'age'], { id: '1' });
    expect(result).toEqual({
      id: '1',
      name: 'Test User',
      age: 30
    });

    // Should return null for non-matching filter
    const nullResult = await client.findOne('TestSchema', ['id', 'name'], { id: 'non-existent' });
    expect(nullResult).toBeNull();
  });

  test('should update a record', async () => {
    const result = await client.update('TestSchema', { id: '1' }, { name: 'Updated User' });
    expect(result.success).toBe(true);
    expect(result.affected_count).toBe(1);

    // Verify the update
    const updatedRecord = await client.findOne('TestSchema', ['id', 'name', 'age'], { id: '1' });
    expect(updatedRecord).toEqual({
      id: '1',
      name: 'Updated User',
      age: 30
    });
  });

  test('should delete a record', async () => {
    const result = await client.delete('TestSchema', { id: '1' });
    expect(result.success).toBe(true);
    expect(result.affected_count).toBe(1);

    // Verify the delete
    const results = await client.find('TestSchema', ['id', 'name', 'age']);
    expect(results.length).toBe(0);
  });

  test('should delete a schema', async () => {
    const result = await client.deleteSchema('TestSchema');
    expect(result).toBe(true);

    // Verify schema was deleted
    const schemas = await client.listSchemas();
    expect(schemas).not.toContain('TestSchema');
  });

  test('should get network status', async () => {
    const status = await client.getNetworkStatus();
    expect(status.running).toBe(true);
    expect(status.node_count).toBe(1);
    expect(status.local_node_id).toBe('test-node');
  });

  test('should handle errors gracefully', async () => {
    // Try to find records in a non-existent schema
    let errorThrown = false;
    try {
      await client.find('NonExistentSchema', ['id']);
    } catch (error) {
      errorThrown = true;
      expect(error).toBeDefined();
    }
    expect(errorThrown).toBe(true);

    // Try to create a record in a non-existent schema
    errorThrown = false;
    try {
      await client.create('NonExistentSchema', { id: '1' });
    } catch (error) {
      errorThrown = true;
      expect(error).toBeDefined();
    }
    expect(errorThrown).toBe(true);
  });
});
