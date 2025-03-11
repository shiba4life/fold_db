import { DataFoldClient } from '../src';
import { describe, test, expect, beforeAll } from '@jest/globals';

describe('Error Test', () => {
  let client: DataFoldClient;

  beforeAll(async () => {
    // Create client
    client = new DataFoldClient({
      baseUrl: 'http://localhost:8082'
    });
  });

  test('should handle errors gracefully', async () => {
    // Try to find records in a non-existent schema
    await expect(async () => {
      await client.find('NonExistentSchema', ['id']);
    }).rejects.toThrow();

    // Try to create a record in a non-existent schema
    await expect(async () => {
      await client.create('NonExistentSchema', { id: '1' });
    }).rejects.toThrow();
  });
});
