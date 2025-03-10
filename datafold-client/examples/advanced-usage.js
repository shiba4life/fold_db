const { DataFoldClient } = require('datafold-client');

// Create a client instance with authentication
const client = new DataFoldClient({
  baseUrl: 'http://localhost:3000',
  auth: {
    public_key: 'your-public-key',
    private_key: 'your-private-key',
  },
  timeout: 15000,
});

async function createSchema() {
  console.log('Creating UserProfile schema...');
  
  const userProfileSchema = {
    name: 'UserProfile',
    fields: {
      username: {
        permission_policy: {
          read_policy: { NoRequirement: null },
          write_policy: { Distance: 0 }
        },
        payment_config: {
          base_multiplier: 1.0,
          trust_distance_scaling: { None: null }
        },
        field_mappers: {}
      },
      email: {
        permission_policy: {
          read_policy: { Distance: 1 },
          write_policy: { Distance: 0 }
        },
        payment_config: {
          base_multiplier: 2.0,
          trust_distance_scaling: {
            Linear: {
              slope: 0.5,
              intercept: 1.0,
              min_factor: 1.0
            }
          },
          min_payment: 1000
        },
        field_mappers: {}
      },
      full_name: {
        permission_policy: {
          read_policy: { Distance: 1 },
          write_policy: { Distance: 0 }
        },
        payment_config: {
          base_multiplier: 1.5,
          trust_distance_scaling: { None: null }
        },
        field_mappers: {}
      },
      bio: {
        permission_policy: {
          read_policy: { NoRequirement: null },
          write_policy: { Distance: 0 }
        },
        payment_config: {
          base_multiplier: 1.0,
          trust_distance_scaling: { None: null }
        },
        field_mappers: {}
      },
      age: {
        permission_policy: {
          read_policy: { Distance: 2 },
          write_policy: { Distance: 0 }
        },
        payment_config: {
          base_multiplier: 1.2,
          trust_distance_scaling: { None: null }
        },
        field_mappers: {}
      },
      location: {
        permission_policy: {
          read_policy: { Distance: 1 },
          write_policy: { Distance: 0 }
        },
        payment_config: {
          base_multiplier: 1.3,
          trust_distance_scaling: { None: null }
        },
        field_mappers: {}
      }
    },
    payment_config: {
      base_multiplier: 1.5,
      min_payment_threshold: 500
    }
  };
  
  const result = await client.createSchema(userProfileSchema);
  console.log('Schema created:', result);
  return result;
}

async function createMappedSchema() {
  console.log('\nCreating UserProfile2 schema with field mappings...');
  
  const userProfile2Schema = {
    name: 'UserProfile2',
    fields: {
      user_name: {
        permission_policy: {
          read_policy: { NoRequirement: null },
          write_policy: { Distance: 0 }
        },
        payment_config: {
          base_multiplier: 1.0,
          trust_distance_scaling: { None: null }
        },
        field_mappers: {
          "UserProfile": {
            field: "username",
            mapping_type: "Direct"
          }
        }
      },
      contact_email: {
        permission_policy: {
          read_policy: { NoRequirement: null }, // More permissive than original
          write_policy: { Distance: 0 }
        },
        payment_config: {
          base_multiplier: 1.0, // Lower cost than original
          trust_distance_scaling: { None: null }
        },
        field_mappers: {
          "UserProfile": {
            field: "email",
            mapping_type: "Direct"
          }
        }
      },
      display_name: {
        permission_policy: {
          read_policy: { NoRequirement: null }, // More permissive than original
          write_policy: { Distance: 0 }
        },
        payment_config: {
          base_multiplier: 1.0,
          trust_distance_scaling: { None: null }
        },
        field_mappers: {
          "UserProfile": {
            field: "full_name",
            mapping_type: "Direct"
          }
        }
      },
      profile_description: {
        permission_policy: {
          read_policy: { NoRequirement: null },
          write_policy: { Distance: 0 }
        },
        payment_config: {
          base_multiplier: 1.0,
          trust_distance_scaling: { None: null }
        },
        field_mappers: {
          "UserProfile": {
            field: "bio",
            mapping_type: "Direct"
          }
        }
      },
      user_location: {
        permission_policy: {
          read_policy: { NoRequirement: null }, // More permissive than original
          write_policy: { Distance: 0 }
        },
        payment_config: {
          base_multiplier: 1.0,
          trust_distance_scaling: { None: null }
        },
        field_mappers: {
          "UserProfile": {
            field: "location",
            mapping_type: "Direct"
          }
        }
      }
    },
    payment_config: {
      base_multiplier: 1.0, // Lower cost than original
      min_payment_threshold: 100
    }
  };
  
  const result = await client.createSchema(userProfile2Schema);
  console.log('Mapped schema created:', result);
  return result;
}

