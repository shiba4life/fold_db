#!/bin/bash
set -e

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "Docker is not running. Please start Docker and try again."
    exit 1
fi

# Check if the DataFold node is running
if ! nc -z localhost 9000 > /dev/null 2>&1; then
    echo "DataFold node is not running. Please start it with:"
    echo "cargo run -p fold_node --bin datafold_node -- --port 9000"
    exit 1
fi

# Build the social app
echo "Building the Social App..."
cd "$(dirname "$0")"
cargo build

# Launch the app with direct connection to the DataFold node
echo "Launching the Social App..."
cargo run --bin social_app

echo "Done!"
