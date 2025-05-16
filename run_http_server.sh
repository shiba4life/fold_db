#!/bin/bash

# Navigate to the fold_node directory
cd fold_node

# Build the project
echo "Building the project..."
cargo build

# Run the HTTP server
echo "Starting the HTTP server on port 9001..."
cargo run --bin datafold_http_server -- --port 9001