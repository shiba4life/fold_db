# DataFold

## Test UI
./run_http_server

DataFold is a Rust-based distributed data platform providing a core library, a node server, and a command‑line interface to load schemas, run queries, and execute mutations across connected nodes.

For AI, read .cursor_rules

## Repository Structure

- **src/**  
  Core Rust library (`datafold_lib`) defining schema types, query/mutation APIs, and shared components.

- **fold_node/**  
  Rust crate that implements a DataFold node. Exposes a TCP‐based server for schema loading, network discovery, query execution, and mutation handling. The underlying `FoldDB` stores an `Arc<DbOperations>` accessible via `db_ops()` for unified database access.

- **datafold_api_examples/**  
  Python example scripts showing how to interact with a running DataFold node: load schemas, list schemas, run queries, and execute mutations.

- **tests/**  
  Top‑level integration tests spanning multiple crates in the workspace.

- **config/**  
  Default configuration files (e.g. `node_config.json`) for the node server.

- **cline_docs/**  
  Architecture, design decisions, and workflow documentation maintained by the Cline Memory Bank.

## Installation

Build all crates and binaries in release mode:

```bash
cargo build --release --workspace
```

### Binaries

- **datafold_cli**  
  Command‑line tool (`target/release/datafold_cli`) – see [README_CLI.md](README_CLI.md) for usage.
- **datafold_node**  
  Node server executable (`target/release/datafold_node`).

## Configuration

By default, both `datafold_cli` and `datafold_node` look for `config/node_config.json`. Example:

```json
{
  "storage_path": "data/db",
  "default_trust_distance": 1
}
```

Override with `-c, --config <PATH>` flag.
Additional server flags:

- `--port <PORT>` – P2P network port (default `9000` for `datafold_node`, `9001` for `datafold_http_server`)
- `--tcp-port <PORT>` – TCP server port (defaults to `9000`)

## Usage

### Command‑Line Interface

See [README_CLI.md](README_CLI.md) for detailed CLI commands: loading schemas, listing schemas, querying, mutating, and executing operations from JSON.

Once running, use the CLI or HTTP/TCP clients to interact.

### Running the HTTP Server and Web UI

To run DataFold with the web interface, you'll need to start both the Rust backend server and the React frontend development server:

1. Start the Rust HTTP server:
```bash
# Kill any existing server process first
ps aux | grep datafold_http_server | grep -v grep | awk '{print $2}' | xargs kill -9 2>/dev/null
rm -f data/db.lock  # Remove any stale lock files

# Build and start the server
cd fold_node
cargo run --bin datafold_http_server -- --port 9001
```

2. Start the React development server:
```bash
# Navigate to the React project directory
cd fold_node/src/datafold_node/static-react

# Install dependencies (only needed first time)
npm install

# Start the development server
npm run dev
```

The React UI will be available at `http://localhost:5173` and will automatically connect to the Rust backend at `http://localhost:9001`. Any changes to the React code will be hot-reloaded in your browser.

### Loading Sample Data

The UI includes a **Samples** tab with one‑click loading for schemas,
queries, and mutations. Use this tab to quickly populate your node with
the bundled examples or to preview them before loading.

Sample JSON files live under
`fold_node/src/datafold_node/samples/data/`. Add your own files to this
directory to make them available in the Samples tab.

The `TransformSchema` example relies on values from the accompanying
`TransformBase` schema. Populate `value1` and `value2` on `TransformBase`
before running the `TransformSchema.result` transform.

### Network Features

The **Network** tab exposes new peer‑to‑peer features:

- Initialize the networking layer with custom settings
- Start or stop networking services
- View current status and connected nodes
- Discover peers on the local network
- Connect directly to a node by ID

These operations map to the `/api/network/*` endpoints served by
`datafold_http_server`.

## Running Tests

Run all unit and integration tests across the workspace:

```bash
cargo test --workspace
```

If you encounter `cargo: command not found`, install Rust with `rustup` first:

```bash
curl https://sh.rustup.rs -sSf | sh
rustup component add clippy
```

Run only integration tests:

```bash
cargo test --test integration_tests
```

Run tests for a specific crate (e.g., fold_node):

```bash
cargo test --package fold_node
```

### UI Tests

The React UI can include its own test suite. If a `test` script exists in the
package configuration, run the tests with:

1. Install Node.js dependencies:
```bash
cd fold_node/src/datafold_node/static-react
npm install
```

2. Execute the tests:
```bash
npm test
```

Any available tests will verify the React components and UI logic.

## Generating Coverage Reports

DataFold uses [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) to produce code coverage information. Install the tool if it is not already available:

```bash
cargo install cargo-llvm-cov
```

Then run the helper script to create an HTML report for the entire workspace:

```bash
./generate_coverage.sh
```

The report will be generated at `target/llvm-cov/html/index.html`.

## Examples

See **datafold_api_examples/** for Python scripts that demonstrate:

- Creating a user
- Listing schemas
- Unloading schemas
- Querying data
- Executing mutations
- Updating records

## Logging

DataFold includes a comprehensive logging system with feature-specific filtering, multiple output formats, and runtime configuration management.

### Quick Start

```rust
use fold_node::logging::LoggingSystem;
use fold_node::{log_transform_info, log_network_debug, log_schema_error};

// Initialize logging
LoggingSystem::init_default().await?;

// Use feature-specific logging
log_transform_info!("Transform completed successfully");
log_network_debug!("Peer connection established");
log_schema_error!("Schema validation failed");
```

### Configuration

Configure logging via [`config/logging.toml`](config/logging.toml) or environment variables:

```bash
# Set feature-specific log levels
export DATAFOLD_LOG_FEATURE_TRANSFORM=DEBUG
export DATAFOLD_LOG_FEATURE_NETWORK=INFO

# Configure outputs
export DATAFOLD_LOG_FILE_ENABLED=true
export DATAFOLD_LOG_FILE_PATH=/var/log/datafold.log
```

### Runtime Management

Adjust log levels via HTTP API without restarting:

```bash
# Update feature log level
curl -X POST http://localhost:9001/api/logs/features \
  -H "Content-Type: application/json" \
  -d '{"feature": "transform", "level": "TRACE"}'

# Stream logs in real-time
curl http://localhost:9001/api/logs/stream

# Reload configuration
curl -X POST http://localhost:9001/api/logs/reload
```

### Available Features

- **transform** - Data transformation and DSL execution
- **network** - P2P networking and peer discovery
- **schema** - Schema validation and management
- **database** - Database operations and storage
- **query** - Query execution and optimization
- **mutation** - Data mutations and updates
- **permissions** - Access control and authorization
- **http_server** - HTTP API and web interface
- **tcp_server** - TCP protocol and connections
- **ingestion** - Data ingestion and processing

### Migration

Migrate existing code to use feature-specific logging:

```bash
# Analyze existing log statements
python scripts/migrate_logging.py fold_node/src/ --report

# Get detailed suggestions
python scripts/migrate_logging.py fold_node/src/ --detailed

# Generate migration guide
python scripts/migrate_logging.py fold_node/src/ --output migration.md
```

For comprehensive documentation, see [`docs/LOGGING_GUIDE.md`](docs/LOGGING_GUIDE.md).

## Documentation

In‑depth technical and architectural notes are in the **cline_docs/** directory:

- `productContext.md` – why DataFold exists and its use cases
- `systemPatterns.md` – architecture and design patterns
- `techContext.md` – technologies and development setup
- `progress.md` – current progress and roadmap
- [Unified_Architecture.md](docs/Unified_Architecture.md) – canonical overview of the final unified architecture

## Pre‑commit Hooks

This repo uses custom Git hooks under `.git/hooks/` to enforce formatting and testing. Install with:

```bash
./install-hooks.sh
```

See [HOOKS_README.md](HOOKS_README.md) for details.

## Contributing

1. Fork the repository.  
2. Implement features or fix bugs on a feature branch.  
3. Run tests and ensure all pass.  
4. Follow existing code style and update documentation if needed.  
5. Submit a pull request.

---

&copy; 2025 DataFold Contributors
