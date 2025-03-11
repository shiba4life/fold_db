const { DataFoldClient } = require('datafold-client');
const fs = require('fs');
const path = require('path');
const { spawn } = require('child_process');
const assert = require('assert');

// Initialize DataFold client
const client = new DataFoldClient({
  baseUrl: process.env.DATAFOLD_API_URL || 'http://localhost:8080',
});

// Path to the data file
const dataFilePath = path.join(__dirname, '../data/data.json');

// Test data
const testPosts = [
  {
    id: 'test-1',
    content: 'This is a test post for persistence testing',
    author: 'Persistence Tester',
    timestamp: new Date().toISOString()
  },
  {
    id: 'test-2',
    content: 'Another test post to verify data persistence',
    author: 'Persistence Tester',
    timestamp: new Date(Date.now() - 3600000).toISOString() // 1 hour ago
  }
];

// Start the DataFold Node
async function startDataFoldNode() {
  console.log('Starting DataFold Node...');
  
  // Check if the node is already running
  try {
    await client.listSchemas();
    console.log('DataFold Node is already running');
    return null; // Return null to indicate we didn't start a new process
  } catch (error) {
    // Node is not running, start it
    console.log('DataFold Node is not running, starting it...');
    
    // Use the start-node.sh script to start the node
    const nodeProcess = spawn('sh', ['./start-node.sh'], {
      stdio: 'inherit',
      detached: true
    });
    
    // Wait for the node to start
    await new Promise(resolve => setTimeout(resolve, 5000));
    
    console.log('DataFold Node started');
    return nodeProcess;
  }
}

// Check if DataFold node is running
async function checkDataFoldNode() {
  try {
    await client.listSchemas();
    return true;
  } catch (error) {
    console.error('\x1b[31m%s\x1b[0m', 'Error connecting to DataFold node:', error.message);
    return false;
  }
}

