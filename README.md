# DataFold

DataFold is a Rust-based distributed data platform providing a core library, a node server, and a command‑line interface to load schemas, run queries, and execute mutations across connected nodes.

## Repository Structure

- **src/**  
  Core Rust library (`datafold_lib`) defining schema types, query/mutation APIs, and shared components.

- **fold_node/**  
  Rust crate that implements a DataFold node. Exposes a TCP‐based server for schema loading, network discovery, query execution, and mutation handling.

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

## Usage

### Command‑Line Interface

See [README_CLI.md](README_CLI.md) for detailed CLI commands: loading schemas, listing schemas, querying, mutating, and executing operations from JSON.

### Running a DataFold Node

```bash
target/release/datafold_node \
  --config config/node_config.json
```

Once running, use the CLI or HTTP/TCP clients to interact.

### Running the HTTP Server and Web UI

Build the HTTP server binary and start it on your machine:

```bash
cargo build --release --bin datafold_http_server
NODE_CONFIG=config/node_config.json \
  target/release/datafold_http_server --port 9001
```

The server hosts a web UI at `http://localhost:9001`. Open this URL in your browser to
interact with DataFold without using the CLI.

### Loading Sample Data

The UI includes a **Samples** tab with one‑click loading for schemas,
queries, and mutations. Use this tab to quickly populate your node with
the bundled examples or to preview them before loading.

Sample JSON files live under
`fold_node/src/datafold_node/samples/data/`. Add your own files to this
directory to make them available in the Samples tab.

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

Run only integration tests:

```bash
cargo test --test integration_tests
```

Run tests for a specific crate (e.g., fold_node):

```bash
cargo test --package fold_node
```

### UI Tests

The web UI components are tested using Jest. To run the UI tests:

1. Install Node.js dependencies:
```bash
cd fold_node/src/datafold_node/static
npm install
```

2. Run the tests:
```bash
npm test
```

This will run all Jest tests for UI components, including network tab functionality, form handling, and status display tests.

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
- Querying data
- Executing mutations
- Updating records

## Documentation

In‑depth technical and architectural notes are in the **cline_docs/** directory:

- `productContext.md` – why DataFold exists and its use cases  
- `systemPatterns.md` – architecture and design patterns  
- `techContext.md` – technologies and development setup  
- `progress.md` – current progress and roadmap  

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
