import { DataFoldClient } from '../src';
import axios from 'axios';

// Mock axios
jest.mock('axios');
const mockedAxios = axios as jest.Mocked<typeof axios>;

describe('DataFoldClient', () => {
  let client: DataFoldClient;
  
  beforeEach(() => {
    // Reset mocks
    jest.clearAllMocks();
    
    // Create a mock axios instance
    mockedAxios.create.mockReturnValue(mockedAxios as any);
    
    // Create a client instance
    client = new DataFoldClient({
      baseUrl: 'http://localhost:3000'
    });
  });
  
  describe('Schema operations', () => {
    test('listSchemas should return schema names', async () => {
      // Mock response
      mockedAxios.get.mockResolvedValueOnce({
        data: {
          schemas: ['UserProfile', 'UserProfile2']
        }
      });
      
      // Call the method
      const result = await client.listSchemas();
      
      // Verify the result
      expect(result).toEqual(['UserProfile', 'UserProfile2']);
      expect(mockedAxios.get).toHaveBeenCalledWith('/api/schemas');
    });
    
    test('createSchema should create a schema', async () => {
      // Mock response
      mockedAxios.post.mockResolvedValueOnce({
        data: {
          success: true
        }
      });
      
      // Create a schema
      const schema = {
        name: 'TestSchema',
        fields: {
          field1: {
            permission_policy: {
              read_policy: { NoRequirement: null },
              write_policy: { Distance: 0 }
            },
            payment_config: {
              base_multiplier: 1.0,
              trust_distance_scaling: { None: null }
            },
            field_mappers: {}
          }
        }
      };
      
      // Call the method
      const result = await client.createSchema(schema);
      
      // Verify the result
      expect(result).toBe(true);
      expect(mockedAxios.post).toHaveBeenCalledWith('/api/schema', schema);
    });
    
    test('deleteSchema should delete a schema', async () => {
      // Mock response
      mockedAxios.delete.mockResolvedValueOnce({
        data: {
          success: true
        }
      });
      
      // Call the method
      const result = await client.deleteSchema('TestSchema');
      
      // Verify the result
      expect(result).toBe(true);
      expect(mockedAxios.delete).toHaveBeenCalledWith('/api/schema/TestSchema');
    });
  });
  
  describe('Query operations', () => {
    test('find should return query results', async () => {
      // Mock response
      mockedAxios.post.mockResolvedValueOnce({
        data: {
          results: [
            { username: 'user1', email: 'user1@example.com' },
            { username: 'user2', email: 'user2@example.com' }
          ],
          count: 2
        }
      });
      
      // Call the method
      const result = await client.find('UserProfile', ['username', 'email'], { age: { gt: 30 } });
      
      // Verify the result
      expect(result).toEqual([
        { username: 'user1', email: 'user1@example.com' },
        { username: 'user2', email: 'user2@example.com' }
      ]);
      
      // Verify the request
      expect(mockedAxios.post).toHaveBeenCalledWith('/api/execute', {
        operation: JSON.stringify({
          type: 'query',
          schema: 'UserProfile',
          fields: ['username', 'email'],
          filter: { age: { gt: 30 } }
        })
      });
    });
    
    test('findOne should return a single result', async () => {
      // Mock response
      mockedAxios.post.mockResolvedValueOnce({
        data: {
          results: [
            { username: 'user1', email: 'user1@example.com' }
          ],
          count: 1
        }
      });
      
      // Call the method
      const result = await client.findOne('UserProfile', ['username', 'email'], { username: 'user1' });
      
      // Verify the result
      expect(result).toEqual({ username: 'user1', email: 'user1@example.com' });
      
      // Verify the request
      expect(mockedAxios.post).toHaveBeenCalledWith('/api/execute', {
        operation: JSON.stringify({
          type: 'query',
          schema: 'UserProfile',
          fields: ['username', 'email'],
          filter: { username: 'user1' }
        })
      });
    });
  });
  
  describe('Mutation operations', () => {
    test('create should create a record', async () => {
      // Mock response
      mockedAxios.post.mockResolvedValueOnce({
        data: {
          success: true,
          affected_count: 1
        }
      });
      
      // Call the method
      const result = await client.create('UserProfile', {
        username: 'newuser',
        email: 'newuser@example.com'
      });
      
      // Verify the result
      expect(result).toEqual({
        success: true,
        affected_count: 1
      });
      
      // Verify the request
      expect(mockedAxios.post).toHaveBeenCalledWith('/api/execute', {
        operation: JSON.stringify({
          type: 'mutation',
          schema: 'UserProfile',
          operation: 'create',
          data: {
            username: 'newuser',
            email: 'newuser@example.com'
          }
        })
      });
    });
    
    test('update should update records', async () => {
      // Mock response
      mockedAxios.post.mockResolvedValueOnce({
        data: {
          success: true,
          affected_count: 1
        }
      });
      
      // Call the method
      const result = await client.update(
        'UserProfile',
        { username: 'user1' },
        { email: 'updated@example.com' }
      );
      
      // Verify the result
      expect(result).toEqual({
        success: true,
        affected_count: 1
      });
      
      // Verify the request
      expect(mockedAxios.post).toHaveBeenCalledWith('/api/execute', {
        operation: JSON.stringify({
          type: 'mutation',
          schema: 'UserProfile',
          operation: 'update',
          filter: { username: 'user1' },
          data: { email: 'updated@example.com' }
        })
      });
    });
    
    test('delete should delete records', async () => {
      // Mock response
      mockedAxios.post.mockResolvedValueOnce({
        data: {
          success: true,
          affected_count: 1
        }
      });
      
      // Call the method
      const result = await client.delete('UserProfile', { username: 'user1' });
      
      // Verify the result
      expect(result).toEqual({
        success: true,
        affected_count: 1
      });
      
      // Verify the request
      expect(mockedAxios.post).toHaveBeenCalledWith('/api/execute', {
        operation: JSON.stringify({
          type: 'mutation',
          schema: 'UserProfile',
          operation: 'delete',
          filter: { username: 'user1' }
        })
      });
    });
  });
  
  describe('Network operations', () => {
    test('getNetworkStatus should return network status', async () => {
      // Mock response
      mockedAxios.get.mockResolvedValueOnce({
        data: {
          running: true,
          node_count: 3,
          connection_count: 2,
          local_node_id: 'node1'
        }
      });
      
      // Call the method
      const result = await client.getNetworkStatus();
      
      // Verify the result
      expect(result).toEqual({
        running: true,
        node_count: 3,
        connection_count: 2,
        local_node_id: 'node1'
      });
      
      // Verify the request
      expect(mockedAxios.get).toHaveBeenCalledWith('/api/network/status');
    });
    
    test('listNodes should return node info', async () => {
      // Mock response
      mockedAxios.get.mockResolvedValueOnce({
        data: {
          nodes: [
            {
              id: 'node1',
              addr: '192.168.1.100',
              port: 8000,
              public_key: 'key1',
              status: 'connected'
            },
            {
              id: 'node2',
              addr: '192.168.1.101',
              port: 8000,
              public_key: 'key2',
              status: 'connected'
            }
          ]
        }
      });
      
      // Call the method
      const result = await client.listNodes();
      
      // Verify the result
      expect(result).toEqual([
        {
          id: 'node1',
          addr: '192.168.1.100',
          port: 8000,
          public_key: 'key1',
          status: 'connected'
        },
        {
          id: 'node2',
          addr: '192.168.1.101',
          port: 8000,
          public_key: 'key2',
          status: 'connected'
        }
      ]);
      
      // Verify the request
      expect(mockedAxios.get).toHaveBeenCalledWith('/api/network/nodes');
    });
  });
  
  describe('Error handling', () => {
    test('should handle axios errors', async () => {
      // Mock error response
      const errorResponse = {
        response: {
          data: {
            code: 'SCHEMA_NOT_FOUND',
            message: 'Schema not found',
            details: { schema_name: 'NonExistentSchema' }
          }
        }
      };
      
      mockedAxios.get.mockRejectedValueOnce(errorResponse);
      
      // Call the method and expect it to throw
      await expect(async () => {
        await client.listSchemas();
      }).rejects.toMatchObject({
        response: {
          data: {
            code: 'SCHEMA_NOT_FOUND',
            message: 'Schema not found',
            details: { schema_name: 'NonExistentSchema' }
          }
        }
      });
    });
  });
});
