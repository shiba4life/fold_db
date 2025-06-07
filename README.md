# DataFold

[![Crates.io](https://img.shields.io/crates/v/datafold.svg)](https://crates.io/crates/datafold)
[![Documentation](https://docs.rs/datafold/badge.svg)](https://docs.rs/datafold)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/yourusername/datafold)

A Rust-based distributed data platform with schema-based storage, AI-powered ingestion, and real-time data processing capabilities. DataFold provides a complete solution for distributed data management with automatic schema generation, field mapping, and extensible ingestion pipelines.

## ✨ Features

- **🤖 AI-Powered Data Ingestion** - Automatic schema creation and field mapping using AI [Initial prototype]
- **🔄 Real-Time Processing** - Event-driven architecture with automatic transform execution [working]
- **🌐 Distributed Architecture** - P2P networking with automatic peer discovery [untested]
- **📊 Flexible Schema System** - Dynamic schema management with validation [working]
- **🔐 Permission Management** - Fine-grained access control and trust-based permissions [working]
- **⚡ High Performance** - Rust-based core with optimized storage and query execution [maybe]
- **🔌 Extensible Ingestion** - Plugin system for social media and external data sources [not yet begun]

## 🚀 Quick Start

### Installation

Add DataFold to your `Cargo.toml`:

```toml
[dependencies]
datafold = "0.1.0"
```

Or install the CLI tools:

```bash
cargo install datafold
```

This provides three binaries:
- `datafold_cli` - Command-line interface
- `datafold_http_server` - HTTP server with web UI
- `datafold_node` - P2P node server

### Basic Usage

```rust
use datafold::{DataFoldNode, IngestionCore, Schema};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize a DataFold node
    let node = DataFoldNode::new_with_defaults().await?;
    
    // Create an ingestion pipeline
    let config = datafold::IngestionConfig::from_env_allow_empty();
    let ingestion = IngestionCore::new(config)?;
    
    // Process JSON data with automatic schema generation
    let data = json!({
        "name": "John Doe",
        "email": "john@example.com",
        "age": 30,
        "preferences": {
            "theme": "dark",
            "notifications": true
        }
    });
    
    let response = ingestion.process_json_ingestion(
        datafold::IngestionRequest { data }
    ).await?;
    
    println!("Ingestion result: {:?}", response);
    Ok(())
}
```

### Running the HTTP Server

```bash
# Start the HTTP server with web UI
datafold_http_server --port 9001
```

Then visit `http://localhost:9001` for the web interface.

## 📖 Core Concepts

### Schemas
DataFold uses dynamic schemas that define data structure and operations:

```rust
use datafold::{Schema, Operation};

// Load a schema
let schema_json = std::fs::read_to_string("my_schema.json")?;
let schema: Schema = serde_json::from_str(&schema_json)?;

// Execute operations
let operation = Operation::Query(query_data);
let result = node.execute_operation(operation).await?;
```

### AI-Powered Ingestion
Automatically analyze and ingest data from any source:

```rust
use datafold::{IngestionConfig, IngestionCore};

// Configure with OpenRouter API
let config = IngestionConfig {
    openrouter_api_key: Some("your-api-key".to_string()),
    openrouter_model: "anthropic/claude-3.5-sonnet".to_string(),
    ..Default::default()
};

let ingestion = IngestionCore::new(config)?;

// Process any JSON data
let result = ingestion.process_json_ingestion(request).await?;
```

### Distributed Networking
Connect nodes in a P2P network:

```rust
use datafold::{NetworkConfig, NetworkCore};

let network_config = NetworkConfig::default();
let network = NetworkCore::new(network_config).await?;

// Start networking
network.start().await?;

// Discover peers
let peers = network.discover_peers().await?;
```

## 🔌 Extensible Ingestion

DataFold supports ingesting data from various sources with the new adapter-based architecture:

- **Social Media APIs** - Twitter, Facebook, Reddit, TikTok
- **Real-time Streams** - WebSockets, Server-Sent Events
- **File Uploads** - JSON, CSV, JSONL
- **Webhooks** - Real-time event processing
- **Custom Adapters** - Extensible plugin system

See [`SOCIAL_MEDIA_INGESTION_PROPOSAL.md`](SOCIAL_MEDIA_INGESTION_PROPOSAL.md) for the complete ingestion architecture.

## 🛠️ Development Setup

### Prerequisites

- Rust 1.70+ with Cargo
- Node.js 16+ (for web UI development)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/datafold.git
cd datafold

# Build all components
cargo build --release --workspace

# Run tests
cargo test --workspace
```

### Running the Web UI

For development with hot-reload:

```bash
# Start the Rust backend
cargo run --bin datafold_http_server -- --port 9001

# In another terminal, start the React frontend
cd src/datafold_node/static-react
npm install
npm run dev
```

The UI will be available at `http://localhost:5173`.

## 📊 Examples

### Loading Sample Data

```bash
# Use the CLI to load a schema
datafold_cli load-schema examples/user_schema.json

# Query data
datafold_cli query examples/user_query.json

# Execute mutations
datafold_cli mutate examples/user_mutation.json
```

### Python Integration

See [`datafold_api_examples/`](datafold_api_examples/) for Python scripts demonstrating:

- Schema management
- Data querying
- Mutations and updates
- User management

## 🔧 Configuration

DataFold uses JSON configuration files. Default config:

```json
{
  "storage_path": "data/db",
  "default_trust_distance": 1,
  "network": {
    "port": 9000,
    "enable_mdns": true
  },
  "ingestion": {
    "enabled": true,
    "openrouter_model": "anthropic/claude-3.5-sonnet"
  }
}
```

Environment variables:
- `OPENROUTER_API_KEY` - API key for AI-powered ingestion
- `DATAFOLD_CONFIG` - Path to configuration file
- `DATAFOLD_LOG_LEVEL` - Logging level (trace, debug, info, warn, error)

## 📚 Documentation

- **[API Documentation](https://docs.rs/datafold)** - Complete API reference
- **[CLI Guide](README_CLI.md)** - Command-line interface usage
- **[Ingestion Guide](INGESTION_README.md)** - AI-powered data ingestion
- **[Architecture](docs/Unified_Architecture.md)** - System design and patterns

## 🤝 Contributing

We welcome contributions! Please see our contributing guidelines:

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Run `cargo test --workspace`
5. Submit a pull request

## 📄 License

This project is licensed under either of:

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.

## 🌟 Community

- **Issues** - Report bugs and request features on [GitHub Issues](https://github.com/yourusername/datafold/issues)
- **Discussions** - Join discussions on [GitHub Discussions](https://github.com/yourusername/datafold/discussions)

---

**DataFold** - Distributed data platform for the modern world 🚀
