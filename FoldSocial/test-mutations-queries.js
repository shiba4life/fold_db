const { DataFoldClient } = require('datafold-client');

// Initialize DataFold client
const client = new DataFoldClient({
  baseUrl: process.env.DATAFOLD_API_URL || 'http://localhost:8080',
});

// Test data
const testPost = {
  id: 'test-mutation-1',
  content: 'This is a test post for mutation testing',
  author: 'Mutation Tester',
  timestamp: new Date().toISOString()
};

// Test schema
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

// Test functions
async function testListSchemas() {
  console.log('\n--- Testing List Schemas ---');
  try {
    const schemas = await client.listSchemas();
    console.log('Available schemas:', schemas);
    return schemas;
  } catch (error) {
    console.error('Error listing schemas:', error);
    return [];
  }
}

async function testCreateSchema() {
  console.log('\n--- Testing Create Schema ---');
  try {
    const result = await client.createSchema(postSchema);
    console.log('Schema creation result:', result);
    return result;
  } catch (error) {
    console.error('Error creating schema:', error);
    return false;
  }
}

async function testCreatePost() {
  console.log('\n--- Testing Create Post (Mutation) ---');
  try {
    const result = await client.create('Post', testPost);
    console.log('Create post result:', result);
    return result;
  } catch (error) {
    console.error('Error creating post:', error);
    return { success: false };
  }
}

async function testQueryPosts() {
  console.log('\n--- Testing Query Posts ---');
  try {
    const posts = await client.find('Post', ['id', 'content', 'author', 'timestamp'], {});
    console.log(`Found ${posts.length} posts`);
    posts.forEach(post => {
      console.log(`- ID: ${post.id}`);
      console.log(`  Author: ${post.author}`);
      console.log(`  Content: ${post.content}`);
      console.log(`  Timestamp: ${post.timestamp}`);
      console.log('');
    });
    return posts;
  } catch (error) {
    console.error('Error querying posts:', error);
    return [];
  }
}

async function testUpdatePost() {
  console.log('\n--- Testing Update Post (Mutation) ---');
  try {
    const updatedContent = 'This post has been updated';
    const result = await client.update('Post', { id: testPost.id }, { content: updatedContent });
    console.log('Update post result:', result);
    return result;
  } catch (error) {
    console.error('Error updating post:', error);
    return { success: false };
  }
}

async function testDeletePost() {
  console.log('\n--- Testing Delete Post (Mutation) ---');
  try {
    const result = await client.delete('Post', { id: testPost.id });
    console.log('Delete post result:', result);
    return result;
  } catch (error) {
    console.error('Error deleting post:', error);
    return { success: false };
  }
}

async function testDirectMutation() {
  console.log('\n--- Testing Direct Mutation ---');
  try {
    const mutation = {
      type: 'mutation',
      schema: 'Post',
      operation: 'create',
      data: {
        id: 'direct-mutation-1',
        content: 'This post was created using a direct mutation',
        author: 'Direct Mutation Tester',
        timestamp: new Date().toISOString()
      }
    };
    
    const result = await client.mutate(mutation);
    console.log('Direct mutation result:', result);
    return result;
  } catch (error) {
    console.error('Error with direct mutation:', error);
    return { success: false };
  }
}

async function testDirectQuery() {
  console.log('\n--- Testing Direct Query ---');
  try {
    const query = {
      type: 'query',
      schema: 'Post',
      fields: ['id', 'content', 'author', 'timestamp'],
      filter: null
    };
    
    const result = await client.query(query);
    console.log(`Direct query found ${result.results.length} posts`);
    console.log('Query results:', result.results);
    return result;
  } catch (error) {
    console.error('Error with direct query:', error);
    return { results: [], count: 0 };
  }
}

// Run all tests
async function runTests() {
  console.log('Starting DataFold mutations and queries tests...\n');
  
  // List schemas
  const schemas = await testListSchemas();
  
  // Create schema if it doesn't exist
  if (!schemas.includes('Post')) {
    await testCreateSchema();
  } else {
    console.log('Post schema already exists, skipping creation');
  }
  
  // Test mutations
  await testCreatePost();
  await testQueryPosts();
  await testUpdatePost();
  await testQueryPosts();
  await testDeletePost();
  
  // Test direct operations
  await testDirectMutation();
  await testDirectQuery();
  
  console.log('\nAll tests completed!');
}

// Run the tests
runTests().catch(console.error);
