# DataFold CLI

A command-line interface for interacting with the DataFold node, allowing you to load schemas, run queries, and execute mutations.

## Installation

The CLI is built as part of the main project. To build it, run:

```bash
cargo build --release
```

The binary will be available at `target/release/datafold_cli`.

## Configuration

The CLI requires a configuration file for the DataFold node. By default, it looks for a file at `config/node_config.json`. You can specify a different configuration file using the `-c` or `--config` option.

Example configuration file:

```json
{
  "storage_path": "data/db",
  "default_trust_distance": 1
}
```

## Usage

```
datafold_cli [OPTIONS] <COMMAND>
```

### Options

- `-c, --config <CONFIG>`: Path to the node configuration file (default: `config/node_config.json`)
- `-h, --help`: Print help
- `-V, --version`: Print version

### Commands

#### Load Schema

Load a schema from a JSON file:

```bash
datafold_cli load-schema <PATH>
```

Example:

```bash
datafold_cli load-schema src/datafold_node/examples/schema1.json
```

#### List Schemas

List all loaded schemas:

```bash
datafold_cli list-schemas
```

#### Query

Execute a query operation:

```bash
datafold_cli query --schema <SCHEMA> --fields <FIELDS> [--filter <FILTER>] [--output <OUTPUT>]
```

Options:
- `-s, --schema <SCHEMA>`: Schema name to query
- `-f, --fields <FIELDS>`: Fields to retrieve (comma-separated)
- `-i, --filter <FILTER>`: Optional filter in JSON format
- `-o, --output <OUTPUT>`: Output format (json or pretty, default: pretty)

Example:

```bash
datafold_cli query --schema UserProfile --fields username,email
```

With filter:

```bash
datafold_cli query --schema UserProfile --fields username,email --filter '{"username": "johndoe"}'
```

#### Mutate

Execute a mutation operation:

```bash
datafold_cli mutate --schema <SCHEMA> --mutation-type <MUTATION_TYPE> --data <DATA>
```

Options:
- `-s, --schema <SCHEMA>`: Schema name to mutate
- `-m, --mutation-type <MUTATION_TYPE>`: Mutation type (create, update, delete)
- `-d, --data <DATA>`: Data in JSON format

Example:

```bash
datafold_cli mutate --schema UserProfile --mutation-type create --data '{"username": "johndoe", "email": "john@example.com"}'
```

#### Execute

Load and execute an operation from a JSON file:

```bash
datafold_cli execute <PATH>
```

Example:

```bash
datafold_cli execute src/datafold_node/examples/query1.json
```

## Example Operations

### Query Operation

```json
{
  "type": "query",
  "schema": "UserProfile",
  "fields": ["username"],
  "filter": null
}
```

### Mutation Operation

```json
{
  "type": "mutation",
  "schema": "UserProfile",
  "mutation_type": "create",
  "data": {
    "username": "johndoe"
  }
}
```

## Error Handling

The CLI will display error messages if operations fail. Common errors include:
- Invalid configuration file
- Schema not found
- Invalid operation format
- Permission denied for certain operations
