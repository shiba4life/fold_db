#!/usr/bin/env node

/**
 * Single Test Runner for Social App
 * 
 * This script allows running a single test case directly from the command line.
 * It's useful for debugging specific issues without running the entire test suite.
 * 
 * Usage:
 *   node run-single-test.js --test="Test name" [--headless]
 * 
 * Example:
 *   node run-single-test.js --test="Should switch to profile view"
 */

const puppeteer = require('puppeteer');
const path = require('path');
const http = require('http');
const handler = require('serve-handler');

// Parse command line arguments
const args = process.argv.slice(2);
const testName = args.find(arg => arg.startsWith('--test='))?.split('=')[1];
const headless = args.includes('--headless');

if (!testName) {
  console.error('Error: Test name is required');
  console.log('Usage: node run-single-test.js --test="Test name" [--headless]');
  process.exit(1);
}

// Configuration
const PORT = 3000;
const TEST_TIMEOUT = 60000; // 60 seconds

// Create a simple HTTP server to serve the app files
const server = http.createServer((request, response) => {
  return handler(request, response, {
    public: path.resolve(__dirname),
  });
});

// Test data (same as in test-harness.html)
const testData = {
  profile: {
    id: '1',
    username: 'alice',
    bio: 'Software developer and Datafold enthusiast'
  },
  posts: [
    {
      id: '1',
      content: 'Test post 1',
      author: { id: '1', username: 'testuser' },
      timestamp: new Date().toISOString(),
      likes: [],
      comments: []
    },
    {
      id: '2',
      content: 'Test post 2',
      author: { id: '2', username: 'otheruser' },
      timestamp: new Date().toISOString(),
      likes: [{ id: '1', username: 'testuser' }],
      comments: []
    }
  ],
  friends: [
    { id: '2', username: 'friend1', bio: 'Test friend 1' },
    { id: '3', username: 'friend2', bio: 'Test friend 2' }
  ],
  suggestions: [
    { id: '4', username: 'suggestion1', bio: 'Test suggestion 1' },
    { id: '5', username: 'suggestion2', bio: 'Test suggestion 2' }
  ]
};

// Mock APIs for testing
const mockApis = {
  data: {
    query: async (schema, filter, fields) => {
      console.log('Mock API query:', { schema, filter, fields });
      
      if (schema === 'user-profile') {
        return { results: [testData.profile] };
      } else if (schema === 'post') {
        if (filter && filter.author && filter.author.id === testData.profile.id) {
          return { 
            results: testData.posts.filter(p => p.author.id === testData.profile.id) 
          };
        } else {
          return { results: testData.posts };
        }
      }
      
      return { results: [] };
    },
    mutate: async (schema, data, mutationType) => {
      console.log('Mock API mutate:', { schema, data, mutationType });
      
      if (schema === 'post' && mutationType === 'create') {
        const newPost = {
          id: Date.now().toString(),
          content: data.content,
          author: testData.profile,
          timestamp: new Date().toISOString(),
          likes: [],
          comments: []
        };
        
        testData.posts.unshift(newPost);
        return { success: true, data: newPost };
      }
      
      return { success: true };
    }
  }
};

// Test runner
class SingleTestRunner {
  constructor(page) {
    this.page = page;
  }
  
  async setup() {
    // Navigate to the app
    await this.page.goto(`http://localhost:${PORT}/index.html`, {
      waitUntil: 'networkidle2',
      timeout: TEST_TIMEOUT
    });
    
    // Initialize the app with test data
    await this.page.evaluate((testData, mockApis) => {
      window.testMode = true;
      window.testData = testData;
      window.apis = mockApis;
      
      // If the app is already initialized, call initializeApp directly
      if (typeof initializeApp === 'function') {
        initializeApp();
      }
    }, testData, mockApis);
    
    // Wait for app to initialize
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    console.log('App initialized with test data');
  }
  
  // Helper methods for assertions
  async assertElementVisible(selector) {
    const element = await this.page.$(selector);
    if (!element) {
      throw new Error(`Element not found: ${selector}`);
    }
    
    const isHidden = await this.page.evaluate(el => {
      return el.classList.contains('hidden');
    }, element);
    
    if (isHidden) {
      throw new Error(`Element is hidden: ${selector}`);
    }
    
    return element;
  }
  
  async assertElementHidden(selector) {
    const element = await this.page.$(selector);
    if (!element) {
      throw new Error(`Element not found: ${selector}`);
    }
    
    const isHidden = await this.page.evaluate(el => {
      return el.classList.contains('hidden');
    }, element);
    
    if (!isHidden) {
      throw new Error(`Element is visible but should be hidden: ${selector}`);
    }
    
    return element;
  }
  
  async assertElementExists(selector) {
    const element = await this.page.$(selector);
    if (!element) {
      throw new Error(`Element not found: ${selector}`);
    }
    return element;
  }
  
  async assertElementText(selector, expectedText) {
    const element = await this.page.$(selector);
    if (!element) {
      throw new Error(`Element not found: ${selector}`);
    }
    
    const actualText = await this.page.evaluate(el => el.textContent.trim(), element);
    if (actualText !== expectedText) {
      throw new Error(`Element text mismatch for ${selector}. Expected: "${expectedText}", Actual: "${actualText}"`);
    }
    
    return element;
  }
  
  async clickElement(selector) {
    await this.page.click(selector);
    // Wait for any UI updates
    await new Promise(resolve => setTimeout(resolve, 300));
  }
  
  async typeInElement(selector, text) {
    await this.page.type(selector, text);
    // Wait for any UI updates
    await new Promise(resolve => setTimeout(resolve, 100));
  }
}

