#!/bin/bash

# Run the schema loading test script
echo "Running FoldSocial schema loading test..."
echo "Make sure your DataFold node is running at http://localhost:8080"
echo ""

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
  echo "Installing dependencies..."
  npm install
fi

# Run the test script
echo "Running schema loading test..."
node test/test-schema-loading.js
