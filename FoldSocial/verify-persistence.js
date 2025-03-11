
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
    console.log(`Found ${users.length} users after restart`);
    
    if (users.length === 0) {
      console.error('❌ No users found! Data was not persisted.');
      return false;
    }
    
    // Display users
    users.forEach(user => {
      console.log(`- ID: ${user.id}`);
      console.log(`  Username: ${user.username}`);
      console.log(`  Email: ${user.email}`);
      console.log(`  Profile:`, user.profile);
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
