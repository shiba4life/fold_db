#!/usr/bin/env node

/**
 * Integration Tests for Social App with Real DataFold Node
 * 
 * This script runs comprehensive integration tests for the Social App,
 * testing the entire flow from UI to API to a real DataFold node.
 * 
 * It starts the DataFold node, starts the server, launches a browser to interact with the UI,
 * and verifies that data is properly persisted in the DataFold node.
 * 
 * Usage:
 *   node integration-tests-with-datafold.js [--verbose] [--headless]
 */

const { spawn, fork } = require('child_process');
const path = require('path');
const puppeteer = require('puppeteer');
const fetch = require('node-fetch');
const assert = require('assert').strict;
const fs = require('fs');

// Configuration
const DATAFOLD_PORT = 8080;
const SERVER_PORT = 3003; // Use a different port to avoid conflicts
const SERVER_URL = `http://localhost:${SERVER_PORT}`;
const API_URL = `${SERVER_URL}/api`;
const DATAFOLD_API_URL = `http://localhost:${DATAFOLD_PORT}/api`;
const VERBOSE = process.argv.includes('--verbose');
const HEADLESS = process.argv.includes('--headless');

// Utility functions
function log(...args) {
  if (VERBOSE) {
    console.log(...args);
  }
}

async function executeOperation(operation) {
  const response = await fetch(`${API_URL}/execute`, {
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
class IntegrationTestRunner {
  constructor() {
    this.results = [];
    this.createdPosts = [];
    this.datafoldProcess = null;
    this.serverProcess = null;
    this.browser = null;
    this.page = null;
  }

  async setup() {
    // Create necessary directories
    const configDir = path.join(__dirname, '..', '..', 'config');
    if (!fs.existsSync(configDir)) {
      fs.mkdirSync(configDir, { recursive: true });
    }
    
    // Create a basic node config if it doesn't exist
    const configPath = path.join(configDir, 'node_config.json');
    if (!fs.existsSync(configPath)) {
      const basicConfig = {
        "node_id": "test-node",
        "storage_path": "data/test-node",
        "network": {
          "enabled": false,
          "listen_addr": "127.0.0.1:9000",
          "discovery_port": 9001,
          "bootstrap_nodes": []
        }
      };
      fs.writeFileSync(configPath, JSON.stringify(basicConfig, null, 2));
    }
    
    // Start the DataFold node
    console.log('Starting DataFold node...');
    this.datafoldProcess = spawn('cargo', ['run', '--bin', 'datafold_node'], {
      stdio: VERBOSE ? 'inherit' : 'pipe',
      shell: true
    });
    
    if (!VERBOSE && this.datafoldProcess.stdout) {
      this.datafoldProcess.stdout.on('data', (data) => {
        log(`[DATAFOLD] ${data.toString().trim()}`);
      });
    }
    
    if (this.datafoldProcess.stderr) {
      this.datafoldProcess.stderr.on('data', (data) => {
        const output = data.toString().trim();
        log(`[DATAFOLD ERROR] ${output}`);
        if (output.includes('error:')) {
          console.error(`DataFold node error: ${output}`);
        }
      });
    }
    
    // Wait for DataFold node to start
    console.log('Waiting for DataFold node to start...');
    await this.waitForService(`http://localhost:${DATAFOLD_PORT}`, 30);
    console.log('DataFold node started successfully');
    
    // Modify server.js to use real_fold_db_client.js
    console.log('Configuring server to use real FoldDB client...');
    process.env.USE_REAL_FOLDDB = 'true';
    
    // Start the server
    console.log('Starting API server...');
    this.serverProcess = fork(path.join(__dirname, 'server.js'), [], {
      env: { ...process.env, PORT: SERVER_PORT, USE_REAL_FOLDDB: 'true' },
      stdio: 'pipe'
    });
    
    // Log server output
    this.serverProcess.stdout?.on('data', (data) => {
      log(`[SERVER] ${data.toString().trim()}`);
    });
    
    this.serverProcess.stderr?.on('data', (data) => {
      console.error(`[SERVER ERROR] ${data.toString().trim()}`);
    });
    
    // Wait for server to start
    console.log('Waiting for server to start...');
    await this.waitForService(SERVER_URL, 10);
    console.log('Server started successfully');
    
    // Launch browser
    console.log('Launching browser...');
    this.browser = await puppeteer.launch({ 
      headless: HEADLESS ? 'new' : false,
      args: ['--no-sandbox', '--disable-setuid-sandbox']
    });
    this.page = await this.browser.newPage();
    
    // Set viewport size
    await this.page.setViewport({ width: 1280, height: 800 });
    
    // Enable console logging from the page
    this.page.on('console', (msg) => log(`[BROWSER CONSOLE] ${msg.text()}`));
  }

  async waitForService(url, maxRetries = 10) {
    for (let i = 0; i < maxRetries; i++) {
      try {
        const response = await fetch(url);
        if (response.ok || response.status === 404) {
          return true;
        }
      } catch (error) {
        log(`Waiting for service at ${url}... (${i + 1}/${maxRetries})`);
      }
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
    throw new Error(`Service at ${url} did not start within the timeout period`);
  }

  async teardown() {
    // Clean up test data
    await this.cleanup();
    
    // Close browser
    if (this.browser) {
      await this.browser.close();
      console.log('Browser closed');
    }
    
    // Stop server
    if (this.serverProcess && !this.serverProcess.killed) {
      this.serverProcess.kill();
      console.log('Server process terminated');
    }
    
    // Stop DataFold node
    if (this.datafoldProcess && !this.datafoldProcess.killed) {
      this.datafoldProcess.kill();
      console.log('DataFold node process terminated');
    }
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
      
      // Take screenshot on failure
      if (this.page) {
        const screenshotPath = path.join(__dirname, 'test-results', `${name.replace(/\s+/g, '-')}-failure.png`);
        await this.page.screenshot({ path: screenshotPath, fullPage: true });
        console.log(`Screenshot saved to: ${screenshotPath}`);
      }
      
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
      return 1;
    } else {
      return 0;
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
async function runIntegrationTests() {
  const runner = new IntegrationTestRunner();
  let exitCode = 0;
  
  try {
    await runner.setup();
    
    // Test 1: Navigate to the app and verify initial state
    await runner.runTest('Should load the app and display the feed view', async () => {
      await runner.page.goto(SERVER_URL, { waitUntil: 'networkidle0' });
      
      // Verify the app loaded
      const appTitle = await runner.page.$eval('h1', el => el.textContent);
      assert.strictEqual(appTitle, 'Social App', 'App title should be "Social App"');
      
      // Verify the feed view is active
      const feedView = await runner.page.$('[data-testid="feed-view"]');
      assert.ok(feedView, 'Feed view should be visible');
      
      // Verify the navigation buttons
      const feedButton = await runner.page.$('[data-testid="nav-feed"]');
      const profileButton = await runner.page.$('[data-testid="nav-profile"]');
      const friendsButton = await runner.page.$('[data-testid="nav-friends"]');
      
      assert.ok(feedButton, 'Feed button should exist');
      assert.ok(profileButton, 'Profile button should exist');
      assert.ok(friendsButton, 'Friends button should exist');
    });
    
    // Test 2: Create a post via UI and verify it's displayed
    await runner.runTest('Should create a post via UI and display it', async () => {
      // Navigate to the app
      await runner.page.goto(SERVER_URL, { waitUntil: 'networkidle0' });
      
      // Create a unique post content
      const postContent = `Integration Test Post ${Date.now()}`;
      
      // Fill in the post form
      await runner.page.type('[data-testid="post-input"]', postContent);
      
      // Click the post button
      await runner.page.click('[data-testid="post-button"]');
      
      // Wait for the post to be created and displayed
      await runner.page.waitForSelector('[data-testid="post-item"]', { timeout: 5000 });
      
      // Verify the post is displayed
      const postText = await runner.page.$eval('[data-testid="post-content"]', el => el.textContent);
      assert.strictEqual(postText, postContent, 'Post content should match what was entered');
      
      // Get the post ID from the data attribute
      const postId = await runner.page.$eval('[data-testid="post-item"]', el => el.getAttribute('data-post-id'));
      assert.ok(postId, 'Post should have an ID');
      
      // Store the post ID for later verification and cleanup
      runner.createdPosts.push(postId);
    });
    
    // Test 3: Verify the post was persisted in DataFold node
    await runner.runTest('Should persist the post in DataFold node', async () => {
      // Skip if no posts were created
      if (runner.createdPosts.length === 0) {
        throw new Error('No posts were created in the previous test');
      }
      
      const postId = runner.createdPosts[0];
      
      // Query for the post directly via the API
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
    });
    
    // Test 4: Navigate to profile view
    await runner.runTest('Should navigate to profile view', async () => {
      // Navigate to the app
      await runner.page.goto(SERVER_URL, { waitUntil: 'networkidle0' });
      
      // Click the profile button
      await runner.page.click('[data-testid="nav-profile"]');
      
      // Wait for the profile view to be displayed
      await runner.page.waitForSelector('[data-testid="profile-view"]', { timeout: 5000 });
      
      // Verify the profile view is active
      const profileView = await runner.page.$('[data-testid="profile-view"]');
      assert.ok(profileView, 'Profile view should be visible');
      
      // Verify profile information is displayed
      const profileName = await runner.page.$('[data-testid="profile-name"]');
      assert.ok(profileName, 'Profile name should be displayed');
    });
    
    // Test 5: Navigate to friends view
    await runner.runTest('Should navigate to friends view', async () => {
      // Navigate to the app
      await runner.page.goto(SERVER_URL, { waitUntil: 'networkidle0' });
      
      // Click the friends button
      await runner.page.click('[data-testid="nav-friends"]');
      
      // Wait for the friends view to be displayed
      await runner.page.waitForSelector('[data-testid="friends-view"]', { timeout: 5000 });
      
      // Verify the friends view is active
      const friendsView = await runner.page.$('[data-testid="friends-view"]');
      assert.ok(friendsView, 'Friends view should be visible');
      
      // Verify friends list is displayed
      const friendsList = await runner.page.$('[data-testid="friends-list"]');
      assert.ok(friendsList, 'Friends list should be displayed');
    });
    
    // Test 6: Like a post
    await runner.runTest('Should like a post', async () => {
      // Navigate to the app
      await runner.page.goto(SERVER_URL, { waitUntil: 'networkidle0' });
      
      // Wait for posts to load
      await runner.page.waitForSelector('[data-testid="post-item"]', { timeout: 5000 });
      
      // Get the initial like count
      const initialLikeCount = await runner.page.$eval('[data-testid="like-count"]', el => parseInt(el.textContent, 10) || 0);
      
      // Click the like button
      await runner.page.click('[data-testid="like-button"]');
      
      // Wait for the like to be processed
      await runner.page.waitForTimeout(1000);
      
      // Get the updated like count
      const updatedLikeCount = await runner.page.$eval('[data-testid="like-count"]', el => parseInt(el.textContent, 10) || 0);
      
      // Verify the like count increased
      assert.strictEqual(updatedLikeCount, initialLikeCount + 1, 'Like count should increase by 1');
      
      // Verify the like was persisted in DataFold node
      if (runner.createdPosts.length > 0) {
        const postId = runner.createdPosts[0];
        
        // Query for the post directly via the API
        const queryOperation = {
          type: "query",
          schema: "post",
          fields: ["id", "likes"],
          filter: { id: postId }
        };
        
        const queryResult = await executeOperation(queryOperation);
        const post = queryResult.data.results[0];
        
        assert.ok(Array.isArray(post.likes), 'Post likes should be an array');
        assert.ok(post.likes.length > 0, 'Post should have at least one like');
      }
    });
    
    // Summarize test results
    exitCode = runner.summarize();
    
  } catch (error) {
    console.error('Unhandled error:', error);
    exitCode = 1;
  } finally {
    // Clean up
    await runner.teardown();
  }
  
  return exitCode;
}

// Run the tests
console.log('Starting integration tests with DataFold node...');
runIntegrationTests().then(exitCode => {
  process.exit(exitCode);
}).catch(error => {
  console.error('Fatal error:', error);
  process.exit(1);
});
