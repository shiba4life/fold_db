#!/bin/bash
set -e

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "Docker is not running. Please start Docker and try again."
    exit 1
fi

# Check if port 9000 is already in use
if nc -z localhost 9000 > /dev/null 2>&1; then
    echo "Port 9000 is already in use. Please stop any existing DataFold Node or change the port."
    exit 1
fi

echo "Starting DataFold Node with integrated FoldClient..."
# Start the DataFold node with integrated FoldClient in the background
cargo run -p fold_node --bin datafold_node -- --port 9000 --tcp-port 9000 &
NODE_PID=$!

# Wait for the node to start
echo "Waiting for DataFold Node to start..."
sleep 8

# Check if the node is running
if ! nc -z localhost 9000 > /dev/null 2>&1; then
    echo "Failed to start DataFold Node. Check the logs for errors."
    kill $NODE_PID 2>/dev/null || true
    exit 1
fi

echo "DataFold Node is now running with integrated FoldClient"
echo "This means you no longer need to run a separate FoldClient process"
echo "You can now run your social app directly against the DataFold Node"
echo ""
echo "To test the functionality, you can run the social app in a separate terminal:"
echo "cd social_app && ./run_social_app.sh"
echo ""
echo "Press Ctrl+C to stop the DataFold Node"

# Wait for user to press Ctrl+C
wait $NODE_PID || {
    # Handle the case where the node process exits unexpectedly
    echo "DataFold Node exited unexpectedly. Check the logs for errors."
    exit 1
}
