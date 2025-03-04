#!/usr/bin/env node

/**
 * API Tests for Social App
 * 
 * This script tests the API endpoints directly to ensure posts are properly persisted.
 * It bypasses the UI and mock data to test the actual backend functionality.
 * 
 * Usage:
 *   node api-tests.js [--verbose]
 */

const fetch = require('node-fetch');
const assert = require('assert').strict;

// Configuration
const PORT = process.env.API_PORT || 3002; // Match the server port
const API_BASE_URL = `http://localhost:${PORT}/api`;
const VERBOSE = process.argv.includes('--verbose');

// Test data
const testPost = {
  content: `API Test Post ${Date.now()}`,
  author: { id: '1', username: 'api-test-user' },
  timestamp: new Date().toISOString(),
  likes: [],
  comments: []
};

// Utility functions
function log(...args) {
  if (VERBOSE) {
    console.log(...args);
  }
}

async function executeOperation(operation) {
  const response = await fetch(`${API_BASE_URL}/execute`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({ operation: JSON.stringify(operation) })
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`API request failed with status ${response.status}: ${errorText}`);
  }

  return response.json();
}

// Test runner
class APITestRunner {
  constructor() {
    this.results = [];
    this.createdPosts = [];
  }

  async runTest(name, testFn) {
    try {
      console.log(`Running test: ${name}`);
      await testFn();
      this.results.push({ name, passed: true });
      console.log(`✅ Test passed: ${name}`);
      return true;
    } catch (error) {
      this.results.push({ name, passed: false, error: error.message });
      console.log(`❌ Test failed: ${name}`);
      console.log(`Error: ${error.message}`);
      return false;
    }
  }

  summarize() {
    const total = this.results.length;
    const passed = this.results.filter(r => r.passed).length;
    const failed = total - passed;

    console.log(`\nTest Summary:`);
    console.log(`Total: ${total}`);
    console.log(`Passed: ${passed}`);
    console.log(`Failed: ${failed}`);

    if (failed > 0) {
      console.log('\nFailed Tests:');
      this.results.filter(r => !r.passed).forEach(result => {
        console.log(`- ${result.name}: ${result.error}`);
      });
      process.exit(1);
    } else {
      process.exit(0);
    }
  }

  // Test cleanup
  async cleanup() {
    if (this.createdPosts.length > 0) {
      console.log(`\nCleaning up ${this.createdPosts.length} test posts...`);
      
      for (const postId of this.createdPosts) {
        try {
          const deleteOperation = {
            type: "mutation",
            schema: "post",
            data: { id: postId },
            mutation_type: "delete"
          };
          
          await executeOperation(deleteOperation);
          log(`Deleted post ${postId}`);
        } catch (error) {
          console.log(`Warning: Failed to delete test post ${postId}: ${error.message}`);
        }
      }
    }
  }
}

// Test cases
async function testCreatePost() {
  const runner = new APITestRunner();

  await runner.runTest('Should create a post via API', async () => {
    // Create a post
    const createOperation = {
      type: "mutation",
      schema: "post",
      data: testPost,
      mutation_type: "create"
    };

    log('Creating post with operation:', createOperation);
    const createResult = await executeOperation(createOperation);
    log('Create result:', createResult);

    // Verify the result
    assert.ok(createResult.success, 'Create operation should succeed');
    assert.ok(createResult.data, 'Create operation should return data');
    assert.ok(createResult.data.id, 'Created post should have an ID');
    
    // Store the created post ID for later tests and cleanup
    const postId = createResult.data.id;
    runner.createdPosts.push(postId);
    
    // Verify the post was created with the correct content
    assert.strictEqual(createResult.data.content, testPost.content, 'Post content should match');
    assert.strictEqual(createResult.data.author.username, testPost.author.username, 'Post author should match');
  });

  await runner.runTest('Should retrieve the created post', async () => {
    // Skip if no posts were created
    if (runner.createdPosts.length === 0) {
      throw new Error('No posts were created in the previous test');
    }

    const postId = runner.createdPosts[0];
    
    // Query for the post
    const queryOperation = {
      type: "query",
      schema: "post",
      fields: ["id", "content", "timestamp", "author", "likes", "comments"],
      filter: { id: postId }
    };

    log('Querying post with operation:', queryOperation);
    const queryResult = await executeOperation(queryOperation);
    log('Query result:', queryResult);

    // Verify the result
    assert.ok(queryResult.data, 'Query should return data');
    assert.ok(Array.isArray(queryResult.data.results), 'Query results should be an array');
    assert.strictEqual(queryResult.data.results.length, 1, 'Query should return exactly one post');
    
    const post = queryResult.data.results[0];
    assert.strictEqual(post.id, postId, 'Retrieved post ID should match');
    assert.strictEqual(post.content, testPost.content, 'Retrieved post content should match');
    assert.strictEqual(post.author.username, testPost.author.username, 'Retrieved post author should match');
  });

  await runner.runTest('Should update a post', async () => {
    // Skip if no posts were created
    if (runner.createdPosts.length === 0) {
      throw new Error('No posts were created in the previous test');
    }

    const postId = runner.createdPosts[0];
    const updatedContent = `Updated content ${Date.now()}`;
    
    // Update the post
    const updateOperation = {
      type: "mutation",
      schema: "post",
      data: {
        id: postId,
        content: updatedContent
      },
      mutation_type: "update"
    };

    log('Updating post with operation:', updateOperation);
    const updateResult = await executeOperation(updateOperation);
    log('Update result:', updateResult);

    // Verify the update succeeded
    assert.ok(updateResult.success, 'Update operation should succeed');
    
    // Query for the post to verify the update
    const queryOperation = {
      type: "query",
      schema: "post",
      fields: ["id", "content"],
      filter: { id: postId }
    };

    const queryResult = await executeOperation(queryOperation);
    const post = queryResult.data.results[0];
    
    assert.strictEqual(post.content, updatedContent, 'Post content should be updated');
  });

  await runner.runTest('Should list all posts', async () => {
    // Query all posts
    const queryOperation = {
      type: "query",
      schema: "post",
      fields: ["id", "content", "author", "timestamp"],
      sort: { field: "timestamp", order: "desc" }
    };

    log('Querying all posts with operation:', queryOperation);
    const queryResult = await executeOperation(queryOperation);
    log('Query result:', queryResult);

    // Verify the result
    assert.ok(queryResult.data, 'Query should return data');
    assert.ok(Array.isArray(queryResult.data.results), 'Query results should be an array');
    assert.ok(queryResult.data.results.length > 0, 'Query should return at least one post');
    
    // Verify our created post is in the list
    if (runner.createdPosts.length > 0) {
      const postId = runner.createdPosts[0];
      const found = queryResult.data.results.some(post => post.id === postId);
      assert.ok(found, 'Created post should be in the list of all posts');
    }
  });

  // Clean up test data
  await runner.cleanup();
  
  // Summarize test results
  runner.summarize();
}

// Run the tests
console.log('Starting API tests...');
testCreatePost().catch(error => {
  console.error('Unhandled error:', error);
  process.exit(1);
});
