#!/bin/bash

# Run the FoldSocial test script
echo "Running FoldSocial test script..."
echo "Make sure your DataFold node is running at http://localhost:8080"
echo ""

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
  echo "Installing dependencies..."
  npm install
fi

# Run the test script
echo "Running test script..."
node test/test-posts.js
