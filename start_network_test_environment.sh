#!/bin/bash

# Script to start the DataFold Network Test Environment
# This script starts all three nodes and the network visualizer

echo "Starting DataFold Network Test Environment..."

# Function to check if a port is in use
port_in_use() {
  lsof -i:$1 > /dev/null 2>&1
  return $?
}

# Check if any of the required ports are already in use
for port in 3000 3001 3002 8000; do
  if port_in_use $port; then
    echo "Error: Port $port is already in use."
    if [ $port -eq 8000 ]; then
      echo "The network visualizer server is already running."
      echo "Please stop it before running this script."
    else
      echo "A node is already running on port $port."
      echo "Please stop any existing node processes."
    fi
    exit 1
  fi
done

# Start the three nodes in the background
echo "Starting Node 1 (port 3000)..."
NODE_CONFIG=config/node1_config.json cargo run --bin datafold_node -- --port 3000 > logs/node1.log 2>&1 &
NODE1_PID=$!

echo "Starting Node 2 (port 3001)..."
NODE_CONFIG=config/node2_config.json cargo run --bin datafold_node -- --port 3001 > logs/node2.log 2>&1 &
NODE2_PID=$!

echo "Starting Node 3 (port 3002)..."
NODE_CONFIG=config/node3_config.json cargo run --bin datafold_node -- --port 3002 > logs/node3.log 2>&1 &
NODE3_PID=$!

# Create a PID file to track the node processes
echo "$NODE1_PID $NODE2_PID $NODE3_PID" > .node_pids

echo "All nodes started in the background."
echo "Node 1 PID: $NODE1_PID (logs in logs/node1.log)"
echo "Node 2 PID: $NODE2_PID (logs in logs/node2.log)"
echo "Node 3 PID: $NODE3_PID (logs in logs/node3.log)"

# Wait a moment for the nodes to start
echo "Waiting for nodes to start..."
sleep 5

# Initialize the network on each node
echo "Initializing network on Node 1..."
curl -s -X POST http://localhost:3000/api/init_network \
  -H "Content-Type: application/json" \
  -d '{
    "enable_discovery": true,
    "discovery_port": 8090,
    "listen_port": 8091,
    "max_connections": 50,
    "connection_timeout_secs": 10,
    "announcement_interval_secs": 60
  }' > /dev/null

echo "Initializing network on Node 2..."
curl -s -X POST http://localhost:3001/api/init_network \
  -H "Content-Type: application/json" \
  -d '{
    "enable_discovery": true,
    "discovery_port": 8091,
    "listen_port": 8092,
    "max_connections": 50,
    "connection_timeout_secs": 10,
    "announcement_interval_secs": 60
  }' > /dev/null

echo "Initializing network on Node 3..."
curl -s -X POST http://localhost:3002/api/init_network \
  -H "Content-Type: application/json" \
  -d '{
    "enable_discovery": true,
    "discovery_port": 8092,
    "listen_port": 8093,
    "max_connections": 50,
    "connection_timeout_secs": 10,
    "announcement_interval_secs": 60
  }' > /dev/null

echo "All nodes initialized."
sleep 2

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "Error: Node.js is not installed. Please install Node.js to use the network visualizer."
    exit 1
fi

# Check if npm is installed
if ! command -v npm &> /dev/null; then
    echo "Error: npm is not installed. Please install npm to use the network visualizer."
    exit 1
fi

# Navigate to the network visualizer directory
cd network_visualizer

# Install dependencies if node_modules doesn't exist
if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    npm install
fi

# Start the server
echo "Starting network visualizer server..."
npm start &
VISUALIZER_PID=$!

# Add the visualizer PID to the PID file
cd ..
echo "$VISUALIZER_PID" >> .node_pids

# Wait a moment for the server to start
sleep 3

# Open the visualizer in the default browser
if [ "$(uname)" == "Darwin" ]; then
    # macOS
    open http://localhost:8000
elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
    # Linux
    xdg-open http://localhost:8000
elif [ "$(expr substr $(uname -s) 1 10)" == "MINGW32_NT" ] || [ "$(expr substr $(uname -s) 1 10)" == "MINGW64_NT" ]; then
    # Windows
    start http://localhost:8000
fi

echo "Network visualizer is running at http://localhost:8000"
echo ""
echo "=== NETWORK TEST ENVIRONMENT INSTRUCTIONS ==="
echo "1. The nodes have been automatically initialized"
echo "2. Use the 'Refresh' button to see the current status of each node"
echo "3. Use the 'Disable'/'Enable' buttons to toggle nodes for testing"
echo "4. Press Ctrl+C in this terminal to stop all nodes and the visualizer"
echo "================================================"

# Function to clean up processes on exit
cleanup() {
    echo "Stopping all nodes and the visualizer..."
    if [ -f .node_pids ]; then
        for pid in $(cat .node_pids); do
            kill $pid 2>/dev/null
        done
        rm .node_pids
    fi
    echo "All processes stopped."
    exit 0
}

# Set up trap to catch Ctrl+C and other termination signals
trap cleanup SIGINT SIGTERM

# Keep the script running until Ctrl+C
echo "Press Ctrl+C to stop all processes."
while true; do
    sleep 1
done
