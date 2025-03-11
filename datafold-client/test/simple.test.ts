import { DataFoldClient } from '../src';
import { describe, test, expect, beforeAll, afterAll } from '@jest/globals';
import { 
  NODE_SERVER_URL, 
  startTestServer, 
  stopTestServer, 
  cleanupTestData 
} from './test_data/test-server-helper';

describe('Simple Test', () => {
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
  });
});
