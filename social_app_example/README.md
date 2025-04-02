# DataFold Social App Example

This crate contains examples of social applications built on top of the DataFold network. These examples demonstrate how to use the DataFold SDK to create, query, and mutate data in a decentralized social network.

## Available Examples

1. `social_app_tcp` - An example that uses a TCP connection to connect to a real DataFold node.

## Running the Examples

### Prerequisites

- Rust and Cargo installed
- DataFold project cloned and built

### Running the TCP Social App Example

To run the TCP social app example, which connects to a real DataFold node using TCP:

1. Start a DataFold node with the TCP server enabled:

```bash
cd ..
cargo run --bin datafold_node -- --port 9876 --tcp-port 9000
```

2. In a separate terminal, run the TCP social app example:

```bash
cargo run --bin social_app_tcp
```

The DataFold node includes a built-in TCP server that listens on the specified TCP port (default: 9000). This allows the social app to communicate with the node using TCP sockets, making it easier to develop and test applications on different platforms.

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

1. **TCP Connection**: Connecting to a DataFold node using TCP sockets.
2. **Schema Creation**: Creating schemas for users, posts, and comments.
3. **Data Mutation**: Creating, updating, and deleting data.
4. **Data Querying**: Querying data with filters and field selection.
5. **Remote Node Discovery**: Discovering and connecting to remote nodes.
6. **Remote Data Access**: Querying and mutating data on remote nodes.

## Implementation Details

The social app example uses the DataFold SDK to interact with the DataFold network. The key components are:

1. **DataFoldClient**: The main client for interacting with the DataFold network.
2. **QueryBuilder**: A builder for constructing and executing queries.
3. **MutationBuilder**: A builder for constructing and executing mutations.
4. **NodeConnection**: A connection to a DataFold node.
5. **AppRequest**: A request from an app to the DataFold node.

## Example Workflow

1. Create a client for the app with a TCP connection to the node.
2. Discover available schemas.
3. Create schemas if they don't exist.
4. Create users, posts, and comments.
5. Query users, posts, and comments.
6. Discover remote nodes.
7. Query data on remote nodes.
