# DataFold Social App Example

This directory contains examples of social applications built on top of the DataFold network. These examples demonstrate how to use the DataFold SDK to create, query, and mutate data in a decentralized social network.

## Available Examples

1. `social_app_example.rs` - A simulated social app example that demonstrates the SDK's capabilities without requiring a running node.
2. `social_app_example_real.rs` - A social app example that connects to a real DataFold node, but uses simulated data.
3. `social_app_real.rs` - A fully functional social app example that connects to a real DataFold node and uses real data.
4. `social_app_mock.rs` - A simplified example that uses the mock implementation for testing.
5. `social_app_simple.rs` - A minimal example that demonstrates the basic usage of the SDK.
6. `social_app_tcp.rs` - An example that uses a TCP connection to connect to a real DataFold node.

## Running the Examples

### Prerequisites

- Rust and Cargo installed
- DataFold project cloned and built

### Running the Social App Example

To run the simulated social app example:

```bash
cargo run --example social_app_example
```

### Running the Real Social App Example

To run the real social app example, you need to have a DataFold node running:

1. Start a DataFold node:

```bash
cargo run --bin datafold_node -- --port 9000
```

2. In a separate terminal, run the social app example:

```bash
cargo run --example social_app_real
```

### Running the TCP Social App Example

To run the TCP social app example, which connects to a real DataFold node using TCP:

1. Start a DataFold node with the TCP server enabled:

```bash
cargo run --bin datafold_node -- --port 9876 --tcp-port 9000
```

2. In a separate terminal, run the TCP social app example:

```bash
cargo run --example social_app_tcp
```

The DataFold node now includes a built-in TCP server that listens on the specified TCP port (default: 9000). This allows the social app to communicate with the node using TCP sockets, making it easier to develop and test applications on different platforms.

## Social App Architecture

The social app example demonstrates a simple social network with the following schemas:

### User Schema

```json
{
  "name": "user",
  "fields": [
    { "name": "id", "field_type": "string", "required": true },
    { "name": "username", "field_type": "string", "required": true },
    { "name": "full_name", "field_type": "string", "required": false },
    { "name": "bio", "field_type": "string", "required": false },
    { "name": "created_at", "field_type": "string", "required": true }
  ]
}
```

### Post Schema

```json
{
  "name": "post",
  "fields": [
    { "name": "id", "field_type": "string", "required": true },
    { "name": "title", "field_type": "string", "required": true },
    { "name": "content", "field_type": "string", "required": true },
    { "name": "author_id", "field_type": "string", "required": true },
    { "name": "created_at", "field_type": "string", "required": true }
  ]
}
```

### Comment Schema

```json
{
  "name": "comment",
  "fields": [
    { "name": "id", "field_type": "string", "required": true },
    { "name": "content", "field_type": "string", "required": true },
    { "name": "author_id", "field_type": "string", "required": true },
    { "name": "post_id", "field_type": "string", "required": true },
    { "name": "created_at", "field_type": "string", "required": true }
  ]
}
```

## Key Features Demonstrated

1. **Schema Creation**: Creating schemas for users, posts, and comments.
2. **Data Mutation**: Creating, updating, and deleting data.
3. **Data Querying**: Querying data with filters and field selection.
4. **Remote Node Discovery**: Discovering and connecting to remote nodes.
5. **Remote Data Access**: Querying and mutating data on remote nodes.

## Implementation Details

The social app example uses the DataFold SDK to interact with the DataFold network. The key components are:

1. **DataFoldClient**: The main client for interacting with the DataFold network.
2. **QueryBuilder**: A builder for constructing and executing queries.
3. **MutationBuilder**: A builder for constructing and executing mutations.
4. **NodeConnection**: A connection to a DataFold node.
5. **AppRequest**: A request from an app to the DataFold node.

## Example Workflow

1. Create a client for the app.
2. Discover available schemas.
3. Create schemas if they don't exist.
4. Create users, posts, and comments.
5. Query users, posts, and comments.
6. Discover remote nodes.
7. Query data on remote nodes.

## Next Steps

- Add authentication and authorization.
- Add a web interface.
- Add support for media attachments.
- Add support for likes and shares.
- Add support for following users.
- Add support for notifications.
