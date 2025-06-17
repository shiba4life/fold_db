# CLI Interface

The DataFold CLI provides a command-line interface for all database operations including schema management, data queries, network operations, and transforms.

## Installation

The CLI tool is built as part of the main build process:

```bash
cargo build --release --workspace
# Binary available at target/release/datafold_cli
```

## Global Options

```bash
datafold_cli [OPTIONS] <COMMAND>

OPTIONS:
    -c, --config <PATH>    Configuration file path [default: config/node_config.json]
    -h, --help            Print help information
    -V, --version         Print version information
```

## Schema Commands

### load-schema
Load a schema definition into the node.

```bash
datafold_cli load-schema <SCHEMA_FILE>

ARGUMENTS:
    <SCHEMA_FILE>    Path to schema JSON file

EXAMPLES:
    datafold_cli load-schema schemas/user_profile.json
    datafold_cli load-schema -c custom_config.json schemas/analytics.json
```

### list-schemas
List all loaded schemas in the node.

```bash
datafold_cli list-schemas [OPTIONS]

OPTIONS:
    --format <FORMAT>    Output format [default: table] [possible values: table, json, yaml]

EXAMPLES:
    datafold_cli list-schemas
    datafold_cli list-schemas --format json
```

### get-schema
Get detailed information about a specific schema.

```bash
datafold_cli get-schema <SCHEMA_NAME>

ARGUMENTS:
    <SCHEMA_NAME>    Name of the schema to retrieve

EXAMPLES:
    datafold_cli get-schema UserProfile
    datafold_cli get-schema EventAnalytics --format json
```

### unload-schema
Unload a schema from the node.

```bash
datafold_cli unload-schema <SCHEMA_NAME>

ARGUMENTS:
    <SCHEMA_NAME>    Name of the schema to unload

EXAMPLES:
    datafold_cli unload-schema UserProfile
```

## Data Commands

### query
Execute a query against a schema.

```bash
datafold_cli query [OPTIONS] --schema <SCHEMA>

OPTIONS:
    -s, --schema <SCHEMA>           Schema name
    -f, --fields <FIELDS>           Comma-separated list of fields
    -w, --where <FILTER>            Filter condition (JSON)
    -l, --limit <LIMIT>             Maximum number of results
    -o, --output <FORMAT>           Output format [default: table]

EXAMPLES:
    datafold_cli query --schema UserProfile --fields username,email
    datafold_cli query --schema UserProfile --fields username --where '{"username":"alice"}'
    datafold_cli query --schema EventAnalytics --fields event_name,metrics_by_timeframe --where '{"field":"metrics_by_timeframe","range_filter":{"KeyPrefix":"2024-01-01"}}'
```

### mutate
Execute a mutation (create, update, delete) against a schema.

```bash
datafold_cli mutate [OPTIONS] --schema <SCHEMA> --operation <OPERATION>

OPTIONS:
    -s, --schema <SCHEMA>           Schema name
    -o, --operation <OPERATION>     Operation type [possible values: create, update, delete]
    -d, --data <DATA>               Data payload (JSON)
    -w, --where <FILTER>            Filter for update/delete operations (JSON)

EXAMPLES:
    # Create
    datafold_cli mutate --schema UserProfile --operation create --data '{"username":"bob","email":"bob@example.com"}'
    
    # Update
    datafold_cli mutate --schema UserProfile --operation update --where '{"username":"bob"}' --data '{"email":"newemail@example.com"}'
    
    # Delete
    datafold_cli mutate --schema UserProfile --operation delete --where '{"username":"bob"}'
```

## Network Commands

### discover-nodes
Discover peers on the network.

```bash
datafold_cli discover-nodes [OPTIONS]

OPTIONS:
    --timeout <SECONDS>    Discovery timeout [default: 10]

EXAMPLES:
    datafold_cli discover-nodes
    datafold_cli discover-nodes --timeout 30
```

### connect-node
Connect to a specific peer node.

```bash
datafold_cli connect-node <NODE_ID> <ADDRESS>

ARGUMENTS:
    <NODE_ID>     Peer node identifier
    <ADDRESS>     Peer address (multiaddr format)

EXAMPLES:
    datafold_cli connect-node 12D3KooWGK8YLjL... /ip4/192.168.1.100/tcp/9000
```

## Transform Commands

### register-transform
Register a new transform function.

```bash
datafold_cli register-transform <TRANSFORM_FILE>

ARGUMENTS:
    <TRANSFORM_FILE>    Path to transform definition JSON file

EXAMPLES:
    datafold_cli register-transform transforms/user_status.json
```

### list-transforms
List all registered transforms.

```bash
datafold_cli list-transforms [OPTIONS]

OPTIONS:
    --schema <SCHEMA>    Filter by schema name

EXAMPLES:
    datafold_cli list-transforms
    datafold_cli list-transforms --schema UserProfile
```

## Related Documentation

- [Schema Management API](./schema-management-api.md) - HTTP API equivalents
- [Data Operations API](./data-operations-api.md) - HTTP API equivalents  
- [CLI Authentication Guide](../guides/cli-authentication.md) - Authentication setup
- [CLI Integration Examples](../guides/integration/quickstart/5-minute-integration.md) - Quick start guide

## Return to Index

[← Back to API Reference Index](./index.md)