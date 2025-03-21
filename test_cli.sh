#!/bin/bash

# Build the CLI
echo "Building DataFold CLI..."
cargo build

# Create data directory if it doesn't exist
mkdir -p data/db

# Test the CLI
echo -e "\n1. Testing list-schemas (should be empty initially):"
./target/debug/datafold_cli list-schemas

echo -e "\n2. Loading a schema:"
./target/debug/datafold_cli load-schema src/datafold_node/examples/schema1.json

echo -e "\n3. Listing schemas (should show UserProfile):"
./target/debug/datafold_cli list-schemas

echo -e "\n4. Creating a user:"
./target/debug/datafold_cli mutate --schema UserProfile --mutation-type create --data '{"username": "johndoe", "email": "john@example.com"}'

echo -e "\n5. Querying username field:"
./target/debug/datafold_cli query --schema UserProfile --fields username

echo -e "\n6. Querying multiple fields:"
./target/debug/datafold_cli query --schema UserProfile --fields username,email --output pretty

echo -e "\n7. Executing a query from file:"
./target/debug/datafold_cli execute src/datafold_node/examples/query1.json

echo -e "\n8. Executing a mutation from file:"
./target/debug/datafold_cli execute src/datafold_node/examples/mutation1.json

echo -e "\nAll tests completed!"
