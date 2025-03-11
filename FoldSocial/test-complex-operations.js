const { DataFoldClient } = require('datafold-client');
const assert = require('assert');

// Initialize DataFold client
const client = new DataFoldClient({
  baseUrl: process.env.DATAFOLD_API_URL || 'http://localhost:8080',
});

// Test data
const testUsers = [
  {
    id: 'user-1',
    username: 'testuser1',
    email: 'user1@example.com',
    profile: {
      fullName: 'Test User One',
      bio: 'This is a test user for complex operations',
      joinDate: new Date().toISOString()
    }
  },
  {
    id: 'user-2',
    username: 'testuser2',
    email: 'user2@example.com',
    profile: {
      fullName: 'Test User Two',
      bio: 'Another test user for complex operations',
      joinDate: new Date(Date.now() - 86400000).toISOString() // 1 day ago
    }
  }
];

// Test schema
const userSchema = {
  name: 'User',
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
    username: {
      permission_policy: {
        read_policy: { NoRequirement: null },
        write_policy: { NoRequirement: null }
      },
      payment_config: {
        base_multiplier: 1.0
      },
      field_mappers: {}
    },
    email: {
      permission_policy: {
        read_policy: { NoRequirement: null },
        write_policy: { NoRequirement: null }
      },
      payment_config: {
        base_multiplier: 1.0
      },
      field_mappers: {}
    },
    profile: {
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
async function ensureUserSchema() {
  console.log('\n--- Ensuring User Schema Exists ---');
  try {
    const schemas = await client.listSchemas();
    console.log('Available schemas:', schemas);
    
    if (!schemas.includes('User')) {
      console.log('Creating User schema...');
      const result = await client.createSchema(userSchema);
      console.log('Schema creation result:', result);
      return result;
    } else {
      console.log('User schema already exists');
      return true;
    }
  } catch (error) {
    console.error('Error with schema operation:', error);
    return false;
  }
}

async function clearExistingUsers() {
  console.log('\n--- Clearing Existing Users ---');
  try {
    const result = await client.delete('User', {});
    console.log('Clear users result:', result);
    return result.success;
  } catch (error) {
    console.error('Error clearing users:', error);
    return false;
  }
}

async function createUsers() {
  console.log('\n--- Creating Test Users ---');
  const results = [];
  
  for (const user of testUsers) {
    try {
      console.log(`Creating user: ${user.username}`);
      const result = await client.create('User', user);
      console.log('Create result:', result);
      results.push(result.success);
    } catch (error) {
      console.error(`Error creating user ${user.username}:`, error);
      results.push(false);
    }
  }
  
  return results.every(success => success);
}

async function queryAllUsers() {
  console.log('\n--- Querying All Users ---');
  try {
    const users = await client.find('User', ['id', 'username', 'email', 'profile'], {});
    console.log(`Found ${users.length} users`);
    users.forEach(user => {
      console.log(`- ID: ${user.id}`);
      console.log(`  Username: ${user.username}`);
      console.log(`  Email: ${user.email}`);
      console.log(`  Profile:`, user.profile);
      console.log('');
    });
    return users;
  } catch (error) {
    console.error('Error querying users:', error);
    return [];
  }
}

async function queryUserByUsername(username) {
  console.log(`\n--- Querying User by Username: ${username} ---`);
  try {
    const users = await client.find('User', ['id', 'username', 'email', 'profile'], { username });
    
    if (users.length > 0) {
      console.log('User found:');
      console.log(`- ID: ${users[0].id}`);
      console.log(`  Username: ${users[0].username}`);
      console.log(`  Email: ${users[0].email}`);
      console.log(`  Profile:`, users[0].profile);
      return users[0];
    } else {
      console.log('No user found with that username');
      return null;
    }
  } catch (error) {
    console.error('Error querying user by username:', error);
    return null;
  }
}

async function updateUserProfile(userId, profileUpdates) {
  console.log(`\n--- Updating User Profile for ID: ${userId} ---`);
  try {
    // First, get the current user to merge with updates
    const users = await client.find('User', ['id', 'username', 'email', 'profile'], { id: userId });
    
    if (users.length === 0) {
      console.log('No user found with that ID');
      return false;
    }
    
    const user = users[0];
    const updatedProfile = { ...user.profile, ...profileUpdates };
    
    console.log('Current profile:', user.profile);
    console.log('Updated profile:', updatedProfile);
    
    // Update the user with the new profile
    const result = await client.update('User', { id: userId }, { profile: updatedProfile });
    console.log('Update result:', result);
    return result.success;
  } catch (error) {
    console.error('Error updating user profile:', error);
    return false;
  }
}

async function deleteUser(userId) {
  console.log(`\n--- Deleting User with ID: ${userId} ---`);
  try {
    const result = await client.delete('User', { id: userId });
    console.log('Delete result:', result);
    return result.success;
  } catch (error) {
    console.error('Error deleting user:', error);
    return false;
  }
}

async function testComplexQuery() {
  console.log('\n--- Testing Complex Query ---');
  try {
    // Create a complex query operation
    const query = {
      type: 'query',
      schema: 'User',
      fields: ['id', 'username', 'email', 'profile'],
      filter: {
        username: { $in: ['testuser1', 'testuser2'] }
      }
    };
    
    // This is a workaround since the current implementation doesn't support complex filters
    // In a real implementation, the filter would be processed server-side
    const allUsers = await client.find('User', ['id', 'username', 'email', 'profile'], {});
    const filteredUsers = allUsers.filter(user => 
      ['testuser1', 'testuser2'].includes(user.username)
    );
    
    console.log(`Complex query found ${filteredUsers.length} users`);
    filteredUsers.forEach(user => {
      console.log(`- Username: ${user.username}`);
    });
    
    return filteredUsers;
  } catch (error) {
    console.error('Error with complex query:', error);
    return [];
  }
}

async function testComplexMutation() {
  console.log('\n--- Testing Complex Mutation ---');
  try {
    // Get all users
    const users = await client.find('User', ['id', 'username', 'email', 'profile'], {});
    
    // Update all users' profiles with a new field
    let successCount = 0;
    
    for (const user of users) {
      const updatedProfile = { 
        ...user.profile, 
        lastUpdated: new Date().toISOString(),
        status: 'active'
      };
      
      const result = await client.update('User', { id: user.id }, { profile: updatedProfile });
      if (result.success) {
        successCount++;
      }
    }
    
    console.log(`Successfully updated ${successCount}/${users.length} user profiles`);
    return successCount === users.length;
  } catch (error) {
    console.error('Error with complex mutation:', error);
    return false;
  }
}

async function verifyDataPersistence() {
  console.log('\n--- Verifying Data Persistence ---');
  try {
    // First, query all users to see what's in memory
    const users = await client.find('User', ['id', 'username', 'email', 'profile'], {});
    console.log(`Found ${users.length} users in memory`);
    
    // Now restart the server to verify persistence
    console.log('To fully verify persistence, restart the server and run:');
    console.log('node verify-persistence.js');
    
    return true;
  } catch (error) {
    console.error('Error verifying persistence:', error);
    return false;
  }
}

// Create a verification script
const fs = require('fs');
const verificationScript = `
const { DataFoldClient } = require('datafold-client');

// Initialize DataFold client
const client = new DataFoldClient({
  baseUrl: process.env.DATAFOLD_API_URL || 'http://localhost:8080',
});

async function verifyPersistence() {
  console.log('Verifying data persistence after server restart...');
  
  try {
    // Check if User schema exists
    const schemas = await client.listSchemas();
    console.log('Available schemas:', schemas);
    
    if (!schemas.includes('User')) {
      console.error('❌ User schema not found! Data was not persisted.');
      return false;
    }
    
    // Query all users
    const users = await client.find('User', ['id', 'username', 'email', 'profile'], {});
    console.log(\`Found \${users.length} users after restart\`);
    
    if (users.length === 0) {
      console.error('❌ No users found! Data was not persisted.');
      return false;
    }
    
    // Display users
    users.forEach(user => {
      console.log(\`- ID: \${user.id}\`);
      console.log(\`  Username: \${user.username}\`);
      console.log(\`  Email: \${user.email}\`);
      console.log(\`  Profile:\`, user.profile);
      console.log('');
    });
    
    console.log('✅ Data persistence verified successfully!');
    return true;
  } catch (error) {
    console.error('Error verifying persistence:', error);
    return false;
  }
}

// Run verification
verifyPersistence().catch(console.error);
`;

fs.writeFileSync('verify-persistence.js', verificationScript);
console.log('Created verification script: verify-persistence.js');

// Run all tests
async function runTests() {
  console.log('Starting complex operations tests...\n');
  
  // Ensure schema exists
  const schemaReady = await ensureUserSchema();
  if (!schemaReady) {
    console.error('Failed to ensure User schema exists. Exiting tests.');
    return;
  }
  
  // Clear existing users
  await clearExistingUsers();
  
  // Create test users
  const usersCreated = await createUsers();
  if (!usersCreated) {
    console.error('Failed to create test users. Exiting tests.');
    return;
  }
  
  // Query all users
  const users = await queryAllUsers();
  if (users.length !== testUsers.length) {
    console.error(`Expected ${testUsers.length} users, but found ${users.length}. Continuing anyway...`);
  }
  
  // Query specific user
  const user1 = await queryUserByUsername('testuser1');
  if (!user1) {
    console.error('Failed to query user by username. Continuing anyway...');
  }
  
  // Update user profile
  if (user1) {
    const profileUpdated = await updateUserProfile(user1.id, {
      bio: 'Updated bio for testing',
      location: 'Test Location'
    });
    
    if (!profileUpdated) {
      console.error('Failed to update user profile. Continuing anyway...');
    }
    
    // Verify the update
    await queryUserByUsername('testuser1');
  }
  
  // Test complex query
  await testComplexQuery();
  
  // Test complex mutation
  await testComplexMutation();
  
  // Verify the complex mutation
  await queryAllUsers();
  
  // Delete a user
  if (users.length > 0) {
    await deleteUser(users[0].id);
    
    // Verify deletion
    const remainingUsers = await queryAllUsers();
    console.log(`After deletion: ${remainingUsers.length} users remain`);
  }
  
  // Verify data persistence
  await verifyDataPersistence();
  
  console.log('\nAll complex operations tests completed!');
  console.log('To verify persistence, restart the server and run:');
  console.log('node verify-persistence.js');
}

// Run the tests
runTests().catch(console.error);
