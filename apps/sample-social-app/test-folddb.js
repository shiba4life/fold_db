#!/usr/bin/env node

/**
 * FoldDB Integration Tests for Social App
 * 
 * This script tests the integration between the Social App and FoldDB.
 * It verifies that data is properly persisted and retrieved.
 * 
 * Usage:
 *   node test-folddb.js [--verbose]
 */

const assert = require('assert').strict;
const path = require('path');
const foldDBClient = require('./fold_db_client');

// Configuration
const SCHEMAS_DIR = path.join(__dirname, 'schemas');
const VERBOSE = process.argv.includes('--verbose');

// Utility functions
function log(...args) {
  if (VERBOSE) {
    console.log(...args);
  }
}

// Test runner
class FoldDBTestRunner {
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
          
          await foldDBClient.executeMutation(deleteOperation);
          log(`Deleted post ${postId}`);
        } catch (error) {
          console.log(`Warning: Failed to delete test post ${postId}: ${error.message}`);
        }
      }
    }
  }
}

// Test cases
async function testFoldDBIntegration() {
  const runner = new FoldDBTestRunner();

  // Initialize FoldDB client
  await foldDBClient.initialize(SCHEMAS_DIR);
  console.log('FoldDB client initialized with schemas');

  // Test schema loading
  await runner.runTest('Should load schemas correctly', async () => {
    const postSchema = foldDBClient.getSchema('post');
    assert.ok(postSchema, 'Post schema should be loaded');
    assert.strictEqual(postSchema.name, 'post', 'Schema name should be "post"');
    assert.ok(postSchema.fields.content, 'Schema should have content field');
    assert.ok(postSchema.fields.author, 'Schema should have author field');
    
    const userProfileSchema = foldDBClient.getSchema('user-profile');
    assert.ok(userProfileSchema, 'User profile schema should be loaded');
    
    const commentSchema = foldDBClient.getSchema('comment');
    assert.ok(commentSchema, 'Comment schema should be loaded');
  });

  // Test post creation
  await runner.runTest('Should create a post', async () => {
    const testPost = {
      content: `FoldDB Test Post ${Date.now()}`,
      author: { id: '1', username: 'alice' },
      timestamp: new Date().toISOString(),
      likes: [],
      comments: []
    };
    
    const createOperation = {
      type: "mutation",
      schema: "post",
      data: testPost,
      mutation_type: "create"
    };
    
    log('Creating post with operation:', createOperation);
    const createResult = await foldDBClient.executeMutation(createOperation);
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

  // Test post retrieval
  await runner.runTest('Should retrieve a post', async () => {
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
    const posts = await foldDBClient.executeQuery(queryOperation);
    log('Query result:', posts);
    
    // Verify the result
    assert.ok(Array.isArray(posts), 'Query should return an array');
    assert.strictEqual(posts.length, 1, 'Query should return exactly one post');
    
    const post = posts[0];
    assert.strictEqual(post.id, postId, 'Retrieved post ID should match');
  });

  // Test post update
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
    const updateResult = await foldDBClient.executeMutation(updateOperation);
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
    
    const posts = await foldDBClient.executeQuery(queryOperation);
    const post = posts[0];
    
    assert.strictEqual(post.content, updatedContent, 'Post content should be updated');
  });

  // Test post deletion
  await runner.runTest('Should delete a post', async () => {
    // Skip if no posts were created
    if (runner.createdPosts.length === 0) {
      throw new Error('No posts were created in the previous test');
    }
    
    const postId = runner.createdPosts[0];
    
    // Delete the post
    const deleteOperation = {
      type: "mutation",
      schema: "post",
      data: { id: postId },
      mutation_type: "delete"
    };
    
    log('Deleting post with operation:', deleteOperation);
    const deleteResult = await foldDBClient.executeMutation(deleteOperation);
    log('Delete result:', deleteResult);
    
    // Verify the delete succeeded
    assert.ok(deleteResult.success, 'Delete operation should succeed');
    
    // Query for the post to verify it's gone
    const queryOperation = {
      type: "query",
      schema: "post",
      fields: ["id"],
      filter: { id: postId }
    };
    
    const posts = await foldDBClient.executeQuery(queryOperation);
    assert.strictEqual(posts.length, 0, 'Post should be deleted');
    
    // Remove from cleanup list since we already deleted it
    runner.createdPosts = runner.createdPosts.filter(id => id !== postId);
  });

  // Clean up test data
  await runner.cleanup();
  
  // Summarize test results
  runner.summarize();
}

// Run the tests
console.log('Starting FoldDB integration tests...');
testFoldDBIntegration().catch(error => {
  console.error('Unhandled error:', error);
  process.exit(1);
});
