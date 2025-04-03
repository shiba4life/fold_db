#!/bin/bash

# This script runs the social_app_two_nodes example

# Make sure the test directories exist
mkdir -p test_data/two_node_example/node1/db
mkdir -p test_data/two_node_example/node2/db

# Run the example
cargo run --example social_app_two_nodes
