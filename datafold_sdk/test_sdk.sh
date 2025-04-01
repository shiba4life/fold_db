#!/bin/bash

# Test script for the DataFold SDK
# This script runs all the tests for the SDK and reports the results

set -e  # Exit on error

echo "===== Testing DataFold SDK ====="
echo

# Check if we're in the right directory
if [ ! -d "src" ] || [ ! -f "Cargo.toml" ]; then
    echo "Error: This script must be run from the datafold_sdk directory"
    exit 1
fi

# Build the SDK
echo "Building the SDK..."
cargo build
echo "✓ SDK built successfully"
echo

# Run the unit tests
echo "Running unit tests..."
cargo test --lib -- --nocapture
echo "✓ Unit tests completed"
echo

# Run the integration tests
echo "Running integration tests..."
cargo test --test integration_tests -- --nocapture
echo "✓ Integration tests completed"
echo

# Run the SDK tests
echo "Running SDK tests..."
cargo test --test sdk_tests -- --nocapture
echo "✓ SDK tests completed"
echo

# Run the examples to make sure they compile
echo "Testing examples..."
cargo build --examples
echo "✓ Examples built successfully"
echo

# Run the basic usage example if requested
if [ "$1" == "--run-examples" ]; then
    echo "Running basic_usage example..."
    cargo run --example basic_usage
    echo "✓ Basic usage example completed"
    echo
    
    echo "Running container_management example..."
    cargo run --example container_management
    echo "✓ Container management example completed"
    echo
fi

echo "===== All tests passed! ====="