// Define the Post schema if it doesn't exist
async function ensurePostSchema() {
  try {
    const schemas = await client.listSchemas();
    
    if (!schemas.includes('Post')) {
      console.log('Creating Post schema...');
      
      const postSchema = {
        name: 'Post',
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
          content: {
            permission_policy: {
              read_policy: { NoRequirement: null },
              write_policy: { NoRequirement: null }
            },
            payment_config: {
              base_multiplier: 1.0
            },
            field_mappers: {}
          },
          author: {
            permission_policy: {
              read_policy: { NoRequirement: null },
              write_policy: { NoRequirement: null }
            },
            payment_config: {
              base_multiplier: 1.0
            },
            field_mappers: {}
          },
          timestamp: {
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
      
      await client.createSchema(postSchema);
      console.log('Post schema created successfully');
      return true;
    } else {
      console.log('Post schema already exists');
      return true;
    }
  } catch (error) {
    console.error('Error ensuring Post schema:', error);
    return false;
  }
}

// Create test posts
async function createTestPosts() {
  try {
    console.log('Creating test posts...');
    
    // First, clear any existing posts
    await client.delete('Post', {});
    console.log('Cleared existing posts');
    
    // Create new test posts
    for (const post of testPosts) {
      const result = await client.create('Post', post);
      if (result.success) {
        console.log(`Created post by ${post.author}`);
      } else {
        console.error(`Failed to create post by ${post.author}:`, result.error);
      }
    }
    
    console.log('Test posts created successfully');
    return true;
  } catch (error) {
    console.error('Error creating test posts:', error);
    return false;
  }
}

// Verify posts were created
async function verifyPosts() {
  try {
    console.log('\nVerifying posts in memory...');
    
    const posts = await client.find('Post', ['id', 'content', 'author', 'timestamp'], {});
    console.log(`Found ${posts.length} posts in memory`);
    
    if (posts.length === testPosts.length) {
      console.log('All test posts were created successfully in memory');
      console.log('\nPost details:');
      posts.forEach(post => {
        console.log(`- Author: ${post.author}`);
        console.log(`  Content: ${post.content}`);
        console.log(`  Timestamp: ${post.timestamp}`);
        console.log('');
      });
      return true;
    } else {
      console.error('Not all test posts were created in memory');
      return false;
    }
  } catch (error) {
    console.error('Error verifying posts in memory:', error);
    return false;
  }
}

// Verify data was saved to disk
async function verifyDataPersistence() {
  try {
    console.log('\nVerifying data persistence on disk...');
    
    // Check if the data file exists
    if (!fs.existsSync(dataFilePath)) {
      console.error(`Data file not found: ${dataFilePath}`);
      return false;
    }
    
    // Read the data file
    const dataFileContent = fs.readFileSync(dataFilePath, 'utf8');
    const data = JSON.parse(dataFileContent);
    
    // Check if the Post schema data exists
    if (!data.Post || !Array.isArray(data.Post)) {
      console.error('Post schema data not found in data file');
      return false;
    }
    
    // Check if all test posts are in the data file
    const savedPosts = data.Post;
    console.log(`Found ${savedPosts.length} posts in data file`);
    
    // Verify each test post is in the saved data
    for (const testPost of testPosts) {
      const foundPost = savedPosts.find(post => post.id === testPost.id);
      
      if (!foundPost) {
        console.error(`Test post with ID ${testPost.id} not found in saved data`);
        return false;
      }
      
      // Verify post content
      if (foundPost.content !== testPost.content) {
        console.error(`Content mismatch for post ${testPost.id}`);
        console.error(`Expected: ${testPost.content}`);
        console.error(`Found: ${foundPost.content}`);
        return false;
      }
      
      // Verify post author
      if (foundPost.author !== testPost.author) {
        console.error(`Author mismatch for post ${testPost.id}`);
        console.error(`Expected: ${testPost.author}`);
        console.error(`Found: ${foundPost.author}`);
        return false;
      }
      
      console.log(`Verified post ${testPost.id} was saved correctly`);
    }
    
    console.log('\x1b[32m%s\x1b[0m', '✅ All test posts were saved to disk correctly!');
    return true;
  } catch (error) {
    console.error('Error verifying data persistence:', error);
    return false;
  }
}

// Run the tests
async function runTests() {
  console.log('Starting DataFold Node persistence tests...\n');
  
  // Start the DataFold Node
  const nodeProcess = await startDataFoldNode();
  
  // Check if DataFold node is running
  const isNodeRunning = await checkDataFoldNode();
  
  if (!isNodeRunning) {
    console.error('\x1b[31m%s\x1b[0m', '⚠️  DataFold node is not running!');
    console.error('\x1b[31m%s\x1b[0m', 'Please start the DataFold node using:');
    console.error('\x1b[33m%s\x1b[0m', './start-node.sh');
    console.error('\x1b[31m%s\x1b[0m', 'Tests cannot proceed without a running DataFold node.');
    return;
  }
  
  console.log('\x1b[32m%s\x1b[0m', '✅ Connected to DataFold node successfully!');
  
  // Ensure schema exists
  const schemaCreated = await ensurePostSchema();
  if (!schemaCreated) {
    console.error('\x1b[31m%s\x1b[0m', 'Failed to create schema. Exiting tests.');
    return;
  }
  
  // Create test posts
  const postsCreated = await createTestPosts();
  if (!postsCreated) {
    console.error('\x1b[31m%s\x1b[0m', 'Failed to create test posts. Exiting tests.');
    return;
  }
  
  // Verify posts in memory
  const postsVerified = await verifyPosts();
  if (!postsVerified) {
    console.error('\x1b[31m%s\x1b[0m', 'Failed to verify posts in memory. Exiting tests.');
    return;
  }
  
  // Verify data persistence
  const dataPersisted = await verifyDataPersistence();
  if (!dataPersisted) {
    console.error('\x1b[31m%s\x1b[0m', 'Failed to verify data persistence. Exiting tests.');
    return;
  }
  
  console.log('\n\x1b[32m%s\x1b[0m', '🎉 All tests completed successfully!');
  console.log('\x1b[32m%s\x1b[0m', '✅ DataFold Node is running correctly');
  console.log('\x1b[32m%s\x1b[0m', '✅ Schema creation is working');
  console.log('\x1b[32m%s\x1b[0m', '✅ Data queries are working');
  console.log('\x1b[32m%s\x1b[0m', '✅ Data is being saved to disk correctly');
  
  // If we started the node process, kill it
  if (nodeProcess) {
    console.log('Stopping DataFold Node...');
    process.kill(-nodeProcess.pid);
    console.log('DataFold Node stopped');
  }
}

// Run the tests
runTests().catch(console.error);
