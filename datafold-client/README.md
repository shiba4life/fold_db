# DataFold Client

A Node.js client for interfacing with a DataFold node.

## Installation

```bash
npm install datafold-client
```

## Usage

### Basic Setup

```typescript
import { DataFoldClient } from 'datafold-client';

// Create a client instance
const client = new DataFoldClient({
  baseUrl: 'http://localhost:3000',
  // Optional: Unix socket path for direct socket communication
  // socketPath: '/path/to/socket',
  // Optional: Authentication configuration
  // auth: {
  //   public_key: 'your-public-key',
  //   private_key: 'your-private-key',
  // },
  // Optional: Request timeout in milliseconds
  // timeout: 5000,
});
```

### Working with Schemas

```typescript
// List all schemas
const schemas = await client.listSchemas();
console.log('Available schemas:', schemas);

// Create a new schema
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
    }
  },
  payment_config: {
    base_multiplier: 1.5,
    min_payment_threshold: 500
  }
};

const schemaCreated = await client.createSchema(userProfileSchema);
console.log('Schema created:', schemaCreated);

// Delete a schema
const schemaDeleted = await client.deleteSchema('UserProfile');
console.log('Schema deleted:', schemaDeleted);
```

### Querying Data

```typescript
// Find all records
const allUsers = await client.find('UserProfile', ['username', 'email']);
console.log('All users:', allUsers);

// Find records with a filter
const filteredUsers = await client.find('UserProfile', ['username', 'email'], {
  age: { gt: 30 }
});
console.log('Users over 30:', filteredUsers);

// Find a single record
const user = await client.findOne('UserProfile', ['username', 'email', 'bio'], {
  username: 'johndoe'
});
console.log('User:', user);

// Execute a custom query
const queryResults = await client.query({
  type: 'query',
  schema: 'UserProfile',
  fields: ['username', 'email', 'bio'],
  filter: { location: 'San Francisco, CA' }
});
console.log('Query results:', queryResults.results);
console.log('Result count:', queryResults.count);
```

### Mutating Data

```typescript
// Create a record
const createResult = await client.create('UserProfile', {
  username: 'johndoe',
  email: 'john.doe@example.com',
  full_name: 'John Doe',
  bio: 'Software developer with 10 years of experience',
  age: 35,
  location: 'San Francisco, CA'
});
console.log('Create result:', createResult);

// Update a record
const updateResult = await client.update(
  'UserProfile',
  { username: 'johndoe' },
  { bio: 'Senior software engineer with expertise in distributed systems' }
);
console.log('Update result:', updateResult);

// Delete a record
const deleteResult = await client.delete('UserProfile', { username: 'johndoe' });
console.log('Delete result:', deleteResult);

// Execute a custom mutation
const mutationResult = await client.mutate({
  type: 'mutation',
  schema: 'UserProfile',
  operation: 'create',
  data: {
    username: 'janedoe',
    email: 'jane.doe@example.com'
  }
});
console.log('Mutation result:', mutationResult);
```

### Network Operations

```typescript
// Get network status
const networkStatus = await client.getNetworkStatus();
console.log('Network status:', networkStatus);

// Initialize network
const networkInitialized = await client.initNetwork({
  listen_addr: '0.0.0.0',
  port: 8000
});
console.log('Network initialized:', networkInitialized);

// Start network
const networkStarted = await client.startNetwork();
console.log('Network started:', networkStarted);

// Discover nodes
const nodesDiscovered = await client.discoverNodes();
console.log('Nodes discovered:', nodesDiscovered);

// Connect to a node
const nodeConnected = await client.connectToNode('192.168.1.100', 8000);
console.log('Node connected:', nodeConnected);

// List nodes
const nodes = await client.listNodes();
console.log('Connected nodes:', nodes);

// Stop network
const networkStopped = await client.stopNetwork();
console.log('Network stopped:', networkStopped);
```

### Authentication

```typescript
// Register a public key
const keyRegistered = await client.registerKey(
  'your-public-key',
  'your-signature'
);
console.log('Key registered:', keyRegistered);
```

## Error Handling

The client throws structured errors that include a code, message, and optional details:

```typescript
try {
  await client.find('NonExistentSchema', ['field1']);
} catch (error) {
  console.error('Error code:', error.code);
  console.error('Error message:', error.message);
  console.error('Error details:', error.details);
}
```

## API Reference

### Client Configuration

```typescript
interface DataFoldClientConfig {
  baseUrl: string;
  auth?: {
    public_key: string;
    private_key: string;
    trust_level?: number;
  };
  timeout?: number;
  socketPath?: string;
}
```

### Schema Methods

- `listSchemas(): Promise<string[]>`
- `createSchema(schema: SchemaConfig): Promise<boolean>`
- `deleteSchema(name: string): Promise<boolean>`

### Query Methods

- `query<T = any>(query: Operation): Promise<QueryResponse>`
- `find<T = any>(schema: string, fields: string[], filter?: Record<string, any> | null): Promise<T[]>`
- `findOne<T = any>(schema: string, fields: string[], filter: Record<string, any>): Promise<T | null>`

### Mutation Methods

- `mutate(mutation: Operation): Promise<MutationResponse>`
- `create(schema: string, data: Record<string, any>): Promise<MutationResponse>`
- `update(schema: string, filter: Record<string, any>, data: Record<string, any>): Promise<MutationResponse>`
- `delete(schema: string, filter: Record<string, any>): Promise<MutationResponse>`

### Network Methods

- `getNetworkStatus(): Promise<NetworkStatus>`
- `initNetwork(config: NetworkConfig): Promise<boolean>`
- `startNetwork(): Promise<boolean>`
- `stopNetwork(): Promise<boolean>`
- `discoverNodes(): Promise<boolean>`
- `connectToNode(address: string, port: number): Promise<boolean>`
- `listNodes(): Promise<NodeInfo[]>`

### Authentication Methods

- `registerKey(publicKey: string, signature: string): Promise<boolean>`

## License

MIT