// Test cases
const testCases = {
  // Navigation tests
  "Initial view should be feed": async (runner) => {
    await runner.assertElementVisible('[data-testid="feed-view"]');
    await runner.assertElementHidden('[data-testid="profile-view"]');
    await runner.assertElementHidden('[data-testid="friends-view"]');
  },
  
  "Should switch to profile view": async (runner) => {
    await runner.clickElement('[data-testid="profile-btn"]');
    await runner.assertElementHidden('[data-testid="feed-view"]');
    await runner.assertElementVisible('[data-testid="profile-view"]');
    await runner.assertElementHidden('[data-testid="friends-view"]');
  },
  
  "Should switch to friends view": async (runner) => {
    await runner.clickElement('[data-testid="friends-btn"]');
    await runner.assertElementHidden('[data-testid="feed-view"]');
    await runner.assertElementHidden('[data-testid="profile-view"]');
    await runner.assertElementVisible('[data-testid="friends-view"]');
  },
  
  "Should switch back to feed view": async (runner) => {
    await runner.clickElement('[data-testid="feed-btn"]');
    await runner.assertElementVisible('[data-testid="feed-view"]');
    await runner.assertElementHidden('[data-testid="profile-view"]');
    await runner.assertElementHidden('[data-testid="friends-view"]');
  },
  
  // Post tests
  "Should display posts in feed": async (runner) => {
    const postsContainer = await runner.assertElementExists('[data-testid="posts-container"]');
    const posts = await runner.page.$$('[data-testid="posts-container"] .post');
    if (posts.length !== 2) {
      throw new Error(`Expected 2 posts, found ${posts.length}`);
    }
  },
  
  "Should create a new post": async (runner) => {
    const testContent = 'This is a test post ' + Date.now();
    await runner.typeInElement('[data-testid="post-content"]', testContent);
    await runner.clickElement('[data-testid="post-btn"]');
    
    // Wait for post creation and refresh
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Check if the new post appears at the top
    const firstPostContent = await runner.page.$eval('[data-testid="posts-container"] .post:first-child .post-content', el => el.textContent);
    
    if (!firstPostContent.includes(testContent)) {
      throw new Error(`New post content not found. Expected: "${testContent}", Found: "${firstPostContent}"`);
    }
  },
  
  // Profile tests
  "Should display profile information": async (runner) => {
    await runner.clickElement('[data-testid="profile-btn"]');
    
    // Wait for profile data to load
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    await runner.assertElementText('[data-testid="profile-username"]', testData.profile.username);
    await runner.assertElementText('[data-testid="profile-bio"]', testData.profile.bio);
  },
  
  "Should display user posts in profile": async (runner) => {
    await runner.clickElement('[data-testid="profile-btn"]');
    
    // Wait for profile data to load
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    const posts = await runner.page.$$('[data-testid="my-posts-container"] .post');
    
    // We expect at least one post from the test user
    if (posts.length < 1) {
      throw new Error(`Expected at least 1 user post, found ${posts.length}`);
    }
  },
  
  // Friend tests
  "Should display friends list": async (runner) => {
    await runner.clickElement('[data-testid="friends-btn"]');
    
    // Wait for friends data to load
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    const friends = await runner.page.$$('[data-testid="friends-container"] .friend');
    
    if (friends.length !== 2) {
      throw new Error(`Expected 2 friends, found ${friends.length}`);
    }
  },
  
  "Should display friend suggestions": async (runner) => {
    await runner.clickElement('[data-testid="friends-btn"]');
    
    // Wait for friends data to load
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    const suggestions = await runner.page.$$('[data-testid="suggestions-container"] .friend');
    
    if (suggestions.length !== 2) {
      throw new Error(`Expected 2 friend suggestions, found ${suggestions.length}`);
    }
  }
};

// Main function
async function runSingleTest() {
  console.log(`Running test: "${testName}"`);
  console.log(`Headless mode: ${headless ? 'enabled' : 'disabled'}`);
  
  let browser;
  let exitCode = 0;
  
  try {
    // Start the server
    await new Promise((resolve) => {
      server.listen(PORT, () => {
        console.log(`Server running at http://localhost:${PORT}`);
        resolve();
      });
    });
    
    // Launch browser
    browser = await puppeteer.launch({
      headless: headless ? 'new' : false,
      args: ['--no-sandbox', '--disable-setuid-sandbox'],
      defaultViewport: { width: 1024, height: 768 }
    });
    
    const page = await browser.newPage();
    
    // Set up console logging
    page.on('console', message => {
      const type = message.type().substr(0, 3).toUpperCase();
      const text = message.text();
      
      // Filter out noisy messages
      if (text.includes('webpack') || text.includes('DevTools')) return;
      
      console.log(`[BROWSER ${type}] ${text}`);
    });
    
    // Create test runner
    const runner = new SingleTestRunner(page);
    
    // Set up the test environment
    await runner.setup();
    
    // Find the test case
    const testCase = testCases[testName];
    if (!testCase) {
      throw new Error(`Test case not found: "${testName}"`);
    }
    
    // Run the test
    console.log(`Executing test: "${testName}"`);
    await testCase(runner);
    
    console.log(`\n✅ Test passed: "${testName}"`);
    
  } catch (error) {
    console.error(`\n❌ Test failed: "${testName}"`);
    console.error(`Error: ${error.message}`);
    exitCode = 1;
  } finally {
    // Clean up
    if (browser) {
      await browser.close();
    }
    
    server.close(() => {
      console.log('Server closed');
      process.exit(exitCode);
    });
  }
}

// Run the test
runSingleTest();
