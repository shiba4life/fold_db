import { DataFoldClient } from '../src';
import axios from 'axios';
import MockAdapter from 'axios-mock-adapter';

describe('Schema Loading Tests', () => {
  let client: DataFoldClient;
  let mock: MockAdapter;

  beforeEach(() => {
    // Create a new instance of the client
    client = new DataFoldClient({
      baseUrl: 'http://localhost:8080',
    });

    // Create a mock adapter for axios
    mock = new MockAdapter(axios);
  });

  afterEach(() => {
    // Reset the mock adapter
    mock.reset();
  });

  test('loadSchemaFromFile should send correct request', async () => {
    // Mock the API response
    mock.onPost('http://localhost:8080/api/schema/load/file').reply(200, {
      data: {
        schema_name: 'TestSchema',
        message: 'Schema loaded successfully',
      },
    });

    // Call the method
    const result = await client.loadSchemaFromFile('/path/to/schema.json');

    // Check the result
    expect(result).toEqual({
      schema_name: 'TestSchema',
      message: 'Schema loaded successfully',
    });

    // Check that the request was made with the correct data
    expect(mock.history.post.length).toBe(1);
    expect(JSON.parse(mock.history.post[0].data)).toEqual({
      file_path: '/path/to/schema.json',
    });
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
    mock.onPost('http://localhost:8080/api/schema/load/json').reply(200, {
      data: {
        schema_name: 'TestSchema',
        message: 'Schema loaded successfully',
      },
    });

    // Call the method
    const result = await client.loadSchemaFromJson(schema);

    // Check the result
    expect(result).toEqual({
      schema_name: 'TestSchema',
      message: 'Schema loaded successfully',
    });

    // Check that the request was made with the correct data
    expect(mock.history.post.length).toBe(1);
    expect(JSON.parse(mock.history.post[0].data)).toEqual({
      schema_json: schema,
    });
  });

  test('loadSchemaFromFile should handle errors', async () => {
    // Mock the API response with an error
    mock.onPost('http://localhost:8080/api/schema/load/file').reply(400, {
      error: 'Schema file not found',
    });

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
    mock.onPost('http://localhost:8080/api/schema/load/json').reply(400, {
      error: 'Invalid schema format',
    });

    // Call the method and expect it to return a default error response
    const result = await client.loadSchemaFromJson(invalidSchema);

    // Check the result
    expect(result).toEqual({
      schema_name: '',
      message: 'Failed to load schema',
    });
  });
});
