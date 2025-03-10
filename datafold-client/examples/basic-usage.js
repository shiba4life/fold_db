const { DataFoldClient } = require('datafold-client');

// Create a client instance
const client = new DataFoldClient({
  baseUrl: 'http://localhost:3000',
});

async function main() {
  try {
    // List available schemas
    console.log('Listing schemas...');
    const schemas = await client.listSchemas();
    console.log('Available schemas:', schemas);

    // Create a user profile
    console.log('\nCreating a user profile...');
    const createResult = await client.create('UserProfile', {
      username: 'johndoe',
      email: 'john.doe@example.com',
      full_name: 'John Doe',
      bio: 'Software developer with 10 years of experience',
      age: 35,
      location: 'San Francisco, CA'
    });
    console.log('Create result:', createResult);

    // Query for the user
    console.log('\nQuerying for the user...');
    const user = await client.findOne('UserProfile', 
      ['username', 'email', 'full_name', 'bio', 'age', 'location'], 
      { username: 'johndoe' }
    );
    console.log('User found:', user);

    // Update the user
    console.log('\nUpdating the user...');
    const updateResult = await client.update(
      'UserProfile',
      { username: 'johndoe' },
      { bio: 'Senior software engineer with expertise in distributed systems' }
    );
    console.log('Update result:', updateResult);

    // Query for the updated user
    console.log('\nQuerying for the updated user...');
    const updatedUser = await client.findOne('UserProfile', 
      ['username', 'email', 'full_name', 'bio', 'age', 'location'], 
      { username: 'johndoe' }
    );
    console.log('Updated user:', updatedUser);

    // Delete the user
    console.log('\nDeleting the user...');
    const deleteResult = await client.delete('UserProfile', { username: 'johndoe' });
    console.log('Delete result:', deleteResult);

    // Check if the user was deleted
    console.log('\nChecking if the user was deleted...');
    const deletedUser = await client.findOne('UserProfile', ['username'], { username: 'johndoe' });
    console.log('Deleted user:', deletedUser);

  } catch (error) {
    console.error('Error:', error);
  }
}

// Run the example
main().catch(console.error);
