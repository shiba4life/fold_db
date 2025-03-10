const { DataFoldClient } = require('datafold-client');

// Create a client instance using Unix socket
const client = new DataFoldClient({
  baseUrl: 'http://localhost', // This is ignored when using socketPath
  socketPath: '/tmp/datafold.sock', // Path to the Unix socket
});

async function main() {
  try {
    console.log('Using Unix socket connection to DataFold node...');
    
    // List available schemas
    console.log('\nListing schemas...');
    const schemas = await client.listSchemas();
    console.log('Available schemas:', schemas);
    
    // Get network status
    console.log('\nGetting network status...');
    const networkStatus = await client.getNetworkStatus();
    console.log('Network status:', networkStatus);
    
    // List connected nodes
    console.log('\nListing connected nodes...');
    const nodes = await client.listNodes();
    console.log('Connected nodes:', nodes);
    
    // Execute a simple query if schemas exist
    if (schemas.length > 0) {
      const schema = schemas[0];
      console.log(`\nExecuting a query on schema: ${schema}`);
      
      try {
        const queryResults = await client.query({
          type: 'query',
          schema,
          fields: ['*'], // Request all fields
          filter: null
        });
        
        console.log('Query results:', queryResults);
      } catch (error) {
        console.log(`Query failed: ${error.message}`);
      }
    }
    
  } catch (error) {
    console.error('Error:', error);
    
    if (error.code === 'ECONNREFUSED') {
      console.error('\nConnection refused. Make sure the DataFold node is running and the Unix socket exists at the specified path.');
      console.error('You can check if the socket exists with: ls -la /tmp/datafold.sock');
    }
  }
}

// Run the example
main().catch(console.error);
