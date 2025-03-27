#!/bin/bash

# This script demonstrates running multiple DataFold nodes with network configuration
# Each node will run in its own terminal window

# Create temporary directories for each node
mkdir -p test_data/node1/db
mkdir -p test_data/node2/db
mkdir -p test_data/node3/db

# Create config files for each node
cat > test_data/node1/config.json << EOF
{
  "storage_path": "test_data/node1/db",
  "default_trust_distance": 1
}
EOF

cat > test_data/node2/config.json << EOF
{
  "storage_path": "test_data/node2/db",
  "default_trust_distance": 1
}
EOF

cat > test_data/node3/config.json << EOF
{
  "storage_path": "test_data/node3/db",
  "default_trust_distance": 1
}
EOF

# Run each node in a separate terminal window
# Each node uses a different port for the network service

# Node 1 - Port 9001
NODE_CONFIG=test_data/node1/config.json \
  cargo run --bin datafold_node -- --port 9001 &

# Wait a moment before starting the next node
sleep 2

# Node 2 - Port 9002
NODE_CONFIG=test_data/node2/config.json \
  cargo run --bin datafold_node -- --port 9002 &

# Wait a moment before starting the next node
sleep 2

# Node 3 - Port 9003
NODE_CONFIG=test_data/node3/config.json \
  cargo run --bin datafold_node -- --port 9003 &

echo "All nodes started. Press Ctrl+C to stop all nodes."
wait
