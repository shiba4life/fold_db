const { DataFoldClient } = require('../dist');

// Create a client instance
const client = new DataFoldClient({
  baseUrl: 'http://localhost:8080',
  // Optional authentication
  auth: {
    public_key: 'your-public-key',
    private_key: 'your-private-key'
  }
});

// Example schema
const exampleSchema = {
  name: 'ExampleSchema',
  fields: {
    title: {
      permission_policy: {
        read_policy: {
          NoRequirement: null
        },
        write_policy: {
          Distance: 0
        }
      },
      payment_config: {
        base_multiplier: 1.0,
        trust_distance_scaling: {
          None: null
        }
      },
      field_mappers: {}
    },
    content: {
      permission_policy: {
        read_policy: {
          NoRequirement: null
        },
        write_policy: {
          Distance: 0
        }
      },
      payment_config: {
        base_multiplier: 1.0,
        trust_distance_scaling: {
          None: null
        }
      },
      field_mappers: {}
    }
  },
  payment_config: {
    base_multiplier: 1.0,
    min_payment_threshold: 0
  }
};

async function main() {
  try {
    console.log('DataFold Schema Loading Example');
    
    // List existing schemas
    console.log('\nListing existing schemas...');
    const schemas = await client.listSchemas();
    console.log('Existing schemas:', schemas);
    
    // Load schema from JSON
    console.log('\nLoading schema from JSON...');
    const loadResult = await client.loadSchemaFromJson(exampleSchema);
    console.log('Load result:', loadResult);
    
    // Load schema from file (if you have a schema file)
    // console.log('\nLoading schema from file...');
    // const fileLoadResult = await client.loadSchemaFromFile('path/to/schema.json');
    // console.log('File load result:', fileLoadResult);
    
    // List schemas again to verify the schema was loaded
    console.log('\nListing schemas after loading...');
    const updatedSchemas = await client.listSchemas();
    console.log('Updated schemas:', updatedSchemas);
    
    // Create some data in the schema
    console.log('\nCreating data in the schema...');
    const createResult = await client.create('ExampleSchema', {
      title: 'Example Title',
      content: 'This is some example content.'
    });
    console.log('Create result:', createResult);
    
    // Query the data
    console.log('\nQuerying data from the schema...');
    const queryResult = await client.find('ExampleSchema', ['title', 'content']);
    console.log('Query result:', queryResult);
    
  } catch (error) {
    console.error('Error:', error);
  }
}

main();
