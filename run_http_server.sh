#!/bin/bash

# Function to kill existing datafold processes and clean up locks
cleanup_locks() {
    echo "Checking for existing datafold processes..."
    
    # Kill any existing datafold processes
    pkill -f datafold_http_server 2>/dev/null || true
    pkill -f "cargo run.*datafold_http_server" 2>/dev/null || true
    
    # Wait a moment for processes to terminate
    sleep 2
    
    # Force kill if still running
    pkill -9 -f datafold_http_server 2>/dev/null || true
    pkill -9 -f "cargo run.*datafold_http_server" 2>/dev/null || true
    
    echo "Cleaned up existing processes."
}

# Navigate to the fold_node directory
cd fold_node

# Clean up any existing locks and processes
cleanup_locks

# Build the project
echo "Building the project..."
cargo build

if [ $? -ne 0 ]; then
    echo "Build failed. Exiting."
    exit 1
fi

# Run the HTTP server in the background
echo "Starting the HTTP server on port 9001 in the background..."
nohup cargo run --bin datafold_http_server -- --port 9001 > ../server.log 2>&1 &

# Get the process ID
SERVER_PID=$!

# Wait a moment to check if the server started successfully
sleep 3

# Check if the process is still running
if kill -0 $SERVER_PID 2>/dev/null; then
    echo "HTTP server started successfully with PID: $SERVER_PID"
    echo "Server logs are being written to: ../server.log"
    echo "To stop the server, run: kill $SERVER_PID"
    echo "To view logs, run: tail -f ../server.log"
else
    echo "Failed to start HTTP server. Check ../server.log for details."
    exit 1
fi