async function createUsers() {
  console.log('\nCreating sample users...');
  
  const users = [
    {
      username: 'johndoe',
      email: 'john.doe@example.com',
      full_name: 'John Doe',
      bio: 'Software developer with 10 years of experience',
      age: 35,
      location: 'San Francisco, CA'
    },
    {
      username: 'janedoe',
      email: 'jane.doe@example.com',
      full_name: 'Jane Doe',
      bio: 'UX Designer passionate about user-centered design',
      age: 28,
      location: 'New York, NY'
    },
    {
      username: 'bobsmith',
      email: 'bob.smith@example.com',
      full_name: 'Bob Smith',
      bio: 'Data scientist specializing in machine learning',
      age: 42,
      location: 'Seattle, WA'
    }
  ];
  
  const results = [];
  for (const user of users) {
    console.log(`Creating user: ${user.username}`);
    const result = await client.create('UserProfile', user);
    results.push(result);
  }
  
  console.log('Users created:', results);
  return results;
}

async function queryOriginalSchema() {
  console.log('\nQuerying original UserProfile schema...');
  
  // Query all users
  const allUsers = await client.find('UserProfile', ['username', 'email', 'bio']);
  console.log('All users:', allUsers);
  
  // Query users over 30
  const olderUsers = await client.find('UserProfile', 
    ['username', 'full_name', 'age', 'location'], 
    { age: { gt: 30 } }
  );
  console.log('Users over 30:', olderUsers);
  
  return { allUsers, olderUsers };
}

async function queryMappedSchema() {
  console.log('\nQuerying mapped UserProfile2 schema...');
  
  // Query all mapped users
  const allMappedUsers = await client.find('UserProfile2', 
    ['user_name', 'contact_email', 'profile_description']
  );
  console.log('All mapped users:', allMappedUsers);
  
  // Query specific mapped user
  const mappedUser = await client.findOne('UserProfile2',
    ['user_name', 'display_name', 'user_location'],
    { user_name: 'johndoe' }
  );
  console.log('Mapped user (johndoe):', mappedUser);
  
  return { allMappedUsers, mappedUser };
}

async function networkOperations() {
  console.log('\nPerforming network operations...');
  
  // Get network status
  const status = await client.getNetworkStatus();
  console.log('Network status:', status);
  
  if (!status.running) {
    // Initialize network
    console.log('\nInitializing network...');
    const initResult = await client.initNetwork({
      listen_addr: '0.0.0.0',
      port: 8000
    });
    console.log('Network initialized:', initResult);
    
    // Start network
    console.log('\nStarting network...');
    const startResult = await client.startNetwork();
    console.log('Network started:', startResult);
  }
  
  // Discover nodes
  console.log('\nDiscovering nodes...');
  const discoverResult = await client.discoverNodes();
  console.log('Nodes discovered:', discoverResult);
  
  // List nodes
  console.log('\nListing nodes...');
  const nodes = await client.listNodes();
  console.log('Connected nodes:', nodes);
  
  return { status, nodes };
}

async function cleanup() {
  console.log('\nCleaning up...');
  
  // Delete users
  const users = ['johndoe', 'janedoe', 'bobsmith'];
  for (const username of users) {
    console.log(`Deleting user: ${username}`);
    await client.delete('UserProfile', { username });
  }
  
  // Delete schemas
  console.log('Deleting schemas...');
  await client.deleteSchema('UserProfile2'); // Delete mapped schema first
  await client.deleteSchema('UserProfile');
  
  console.log('Cleanup complete');
}

async function main() {
  try {
    // List initial schemas
    console.log('Listing initial schemas...');
    const initialSchemas = await client.listSchemas();
    console.log('Initial schemas:', initialSchemas);
    
    // Create schemas and data
    await createSchema();
    await createMappedSchema();
    await createUsers();
    
    // Query data
    await queryOriginalSchema();
    await queryMappedSchema();
    
    // Network operations
    await networkOperations();
    
    // Cleanup
    await cleanup();
    
  } catch (error) {
    console.error('Error:', error);
  }
}

// Run the example
main().catch(console.error);
