#!/bin/bash

# Script to start the DataFold Network Visualizer

echo "Starting DataFold Network Visualizer..."

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
echo "Starting server..."
npm start

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
