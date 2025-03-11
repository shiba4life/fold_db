import { DataFoldClient } from '../src';
import axios from 'axios';
import { describe, test, expect, beforeEach, jest } from '@jest/globals';

// Mock the axios module
jest.mock('axios');

describe('Schema Loading Tests', () => {
  let client: DataFoldClient;
  let mockAxios: any;

  beforeEach(() => {
    // Reset axios mock
    mockAxios = {
      create: jest.fn().mockReturnThis(),
      get: jest.fn(),
      post: jest.fn(),
      delete: jest.fn(),
      interceptors: {
        request: {
          use: jest.fn()
        }
      }
    };
    
    // Set the mock implementation
    (axios.create as jest.Mock).mockReturnValue(mockAxios);
    
    // Create a new instance of the client
    client = new DataFoldClient({
      baseUrl: 'http://localhost:8080',
    });
  });

  test('loadSchemaFromFile should send correct request', async () => {
    // Mock the API response
    mockAxios.post.mockResolvedValueOnce({
      data: {
        data: {
          schema_name: 'TestSchema',
          message: 'Schema loaded successfully',
        }
      }
    });

    // Call the method
    const result = await client.loadSchemaFromFile('/path/to/schema.json');

    // Check the result
    expect(result).toEqual({
      schema_name: 'TestSchema',
      message: 'Schema loaded successfully',
    });

    // Check that the request was made with the correct data
    expect(mockAxios.post).toHaveBeenCalledWith(
      '/api/schema/load/file',
      { file_path: '/path/to/schema.json' }
    );
  });

  test('loadSchemaFromJson should send correct request', async () => {
    // Example schema
    const schema = {
      name: 'TestSchema',
      fields: {
        title: {
          permission_policy: {
            read_policy: {
              NoRequirement: null,
            },
            write_policy: {
              Distance: 0,
            },
          },
          payment_config: {
            base_multiplier: 1.0,
          },
          field_mappers: {},
        },
      },
      payment_config: {
        base_multiplier: 1.0,
        min_payment_threshold: 0,
      },
    };

    // Mock the API response
    mockAxios.post.mockResolvedValueOnce({
      data: {
        data: {
          schema_name: 'TestSchema',
          message: 'Schema loaded successfully',
        }
      }
    });

    // Call the method
    const result = await client.loadSchemaFromJson(schema);

    // Check the result
    expect(result).toEqual({
      schema_name: 'TestSchema',
      message: 'Schema loaded successfully',
    });

    // Check that the request was made with the correct data
    expect(mockAxios.post).toHaveBeenCalledWith(
      '/api/schema/load/json',
      { schema_json: schema }
    );
  });

  test('loadSchemaFromFile should handle errors', async () => {
    // Mock the API response with an error
    const error = {
      isAxiosError: true,
      response: {
        data: {
          error: 'Schema file not found'
        }
      }
    };
    mockAxios.post.mockRejectedValueOnce(error);

    // Call the method and expect it to return a default error response
    const result = await client.loadSchemaFromFile('/path/to/nonexistent.json');

    // Check the result
    expect(result).toEqual({
      schema_name: '',
      message: 'Failed to load schema',
    });
  });

  test('loadSchemaFromJson should handle errors', async () => {
    // Example invalid schema
    const invalidSchema = {
      // Missing required fields
      name: 'TestSchema',
    };

    // Mock the API response with an error
    const error = {
      isAxiosError: true,
      response: {
        data: {
          error: 'Invalid schema format'
        }
      }
    };
    mockAxios.post.mockRejectedValueOnce(error);

    // Call the method and expect it to return a default error response
    const result = await client.loadSchemaFromJson(invalidSchema);

    // Check the result
    expect(result).toEqual({
      schema_name: '',
      message: 'Failed to load schema',
    });
  });
});
