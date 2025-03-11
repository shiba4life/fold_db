const { DataFoldClient } = require('datafold-client');
const path = require('path');
const fs = require('fs');

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

// Load schema from file
async function loadSchemaFromFile() {
  try {
    console.log('Loading Post schema from file...');
    
    const schemaPath = path.join(__dirname, '../data/post-schema.json');
    
    // Check if the schema file exists
    if (!fs.existsSync(schemaPath)) {
      console.error('\x1b[31m%s\x1b[0m', `Schema file not found: ${schemaPath}`);
      return false;
    }
    
    // Load schema from file
    const result = await client.loadSchemaFromFile(schemaPath);
    console.log('\x1b[32m%s\x1b[0m', `Schema loaded from file: ${result.schema_name} - ${result.message}`);
    return true;
  } catch (error) {
    console.error('\x1b[31m%s\x1b[0m', 'Error loading schema from file:', error);
    return false;
  }
}

// Create a test post
async function createTestPost() {
  try {
    console.log('\nCreating a test post...');
    
    const postData = {
      id: Date.now().toString(),
      content: 'This is a test post created using schema loading functionality',
      author: 'Schema Loader',
      timestamp: new Date().toISOString()
    };
    
    const result = await client.create('Post', postData);
    
    if (result.success) {
      console.log('\x1b[32m%s\x1b[0m', 'Test post created successfully');
      return true;
    } else {
      console.error('\x1b[31m%s\x1b[0m', 'Failed to create test post:', result.error);
      return false;
    }
  } catch (error) {
    console.error('\x1b[31m%s\x1b[0m', 'Error creating test post:', error);
    return false;
  }
}

// Fetch and display all posts
async function fetchPosts() {
  try {
    console.log('\nFetching all posts...');
    
    const posts = await client.find('Post', ['id', 'content', 'author', 'timestamp'], {});
    
    // Sort posts by timestamp in descending order (newest first)
    posts.sort((a, b) => new Date(b.timestamp) - new Date(a.timestamp));
    
    console.log('\x1b[32m%s\x1b[0m', `Found ${posts.length} posts:`);
    
    posts.forEach((post, index) => {
      console.log(`\n--- Post ${index + 1} ---`);
      console.log(`Author: ${post.author}`);
      console.log(`Content: ${post.content}`);
      console.log(`Timestamp: ${new Date(post.timestamp).toLocaleString()}`);
    });
    
    return true;
  } catch (error) {
    console.error('\x1b[31m%s\x1b[0m', 'Error fetching posts:', error);
    return false;
  }
}

// Run the test
async function runTest() {
  console.log('Starting schema loading test...\n');
  
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
  
  // Load schema from file
  const schemaLoaded = await loadSchemaFromFile();
  
  if (!schemaLoaded) {
    console.error('\x1b[31m%s\x1b[0m', 'Failed to load schema. Exiting test.');
    return;
  }
  
  // Create a test post
  const postCreated = await createTestPost();
  
  if (!postCreated) {
    console.error('\x1b[31m%s\x1b[0m', 'Failed to create test post. Exiting test.');
    return;
  }
  
  // Fetch and display posts
  const postsFetched = await fetchPosts();
  
  if (!postsFetched) {
    console.error('\x1b[31m%s\x1b[0m', 'Failed to fetch posts. Exiting test.');
    return;
  }
  
  console.log('\n\x1b[32m%s\x1b[0m', '✅ Schema loading test completed successfully!');
  console.log('\x1b[32m%s\x1b[0m', 'You can now run the FoldSocial app to see the posts.');
}

// Run the test
runTest().catch(console.error);
