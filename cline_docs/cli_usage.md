# FoldDB CLI Usage Guide

The FoldDB CLI provides a command-line interface for interacting with the FoldDB database system. It allows you to initialize nodes, manage schemas, execute queries and mutations, and perform network operations.

> **Note:** The CLI works with the centralized examples in `src/datafold_node/examples/`. These examples are the single source of truth for schemas, mutations, and queries across the FoldDB system.

## Installation

The CLI is built as part of the FoldDB project. To build it, run:

```bash
cargo build --bin fold_cli
```

## Basic Usage

```bash
fold_cli [OPTIONS] <COMMAND>
```

### Global Options

- `-c, --config <FILE>`: Specify a custom config file path
- `-h, --help`: Print help information

## Commands

### Initialize a Node

```bash
fold_cli init --storage-path <PATH>
```

This command initializes a new FoldDB node with the specified storage path. It creates a default configuration and saves it to `config/node_config.json`.

### Schema Management

#### Load a Schema

```bash
fold_cli load-schema --file <PATH>
```

Loads a schema from a JSON file into the database.

#### List Schemas

```bash
fold_cli list-schemas
```

Lists all schemas currently loaded in the database.

### Data Operations

#### Execute a Query

```bash
# Using command line arguments
fold_cli query --schema <NAME> --fields <FIELDS> [--filter <FILTER>]

# Using a JSON file
fold_cli query --schema <NAME> -j <JSON_FILE>
```

Executes a query against the specified schema. Fields can be specified as comma-separated values or in a JSON file.

Example JSON query file:
```json
{
  "fields": ["username", "email", "age"],
  "filter": null
}
```

Examples:
```bash
# Using command line arguments
fold_cli query --schema UserProfile --fields username,email,age

# Using a JSON file
fold_cli query --schema UserProfile -j ./queries/user_query.json

# Using an example from the centralized examples directory
# Note: The examples file contains an array of queries, so you need to extract a single query first
fold_cli query --schema UserProfile --fields "username, email, bio" --filter '{"username": "johndoe"}'
```

> **Note on Centralized Examples:** The `user_profile_queries.json` file in the examples directory contains an array of queries. To use these with the CLI, you need to extract a single query into a separate file or use the command line arguments directly.

#### Execute a Mutation

```bash
# Using command line arguments
fold_cli mutate --schema <NAME> --mutation-type <TYPE> --data <JSON>

# Using a JSON file
fold_cli mutate --schema <NAME> --mutation-type <TYPE> -j <JSON_FILE>
```

Executes a mutation on the specified schema. Mutation type can be `create`, `update`, or `delete`. Data can be provided as a JSON string or in a JSON file.

Example JSON mutation file:
```json
{
  "username": "alice",
  "email": "alice@example.com",
  "full_name": "Alice Smith",
  "age": 28
}
```

Examples:
```bash
# Using command line arguments
fold_cli mutate --schema UserProfile --mutation-type create --data '{"username": "alice", "email": "alice@example.com"}'

# Using a JSON file
fold_cli mutate --schema UserProfile --mutation-type create -j ./mutations/create_user.json

# Using data similar to the centralized examples
fold_cli mutate --schema UserProfile --mutation-type create --data '{"username": "johndoe", "email": "john.doe@example.com", "full_name": "John Doe", "bio": "Software developer", "age": 35, "location": "San Francisco, CA"}'
```

> **Note on Centralized Examples:** The `user_profile_mutations.json` file in the examples directory contains an array of mutations. To use these with the CLI, you need to extract a single mutation into a separate file or use the command line arguments directly.

### Network Operations

#### Initialize Network

```bash
fold_cli network init --listen-addr <ADDRESS>
```

Initializes the network layer with the specified listen address.

Example:
```bash
fold_cli network init --listen-addr 127.0.0.1:9000
```

#### Start Network

```bash
fold_cli network start
```

Starts the network layer.

#### Stop Network

```bash
fold_cli network stop
```

Stops the network layer.

#### Discover Nodes

```bash
fold_cli network discover
```

Discovers nodes on the network.

#### Connect to a Node

```bash
fold_cli network connect --node-id <ID>
```

Connects to a node with the specified ID.

#### List Connected Nodes

```bash
fold_cli network list-connected
```

Lists all connected nodes.

#### List Known Nodes

```bash
fold_cli network list-known
```

Lists all known nodes.

#### Query a Remote Node

```bash
fold_cli network query-node --node-id <ID> --schema <NAME> --fields <FIELDS>
```

Queries a remote node for data from the specified schema.

#### List Schemas on a Remote Node

```bash
fold_cli network list-node-schemas --node-id <ID>
```

Lists schemas available on a remote node.

## Examples

### Initialize a Node and Load a Schema

```bash
# Initialize a node
fold_cli init --storage-path ./data

# Load a schema from the centralized examples
fold_cli load-schema --file src/datafold_node/examples/user_profile_schema.json

# List loaded schemas
fold_cli list-schemas
```

### Execute Queries and Mutations

```bash
# Create a new user
fold_cli mutate --schema UserProfile --mutation-type create --data '{"username": "bob", "email": "bob@example.com", "age": 30}'

# Query user data
fold_cli query --schema UserProfile --fields username,email,age

# Query with a filter
fold_cli query --schema UserProfile --fields "username, email, bio" --filter '{"username": "bob"}'

# Query with a comparison operator
fold_cli query --schema UserProfile --fields "username, full_name, age" --filter '{"age": {"gt": 30}}'
```

### Network Operations

```bash
# Initialize the network
fold_cli network init --listen-addr 127.0.0.1:9000

# Start the network
fold_cli network start

# Discover nodes
fold_cli network discover

# Connect to a node
fold_cli network connect --node-id <node_id_from_discovery>

# Query a remote node
fold_cli network query-node --node-id <node_id> --schema user_profile --fields username,email
```
