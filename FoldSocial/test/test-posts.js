const { DataFoldClient } = require('datafold-client');

// Initialize DataFold client
const client = new DataFoldClient({
  baseUrl: process.env.DATAFOLD_API_URL || 'http://localhost:8080',
});

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

// Test data
const testPosts = [
  {
    id: '1',
    content: 'Hello, this is my first post on FoldSocial!',
    author: 'Alice',
    timestamp: new Date().toISOString()
  },
  {
    id: '2',
    content: 'DataFold is an amazing database system with atomic operations and schema-based storage.',
    author: 'Bob',
    timestamp: new Date(Date.now() - 3600000).toISOString() // 1 hour ago
  },
  {
    id: '3',
    content: 'Just learned about field-level permissions in DataFold. Very powerful feature!',
    author: 'Charlie',
    timestamp: new Date(Date.now() - 7200000).toISOString() // 2 hours ago
  }
];

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
    console.log('\nVerifying posts...');
    
    const posts = await client.find('Post', ['id', 'content', 'author', 'timestamp'], {});
    console.log(`Found ${posts.length} posts`);
    
    if (posts.length === testPosts.length) {
      console.log('All test posts were created successfully');
      console.log('\nPost details:');
      posts.forEach(post => {
        console.log(`- Author: ${post.author}`);
        console.log(`  Content: ${post.content}`);
        console.log(`  Timestamp: ${post.timestamp}`);
        console.log('');
      });
      return true;
    } else {
      console.error('Not all test posts were created');
      return false;
    }
  } catch (error) {
    console.error('Error verifying posts:', error);
    return false;
  }
}

// Run the tests
async function runTests() {
  console.log('Starting FoldSocial tests...\n');
  
  // Check if DataFold node is running
  const isNodeRunning = await checkDataFoldNode();
  
  if (!isNodeRunning) {
    console.error('\x1b[31m%s\x1b[0m', '⚠️  DataFold node is not running!');
    console.error('\x1b[31m%s\x1b[0m', 'Please start the DataFold node using:');
    console.error('\x1b[33m%s\x1b[0m', './setup_sandbox_local.sh');
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
    console.error('Failed to create test posts. Exiting tests.');
    return;
  }
  
  // Verify posts
  const postsVerified = await verifyPosts();
  if (!postsVerified) {
    console.error('Failed to verify posts. Exiting tests.');
    return;
  }
  
  console.log('\nAll tests completed successfully!');
  console.log('You can now run the FoldSocial app to see the test posts.');
}

// Run the tests
runTests().catch(console.error);
